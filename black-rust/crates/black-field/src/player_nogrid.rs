//! `field_player_nogrid.c` (overlay 21, `0x021A5880..0x021A6CB8`; F3b): the player on the free-movement (rail) maps: pad ->
//! move value by form (walk / cycle / surf) -> MMDL acmd, slope tiles of the rail attributes (forced slides), the cycling-road
//! effect sound and the input-hold record.
//!
//! NOGRID PLAYER N (0x24, `Heap_AllocDebug(heap, 0x24, 1, "field_player_nogrid.c", 0x1A0)`): `+0` move state (0 stand, 1 move,
//! 2 turn, 3 bump), `+4` flags (1 sliding, 2, 4, 8 slide kind A, 0x10 slide kind B, 0x20 bounce, 0x40 hold), `+8` the sound
//! record ([`eff_start`]: u8 playing, u8 kind 0..1, s16 frame), `+0xC` the input record ([`rec_input`]: u8 flags, s16 counter at
//! `+2`, u32 held direction at `+4`), `+0x14` the PLAYER CORE, `+0x18` its FIELDMAP, `+0x1C` the player MMDL, `+0x20` its RAIL
//! work. Directions 0..3, 9 = none.
//!
//! Every callee outside this area (the PLAYER CORE, fldmmdl, the acmd table, map attributes, the rail attribute predicates of
//! [`crate::rail_attr`], sound, the MMDL rail glue `0219E2C0` ..) is reached through [`FieldEnv::call`] by address; the ROM
//! table 0x021D8ABC of the slope procs is dispatched to the Rust procs. Verified by `tools/check-fn.sh black-field
//! player_nogrid --per-function` (spec `tools/rom-diff/specs/black-field/player_nogrid.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_5880,
    0x021A_5920,
    0x021A_5928,
    0x021A_5964,
    0x021A_596C,
    0x021A_59DC,
    0x021A_59EC,
    0x021A_5A00,
    0x021A_5A0C,
    0x021A_5A18,
    0x021A_5A24,
    0x021A_5A30,
    0x021A_5A34,
    0x021A_5A68,
    0x021A_5AD4,
    0x021A_5B4C,
    0x021A_5B6C,
    0x021A_5B7C,
    0x021A_5BC0,
    0x021A_5C3C,
    0x021A_5CA4,
    0x021A_5D04,
    0x021A_5DD8,
    0x021A_5E28,
    0x021A_5EA0,
    0x021A_5EA4,
    0x021A_5EDC,
    0x021A_5F14,
    0x021A_5F38,
    0x021A_5F74,
    0x021A_5F98,
    0x021A_6014,
    0x021A_607C,
    0x021A_60DC,
    0x021A_61B0,
    0x021A_6200,
    0x021A_6278,
    0x021A_627C,
    0x021A_62B4,
    0x021A_62D8,
    0x021A_62FC,
    0x021A_6338,
    0x021A_635C,
    0x021A_63EC,
    0x021A_63F0,
    0x021A_6428,
    0x021A_6454,
    0x021A_6478,
    0x021A_64B4,
    0x021A_652C,
    0x021A_658C,
    0x021A_661C,
    0x021A_666C,
    0x021A_66E4,
    0x021A_66EC,
    0x021A_66F8,
    0x021A_6708,
    0x021A_6714,
    0x021A_6720,
    0x021A_672C,
    0x021A_6738,
    0x021A_6744,
    0x021A_6750,
    0x021A_675C,
    0x021A_6768,
    0x021A_6774,
    0x021A_6780,
    0x021A_678C,
    0x021A_6798,
    0x021A_67A4,
    0x021A_682C,
    0x021A_6870,
    0x021A_6908,
    0x021A_6974,
    0x021A_69D8,
    0x021A_6A3C,
    0x021A_6A64,
    0x021A_6A8C,
    0x021A_6AA0,
    0x021A_6B08,
    0x021A_6B30,
    0x021A_6B50,
    0x021A_6B88,
    0x021A_6B9C,
    0x021A_6BAC,
    0x021A_6BB4,
    0x021A_6C30,
    0x021A_6C4C,
    0x021A_6C64,
    0x021A_6C6C,
];

/// Address of the string `"field_player_nogrid.c"`.
pub const FILE_PLAYER_NOGRID: u32 = 0x021D_DF04;
/// The MMDL header in overlay 21 data (`+2` = the player's OBJ code before the rail MMDL is set up).
pub const RAIL_HEADER: u32 = 0x021D_DEE0;
/// ROM `0x021D8AB0`: the spin directions of the sound record's two kinds.
pub const SPIN_DIRS: [[u8; 4]; 2] = [[0, 2, 1, 3], [0, 3, 1, 2]];
/// "No direction".
pub const DIR_NONE: u32 = 9;
/// The bump sound.
const SE_BUMP: u32 = 0x55A;
/// The slope sound.
const SE_SLOPE: u32 = 0x55E;
/// The cycling-road sound.
const SE_ROAD: u32 = 0x6D1;

