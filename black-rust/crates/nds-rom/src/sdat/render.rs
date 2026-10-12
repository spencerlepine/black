//! Offline DS sequencer: SSEQ + SBNK + SWAR -> stereo PCM (A9).
//!
//! Model (SDK conventions, reconstructed; see the deviations list in the A9 report):
//! * the sequencer runs once per "update" (192 Hz); each update adds `tempo` (BPM) to an
//!   accumulator and every 240 accumulated units is one tick, so 48 ticks = one quarter note;
//! * envelopes are stepped once per update on a dB-like amplitude (`-92544 ..= 0`, 128 units per
//!   1/10 dB), attack is geometric, decay/release linear (rates from the usual SDK tables);
//! * volume, velocity, expression, master and sustain are combined in the dB domain with
//!   `cB = 400 * log10(v / 127)`;
//! * a "loop" is the first backward `Jump` (or infinite `LoopEnd`) of a track; the loop with the
//!   latest end among all tracks is reported, found by a dry run without audio.

use super::sbnk::{Instrument, Region, Sbnk};
use super::sseq::{Ctrl, Event, Modifier, Sseq, VarOp};
use super::swar::Swar;
use super::{Kind, Pcm, Sdat};
use crate::error::{Error, Result};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

/// Output sample rate (the DS mixer runs at ~32.728 kHz).
pub const OUT_RATE: u32 = 32728;
const UPDATE_HZ: u64 = 192;
const AMPL_MIN: i32 = -92544;
const MAX_VOICES: usize = 32;
const MASTER_GAIN: f32 = 0.8;

/// Options for [`Renderer::render`].
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Number of passes through the loop (1 = intro plus one loop body). Ignored without a loop.
    pub loops: u32,
    /// Hard cap on song length in seconds (before the tail).
    pub max_seconds: f64,
    /// Release tail rendered after the song ends, in seconds.
    pub tail_seconds: f64,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            loops: 2,
            max_seconds: 600.0,
            tail_seconds: 2.0,
        }
    }
}

/// Result of a render.
#[derive(Debug, Clone)]
pub struct Rendered {
    /// Stereo PCM at [`OUT_RATE`]; `looping` holds the loop points in samples when detected.
    pub pcm: Pcm,
    /// True when the sequence loops (otherwise it ended by itself or hit the cap).
    pub loops: bool,
}

impl Rendered {
    /// WAV file with a `smpl` chunk carrying the loop points (if any).
    pub fn to_wav(&self) -> Vec<u8> {
        let mut w = self.pcm.to_wav();
        if let Some((s, e)) = self.pcm.looping {
            let sr = self.pcm.sample_rate;
            let mut c = Vec::new();
            for v in [
                0u32,
                0,
                1_000_000_000 / sr,
                60,
                0,
                0,
                0,
                1,
                0,
                0,
                0,
                s as u32,
                e as u32,
                0,
                0,
            ] {
                c.extend_from_slice(&v.to_le_bytes());
            }
            w.extend_from_slice(b"smpl");
            w.extend_from_slice(&(c.len() as u32).to_le_bytes());
            w.extend_from_slice(&c);
            let riff = (w.len() - 8) as u32;
            w[4..8].copy_from_slice(&riff.to_le_bytes());
        }
        w
    }
}

/// One decoded sample.
struct Wave {
    data: Vec<i16>,
    rate: f64,
    looping: Option<(usize, usize)>,
}

struct BankSet {
    sbnk: Sbnk,
    waves: [Option<Rc<Vec<Wave>>>; 4],
}

/// Renders sequences of one SDAT, caching decoded banks and wave archives.
pub struct Renderer<'a> {
    sdat: &'a Sdat<'a>,
    waves: HashMap<u16, Rc<Vec<Wave>>>,
    banks: HashMap<u16, Rc<BankSet>>,
}

fn err(d: impl Into<String>) -> Error {
    super::bad("SDAT render", d)
}

impl<'a> Renderer<'a> {
    pub fn new(sdat: &'a Sdat<'a>) -> Self {
        Renderer {
            sdat,
            waves: HashMap::new(),
            banks: HashMap::new(),
        }
    }

