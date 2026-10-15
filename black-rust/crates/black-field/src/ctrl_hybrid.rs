//! `fieldmap_ctrl_hybrid.c` (overlay 21, `0x021A6DC4..0x021A7358`; F3b): the MAP CTRL of maps that mix the grid and the rails
//! (the hybrid entry of the MAP CTRL table, ROM 0x021D8AF0): the player walks on the grid until it steps onto a rail entry
//! (map attribute 0xA3 ahead and a rail location found below / above it), and back.
//!
//! HYBRID work H (0xC, `Heap_AllocDebug(heap, 0xC, 1, "fieldmap_ctrl_hybrid.c", 0xBD)`): `+0` mode (0 grid, 1 rail, 2 not yet
//! set), `+4` the FIELD_PLAYER, `+8` "keep the move direction" (rail mode). Frame procs by mode: ROM 0x021DDF38 (grid
//! [`frame_grid`], rail [`frame_rail`]). Switching ([`set_mode`]) moves the player MMDL between the grid header (overlay-21 data
//! 0x021DDF40, `+2` = OBJ code) at the grid position and the rail system, and the camera with it.
//!
//! Every callee (FIELDMAP getters, FIELD_PLAYER, MMDL, map attributes, the rail system, the camera, pad, heap) is reached through
//! [`FieldEnv::call`] by address. Verified by `tools/check-fn.sh black-field ctrl_hybrid --per-function` (spec
//! `tools/rom-diff/specs/black-field/ctrl_hybrid.spec`).

use crate::ctrl_nogrid::{
    FM_CAMERA, FM_CTRL_WORK, FM_GAMESYS, FM_HEAP, FM_MAPPER, FM_PLAYER, FM_SET_CTRL_WORK,
    FM_SET_TARGET, M_POS_PTR, PAD_HELD, PAD_TRIG, PL_MMDL,
};
use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_6DC4,
    0x021A_6E30,
    0x021A_6F18,
    0x021A_6F24,
    0x021A_6F4C,
    0x021A_6F5C,
    0x021A_7010,
    0x021A_70D0,
    0x021A_712C,
    0x021A_7198,
    0x021A_71E4,
    0x021A_7258,
];

/// Address of the string `"fieldmap_ctrl_hybrid.c"`.
pub const FILE_CTRL_HYBRID: u32 = 0x021D_DF64;
/// The grid MMDL header in overlay 21 data (`+2` = OBJ code).
pub const GRID_HEADER: u32 = 0x021D_DF40;

const PL_POS: u32 = 0x021A_3038;
const PL_SET_POS: u32 = 0x021A_3044;
const PL_SET_DIR: u32 = 0x021A_305C;
const PL_KEY_DIR: u32 = 0x021A_2FA4;
const PL_2FB0: u32 = 0x021A_2FB0;
const PL_ATTR_HERE: u32 = 0x021A_30C8;
const PL_ATTR_FRONT: u32 = 0x021A_3128;
const PL_RUN_OK: u32 = 0x021A_35BC;
const PL_GRID_FRAME: u32 = 0x021A_35F0;
const PL_RAIL_FRAME: u32 = 0x021A_3674;
const PL_KEEP_DIR: u32 = 0x021A_3698;
const M_ACMD_END: u32 = 0x0216_D3CC;
const M_MOVE_DIR: u32 = 0x0216_D608;
const M_DIR: u32 = 0x0216_D5F8;
const BLACT_FRAMES: u32 = 0x0219_F058;
const ATTR_VALUE: u32 = 0x021A_B0F0;
const ATTR_HYBRID_OK: u32 = 0x021A_B11C;
const RAIL_ATTR_A3: u32 = 0x021B_B1A0;

