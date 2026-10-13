//! `fldeff_namipoke.c` (overlay 21, `0x021ACA38..0x021AD758`): the surf Pokemon field effect: the effect task the
//! player core adds when surfing starts (021ACB38), its state (mode +9: 0 off, 1 bobbing, 2 ridden; +0xA hide), the
//! position logic of its update procs (bobbing offset applied to the player MMDL, the follower that trails the player),
//! and the splash effect of the surf start / end jumps (021AD564 / 021AD598).
//!
//! F3a-spec specced the 18 logic / mixed functions (`tools/rom-diff/specs/black-field/namipoke.spec`, header: task
//! work layouts); the 3D-model resource loads / frees and the draw procs (021ACA70, 021ACB0C, 021ACCB0, 021ACFD8,
//! 021AD274, 021AD410, 021AD530, 021AD6A4, 021AD738) are render-only (D3: U2 rewrites them) and are reached only as
//! addresses. This module holds the smoke ports.

use crate::hooks::{FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021A_CBB8, 0x021A_CBE4, 0x021A_CC04, 0x021A_CCAC];

/// `FUN_021AC1E0`: the work of an effect task (field effect system, U2).
const TASK_WORK: u32 = 0x021A_C1E0;

/// C: `FUN_021ACBB8` (unlabeled routine at `0x021ACBB8`): set the surf effect mode (`work +9`, u8) of a task;
/// nothing for a null task.
pub fn set_mode<E: FieldEnv>(env: &mut E, task: u32, mode: u32) {
    if task != 0 {
        let w = env.callv(TASK_WORK, &[task]);
        env.mem().w8(w.wrapping_add(9), mode as u8);
    }
}

/// C: `FUN_021ACBE4` (unlabeled routine at `0x021ACBE4`): the animation speed of a follower distance:
/// `min(d, 0x40000) / 0xA3D * 0x28` (signed, the division truncates toward zero).
pub fn follow_speed(d: i32) -> i32 {
    d.min(0x4_0000).wrapping_div(0xA3D).wrapping_mul(0x28)
}

/// C: `FUN_021ACC04` (unlabeled routine at `0x021ACC04`): how far VecFx32 `b` is ahead of `a` along direction
/// `dir` (0 up: `b.z - a.z`, 1 down: `a.z - b.z`, 2 left: `b.x - a.x`, 3 right: `a.x - b.x`), 0 for other dirs.
pub fn ahead_along(mem: &Mem, a: u32, b: u32, dir: u32) -> u32 {
    let r = |p: u32| mem.r32(p);
    match dir {
        0 => r(b.wrapping_add(8)).wrapping_sub(r(a.wrapping_add(8))),
        1 => r(a.wrapping_add(8)).wrapping_sub(r(b.wrapping_add(8))),
        2 => r(b).wrapping_sub(r(a)),
        3 => r(a).wrapping_sub(r(b)),
        _ => 0,
    }
}

/// C: `FUN_021ACCAC` (unlabeled routine at `0x021ACCAC`): the empty draw proc of the grid surf effect.
pub fn draw_none() {}

#[cfg(test)]
mod tests;