    fn wave_arc(&mut self, id: u16) -> Result<Rc<Vec<Wave>>> {
        if let Some(w) = self.waves.get(&id) {
            return Ok(w.clone());
        }
        let swar = Swar::parse(self.sdat.entry_file(Kind::WaveArc, id as usize)?)?;
        let v: Vec<Wave> = swar
            .waves
            .iter()
            .map(|w| match w.decode() {
                Ok(p) => Wave {
                    rate: f64::from(p.sample_rate),
                    looping: p.looping,
                    data: p.channels.into_iter().next().unwrap_or_default(),
                },
                Err(_) => Wave {
                    data: Vec::new(),
                    rate: 1.0,
                    looping: None,
                },
            })
            .collect();
        let rc = Rc::new(v);
        self.waves.insert(id, rc.clone());
        Ok(rc)
    }

    fn bank(&mut self, id: u16) -> Result<Rc<BankSet>> {
        if let Some(b) = self.banks.get(&id) {
            return Ok(b.clone());
        }
        let sbnk = Sbnk::parse(self.sdat.entry_file(Kind::Bank, id as usize)?)?;
        let arcs = self
            .sdat
            .entry(Kind::Bank, id as usize)
            .ok_or_else(|| err(format!("no bank {id}")))?
            .info
            .wave_arcs;
        let mut waves: [Option<Rc<Vec<Wave>>>; 4] = [None, None, None, None];
        for (i, a) in arcs.iter().enumerate() {
            if *a != 0xFFFF {
                waves[i] = self.wave_arc(*a).ok();
            }
        }
        let rc = Rc::new(BankSet { sbnk, waves });
        self.banks.insert(id, rc.clone());
        Ok(rc)
    }

    /// Render SEQ `seq_id` to stereo PCM.
    pub fn render(&mut self, seq_id: usize, opts: &RenderOptions) -> Result<Rendered> {
        let entry = self
            .sdat
            .entry(Kind::Seq, seq_id)
            .ok_or_else(|| err(format!("no SEQ {seq_id}")))?;
        let (bank_id, seq_vol) = (entry.info.bank, entry.info.volume);
        let sseq = Sseq::parse(self.sdat.entry_file(Kind::Seq, seq_id)?)?;
        let bank = self.bank(bank_id)?;
        let events = &sseq.sequence.events;
        let start = sseq.sequence.tracks.first().copied().unwrap_or(0);
        let cap = (opts.max_seconds * UPDATE_HZ as f64) as u64;

        // Pass 1: dry run (no voices) to find the loop.
        let mut dry = Engine::new(events, start, bank.clone(), seq_vol, false);
        let mut chosen: Option<(u64, u64)> = None;
        let mut ended_at = cap;
        loop {
            dry.update();
            if dry.all_ended() {
                ended_at = dry.upd;
                break;
            }
            if dry.upd >= cap {
                break;
            }
            if dry.all_looped_or_ended() {
                chosen = dry.best_loop();
                break;
            }
        }
        let loops = opts.loops.max(1) as u64;
        let total = match chosen {
            Some((s, e)) => (e + (loops - 1) * (e - s)).min(cap),
            None => ended_at.min(cap),
        };

        // Pass 2: mix.
        let mut eng = Engine::new(events, start, bank, seq_vol, true);
        let (mut l, mut r) = (Vec::<i16>::new(), Vec::<i16>::new());
        let tail_updates = (opts.tail_seconds * UPDATE_HZ as f64) as u64;
        let mut stop_at: Option<u64> = None;
        let (mut bl, mut br) = (Vec::<f32>::new(), Vec::<f32>::new());
        let mut k = 0u64;
        loop {
            if stop_at.is_none() && k >= total {
                eng.halt();
                stop_at = Some(k);
            }
            if let Some(s) = stop_at {
                if eng.voices.is_empty() || k >= s + tail_updates {
                    break;
                }
            }
            if stop_at.is_none() {
                eng.update();
            } else {
                eng.update_voices_only();
            }
            let n = (((k + 1) * u64::from(OUT_RATE)) / UPDATE_HZ
                - (k * u64::from(OUT_RATE)) / UPDATE_HZ) as usize;
            bl.clear();
            br.clear();
            bl.resize(n, 0.0);
            br.resize(n, 0.0);
            eng.mix(&mut bl, &mut br);
            for i in 0..n {
                l.push(((bl[i] * MASTER_GAIN).clamp(-1.0, 1.0) * 32767.0) as i16);
                r.push(((br[i] * MASTER_GAIN).clamp(-1.0, 1.0) * 32767.0) as i16);
            }
            k += 1;
        }
        let to_s = |u: u64| (u * u64::from(OUT_RATE) / UPDATE_HZ) as usize;
        let looping = chosen.map(|(s, e)| (to_s(s), to_s(e)));
        Ok(Rendered {
            pcm: Pcm {
                sample_rate: OUT_RATE,
                channels: vec![l, r],
                looping,
            },
            loops: chosen.is_some(),
        })
    }
}

