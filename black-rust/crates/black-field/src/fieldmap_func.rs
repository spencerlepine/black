//! `fieldmap_func.c` (overlay 21, `0x021BB890..0x021BBCB8`): the FIELDMAP FUNC system (slots of overlay-loaded
//! field functions run by two task systems) and the 30-entry pass-by ring after it.
//!
//! F2a-spec specced all 13 functions (`tools/rom-diff/specs/black-field/fieldmap_func.spec`, whose header
//! documents the system, slot, function-data and ring layouts); this module holds the smoke port.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021B_BBD8];

/// Size of one ring entry.
pub const RING_ENTRY: usize = 0x24;
/// Entries in the ring.
pub const RING_LEN: u8 = 30;
/// Ring: `u8` index of the oldest entry.
pub const RING_OLDEST: usize = 0x438;
/// Ring: `u8` index of the next entry to write.
pub const RING_NEXT: usize = 0x439;
/// Ring: `s32` total number of entries ever written.
pub const RING_COUNT: usize = 0x43C;

/// C: `FUN_021BBBD8` (unlabeled routine at `0x021BBBD8`): take the next ring slot: when the ring has entries
/// (`count > 0`) and `next == oldest`, `oldest` moves on (wrapping at 30); `next` moves on the same way; returns
/// the byte offset of the old `next` slot (`slot * 0x24`; the original returns `ring + offset`).
/// The indices are not range-checked (the original computes the slot from whatever byte is there).
pub fn ring_take(ring: &mut [u8]) -> usize {
    let count = i32::from_le_bytes(ring[RING_COUNT..RING_COUNT + 4].try_into().unwrap());
    if count > 0 && ring[RING_NEXT] == ring[RING_OLDEST] {
        ring[RING_OLDEST] = ring[RING_OLDEST].wrapping_add(1);
        if ring[RING_OLDEST] >= RING_LEN {
            ring[RING_OLDEST] = 0;
        }
    }
    let slot = ring[RING_NEXT];
    ring[RING_NEXT] = slot.wrapping_add(1);
    if ring[RING_NEXT] >= RING_LEN {
        ring[RING_NEXT] = 0;
    }
    slot as usize * RING_ENTRY
}

#[cfg(test)]
mod tests;
