//! B19: the Unicorn differential of whole battles. `tools/rom-diff/battle_diff.py` runs each
//! battle of [`BATTLES`] entirely in the original code (ARM9 main + overlays 93-96 + 231 under
//! Unicorn, with a boot / OS / heap / file-system / view shim) and writes a trace; [`trace`] runs the
//! same battle through the integration (the ports) and writes the same trace format. The test
//! compares the hash of the trace with the one the Python side recorded ([`RECORDED`]), and, when
//! `B19_ROM_TRACES` names the directory the Python side wrote, the traces line by line (first
//! difference reported). `tools/check-battle-diff.sh` does both.
//!
//! Trace format (one item per line):
//! - `b19 NAME`
//! - `parties FNV`: FNV-1a 64 of the two PokeParty images the setup points at (player, foe);
//! - `init_frames N`: proc_init calls before it reports the end;
//! - `f K hp ID:HP/MAX ... rng X`: after the K-th proc_main call, the server pokecon's battle
//!   Pokemon (`M + 0xC8`, slots 0..24) and the battle RNG state (`0x021F6368`, `BattleCalc::rng.x`);
//! - `main_frames K`, `exit_frames N`, `end hp ...` (the last `f` line's data), `choices MOVE ...`
//!   (the scripted player's moves), `switches SLOT ...` (the party slots it chose);
//! - `frame I at K len N HEX`: the I-th command frame the server sent (adapter message 8, a
//!   repeat of the previous frame not counted), during main frame K (-1 = proc_init, -2 =
//!   proc_exit), and its commands decoded (`  cmd ID W0 .. W15`; the Python side decodes with the
//!   original reader 021cd5e0, this side with its port);
//! - `mt_calls N`: calls of the system Mersenne Twister (`0203F0A8`).

use super::tests::{build_setup, decode_frame, make_party, make_pokemon, rom};
use super::Env;
use crate::server::server1;
use std::fmt::Write as _;

/// One scripted battle.
pub struct Scenario {
    pub name: &'static str,
    /// The player's party: `(species, level, pid)` (built with `tests::make_pokemon`).
    pub party0: &'static [(u32, u32, u32)],
    /// The foe's party (a wild battle: the AI client).
    pub party1: &'static [(u32, u32, u32)],
    /// The player's move slot per move-menu answer (`Env::script`; the last one repeats).
    pub script: &'static [u32],
    /// The AI client's script mask (`Env::ai_mask`, `021b9fbc`), `None` = the setup's (wild: 0).
    pub ai_mask: Option<u32>,
    /// The player's party slot per party-list opening (`Env::switch_script`; the last repeats).
    pub switch_to: &'static [u32],
}

/// The battles both sides run (mirrored by `BATTLES` in tools/rom-diff/battle_diff.py).
pub const BATTLES: &[Scenario] = &[
    // B16: Snivy lv5 (Tackle every turn) vs a wild Patrat lv5
    Scenario {
        name: "b16",
        party0: &[(495, 5, 0x1234_5678)],
        party1: &[(504, 5, 0x8765_4321)],
        script: &[0],
        ai_mask: None,
        switch_to: &[1],
    },
    // the same with trainer-AI scripts (mask 7: the basic / strong / expert scripts, B17)
    Scenario {
        name: "trainer_ai",
        party0: &[(495, 5, 0x1234_5678)],
        party1: &[(504, 5, 0x8765_4321)],
        script: &[0],
        ai_mask: Some(0x7),
        switch_to: &[1],
    },
    // status moves: Snivy uses Leer (slot 1) twice, then Tackle
    Scenario {
        name: "status",
        party0: &[(495, 5, 0x1234_5678)],
        party1: &[(504, 5, 0x8765_4321)],
        script: &[1, 1, 0],
        ai_mask: None,
        switch_to: &[1],
    },
    // a switch: Snivy (Tackle) faints in turn 3, the player sends out Tepig lv5 (party slot 1),
    // which fights on (Tackle)
    Scenario {
        name: "switch",
        party0: &[(495, 5, 0x1234_5678), (498, 5, 0x0BAD_BEEF)],
        party1: &[(504, 5, 0x8765_4321)],
        script: &[0],
        ai_mask: None,
        switch_to: &[1],
    },
];