// PLAYER CORE (field_player_core.c)
const CORE_FIELDMAP: u32 = 0x021A_395C;
const CORE_MMDL: u32 = 0x021A_3960;
const CORE_SET_STATE: u32 = 0x021A_3964;
const CORE_FORM: u32 = 0x021A_3974;
const CORE_KEY_DIR: u32 = 0x021A_3AB8;
const CORE_3AC4: u32 = 0x021A_3AC4;
const CORE_REQUEST: u32 = 0x021A_3CB0;
const CORE_3CC0: u32 = 0x021A_3CC0;
const CORE_3CE8: u32 = 0x021A_3CE8;
// fldmmdl
const M_ACMD_SET: u32 = 0x0216_D410;
const M_ACMD_END: u32 = 0x0216_D3CC;
const M_ACMD_CANCEL: u32 = 0x0216_D4B4;
const M_STATUS_ON: u32 = 0x0216_D548;
const M_STATUS_OFF: u32 = 0x0216_D550;
const M_CODE: u32 = 0x0216_D598;
const M_MOVE_CODE: u32 = 0x0216_D5A0;
const M_SET_SPIN: u32 = 0x0216_D5D8;
const M_SET_DIR: u32 = 0x0216_D5E0;
const M_DIR: u32 = 0x0216_D5F8;
const M_MOVE_DIR: u32 = 0x0216_D608;
const M_SET_MOVE_DIR: u32 = 0x0216_D60C;
const M_D674: u32 = 0x0216_D674;
const M_ATTR: u32 = 0x0216_D7DC;
const M_SET_HEADER: u32 = 0x0216_E0AC;
const ACMD_CODE: u32 = 0x0219_7E7C;
// MMDL rail glue (fldmmdl_rail.c, other F items)
const MR_LOC_FRONT: u32 = 0x0219_D724;
const MR_SET_LOC: u32 = 0x0219_E2C0;
const MR_GET_LOC: u32 = 0x0219_E2F4;
const MR_WORK: u32 = 0x0219_E348;
const MR_HIT: u32 = 0x0219_E424;
const MR_FRONT: u32 = 0x0219_E664;
// map attributes (map_attr.c) and rail attributes (rail_attr.c)
const ATTR_VALUE: u32 = 0x021A_B0F0;
const ATTR_FLAGS: u32 = 0x021A_B0F8;
const ATTR_DIR_UP: u32 = 0x021A_B158;
const ATTR_DIR_DOWN: u32 = 0x021A_B164;
const ATTR_DIR_LEFT: u32 = 0x021A_B170;
const ATTR_DIR_RIGHT: u32 = 0x021A_B17C;
const ATTR_AB348: u32 = 0x021A_B348;
const RAIL_ATTR_PREDS: [u32; 6] = [
    0x021B_B1AC,
    0x021B_B1B8,
    0x021B_B1C4,
    0x021B_B1D0,
    0x021B_B1DC,
    0x021B_B1E8,
];
// others
const SE_PLAY: u32 = 0x0200_61E4;
const SE_HANDLE: u32 = 0x0200_6148;
const SE_STOP: u32 = 0x0200_61F8;
const DIR_REVERSE: u32 = 0x0216_42A4;
const RAIL_GET_POS: u32 = 0x021B_8028;

fn core<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    env.r32(n + 0x14)
}

fn mmdl<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    env.r32(n + 0x1C)
}

fn set_core_state<E: FieldEnv>(env: &mut E, n: u32, st: u32) {
    env.w32(n, st);
    let c = core(env, n);
    env.callv(CORE_SET_STATE, &[c, st]);
}

/// `acmd(dir, base)` on the MMDL.
fn acmd<E: FieldEnv>(env: &mut E, m: u32, dir: u32, base: u32) {
    let code = env.callv(ACMD_CODE, &[dir, base]);
    env.callv(M_ACMD_SET, &[m, code]);
}

/// C: `FUN_021A5880(core, heap)`: a new no-grid player: the CORE, its FIELDMAP and MMDL, the MMDL's rail work (the MMDL is
/// switched to the rail header unless its move code is 0x45), the input record; surf forms request their surf state.
pub fn create<E: FieldEnv>(env: &mut E, core_: u32, heap: u32) -> u32 {
    let n = env.callv(HEAP_ALLOC, &[heap, 0x24, 1, FILE_PLAYER_NOGRID, 0x1A0]);
    env.w32(n + 0x14, core_);
    let fm = env.callv(CORE_FIELDMAP, &[core_]);
    env.w32(n + 0x18, fm);
    let c = env.r32(n + 0x14);
    let m = env.callv(CORE_MMDL, &[c]);
    env.w32(n + 0x1C, m);
    rail_header(env, n);
    let m = mmdl(env, n);
    let rw = env.callv(MR_WORK, &[m]);
    env.w32(n + 0x20, rw);
    let zero = [0u8; 8];
    let m = mmdl(env, n);
    env.call(MR_SET_LOC, &mut [CArg::V(m), CArg::InBuf(&zero)]);
    rec_init(env, n + 0xC);
    let c = core(env, n);
    let bits = match env.callv(CORE_FORM, &[c]) {
        2 => Some(4),
        3 => Some(0x100),
        _ => None,
    };
    if let Some(b) = bits {
        let c = core(env, n);
        env.callv(CORE_REQUEST, &[c, b]);
        let c = core(env, n);
        env.callv(CORE_3CE8, &[c]);
    }
    n
}

/// Switch the MMDL to the rail header (its OBJ code kept) unless its move code is 0x45.
fn rail_header<E: FieldEnv>(env: &mut E, n: u32) {
    let m = mmdl(env, n);
    if env.callv(M_MOVE_CODE, &[m]) != 0x45 {
        let m = mmdl(env, n);
        let code = env.callv(M_CODE, &[m]);
        env.w16(RAIL_HEADER + 2, code as u16);
        let m = mmdl(env, n);
        env.callv(M_SET_HEADER, &[m, RAIL_HEADER]);
    }
}

/// C: `FUN_021A5920(n)`: free (veneer to Heap_Free).
pub fn delete<E: FieldEnv>(env: &mut E, n: u32) {
    env.callv(HEAP_FREE, &[n]);
}

/// C: `FUN_021A5928(n, loc)`: switch the MMDL to the rail header again, take its rail work and place it at `loc`.
pub fn reset_rail<E: FieldEnv>(env: &mut E, n: u32, loc: u32) {
    rail_header(env, n);
    let m = mmdl(env, n);
    let rw = env.callv(MR_WORK, &[m]);
    env.w32(n + 0x20, rw);
    let m = mmdl(env, n);
    env.call(MR_SET_LOC, &mut [CArg::V(m), CArg::In(loc, 8)]);
}

/// C: `FUN_021A5964(n)`: drop the rail work.
pub fn clear_rail<E: FieldEnv>(env: &mut E, n: u32) {
    env.w32(n + 0x20, 0);
}

