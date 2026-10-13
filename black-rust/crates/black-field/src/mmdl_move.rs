//! MMDL movement helpers in F1 row 10.23 (overlay 10, `0x02163010..0x021643AC`): the grid move procs, height and
//! attribute tracking, the attribute-driven field effects (ATTR INFO, 0x1C bytes), the hit checks of a step
//! (move limit, blocking attributes, height, other MMDLs, map bounds), direction tables and the rail-header glue.
//!
//! F6a-spec specced all 70 functions (`tools/rom-diff/specs/black-field/mmdl_move.spec`, whose header documents
//! ATTR INFO and the direction tables); this module holds the smoke ports. MMDL fields are read only through the
//! fldmmdl getters (`crate::fldmmdl`, F4a). Directions: 0 up, 1 down, 2 left, 3 right.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0216_42B4];

/// C: `FUN_021642B4` (unlabeled routine at `0x021642B4`): the direction from `(x0, z0)` to `(x1, z1)`: x first
/// (`x0 > x1`: 2 left, `x0 < x1`: 3 right), then z (`z0 > z1`: 0 up, else 1 down). Signed compares of the
/// whole registers.
pub fn dir_between(x0: i32, z0: i32, x1: i32, z1: i32) -> u16 {
    if x0 > x1 {
        2
    } else if x0 < x1 {
        3
    } else if z0 > z1 {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests;
