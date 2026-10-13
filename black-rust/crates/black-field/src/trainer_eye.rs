//! `event_trainer_eye.c` (overlay 21, `0x021AE0CC..0x021AE948`): trainer line-of-sight spotting and the approach
//! task. The field event check (`FUN_021AE0CC`, called from `fieldmap_event`) walks every MMDL in use, keeps the
//! trainers (event type -> `fldmmdl::kind_434` = 1, or 2 for a trainer that looks in all four directions) that are not
//! defeated, tests the straight line from the trainer to the player within the sight range (header param 0) and checks
//! that every tile in between is free; the hits become EYE HIT records for the trainer script (0xE20). The approach
//! itself (turn, "!" balloon, walk up, the player turns round) is an 18-state TASK driven by a GMEVENT.
//!
//! F4c-spec specced all 38 functions (`tools/rom-diff/specs/black-field/trainer_eye.spec`, whose header documents the
//! sight algorithm, the EYE HIT record and the TASK); F4c ported them. MMDL fields are read only through the fldmmdl
//! getters (F4a ports, `force` stubs in the spec), so the ports reach them, the script system, the field-map glue and
//! the effect-encount tile test through [`FieldEnv::call`] by address; the "!" balloon (`fldeff_gyoe`) and the walk /
//! turn animations (`fldmmdl_acmd` codes) stay behind the same seam (D3). Callees inside the area are Rust calls; the
//! positions and records the original keeps on its stack are [`Mem::push_local`] windows (or Rust buffers when only
//! stubs see them). Directions: 0 up, 1 down, 2 left, 3 right.
//!
//! Grid positions are `s16[3]` = x, y (height), z (`FUN_0216D828`, MMDL `+0x3C..+0x40`).
//!
//! EYE HIT record (0x18): `+0` MMDL, `+4` distance, `+8` dir, `+0xC` script id, `+0x10` trainer id, `+0x14` kind (0 one
//! trainer, 1 double-battle pair, 2 two separate trainers at once). TASK (0x38, [`task_new`]): `+0` state (0..=17, ROM
//! `0x021D9348`), `+4` done, `+8..+0x1F` the EYE HIT record, `+0x20` 0, `+0x24` slot index (0 / 1), `+0x28` frame
//! counter, `+0x2C` the "!" effect, `+0x30` fieldmap, `+0x34` FIELD_PLAYER.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_E0CC,
    0x021A_E198,
    0x021A_E1FC,
    0x021A_E288,
    0x021A_E348,
    0x021A_E360,
    0x021A_E394,
    0x021A_E3C8,
    0x021A_E3FC,
    0x021A_E430,
    0x021A_E454,
    0x021A_E498,
    0x021A_E4B4,
    0x021A_E4E0,
    0x021A_E524,
    0x021A_E550,
    0x021A_E56C,
    0x021A_E58C,
    0x021A_E5A8,
    0x021A_E5BC,
    0x021A_E5D4,
    0x021A_E5EC,
    0x021A_E604,
    0x021A_E630,
    0x021A_E650,
    0x021A_E66C,
    0x021A_E6E0,
    0x021A_E708,
    0x021A_E724,
    0x021A_E72C,
    0x021A_E754,
    0x021A_E79C,
    0x021A_E7A8,
    0x021A_E884,
    0x021A_E890,
    0x021A_E8B4,
    0x021A_E8D4,
    0x021A_E930,
];