/// C: `FUN_021A6DC4(h, fm)`: re-enter the current mode's place (grid: the rail location below the player; rail: the grid
/// tile ahead) when one is found; clears `+8`.
pub fn reenter<E: FieldEnv>(env: &mut E, h: u32, fm: u32) {
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    let d = env.callv(M_DIR, &[m]);
    let l = env.mem().push_local(12);
    if env.r32(h) == 0 {
        if rail_entry(env, fm, h, l, d) != 0 {
            to_rail(env, fm, h, d, 0, l, 0, 0);
        }
    } else if grid_entry(env, fm, h, l, d) != 0 {
        to_grid(env, fm, h, d, 0, l, 0, 0);
    }
    env.mem().pop_local(l);
    env.w32(h + 8, 0);
}

/// C: `FUN_021A6E30(fm, pos, dir)`: create: the work (mode 2), then the mode of the PLAYER record (0 grid at `pos`, else rail
/// at its rail location), facing `dir`; the camera target follows the MMDL.
pub fn create<E: FieldEnv>(env: &mut E, fm: u32, pos: u32, dir: u32) {
    let gs = env.callv(FM_GAMESYS, &[fm]);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let rec = env.callv(0x0201_27A4, &[gd]);
    let heap = env.callv(FM_HEAP, &[fm]);
    let h = env.callv(HEAP_ALLOC, &[heap, 0xC, 1, FILE_CTRL_HYBRID, 0xBD]);
    env.callv(FM_SET_CTRL_WORK, &[fm, h]);
    let pl = env.callv(FM_PLAYER, &[fm]);
    env.w32(h + 4, pl);
    let kind = env.callv(0x0201_2AC0, &[rec]);
    env.w32(h, 2);
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    let (a, b) = blact_frames(env, m);
    if kind == 0 {
        let pl = env.r32(h + 4);
        env.callv(0x021A_35E0, &[pl, heap]);
        let pl = env.r32(h + 4);
        env.callv(0x021A_3664, &[pl, heap]);
        let pl = env.r32(h + 4);
        env.callv(PL_SET_DIR, &[pl, dir]);
        set_mode(env, fm, h, kind, pos, dir, a, b);
    } else {
        let loc = env.callv(0x0201_2AB8, &[rec]);
        let pl = env.r32(h + 4);
        env.callv(0x021A_3664, &[pl, heap]);
        let pl = env.r32(h + 4);
        env.callv(0x021A_35E0, &[pl, heap]);
        let pl = env.r32(h + 4);
        env.callv(PL_SET_DIR, &[pl, dir]);
        set_mode(env, fm, h, kind, loc, dir, a, b);
    }
    let p = env.callv(M_POS_PTR, &[m]);
    env.callv(FM_SET_TARGET, &[fm, p]);
    let pl = env.r32(h + 4);
    env.call(PL_POS, &mut [CArg::V(pl), CArg::Out(pos, 12)]);
}

/// C: `FUN_021A6F18(fm)`: free the work.
pub fn delete<E: FieldEnv>(env: &mut E, fm: u32) {
    let h = env.callv(FM_CTRL_WORK, &[fm]);
    env.callv(HEAP_FREE, &[h]);
}

/// C: `FUN_021A6F24(fm, out)`: one frame: the mode's proc (ROM 0x021DDF38), then the player position to `out`.
pub fn frame<E: FieldEnv>(env: &mut E, fm: u32, out: u32) {
    let h = env.callv(FM_CTRL_WORK, &[fm]);
    match env.r32(h) {
        0 => frame_grid(env, fm, h),
        1 => frame_rail(env, fm, h),
        m => panic!("UNVERIFIED: hybrid mode {m} reads past the ROM proc table 0x021DDF38"),
    }
    let pl = env.r32(h + 4);
    env.call(PL_POS, &mut [CArg::V(pl), CArg::Out(out, 12)]);
}

/// C: `FUN_021A6F4C(fm)`: the player MMDL's position (pointer).
pub fn pos<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let pl = env.callv(FM_PLAYER, &[fm]);
    let m = env.callv(PL_MMDL, &[pl]);
    env.callv(M_POS_PTR, &[m])
}

