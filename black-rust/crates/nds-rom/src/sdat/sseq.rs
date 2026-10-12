//! SSEQ: a sequence (MIDI-like event stream) parsed into a typed event list.
//!
//! File: `SSEQ` Nitro header (16 bytes), `DATA` block (`tag`, `size`, `u32 data_offset`), event
//! bytes at `data_offset` (file-relative). All jump/call/open-track offsets are relative to that
//! event data start. Events are discovered by following control flow from the first track and
//! every `OpenTrack`/`Jump`/`Call` target, so padding and embedded data are never decoded.

use super::{bad, find_block, u16_at, u32_at, u8_at};
use crate::error::{Error, Result};
use std::collections::BTreeMap;

const W: &str = "SSEQ";

/// Simple one-parameter track commands (0xC0..0xE3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ctrl {
    Pan,
    Volume,
    MasterVolume,
    Transpose,
    PitchBend,
    BendRange,
    Priority,
    NoteWait,
    Tie,
    PortaControl,
    ModDepth,
    ModSpeed,
    ModType,
    ModRange,
    PortaOnOff,
    PortaTime,
    Attack,
    Decay,
    Sustain,
    Release,
    LoopStart,
    Expression,
    PrintVar,
    ModDelay,
    Tempo,
    SweepPitch,
}

/// Variable operations (0xB0..0xBD).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VarOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Shift,
    Rand,
    Unused,
    CmpEq,
    CmpGe,
    CmpGt,
    CmpLe,
    CmpLt,
    CmpNe,
}

/// A prefix that replaces the last parameter of the following command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modifier {
    /// 0xA0: random in `[min, max]`.
    Random(i16, i16),
    /// 0xA1: value read from variable.
    Variable(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Note {
        key: u8,
        velocity: u8,
        duration: u32,
    },
    Rest(u32),
    Program(u32),
    OpenTrack {
        track: u8,
        offset: u32,
    },
    Jump(u32),
    Call(u32),
    Var {
        op: VarOp,
        var: u8,
        value: i16,
    },
    Ctrl {
        kind: Ctrl,
        value: i32,
    },
    LoopEnd,
    Return,
    AllocTracks(u16),
    EndTrack,
    /// 0xA0/0xA1 prefix. The inner event's replaced parameter is 0.
    Modified {
        modifier: Modifier,
        inner: Box<Event>,
    },
    /// 0xA2 prefix: the next command only runs if the last comparison was true.
    If(Box<Event>),
}

/// A decoded sequence: events keyed by offset (relative to the event data start).
#[derive(Debug, Clone, Default)]
pub struct Sequence {
    pub events: BTreeMap<u32, Event>,
    /// Entry offsets of every track (the first is the start offset itself).
    pub tracks: Vec<u32>,
}

#[derive(Clone, Copy)]
enum P {
    U8,
    S8,
    U16,
    S16,
    U24,
    Var,
}

fn spec(cmd: u8) -> Option<&'static [P]> {
    Some(match cmd {
        0x00..=0x7F => &[P::U8, P::Var],
        0x80 | 0x81 => &[P::Var],
        0x93 => &[P::U8, P::U24],
        0x94 | 0x95 => &[P::U24],
        0xB0..=0xBD => &[P::U8, P::S16],
        0xC3 | 0xC4 => &[P::S8],
        0xC0..=0xD6 => &[P::U8],
        0xE0 | 0xE1 => &[P::U16],
        0xE3 => &[P::S16],
        0xFC | 0xFD | 0xFF => &[],
        0xFE => &[P::U16],
        _ => return None,
    })
}

fn ctrl_of(cmd: u8) -> Option<Ctrl> {
    use Ctrl::*;
    Some(match cmd {
        0xC0 => Pan,
        0xC1 => Volume,
        0xC2 => MasterVolume,
        0xC3 => Transpose,
        0xC4 => PitchBend,
        0xC5 => BendRange,
        0xC6 => Priority,
        0xC7 => NoteWait,
        0xC8 => Tie,
        0xC9 => PortaControl,
        0xCA => ModDepth,
        0xCB => ModSpeed,
        0xCC => ModType,
        0xCD => ModRange,
        0xCE => PortaOnOff,
        0xCF => PortaTime,
        0xD0 => Attack,
        0xD1 => Decay,
        0xD2 => Sustain,
        0xD3 => Release,
        0xD4 => LoopStart,
        0xD5 => Expression,
        0xD6 => PrintVar,
        0xE0 => ModDelay,
        0xE1 => Tempo,
        0xE3 => SweepPitch,
        _ => return None,
    })
}

