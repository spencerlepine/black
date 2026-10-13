//! `fldmmdl_acmd.c`, part 2e (overlay 21, `0x02199888..0x0219A098`; F4b job 2): the arc moves (02199888 and its 12 init steps,
//! update 02199970) and the special codes 0xAB..0xB4 / 0xBC (counters, a slow step left, waits on the 3D model animation).
//!
//! F4b-spec2 specced all 32 functions (`tools/rom-diff/specs/black-field/acmd2e.spec`); this module holds the smoke ports.

use crate::acmd2a::{
    ACMD_WORK, ACMD_WORK_INIT, GRID_STEP, INC_SEQ, PLAY_SE, SAVE_OLD_GRID, SET_ANIM, SET_DIR_DISP,
    SET_DRAW_OFFSET, SET_MOVE_BITS,
};
use crate::acmd2b::{vec_bytes, JUMP_HEIGHTS};
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_9888,
    0x0219_9970,
    0x0219_9AEC,
    0x0219_9CE4,
    0x0219_9D14,
    0x0219_9F94,
];

/// `FUN_0216D600(m, dir)`: move dir = dir.
const SET_DIR_MOVE: u32 = 0x0216_D600;
/// `FUN_0216DA70(m)`: move bit 0x20 on.
const SET_BIT20: u32 = 0x0216_DA70;
/// Grid getters x / y / z (`FUN_0216D83C` / `FUN_0216D854` / `FUN_0216D86C`).
const GRID: [u32; 3] = [0x0216_D83C, 0x0216_D854, 0x0216_D86C];
/// Old-grid setters x / y / z (`FUN_0216D80C` / `FUN_0216D818` / `FUN_0216D824`).
const SET_OLD_GRID: [u32; 3] = [0x0216_D80C, 0x0216_D818, 0x0216_D824];
/// Grid adders x / y / z (`FUN_0216D848` / `FUN_0216D860` / `FUN_0216D87C`).
const ADD_GRID: [u32; 3] = [0x0216_D848, 0x0216_D860, 0x0216_D87C];
/// `FUN_0216D88C(m, &out)`: the position VecFx32.
const GET_POS: u32 = 0x0216_D88C;
/// `FUN_0216D89C(m, &v)`: position = `*v`.
const SET_POS: u32 = 0x0216_D89C;
/// `FUN_0216D598(m)`: the OBJ code.
const OBJ_CODE: u32 = 0x0216_D598;
/// `FUN_0219FCC0(m, &kind, &frame)` (fldmmdl_g3dobj): the 3D model animation frame; 1 when valid.
const MODEL_FRAME: u32 = 0x0219_FCC0;
/// Move bits a finished arc sets: 0x400 (done) | 0x100 | 0x40.
const LANDED: u32 = 0x540;

/// The grid step of an arc move on `axis` (0 x, 1 y, 2 z): one tile by the sign of `dist` (y moves by two units).
fn arc_grid_step<E: FieldEnv>(env: &mut E, m: u32, axis: u8, dist: i32) {
    let s: i32 = if dist < 0 { -1 } else { 1 };
    match axis {
        0 => env.callv(ADD_GRID[0], &[m, s as u32]),
        1 => env.callv(ADD_GRID[1], &[m, (2 * s) as u32]),
        2 => env.callv(ADD_GRID[2], &[m, s as u32]),
        _ => 0,
    };
}

/// Old grid x / y / z = grid x / y / z.
fn save_old_grid_xyz<E: FieldEnv>(env: &mut E, m: u32) {
    for k in 0..3 {
        let g = env.callv(GRID[k], &[m]);
        env.callv(SET_OLD_GRID[k], &[m, g]);
    }
}

