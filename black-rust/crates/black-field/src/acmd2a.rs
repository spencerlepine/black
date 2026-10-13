//! `fldmmdl_acmd.c`, part 2a (overlay 21, `0x02197F5C..0x02198318`; F1 row 21.09, F4b job 2): the end step of every grid step
//! list, face (acmd codes 0..3) and the one-tile walks (02197FC0 and its 32 init steps, update 0219800C).
//!
//! F4b-spec2 specced all 40 functions (`tools/rom-diff/specs/black-field/acmd2a.spec`; the header has the step contract and the
//! conventions shared by `acmd2a`..`acmd2e`); this module holds the smoke ports and the callee addresses the five acmd2 modules
//! share. Every callee outside the area (the fldmmdl.c / mmdl_move.c accessors, sound, the draw side) is reached through
//! [`FieldEnv::call`] by address; in-area callees that are ported are Rust calls.

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_7F5C,
    0x0219_7F6C,
    0x0219_7F90,
    0x0219_7FC0,
    0x0219_800C,
];

/// `FUN_0216D6BC(m, n)`: clear `n` bytes of the acmd work (`m + 0xB4`) and return it (`fldmmdl::acmd_work_init`).
pub(crate) const ACMD_WORK_INIT: u32 = 0x0216_D6BC;
/// `FUN_0216D6D4(m)`: the acmd work (`m + 0xB4`).
pub(crate) const ACMD_WORK: u32 = 0x0216_D6D4;
/// `FUN_0216D7CC(m)`: acmd seq + 1.
pub(crate) const INC_SEQ: u32 = 0x0216_D7CC;
/// `FUN_0216D674(m, v)`: draw animation code (`u16 +0x2A`) = v.
pub(crate) const SET_ANIM: u32 = 0x0216_D674;
/// `FUN_0216D568(m, v)`: move bits |= v.
pub(crate) const SET_MOVE_BITS: u32 = 0x0216_D568;
/// `FUN_0216D5E0(m, dir)`: display dir = dir unless the dir is locked.
pub(crate) const SET_DIR_DISP: u32 = 0x0216_D5E0;
/// `FUN_0216D60C(m, dir)`: display and move dir = dir.
pub(crate) const SET_DIR: u32 = 0x0216_D60C;
/// `FUN_02163F10(m)`: old grid = grid.
pub(crate) const SAVE_OLD_GRID: u32 = 0x0216_3F10;
/// `FUN_02163E7C(m, dir)`: step the grid one tile in `dir`.
pub(crate) const GRID_STEP: u32 = 0x0216_3E7C;
/// `FUN_02163FC0(m, dir, n)`: position += n in `dir`.
pub(crate) const POS_MOVE: u32 = 0x0216_3FC0;
/// `FUN_02164010(m)`: put the MMDL on the map height.
pub(crate) const POS_HEIGHT: u32 = 0x0216_4010;
/// `FUN_0216D8C0(m, &v)`: the draw offset VecFx32 (`+0x50`) = `*v`.
pub(crate) const SET_DRAW_OFFSET: u32 = 0x0216_D8C0;
/// `FUN_020061E4(id)`: play a sound effect.
pub(crate) const PLAY_SE: u32 = 0x0200_61E4;

/// Move bit 0x400: the acmd is done.
pub(crate) const ACMD_DONE: u32 = 0x400;
/// Move bits a finished walk sets: 0x400 (done) | 0x40.
pub(crate) const WALK_DONE: u32 = 0x440;
/// Move bit 0x20: moving.
pub(crate) const MOVING: u32 = 0x20;

/// C: `FUN_02197F5C` (unlabeled routine at `0x02197F5C`): the last entry of every grid step list: move bits |= 0x400 (acmd done);
/// returns 0 (stop for this frame).
pub fn end_step<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(SET_MOVE_BITS, &[m, ACMD_DONE]);
    0
}

/// C: `FUN_02197F6C` (unlabeled routine at `0x02197F6C`): face `dir`: display dir = `dir` (as a u16), draw animation code 0, old
/// grid = grid, acmd seq + 1.
pub fn face<E: FieldEnv>(env: &mut E, m: u32, dir: u32) {
    env.callv(SET_DIR_DISP, &[m, dir & 0xFFFF]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02197F90` (unlabeled routine at `0x02197F90`): acmd code 0 (face up): [`face`]`(m, 0)`; returns 1.
pub fn face_up<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    face(env, m, 0);
    1
}

/// C: `FUN_02197FC0` (unlabeled routine at `0x02197FC0`): start a one-tile walk: acmd work (0xC bytes) = `u16 anim`, `u16 frames`,
/// `u32 dir`, `u32 speed` (fx32 per frame); step the grid one tile in `dir`, display and move dir = `dir`, draw animation code =
/// `anim` (both as u16), move bit 0x20, acmd seq + 1.
pub fn walk_start<E: FieldEnv>(env: &mut E, m: u32, dir: u32, speed: u32, frames: u32, anim: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0xC]);
    env.w16(w, anim as u16);
    env.w16(w.wrapping_add(2), frames as u16);
    env.w32(w.wrapping_add(4), dir);
    env.w32(w.wrapping_add(8), speed);
    env.callv(GRID_STEP, &[m, dir & 0xFFFF]);
    env.callv(SET_DIR, &[m, dir & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim & 0xFFFF]);
    env.callv(SET_MOVE_BITS, &[m, MOVING]);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_0219800C` (unlabeled routine at `0x0219800C`): walk update: position += speed in dir, map height; frames (s16) - 1;
/// at 0 or below: move bits 0x440, old grid = grid, draw animation code 0, acmd seq + 1. Returns 0 either way.
pub fn walk_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let dir = env.r32(w.wrapping_add(4)) & 0xFFFF;
    let speed = env.r32(w.wrapping_add(8));
    env.callv(POS_MOVE, &[m, dir, speed]);
    env.callv(POS_HEIGHT, &[m]);
    let n = (env.r16(w.wrapping_add(2)) as i16).wrapping_sub(1);
    env.w16(w.wrapping_add(2), n as u16);
    if n > 0 {
        return 0;
    }
    env.callv(SET_MOVE_BITS, &[m, WALK_DONE]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    0
}

#[cfg(test)]
mod tests;