// callees in other units / areas (the spec's `force` stubs), named by address
const GMEVENT_NEW: u32 = 0x0201_22C0; // GMEVENT create (gsys, parent, proc, work size)
const GMEVENT_WORK: u32 = 0x0201_24E8; // GMEVENT -> work
const GAMEDATA: u32 = 0x0201_20F4; // GAMESYS -> GAMEDATA
const EVENT_WORK: u32 = 0x0201_296C; // GAMEDATA -> EVENT WORK
const SCRIPT_EVENT: u32 = 0x0215_8768; // script_sys: script GMEVENT (gsys, script, MMDL, 1)
const STORE_EYE: u32 = 0x0215_A438; // script_sys: store EYE HIT record (event, slot, &record)
const TRAINER_ID: u32 = 0x0215_A460; // script_sys: script id -> trainer id
const BATTLE_TYPE: u32 = 0x0215_A4C8; // script_sys: trainer battle type
const DEFEATED: u32 = 0x0215_A4D8; // script_sys: trainer defeated (event work, trainer id)
const STEP_HITS: u32 = 0x0216_3994; // mmdl_move step_hits_here(m, x, y, z, dir)
const DIR_DX: u32 = 0x0216_41E8; // mmdl_move dir_dx(dir)
const DIR_DZ: u32 = 0x0216_41F8; // mmdl_move dir_dz(dir)
const DIR_BETWEEN: u32 = 0x0216_42B4; // mmdl_move dir_between(x0, z0, x1, z1)
const ACMD_CAN_SET: u32 = 0x0216_D3CC; // fldmmdl acmd_can_set(m)
const ACMD_SET: u32 = 0x0216_D410; // fldmmdl acmd_set(m, code)
const ACMD_DONE: u32 = 0x0216_D454; // fldmmdl acmd_done(m)
const ACMD_END: u32 = 0x0216_D480; // fldmmdl acmd_end(m) (result unused)
const MOVE_CODE: u32 = 0x0216_D5A0; // fldmmdl move_code(m)
const EVENT_TYPE: u32 = 0x0216_D5A8; // fldmmdl event_type(m)
const EVENT_ID: u32 = 0x0216_D5B8; // fldmmdl event_id(m) (the script id)
const SET_DIR_DISP: u32 = 0x0216_D5E0; // fldmmdl set_dir_disp(m, dir)
const DIR_DISP: u32 = 0x0216_D5F8; // fldmmdl dir_disp(m)
const PARAM: u32 = 0x0216_D63C; // fldmmdl param(m, i)
const SYS_OF: u32 = 0x0216_D67C; // fldmmdl sys_of(m)
const GRID: u32 = 0x0216_D828; // fldmmdl grid(m, &s16[3])
const GX: u32 = 0x0216_D83C; // fldmmdl gx(m)
const GZ: u32 = 0x0216_D86C; // fldmmdl gz(m)
const BIT10: u32 = 0x0216_DA5C; // fldmmdl move bit 0x10
const PAUSE: u32 = 0x0216_DAF8; // fldmmdl pause(m)
const UNPAUSE: u32 = 0x0216_DB04; // fldmmdl unpause(m)
const NEXT_IN_USE: u32 = 0x0216_DD1C; // fldmmdl iterator (sys, &m, &i)
const FIND_BY_ID: u32 = 0x0216_DE94; // fldmmdl find_by_id(sys, id)
const CHANGE_MOVE_CODE: u32 = 0x0216_E0E8; // fldmmdl change_move_code(m, code)
const KIND_434: u32 = 0x0216_E434; // fldmmdl kind_434(event type)
const FM_MMDLSYS: u32 = 0x0218_8C7C; // fieldmap -> MMDLSYS
const FM_GAMESYS: u32 = 0x0218_8C80; // fieldmap -> GAMESYS
const FM_PLAYER: u32 = 0x0218_8CB4; // fieldmap -> FIELD_PLAYER
const FM_FX_CTRL: u32 = 0x0218_8CD0; // fieldmap -> field-effect control
const FM_ENCOUNT: u32 = 0x0218_8CE8; // fieldmap -> effect-encount object
const PARTY_ABLE: u32 = 0x0218_B6AC; // fieldmap_event: party members able to battle (gsys)
const ACMD_CODE: u32 = 0x0219_7E7C; // fldmmdl_acmd: acmd code of (dir, kind)
const PLAYER_MMDL: u32 = 0x021A_3074; // field_player: FIELD_PLAYER -> MMDL
const ENC_TILE: u32 = 0x021A_A900; // effect_encount: is &pos the active effect-encount tile (enc, &pos)
const FX_DELETE: u32 = 0x021A_C154; // field_effect: delete
const GYOE_NEW: u32 = 0x021B_B528; // fldeff_gyoe: "!" balloon create (ctrl, m, 0, 0)
const GYOE_DONE: u32 = 0x021B_B5C8; // fldeff_gyoe: finished

/// The GMEVENT proc of [`approach_event_new`] (`FUN_021AE754`, Thumb bit set).
const APPROACH_PROC: u32 = 0x021A_E755;
/// Address of the string `"event_trainer_eye.c"` (the file argument of the allocation in [`task_new`]).
pub const FILE_TRAINER_EYE: u32 = 0x021D_E138;
/// The script of the trainer-eye event (`021AE930`).
pub const SCRIPT_TRAINER_EYE: u32 = 0xE20;

/// EYE HIT record size.
pub const EYE_HIT_SIZE: u32 = 0x18;
/// EYE HIT `+0x00`: the MMDL.
pub const HIT_MMDL: u32 = 0x00;
/// EYE HIT `+0x04`: the distance in tiles.
pub const HIT_DIST: u32 = 0x04;
/// EYE HIT `+0x08`: the direction the trainer looks in.
pub const HIT_DIR: u32 = 0x08;
/// EYE HIT `+0x0C`: the script id (MMDL event id).
pub const HIT_SCRIPT: u32 = 0x0C;
/// EYE HIT `+0x10`: the trainer id.
pub const HIT_TRAINER: u32 = 0x10;
/// EYE HIT `+0x14`: the kind (0 one trainer, 1 double-battle pair, 2 two separate trainers).
pub const HIT_KIND: u32 = 0x14;

/// TASK size.
pub const TASK_SIZE: u32 = 0x38;
/// TASK `+0x00`: the state (index of ROM `0x021D9348`, 0..=17).
pub const TASK_STATE: u32 = 0x00;
/// TASK `+0x04`: done (1 after state 17).
pub const TASK_DONE: u32 = 0x04;
/// TASK `+0x08`: the EYE HIT record (0x18 bytes).
pub const TASK_HIT: u32 = 0x08;
/// TASK `+0x20`: always 0.
pub const TASK_20: u32 = 0x20;
/// TASK `+0x24`: the EYE HIT slot index (0 / 1).
pub const TASK_INDEX: u32 = 0x24;
/// TASK `+0x28`: the frame counter of the wait states 9 and 13.
pub const TASK_COUNTER: u32 = 0x28;
/// TASK `+0x2C`: the "!" balloon field effect.
pub const TASK_EFFECT: u32 = 0x2C;
/// TASK `+0x30`: the fieldmap.
pub const TASK_FIELDMAP: u32 = 0x30;
/// TASK `+0x34`: the FIELD_PLAYER.
pub const TASK_PLAYER: u32 = 0x34;

const T_MMDL: u32 = TASK_HIT + HIT_MMDL;
const T_DIST: u32 = TASK_HIT + HIT_DIST;
const T_DIR: u32 = TASK_HIT + HIT_DIR;
const T_KIND: u32 = TASK_HIT + HIT_KIND;

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn c2<E: FieldEnv>(env: &mut E, addr: u32, a: u32, b: u32) -> u32 {
    env.callv(addr, &[a, b])
}