/// The common test of both frames: the MMDL is free (or the player says so), a direction is pressed, the tile here is a
/// rail entry (0xA3) and the attribute ahead allows the switch.
fn switch_ok<E: FieldEnv>(env: &mut E, h: u32, m: u32, d: u32) -> bool {
    let free = env.callv(M_ACMD_END, &[m]) == 1 || {
        let pl = env.r32(h + 4);
        env.callv(PL_2FB0, &[pl]) == 1
    };
    if !free || d == 9 {
        return false;
    }
    let pl = env.r32(h + 4);
    let here = env.callv(PL_ATTR_HERE, &[pl]);
    let pl = env.r32(h + 4);
    let front = env.callv(PL_ATTR_FRONT, &[pl, d]);
    let v = env.callv(ATTR_VALUE, &[here]);
    if env.callv(RAIL_ATTR_A3, &[v]) == 0 {
        return false;
    }
    env.callv(ATTR_HYBRID_OK, &[front]) != 0
}

/// C: `FUN_021A6F5C(fm, h)`: grid frame: switch to the rails when stepping onto a rail entry with a rail location below;
/// else the grid player frame (running when allowed).
pub fn frame_grid<E: FieldEnv>(env: &mut E, fm: u32, h: u32) {
    let t = env.callv(PAD_TRIG, &[]);
    let held = env.callv(PAD_HELD, &[]);
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    let pl = env.r32(h + 4);
    let d = env.callv(PL_KEY_DIR, &[pl, held]);
    if switch_ok(env, h, m, d) {
        let l = env.mem().push_local(8);
        if rail_entry(env, fm, h, l, d) != 0 {
            to_rail(env, fm, h, d, 1, l, t, held);
            env.mem().pop_local(l);
            return;
        }
        env.mem().pop_local(l);
    }
    let pl = env.r32(h + 4);
    let run = (env.callv(PL_RUN_OK, &[pl]) == 1) as u32;
    let pl = env.r32(h + 4);
    env.callv(PL_GRID_FRAME, &[pl, t, held, run]);
}

/// C: `FUN_021A7010(fm, h)`: rail frame: switch to the grid when stepping off onto a rail entry with a free grid tile ahead
/// (the direction is the pad's, or the MMDL's move direction while `+8`); else the rail player frame (`+8` from the player).
pub fn frame_rail<E: FieldEnv>(env: &mut E, fm: u32, h: u32) {
    let t = env.callv(PAD_TRIG, &[]);
    let held = env.callv(PAD_HELD, &[]);
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    let (d, keep) = if env.r32(h + 8) == 0 {
        let pl = env.r32(h + 4);
        (env.callv(PL_KEY_DIR, &[pl, held]), 1)
    } else {
        (env.callv(M_MOVE_DIR, &[m]), 0)
    };
    if switch_ok(env, h, m, d) {
        let l = env.mem().push_local(12);
        if grid_entry(env, fm, h, l, d) != 0 {
            to_grid(env, fm, h, d, keep, l, t, held);
            env.mem().pop_local(l);
            env.w32(h + 8, 0);
            return;
        }
        env.mem().pop_local(l);
    }
    let pl = env.r32(h + 4);
    let k = env.callv(PL_KEEP_DIR, &[pl]);
    env.w32(h + 8, k);
    let pl = env.r32(h + 4);
    env.callv(PL_RAIL_FRAME, &[pl, t, held]);
}

/// The two u16 animation values of the MMDL's billboard actor (`0219F058`).
fn blact_frames<E: FieldEnv>(env: &mut E, m: u32) -> (u32, u32) {
    let mut fr = [0u8; 4];
    let (x, y) = fr.split_at_mut(2);
    env.call(
        BLACT_FRAMES,
        &mut [CArg::V(m), CArg::OutBuf(x), CArg::OutBuf(y)],
    );
    (
        u32::from(u16::from_le_bytes([fr[0], fr[1]])),
        u32::from(u16::from_le_bytes([fr[2], fr[3]])),
    )
}

