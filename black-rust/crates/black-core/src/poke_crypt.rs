//! `poke_tool.c` crypt helpers (P1a): the LCG32 step that drives the Pokemon block cipher, and
//! the u16-sum checksum over a block. Main ARM9 code, verified per function with
//! `tools/check-fn.sh black-core poke_crypt` (spec `tools/rom-diff/specs/black-core/poke_crypt.spec`).

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0201_9A7C, 0x0201_9A9C];

/// C: `FUN_02019A7C`: one step of the 32-bit LCG `x = x * 0x41C64E6D + 0x6073` on `*state`;
/// returns the new state `>> 16`.
pub fn lcg32_step(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(0x41C6_4E6D).wrapping_add(0x6073);
    *state >> 16
}

/// C: `FUN_02019A9C`: 16-bit sum of the first `size / 2` little-endian halfwords of `data`
/// (a trailing odd byte is ignored).
pub fn checksum(data: &[u8], size: u32) -> u16 {
    let mut sum = 0u16;
    for i in 0..(size >> 1) as usize {
        sum = sum.wrapping_add(u16::from_le_bytes([data[2 * i], data[2 * i + 1]]));
    }
    sum
}

#[cfg(test)]
mod tests;