/// `FUN_0216D828(m, pos)`: the grid position of `m` into the 6 bytes at `pos` (a window of the memory).
fn grid<E: FieldEnv>(env: &mut E, m: u32, pos: u32) {
    env.call(GRID, &mut [CArg::V(m), CArg::Out(pos, 6)]);
}

/// The iterator step `FUN_0216DD1C(sys, &m, &i)` of the callers' `while` loops, on the original's stack words at
/// `m_at` / `i_at` (windows of the memory): `Some(m)` while it answers 1.
fn next_in_use<E: FieldEnv>(env: &mut E, sys: u32, m_at: u32, i_at: u32) -> Option<u32> {
    let more = env.call(
        NEXT_IN_USE,
        &mut [CArg::V(sys), CArg::Out(m_at, 4), CArg::P(i_at)],
    );
    (more == 1).then(|| env.r32(m_at))
}

/// C: `FUN_021AE0CC` (unlabeled routine at `0x021AE0CC`): the trainer-eye check of the field event loop (called from
/// `fieldmap_event` `02189C18`): the GMEVENT of the trainer script, or 0. A first [`search`] (none: 0); `type =
/// FUN_0215A4C8((u16) trainer id)` (battle type), `n = FUN_0218B6AC(gamesys)` (party members able to battle).
/// Type 0: `n >= 2` and a second [`search`] excluding the first trainer finds one -> both records, kind 2; else one
/// record, kind 0. Type 1 (needs `n >= 2`): the partner ([`partner`]) recorded with the first trainer's distance and
/// direction, both kind 1. Types 2 / 3 (need `n >= 3`): one record, kind 0. Others: 0. The event is [`script_event`];
/// the records go to it with `FUN_0215A438(event, slot, &record)`. (Unsigned compares on `n`.)
pub fn check<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    // the two EYE HIT records of the original's stack: rec2 at +0, rec at +0x18
    let s = env.mem().push_local(2 * EYE_HIT_SIZE);
    let (rec2, rec) = (s, s + EYE_HIT_SIZE);
    let mut ev = 0;
    if search(env, fm, 0, rec) == 1 {
        let tid = env.r32(rec + HIT_TRAINER) & 0xFFFF;
        let ty = c1(env, BATTLE_TYPE, tid);
        let gs = c1(env, FM_GAMESYS, fm);
        let n = c1(env, PARTY_ABLE, gs);
        let m = env.r32(rec + HIT_MMDL);
        // Some(kind) = store both records with that kind
        let pair = match ty {
            0 => {
                if n >= 2 && search(env, fm, m, rec2) != 0 {
                    ev = script_event(env, fm, m);
                    Some(2)
                } else {
                    ev = script_event(env, fm, m);
                    env.w32(rec + HIT_KIND, 0);
                    env.call(STORE_EYE, &mut [CArg::V(ev), CArg::V(0), CArg::In(rec, 24)]);
                    None
                }
            }
            1 => {
                if n >= 2 {
                    let tid = env.r32(rec + HIT_TRAINER) & 0xFFFF;
                    let p = partner(env, m, tid);
                    let dist = env.r32(rec + HIT_DIST);
                    let dir = env.r32(rec + HIT_DIR) & 0xFFFF;
                    record_fill(env, rec2, p, dist, dir);
                    ev = script_event(env, fm, m);
                    Some(1)
                } else {
                    None
                }
            }
            _ => {
                if ty.wrapping_sub(2) <= 1 && n >= 3 {
                    ev = script_event(env, fm, m);
                    env.w32(rec + HIT_KIND, 0);
                    env.call(STORE_EYE, &mut [CArg::V(ev), CArg::V(0), CArg::In(rec, 24)]);
                }
                None
            }
        };
        if let Some(kind) = pair {
            env.w32(rec + HIT_KIND, kind);
            env.call(STORE_EYE, &mut [CArg::V(ev), CArg::V(0), CArg::In(rec, 24)]);
            env.w32(rec2 + HIT_KIND, kind);
            env.call(
                STORE_EYE,
                &mut [CArg::V(ev), CArg::V(1), CArg::In(rec2, 24)],
            );
        }
    }
    env.mem().pop_local(s);
    ev
}

/// C: `FUN_021AE198` (unlabeled routine at `0x021AE198`): does trainer `m` looking `dir` (sight `range`, used as
/// `(s16)`) see the player (`FUN_0216DE94(sys_of(m), 0xFF)`; none: -1)? The [`line_test`] distance, or -1 when there
/// is no line or [`path_blocked`] answers 1 (`enc` = the effect-encount object). Called from `fldmmdl_acmd` `0219AE38`.
pub fn sees_dir<E: FieldEnv>(env: &mut E, m: u32, dir: u32, range: u32, enc: u32) -> u32 {
    let sys = c1(env, SYS_OF, m);
    let player = c2(env, FIND_BY_ID, sys, 0xFF);
    if player == 0 {
        return u32::MAX;
    }
    // player position at +0, trainer position at +8 (the original's stack)
    let s = env.mem().push_local(0x10);
    grid(env, player, s);
    grid(env, m, s + 8);
    let mut d = line_test(env.mem(), dir, range as i16 as i32, s + 8, s) as u32;
    if d != u32::MAX && path_blocked(env, m, dir, d, enc) == 1 {
        d = u32::MAX;
    }
    env.mem().pop_local(s);
    d
}