// FNV-1a 64 of each battle's ROM trace (`battle_diff.py` output). Written by
// `tools/check-battle-diff.sh --record` only, when every Rust trace equals the ROM trace.
// b19-recorded-begin
pub const RECORDED: &[(&str, &str)] = &[
    ("b16", "0c59ef55eb2a0a1e"),
    ("trainer_ai", "1de86f0e1b3c06db"),
    ("status", "a970ab0328b561b1"),
    ("switch", "aae296fd4f8ea341"),
];
// b19-recorded-end

fn fnv64(data: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in data {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The `hp ... rng ...` part of a frame line.
fn hp_line(env: &Env, m: u32) -> String {
    let mut s = String::from("hp");
    for i in 0..24 {
        let b = env.r32(m + 0xC8 + 0x84 + 4 * i);
        if b != 0 {
            let mon = env
                .mons
                .get(&b)
                .unwrap_or_else(|| panic!("B19: pokecon slot {i} {b:#x} is not a battle Pokemon"));
            write!(s, " {}:{}/{}", mon.poke_id(), mon.hp(), mon.max_hp()).unwrap();
        }
    }
    write!(s, " rng {:016x}", env.calc.rng.x).unwrap();
    s
}

/// Runs `sc` through the integration and returns its trace.
pub fn trace(rom: std::rc::Rc<super::rom::Rom>, sc: &Scenario) -> String {
    let mut env = Env::new(rom);
    env.ai_mask = sc.ai_mask;
    env.script = sc.script.to_vec();
    env.switch_script = sc.switch_to.to_vec();
    let arc = env.arc.clone();
    let mons = |p: &[(u32, u32, u32)]| -> Vec<Vec<u8>> {
        p.iter()
            .map(|&(s, l, pid)| make_pokemon(&arc, s, l, pid))
            .collect()
    };
    let (m0, m1) = (mons(sc.party0), mons(sc.party1));
    let player = make_party(&mut env, &m0);
    let foe = make_party(&mut env, &m1);
    let parties = {
        let n = super::arm9::PARTY_SIZE as usize;
        let mut b = vec![0u8; 2 * n];
        env.objs.borrow_mut().read(player, &mut b[..n]);
        env.objs.borrow_mut().read(foe, &mut b[n..]);
        fnv64(&b)
    };
    let setup = build_setup(&mut env, player, foe);
    env.setup = setup;
    let (proc, seq) = {
        let mut o = env.objs.borrow_mut();
        (o.alloc(0x20, "GameProc"), o.alloc(4, "proc seq"))
    };
    let mut out = format!("b19 {}\nparties {parties:016x}\n", sc.name);
    // the main frame each command frame was sent in
    let mut sent_at: Vec<i32> = Vec::new();
    let mut mark = |env: &Env, at: i32| {
        while sent_at.len() < env.cmd_frames.len() {
            sent_at.push(at);
        }
    };
    let mut n = 0;
    loop {
        let m = env.r32(proc + 0x14);
        let mut mem = env.mem();
        let done = server1::proc_init(&mut env, &mut mem, proc, seq, setup, m) != 0;
        mark(&env, -1);
        if done {
            break;
        }
        n += 1;
        assert!(n < 1000, "B19: proc_init does not finish");
    }
    writeln!(out, "init_frames {n}").unwrap();
    let m = env.r32(proc + 0x14);
    let mut frame = 0;
    // diagnostics: `B19_TRACE=K` prints every dispatched call of main frame K
    let trace_frame = std::env::var("B19_TRACE")
        .ok()
        .and_then(|v| v.parse::<i32>().ok());
    let last = loop {
        env.trace = u32::from(trace_frame == Some(frame));
        let mut mem = env.mem();
        let done = server1::proc_main(&mut env, &mut mem, m) != 0;
        mark(&env, frame);
        let line = hp_line(&env, m);
        writeln!(out, "f {frame} {line}").unwrap();
        if done {
            break line;
        }
        frame += 1;
        assert!(frame < 2000, "B19: the battle does not end");
    };
    writeln!(out, "main_frames {frame}").unwrap();
    let mut n = 0;
    loop {
        let mut mem = env.mem();
        let done = server1::proc_exit(&mut env, &mut mem, proc, seq, m) != 0;
        mark(&env, -2);
        if done {
            break;
        }
        n += 1;
        assert!(n < 100, "B19: proc_exit does not finish");
    }
    writeln!(out, "exit_frames {n}").unwrap();
    writeln!(out, "end {last}").unwrap();
    let ch: Vec<String> = env.choices.iter().map(|c| c.to_string()).collect();
    writeln!(out, "choices {}", ch.join(" ")).unwrap();
    let sw: Vec<String> = env.switches.iter().map(|c| c.to_string()).collect();
    writeln!(out, "switches {}", sw.join(" ")).unwrap();
    for (k, f) in env.cmd_frames.iter().enumerate() {
        let hex: String = f.iter().map(|b| format!("{b:02x}")).collect();
        writeln!(out, "frame {k} at {} len {} {hex}", sent_at[k], f.len()).unwrap();
        let cmds = decode_frame(f).unwrap_or_else(|e| panic!("B19: frame {k}: {e}"));
        for (id, args) in cmds {
            let a: Vec<String> = args.iter().map(|w| format!("{w:x}")).collect();
            writeln!(out, "  cmd {id:02x} {}", a.join(" ")).unwrap();
        }
    }
    let mt = env.counts.get(&0x0203_F0A8).copied().unwrap_or(0);
    writeln!(out, "mt_calls {mt}").unwrap();
    out
}

/// B19: every battle of [`BATTLES`] gives the same trace in the ports (integration) as in the
/// original code under Unicorn (`battle_diff.py`, recorded in [`RECORDED`]).
#[test]
fn b19_battles_match_rom() {
    let Some(rom) = rom() else { return };
    let dir = std::env::var("B19_ROM_TRACES").ok();
    let out_dir = std::env::var("BLK_OUT")
        .ok()
        .map(|d| std::path::PathBuf::from(d).join("b19"));
    let mut bad = Vec::new();
    for sc in BATTLES {
        let t = trace(rom.clone(), sc);
        let h = format!("{:016x}", fnv64(t.as_bytes()));
        if let Some(d) = &out_dir {
            let _ = std::fs::create_dir_all(d);
            let _ = std::fs::write(d.join(format!("{}.rust.trace", sc.name)), &t);
        }
        if let Some(d) = &dir {
            let p = std::path::Path::new(d).join(format!("{}.trace", sc.name));
            match std::fs::read_to_string(&p) {
                Ok(rom_t) if rom_t != t => {
                    let (i, (a, b)) = rom_t
                        .lines()
                        .chain(std::iter::repeat("<end>"))
                        .zip(t.lines().chain(std::iter::repeat("<end>")))
                        .enumerate()
                        .find(|(_, (a, b))| a != b)
                        .unwrap();
                    eprintln!(
                        "B19 {}: first difference at line {}:\n  rom:  {a}\n  rust: {b}",
                        sc.name,
                        i + 1
                    );
                    bad.push(sc.name);
                }
                Ok(_) => eprintln!("B19 {}: trace equal to the ROM trace ({h})", sc.name),
                Err(_) => eprintln!("B19 {}: no ROM trace {}", sc.name, p.display()),
            }
        }
        let rec = RECORDED.iter().find(|r| r.0 == sc.name).map(|r| r.1);
        if rec != Some(h.as_str()) {
            eprintln!(
                "B19 {}: trace hash {h}, recorded {}",
                sc.name,
                rec.unwrap_or("none")
            );
            if !bad.contains(&sc.name) {
                bad.push(sc.name);
            }
        }
    }
    assert!(
        bad.is_empty(),
        "B19: battles differing from the ROM: {bad:?}"
    );
}
