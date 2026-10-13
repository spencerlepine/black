//! `fldmmdl_acmd.c`, part 2a (overlay 21, `0x02197F5C..0x02198318`; F1 row 21.09, F4b job 2): the end step of every grid step
//! list, face (acmd codes 0..3) and the one-tile walks (02197FC0 and its 32 init steps, update 0219800C).
//!
//! F4b-spec2 specced all 40 functions (`tools/rom-diff/specs/black-field/acmd2a.spec`; the header has the step contract and the
//! conventions shared by `acmd2a`..`acmd2e`); F4b-port1 ported them; this module also holds the callee addresses the five acmd2
//! modules share. Every callee outside the area (the fldmmdl.c / mmdl_move.c accessors, sound, the draw side) is reached through
//! [`FieldEnv::call`] by address; in-area callees that are ported are Rust calls.

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_7F5C,
    0x0219_7F6C,
    0x0219_7F90,
    0x0219_7F9C,
    0x0219_7FA8,
    0x0219_7FB4,
    0x0219_7FC0,
    0x0219_800C,
    0x0219_8060,
    0x0219_8074,
    0x0219_8088,
    0x0219_809C,
    0x0219_80B0,
    0x0219_80C4,
    0x0219_80DC,
    0x0219_80F0,
    0x0219_8104,
    0x0219_811C,
    0x0219_8134,
    0x0219_8148,
    0x0219_815C,
    0x0219_8170,
    0x0219_8184,
    0x0219_8198,
    0x0219_81AC,
    0x0219_81C4,
    0x0219_81DC,
    0x0219_81F0,
    0x0219_8208,
    0x0219_821C,
    0x0219_8234,
    0x0219_8248,
    0x0219_8260,
    0x0219_8278,
    0x0219_8290,
    0x0219_82A4,
    0x0219_82BC,
    0x0219_82D4,
    0x0219_82EC,
    0x0219_8300,
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

/// C: `FUN_02197F9C` (unlabeled routine at `0x02197F9C`): acmd code 1 (face down): [`face`]`(m, 1)`; returns 1.
pub fn face_down<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    face(env, m, 1);
    1
}

/// C: `FUN_02197FA8` (unlabeled routine at `0x02197FA8`): acmd code 2 (face left): [`face`]`(m, 2)`; returns 1.
pub fn face_left<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    face(env, m, 2);
    1
}

/// C: `FUN_02197FB4` (unlabeled routine at `0x02197FB4`): acmd code 3 (face right): [`face`]`(m, 3)`; returns 1.
pub fn face_right<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    face(env, m, 3);
    1
}

/// C: `FUN_02198060` (unlabeled routine at `0x02198060`): acmd code 0x04: walk one tile up, 32 frames of
/// 0x800 (fx32), anim 1: [`walk_start`]`(m, 0, 0x800, 32, 1)`; returns 1.
pub fn walk_04<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x800, 32, 1);
    1
}

/// C: `FUN_02198074` (unlabeled routine at `0x02198074`): acmd code 0x05: walk one tile down, 32 frames of
/// 0x800 (fx32), anim 1: [`walk_start`]`(m, 1, 0x800, 32, 1)`; returns 1.
pub fn walk_05<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x800, 32, 1);
    1
}

/// C: `FUN_02198088` (unlabeled routine at `0x02198088`): acmd code 0x06: walk one tile left, 32 frames of
/// 0x800 (fx32), anim 1: [`walk_start`]`(m, 2, 0x800, 32, 1)`; returns 1.
pub fn walk_06<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x800, 32, 1);
    1
}

/// C: `FUN_0219809C` (unlabeled routine at `0x0219809C`): acmd code 0x07: walk one tile right, 32 frames of
/// 0x800 (fx32), anim 1: [`walk_start`]`(m, 3, 0x800, 32, 1)`; returns 1.
pub fn walk_07<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x800, 32, 1);
    1
}

/// C: `FUN_021980B0` (unlabeled routine at `0x021980B0`): acmd code 0x08: walk one tile up, 16 frames of
/// 0x1000 (fx32), anim 2: [`walk_start`]`(m, 0, 0x1000, 16, 2)`; returns 1.
pub fn walk_08<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x1000, 16, 2);
    1
}

/// C: `FUN_021980C4` (unlabeled routine at `0x021980C4`): acmd code 0x09: walk one tile down, 16 frames of
/// 0x1000 (fx32), anim 2: [`walk_start`]`(m, 1, 0x1000, 16, 2)`; returns 1.
pub fn walk_09<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x1000, 16, 2);
    1
}