/// C: `FUN_021A596C(n, trig, held)`: one frame: pad -> direction, input hold, slopes, move value, action, sound record.
pub fn frame<E: FieldEnv>(env: &mut E, n: u32, trig: u32, held: u32) {
    let c = core(env, n);
    let d = env.callv(CORE_KEY_DIR, &[c, held]);
    let c = core(env, n);
    env.callv(CORE_3CC0, &[c]);
    let m = mmdl(env, n);
    rec_input(env, n + 0xC, d, m);
    let d2 = slope(env, n, d, 0);
    let v = move_value(env, n, d2, trig, held, 0);
    after_value(env, n, held, v);
    action(env, n, v, d2, trig, held, 0);
    eff_update(env, n);
    rec_end(env, n + 0xC);
}

/// C: `FUN_021A59DC(n)`: the state is 3 (bump).
pub fn is_bump<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    (env.r32(n) == 3) as u32
}

/// C: `FUN_021A59EC(n)`: flag 1 (sliding).
pub fn is_sliding<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    (has_flags(env, n, 1) != 0) as u32
}

/// C: `FUN_021A5A00(n, loc)`: place the MMDL at the rail LOCATION `loc` (`0219E2C0`).
pub fn set_loc<E: FieldEnv>(env: &mut E, n: u32, loc: u32) {
    let m = mmdl(env, n);
    env.call(MR_SET_LOC, &mut [CArg::V(m), CArg::In(loc, 8)]);
}

/// C: `FUN_021A5A0C(n, out)`: the MMDL's rail LOCATION (`0219E2F4`).
pub fn get_loc<E: FieldEnv>(env: &mut E, n: u32, out: u32) {
    let m = mmdl(env, n);
    env.call(MR_GET_LOC, &mut [CArg::V(m), CArg::Out(out, 8)]);
}

/// C: `FUN_021A5A18(n, dir, out)`: the rail LOCATION in front of the MMDL (`0219E664`).
pub fn front_loc<E: FieldEnv>(env: &mut E, n: u32, dir: u32, out: u32) {
    let m = mmdl(env, n);
    env.call(MR_FRONT, &mut [CArg::V(m), CArg::V(dir), CArg::Out(out, 8)]);
}

/// C: `FUN_021A5A24(n, out)`: the position of the rail work (`021B8028`).
pub fn rail_pos<E: FieldEnv>(env: &mut E, n: u32, out: u32) {
    let rw = env.r32(n + 0x20);
    env.call(RAIL_GET_POS, &mut [CArg::V(rw), CArg::Out(out, 12)]);
}

/// C: `FUN_021A5A30(n)`: the rail work.
pub fn rail_work<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    env.r32(n + 0x20)
}

/// C: `FUN_021A5A34(n)`: end a bump: cancel the acmd, keep the facing, state 0.
pub fn end_bump<E: FieldEnv>(env: &mut E, n: u32) {
    if env.r32(n) != 3 {
        return;
    }
    let m = mmdl(env, n);
    let d = env.callv(M_DIR, &[m]);
    env.callv(M_ACMD_CANCEL, &[m]);
    env.callv(M_SET_DIR, &[m, d]);
    env.callv(M_D674, &[m, 0]);
    let c = core(env, n);
    env.callv(CORE_SET_STATE, &[c, 0]);
}

/// C: `FUN_021A5A68(n, dir, trig, held, flag)`: the move value by form (walk 0, cycle 1, surf 2 / 3).
pub fn move_value<E: FieldEnv>(
    env: &mut E,
    n: u32,
    dir: u32,
    trig: u32,
    held: u32,
    flag: u32,
) -> u32 {
    let c = core(env, n);
    let d = dir & 0xFFFF;
    match env.callv(CORE_FORM, &[c]) {
        0 => walk_value(env, n, trig, held, d, flag),
        1 => cycle_value(env, n, trig, held, d, flag),
        2 | 3 => surf_value(env, n, trig, held, d, flag),
        // the original returns an uninitialised stack word (0 under the test entry state)
        _ => 0,
    }
}

/// C: `FUN_021A5AD4(n, value, dir, trig, held, flag)`: the action for the move value by form.
#[allow(clippy::too_many_arguments)]
pub fn action<E: FieldEnv>(env: &mut E, n: u32, v: u32, dir: u32, trig: u32, held: u32, flag: u32) {
    let c = core(env, n);
    let d = dir & 0xFFFF;
    match env.callv(CORE_FORM, &[c]) {
        0 => walk_action(env, n, v, trig, held, d, flag),
        1 => cycle_action(env, n, v, trig, held, d, flag),
        2 | 3 => surf_action(env, n, v, trig, held, d, flag),
        _ => {}
    }
}

/// C: `FUN_021A5B4C(n, held, value)`: tell the CORE; clear flag 4 (and 2 for value 2).
pub fn after_value<E: FieldEnv>(env: &mut E, n: u32, held: u32, v: u32) {
    let c = core(env, n);
    env.callv(CORE_3AC4, &[c, held]);
    clear_flags(env, n, 4);
    if v == 2 {
        clear_flags(env, n, 2);
    }
}

/// C: `FUN_021A5B6C(n, m, dir, out)`: the MMDL's hit check in `dir` (`0219E424(m, dir, out)`; `out` = 0 or a word the
/// callee writes the attribute to; `n` is unused).
pub fn hit<E: FieldEnv>(env: &mut E, m: u32, dir: u32, out: u32) -> u32 {
    if out == 0 {
        env.callv(MR_HIT, &[m, dir, 0])
    } else {
        env.call(MR_HIT, &mut [CArg::V(m), CArg::V(dir), CArg::Out(out, 4)])
    }
}

/// C: `FUN_021A5B7C(n)`: the sound record: kind 0 while flag 8, kind 1 while flag 0x10, else stopped; then its frame.
pub fn eff_update<E: FieldEnv>(env: &mut E, n: u32) {
    if has_flags(env, n, 8) != 0 {
        eff_start(env, n + 8, 0);
    } else if has_flags(env, n, 0x10) != 0 {
        eff_start(env, n + 8, 1);
    } else {
        eff_stop(env, n + 8);
    }
    let m = mmdl(env, n);
    eff_frame(env, n + 8, m);
}

