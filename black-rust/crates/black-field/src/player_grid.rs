//! `field_player_grid.c` (overlay 21, `0x021A4140..0x021A5880`): PLAYER GRID, the 0x20-byte grid half of the player:
//! each frame the pad (keys, direction) becomes a move value by form (walk 021A44C8 / cycle 021A48C8 / surf
//! 021A4B00, with the ice / slide variant 021A46F0), the move value becomes an MMDL acmd (021A4CA8: walk, run, turn,
//! bump, jump, slide, fall) plus the footstep SE (021A428C, tables 0x021D8A00.. / 0x021D89F4..), and the forced-move
//! tiles take over (021A55A8, table 0x021D89EC).
//!
//! F3a-spec specced all 97 functions (`tools/rom-diff/specs/black-field/player_grid.spec`, header: layout, move
//! values, acmd codes); this module holds the smoke ports. GRID: `+0` move state (0 stop, 1 walk, 2 turn, 3 bump,
//! 4 .., 5 slide, 6 .., 7 fall), `+4` move value, `+8` flags (1 forced move, 2 .., 4 .., 8 ice spin, 0x10 sliding,
//! 0x20 running), `+0xC` -> CORE, `+0x10` FIELDMAP, `+0x14 u16` / `+0x16 u8` ice-spin counter, `+0x18 u16` footstep
//! SE (0x546 = none), `+0x1A u16` SE frame counter, `+0x1C` a one-shot flag.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_5304,
    0x021A_530C,
    0x021A_5318,
    0x021A_5768,
    0x021A_5778,
];

/// C: `FUN_021A5304` (unlabeled routine at `0x021A5304`): `+8 |= bits`.
pub fn set_flags(mem: &mut Mem, grid: u32, bits: u32) {
    let a = grid.wrapping_add(8);
    mem.w32(a, mem.r32(a) | bits);
}

/// C: `FUN_021A530C` (unlabeled routine at `0x021A530C`): `+8 &= !bits`.
pub fn clear_flags(mem: &mut Mem, grid: u32, bits: u32) {
    let a = grid.wrapping_add(8);
    mem.w32(a, mem.r32(a) & !bits);
}

/// C: `FUN_021A5318` (unlabeled routine at `0x021A5318`): any of `bits` set in `+8`.
pub fn has_flags(mem: &Mem, grid: u32, bits: u32) -> bool {
    mem.r32(grid.wrapping_add(8)) & bits != 0
}

/// C: `FUN_021A5768` (unlabeled routine at `0x021A5768`): `+0x1C` = 1 when `on == 1`, else 0.
pub fn set_1c(mem: &mut Mem, grid: u32, on: u32) {
    mem.w32(grid.wrapping_add(0x1C), (on == 1) as u32);
}

/// C: `FUN_021A5778` (unlabeled routine at `0x021A5778`): the move state `+0` is 3.
pub fn state_is_3(mem: &Mem, grid: u32) -> bool {
    mem.r32(grid) == 3
}

#[cfg(test)]
mod tests;