/// Loop points `(start, end)` in samples of a sequence, via the dry run only (no audio).
pub fn detect_loop(sdat: &Sdat, seq_id: usize, max_seconds: f64) -> Result<Option<(usize, usize)>> {
    let entry = sdat
        .entry(Kind::Seq, seq_id)
        .ok_or_else(|| err(format!("no SEQ {seq_id}")))?;
    let sseq = Sseq::parse(sdat.entry_file(Kind::Seq, seq_id)?)?;
    let bank = Rc::new(BankSet {
        sbnk: Sbnk {
            instruments: Vec::new(),
        },
        waves: [None, None, None, None],
    });
    let start = sseq.sequence.tracks.first().copied().unwrap_or(0);
    let mut dry = Engine::new(&sseq.sequence.events, start, bank, entry.info.volume, false);
    let cap = (max_seconds * UPDATE_HZ as f64) as u64;
    while dry.upd < cap && !dry.all_ended() && !dry.all_looped_or_ended() {
        dry.update();
    }
    let to_s = |u: u64| (u * u64::from(OUT_RATE) / UPDATE_HZ) as usize;
    Ok(dry.best_loop().map(|(s, e)| (to_s(s), to_s(e))))
}

// ---------------------------------------------------------------------------------------------

fn conv_vol(v: i32) -> i32 {
    let v = v.clamp(0, 127);
    if v == 0 {
        return -723;
    }
    ((400.0 * (f64::from(v) / 127.0).log10()) as i32).max(-723)
}

fn attack_rate(a: u8) -> i32 {
    const T: [i32; 19] = [
        0x00, 0x01, 0x05, 0x0E, 0x1A, 0x26, 0x33, 0x3F, 0x49, 0x54, 0x5C, 0x64, 0x6D, 0x74, 0x7B,
        0x7F, 0x84, 0x89, 0x8F,
    ];
    let a = a.min(127);
    if a >= 109 {
        T[(127 - a) as usize]
    } else {
        255 - i32::from(a)
    }
}

fn decay_rate(d: u8) -> i32 {
    let d = i32::from(d.min(127));
    match d {
        127 => 0xFFFF,
        126 => 0x3C00,
        _ if d < 50 => d * 2 + 1,
        _ => 0x1E00 / (126 - d),
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Attack,
    Decay,
    Sustain,
    Release,
}

enum Source {
    Pcm(Rc<Vec<Wave>>, usize),
    Pulse(u8),
    Noise,
}

struct Voice {
    id: u64,
    track: usize,
    source: Source,
    key: i32,
    base_note: i32,
    vel_cb: i32,
    region_pan: i32,
    remaining: i32,
    dur: i32,
    stage: Stage,
    ampl: i32,
    a_rate: i32,
    d_rate: i32,
    s_level: i32,
    r_rate: i32,
    sweep: f32,
    pos: f64,
    phase: f64,
    lfo_t: u32,
    lfsr: u16,
    dead: bool,
    // per-update cached mix parameters
    gl: f32,
    gr: f32,
    step: f64,
}

struct Track {
    pc: Option<u32>,
    wait: u32,
    prog: u32,
    vol: i32,
    expr: i32,
    pan: i32,
    transpose: i32,
    bend: i32,
    bend_range: i32,
    note_wait: bool,
    tie: bool,
    mod_depth: i32,
    mod_speed: i32,
    mod_type: i32,
    mod_range: i32,
    mod_delay: i32,
    atk: Option<u8>,
    dec: Option<u8>,
    sus: Option<u8>,
    rel: Option<u8>,
    sweep: i32,
    calls: Vec<u32>,
    loops: Vec<(u32, u32)>,
    vars: [i16; 16],
    cond: bool,
    last_voice: Option<u64>,
    visited: HashMap<u32, u64>,
    first_loop: Option<(u64, u64)>,
}

impl Track {
    fn new(pc: u32) -> Track {
        Track {
            pc: Some(pc),
            wait: 0,
            prog: 0,
            vol: 127,
            expr: 127,
            pan: 64,
            transpose: 0,
            bend: 0,
            bend_range: 2,
            note_wait: true,
            tie: false,
            mod_depth: 0,
            mod_speed: 16,
            mod_type: 0,
            mod_range: 1,
            mod_delay: 0,
            atk: None,
            dec: None,
            sus: None,
            rel: None,
            sweep: 0,
            calls: Vec::new(),
            loops: Vec::new(),
            vars: [0; 16],
            cond: true,
            last_voice: None,
            visited: HashMap::new(),
            first_loop: None,
        }
    }
}

struct Engine<'a> {
    events: &'a BTreeMap<u32, Event>,
    bank: Rc<BankSet>,
    seq_vol: i32,
    mix: bool,
    tracks: Vec<Option<Track>>,
    voices: Vec<Voice>,
    tempo: u32,
    tempo_acc: u32,
    master: i32,
    globals: [i16; 16],
    rng: u32,
    next_id: u64,
    upd: u64,
}