/// C: `FUN_021A70D0(fm, h, dir, keep, loc, trig, held)`: to the rails at `loc` (the MMDL's facing without `keep`).
#[allow(clippy::too_many_arguments)]
pub fn to_rail<E: FieldEnv>(
    env: &mut E,
    fm: u32,
    h: u32,
    d: u32,
    keep: u32,
    loc: u32,
    trig: u32,
    held: u32,
) {
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    let (a, b) = blact_frames(env, m);
    let pl = env.r32(h + 4);
    env.callv(PL_GRID_FRAME, &[pl, 0, 0, 0]);
    let d = if keep == 0 { env.callv(M_DIR, &[m]) } else { d };
    set_mode(env, fm, h, 1, loc, d, a, b);
    let pl = env.r32(h + 4);
    env.callv(PL_RAIL_FRAME, &[pl, trig, held]);
}

/// C: `FUN_021A712C(fm, h, dir, keep, pos, trig, held)`: to the grid at `pos` (the MMDL's facing without `keep`).
#[allow(clippy::too_many_arguments)]
pub fn to_grid<E: FieldEnv>(
    env: &mut E,
    fm: u32,
    h: u32,
    d: u32,
    keep: u32,
    pos: u32,
    trig: u32,
    held: u32,
) {
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    let (a, b) = blact_frames(env, m);
    let pl = env.r32(h + 4);
    env.callv(PL_RAIL_FRAME, &[pl, 0, 0]);
    let d = if keep == 0 { env.callv(M_DIR, &[m]) } else { d };
    set_mode(env, fm, h, 0, pos, d, a, b);
    let pl = env.r32(h + 4);
    let run = (env.callv(PL_RUN_OK, &[pl]) == 1) as u32;
    let pl = env.r32(h + 4);
    env.callv(PL_GRID_FRAME, &[pl, trig, held, run]);
}

/// C: `FUN_021A7198(fm, h, out, dir)`: the grid position one tile ahead of the player (`out` = the player position); 1 when
/// its attribute allows the switch to the grid (`021AB11C` answers 0).
pub fn grid_entry<E: FieldEnv>(env: &mut E, fm: u32, h: u32, out: u32, d: u32) -> u32 {
    let mp = env.callv(0x0218_8C98, &[fm]);
    let pl = env.r32(h + 4);
    env.call(PL_POS, &mut [CArg::V(pl), CArg::Out(out, 12)]);
    let p = env.mem().push_local(12);
    env.mem().copy(p, out, 12);
    env.call(
        0x0216_4208,
        &mut [CArg::V(d), CArg::In(p, 12), CArg::V(0x10000)],
    );
    let a = env.call(0x021A_B0D4, &mut [CArg::V(mp), CArg::In(p, 12)]);
    env.mem().pop_local(p);
    (env.callv(ATTR_HYBRID_OK, &[a]) == 0) as u32
}

/// C: `FUN_021A71E4(fm, h, loc, dir)`: the rail location below the player (the segment from 4.0 above to 4.0 below its
/// position hits a line, `021B7A84`), stepped once in `dir` (`021B7CD8`); the hit location to `loc` (stale stack words of the
/// original without a hit). Returns the step's answer, 0 without a hit.
pub fn rail_entry<E: FieldEnv>(env: &mut E, fm: u32, h: u32, loc: u32, d: u32) -> u32 {
    let mp = env.callv(FM_MAPPER, &[fm]);
    let r = env.callv(0x021B_B480, &[mp]);
    let l = env.mem().push_local(0x38);
    let (p_lo, p_hi, hit, step, pos) = (l + 32, l + 20, l + 12, l + 4, l + 44);
    let pl = env.r32(h + 4);
    env.call(PL_POS, &mut [CArg::V(pl), CArg::Out(p_lo, 12)]);
    let x = env.r32(p_lo);
    let y = env.r32(p_lo + 4);
    let z = env.r32(p_lo + 8);
    env.w32(p_hi, x);
    env.w32(p_hi + 4, y.wrapping_add(0x4000));
    env.w32(p_hi + 8, z);
    env.w32(p_lo + 4, y.wrapping_sub(0x4000));
    let mut res = env.call(
        0x021B_7A84,
        &mut [
            CArg::V(r),
            CArg::In(p_lo, 12),
            CArg::In(p_hi, 12),
            CArg::Out(hit, 8),
            CArg::Out(pos, 12),
        ],
    );
    if res != 0 {
        let k = env.callv(0x0201_441C, &[d]);
        res = env.call(
            0x021B_7CD8,
            &mut [CArg::V(r), CArg::In(hit, 8), CArg::V(k), CArg::Out(step, 8)],
        );
    }
    for i in 0..4 {
        let v = env.r16(hit + 2 * i);
        env.w16(loc + 2 * i, v);
    }
    env.mem().pop_local(l);
    res
}

