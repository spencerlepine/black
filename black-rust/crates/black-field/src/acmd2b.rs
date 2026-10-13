//! `fldmmdl_acmd.c`, part 2b (overlay 21, `0x02198318..0x02198960`; F4b job 2): walking in place (02198318 and its 20 init steps,
//! update 0219834C) and the jumps (021984C0 / 0219852C and their 24 init steps, update 02198550 with the jump-height tables).
//!
//! F4b-spec2 specced all 49 functions (`tools/rom-diff/specs/black-field/acmd2b.spec`); this module holds the smoke ports.

use crate::acmd2a::{
    ACMD_DONE, ACMD_WORK, ACMD_WORK_INIT, GRID_STEP, INC_SEQ, MOVING, PLAY_SE, POS_HEIGHT,
    POS_MOVE, SAVE_OLD_GRID, SET_ANIM, SET_DIR, SET_DIR_DISP, SET_DRAW_OFFSET, SET_MOVE_BITS,
};
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_8318,
    0x0219_834C,
    0x0219_8380,
    0x0219_84C0,
    0x0219_852C,
    0x0219_8550,
];

/// ROM `0x021DDBD4` (`fldmmdl_acmd.c` .data): the three jump-height tables (fx32 draw-offset y per phase step `phase >> 8`):
/// 0 = `0x021D4CC4` (high), 1 = `0x021D4C44` (low), 2 = `0x021D4C84` (middle). `02199970` uses table 1, `02198FB0` table 0.
pub const JUMP_HEIGHTS: [[u32; 16]; 3] = [
    [
        0x4000, 0x6000, 0x8000, 0xA000, 0xB000, 0xC000, 0xC000, 0xC000, 0xB000, 0xA000, 0x9000,
        0x8000, 0x6000, 0x4000, 0, 0,
    ],
    [
        0, 0x2000, 0x3000, 0x4000, 0x5000, 0x6000, 0x6000, 0x6000, 0x5000, 0x5000, 0x4000, 0x3000,
        0x2000, 0, 0, 0,
    ],
    [
        0x2000, 0x4000, 0x6000, 0x8000, 0x9000, 0xA000, 0xA000, 0xA000, 0x9000, 0x8000, 0x6000,
        0x5000, 0x3000, 0x2000, 0, 0,
    ],
];

/// Move bits a jump start sets: 0x80 (jumping) | 0x20 (moving).
const JUMPING: u32 = 0xA0;
/// Move bits a landed jump sets: 0x400 (done) | 0x100 | 0x40.
const LANDED: u32 = 0x540;
/// SE of the jumps started through [`jump_start_se`].
const SE_JUMP: u32 = 0x55E;
/// SE when a jump lands.
const SE_LAND: u32 = 0x67B;
/// The highest jump phase (`phase >> 8` indexes [`JUMP_HEIGHTS`]).
const PHASE_MAX: u16 = 0xF00;

/// A VecFx32 as the bytes of a stack local.
pub(crate) fn vec_bytes(x: u32, y: u32, z: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    b[..4].copy_from_slice(&x.to_le_bytes());
    b[4..8].copy_from_slice(&y.to_le_bytes());
    b[8..].copy_from_slice(&z.to_le_bytes());
    b
}