// ---- walk

/// C: `FUN_021A5BC0(n, value, trig, held, dir, flag)`: walk action 0..5.
#[allow(clippy::too_many_arguments)]
pub fn walk_action<E: FieldEnv>(
    env: &mut E,
    n: u32,
    v: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) {
    let m = mmdl(env, n);
    match v {
        0 => {}
        1 => act_stand(env, n, m, d),
        2 => act_walk(env, n, m, held, d, flag),
        3 => act_turn(env, n, m, d),
        4 => act_bump(env, n, m, d),
        5 => act_base(env, n, m, d, 0x38, 1),
        _ => {}
    }
    let _ = trig;
}

/// C: `FUN_021A5C3C(n, trig, held, dir, flag)`: walk move value by state.
pub fn walk_value<E: FieldEnv>(
    env: &mut E,
    n: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    let m = mmdl(env, n);
    match env.r32(n) {
        0 => v_stand(env, n, m, trig, held, d, flag, Form::Walk),
        1 => v_walk(env, n, m, trig, held, d, flag),
        2 => v_turn(env, n, m, trig, held, d, flag, Form::Walk),
        3 => v_bump(env, n, m, trig, held, d, flag, Form::Walk),
        _ => 0,
    }
}

#[derive(Clone, Copy)]
enum Form {
    Walk,
    Cycle,
    Surf,
}

#[allow(clippy::too_many_arguments)]
fn v_walk_of<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
    f: Form,
) -> u32 {
    match f {
        Form::Walk => v_walk(env, n, m, trig, held, d, flag),
        Form::Cycle => cycle_v_walk(env, n, m, trig, held, d, flag),
        Form::Surf => surf_v_walk(env, n, m, trig, held, d, flag),
    }
}

/// C: `FUN_021A5CA4` / `FUN_021A607C` / `FUN_021A652C` (n, m, trig, held, dir, flag): standing: 0 while the acmd runs, 1
/// without input, 3 (turn) for another direction unless `flag`, else the moving value.
#[allow(clippy::too_many_arguments)]
fn v_stand<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
    f: Form,
) -> u32 {
    if env.callv(M_ACMD_END, &[m]) != 1 {
        return 0;
    }
    if d & 0xFFFF == DIR_NONE {
        return 1;
    }
    let md = env.callv(M_DIR, &[m]);
    if d & 0xFFFF != md && flag == 0 {
        return 3;
    }
    v_walk_of(env, n, m, trig, held, d & 0xFFFF, flag, f)
}

/// C: `FUN_021A5CA4(n, m, trig, held, dir, flag)`: walk standing value.
#[allow(clippy::too_many_arguments)]
pub fn walk_v_stand<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_stand(env, n, m, trig, held, d, flag, Form::Walk)
}

/// The tile in front: 5 when it is a one-way tile (map attribute direction) facing `d`, else 4.
fn front_tile<E: FieldEnv>(env: &mut E, m: u32, d: u32) -> u32 {
    let mut loc = [0u8; 8];
    env.call(
        MR_FRONT,
        &mut [CArg::V(m), CArg::V(d), CArg::OutBuf(&mut loc)],
    );
    let a = env.call(MR_LOC_FRONT, &mut [CArg::V(m), CArg::InBuf(&loc)]);
    let v = env.callv(ATTR_VALUE, &[a]);
    env.callv(ATTR_FLAGS, &[a]);
    let k = if env.callv(ATTR_DIR_UP, &[v]) != 0 {
        0
    } else if env.callv(ATTR_DIR_DOWN, &[v]) != 0 {
        1
    } else if env.callv(ATTR_DIR_LEFT, &[v]) != 0 {
        2
    } else if env.callv(ATTR_DIR_RIGHT, &[v]) != 0 {
        3
    } else {
        9
    };
    if k != 9 && k == d {
        5
    } else {
        4
    }
}

/// C: `FUN_021A5D04(n, m, trig, held, dir, flag)`: walk moving value: 0 while the acmd runs, the standing value without
/// input, 2 (move) when the hit check is clear (or only bit 2 with `flag` 1), 5 on a one-way tile in `dir`, else 4.
#[allow(clippy::too_many_arguments)]
pub fn v_walk<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    if env.callv(M_ACMD_END, &[m]) == 0 {
        return 0;
    }
    if d == DIR_NONE {
        return v_stand(env, n, m, trig, held, DIR_NONE, flag, Form::Walk);
    }
    let mut h = hit(env, m, d, 0);
    if flag == 1 && h & 2 != 0 {
        h = 0;
    }
    if h == 0 {
        return 2;
    }
    if h & 2 != 0 {
        return front_tile(env, m, d);
    }
    4
}

/// C: `FUN_021A5DD8` / `FUN_021A61B0` / `FUN_021A661C` (n, m, trig, held, dir, flag): turning.
#[allow(clippy::too_many_arguments)]
fn v_turn<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
    f: Form,
) -> u32 {
    if env.callv(M_ACMD_END, &[m]) == 0 {
        return 0;
    }
    if d & 0xFFFF == DIR_NONE {
        return v_stand(env, n, m, trig, held, DIR_NONE, flag, f);
    }
    v_walk_of(env, n, m, trig, held, d & 0xFFFF, flag, f)
}

/// C: `FUN_021A5E28` / `FUN_021A6200` / `FUN_021A666C` (n, m, trig, held, dir, flag): bumping: a new direction cancels the
/// acmd.
#[allow(clippy::too_many_arguments)]
fn v_bump<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
    f: Form,
) -> u32 {
    let md = env.callv(M_DIR, &[m]);
    if d != DIR_NONE && d != md {
        env.callv(M_ACMD_CANCEL, &[m]);
        return v_walk_of(env, n, m, trig, held, d, flag, f);
    }
    if env.callv(M_ACMD_END, &[m]) == 0 {
        return 0;
    }
    if d == DIR_NONE {
        return v_stand(env, n, m, trig, held, DIR_NONE, flag, f);
    }
    v_walk_of(env, n, m, trig, held, d, flag, f)
}