/// `div16` as the original computes the grid coordinate: `((x >> 12) + adj) << 12 >> 16` (wrapping).
fn grid_coord(v: u32) -> u32 {
    let t = v as i32 >> 12;
    let a = t.wrapping_add(((t >> 3) as u32 >> 28) as i32);
    (a.wrapping_shl(12) >> 16) as u32
}

/// C: `FUN_021A7258(fm, h, mode, at, dir, a, b)`: switch to `mode` (nothing when already in it): cancel a running acmd; grid:
/// the MMDL gets the grid header (its OBJ code kept) at the grid tile of the position `at`, leaves the rails, the camera goes
/// back to normal; rail: the player goes to the rail location `at` and becomes the mapper's current RAIL WORK. Then the
/// direction and the billboard animation values are restored.
#[allow(clippy::too_many_arguments)]
pub fn set_mode<E: FieldEnv>(
    env: &mut E,
    fm: u32,
    h: u32,
    mode: u32,
    at: u32,
    d: u32,
    a: u32,
    b: u32,
) {
    let mp = env.callv(FM_MAPPER, &[fm]);
    let cam = env.callv(FM_CAMERA, &[fm]);
    if env.r32(h) == mode {
        return;
    }
    let pl = env.r32(h + 4);
    let m = env.callv(PL_MMDL, &[pl]);
    if env.callv(0x0216_DC4C, &[m]) != 0 {
        env.callv(0x0216_D4B4, &[m]);
    }
    if mode == 0 {
        let v = env.mem().push_local(12);
        env.mem().copy(v, at, 12);
        let code = env.callv(0x0216_D598, &[m]);
        env.w16(GRID_HEADER + 2, code as u16);
        let pl = env.r32(h + 4);
        env.callv(0x021A_368C, &[pl]);
        env.callv(0x0216_E0AC, &[m, GRID_HEADER]);
        let gx = grid_coord(env.r32(v));
        let gy = grid_coord(env.r32(v + 4));
        let gz = grid_coord(env.r32(v + 8));
        env.callv(0x0216_E084, &[m, gx, gy, gz, d & 0xFFFF]);
        env.call(0x0216_D88C, &mut [CArg::V(m), CArg::Out(v, 12)]);
        let pl = env.r32(h + 4);
        env.call(PL_SET_POS, &mut [CArg::V(pl), CArg::In(v, 12)]);
        env.mem().pop_local(v);
        env.callv(0x021B_B3B8, &[mp]);
        env.callv(0x0218_E13C, &[cam, 0]);
        env.callv(0x0218_EC60, &[cam]);
        env.callv(0x0218_E160, &[cam]);
    } else {
        let pl = env.r32(h + 4);
        env.call(0x021A_3680, &mut [CArg::V(pl), CArg::In(at, 8)]);
        let pl = env.r32(h + 4);
        let rw = env.callv(0x021A_36A4, &[pl]);
        env.callv(0x021B_B3AC, &[mp, rw]);
        env.callv(0x0218_F05C, &[cam]);
    }
    env.callv(0x0216_D5E0, &[m, d & 0xFFFF]);
    env.callv(0x0216_D600, &[m, d & 0xFFFF]);
    env.callv(0x0219_F0B0, &[m, a & 0xFFFF, b & 0xFFFF]);
    env.w32(h, mode);
}

#[cfg(test)]
mod tests;
