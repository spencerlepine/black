//! `fieldmap.c`, first part (overlay 21, `0x02187F00..0x02189BD0`): the FIELDMAP proc (create, the
//! setup sequence, the per-frame update and the teardown), its accessors, the zone-change check and the
//! map-setup record. The event checks of the same file are [`crate::fieldmap_event`].
//!
//! F2a-spec specced the 101 logic functions of the range (`tools/rom-diff/specs/black-field/fieldmap.spec`;
//! its header documents the 0x154-byte FIELDMAP record, the MAP CTRL and MAP SETUP tables and the proc table);
//! the render-only functions (G8) are listed in the crate README. This module holds the smoke ports; the
//! draw side stays behind a trait when F2a ports the rest (D3).

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0218_9B04];

/// Size of the FIELDMAP record (`Heap_AllocDebug(heap, 0x154, 1, "fieldmap.c", 0x209)` in `0x02188030`).
pub const FIELDMAP_SIZE: u32 = 0x154;

/// C: `FUN_02189B04` (unlabeled routine at `0x02189B04`): zone `z` is one of `0x3F`, `0x8C`, `0x8D`,
/// `0x8E` (the start step `0x021885E8` returns early for these zones while `0x02005C64()` is set;
/// meaning of the zone group UNVERIFIED).
pub fn is_hold_zone(z: u32) -> bool {
    matches!(z, 0x3F | 0x8C | 0x8D | 0x8E)
}

#[cfg(test)]
mod tests;
