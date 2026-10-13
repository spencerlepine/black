//! `event_trainer_eye.c` (overlay 21, `0x021AE0CC..0x021AE948`): trainer line-of-sight spotting and the approach
//! task. The field event check (`FUN_021AE0CC`, called from `fieldmap_event`) walks every MMDL in use, keeps the
//! trainers (event type -> `fldmmdl::kind_434` = 1, or 2 for a trainer that looks in all four directions) that are not
//! defeated, tests the straight line from the trainer to the player within the sight range (header param 0) and checks
//! that every tile in between is free; the hits become EYE HIT records for the trainer script (0xE20). The approach
//! itself (turn, "!" balloon, walk up, the player turns round) is an 18-state TASK driven by a GMEVENT.
//!
//! F4c-spec specced all 38 functions (`tools/rom-diff/specs/black-field/trainer_eye.spec`, whose header documents the
//! sight algorithm, the EYE HIT record and the TASK); this module holds the smoke ports: the four line tests, their
//! dispatcher and one task state. MMDL fields are read only through the fldmmdl getters (F4a ports, `force` stubs in
//! the spec), so the F4c port reaches them through [`crate::hooks::FieldEnv::call`]; the "!" balloon and the walk /
//! turn animations stay behind the same seam (D3). Directions: 0 up, 1 down, 2 left, 3 right.
//!
//! Grid positions are `s16[3]` = x, y (height), z (`FUN_0216D828`, MMDL `+0x3C..+0x40`).

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_E348,
    0x021A_E360,
    0x021A_E394,
    0x021A_E3C8,
    0x021A_E3FC,
    0x021A_E5D4,
];

/// TASK `+0x00`: the state (index of ROM `0x021D9348`, 0..=17).
pub const TASK_STATE: u32 = 0x00;
/// TASK `+0x28`: the frame counter of the wait states 9 and 13.
pub const TASK_COUNTER: u32 = 0x28;

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

#[cfg(test)]
mod tests;
