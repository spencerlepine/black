//! `btl_client.c` part 3 (0x021CF5B0-0x021D10B8): selection checks and party/target helpers (B13-spec1).
//!
//! Specs: `tools/rom-diff/svflow_specs/client3.spec` (`nowork`, the client object is `arg buf 0x1C0`). Smoke ports only.

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021C_FBBC];

const OFS_SLOTS: usize = 0x10C;
const SLOT_COUNT: usize = 6;

/// C: `FUN_overlay_d_93__021cfbbc`: is `id` one of the six signed bytes at `+0x10C` (0xFF = empty slot, i.e. -1).
pub fn slot_holds(c: &[u8], id: i32) -> bool {
    (0..SLOT_COUNT).any(|i| c[OFS_SLOTS + i] as i8 as i32 == id)
}

#[cfg(test)]
mod tests;