/// C: `FUN_021980DC` (unlabeled routine at `0x021980DC`): acmd code 0x0A: walk one tile left, 16 frames of
/// 0x1000 (fx32), anim 2: [`walk_start`]`(m, 2, 0x1000, 16, 2)`; returns 1.
pub fn walk_0a<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x1000, 16, 2);
    1
}

/// C: `FUN_021980F0` (unlabeled routine at `0x021980F0`): acmd code 0x0B: walk one tile right, 16 frames of
/// 0x1000 (fx32), anim 2: [`walk_start`]`(m, 3, 0x1000, 16, 2)`; returns 1.
pub fn walk_0b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x1000, 16, 2);
    1
}

/// C: `FUN_02198104` (unlabeled routine at `0x02198104`): acmd code 0x0C: walk one tile up, 8 frames of
/// 0x2000 (fx32), anim 3: [`walk_start`]`(m, 0, 0x2000, 8, 3)`; returns 1.
pub fn walk_0c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x2000, 8, 3);
    1
}

/// C: `FUN_0219811C` (unlabeled routine at `0x0219811C`): acmd code 0x0D: walk one tile down, 8 frames of
/// 0x2000 (fx32), anim 3: [`walk_start`]`(m, 1, 0x2000, 8, 3)`; returns 1.
pub fn walk_0d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x2000, 8, 3);
    1
}

/// C: `FUN_02198134` (unlabeled routine at `0x02198134`): acmd code 0x0E: walk one tile left, 8 frames of
/// 0x2000 (fx32), anim 3: [`walk_start`]`(m, 2, 0x2000, 8, 3)`; returns 1.
pub fn walk_0e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x2000, 8, 3);
    1
}

/// C: `FUN_02198148` (unlabeled routine at `0x02198148`): acmd code 0x0F: walk one tile right, 8 frames of
/// 0x2000 (fx32), anim 3: [`walk_start`]`(m, 3, 0x2000, 8, 3)`; returns 1.
pub fn walk_0f<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x2000, 8, 3);
    1
}

/// C: `FUN_0219815C` (unlabeled routine at `0x0219815C`): acmd code 0x10: walk one tile up, 4 frames of
/// 0x4000 (fx32), anim 4: [`walk_start`]`(m, 0, 0x4000, 4, 4)`; returns 1.
pub fn walk_10<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x4000, 4, 4);
    1
}

/// C: `FUN_02198170` (unlabeled routine at `0x02198170`): acmd code 0x11: walk one tile down, 4 frames of
/// 0x4000 (fx32), anim 4: [`walk_start`]`(m, 1, 0x4000, 4, 4)`; returns 1.
pub fn walk_11<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x4000, 4, 4);
    1
}

/// C: `FUN_02198184` (unlabeled routine at `0x02198184`): acmd code 0x12: walk one tile left, 4 frames of
/// 0x4000 (fx32), anim 4: [`walk_start`]`(m, 2, 0x4000, 4, 4)`; returns 1.
pub fn walk_12<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x4000, 4, 4);
    1
}

/// C: `FUN_02198198` (unlabeled routine at `0x02198198`): acmd code 0x13: walk one tile right, 4 frames of
/// 0x4000 (fx32), anim 4: [`walk_start`]`(m, 3, 0x4000, 4, 4)`; returns 1.
pub fn walk_13<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x4000, 4, 4);
    1
}

/// C: `FUN_021981AC` (unlabeled routine at `0x021981AC`): acmd code 0x14: walk one tile up, 2 frames of
/// 0x8000 (fx32), anim 5: [`walk_start`]`(m, 0, 0x8000, 2, 5)`; returns 1.
pub fn walk_14<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x8000, 2, 5);
    1
}

/// C: `FUN_021981C4` (unlabeled routine at `0x021981C4`): acmd code 0x15: walk one tile down, 2 frames of
/// 0x8000 (fx32), anim 5: [`walk_start`]`(m, 1, 0x8000, 2, 5)`; returns 1.
pub fn walk_15<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x8000, 2, 5);
    1
}

/// C: `FUN_021981DC` (unlabeled routine at `0x021981DC`): acmd code 0x16: walk one tile left, 2 frames of
/// 0x8000 (fx32), anim 5: [`walk_start`]`(m, 2, 0x8000, 2, 5)`; returns 1.
pub fn walk_16<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x8000, 2, 5);
    1
}

