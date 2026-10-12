//! The 9-bit compressed message strings (F9c). A string whose first unit is `0xF100` is a
//! LSB-first stream of 9-bit codes in the units after the header; `0x1FF` ends it.
//!
//! The decode and the insertion table are those of `wordset::append_word` (`FUN_0201F4A0`, table
//! at `0x0209E864`), which is verified against the ROM by `tools/check-fn.sh black-text wordset`;
//! [`decode`] is the same algorithm on a plain `&[u16]` and the tests check that the two agree.
//! Codes below `0x100` are the character itself, `0x100..0x132` map through the 50-entry table,
//! anything higher becomes `'?'`.

use crate::wordset::{CODE_END, CODE_TABLE, COMPRESSED};

/// The character a 9-bit code stands for once it reaches the engine.
pub fn map_code(code: u16) -> u16 {
    if code >= 0x132 {
        0x3F
    } else if code >= 0x100 {
        CODE_TABLE[code as usize - 0x100]
    } else {
        code
    }
}

/// Unpacks a compressed string (with or without its `0xF100` header) into characters. Stops at
/// the end code; a stream that ends without one yields what was decoded so far (the original
/// would run on into whatever follows the string).
pub fn decode(units: &[u16]) -> Vec<u16> {
    let body = match units.first() {
        Some(&COMPRESSED) => &units[1..],
        _ => units,
    };
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for &u in body {
        acc |= (u as u32) << bits;
        bits += 16;
        while bits >= 9 {
            let code = (acc & 0x1FF) as u16;
            acc >>= 9;
            bits -= 9;
            if code == CODE_END {
                return out;
            }
            out.push(map_code(code));
        }
    }
    out
}

#[cfg(test)]
mod tests;
