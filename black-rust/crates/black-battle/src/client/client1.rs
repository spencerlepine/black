//! `btl_client.c` part 1 (0x021CD6EC-0x021CDDDC): client object lifecycle and accessors (B13-spec1).
//!
//! Specs: `tools/rom-diff/svflow_specs/client1.spec` (`nowork`, the client object is `arg buf 0x1C0`). Smoke ports only.

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021C_D9F4, 0x021C_DD34];

const OFS_COUNTER: usize = 0x1AB;
const OFS_RESULT_PTR: usize = 0xD8;
const OFS_RESULT_SIZE: usize = 0xDC;
const OFS_RESULT_WORD: usize = 0xE0;

/// C: `FUN_overlay_d_93__021cd9f4`: counter byte at `+0x1AB` += `n` (stored as a byte), clamped to 14.
pub fn counter_add(c: &mut [u8], n: u32) {
    let v = c[OFS_COUNTER].wrapping_add(n as u8);
    c[OFS_COUNTER] = if v > 0xE { 0xE } else { v };
}

/// C: `FUN_overlay_d_93__021cdd34`: result block = one zeroed word at `+0xE0` (`c_addr` is the address of the object):
/// result pointer `+0xD8` = `&c[0xE0]`, result size `+0xDC` = 4.
pub fn result_clear(c: &mut [u8], c_addr: u32) {
    super::wr32(c, OFS_RESULT_WORD, 0);
    super::wr32(
        c,
        OFS_RESULT_PTR,
        c_addr.wrapping_add(OFS_RESULT_WORD as u32),
    );
    super::wr32(c, OFS_RESULT_SIZE, 4);
}

#[cfg(test)]
mod tests;