/// C: `FUN_021A5DD8(n, m, trig, held, dir, flag)`: walk turning value.
#[allow(clippy::too_many_arguments)]
pub fn walk_v_turn<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_turn(env, n, m, trig, held, d, flag, Form::Walk)
}

/// C: `FUN_021A5E28(n, m, trig, held, dir, flag)`: walk bump value.
#[allow(clippy::too_many_arguments)]
pub fn walk_v_bump<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_bump(env, n, m, trig, held, d, flag, Form::Walk)
}

/// C: `FUN_021A5EA0` / `FUN_021A6278` / `FUN_021A63EC`: action 0 (none).
pub fn act_none() {}

/// C: `FUN_021A5EA4` / `FUN_021A627C` / `FUN_021A63F0` (n, m, .., dir): stand (acmd base 0; no direction: the facing one),
/// state 0.
pub fn act_stand<E: FieldEnv>(env: &mut E, n: u32, m: u32, d: u32) {
    let d = if d & 0xFFFF == DIR_NONE {
        env.callv(M_DIR, &[m]) & 0xFFFF
    } else {
        d & 0xFFFF
    };
    acmd(env, m, d, 0);
    set_core_state(env, n, 0);
}

/// C: `FUN_021A5F74` / `021a62b4` / `021a6338` / `021a6428`: an action with a fixed acmd base and new state (0x38 / 1, 0x14 / 1,
/// 0x38 / 1, 0x14 with `flag` 1 else 0x10 / 1).
pub fn act_base<E: FieldEnv>(env: &mut E, n: u32, m: u32, d: u32, base: u32, st: u32) {
    acmd(env, m, d & 0xFFFF, base);
    set_core_state(env, n, st);
}

/// C: `FUN_021A5EDC(n, m, trig, held, dir, flag)`: walk (acmd base 0x14 with `flag` 1, 0x58 running (held bit 1), else
/// 0xC), state 1.
pub fn act_walk<E: FieldEnv>(env: &mut E, n: u32, m: u32, held: u32, d: u32, flag: u32) {
    let base = if flag == 1 {
        0x14
    } else if held & 2 != 0 {
        0x58
    } else {
        0xC
    };
    act_base(env, n, m, d, base, 1);
}

/// C: `FUN_021A5F14` / `FUN_021A62D8` / `FUN_021A6454` (n, m, .., dir): turn (acmd base 0x28), state 2.
pub fn act_turn<E: FieldEnv>(env: &mut E, n: u32, m: u32, d: u32) {
    act_base(env, n, m, d, 0x28, 2);
}

/// C: `FUN_021A5F38` / `FUN_021A62FC` / `FUN_021A6478` (n, m, .., dir): bump (acmd base 0x1C), state 3 (CORE state 0), the
/// bump sound unless the tile in front is a warp ([`bump_ok`]).
pub fn act_bump<E: FieldEnv>(env: &mut E, n: u32, m: u32, d: u32) {
    acmd(env, m, d & 0xFFFF, 0x1C);
    env.w32(n, 3);
    let c = core(env, n);
    env.callv(CORE_SET_STATE, &[c, 0]);
    if bump_ok(env, n, d & 0xFFFF) == 0 {
        env.callv(SE_PLAY, &[SE_BUMP]);
    }
}

// ---- cycle

/// C: `FUN_021A5F98(n, value, trig, held, dir, flag)`: cycle action 0..5.
#[allow(clippy::too_many_arguments)]
pub fn cycle_action<E: FieldEnv>(
    env: &mut E,
    n: u32,
    v: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) {
    let m = mmdl(env, n);
    match v {
        0 => {}
        1 => act_stand(env, n, m, d),
        2 => act_base(env, n, m, d, 0x14, 1),
        3 => act_turn(env, n, m, d),
        4 => act_bump(env, n, m, d),
        5 => act_base(env, n, m, d, 0x38, 1),
        _ => {}
    }
    let _ = (trig, held, flag);
}

/// C: `FUN_021A6014(n, trig, held, dir, flag)`: cycle move value by state.
pub fn cycle_value<E: FieldEnv>(
    env: &mut E,
    n: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    let m = mmdl(env, n);
    match env.r32(n) {
        0 => v_stand(env, n, m, trig, held, d, flag, Form::Cycle),
        1 => cycle_v_walk(env, n, m, trig, held, d, flag),
        2 => v_turn(env, n, m, trig, held, d, flag, Form::Cycle),
        3 => v_bump(env, n, m, trig, held, d, flag, Form::Cycle),
        _ => 0,
    }
}

/// C: `FUN_021A60DC(n, m, trig, held, dir, flag)`: cycle moving value (as [`v_walk`], standing through the cycle procs).
#[allow(clippy::too_many_arguments)]
pub fn cycle_v_walk<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    if env.callv(M_ACMD_END, &[m]) == 0 {
        return 0;
    }
    if d == DIR_NONE {
        return v_stand(env, n, m, trig, held, DIR_NONE, flag, Form::Cycle);
    }
    let mut h = hit(env, m, d, 0);
    if flag == 1 && h & 2 != 0 {
        h = 0;
    }
    if h == 0 {
        return 2;
    }
    if h & 2 != 0 {
        return front_tile(env, m, d);
    }
    4
}

/// C: `FUN_021A607C(n, m, trig, held, dir, flag)`: cycle standing value.
#[allow(clippy::too_many_arguments)]
pub fn cycle_v_stand<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_stand(env, n, m, trig, held, d, flag, Form::Cycle)
}

/// C: `FUN_021A61B0(n, m, trig, held, dir, flag)`: cycle turning value.
#[allow(clippy::too_many_arguments)]
pub fn cycle_v_turn<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_turn(env, n, m, trig, held, d, flag, Form::Cycle)
}

/// C: `FUN_021A6200(n, m, trig, held, dir, flag)`: cycle bump value.
#[allow(clippy::too_many_arguments)]
pub fn cycle_v_bump<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_bump(env, n, m, trig, held, d, flag, Form::Cycle)
}