const VAR_OPS: [VarOp; 14] = [
    VarOp::Set,
    VarOp::Add,
    VarOp::Sub,
    VarOp::Mul,
    VarOp::Div,
    VarOp::Shift,
    VarOp::Rand,
    VarOp::Unused,
    VarOp::CmpEq,
    VarOp::CmpGe,
    VarOp::CmpGt,
    VarOp::CmpLe,
    VarOp::CmpLt,
    VarOp::CmpNe,
];

struct Cursor<'a> {
    d: &'a [u8],
    pos: usize,
}

impl Cursor<'_> {
    fn u8(&mut self) -> Result<u8> {
        let v = u8_at(self.d, self.pos, W)?;
        self.pos += 1;
        Ok(v)
    }
    fn read(&mut self, p: P) -> Result<i32> {
        Ok(match p {
            P::U8 => i32::from(self.u8()?),
            P::S8 => i32::from(self.u8()? as i8),
            P::U16 => {
                let v = u16_at(self.d, self.pos, W)?;
                self.pos += 2;
                i32::from(v)
            }
            P::S16 => {
                let v = u16_at(self.d, self.pos, W)?;
                self.pos += 2;
                i32::from(v as i16)
            }
            P::U24 => {
                let (a, b, c) = (self.u8()?, self.u8()?, self.u8()?);
                i32::from(a) | i32::from(b) << 8 | i32::from(c) << 16
            }
            P::Var => {
                let mut v = 0i32;
                for _ in 0..4 {
                    let b = self.u8()?;
                    v = v << 7 | i32::from(b & 0x7F);
                    if b & 0x80 == 0 {
                        return Ok(v);
                    }
                }
                return Err(bad(W, "variable-length value too long"));
            }
        })
    }

    fn command(&mut self, depth: u8) -> Result<Event> {
        let at = self.pos;
        let cmd = self.u8()?;
        if depth > 4 {
            return Err(bad(W, format!("modifier chain too deep at {at:#x}")));
        }
        match cmd {
            0xA2 => return Ok(Event::If(Box::new(self.command(depth + 1)?))),
            0xA0 | 0xA1 => {
                let modifier = if cmd == 0xA0 {
                    let (a, b) = (self.read(P::S16)?, self.read(P::S16)?);
                    Modifier::Random(a as i16, b as i16)
                } else {
                    Modifier::Variable(self.read(P::U8)? as u8)
                };
                // The following command with its last parameter replaced.
                let c2 = self.u8()?;
                let ps = spec(c2).ok_or_else(|| bad(W, format!("unknown command {c2:#x}")))?;
                let (_, init) = ps
                    .split_last()
                    .ok_or_else(|| bad(W, format!("modifier on parameterless {c2:#x}")))?;
                let mut args = Vec::new();
                for p in init {
                    args.push(self.read(*p)?);
                }
                args.push(0);
                return Ok(Event::Modified {
                    modifier,
                    inner: Box::new(build(c2, &args)?),
                });
            }
            _ => {}
        }
        let ps = spec(cmd).ok_or_else(|| bad(W, format!("unknown command {cmd:#x} at {at:#x}")))?;
        let mut args = Vec::new();
        for p in ps {
            args.push(self.read(*p)?);
        }
        build(cmd, &args)
    }
}

fn build(cmd: u8, a: &[i32]) -> Result<Event> {
    Ok(match cmd {
        0x00..=0x7F => Event::Note {
            key: cmd,
            velocity: a[0] as u8,
            duration: a[1] as u32,
        },
        0x80 => Event::Rest(a[0] as u32),
        0x81 => Event::Program(a[0] as u32),
        0x93 => Event::OpenTrack {
            track: a[0] as u8,
            offset: a[1] as u32,
        },
        0x94 => Event::Jump(a[0] as u32),
        0x95 => Event::Call(a[0] as u32),
        0xB0..=0xBD => Event::Var {
            op: VAR_OPS[(cmd - 0xB0) as usize],
            var: a[0] as u8,
            value: a[1] as i16,
        },
        0xFC => Event::LoopEnd,
        0xFD => Event::Return,
        0xFE => Event::AllocTracks(a[0] as u16),
        0xFF => Event::EndTrack,
        _ => Event::Ctrl {
            kind: ctrl_of(cmd).ok_or_else(|| bad(W, format!("unknown command {cmd:#x}")))?,
            value: a[0],
        },
    })
}

