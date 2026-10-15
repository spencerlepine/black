//! The F12 session: boot the field on a new save in Nuvema Town, then a scripted play session through the
//! ported field code — walk, talk to an NPC (its script runs through the black-script ports, the printed text is
//! decoded with black-text), enter a house and leave it (two warps through the event_mapchange ports), walk north
//! to Route 1's tall grass until the encounter roll starts a wild battle (the battle event and its setup are
//! recorded; the session stops before the battle overlays load).
//!
//! Every frame adds one line to the trace (format in [`Session::trace_line`]); the test compares the FNV-1a hash of
//! the whole trace and of each phase with `RECORDED`. `F12_TRACE_OUT=<file>` writes the trace (diagnostics, and
//! the input of a ROM-side differential).

use super::Env;
use crate::hooks::HEAP_ALLOC;

/// The machine, or `None` (skip) without the ROM build; with `REQUIRE_ROM` set a missing ROM fails.
fn env() -> Option<Env> {
    let e = Env::new();
    if e.is_none() {
        assert!(
            std::env::var_os("REQUIRE_ROM").is_none_or(|v| v == "0"),
            "F12: REQUIRE_ROM is set but the ROM build / files are missing"
        );
        eprintln!("F12: ROM build missing, skipped");
    }
    e
}

/// The global the save-data code reads the backup-device state through (`02031AE0`: `**0x02146994 == 0` = no
/// backup device, so the boot initialises a new save).
const BACKUP_STATE: u32 = 0x0214_6994;
/// The GAMESYS pointer (`02011D9C` stores it).
const GAMESYS: u32 = 0x0214_6248;
/// The gfl system block (`0x020AA194`: `+0x10` -> the 0x28-byte system work: the MathRand context at `+0`, two
/// task systems at `+0x18` / `+0x20`).
const GFL_SYS: u32 = 0x020A_A194;
/// The RTC module's state (0x48 bytes: `+0x10` RTCDate year / month / day / week, `+0x20` RTCTime hour / minute /
/// second).
const RTC: u32 = 0x0214_69FC;
/// The script event's function (`script_sys::event_fn`, Thumb).
const SCRIPT_EVENT: u32 = 0x0215_88DD;
/// The wild / trainer battle event's proc (`event_battle::BATTLE_PROC`).
const BATTLE_EVENT: u32 = crate::event_battle::BATTLE_PROC;

/// Keys (DS KEYINPUT bit order).
pub const KEY_A: u32 = 0x01;
pub const KEY_RIGHT: u32 = 0x10;
pub const KEY_LEFT: u32 = 0x20;
pub const KEY_UP: u32 = 0x40;
pub const KEY_DOWN: u32 = 0x80;

/// The starter the session's party holds (Snivy, level 5): wild encounters need a lead Pokemon.
const STARTER: (u32, u32) = (495, 5);

fn alloc(e: &mut Env, size: u32) -> u32 {
    e.cv(HEAP_ALLOC, &[0, size, 0, 0, 0])
}

/// The parts of the gfl boot `0200545C` the field reads (the rest of it initialises hardware, D3 / system
/// boundary): the std block (`0203EF5C`: Mersenne Twister state, CRC table), the UI system (`02034E24`: keys,
/// touch; the touch panel's ARM7 calls are host models), the main proc system (`0203129C`), the system work
/// with its two task systems (`02030CB0` size / `02030CBC` create, 4 and 0x10 tasks), the MathRand context
/// seeded (`020056D0`: the RTC seed is the host's fixed seed, then the Mersenne Twister seeded from it),
/// `0204E5E8` / `0204E130` / `0200566C` / `0204E6F4`; the RTC's date and time are fixed.
fn boot_system(e: &mut Env) {
    e.cv(0x0203_EF5C, &[0]);
    e.cv(0x0203_4E24, &[0, 0]);
    e.cv(0x0203_129C, &[0]);
    let sys = alloc(e, 0x28);
    e.w32(GFL_SYS + 0x10, sys);
    for (n, mem, task) in [(4, 0x1C, 0x18), (0x10, 0x24, 0x20)] {
        let size = e.cv(0x0203_0CB0, &[n]);
        let m = alloc(e, size);
        e.w32(sys + mem, m);
        let t = e.cv(0x0203_0CBC, &[n, m]);
        e.w32(sys + task, t);
    }
    e.cv(0x0200_56D0, &[]);
    e.cv(0x0204_E5E8, &[0]);
    e.cv(0x0204_E130, &[0]);
    e.cv(0x0200_566C, &[]);
    e.cv(0x0204_E6F4, &[2]);
    // the RTC module's date / time (`0203F49C` copies them; the ARM7 RTC updates them every 10 frames
    // (`0203F3F0`), not run here: the clock stands): Saturday 2010-09-18 12:00:00
    for (o, v) in [
        (0x10, 10),
        (0x14, 9),
        (0x18, 18),
        (0x1C, 6),
        (0x20, 12),
        (0x24, 0),
        (0x28, 0),
    ] {
        e.w32(RTC + o, v);
    }
}