/// C: `FUN_021AE1FC` (unlabeled routine at `0x021AE1FC`): the first MMDL in use (`FUN_0216DD1C` over the MMDLSYS of
/// `fm`) that is not the player's MMDL (`FUN_021A3074(FIELD_PLAYER)`), not `exclude` (0 = none), [`sees`] the player
/// and is not [`defeated`]: [`record_fill`] of `rec` (dir as u16), 1; else 0.
pub fn search<E: FieldEnv>(env: &mut E, fm: u32, exclude: u32, rec: u32) -> u32 {
    let enc = c1(env, FM_ENCOUNT, fm);
    let fp = c1(env, FM_PLAYER, fm);
    let pm = c1(env, PLAYER_MMDL, fp);
    let sys = c1(env, FM_MMDLSYS, fm);
    // the original's stack words: the direction (sp + 0xC), the iterator's index (sp + 0x10) and MMDL (sp + 0x14),
    // the last two set to 0
    let s = env.mem().push_local(0xC);
    let mut res = 0;
    while let Some(m) = next_in_use(env, sys, s + 8, s + 4) {
        if m == pm || (exclude != 0 && exclude == m) {
            continue;
        }
        let d = sees(env, enc, m, pm, s);
        if d == u32::MAX || defeated(env, fm, m) != 0 {
            continue;
        }
        let dir = env.r16(s) as u32;
        record_fill(env, rec, m, d, dir);
        res = 1;
        break;
    }
    env.mem().pop_local(s);
    res
}

/// C: `FUN_021AE288` (unlabeled routine at `0x021AE288`): does trainer `m` see `player`? Trainer kind ([`trainer_kind`])
/// 1 or 2 (low halfword; else -1), range = `(s16) FUN_0216D63C(m, 0)`. Kind 1 (whole word): the display direction;
/// kind 2: directions 0..3 in turn. The first direction with a [`line_test`] hit and a clear path ([`path_blocked`] 0)
/// stores the direction as u16 at `dir_out` and returns the distance; else -1.
pub fn sees<E: FieldEnv>(env: &mut E, enc: u32, m: u32, player: u32, dir_out: u32) -> u32 {
    let kind = trainer_kind(env, m);
    if kind.wrapping_add(0xFFFF) & 0xFFFF > 1 {
        return u32::MAX;
    }
    let range = c2(env, PARAM, m, 0) as i16 as i32;
    // player position at +0, trainer position at +8 (sp + 0x10 / 0x18 of the original)
    let s = env.mem().push_local(0x10);
    let (pp, tp) = (s, s + 8);
    grid(env, m, tp);
    grid(env, player, pp);
    let mut res = u32::MAX;
    if kind == 1 {
        let dir = c1(env, DIR_DISP, m);
        let d = line_test(env.mem(), dir, range, tp, pp) as u32;
        if d != u32::MAX && path_blocked(env, m, dir, d, enc) == 0 {
            env.w16(dir_out, dir as u16);
            res = d;
        }
    } else {
        for dir in 0..4 {
            let d = line_test(env.mem(), dir, range, tp, pp) as u32;
            if d != u32::MAX && path_blocked(env, m, dir, d, enc) == 0 {
                env.w16(dir_out, dir as u16);
                res = d;
                break;
            }
        }
    }
    env.mem().pop_local(s);
    res
}

/// Reads coordinate `i` (0 x, 1 y, 2 z) of the `s16[3]` grid position at `p`, sign-extended.
fn coord(mem: &Mem, p: u32, i: u32) -> i32 {
    mem.r16(p.wrapping_add(2 * i)) as i16 as i32
}

/// The shared shape of the four line tests: the two cross axes `c0` / `c1` (coordinate indices) must be equal; along axis `along`
/// the player (`b`) must be in front of the trainer (`a`), within `range` tiles (32-bit wrapping arithmetic and
/// signed compares, as the original), `back` = towards smaller coordinates.
fn line(
    mem: &Mem,
    range: i32,
    a: u32,
    b: u32,
    (c0, c1, along): (u32, u32, u32),
    back: bool,
) -> i32 {
    if coord(mem, a, c0) != coord(mem, b, c0) || coord(mem, a, c1) != coord(mem, b, c1) {
        return -1;
    }
    let ta = coord(mem, a, along);
    let pb = coord(mem, b, along);
    if back {
        // up / left: b < a and b >= a - range
        if pb < ta && pb >= ta.wrapping_sub(range) {
            return ta.wrapping_sub(pb);
        }
    } else if pb > ta && pb <= ta.wrapping_add(range) {
        // down / right: b > a and b <= a + range
        return pb.wrapping_sub(ta);
    }
    -1
}

/// C: `FUN_021AE360` (unlabeled routine at `0x021AE360`): line of sight up: same x and y, the player's z below the
/// trainer's and at most `range` away: the distance `a.z - b.z`, else -1.
pub fn line_up(mem: &Mem, range: i32, a: u32, b: u32) -> i32 {
    line(mem, range, a, b, (0, 1, 2), true)
}

/// C: `FUN_021AE394` (unlabeled routine at `0x021AE394`): line of sight down: same x and y, `a.z < b.z <= a.z +
/// range`: `b.z - a.z`, else -1.
pub fn line_down(mem: &Mem, range: i32, a: u32, b: u32) -> i32 {
    line(mem, range, a, b, (0, 1, 2), false)
}

/// C: `FUN_021AE3C8` (unlabeled routine at `0x021AE3C8`): line of sight left: same z and y, `a.x - range <= b.x <
/// a.x`: `a.x - b.x`, else -1.
pub fn line_left(mem: &Mem, range: i32, a: u32, b: u32) -> i32 {
    line(mem, range, a, b, (2, 1, 0), true)
}