impl<'a> Engine<'a> {
    fn new(
        events: &'a BTreeMap<u32, Event>,
        start: u32,
        bank: Rc<BankSet>,
        seq_vol: u8,
        mix: bool,
    ) -> Self {
        let mut tracks: Vec<Option<Track>> = (0..16).map(|_| None).collect();
        tracks[0] = Some(Track::new(start));
        Engine {
            events,
            bank,
            seq_vol: i32::from(seq_vol),
            mix,
            tracks,
            voices: Vec::new(),
            tempo: 120,
            tempo_acc: 0,
            master: 127,
            globals: [0; 16],
            rng: 0x1234_5678,
            next_id: 1,
            upd: 0,
        }
    }

    fn all_ended(&self) -> bool {
        self.tracks.iter().flatten().all(|t| t.pc.is_none())
    }

    fn all_looped_or_ended(&self) -> bool {
        self.tracks
            .iter()
            .flatten()
            .all(|t| t.pc.is_none() || t.first_loop.is_some())
    }

    fn best_loop(&self) -> Option<(u64, u64)> {
        self.tracks
            .iter()
            .flatten()
            .filter_map(|t| t.first_loop)
            .filter(|(s, e)| e > s)
            .max_by_key(|(s, e)| (*e, e - s))
    }

    fn halt(&mut self) {
        for t in self.tracks.iter_mut().flatten() {
            t.pc = None;
        }
        for v in &mut self.voices {
            v.remaining = 0;
        }
    }

    fn rand(&mut self) -> u32 {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.rng >> 8
    }

    /// One 1/192 s step: sequencer ticks, then voice envelopes and cached mix parameters.
    fn update(&mut self) {
        self.tempo_acc += self.tempo;
        while self.tempo_acc >= 240 {
            self.tempo_acc -= 240;
            self.tick();
        }
        self.update_voices_only();
        self.upd += 1;
    }

    fn update_voices_only(&mut self) {
        if !self.mix {
            return;
        }
        self.refresh_voices();
    }

    fn tick(&mut self) {
        for v in &mut self.voices {
            if v.stage != Stage::Release {
                v.remaining -= 1;
                if v.remaining <= 0 {
                    v.stage = Stage::Release;
                }
            }
        }
        for ti in 0..16 {
            let Some(t) = self.tracks[ti].as_mut() else {
                continue;
            };
            if t.wait > 0 {
                t.wait -= 1;
            }
            let mut guard = 0;
            while self.tracks[ti]
                .as_ref()
                .is_some_and(|t| t.wait == 0 && t.pc.is_some())
            {
                self.step(ti);
                guard += 1;
                if guard > 100_000 {
                    // runaway zero-wait loop: end the track
                    if let Some(t) = self.tracks[ti].as_mut() {
                        t.pc = None;
                    }
                }
            }
        }
    }

    fn var(&self, t: &Track, n: u8) -> i16 {
        if n < 16 {
            t.vars[n as usize]
        } else {
            self.globals[(n as usize - 16) % 16]
        }
    }

