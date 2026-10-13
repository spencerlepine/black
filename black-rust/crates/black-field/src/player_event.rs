//! `field_player_grid_event.c` (overlay 21, `0x021C1024..0x021C1A08`): the surf start task (jump onto the water:
//! 021C1024 creates it, proc 021C10B4, work 0x18) and the surf end task (jump off onto land: 021C1228, proc 021C1988
//! over the step table ROM `0x021DA128`, or 021C16AC for a jump down; work 0x4C), plus the MMDL-in-front lookup they
//! share with the boulder push (021C19B4).
//!
//! F3a-spec specced all 17 functions (`tools/rom-diff/specs/black-field/player_event.spec`, header: task works); this
//! module holds the smoke ports.

use crate::hooks::{CArg, FieldEnv, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_1094,
    0x021C_10A0,
    0x021C_12C4,
    0x021C_12D0,
    0x021C_12E4,
];

/// `FUN_02030EE8`: the work of a task (task system, main).
const TASK_WORK: u32 = 0x0203_0EE8;
/// `FUN_02030EAC`: delete a task.
const TASK_DELETE: u32 = 0x0203_0EAC;
/// `FUN_0216D8E0`: set the draw offset VecFx32 of an MMDL (fldmmdl, F4a).
const MMDL_SET_DRAW_OFFSET: u32 = 0x0216_D8E0;

/// C: `FUN_021C1094` (unlabeled routine at `0x021C1094`): the surf start task is done (work `+4`, u16).
pub fn surf_start_done<E: FieldEnv>(env: &mut E, task: u32) -> u16 {
    let w = env.callv(TASK_WORK, &[task]);
    env.r16(w.wrapping_add(4))
}

/// C: `FUN_021C10A0` (unlabeled routine at `0x021C10A0`): free the surf start task's work and delete the task.
pub fn surf_start_delete<E: FieldEnv>(env: &mut E, task: u32) {
    let w = env.callv(TASK_WORK, &[task]);
    env.callv(HEAP_FREE, &[w]);
    env.callv(TASK_DELETE, &[task]);
}

/// C: `FUN_021C12C4` (unlabeled routine at `0x021C12C4`): the surf end task is done (work `+6`, u16).
pub fn surf_end_done<E: FieldEnv>(env: &mut E, task: u32) -> u16 {
    let w = env.callv(TASK_WORK, &[task]);
    env.r16(w.wrapping_add(6))
}

/// C: `FUN_021C12D0` (unlabeled routine at `0x021C12D0`): free the surf end task's work and delete the task.
pub fn surf_end_delete<E: FieldEnv>(env: &mut E, task: u32) {
    surf_start_delete(env, task);
}

/// C: `FUN_021C12E4` (unlabeled routine at `0x021C12E4`): the MMDL draw offset while riding the surf Pokemon: the
/// VecFx32 at `off` raised by (0, 0x1000, 0x5000) (wrapping), passed to `FUN_0216D8E0` as a local copy.
pub fn set_ride_offset<E: FieldEnv>(env: &mut E, m: u32, off: u32) {
    let mut v = [0u8; 12];
    for (i, add) in [0u32, 0x1000, 0x5000].into_iter().enumerate() {
        let w = env.r32(off.wrapping_add(4 * i as u32)).wrapping_add(add);
        v[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    env.call(MMDL_SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
}

#[cfg(test)]
mod tests;
