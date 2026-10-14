//! The B16 battle.

use super::Env;
use crate::server::server1;

use super::data::{Archives, PtHooks};
use black_pokemon::poke_tool::{party_get_value, party_set_value};

/// A level-`level` POKEMON_PARAM of `species` (0xDC bytes, encrypted) built with the verified
/// `black-pokemon` accessors: fixed PID / OT id / IVs, the first ability, EXP for the level, the
/// last four level-up moves of the learnset with full PP, stats from `recalc_stats`.
pub fn make_pokemon(arc: &std::rc::Rc<Archives>, species: u32, level: u32, pid: u32) -> Vec<u8> {
    let mut m = vec![0u8; 0xDC];
    black_pokemon::create::clear_pokemon(&mut m);
    let mut h = PtHooks { arc: arc.clone() };
    let set = |m: &mut Vec<u8>, h: &mut PtHooks, id: u32, v: u32| party_set_value(m, id, v, h);
    set(&mut m, &mut h, 0, pid);
    set(&mut m, &mut h, 5, species);
    set(&mut m, &mut h, 7, 0x0001_2345);
    set(&mut m, &mut h, 8, arc.exp_for_level(species, 0, level));
    set(&mut m, &mut h, 9, arc.personal_param(species, 0, 0x16));
    set(&mut m, &mut h, 0xA, arc.personal_param(species, 0, 0x18));
    set(&mut m, &mut h, 0xC, 2);
    for id in 0x46..=0x4B {
        set(&mut m, &mut h, id, 15);
    }
    set(&mut m, &mut h, 0x70, pid % 25);
    let moves: Vec<u32> = arc
        .learnset(species)
        .into_iter()
        .filter(|&(_, lv)| lv <= level)
        .map(|(mv, _)| mv)
        .collect();
    let moves = &moves[moves.len().saturating_sub(4)..];
    for (i, &mv) in moves.iter().enumerate() {
        set(&mut m, &mut h, 0x36 + i as u32, mv);
        set(&mut m, &mut h, 0x3A + i as u32, arc.max_pp(mv, 0));
    }
    set(&mut m, &mut h, 0x9E, level);
    let mut cache = black_pokemon::personal::RecordCache {
        form: 0xFFFF,
        species: 0xFFFF,
        record: [0; 0x3C],
    };
    black_pokemon::personal::recalc_stats(&mut m, &mut cache, &mut h);
    assert_eq!(party_get_value(&mut m, 5, 0, &mut h), species);
    m
}

/// A PokeParty with the given members.
pub(super) fn make_party(env: &mut Env, mons: &[Vec<u8>]) -> u32 {
    let p = env
        .objs
        .borrow_mut()
        .alloc(super::arm9::PARTY_SIZE, "PokeParty (setup)");
    super::arm9::party_init(env, p, 6);
    for m in mons {
        let tmp = env.objs.borrow_mut().alloc(0xDC, "tmp mon");
        env.objs.borrow_mut().write(tmp, m);
        super::arm9::party_add(env, p, tmp);
        env.objs.borrow_mut().free(tmp);
    }
    p
}

/// Builds the BATTLE_SETUP of a local wild single battle (type 0, rule 0) and the objects it
/// points at; returns its address.
pub(super) fn build_setup(env: &mut Env, player: u32, foe: u32) -> u32 {
    let mut o = env.objs.borrow_mut();
    let setup = o.alloc(0xF0, "BATTLE_SETUP");
    let config = o.alloc(4, "game config");
    o.w32(setup + 0x5C, config);
    o.w32(setup + 0x24, player);
    o.w32(setup + 0x28, foe);
    // the player's MyStatus (+0x34, 0x20 bytes: name, id, gender ...)
    let mystatus = o.alloc(0x20, "MyStatus (player)");
    o.w32(setup + 0x34, mystatus);
    for (i, c) in [0x0130u32, 0x0131, 0x0132, 0x0133, 0xFFFF]
        .iter()
        .enumerate()
    {
        o.w16(mystatus + 2 * i as u32, *c);
    }
    // the trainer id of the parties' original trainer (`make_pokemon`), gender 0
    o.w32(mystatus + 0x10, 0x0001_2345);
    // game records (+0x6C, record.c counters)
    let records = o.alloc(0x400, "game records");
    o.w32(setup + 0x6C, records);
    // game data (+0x58): its first word is the save data
    let save = o.alloc(0x20, "save data");
    let game = o.alloc(4, "game data");
    o.w32(game, save);
    o.w32(setup + 0x58, game);
    setup
}