// ---- surf

/// C: `FUN_021A635C(n, value, trig, held, dir, flag)`: surf action 0..4 (the MMDL is the CORE's).
#[allow(clippy::too_many_arguments)]
pub fn surf_action<E: FieldEnv>(
    env: &mut E,
    n: u32,
    v: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) {
    let c = core(env, n);
    let m = env.callv(CORE_MMDL, &[c]);
    match v {
        0 => {}
        1 => act_stand(env, n, m, d),
        2 => act_base(env, n, m, d, if flag == 1 { 0x14 } else { 0x10 }, 1),
        3 => act_turn(env, n, m, d),
        4 => act_bump(env, n, m, d),
        _ => {}
    }
    let _ = (trig, held);
}

/// C: `FUN_021A64B4(n, trig, held, dir, flag)`: surf move value by state.
pub fn surf_value<E: FieldEnv>(
    env: &mut E,
    n: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    let c = core(env, n);
    let m = env.callv(CORE_MMDL, &[c]);
    match env.r32(n) {
        0 => v_stand(env, n, m, trig, held, d, flag, Form::Surf),
        1 => surf_v_walk(env, n, m, trig, held, d, flag),
        2 => v_turn(env, n, m, trig, held, d, flag, Form::Surf),
        3 => v_bump(env, n, m, trig, held, d, flag, Form::Surf),
        _ => 0,
    }
}

/// C: `FUN_021A658C(n, m, trig, held, dir, flag)`: surf moving value: 0 while the acmd runs, standing without input; the hit
/// check also reports the attribute in front (bit 0x10 of the answer allows `flag` 1); a clear check on an attribute with
/// flag bit 2 (or `flag` 1) is 2, everything else 4.
#[allow(clippy::too_many_arguments)]
pub fn surf_v_walk<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    if env.callv(M_ACMD_END, &[m]) == 0 {
        return 0;
    }
    if d & 0xFFFF == DIR_NONE {
        return v_stand(env, n, m, trig, held, DIR_NONE, flag, Form::Surf);
    }
    let l = env.mem().push_local(4);
    let mut h = hit(env, m, d & 0xFFFF, l);
    if flag == 1 && h != 0 && h & 0x10 == 0 {
        h = 0;
    }
    let a = env.r32(l);
    env.mem().pop_local(l);
    if a == 0xFFFF_FFFF {
        return 4;
    }
    env.callv(ATTR_VALUE, &[a]);
    let f = env.callv(ATTR_FLAGS, &[a]);
    if h != 0 {
        return 4;
    }
    if f & 2 != 0 || flag == 1 {
        return 2;
    }
    4
}

/// C: `FUN_021A652C(n, m, trig, held, dir, flag)`: surf standing value.
#[allow(clippy::too_many_arguments)]
pub fn surf_v_stand<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_stand(env, n, m, trig, held, d, flag, Form::Surf)
}

/// C: `FUN_021A661C(n, m, trig, held, dir, flag)`: surf turning value.
#[allow(clippy::too_many_arguments)]
pub fn surf_v_turn<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_turn(env, n, m, trig, held, d, flag, Form::Surf)
}

/// C: `FUN_021A666C(n, m, trig, held, dir, flag)`: surf bump value.
#[allow(clippy::too_many_arguments)]
pub fn surf_v_bump<E: FieldEnv>(
    env: &mut E,
    n: u32,
    m: u32,
    trig: u32,
    held: u32,
    d: u32,
    flag: u32,
) -> u32 {
    v_bump(env, n, m, trig, held, d, flag, Form::Surf)
}

// ---- flags

/// C: `FUN_021A66E4(n, bits)`: set flags.
pub fn set_flags<E: FieldEnv>(env: &mut E, n: u32, bits: u32) {
    let f = env.r32(n + 4);
    env.w32(n + 4, f | bits);
}

/// C: `FUN_021A66EC(n, bits)`: clear flags.
pub fn clear_flags<E: FieldEnv>(env: &mut E, n: u32, bits: u32) {
    let f = env.r32(n + 4);
    env.w32(n + 4, f & !bits);
}

/// C: `FUN_021A66F8(n, bits)`: any of the flags set.
pub fn has_flags<E: FieldEnv>(env: &mut E, n: u32, bits: u32) -> u32 {
    (env.r32(n + 4) & bits != 0) as u32
}

/// The thunks `021A6708 .. 021A6798` (set / clear one flag): (address, set, bits).
pub const FLAG_THUNKS: [(u32, bool, u32); 13] = [
    (0x021A_6708, true, 1),
    (0x021A_6714, false, 1),
    (0x021A_6720, true, 2),
    (0x021A_672C, false, 2),
    (0x021A_6738, false, 4),
    (0x021A_6744, true, 8),
    (0x021A_6750, false, 8),
    (0x021A_675C, true, 0x10),
    (0x021A_6768, false, 0x10),
    (0x021A_6774, true, 0x20),
    (0x021A_6780, false, 0x20),
    (0x021A_678C, true, 0x40),
    (0x021A_6798, false, 0x40),
];

/// C: `FUN_021A6708` / `021a6714` / `021a6720` / `021a672c` / `021a6738` / `021a6744` / `021a6750` / `021a675c` / `021a6768` /
/// `021a6774` / `021a6780` / `021a678c` / `021a6798`: set / clear one flag ([`FLAG_THUNKS`]).
pub fn flag_thunk<E: FieldEnv>(env: &mut E, addr: u32, n: u32) {
    let &(_, set, bits) = FLAG_THUNKS
        .iter()
        .find(|t| t.0 == addr)
        .expect("flag thunk");
    if set {
        set_flags(env, n, bits);
    } else {
        clear_flags(env, n, bits);
    }
}

// ---- slopes

