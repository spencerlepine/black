//! `eventdata_system.c` (overlay 10, `0x0216237C..0x02163010`): the zone event data of ARC 125 (one member per
//! zone: background events (signs, hidden items), NPC headers, warps and coordinate triggers) and the lookups
//! by position (grid `VecFx32` or rail location), the warp exits and the trigger checks.
//!
//! F6a-spec specced all 56 functions (`tools/rom-diff/specs/black-field/eventdata.spec`, whose header documents
//! the EVENTDATA_SYSTEM (0xAA8 bytes) and the four record layouts; summary in the crate README); this module
//! holds the smoke ports. Records are read through a [`Mem`] window at their original addresses. The rest of F1
//! row 10.23 is the areas `wfbc_data`, `mmdl_move` and `eventdata_tail`.
//!
//! EVENTDATA_SYSTEM: `+0x10 u16` zone, `+0x12..+0x18 u16` bg / npc / warp / trigger counts, `+0x1C..+0x28` the
//! record arrays, `+0x120` -> the tail after the records, `+0x124` the loaded ARC 125 member. WARP (20 bytes):
//! `+0 u16` destination zone, `+2 u16` destination warp, `+4 u8` exit direction, `+6 u16` position kind
//! (0 grid), grid `+8 / +0xA / +0xC s16` x / y / z in `fx32 >> 12` units (16 per tile), `+0xE / +0x10 u16`
//! width / depth in tiles.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0216_25CC, 0x0216_2C54, 0x0216_2D2C];

/// Size of the EVENTDATA_SYSTEM record (`Heap_AllocDebug(heap, 0xAA8, ..)` in `FUN_0216237C`).
pub const SYS_SIZE: u32 = 0xAA8;
/// Size of a background-event record.
pub const BG_SIZE: u32 = 0x14;
/// Size of an NPC record (= the fldmmdl MMDL HEADER).
pub const NPC_SIZE: u32 = 0x24;
/// Size of a warp record.
pub const WARP_SIZE: u32 = 0x14;
/// Size of a trigger record.
pub const TRIGGER_SIZE: u32 = 0x16;

/// C: `FUN_021625CC` (unlabeled routine at `0x021625CC`): warp record `i` of the zone, or 0 when there are no
/// warp records (`+0x24` null) or `i` is not below the warp count (`u16 +0x16`, unsigned compare).
pub fn warp_at(mem: &Mem, sys: u32, i: u32) -> u32 {
    let warps = mem.r32(sys.wrapping_add(0x24));
    if warps == 0 || i >= mem.r16(sys.wrapping_add(0x16)) as u32 {
        return 0;
    }
    warps.wrapping_add(i.wrapping_mul(WARP_SIZE))
}

/// C: `FUN_02162C54` (unlabeled routine at `0x02162C54`): the exit direction of a warp: its byte `+4` 1 / 2 / 3 /
/// 4 is direction 0 / 1 / 2 / 3; any other value (0 included) is 1.
pub fn warp_exit_dir(mem: &Mem, warp: u32) -> u32 {
    match mem.r8(warp.wrapping_add(4)) {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 1,
    }
}

/// C: `FUN_02162D2C` (unlabeled routine at `0x02162D2C`): does the grid warp contain the position `pos`
/// (`VecFx32`)? A rail warp (`+6 == 1`) never does. With `p = pos >> 12` (arithmetic), the warp covers
/// `x <= p.x <= x + 16 (w - 1)`, `y - 2 <= p.y <= y + 2` and `z <= p.z <= z + 16 (d - 1)` (signed, the
/// sums wrap as in the original; a width of 0 makes the range empty).
pub fn warp_contains(mem: &Mem, warp: u32, pos: u32) -> bool {
    if mem.r16(warp.wrapping_add(6)) == 1 {
        return false;
    }
    let px = (mem.r32(pos) as i32) >> 12;
    let py = (mem.r32(pos.wrapping_add(4)) as i32) >> 12;
    let pz = (mem.r32(pos.wrapping_add(8)) as i32) >> 12;
    let x = mem.r16(warp.wrapping_add(8)) as i16 as i32;
    let y = mem.r16(warp.wrapping_add(0xA)) as i16 as i32;
    let z = mem.r16(warp.wrapping_add(0xC)) as i16 as i32;
    let w = mem.r16(warp.wrapping_add(0xE)) as i32;
    let d = mem.r16(warp.wrapping_add(0x10)) as i32;
    x <= px
        && x.wrapping_add((w - 1).wrapping_shl(4)) >= px
        && y - 2 <= py
        && y + 2 >= py
        && z <= pz
        && z.wrapping_add((d - 1).wrapping_shl(4)) >= pz
}

#[cfg(test)]
mod tests;