/// One decoded command of a frame: `(id, argument words)`.
pub type Cmd = (u32, Vec<u32>);

/// Decodes a command frame the server sent (the bytes the client copies to its queue at +8) with the
/// ported reader (`021cd5e0`); `Err` when it is not well-formed (an unknown id, a read past the end,
/// or a frame that does not end on a command boundary).
pub fn decode_frame(data: &[u8]) -> Result<Vec<Cmd>, String> {
    use crate::server_flow::cmd_queue::{read_cmd, CMD_ARG_TABLE};
    let size = data.len();
    let mut q = vec![0u8; 8 + size + 0x40];
    q[..4].copy_from_slice(&(size as u32).to_le_bytes());
    q[8..8 + size].copy_from_slice(data);
    let mut cmds = Vec::new();
    loop {
        let cur = u32::from_le_bytes(q[4..8].try_into().unwrap()) as usize;
        if cur == size {
            return Ok(cmds);
        }
        if cur > size {
            return Err(format!("read past the end ({cur} > {size})"));
        }
        let id = u16::from_be_bytes([q[8 + cur], q[9 + cur]]) as usize;
        if id != 0x5F && (id >= CMD_ARG_TABLE.len() || id == 0) {
            return Err(format!("unknown command id {id:#x} at {cur}"));
        }
        let mut out = [0u8; 0x40];
        let id = read_cmd(&mut q, &mut out);
        if id == 0x5E {
            return Ok(cmds);
        }
        let args: Vec<u32> = (0..16)
            .map(|i| u32::from_le_bytes(out[4 * i..4 * i + 4].try_into().unwrap()))
            .collect();
        // command 0x33's player (`021d215c`) pops its target list (u16 a0 bytes) from the queue
        if id == 0x33 {
            let cur = u32::from_le_bytes(q[4..8].try_into().unwrap());
            q[4..8].copy_from_slice(&(cur + (args[0] & 0xFFFF)).to_le_bytes());
        }
        cmds.push((id, args));
    }
}

/// What one battle run produced: the HP trace (by poke id, one entry per change), the command
/// frames the server sent, and the overlay-96 functions called from outside the overlay.
pub struct Outcome {
    pub hp_log: Vec<Vec<(u8, u16)>>,
    pub frames: Vec<Vec<u8>>,
    pub ai_calls: std::collections::BTreeMap<u32, u32>,
}