/// C: `FUN_021A67A4(n)`: the slope kind of the MMDL's tile (0 none or while flag 2; 1 map attribute `021AB348`; 2..7 the
/// rail attributes 0xA5, 0xA4, 0xA7, 0xA6, 0xA8, 0xB0).
pub fn slope_kind<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    let m = mmdl(env, n);
    let a = env.callv(M_ATTR, &[m]);
    if has_flags(env, n, 2) != 0 {
        return 0;
    }
    let v = env.callv(ATTR_VALUE, &[a]);
    if env.callv(ATTR_AB348, &[v]) == 1 {
        return 1;
    }
    for (k, p) in RAIL_ATTR_PREDS.iter().enumerate() {
        if env.callv(*p, &[v]) == 1 {
            return 2 + k as u32;
        }
    }
    0
}

/// C: `FUN_021A682C(n, dir, x)`: the slope proc (ROM table 0x021D8ABC, kind 1..7) once the MMDL's acmd ended, else the
/// slide end ([`slope_end`]); returns the direction to use.
pub fn slope<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32) -> u32 {
    let m = mmdl(env, n);
    if env.callv(M_ACMD_END, &[m]) == 0 {
        return d;
    }
    if x == 0 {
        let k = slope_kind(env, n);
        if k != 0 {
            return match k {
                1 => slide(env, n, d, x, Slide::Plain),
                2 => slide(env, n, d, x, Slide::KindA),
                3 => slide(env, n, d, x, Slide::KindB),
                4 => slide_se(env, n, d, x, 2),
                5 => slide_se(env, n, d, x, 3),
                6 => slide_bounce(env, n, d, x),
                _ => hold(env, n, d, x),
            };
        }
    }
    slope_end(env, n);
    d
}

/// C: `FUN_021A6870(n)`: end a slide: undo the MMDL status, the flags and directions the slide set.
pub fn slope_end<E: FieldEnv>(env: &mut E, n: u32) {
    let m = mmdl(env, n);
    if has_flags(env, n, 0x59) != 0 {
        env.callv(M_STATUS_OFF, &[m, 24]);
    }
    if has_flags(env, n, 1) != 0 {
        clear_flags(env, n, 1);
        set_flags(env, n, 2);
        let c = core(env, n);
        env.callv(CORE_SET_STATE, &[c, 0]);
    }
    if has_flags(env, n, 0x18) != 0 {
        clear_flags(env, n, 8);
        clear_flags(env, n, 0x10);
        let d = env.callv(M_MOVE_DIR, &[m]);
        let r = env.callv(DIR_REVERSE, &[d]);
        env.callv(M_SET_MOVE_DIR, &[m, r]);
    }
    if has_flags(env, n, 0x20) != 0 {
        clear_flags(env, n, 0x20);
        env.callv(M_SET_MOVE_DIR, &[m, 3]);
    }
    if has_flags(env, n, 0x40) != 0 {
        clear_flags(env, n, 0x40);
        rec_init(env, n + 0xC);
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Slide {
    Plain,
    KindA,
    KindB,
}

/// C: `FUN_021A6908` (plain) / `FUN_021A6974` (kind A, flag 8) / `FUN_021A69D8` (kind B, flag 0x10) (n, dir, x): slide in
/// the MMDL's move direction unless blocked (then the slide ends and `dir` is kept): acmd base 0x10, status 24, state 1.
fn slide<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32, kind: Slide) -> u32 {
    let _ = x;
    let m = mmdl(env, n);
    let md = env.callv(M_MOVE_DIR, &[m]);
    if hit(env, m, md, 0) != 0 {
        slope_end(env, n);
        return d;
    }
    set_flags(env, n, 1);
    match kind {
        Slide::Plain => {
            clear_flags(env, n, 8);
            clear_flags(env, n, 0x10);
            env.callv(M_SET_DIR, &[m, md]);
        }
        Slide::KindA => {
            set_flags(env, n, 8);
            clear_flags(env, n, 0x10);
        }
        Slide::KindB => {
            set_flags(env, n, 0x10);
            clear_flags(env, n, 8);
        }
    }
    acmd(env, m, md, 0x10);
    env.callv(M_STATUS_ON, &[m, 24]);
    set_core_state(env, n, 1);
    md
}

/// C: `FUN_021A6908(n, dir, x)`: plain slide.
pub fn slide_plain<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32) -> u32 {
    slide(env, n, d, x, Slide::Plain)
}

/// C: `FUN_021A6974(n, dir, x)`: slide kind A.
pub fn slide_a<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32) -> u32 {
    slide(env, n, d, x, Slide::KindA)
}

/// C: `FUN_021A69D8(n, dir, x)`: slide kind B.
pub fn slide_b<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32) -> u32 {
    slide(env, n, d, x, Slide::KindB)
}

/// C: `FUN_021A6A3C` (move direction 2) / `FUN_021A6A64` (move direction 3) (n, dir, x): the slope sound when the MMDL
/// moves in that direction, then a plain slide.
pub fn slide_se<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32, dir: u32) -> u32 {
    let m = mmdl(env, n);
    if env.callv(M_MOVE_DIR, &[m]) == dir {
        env.callv(SE_PLAY, &[SE_SLOPE]);
    }
    slide(env, n, d, x, Slide::Plain)
}

/// C: `FUN_021A6A8C(n, dir, x)`: a plain slide, then flag 0x20 (bounce).
pub fn slide_bounce<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32) -> u32 {
    let r = slide(env, n, d, x, Slide::Plain);
    set_flags(env, n, 0x20);
    r
}

/// C: `FUN_021A6AA0(n, dir, x)`: hold tile: start holding (flag 0x40) the first time; while the input record says so the MMDL
/// runs acmd base 8 in the held direction (status 24, CORE state 1; returns 9), else status 24 off and the held direction.
pub fn hold<E: FieldEnv>(env: &mut E, n: u32, d: u32, x: u32) -> u32 {
    let _ = x;
    if has_flags(env, n, 0x40) == 0 {
        set_flags(env, n, 0x40);
        rec_hold(env, n + 0xC);
    }
    let d2 = rec_dir(env, n + 0xC, d) & 0xFFFF;
    let m = mmdl(env, n);
    if rec_bit1(env, n + 0xC) != 0 {
        env.callv(M_STATUS_ON, &[m, 24]);
        let code = env.callv(ACMD_CODE, &[d2, 8]);
        let m = mmdl(env, n);
        env.callv(M_ACMD_SET, &[m, code]);
        let c = core(env, n);
        env.callv(CORE_SET_STATE, &[c, 1]);
        return DIR_NONE;
    }
    env.callv(M_STATUS_OFF, &[m, 24]);
    d2
}