/// C: `FUN_021AE3FC` (unlabeled routine at `0x021AE3FC`): line of sight right: same z and y, `a.x < b.x <= a.x +
/// range`: `b.x - a.x`, else -1.
pub fn line_right(mem: &Mem, range: i32, a: u32, b: u32) -> i32 {
    line(mem, range, a, b, (2, 1, 0), false)
}

/// A line test: `(mem, range, &trainer position, &player position)` -> distance or -1.
pub type LineTest = fn(&Mem, i32, u32, u32) -> i32;

/// The line tests by direction (ROM table `0x021D9338`).
pub const LINE_TESTS: [LineTest; 4] = [line_up, line_down, line_left, line_right];

/// C: `FUN_021AE348` (unlabeled routine at `0x021AE348`): the line test of direction `dir` (ROM `0x021D9338[dir]`)
/// on the trainer position `a` and the player position `b`. UNVERIFIED: `dir > 3` panics (the original calls
/// whatever the ROM words after the table point at); the game only passes display directions 0..3.
pub fn line_test(mem: &Mem, dir: u32, range: i32, a: u32, b: u32) -> i32 {
    LINE_TESTS[dir as usize](mem, range, a, b)
}

/// C: `FUN_021AE430` (unlabeled routine at `0x021AE430`): the GMEVENT that runs the approach TASKs `task0` / `task1`
/// (`FUN_020122C0(gsys, 0, FUN_021AE754, 8)`, its work (`FUN_020124E8`) = `{task0, task1}`).
pub fn approach_event_new<E: FieldEnv>(env: &mut E, gsys: u32, task0: u32, task1: u32) -> u32 {
    let ev = env.callv(GMEVENT_NEW, &[gsys, 0, APPROACH_PROC, 8]);
    let w = c1(env, GMEVENT_WORK, ev);
    env.w32(w, task0);
    env.w32(w + 4, task1);
    ev
}

/// C: `FUN_021AE454` (unlabeled routine at `0x021AE454`): a new approach TASK (`Heap_AllocDebug(heap, 0x38, 1,
/// "event_trainer_eye.c", 506)`) for the EYE HIT record `rec` in slot `index`: `+8..+0x1F` = the record, `+0x20` = 0,
/// `+0x24` = index, `+0x30` = `fm`, `+0x34` = `FUN_02188CB4(fm)` (FIELD_PLAYER). State, done, counter and effect are
/// left as allocated.
pub fn task_new<E: FieldEnv>(env: &mut E, heap: u32, fm: u32, rec: u32, index: u32) -> u32 {
    let t = env.callv(HEAP_ALLOC, &[heap, TASK_SIZE, 1, FILE_TRAINER_EYE, 506]);
    for k in 0..EYE_HIT_SIZE / 4 {
        let w = env.r32(rec + 4 * k);
        env.w32(t + TASK_HIT + 4 * k, w);
    }
    env.w32(t + TASK_20, 0);
    env.w32(t + TASK_INDEX, index);
    env.w32(t + TASK_FIELDMAP, fm);
    let p = c1(env, FM_PLAYER, fm);
    env.w32(t + TASK_PLAYER, p);
    t
}

/// C: `FUN_021AE498` (unlabeled routine at `0x021AE498`): TASK state 0: unpause the trainer if its move bit 0x10 is
/// set (`FUN_0216DA5C == 1` -> `FUN_0216DB04`); state 1; returns 1 (run the next state now).
pub fn state_unpause<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, BIT10, m) == 1 {
        let m = env.r32(task + T_MMDL);
        c1(env, UNPAUSE, m);
    }
    env.w32(task + TASK_STATE, 1);
    1
}

/// C: `FUN_021AE4B4` (unlabeled routine at `0x021AE4B4`): TASK state 1: while move bit 0x10 is set, 0 (wait); else
/// show the record's direction (`FUN_0216D5E0(m, (u16) dir)`), pause the trainer, state 2, 1.
pub fn state_face<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, BIT10, m) == 1 {
        return 0;
    }
    let dir = env.r32(task + T_DIR) & 0xFFFF;
    let m = env.r32(task + T_MMDL);
    c2(env, SET_DIR_DISP, m, dir);
    let m = env.r32(task + T_MMDL);
    c1(env, PAUSE, m);
    env.w32(task + TASK_STATE, 2);
    1
}

/// C: `FUN_021AE4E0` (unlabeled routine at `0x021AE4E0`): TASK state 2: wait (0) until the player's acmd is done
/// (`FUN_0216D454(FUN_021A3074(FIELD_PLAYER))`); then state 3, or state 7 for trainer move codes 0x33..=0x36; 1.
pub fn state_wait_player<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let fp = env.r32(task + TASK_PLAYER);
    let p = c1(env, PLAYER_MMDL, fp);
    if c1(env, ACMD_DONE, p) == 0 {
        return 0;
    }
    env.w32(task + TASK_STATE, 3);
    let m = env.r32(task + T_MMDL);
    if c1(env, MOVE_CODE, m).wrapping_sub(0x33) <= 3 {
        env.w32(task + TASK_STATE, 7);
    }
    1
}

/// C: `FUN_021AE524` (unlabeled routine at `0x021AE524`): TASK state 3: when the trainer can take an acmd
/// (`FUN_0216D3CC == 1`): set the face acmd `FUN_02197E7C((u16) dir, 0)`, state 4. Returns 0.
pub fn state_face_acmd<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, ACMD_CAN_SET, m) == 1 {
        let dir = env.r32(task + T_DIR) & 0xFFFF;
        let code = c2(env, ACMD_CODE, dir, 0);
        let m = env.r32(task + T_MMDL);
        c2(env, ACMD_SET, m, code);
        env.w32(task + TASK_STATE, 4);
    }
    0
}