/// One wild battle end to end — a level-5 Snivy (the player, scripted: move slot 0 every turn)
/// against a level-5 Patrat (the AI) — through the ported server, flow, client and handler code
/// (`proc_init`, `proc_main` until it reports the end, `proc_exit`). Checks: HP never exceeds the
/// maximum (it is unsigned: never negative), one side ends fainted, every command frame decodes.
/// `ai_mask` forces the AI client's script mask, `ai_interp` runs overlay 96 as original Thumb
/// code (B17); `log` prints the outcome.
fn run_battle(
    rom: std::rc::Rc<super::rom::Rom>,
    ai_mask: Option<u32>,
    ai_interp: bool,
    log: bool,
) -> Outcome {
    let mut env = Env::new(rom);
    env.ai_mask = ai_mask;
    env.ai_interp = ai_interp;
    let arc = env.arc.clone();
    // a level-5 Snivy (495) against a level-5 Patrat (504)
    let player = make_party(&mut env, &[make_pokemon(&arc, 495, 5, 0x1234_5678)]);
    let foe = make_party(&mut env, &[make_pokemon(&arc, 504, 5, 0x8765_4321)]);
    let setup = build_setup(&mut env, player, foe);
    env.setup = setup;
    let (proc, seq) = {
        let mut o = env.objs.borrow_mut();
        (o.alloc(0x20, "GameProc"), o.alloc(4, "proc seq"))
    };
    let mut hp_log: Vec<Vec<(u8, u16)>> = Vec::new();
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut frames = 0;
        loop {
            let m = env.r32(proc + 0x14);
            let mut mem = env.mem();
            if server1::proc_init(&mut env, &mut mem, proc, seq, setup, m) != 0 {
                break;
            }
            frames += 1;
            assert!(frames < 1000, "proc_init does not finish");
        }
        if log {
            eprintln!("B16: proc_init done after {frames} frames");
        }
        let m = env.r32(proc + 0x14);
        let mut main_frames = 0;
        loop {
            let tr = std::env::var("B16_TRACE")
                .ok()
                .and_then(|v| v.parse::<u32>().ok());
            env.trace = u32::from(tr == Some(main_frames));
            let mut mem = env.mem();
            let done = server1::proc_main(&mut env, &mut mem, m) != 0;
            // invariant: HP within [0, max] (u16, so never negative) every frame
            let hp: Vec<(u8, u16)> = env.mons.values().map(|b| (b.poke_id(), b.hp())).collect();
            for b in env.mons.values() {
                assert!(
                    b.hp() <= b.max_hp(),
                    "B16: HP {} above max {}",
                    b.hp(),
                    b.max_hp()
                );
            }
            if hp_log.last() != Some(&hp) {
                hp_log.push(hp);
            }
            if done {
                break;
            }
            main_frames += 1;
            assert!(main_frames < 2_000, "the battle does not end");
        }
        if log {
            eprintln!("B16: battle main loop ended after {main_frames} frames");
        }
        let mut exit_frames = 0;
        loop {
            let mut mem = env.mem();
            if server1::proc_exit(&mut env, &mut mem, proc, seq, m) != 0 {
                break;
            }
            exit_frames += 1;
            assert!(exit_frames < 100, "proc_exit does not finish");
        }
        if log {
            eprintln!("B16: proc_exit done after {exit_frames} frames");
        }
    }));
    if log || run.is_err() {
        for (n, k) in &env.notes {
            eprintln!("B16 note x{k}: {n}");
        }
    }
    if let Err(e) = run {
        eprintln!("B16 stack at the panic: {}", env.backtrace());
        std::panic::resume_unwind(e);
    }
    // ---- the outcome ----
    let ai: Vec<String> = env
        .ai_calls
        .iter()
        .map(|(a, n)| format!("{a:08x}x{n}"))
        .collect();
    if log {
        eprintln!("B16: AI ports called: {}", ai.join(" "));
        eprintln!("B16: player moves chosen: {:?}", env.choices);
        eprintln!("B16: HP by poke id over the battle: {hp_log:?}");
    }
    let last = hp_log.last().expect("no HP record");
    let side_down = |side: bool| {
        last.iter()
            .filter(|(id, _)| (*id >= 6) == side)
            .all(|&(_, hp)| hp == 0)
    };
    let (player_down, foe_down) = (side_down(false), side_down(true));
    // B18: with a forced script mask the battle may also end with neither side down (bit 11: the
    // wild foe flees; the original does the same, `b17_ai_scripts_match_original` compares them)
    let fled = ai_mask.is_some() && !player_down && !foe_down;
    assert!(
        player_down != foe_down || fled,
        "B16: exactly one side must end fainted ({last:?})"
    );
    if log {
        eprintln!(
            "B16: winner: {}",
            if foe_down {
                "player (Snivy)"
            } else {
                "foe (Patrat)"
            }
        );
    }
    // ---- the command stream ----
    assert!(!env.cmd_frames.is_empty(), "B16: no command frames");
    let mut total = 0;
    for (k, f) in env.cmd_frames.iter().enumerate() {
        let cmds = decode_frame(f)
            .unwrap_or_else(|e| panic!("B16: command frame {k} is malformed: {e} ({f:02x?})"));
        total += cmds.len();
        if log && (k < 6 || k + 2 >= env.cmd_frames.len()) {
            let ids: Vec<String> = cmds
                .iter()
                .map(|(id, a)| format!("{id:#04x}{:x?}", &a[..3]))
                .collect();
            eprintln!("B16 frame {k} ({} bytes): {}", f.len(), ids.join(" "));
        }
    }
    if log {
        eprintln!(
            "B16: {} command frames, {total} commands, all well-formed",
            env.cmd_frames.len()
        );
    }
    Outcome {
        hp_log,
        frames: env.cmd_frames.clone(),
        ai_calls: env.ai_calls.clone(),
    }
}