/// Boots the field: the gfl system parts ([`boot_system`]), the game's boot (`0200523C`: overlays, the save data
/// `02006F20(7)` with no backup device = a new save whose blocks the original code initialises, the config, the
/// global save handle, the comm block), then the game-system proc's init (`02011D9C`: GAMESYS, GAMEDATA, the start
/// event of kind 1 = continue). Returns the GAMESYS.
fn boot(e: &mut Env) -> u32 {
    boot_system(e);
    e.load_overlay(10);
    let z = alloc(e, 4);
    e.w32(BACKUP_STATE, z);
    e.cv(0x0200_523C, &[]);
    let proc_ = alloc(e, 0x40);
    let param = alloc(e, 0x20);
    e.w32(param, 1);
    e.w16(param + 0x10, 389);
    e.cv(0x0201_1D9C, &[proc_, 0, param]);
    e.r32(GAMESYS)
}

/// The start state on top of the new save (documented deviations from a real new game, which starts with the
/// intro scenes in the player's room): the continue event warps to the GAMEDATA location (`02183DE4` with flag
/// 0x966 set: its work `+0xC`), Nuvema Town (zone 389) at tile (`x`, `z`) facing up; story var 0x4080 = 3
/// (past the opening scenes: no scene on entering the town); the player's name; a level-5 Snivy in the party
/// (`020173F0` new Pokemon, `0201A98C` party add: original code).
fn start_state(e: &mut Env, gs: u32, zone: u32, x: u32, z: u32) {
    let ev = e.r32(gs + 0x18);
    let w = e.r32(ev + 0xC);
    e.w32(w + 0xC, 1);
    let gd = e.cv(0x0201_20F4, &[gs]);
    let loc = e.cv(0x0201_2894, &[gd]);
    e.cv(
        0x0201_4184,
        &[loc, zone, 1, x << 16 | 0x8000, 0, z << 16 | 0x8000],
    );
    let ew = e.cv(0x0201_296C, &[gd]);
    let v = e.cv(0x0201_43CC, &[ew, 0x4080]);
    e.w16(v, 3);
    let my = e.cv(0x0201_2944, &[gd]);
    for (i, c) in "Hilbert".encode_utf16().chain([0xFFFF]).enumerate() {
        e.w16(my + 2 * i as u32, c as u32);
    }
    let mon = e.cv(0x0201_73F0, &[STARTER.0, STARTER.1, 0, 0, 0]);
    let party = e.cv(0x0201_2934, &[gd]);
    e.cv(0x0201_A98C, &[party, mon]);
}