/// C: `FUN_021AE550` (unlabeled routine at `0x021AE550`): TASK state 4: wait (0) until the trainer's acmd is done;
/// then state 5, 1.
pub fn state_face_done<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, ACMD_DONE, m) == 0 {
        return 0;
    }
    env.w32(task + TASK_STATE, 5);
    1
}

/// C: `FUN_021AE56C` (unlabeled routine at `0x021AE56C`): TASK state 5: the "!" balloon over the trainer
/// (`+0x2C = FUN_021BB528(FUN_02188CD0(fm), m, 0, 0)`, D3), state 6. Returns 0.
pub fn state_balloon<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let fm = env.r32(task + TASK_FIELDMAP);
    let ctrl = c1(env, FM_FX_CTRL, fm);
    let m = env.r32(task + T_MMDL);
    let fx = env.callv(GYOE_NEW, &[ctrl, m, 0, 0]);
    env.w32(task + TASK_EFFECT, fx);
    env.w32(task + TASK_STATE, 6);
    0
}

/// C: `FUN_021AE58C` (unlabeled routine at `0x021AE58C`): TASK state 6: when the balloon has finished
/// (`FUN_021BB5C8 == 1`): delete it (`FUN_021AC154`), state 9. Returns 0.
pub fn state_balloon_done<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let fx = env.r32(task + TASK_EFFECT);
    if c1(env, GYOE_DONE, fx) == 1 {
        let fx = env.r32(task + TASK_EFFECT);
        c1(env, FX_DELETE, fx);
        env.w32(task + TASK_STATE, 9);
    }
    0
}

/// C: `FUN_021AE5A8` (unlabeled routine at `0x021AE5A8`): TASK state 7: set acmd 0x65 on the trainer, state 8.
/// Returns 0.
pub fn state_acmd_65<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    c2(env, ACMD_SET, m, 0x65);
    env.w32(task + TASK_STATE, 8);
    0
}

/// C: `FUN_021AE5BC` (unlabeled routine at `0x021AE5BC`): TASK state 8: when the trainer's acmd is done (`== 1`):
/// state 9. Returns 0.
pub fn state_acmd_65_done<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, ACMD_DONE, m) == 1 {
        env.w32(task + TASK_STATE, 9);
    }
    0
}

/// C: `FUN_021AE5D4` (unlabeled routine at `0x021AE5D4`): TASK state 9, the wait after the "!" balloon:
/// `++counter >= 30` (signed): counter 0, state 10. Returns 0 (wait for the next frame).
pub fn state_wait_30(mem: &mut Mem, task: u32) -> u32 {
    let n = mem.r32(task + TASK_COUNTER).wrapping_add(1);
    mem.w32(task + TASK_COUNTER, n);
    if n as i32 >= 30 {
        mem.w32(task + TASK_COUNTER, 0);
        mem.w32(task + TASK_STATE, 10);
    }
    0
}

/// C: `FUN_021AE5EC` (unlabeled routine at `0x021AE5EC`): TASK state 10: `distance <= 1` (signed): state 13 (next to
/// the player), else state 11 (one more step). Returns 1.
pub fn state_walk_check(mem: &mut Mem, task: u32) -> u32 {
    let st = if mem.r32(task + T_DIST) as i32 <= 1 {
        13
    } else {
        11
    };
    mem.w32(task + TASK_STATE, st);
    1
}

/// C: `FUN_021AE604` (unlabeled routine at `0x021AE604`): TASK state 11: when the trainer can take an acmd: set the
/// walk acmd `FUN_02197E7C((u16) dir, 0xC)`, state 12. Returns 0.
pub fn state_walk<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, ACMD_CAN_SET, m) == 1 {
        let dir = env.r32(task + T_DIR) & 0xFFFF;
        let code = c2(env, ACMD_CODE, dir, 0xC);
        let m = env.r32(task + T_MMDL);
        c2(env, ACMD_SET, m, code);
        env.w32(task + TASK_STATE, 12);
    }
    0
}

/// C: `FUN_021AE630` (unlabeled routine at `0x021AE630`): TASK state 12: wait (0) until the step is done; then
/// `distance -= 1`, state 10, 1.
pub fn state_walk_done<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    if c1(env, ACMD_DONE, m) == 0 {
        return 0;
    }
    let d = env.r32(task + T_DIST).wrapping_sub(1);
    env.w32(task + T_DIST, d);
    env.w32(task + TASK_STATE, 10);
    1
}

/// C: `FUN_021AE650` (unlabeled routine at `0x021AE650`): TASK state 13: `++counter < 8` (signed): 0 (wait); else
/// counter 0, state 14, 1.
pub fn state_wait_8(mem: &mut Mem, task: u32) -> u32 {
    let n = mem.r32(task + TASK_COUNTER).wrapping_add(1);
    mem.w32(task + TASK_COUNTER, n);
    if (n as i32) < 8 {
        return 0;
    }
    mem.w32(task + TASK_COUNTER, 0);
    mem.w32(task + TASK_STATE, 14);
    1
}