/// The ROM, or `None` (skip) when it is missing and `REQUIRE_ROM` is not 1.
pub(super) fn rom() -> Option<std::rc::Rc<super::rom::Rom>> {
    match super::rom::Rom::load() {
        Some(r) => Some(std::rc::Rc::new(r)),
        None => {
            assert!(
                std::env::var("REQUIRE_ROM").as_deref() != Ok("1"),
                "B16: ROM build / files missing"
            );
            eprintln!("B16: ROM build or files missing, skipped");
            None
        }
    }
}

/// B16: the battle above with the setup's AI (a wild battle: no AI scripts). B17: the opponent's
/// decisions come from the ai1 ports (create / start / think / chosen move / chosen target; no
/// overlay-96 code runs in the interpreter).
#[test]
fn b16_single_battle() {
    let Some(rom) = rom() else { return };
    let out = run_battle(rom, None, false, true);
    for f in [
        0x0218_7F00,
        0x0218_80BC,
        0x0218_7FB0,
        0x0218_8134,
        0x0218_8140,
    ] {
        assert!(
            out.ai_calls.contains_key(&f),
            "B17: AI entry point {f:#x} not run as a port"
        );
    }
}

/// B17: the same battle with trainer-AI scripts (the AI client's mask forced): the ai1 ports
/// (every command handler the scripts reach, called by the interpreted vm.c step) produce the same
/// battle — HP trace and every command frame — as the original overlay-96 code in the Thumb
/// interpreter. Masks: every script bit whose battle runs end to end in the integration (B18 added
/// bits 0, 1, 4 and 11; bit 12's 12-byte script makes vm.c read past its end in the original too,
/// bits 14 / 15 have no script in archive 0xAB; see the black-battle README "B16 integration").
#[test]
fn b17_ai_scripts_match_original() {
    let Some(rom) = rom() else { return };
    let masks = [
        0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x2000, 0x2C,
        0x7EC, 0x27EC, 0x2FFF,
    ];
    let mut handlers = std::collections::BTreeSet::<u32>::new();
    for mask in masks {
        let port = run_battle(rom.clone(), Some(mask), false, false);
        let orig = run_battle(rom.clone(), Some(mask), true, false);
        assert_eq!(
            port.hp_log, orig.hp_log,
            "B17: AI mask {mask:#x}: HP trace differs"
        );
        assert_eq!(
            port.frames, orig.frames,
            "B17: AI mask {mask:#x}: command frames differ"
        );
        handlers.extend(
            port.ai_calls
                .keys()
                .filter(|&&a| crate::ai::CMD_TABLE.contains(&(a | 1))),
        );
        eprintln!(
            "B17: AI mask {mask:#x}: {} frames, HP trace {:?}",
            port.frames.len(),
            port.hp_log.last()
        );
    }
    eprintln!(
        "B17: {} distinct command handlers ran as ports",
        handlers.len()
    );
    assert!(
        handlers.len() >= 20,
        "B17: the scripts reached too few handlers"
    );
}