/// Decode all events reachable from `start` in `data` (the event bytes; offsets relative to it).
pub fn decode(data: &[u8], start: u32) -> Result<Sequence> {
    let mut seq = Sequence {
        tracks: vec![start],
        ..Sequence::default()
    };
    let mut work = vec![start];
    while let Some(mut pos) = work.pop() {
        loop {
            if seq.events.contains_key(&pos) {
                break;
            }
            if pos as usize >= data.len() {
                return Err(Error::Truncated {
                    what: W,
                    offset: pos as usize,
                    need: 1,
                    have: data.len(),
                });
            }
            let mut c = Cursor {
                d: data,
                pos: pos as usize,
            };
            let ev = c.command(0)?;
            let (target, stop) = flow(&ev);
            if let Event::OpenTrack { offset, .. } = peel(&ev) {
                seq.tracks.push(*offset);
            }
            if let Some(t) = target {
                work.push(t);
            }
            seq.events.insert(pos, ev);
            if stop {
                break;
            }
            pos = c.pos as u32;
        }
    }
    Ok(seq)
}

fn peel(e: &Event) -> &Event {
    match e {
        Event::If(i) => peel(i),
        Event::Modified { inner, .. } => inner,
        e => e,
    }
}

/// (branch target to explore, whether linear flow stops here)
fn flow(e: &Event) -> (Option<u32>, bool) {
    let conditional = matches!(e, Event::If(_));
    match peel(e) {
        Event::OpenTrack { offset, .. } | Event::Call(offset) => (Some(*offset), false),
        Event::Jump(t) => (Some(*t), !conditional),
        Event::EndTrack | Event::Return => (None, !conditional),
        _ => (None, false),
    }
}

/// A parsed SSEQ file.
#[derive(Debug, Clone)]
pub struct Sseq {
    pub sequence: Sequence,
    /// Size of the event data in bytes.
    pub data_len: usize,
}

impl Sseq {
    pub fn parse(file: &[u8]) -> Result<Sseq> {
        let blk = find_block(file, b"SSEQ", b"DATA", W)?;
        let off = u32_at(blk, 8, W)? as usize;
        // `off` is file-relative; the DATA block starts at the first block position.
        let blk_start = u16_at(file, 12, W)? as usize;
        let rel = off
            .checked_sub(blk_start)
            .ok_or_else(|| bad(W, "data offset before DATA block"))?;
        let data = blk
            .get(rel..)
            .ok_or_else(|| bad(W, "data offset past DATA block"))?;
        Ok(Sseq {
            sequence: decode(data, 0)?,
            data_len: data.len(),
        })
    }

    pub fn event_count(&self) -> usize {
        self.sequence.events.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(events: &[u8]) -> Vec<u8> {
        let mut f = b"SSEQ".to_vec();
        f.extend([0xFF, 0xFE, 0, 1]);
        f.extend((0x1C + events.len() as u32).to_le_bytes());
        f.extend([16, 0, 1, 0]);
        f.extend(b"DATA");
        f.extend((12 + events.len() as u32).to_le_bytes());
        f.extend(0x1Cu32.to_le_bytes());
        f.extend(events);
        f
    }

    #[test]
    fn synthetic() {
        // 0: open track 1 at 0x0C; 5: alloc tracks; 8: tempo; 11 wait... then track 0: note,
        // rest, random-modified volume, end.
        let ev = [
            0xFE, 0x02, 0x00, // 0 alloc tracks
            0x93, 1, 0x0F, 0, 0, // 3 open track 1 -> 0x0F
            0xE1, 120, 0,    // 8 tempo
            0xFF, // 11 end
            0, 0, 0, // padding (never decoded)
            0x3C, 100, 0x81, 0x40, // 15 note 60 vel 100 dur 0xC0
            0xA0, 0x00, 0x00, 0x10, 0x00, // placeholder
        ];
        // Make track 1 well-formed: note, rest, modified volume, end.
        let mut e = ev[..15].to_vec();
        e.extend([
            0x3C, 100, 0x81, 0x40, 0x80, 0x30, 0xA0, 0x10, 0x00, 0x20, 0x00, 0xC1, 0xFF,
        ]);
        let s = Sseq::parse(&file(&e)).unwrap();
        assert_eq!(s.event_count(), 4 + 4);
        assert_eq!(s.sequence.tracks, vec![0, 0x0F]);
        assert_eq!(
            s.sequence.events[&0x0F],
            Event::Note {
                key: 0x3C,
                velocity: 100,
                duration: 0xC0
            }
        );
        assert_eq!(
            s.sequence.events[&0x15],
            Event::Modified {
                modifier: Modifier::Random(16, 32),
                inner: Box::new(Event::Ctrl {
                    kind: Ctrl::Volume,
                    value: 0
                })
            }
        );
    }

    #[test]
    fn errors() {
        assert!(Sseq::parse(&file(&[0xD7])).is_err());
        assert!(Sseq::parse(&file(&[0x3C])).is_err());
        assert!(Sseq::parse(b"junk").is_err());
    }
}