/// C: `FUN_021981F0` (unlabeled routine at `0x021981F0`): acmd code 0x17: walk one tile right, 2 frames of
/// 0x8000 (fx32), anim 5: [`walk_start`]`(m, 3, 0x8000, 2, 5)`; returns 1.
pub fn walk_17<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x8000, 2, 5);
    1
}

/// C: `FUN_02198208` (unlabeled routine at `0x02198208`): acmd code 0x54: walk one tile up, 1 frame of
/// 0x10000 (fx32), anim 0: [`walk_start`]`(m, 0, 0x10000, 1, 0)`; returns 1.
pub fn walk_54<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x10000, 1, 0);
    1
}

/// C: `FUN_0219821C` (unlabeled routine at `0x0219821C`): acmd code 0x55: walk one tile down, 1 frame of
/// 0x10000 (fx32), anim 0: [`walk_start`]`(m, 1, 0x10000, 1, 0)`; returns 1.
pub fn walk_55<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x10000, 1, 0);
    1
}

/// C: `FUN_02198234` (unlabeled routine at `0x02198234`): acmd code 0x56: walk one tile left, 1 frame of
/// 0x10000 (fx32), anim 0: [`walk_start`]`(m, 2, 0x10000, 1, 0)`; returns 1.
pub fn walk_56<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x10000, 1, 0);
    1
}

/// C: `FUN_02198248` (unlabeled routine at `0x02198248`): acmd code 0x57: walk one tile right, 1 frame of
/// 0x10000 (fx32), anim 0: [`walk_start`]`(m, 3, 0x10000, 1, 0)`; returns 1.
pub fn walk_57<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x10000, 1, 0);
    1
}

/// C: `FUN_02198260` (unlabeled routine at `0x02198260`): acmd code 0x58: walk one tile up, 4 frames of
/// 0x4000 (fx32), anim 10: [`walk_start`]`(m, 0, 0x4000, 4, 10)`; returns 1.
pub fn walk_58<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x4000, 4, 10);
    1
}

/// C: `FUN_02198278` (unlabeled routine at `0x02198278`): acmd code 0x59: walk one tile down, 4 frames of
/// 0x4000 (fx32), anim 10: [`walk_start`]`(m, 1, 0x4000, 4, 10)`; returns 1.
pub fn walk_59<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x4000, 4, 10);
    1
}

/// C: `FUN_02198290` (unlabeled routine at `0x02198290`): acmd code 0x5A: walk one tile left, 4 frames of
/// 0x4000 (fx32), anim 10: [`walk_start`]`(m, 2, 0x4000, 4, 10)`; returns 1.
pub fn walk_5a<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x4000, 4, 10);
    1
}

/// C: `FUN_021982A4` (unlabeled routine at `0x021982A4`): acmd code 0x5B: walk one tile right, 4 frames of
/// 0x4000 (fx32), anim 10: [`walk_start`]`(m, 3, 0x4000, 4, 10)`; returns 1.
pub fn walk_5b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x4000, 4, 10);
    1
}

/// C: `FUN_021982BC` (unlabeled routine at `0x021982BC`): acmd code 0x9B: walk one tile up, 16 frames of
/// 0x1000 (fx32), anim 5: [`walk_start`]`(m, 0, 0x1000, 16, 5)`; returns 1.
pub fn walk_9b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 0, 0x1000, 16, 5);
    1
}

/// C: `FUN_021982D4` (unlabeled routine at `0x021982D4`): acmd code 0x9C: walk one tile down, 16 frames of
/// 0x1000 (fx32), anim 5: [`walk_start`]`(m, 1, 0x1000, 16, 5)`; returns 1.
pub fn walk_9c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 1, 0x1000, 16, 5);
    1
}

/// C: `FUN_021982EC` (unlabeled routine at `0x021982EC`): acmd code 0x9D: walk one tile left, 16 frames of
/// 0x1000 (fx32), anim 5: [`walk_start`]`(m, 2, 0x1000, 16, 5)`; returns 1.
pub fn walk_9d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 2, 0x1000, 16, 5);
    1
}

/// C: `FUN_02198300` (unlabeled routine at `0x02198300`): acmd code 0x9E: walk one tile right, 16 frames of
/// 0x1000 (fx32), anim 5: [`walk_start`]`(m, 3, 0x1000, 16, 5)`; returns 1.
pub fn walk_9e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    walk_start(env, m, 3, 0x1000, 16, 5);
    1
}

#[cfg(test)]
mod tests;