/// C: `FUN_021AE66C` (unlabeled routine at `0x021AE66C`): TASK state 14: the player turns to the trainer. `d =
/// FUN_021642B4(gx(p), gz(p), gx(m), gz(m))` (p = the player's MMDL); `d == dir_disp(p)`, or slot index != 0 with a
/// kind other than 2: state 16. Else, when the player can take an acmd: the face acmd `FUN_02197E7C(d, 0)`, state 15
/// (otherwise stay). Returns 0.
pub fn state_player_turn<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let fp = env.r32(task + TASK_PLAYER);
    let p = c1(env, PLAYER_MMDL, fp);
    let px = c1(env, GX, p);
    let pz = c1(env, GZ, p);
    let m = env.r32(task + T_MMDL);
    let mx = c1(env, GX, m);
    let m = env.r32(task + T_MMDL);
    let mz = c1(env, GZ, m);
    let d = env.callv(DIR_BETWEEN, &[px, pz, mx, mz]);
    let st = if d == c1(env, DIR_DISP, p)
        || (env.r32(task + TASK_INDEX) != 0 && env.r32(task + T_KIND) != 2)
    {
        16
    } else if c1(env, ACMD_CAN_SET, p) == 1 {
        let code = c2(env, ACMD_CODE, d, 0);
        c2(env, ACMD_SET, p, code);
        15
    } else {
        return 0;
    };
    env.w32(task + TASK_STATE, st);
    0
}

/// C: `FUN_021AE6E0` (unlabeled routine at `0x021AE6E0`): TASK state 15: wait (0) until the player's acmd is done;
/// then end it (`FUN_0216D480`), state 16, 1.
pub fn state_player_turn_done<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let fp = env.r32(task + TASK_PLAYER);
    let p = c1(env, PLAYER_MMDL, fp);
    if c1(env, ACMD_DONE, p) == 0 {
        return 0;
    }
    c1(env, ACMD_END, p);
    env.w32(task + TASK_STATE, 16);
    1
}

/// C: `FUN_021AE708` (unlabeled routine at `0x021AE708`): TASK state 16: end the trainer's acmd, move code 0
/// (`FUN_0216E0E8(m, 0)`: the trainer stops wandering), state 17, 1.
pub fn state_end<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let m = env.r32(task + T_MMDL);
    c1(env, ACMD_END, m);
    let m = env.r32(task + T_MMDL);
    c2(env, CHANGE_MOVE_CODE, m, 0);
    env.w32(task + TASK_STATE, 17);
    1
}

/// C: `FUN_021AE724` (unlabeled routine at `0x021AE724`): TASK state 17: done = 1. Returns 0.
pub fn state_done(mem: &mut Mem, task: u32) -> u32 {
    mem.w32(task + TASK_DONE, 1);
    0
}

/// The TASK state `state` (ROM table `0x021D9348`, 18 entries) on `task`. UNVERIFIED: `state > 17` panics (the
/// original calls whatever the ROM words after the table point at).
fn run_state<E: FieldEnv>(env: &mut E, state: u32, task: u32) -> u32 {
    match state {
        0 => state_unpause(env, task),
        1 => state_face(env, task),
        2 => state_wait_player(env, task),
        3 => state_face_acmd(env, task),
        4 => state_face_done(env, task),
        5 => state_balloon(env, task),
        6 => state_balloon_done(env, task),
        7 => state_acmd_65(env, task),
        8 => state_acmd_65_done(env, task),
        9 => state_wait_30(env.mem(), task),
        10 => state_walk_check(env.mem(), task),
        11 => state_walk(env, task),
        12 => state_walk_done(env, task),
        13 => state_wait_8(env.mem(), task),
        14 => state_player_turn(env, task),
        15 => state_player_turn_done(env, task),
        16 => state_end(env, task),
        17 => state_done(env.mem(), task),
        _ => panic!("trainer-eye TASK state {state} is past the ROM table 0x021D9348"),
    }
}

/// C: `FUN_021AE72C` (unlabeled routine at `0x021AE72C`): one frame of the approach TASK: run the states (ROM
/// `0x021D9348[task.state]`) while they answer 1; returns 1 when the TASK is done (`+4 == 1`), else 0.
pub fn task_step<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    loop {
        let st = env.r32(task + TASK_STATE);
        if run_state(env, st, task) != 1 {
            break;
        }
    }
    (env.r32(task + TASK_DONE) == 1) as u32
}

/// C: `FUN_021AE754` (unlabeled routine at `0x021AE754`): the GMEVENT proc of [`approach_event_new`] (`work` = the
/// two TASK pointers, 0 = gone): steps each live TASK ([`task_step`]) and frees it (`Heap_Free`, slot = 0) when done;
/// returns 1 when both are gone, else 0. `event` and `seq` are not read.
pub fn approach_event<E: FieldEnv>(env: &mut E, _event: u32, _seq: u32, work: u32) -> u32 {
    for k in 0..2 {
        let slot = work + 4 * k;
        let t = env.r32(slot);
        if t != 0 && task_step(env, t) == 1 {
            let t = env.r32(slot);
            c1(env, HEAP_FREE, t);
            env.w32(slot, 0);
        }
    }
    (env.r32(work) == 0 && env.r32(work + 4) == 0) as u32
}

/// C: `FUN_021AE79C` (unlabeled routine at `0x021AE79C`): the trainer kind of `m`: `FUN_0216E434(event_type(m))` (1 =
/// trainer looking along its display direction, 2 = trainer looking all four ways, others no trainer).
pub fn trainer_kind<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let t = c1(env, EVENT_TYPE, m);
    c1(env, KIND_434, t)
}

