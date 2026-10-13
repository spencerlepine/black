//! `fldmmdl_acmd.c`, part 3d (overlay 21, `0x0219BDC4..0x0219C4AC`; F4b job 3): the effect movement types (move codes 0x33..0x36, with
//! the effect-handle accessors the acmd code-0x65 steps use), mirroring the player's facing (0x37..0x3E) and walking along a wall
//! (0x3F..0x42).
//!
//! F4b-spec3 specced all 45 functions (`tools/rom-diff/specs/black-field/mvproc3d.spec`); this module holds the smoke ports.

use crate::hooks::FieldEnv;
use crate::mvproc3a::{MOVE_WORK, MOVE_WORK_INIT, SET_DIR_DISP};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_BE60,
    0x0219_BF3C,
    0x0219_BF48,
    0x0219_BF54,
    0x0219_BF60,
    0x0219_C0D4,
];

/// `FUN_021AC154(handle)`: delete a field-effect task.
const EFFECT_DELETE: u32 = 0x021A_C154;

/// C: `FUN_0219BE60` (unlabeled routine at `0x0219BE60`): move delete proc of codes 0x33..0x36: delete the effect task
/// ([`effect_handle`]) if there is one.
pub fn effect_delete<E: FieldEnv>(env: &mut E, m: u32) {
    let h = effect_handle(env, m);
    if h != 0 {
        env.callv(EFFECT_DELETE, &[h]);
    }
}

/// C: `FUN_0219BF3C` (unlabeled routine at `0x0219BF3C`): `u32` move work `+4` = `h` (the effect handle).
pub fn set_effect_handle<E: FieldEnv>(env: &mut E, m: u32, h: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    env.w32(w.wrapping_add(4), h);
}

/// C: `FUN_0219BF48` (unlabeled routine at `0x0219BF48`): the effect handle (`u32` move work `+4`; 0 = none).
pub fn effect_handle<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(MOVE_WORK, &[m]);
    env.r32(w.wrapping_add(4))
}

/// C: `FUN_0219BF54` (unlabeled routine at `0x0219BF54`): `u8` move work `+2` = 1 (the effect is finished; no new one).
pub fn effect_done<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    env.w8(w.wrapping_add(2), 1);
}

/// C: `FUN_0219BF60` (unlabeled routine at `0x0219BF60`): init of codes 0x37..0x3E: move work (4 bytes cleared) `u8 +1` = 9 (no
/// copied dir yet), `u8 +2` = `k`; display dir = 0. The `_dir` argument (r1) is overwritten before the original reads it.
pub fn mirror_init<E: FieldEnv>(env: &mut E, m: u32, _dir: u32, k: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 4]);
    env.w8(w.wrapping_add(1), 9);
    env.w8(w.wrapping_add(2), k as u8);
    env.callv(SET_DIR_DISP, &[m, 0]);
}

/// C: `FUN_0219C0D4` (unlabeled routine at `0x0219C0D4`): init of codes 0x3F..0x42: move work (0xC bytes cleared) `u32 +4` = `mode`,
/// `u32 +8` = `hand` (0 left, 1 right wall).
pub fn wall_init<E: FieldEnv>(env: &mut E, m: u32, mode: u32, hand: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 0xC]);
    env.w32(w.wrapping_add(4), mode);
    env.w32(w.wrapping_add(8), hand);
}

#[cfg(test)]
mod tests;