/// C: `FUN_02198318` (unlabeled routine at `0x02198318`): start walking in place: acmd work (0xC) = `u16 anim`, `u16 frames`;
/// display dir = `dir` (u16), draw animation code = `anim` (the whole word), old grid = grid, acmd seq + 1.
pub fn stay_start<E: FieldEnv>(env: &mut E, m: u32, dir: u32, frames: u32, anim: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0xC]);
    env.w16(w, anim as u16);
    env.w16(w.wrapping_add(2), frames as u16);
    env.callv(SET_DIR_DISP, &[m, dir & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_0219834C` (unlabeled routine at `0x0219834C`): walk-in-place update: frames (s16) - 1; at 0 or below: acmd done,
/// draw animation code 0, acmd seq + 1. Returns 0.
pub fn stay_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let n = (env.r16(w.wrapping_add(2)) as i16).wrapping_sub(1);
    env.w16(w.wrapping_add(2), n as u16);
    if n > 0 {
        return 0;
    }
    env.callv(SET_MOVE_BITS, &[m, ACMD_DONE]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_02198380` (unlabeled routine at `0x02198380`): acmd code 0x18 (walk in place facing up, 32 frames):
/// [`stay_start`]`(m, 0, 0x20, 1)`; returns 1.
pub fn stay_up_32<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    stay_start(env, m, 0, 0x20, 1);
    1
}

/// C: `FUN_021984C0` (unlabeled routine at `0x021984C0`): start a jump: acmd work (0x10) = `s32 speed`, `u16 step` at +8, `s8 dir`
/// +0xC, `s8 frames` +0xD, `u8 anim` +0xE, `s8 kind` +0xF (the low bytes of the arguments); speed 0 jumps in place (old grid =
/// grid), else the grid steps one tile in `dir`; move bits 0xA0, display and move dir, draw animation code (u16), acmd seq + 1;
/// `se` != 0 plays it.
#[allow(clippy::too_many_arguments)]
pub fn jump_start<E: FieldEnv>(
    env: &mut E,
    m: u32,
    dir: u32,
    speed: u32,
    frames: u32,
    anim: u32,
    kind: u32,
    step: u32,
    se: u32,
) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0x10]);
    env.w8(w.wrapping_add(0xC), dir as u8);
    env.w32(w, speed);
    env.w8(w.wrapping_add(0xD), frames as u8);
    env.w8(w.wrapping_add(0xE), anim as u8);
    env.w8(w.wrapping_add(0xF), kind as u8);
    env.w16(w.wrapping_add(8), step as u16);
    if speed == 0 {
        env.callv(SAVE_OLD_GRID, &[m]);
    } else {
        env.callv(GRID_STEP, &[m, dir & 0xFFFF]);
    }
    env.callv(SET_MOVE_BITS, &[m, JUMPING]);
    env.callv(SET_DIR, &[m, dir & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim & 0xFFFF]);
    env.callv(INC_SEQ, &[m]);
    if se != 0 {
        env.callv(PLAY_SE, &[se]);
    }
}

/// C: `FUN_0219852C` (unlabeled routine at `0x0219852C`): [`jump_start`] with the jump SE 0x55E; `anim` and `step` are passed on
/// as u16, `kind` as a sign-extended i16 (the halves the original reloads from its stack).
#[allow(clippy::too_many_arguments)]
pub fn jump_start_se<E: FieldEnv>(
    env: &mut E,
    m: u32,
    dir: u32,
    speed: u32,
    frames: u32,
    anim: u32,
    kind: u32,
    step: u32,
) {
    let kind = i32::from(kind as u16 as i16) as u32;
    jump_start(
        env,
        m,
        dir,
        speed,
        frames,
        anim & 0xFFFF,
        kind,
        step & 0xFFFF,
        SE_JUMP,
    );
}

/// C: `FUN_02198550` (unlabeled routine at `0x02198550`): jump update: a moving jump advances the position by speed in the s8 dir
/// (map height), and every whole tile of distance (0x10000) steps the grid (move bit 0x20); the phase (u16) advances by the step,
/// capped at 0xF00, and the draw offset is (0, [`JUMP_HEIGHTS`]`[kind][phase >> 8]`, 0); frames (s8) - 1; at 0 or below the jump
/// lands: draw offset 0, move bits 0x540, old grid = grid, draw animation code 0, acmd seq + 1, SE 0x67B, returns 1; else 0.
/// UNVERIFIED: a kind outside 0..=2 panics (the original reads past the 3-pointer table).
pub fn jump_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    if env.r32(w) != 0 {
        let dir = i32::from(env.r8(w.wrapping_add(0xC)) as i8) as u32;
        let speed = env.r32(w);
        env.callv(POS_MOVE, &[m, dir, speed]);
        env.callv(POS_HEIGHT, &[m]);
        if env.r32(w.wrapping_add(4)) as i32 >= 0x1_0000 {
            env.w32(w.wrapping_add(4), 0);
            env.callv(GRID_STEP, &[m, dir]);
            env.callv(SET_MOVE_BITS, &[m, MOVING]);
        }
        let s = (env.r32(w) as i32).wrapping_abs() as u32;
        let d = env.r32(w.wrapping_add(4)).wrapping_add(s);
        env.w32(w.wrapping_add(4), d);
    }
    let mut phase = env
        .r16(w.wrapping_add(0xA))
        .wrapping_add(env.r16(w.wrapping_add(8)));
    if phase > PHASE_MAX {
        phase = PHASE_MAX;
    }
    env.w16(w.wrapping_add(0xA), phase);
    let kind = env.r8(w.wrapping_add(0xF)) as i8;
    let y = JUMP_HEIGHTS[kind as usize][usize::from(phase >> 8)];
    let mut v = vec_bytes(0, y, 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    let n = (env.r8(w.wrapping_add(0xD)) as i8).wrapping_sub(1);
    env.w8(w.wrapping_add(0xD), n as u8);
    if n > 0 {
        return 0;
    }
    v = [0; 12];
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    env.callv(SET_MOVE_BITS, &[m, LANDED]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    env.callv(PLAY_SE, &[SE_LAND]);
    1
}

#[cfg(test)]
mod tests;