/// C: `FUN_021AE7A8` (unlabeled routine at `0x021AE7A8`): is the view of trainer `m` looking `dir` over `dist` tiles
/// blocked? `dist == 0`: 0. From the tile in front of the trainer (`grid(m) + (FUN_021641E8, FUN_021641F8)(dir)`), each
/// of the `dist - 1` tiles in between (signed count) must be free (`FUN_02163994(m, x, y, z, dir) & !1 == 0`) and,
/// with an effect-encount object `enc`, not its active tile (`FUN_021AA900(enc, &pos)`); the player's tile must hit
/// exactly "an MMDL" (`& !1 == 4`) and not be the effect-encount tile. Any failure: 1, else 0. Coordinates are
/// stored back as s16 (wrapping).
pub fn path_blocked<E: FieldEnv>(env: &mut E, m: u32, dir: u32, dist: u32, enc: u32) -> u32 {
    if dist == 0 {
        return 0;
    }
    let mut pos = [0u8; 6];
    env.call(GRID, &mut [CArg::V(m), CArg::OutBuf(&mut pos)]);
    let dx = c1(env, DIR_DX, dir);
    let dz = c1(env, DIR_DZ, dir);
    let get =
        |pos: &[u8; 6], i: usize| i16::from_le_bytes([pos[2 * i], pos[2 * i + 1]]) as i32 as u32;
    let advance = |pos: &mut [u8; 6]| {
        let x = get(pos, 0).wrapping_add(dx) as u16;
        pos[0..2].copy_from_slice(&x.to_le_bytes());
        let z = get(pos, 2).wrapping_add(dz) as u16;
        pos[4..6].copy_from_slice(&z.to_le_bytes());
    };
    advance(&mut pos);
    let count = dist.wrapping_sub(1) as i32;
    let mut i = 0;
    while i < count {
        let h = env.callv(
            STEP_HITS,
            &[m, get(&pos, 0), get(&pos, 1), get(&pos, 2), dir],
        );
        if h & !1 != 0 {
            return 1;
        }
        if enc != 0 && env.call(ENC_TILE, &mut [CArg::V(enc), CArg::InBuf(&pos)]) != 0 {
            return 1;
        }
        i += 1;
        advance(&mut pos);
    }
    let h = env.callv(
        STEP_HITS,
        &[m, get(&pos, 0), get(&pos, 1), get(&pos, 2), dir],
    );
    if h & !1 != 4 {
        return 1;
    }
    if enc != 0 && env.call(ENC_TILE, &mut [CArg::V(enc), CArg::InBuf(&pos)]) != 0 {
        return 1;
    }
    0
}

/// C: `FUN_021AE884` (unlabeled routine at `0x021AE884`): the trainer id of `m`: `FUN_0215A460(event_id(m))` (the
/// trainer of the MMDL's script).
pub fn trainer_id<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let id = c1(env, EVENT_ID, m);
    c1(env, TRAINER_ID, id)
}

/// C: `FUN_021AE890` (unlabeled routine at `0x021AE890`): has the trainer `m` been defeated already?
/// `FUN_0215A4D8(event work of the game data of the GAMESYS of fm, trainer_id(m))`.
pub fn defeated<E: FieldEnv>(env: &mut E, fm: u32, m: u32) -> u32 {
    let gs = c1(env, FM_GAMESYS, fm);
    let gd = c1(env, GAMEDATA, gs);
    let ew = c1(env, EVENT_WORK, gd);
    let tid = trainer_id(env, m);
    c2(env, DEFEATED, ew, tid)
}

/// C: `FUN_021AE8B4` (unlabeled routine at `0x021AE8B4`): fill the EYE HIT record `rec`: `{m, dist, dir,
/// event_id(m), trainer_id(m)}` (the five words; `+0x14` kind is the caller's).
pub fn record_fill<E: FieldEnv>(env: &mut E, rec: u32, m: u32, dist: u32, dir: u32) {
    env.w32(rec + HIT_DIST, dist);
    env.w32(rec + HIT_DIR, dir);
    let sid = c1(env, EVENT_ID, m);
    env.w32(rec + HIT_SCRIPT, sid);
    let tid = trainer_id(env, m);
    env.w32(rec + HIT_TRAINER, tid);
    env.w32(rec + HIT_MMDL, m);
}

/// C: `FUN_021AE8D4` (unlabeled routine at `0x021AE8D4`): the partner of a double-battle trainer: the first other MMDL
/// in use (iterator over `sys_of(m)`) whose [`trainer_kind`] is 1 or 2 (low halfword) and whose [`trainer_id`] is `tid`;
/// else 0.
pub fn partner<E: FieldEnv>(env: &mut E, m: u32, tid: u32) -> u32 {
    // the iterator's index (sp + 0, set to 0) and MMDL (sp + 4) words
    let s = env.mem().push_local(8);
    let sys = c1(env, SYS_OF, m);
    let mut res = 0;
    while let Some(o) = next_in_use(env, sys, s + 4, s) {
        if o == m {
            continue;
        }
        if trainer_kind(env, o).wrapping_add(0xFFFF) & 0xFFFF <= 1 && trainer_id(env, o) == tid {
            res = o;
            break;
        }
    }
    env.mem().pop_local(s);
    res
}

/// C: `FUN_021AE930` (unlabeled routine at `0x021AE930`): the trainer-eye script event of `m`:
/// `FUN_02158768(FUN_02188C80(fm), 0xE20, m, 1)`.
pub fn script_event<E: FieldEnv>(env: &mut E, fm: u32, m: u32) -> u32 {
    let gs = c1(env, FM_GAMESYS, fm);
    env.callv(SCRIPT_EVENT, &[gs, SCRIPT_TRAINER_EYE, m, 1])
}

#[cfg(test)]
mod tests;
