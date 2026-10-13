//! `map_matrix.c` + `map_replace.c` (overlay 10, `0x02159B48..0x0215A07A`): the map matrix (the grid of
//! map blocks of a zone group, ARC 9) and the matrix replacements of ARC 10.
//!
//! F2a-spec specced all 33 functions (`tools/rom-diff/specs/black-field/map_matrix.spec`, whose header
//! documents the MAP MATRIX (0x152C bytes) and MAP REPLACE (0x2C bytes) layouts); this module holds the
//! smoke ports. Records are read through a [`Mem`] window at their original addresses.
//!
//! MAP MATRIX: `+0x08 u16` width, `+0x0A u16` height (blocks), `+0x10 u32` cell count, `+0x14 u16 zone[900]`,
//! `+0x71C u32 land[900]` (cell = `x + y * width`). See the crate README for the full table.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0215_9BC4, 0x0215_9D68];

/// Offset of the zone table (`u16` per cell).
pub const ZONE_OFS: u32 = 0x14;
/// Offset of the land (map container) table (`u32` per cell).
pub const LAND_OFS: u32 = 0x71C;
/// Size of the MAP MATRIX record.
pub const MATRIX_SIZE: u32 = 0x152C;
/// Most cells a matrix holds (30 x 30).
pub const MAX_CELLS: u32 = 900;

/// C: `FUN_02159D68` (unlabeled routine at `0x02159D68`): fx32 world coordinate -> 32-tile block
/// index: `((v / 4096) / 16) / 32`, each a signed division rounding toward zero (the `asrs/lsrs/adds`
/// sign fix of the original, so `-1` gives block 0).
pub fn block_of(v: i32) -> i32 {
    ((v / 4096) / 16) / 32
}

/// C: `FUN_02159BC4` (unlabeled routine at `0x02159BC4`): zone id of block `(x, y)`: `u16` at
/// `m + 0x14 + 2 * (x + y * width)`. No bounds check (the arithmetic wraps like the original's).
pub fn zone_at(mem: &Mem, m: u32, x: u32, y: u32) -> u16 {
    let w = mem.r16(m + 8) as u32;
    let cell = x.wrapping_add(y.wrapping_mul(w));
    mem.r16(m.wrapping_add(ZONE_OFS).wrapping_add(cell.wrapping_mul(2)))
}

#[cfg(test)]
mod tests;