    fn resolve(&mut self, ti: usize, ev: &Event) -> Option<Event> {
        match ev {
            Event::If(inner) => {
                if self.tracks[ti].as_ref().is_some_and(|t| t.cond) {
                    self.resolve(ti, inner)
                } else {
                    None
                }
            }
            Event::Modified { modifier, inner } => {
                let v = match *modifier {
                    Modifier::Random(a, b) => {
                        let (lo, hi) = (i32::from(a.min(b)), i32::from(a.max(b)));
                        lo + (self.rand() % ((hi - lo + 1) as u32)) as i32
                    }
                    Modifier::Variable(n) => {
                        i32::from(self.var(self.tracks[ti].as_ref().expect("track"), n))
                    }
                };
                let inner = self.resolve(ti, inner)?;
                Some(match inner {
                    Event::Note { key, velocity, .. } => Event::Note {
                        key,
                        velocity,
                        duration: v.max(0) as u32,
                    },
                    Event::Rest(_) => Event::Rest(v.max(0) as u32),
                    Event::Program(_) => Event::Program(v.max(0) as u32),
                    Event::Ctrl { kind, .. } => Event::Ctrl { kind, value: v },
                    Event::Var { op, var, .. } => Event::Var {
                        op,
                        var,
                        value: v as i16,
                    },
                    Event::OpenTrack { track, .. } => Event::OpenTrack {
                        track,
                        offset: v.max(0) as u32,
                    },
                    other => other,
                })
            }
            e => Some(e.clone()),
        }
    }

    /// Execute the event at the track's pc.
    fn step(&mut self, ti: usize) {
        let upd = self.upd;
        let t = self.tracks[ti].as_mut().expect("track");
        let cur = t.pc.expect("pc");
        t.visited.entry(cur).or_insert(upd);
        let Some(ev) = self.events.get(&cur) else {
            t.pc = None;
            return;
        };
        let next = self.events.range(cur + 1..).next().map(|(k, _)| *k);
        t.pc = next;
        let ev = ev.clone();
        let is_cond = matches!(ev, Event::If(_));
        let Some(ev) = self.resolve(ti, &ev) else {
            return;
        };
        let _ = is_cond;
        match ev {
            Event::Note {
                key,
                velocity,
                duration,
            } => {
                self.note(ti, key, velocity, duration);
                let t = self.tracks[ti].as_mut().expect("track");
                if t.note_wait {
                    t.wait = duration;
                }
            }
            Event::Rest(n) => self.tracks[ti].as_mut().expect("track").wait = n,
            Event::Program(p) => self.tracks[ti].as_mut().expect("track").prog = p,
            Event::OpenTrack { track, offset } => {
                if (track as usize) < 16 && track != 0 {
                    self.tracks[track as usize] = Some(Track::new(offset));
                }
            }
            Event::Jump(target) => {
                let t = self.tracks[ti].as_mut().expect("track");
                if target <= cur && t.first_loop.is_none() {
                    if let Some(s) = t.visited.get(&target) {
                        t.first_loop = Some((*s, upd));
                    }
                }
                t.pc = Some(target);
            }
            Event::Call(target) => {
                let t = self.tracks[ti].as_mut().expect("track");
                if t.calls.len() < 16 {
                    if let Some(n) = next {
                        t.calls.push(n);
                    }
                    t.pc = Some(target);
                }
            }
            Event::Return => {
                let t = self.tracks[ti].as_mut().expect("track");
                t.pc = t.calls.pop();
            }
            Event::Var { op, var, value } => self.var_op(ti, op, var, value),
            Event::Ctrl { kind, value } => self.ctrl(ti, kind, value),
            Event::LoopEnd => {
                let t = self.tracks[ti].as_mut().expect("track");
                if let Some(top) = t.loops.last_mut() {
                    let start = top.0;
                    if top.1 == 0 {
                        if t.first_loop.is_none() {
                            if let Some(s) = t.visited.get(&start) {
                                t.first_loop = Some((*s, upd));
                            }
                        }
                        t.pc = Some(start);
                    } else {
                        top.1 -= 1;
                        if top.1 == 0 {
                            t.loops.pop();
                        } else {
                            t.pc = Some(start);
                        }
                    }
                }
            }
            Event::AllocTracks(_) => {}
            Event::EndTrack => self.tracks[ti].as_mut().expect("track").pc = None,
            Event::Modified { .. } | Event::If(_) => {}
        }
    }