// ---- the sound record (+8)

/// C: `FUN_021A6B08(e, kind)`: start (sound 0x6D1 unless already playing this kind).
pub fn eff_start<E: FieldEnv>(env: &mut E, e: u32, kind: u32) {
    if env.r8(e) == 1 && kind == u32::from(env.r8(e + 1)) {
        return;
    }
    env.callv(SE_PLAY, &[SE_ROAD]);
    env.w8(e, 1);
    env.w8(e + 1, kind as u8);
    env.w16(e + 2, 0);
}

/// C: `FUN_021A6B30(e)`: stop (and stop the sound).
pub fn eff_stop<E: FieldEnv>(env: &mut E, e: u32) {
    if env.r8(e) == 0 {
        return;
    }
    env.w8(e, 0);
    env.w16(e + 2, 0);
    let h = env.callv(SE_HANDLE, &[SE_ROAD]);
    env.callv(SE_STOP, &[h]);
}

/// C: `FUN_021A6B50(e, m)`: while playing, spin the MMDL: the next of four directions of the kind's row ([`SPIN_DIRS`]).
pub fn eff_frame<E: FieldEnv>(env: &mut E, e: u32, m: u32) {
    if env.r8(e) == 0 {
        return;
    }
    let c = i32::from(env.r16(e + 2) as i16)
        .wrapping_add(1)
        .wrapping_rem(4);
    env.w16(e + 2, c as u16);
    let kind = env.r8(e + 1) as usize;
    let i = env.r16(e + 2) as i16;
    let dir = SPIN_DIRS[kind][i as usize];
    env.callv(M_SET_SPIN, &[m, u32::from(dir)]);
}

// ---- the input record (+0xC)

/// C: `FUN_021A6B88(r)`: clear (8 bytes), held direction 9.
pub fn rec_init<E: FieldEnv>(env: &mut E, r: u32) {
    env.mem().fill(r, 8, 0);
    env.w32(r + 4, DIR_NONE);
}

/// C: `FUN_021A6B9C(r)`: start holding (bit 0).
pub fn rec_hold<E: FieldEnv>(env: &mut E, r: u32) {
    let b = env.r8(r);
    env.w8(r, b | 1);
}

/// C: `FUN_021A6BAC(r)`: thunk to [`rec_init`].
pub fn rec_reset<E: FieldEnv>(env: &mut E, r: u32) {
    rec_init(env, r);
}

/// C: `FUN_021A6BB4(r, dir, m)`: one frame of the hold: with a pending count (bits 3..7) a direction resets it (bit 1:
/// remember the direction); without one, 30 idle frames set the count to 1; then, once the MMDL's acmd ended, bits 1 / 2
/// follow the count.
pub fn rec_input<E: FieldEnv>(env: &mut E, r: u32, d: u32, m: u32) {
    let b = env.r8(r);
    if b & 1 == 0 {
        return;
    }
    if b >> 3 != 0 {
        if d != DIR_NONE {
            env.w16(r + 2, 0);
            env.w8(r, b & !0xF8);
            if env.r8(r) & 2 != 0 {
                env.w32(r + 4, d);
            }
        }
    } else if d == DIR_NONE {
        let c = env.r16(r + 2) as i16;
        if c < 30 {
            env.w16(r + 2, c.wrapping_add(1) as u16);
        } else {
            env.w8(r, (b & !0xF8) | 8);
        }
    }
    if env.callv(M_ACMD_END, &[m]) != 1 {
        return;
    }
    let b = env.r8(r);
    if b >> 3 == 0 {
        env.w8(r, b & !2);
        let b = env.r8(r);
        env.w8(r, b & !4);
    } else {
        env.w8(r, b | 2);
        let b = env.r8(r);
        env.w8(r, b | 4);
    }
}

/// C: `FUN_021A6C30(r)`: end of frame: clear bit 2; without bit 1 the held direction is 9.
pub fn rec_end<E: FieldEnv>(env: &mut E, r: u32) {
    let b = env.r8(r);
    if b & 4 != 0 {
        env.w8(r, b & !4);
    }
    if env.r8(r) & 2 == 0 {
        env.w32(r + 4, DIR_NONE);
    }
}

/// C: `FUN_021A6C4C(r, dir)`: 1 while bit 2, else the held direction (9: `dir`).
pub fn rec_dir<E: FieldEnv>(env: &mut E, r: u32, d: u32) -> u32 {
    if env.r8(r) & 4 != 0 {
        return 1;
    }
    let h = env.r32(r + 4);
    if h == DIR_NONE {
        d
    } else {
        h
    }
}

/// C: `FUN_021A6C64(r)`: bit 1.
pub fn rec_bit1<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    u32::from(env.r8(r) >> 1) & 1
}

/// C: `FUN_021A6C6C(n, dir)`: the rail location in front has an event (eventdata `02162598` answers other than 0xFFFF).
pub fn bump_ok<E: FieldEnv>(env: &mut E, n: u32, dir: u32) -> u32 {
    let c = core(env, n);
    let m = env.callv(CORE_MMDL, &[c]);
    let fm = env.r32(n + 0x18);
    let gs = env.callv(0x0218_8C80, &[fm]);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let ed = env.callv(0x0201_2844, &[gd]);
    let mut loc = [0u8; 8];
    env.call(
        MR_FRONT,
        &mut [CArg::V(m), CArg::V(dir), CArg::OutBuf(&mut loc)],
    );
    let r = env.call(0x0216_2598, &mut [CArg::V(ed), CArg::InBuf(&loc)]);
    (r != 0xFFFF) as u32
}

#[cfg(test)]
mod tests;