/// FNV-1a 64.
pub fn fnv64(b: &[u8]) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for &x in b {
        h ^= x as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// The battle the encounter started.
#[derive(Debug, Clone)]
pub struct Encounter {
    pub frame: u32,
    pub species: u32,
    pub level: u32,
    /// Battle setup type / flags (`+0`, `+0x82`).
    pub setup_type: u32,
    pub setup_flags: u32,
    pub zone: u32,
    pub at: (u32, u32),
}

/// One session.
pub struct Session {
    pub e: Env,
    pub gs: u32,
    pub frame: u32,
    pub trace: Vec<String>,
    /// `(phase, first trace line)`.
    pub phases: Vec<(&'static str, usize)>,
    pub warps: Vec<(u32, u32, u32, u32)>,
    pub encounter: Option<Encounter>,
    last_event: u32,
    /// `F12_TRACE_OUT`: the trace, line by line as it is made.
    out: Option<std::fs::File>,
}

impl Session {
    fn new(mut e: Env) -> Session {
        let gs = boot(&mut e);
        start_state(&mut e, gs, 389, 785, 750);
        Session {
            e,
            gs,
            frame: 0,
            trace: Vec::new(),
            phases: vec![("boot", 0)],
            warps: Vec::new(),
            encounter: None,
            last_event: 0,
            out: std::env::var("F12_TRACE_OUT")
                .ok()
                .map(|p| std::fs::File::create(p).unwrap()),
        }
    }

    fn phase(&mut self, name: &'static str) {
        self.phases.push((name, self.trace.len()));
    }

    fn fm(&mut self) -> u32 {
        self.e.r32(self.gs + 0x20)
    }

    /// The player MMDL, if the field runs.
    fn player_mmdl(&mut self) -> Option<u32> {
        let fm = self.fm();
        if fm == 0 || self.e.r32(fm + 0xC0) == 0 {
            return None;
        }
        let pl = self.e.r32(fm + 0x8C);
        if pl == 0 {
            return None;
        }
        let core = self.e.r32(pl + 4);
        let m = self.e.r32(core + 0x1C);
        (m != 0).then_some(m)
    }

    /// `(x, z, dir)` of the player.
    pub fn player(&mut self) -> (u32, u32, u32) {
        match self.player_mmdl() {
            Some(m) => (
                self.e.r16(m + 0x3C),
                self.e.r16(m + 0x40),
                self.e.r16(m + 0x18),
            ),
            None => (0, 0, 0),
        }
    }

    /// The PLAYER record's zone.
    pub fn zone(&mut self) -> u32 {
        let p = self.e.cv(0x0201_20F8, &[self.gs]);
        self.e.r16(p)
    }

    /// The current GMEVENT (0 = none).
    fn event(&mut self) -> u32 {
        self.e.r32(self.gs + 0x18)
    }

    fn event_proc(&mut self) -> u32 {
        let ev = self.event();
        if ev == 0 {
            0
        } else {
            self.e.r32(ev + 4)
        }
    }

    /// The player's grid move state (`PLAYER GRID +0`: 0 standing).
    fn grid_state(&mut self) -> u32 {
        let fm = self.fm();
        let pl = self.e.r32(fm + 0x8C);
        let grid = self.e.r32(pl + 8);
        self.e.r32(grid)
    }

    /// The trace line of the frame: `f<frame> k<keys> z<zone> p<x>,<z>,<dir> e<proc>:<seq>[ s<vm pc>/<status>/<wait>..]
    /// m[<obj id>:<x>,<z>,<dir> ..] r<MathRand state>`.
    fn trace_line(&mut self, held: u32) -> String {
        let (x, z, d) = self.player();
        let zone = self.zone();
        let ev = self.event();
        let mut s = format!("f{} k{held:03x} z{zone} p{x},{z},{d}", self.frame);
        if ev != 0 {
            let proc_ = self.e.r32(ev + 4);
            s += &format!(" e{proc_:08x}:{}", self.e.r32(ev + 8));
            if proc_ == SCRIPT_EVENT {
                let w = self.e.r32(ev + 0xC);
                let sys = self.e.r32(w + 4);
                for k in 1..4 {
                    let vm = self.e.r32(sys + 4 * k);
                    if vm != 0 {
                        let (pc, st, wait) = (
                            self.e.r32(vm + 0x14),
                            self.e.r8(vm + 0xD),
                            self.e.r32(vm + 0x10),
                        );
                        s += &format!(" s{pc:08x}/{st}/{wait:08x}");
                    }
                }
            }
        }
        let fm = self.fm();
        if fm != 0 {
            let sys = self.e.r32(fm + 0x38);
            if sys != 0 {
                let n = self.e.r16(sys + 4);
                let arr = self.e.r32(sys + 0x18);
                s += " m[";
                for i in 0..n {
                    let m = arr + 0x100 * i;
                    if self.e.r32(m) & 1 != 0 {
                        s += &format!(
                            "{}:{},{},{} ",
                            self.e.r16(m + 8),
                            self.e.r16(m + 0x3C),
                            self.e.r16(m + 0x40),
                            self.e.r16(m + 0x18)
                        );
                    }
                }
                s += "]";
            }
        }
        let rs = self.e.r32(GFL_SYS + 0x10);
        s += &format!(" r{:08x}{:08x}", self.e.r32(rs + 4), self.e.r32(rs));
        s
    }

    /// One frame (one main-loop iteration) with the keys `held`: the gfl UI update reads the hardware keys
    /// (`02034F5C`), the game system runs (`02011FCC`: events, the field proc), then the main loop's after-VBlank
    /// step (`0200567C`: the main task system).
    pub fn step(&mut self, held: u32) {
        let e = &mut self.e;
        e.host.held = held;
        e.host.frame += 1;
        self.frame += 1;
        // KEYINPUT (active low), and X / Y (bits 10 / 11) plus the released debug button (bit 13) in the ARM7's
        // shared word, which the UI ORs with KEYINPUT
        e.w16(0x0400_0130, !held & 0x3FF);
        e.w16(0x02FF_FFA8, (!held & 0x0C00) | 0x2000);
        e.cv(0x0203_4F5C, &[]);
        e.cv(0x0201_1FCC, &[self.gs]);
        e.cv(0x0200_567C, &[]);
        let line = self.trace_line(held);
        if let Some(f) = &mut self.out {
            use std::io::Write;
            writeln!(f, "{line}").unwrap();
        }
        self.trace.push(line);
        // `F12_DUMP=<frame>,<file>`: main RAM 0x02000000..0x02700000 after that frame (diagnostics; field_diff.py
        // --dump writes the ROM run's)
        if let Ok(v) = std::env::var("F12_DUMP") {
            if let Some((f, path)) = v.split_once(',') {
                if f.parse::<u32>().ok() == Some(self.frame) {
                    let bytes: Vec<u8> = (0x0200_0000..0x0270_0000u32)
                        .step_by(4)
                        .flat_map(|a| self.e.r32(a).to_le_bytes())
                        .collect();
                    std::fs::write(path, bytes).expect("F12_DUMP");
                }
            }
        }
        let proc_ = self.event_proc();
        if proc_ != self.last_event {
            self.last_event = proc_;
            if proc_ == BATTLE_EVENT && self.encounter.is_none() {
                self.record_encounter();
            }
        }
    }

    fn idle(&mut self) -> bool {
        self.event() == 0 && self.player_mmdl().is_some() && self.grid_state() == 0
    }

    /// Frames until nothing runs (no event, the player standing) for two frames; while a script runs, A is
    /// pressed every 16 frames (its message pages).
    pub fn wait_idle(&mut self, max: u32) {
        let mut calm = 0;
        for i in 0..max {
            let k = if self.event_proc() == SCRIPT_EVENT && i % 16 < 2 {
                KEY_A
            } else {
                0
            };
            self.step(k);
            calm = if self.idle() { calm + 1 } else { 0 };
            if calm >= 2 {
                return;
            }
        }
        panic!("F12: the field does not settle (frame {})", self.frame);
    }

    /// Walk `n` tiles in direction `key` (held until the player reached the tile), then settle.
    pub fn walk(&mut self, key: u32, n: u32) {
        let (x0, z0, _) = self.player();
        let (tx, tz) = match key {
            KEY_UP => (x0, z0 - n),
            KEY_DOWN => (x0, z0 + n),
            KEY_LEFT => (x0 - n, z0),
            _ => (x0 + n, z0),
        };
        for _ in 0..40 * n + 40 {
            if self.event() != 0 {
                break;
            }
            let (x, z, _) = self.player();
            if (x, z) == (tx, tz) {
                break;
            }
            self.step(key);
        }
        if self.encounter.is_none() && self.event() == 0 {
            self.wait_idle(3000);
        }
    }

    /// Turn to `key`'s direction without moving (a two-frame tap).
    pub fn face(&mut self, key: u32) {
        self.step(key);
        self.step(key);
        self.wait_idle(100);
    }

    /// Press A, then keep pressing A every 16 frames while an event runs (message pages); settle.
    pub fn talk(&mut self) {
        self.step(KEY_A);
        self.step(KEY_A);
        for i in 0..4000 {
            if self.event() == 0 {
                break;
            }
            self.step(if i % 16 < 2 { KEY_A } else { 0 });
        }
        self.wait_idle(3000);
    }

    /// Run frames (keys `held`) until the zone is `zone` and the field settled (a warp).
    pub fn until_zone(&mut self, held: u32, zone: u32) {
        let from = self.zone();
        for _ in 0..2000 {
            if self.zone() == zone && self.event() == 0 && self.player_mmdl().is_some() {
                break;
            }
            let k = if self.event() == 0 { held } else { 0 };
            self.step(k);
        }
        self.wait_idle(3000);
        let (x, z, _) = self.player();
        assert_eq!(self.zone(), zone, "F12: no warp to zone {zone}");
        self.warps.push((from, zone, x, z));
    }

    fn record_encounter(&mut self) {
        let ev = self.event();
        let w = self.e.r32(ev + 0xC);
        let setup = self.e.r32(w + 8);
        let party = self.e.cv(0x0201_313C, &[setup, 1]);
        let mon = self.e.cv(0x0201_AB28, &[party, 0]);
        let species = self.e.cv(0x0201_7E1C, &[mon, 5, 0]) & 0xFFFF;
        let level = self.e.cv(0x0201_7E1C, &[mon, 0x9E, 0]);
        let setup_type = self.e.r32(setup);
        let setup_flags = self.e.r16(setup + 0x82);
        let zone = self.zone();
        let (x, z, _) = self.player();
        self.encounter = Some(Encounter {
            frame: self.frame,
            species,
            level,
            setup_type,
            setup_flags,
            zone,
            at: (x, z),
        });
    }

    /// Walk up and down in the grass until the encounter starts (at most `max` turns).
    pub fn grass_until_encounter(&mut self, max: u32) {
        for i in 0..max {
            if self.encounter.is_some() {
                return;
            }
            let k = if i % 2 == 0 { KEY_UP } else { KEY_DOWN };
            self.walk(k, 3);
            // the encounter event runs: a few frames until its battle event is the current one
            for _ in 0..200 {
                if self.encounter.is_some() || self.event() == 0 {
                    break;
                }
                self.step(0);
            }
        }
    }

    /// The hash of trace lines `a..b`.
    pub fn hash(&self, a: usize, b: usize) -> u64 {
        fnv64(self.trace[a..b].join("\n").as_bytes())
    }
}

/// Runs the whole session; returns it.
fn play(e: Env) -> Session {
    let mut s = Session::new(e);
    // the continue event, the season display, the first frames of the field
    s.wait_idle(2000);
    s.phase("walk");
    s.walk(KEY_UP, 8);
    s.walk(KEY_LEFT, 8);
    s.phase("talk");
    s.face(KEY_UP);
    s.talk();
    s.phase("warp");
    s.walk(KEY_RIGHT, 2);
    s.walk(KEY_DOWN, 7);
    s.walk(KEY_RIGHT, 3);
    s.until_zone(KEY_UP, 390);
    s.until_zone(KEY_DOWN, 389);
    s.phase("grass");
    s.walk(KEY_RIGHT, 5);
    s.walk(KEY_UP, 26);
    s.grass_until_encounter(40);
    s
}

#[test]
fn f12_smoke_zone_cache() {
    let Some(mut e) = env() else { return };
    for ov in super::FIELD_OVERLAYS {
        e.load_overlay(ov);
    }
    // zone cache through the zonedata port; the archive layer runs as original code over the FS host
    e.cv(0x0201_39B0, &[0]);
    assert_eq!(e.cv(0x0201_3B44, &[389]), 0, "Nuvema Town: matrix 0");
    assert_eq!(
        e.cv(0x0201_3B5C, &[389]),
        778,
        "Nuvema Town: script file 778"
    );
}

#[test]
fn f12_field_session() {
    let Some(e) = env() else { return };
    let t = std::time::Instant::now();
    let s = play(e);
    eprintln!(
        "F12: {} frames, {} interpreted steps, {:?}",
        s.frame,
        s.e.stats.steps,
        t.elapsed()
    );
    let mut units = String::new();
    for p in &s.e.host.printed {
        eprintln!("F12 printed f{} {}: {:?}", p.frame, p.window, p.text());
        units += &format!("f{} {}:", p.frame, p.window);
        for u in &p.units {
            units += &format!(" {u:04x}");
        }
        units.push('\n');
    }
    // `F12_TEXT_OUT`: the printed STRBUFs' units (tools/check-field-diff.sh compares them with the ROM run's)
    if let Ok(path) = std::env::var("F12_TEXT_OUT") {
        std::fs::write(path, &units).expect("F12_TEXT_OUT");
    }
    eprintln!("F12 warps {:?}", s.warps);
    eprintln!("F12 encounter {:?}", s.encounter);
    let mut bounds: Vec<usize> = s.phases.iter().map(|p| p.1).collect();
    bounds.push(s.trace.len());
    let mut got: Vec<(&str, u64)> = Vec::new();
    for (i, (name, a)) in s.phases.iter().enumerate() {
        got.push((name, s.hash(*a, bounds[i + 1])));
    }
    got.push(("trace", s.hash(0, s.trace.len())));
    let text: Vec<String> = s.e.host.printed.iter().map(|p| p.text()).collect();
    got.push(("text", fnv64(text.join("\n").as_bytes())));
    for (k, h) in &got {
        eprintln!("F12 hash {k}: {h:016x}");
    }

    // what the session did
    let talk = text
        .iter()
        .position(|t| t.starts_with("Cheren: I'm sorry, but could you\ngo to Bianca's house?"))
        .expect("F12: the NPC (Cheren, Nuvema Town OBJ 4) did not answer");
    assert_eq!(s.e.host.printed[talk].window, "balloon");
    assert_eq!(
        s.warps,
        vec![(389, 390, 6, 10), (390, 389, 782, 749)],
        "F12: the warps into the player's house and back"
    );
    let enc = s.encounter.clone().expect("F12: no wild encounter");
    assert_eq!(
        (enc.zone, enc.at, enc.species, enc.level, enc.setup_type),
        (317, (787, 722), 504, 2, 0),
        "F12: the Route 1 encounter (a level-2 Patrat, single wild battle)"
    );
    for (k, h) in RECORDED {
        let g = got.iter().find(|x| x.0 == *k).unwrap().1;
        assert_eq!(
            format!("{g:016x}"),
            *h,
            "F12: hash of {k} changed (F12_TRACE_OUT=<file> writes the trace)"
        );
    }
}

/// The hashes of the session's trace (by phase, the whole trace, the printed text), recorded from a run of this
/// test; a change means the ported code (or the integration) behaves differently. The original code under Unicorn
/// (`tools/check-field-diff.sh`, `tools/rom-diff/field_diff.py`) gives the same trace line by line ("trace") and
/// the same printed text.
const RECORDED: &[(&str, &str)] = &[
    ("boot", "0dea84d6a5676a3c"),
    ("walk", "dd63e62279d9b35f"),
    ("talk", "523c4ab8080c4506"),
    ("warp", "ea28837a1798d870"),
    ("grass", "218b45d983fe20f9"),
    ("trace", "6491ded865402e16"),
    ("text", "4898972e7af11cec"),
];

#[test]
fn f12_ports_fresh() {
    // every verified port of the generator's sources is dispatched or listed as unmapped, and nothing else
    use super::ports::{PORTS, SOURCES, UNMAPPED};
    let mut want: Vec<u32> = SOURCES.iter().flat_map(|s| s.iter().copied()).collect();
    want.sort_unstable();
    want.dedup();
    let mut have: Vec<u32> = PORTS
        .iter()
        .map(|p| p.0)
        .chain(UNMAPPED.iter().copied())
        .collect();
    have.sort_unstable();
    have.dedup();
    assert_eq!(
        want, have,
        "F12: ports.rs is stale: run python3 -I tools/field-integ/gen_ports.py"
    );
}