    fn var_op(&mut self, ti: usize, op: VarOp, var: u8, value: i16) {
        let rnd = self.rand();
        let mut g = self.globals;
        let t = self.tracks[ti].as_mut().expect("track");
        let slot: &mut i16 = if var < 16 {
            &mut t.vars[var as usize]
        } else {
            &mut g[(var as usize - 16) % 16]
        };
        let cur = i32::from(*slot);
        let val = i32::from(value);
        let mut cmp = None;
        let new = match op {
            VarOp::Set => val,
            VarOp::Add => cur + val,
            VarOp::Sub => cur - val,
            VarOp::Mul => cur * val,
            VarOp::Div => {
                if val != 0 {
                    cur / val
                } else {
                    cur
                }
            }
            VarOp::Shift => {
                if val >= 0 {
                    cur << val.min(15)
                } else {
                    cur >> (-val).min(15)
                }
            }
            VarOp::Rand => {
                let m = val.unsigned_abs() + 1;
                let r = (rnd % m) as i32;
                if val < 0 {
                    -r
                } else {
                    r
                }
            }
            VarOp::Unused => cur,
            VarOp::CmpEq => {
                cmp = Some(cur == val);
                cur
            }
            VarOp::CmpGe => {
                cmp = Some(cur >= val);
                cur
            }
            VarOp::CmpGt => {
                cmp = Some(cur > val);
                cur
            }
            VarOp::CmpLe => {
                cmp = Some(cur <= val);
                cur
            }
            VarOp::CmpLt => {
                cmp = Some(cur < val);
                cur
            }
            VarOp::CmpNe => {
                cmp = Some(cur != val);
                cur
            }
        };
        *slot = new as i16;
        if let Some(c) = cmp {
            t.cond = c;
        }
        self.globals = g;
    }