/// C: `FUN_02199888` (unlabeled routine at `0x02199888`): start an arc move: acmd work (0x10) = `u8 frames`, `u8 arc_axis`,
/// `u8 axis`, `u8 neg`, `s32 dist` (fx32 per frame), `u16 +0xE` = 0x1000 / frames; display dir, move dir (u16), draw animation code
/// (u8), move bit 0x20, old grid = grid; `dist` != 0 steps the grid one tile on `axis` by its sign; acmd seq + 1. The stack
/// arguments are read as bytes. UNVERIFIED: frames 0 (as a byte) panics (the original's division routine 0209C0A4 is not tested
/// with 0).
#[allow(clippy::too_many_arguments)]
pub fn arc_start<E: FieldEnv>(
    env: &mut E,
    m: u32,
    dist: u32,
    disp: u32,
    mv: u32,
    frames: u32,
    anim: u32,
    axis: u32,
    arc_axis: u32,
    neg: u32,
) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0x10]);
    env.w8(w, frames as u8);
    env.w32(w.wrapping_add(4), dist);
    env.w8(w.wrapping_add(2), axis as u8);
    env.w8(w.wrapping_add(1), arc_axis as u8);
    env.w8(w.wrapping_add(3), neg as u8);
    let step = 0x1000 / i32::from(env.r8(w));
    env.w16(w.wrapping_add(0xE), step as u16);
    env.callv(SET_DIR_DISP, &[m, disp & 0xFFFF]);
    env.callv(SET_DIR_MOVE, &[m, mv & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim & 0xFF]);
    env.callv(SET_BIT20, &[m]);
    save_old_grid_xyz(env, m);
    if dist != 0 {
        arc_grid_step(env, m, axis as u8, dist as i32);
    }
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02199970` (unlabeled routine at `0x02199970`): arc move update: the position moves by dist on the axis; the distance
/// grows by |dist| and the phase by the step (capped at 0xF00); the draw offset on the arc axis is the low jump height
/// ([`JUMP_HEIGHTS`]`[1][phase >> 8]`, negated when `neg` is 1); frames - 1; every whole tile of distance while frames are left
/// moves the grid by the sign of dist (also for dist 0: +1); the last frame clears the draw offset, sets move bits 0x540, old grid
/// = grid, draw animation code 0, acmd seq + 1, and returns 1; else 0.
pub fn arc_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut pos = [0u8; 12];
    env.call(GET_POS, &mut [CArg::V(m), CArg::OutBuf(&mut pos)]);
    let axis = env.r8(w.wrapping_add(2));
    if axis < 3 {
        let o = 4 * usize::from(axis);
        let c = u32::from_le_bytes([pos[o], pos[o + 1], pos[o + 2], pos[o + 3]]);
        let c = c.wrapping_add(env.r32(w.wrapping_add(4)));
        pos[o..o + 4].copy_from_slice(&c.to_le_bytes());
    }
    env.call(SET_POS, &mut [CArg::V(m), CArg::InBuf(&pos)]);
    let ad = (env.r32(w.wrapping_add(4)) as i32).wrapping_abs() as u32;
    let d = env.r32(w.wrapping_add(8)).wrapping_add(ad);
    env.w32(w.wrapping_add(8), d);
    let mut phase = env
        .r16(w.wrapping_add(0xC))
        .wrapping_add(env.r16(w.wrapping_add(0xE)));
    if phase > 0xF00 {
        phase = 0xF00;
    }
    env.w16(w.wrapping_add(0xC), phase);
    let mut y = JUMP_HEIGHTS[1][usize::from(phase >> 8)];
    if env.r8(w.wrapping_add(3)) == 1 {
        y = y.wrapping_neg();
    }
    let mut off = [0u32; 3];
    let a = env.r8(w.wrapping_add(1));
    if a < 3 {
        off[usize::from(a)] = y;
    }
    let v = vec_bytes(off[0], off[1], off[2]);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    let n = env.r8(w).wrapping_sub(1);
    env.w8(w, n);
    let d = env.r32(w.wrapping_add(8));
    if d as i32 >= 0x1_0000 && n != 0 {
        env.w32(w.wrapping_add(8), d.wrapping_sub(0x1_0000));
        let dist = env.r32(w.wrapping_add(4)) as i32;
        save_old_grid_xyz(env, m);
        let axis = env.r8(w.wrapping_add(2));
        arc_grid_step(env, m, axis, dist);
    }
    if env.r8(w) != 0 {
        return 0;
    }
    let v = [0u8; 12];
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    env.callv(SET_MOVE_BITS, &[m, LANDED]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199AEC` (unlabeled routine at `0x02199AEC`): acmd code 0x85: [`arc_start`]`(m, 0x2000, 2, 0, 8, 3, 1, 1, 0)`
/// (x += 0x2000 per frame for 8 frames, the arc on y); returns 1.
pub fn arc_85<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 2, 0, 8, 3, 1, 1, 0);
    1
}

/// C: `FUN_02199CE4` (unlabeled routine at `0x02199CE4`): acmd code 0xAB update: frame + 1; above 0x15, acmd seq + 1. Returns 0.
pub fn count_22<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let n = env.r32(w).wrapping_add(1);
    env.w32(w, n);
    if n as i32 > 0x15 {
        env.callv(INC_SEQ, &[m]);
    }
    0
}

/// C: `FUN_02199D14` (unlabeled routine at `0x02199D14`): acmd code 0xAD: start a slow step left (acmd work = anim 0xC, frames 2,
/// dir 2, speed 0x8000; the grid steps one tile left; draw animation code = the stored anim); acmd seq + 1; returns 1.
pub fn slow_left_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0xC]);
    env.w16(w, 0xC);
    env.w16(w.wrapping_add(2), 2);
    env.w32(w.wrapping_add(4), 2);
    env.w32(w.wrapping_add(8), 0x8000);
    env.callv(GRID_STEP, &[m, 2]);
    let anim = u32::from(env.r16(w));
    env.callv(SET_ANIM, &[m, anim]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199F94` (unlabeled routine at `0x02199F94`): acmd code 0xBC update: while the 3D model animation frame is valid,
/// count frames (u16 +0); for OBJ code 0xDD with animation kind 0, a new frame (frame - 1 != u8 +2) is stored and frame 0 / 3
/// play SE 0x7E5 / 0x7E6; from 0x5F counted frames on, acmd seq + 1, returns 1; else 0.
pub fn model_wait<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut kind = [0u8; 2];
    let mut frame = [0u8; 2];
    let ok = env.call(
        MODEL_FRAME,
        &mut [
            CArg::V(m),
            CArg::OutBuf(&mut kind),
            CArg::OutBuf(&mut frame),
        ],
    );
    if ok != 1 {
        return 0;
    }
    let c = env.r16(w).wrapping_add(1);
    env.w16(w, c);
    let frame = u16::from_le_bytes(frame).wrapping_sub(1);
    let kind = u16::from_le_bytes(kind);
    if env.callv(OBJ_CODE, &[m]) == 0xDD
        && kind == 0
        && u16::from(env.r8(w.wrapping_add(2))) != frame
    {
        env.w8(w.wrapping_add(2), frame as u8);
        match frame {
            0 => {
                env.callv(PLAY_SE, &[0x7E5]);
            }
            3 => {
                env.callv(PLAY_SE, &[0x7E6]);
            }
            _ => {}
        }
    }
    if env.r16(w) < 0x5F {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

#[cfg(test)]
mod tests;