    fn ctrl(&mut self, ti: usize, kind: Ctrl, v: i32) {
        match kind {
            Ctrl::Tempo => self.tempo = v.clamp(1, 1023) as u32,
            Ctrl::MasterVolume => self.master = v,
            _ => {
                let t = self.tracks[ti].as_mut().expect("track");
                match kind {
                    Ctrl::Pan => t.pan = v,
                    Ctrl::Volume => t.vol = v,
                    Ctrl::Expression => t.expr = v,
                    Ctrl::Transpose => t.transpose = v,
                    Ctrl::PitchBend => t.bend = v,
                    Ctrl::BendRange => t.bend_range = v,
                    Ctrl::NoteWait => t.note_wait = v != 0,
                    Ctrl::Tie => t.tie = v != 0,
                    Ctrl::ModDepth => t.mod_depth = v,
                    Ctrl::ModSpeed => t.mod_speed = v,
                    Ctrl::ModType => t.mod_type = v,
                    Ctrl::ModRange => t.mod_range = v,
                    Ctrl::ModDelay => t.mod_delay = v,
                    Ctrl::Attack => t.atk = Some(v as u8),
                    Ctrl::Decay => t.dec = Some(v as u8),
                    Ctrl::Sustain => t.sus = Some(v as u8),
                    Ctrl::Release => t.rel = Some(v as u8),
                    Ctrl::SweepPitch => t.sweep = v,
                    // priority, portamento, loop start (handled below), print
                    Ctrl::LoopStart => {
                        if let Some(n) = t.pc {
                            if t.loops.len() < 4 {
                                t.loops.push((n, v as u32));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn note(&mut self, ti: usize, key: u8, vel: u8, dur: u32) {
        if !self.mix {
            return;
        }
        let t = self.tracks[ti].as_ref().expect("track");
        let key = i32::from(key);
        // tie: retarget the held voice
        if t.tie {
            if let Some(id) = t.last_voice {
                let k = key + t.transpose;
                if let Some(v) = self
                    .voices
                    .iter_mut()
                    .find(|v| v.id == id && v.stage != Stage::Release)
                {
                    v.key = k;
                    v.remaining = dur as i32;
                    v.dur = dur as i32;
                    return;
                }
            }
        }
        let Some(inst) = self.bank.sbnk.instruments.get(t.prog as usize) else {
            return;
        };
        let region: Option<&Region> = match inst {
            Instrument::Empty => None,
            Instrument::Single(r) => Some(r),
            Instrument::Drum { lo, hi, regions } => {
                if key >= i32::from(*lo) && key <= i32::from(*hi) {
                    regions.get((key - i32::from(*lo)) as usize)
                } else {
                    None
                }
            }
            Instrument::Split { ends, regions } => ends
                .iter()
                .position(|e| key <= i32::from(*e))
                .and_then(|i| regions.get(i)),
        };
        let Some(r) = region else {
            return;
        };
        let source = match r.kind {
            2 => Source::Pulse(r.swav as u8 & 7),
            3 => Source::Noise,
            _ => {
                let Some(Some(arc)) = self.bank.waves.get(r.swar as usize) else {
                    return;
                };
                if arc.get(r.swav as usize).is_none_or(|w| w.data.is_empty()) {
                    return;
                }
                Source::Pcm(arc.clone(), r.swav as usize)
            }
        };
        let a = t.atk.unwrap_or(r.attack);
        let d = t.dec.unwrap_or(r.decay);
        let s = t.sus.unwrap_or(r.sustain);
        let rl = t.rel.unwrap_or(r.release);
        let id = self.next_id;
        self.next_id += 1;
        let v = Voice {
            id,
            track: ti,
            source,
            key: key + t.transpose,
            base_note: i32::from(r.note),
            vel_cb: conv_vol(i32::from(vel)),
            region_pan: i32::from(r.pan.min(127)),
            remaining: dur as i32,
            dur: dur as i32,
            stage: Stage::Attack,
            ampl: AMPL_MIN,
            a_rate: attack_rate(a),
            d_rate: decay_rate(d),
            s_level: (conv_vol(i32::from(s.min(127))) * 128).max(AMPL_MIN),
            r_rate: decay_rate(rl),
            sweep: t.sweep as f32 / 64.0,
            pos: 0.0,
            phase: 0.0,
            lfo_t: 0,
            lfsr: 0x7FFF,
            dead: false,
            gl: 0.0,
            gr: 0.0,
            step: 0.0,
        };
        if self.voices.len() >= MAX_VOICES {
            // steal the quietest/oldest releasing voice, else the oldest
            let idx = self
                .voices
                .iter()
                .position(|v| v.stage == Stage::Release)
                .unwrap_or(0);
            self.voices.remove(idx);
        }
        self.voices.push(v);
        self.tracks[ti].as_mut().expect("track").last_voice = Some(id);
    }

    fn refresh_voices(&mut self) {
        let master = conv_vol(self.master) + conv_vol(self.seq_vol);
        let tracks = &self.tracks;
        for v in &mut self.voices {
            // envelope
            match v.stage {
                Stage::Attack => {
                    v.ampl = (i64::from(v.ampl) * i64::from(v.a_rate) / 256) as i32;
                    if v.ampl >= 0 {
                        v.ampl = 0;
                        v.stage = Stage::Decay;
                    }
                }
                Stage::Decay => {
                    v.ampl -= v.d_rate;
                    if v.ampl <= v.s_level {
                        v.ampl = v.s_level;
                        v.stage = Stage::Sustain;
                    }
                }
                Stage::Sustain => {}
                Stage::Release => {
                    v.ampl = v.ampl.saturating_sub(v.r_rate);
                    if v.ampl <= AMPL_MIN {
                        v.dead = true;
                    }
                }
            }
            let Some(t) = tracks[v.track].as_ref() else {
                v.dead = true;
                continue;
            };
            // modulation
            let (mut lfo_pitch, mut lfo_vol, mut lfo_pan) = (0.0f32, 0.0f32, 0.0f32);
            if t.mod_depth > 0 {
                v.lfo_t += 1;
                if v.lfo_t as i32 > t.mod_delay {
                    let ph =
                        f64::from(v.lfo_t - t.mod_delay as u32) * f64::from(t.mod_speed) * 0.375
                            / UPDATE_HZ as f64;
                    let s = (ph * std::f64::consts::TAU).sin() as f32;
                    let amt = s * t.mod_depth as f32 / 127.0 * t.mod_range as f32;
                    match t.mod_type {
                        0 => lfo_pitch = amt,
                        1 => lfo_vol = amt * 60.0,
                        _ => lfo_pan = amt * 32.0,
                    }
                }
            }
            let mut sweep = 0.0;
            if v.sweep != 0.0 && v.dur > 0 && v.stage != Stage::Release {
                sweep = v.sweep * (v.remaining.max(0) as f32 / v.dur as f32);
            }
            let cb = conv_vol(t.vol)
                + conv_vol(t.expr)
                + master
                + v.vel_cb
                + (v.ampl >> 7)
                + lfo_vol as i32;
            if cb <= -723 {
                v.gl = 0.0;
                v.gr = 0.0;
            } else {
                let g = 10f32.powf(cb as f32 / 200.0);
                let pan = (t.pan + v.region_pan - 64 + lfo_pan as i32).clamp(0, 127) as f32;
                v.gl = g * ((127.0 - pan) / 64.0).min(1.0);
                v.gr = g * (pan / 64.0).min(1.0);
            }
            let semis = f64::from(v.key)
                + f64::from(t.bend * t.bend_range) / 127.0
                + f64::from(lfo_pitch)
                + f64::from(sweep);
            v.step = match &v.source {
                Source::Pcm(arc, i) => {
                    let w = &arc[*i];
                    2f64.powf((semis - f64::from(v.base_note)) / 12.0) * w.rate
                        / f64::from(OUT_RATE)
                }
                _ => 440.0 * 2f64.powf((semis - 69.0) / 12.0) / f64::from(OUT_RATE),
            };
        }
        self.voices.retain(|v| !v.dead);
    }

    /// Mix one update's worth of samples into `l`/`r`.
    fn mix(&mut self, l: &mut [f32], r: &mut [f32]) {
        let n = l.len();
        for v in &mut self.voices {
            if v.gl == 0.0 && v.gr == 0.0 && !matches!(v.source, Source::Pcm(..)) {
                continue;
            }
            match &v.source {
                Source::Pcm(arc, i) => {
                    let w = &arc[*i];
                    let len = w.data.len();
                    let (ls, le) = w.looping.unwrap_or((0, len));
                    let le = le.min(len);
                    for k in 0..n {
                        if v.pos >= le as f64 {
                            if w.looping.is_some() && le > ls {
                                v.pos = ls as f64 + (v.pos - ls as f64) % (le - ls) as f64;
                            } else {
                                v.dead = true;
                                break;
                            }
                        }
                        let idx = v.pos as usize;
                        let fr = (v.pos - idx as f64) as f32;
                        let mut nx = idx + 1;
                        if nx >= le {
                            nx = if w.looping.is_some() { ls } else { idx };
                        }
                        let s0 = f32::from(w.data[idx]);
                        let s1 = f32::from(w.data[nx]);
                        let s = (s0 + (s1 - s0) * fr) / 32768.0;
                        l[k] += s * v.gl;
                        r[k] += s * v.gr;
                        v.pos += v.step;
                    }
                }
                Source::Pulse(duty) => {
                    let width = f64::from(*duty + 1) / 8.0;
                    for k in 0..n {
                        let s = if v.phase < width { 0.3 } else { -0.3 };
                        l[k] += s * v.gl;
                        r[k] += s * v.gr;
                        v.phase = (v.phase + v.step).fract();
                    }
                }
                Source::Noise => {
                    for k in 0..n {
                        let s = if v.lfsr & 1 == 1 { -0.3 } else { 0.3 };
                        l[k] += s * v.gl;
                        r[k] += s * v.gr;
                        v.phase += v.step * 8.0;
                        while v.phase >= 1.0 {
                            v.phase -= 1.0;
                            let bit = (v.lfsr ^ (v.lfsr >> 1)) & 1;
                            v.lfsr = (v.lfsr >> 1) | (bit << 14);
                        }
                    }
                }
            }
        }
        self.voices.retain(|v| !v.dead);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rates_and_volumes() {
        assert_eq!(conv_vol(127), 0);
        assert_eq!(conv_vol(0), -723);
        assert!(conv_vol(64) < conv_vol(100));
        assert_eq!(attack_rate(127), 0);
        assert_eq!(attack_rate(0), 255);
        assert_eq!(decay_rate(127), 0xFFFF);
        assert_eq!(decay_rate(10), 21);
    }

    #[test]
    fn wav_has_smpl_chunk() {
        let r = Rendered {
            pcm: Pcm {
                sample_rate: OUT_RATE,
                channels: vec![vec![0; 4], vec![0; 4]],
                looping: Some((1, 3)),
            },
            loops: true,
        };
        let w = r.to_wav();
        assert!(w.windows(4).any(|x| x == b"smpl"));
        let riff = u32::from_le_bytes(w[4..8].try_into().unwrap()) as usize;
        assert_eq!(riff, w.len() - 8);
    }
}
