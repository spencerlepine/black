//! `perapvoice.c` (S5b): the Chatot (Pera) voice save block 36 (0x3EC bytes), main ARM9
//! 0x02007B00..0x02007CB0. Verified per function with `tools/check-fn.sh black-storage perapvoice`
//! (spec `tools/rom-diff/specs/black-storage/perapvoice.spec`).
//!
//! Layout: +0x000 `u32` "a recording exists" (1), +0x004 1000 bytes of 4-bit samples, two per
//! byte (low nibble first). [`unpack`] widens them to 8000 signed 8-bit samples (each repeated four
//! times) plus 180 zero bytes; [`Perap::pack`] narrows 8000 samples back.
//!
//! Not ported: `FUN_02007B20` (allocate a block on a heap and init it: heap plumbing),
//! `FUN_02007B44` (`FUN_020071CC(sav, 36)`, ownership). The microphone recording itself is
//! hardware (not here).

use serde::{Deserialize, Serialize};

use crate::le::{r32, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_7B00,
    0x0200_7B08,
    0x0200_7B50,
    0x0200_7B54,
    0x0200_7B5C,
    0x0200_7B64,
    0x0200_7B68,
    0x0200_7BD0,
    0x0200_7C20,
    0x0200_7C34,
    0x0200_7C70,
];

/// Save block number.
pub const BLOCK_ID: u32 = 36;
/// Bytes of the block.
pub const SIZE: usize = 0x3EC;
/// Bytes of packed samples.
pub const DATA_SIZE: usize = 1000;
/// Samples [`Perap::pack`] reads (8 per packed byte, every fourth used).
pub const SAMPLES: usize = 8000;
/// Bytes [`unpack`] writes.
pub const UNPACKED_SIZE: usize = 0x1FF4;
/// Chatot's species number.
pub const CHATOT: u32 = 441;

/// The party queries of [`Perap::check_party`].
pub trait PerapHooks {
    /// C: `FUN_0201AA18(party)`: the number of Pokemon in the party.
    fn party_count(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: the Pokemon at position `i`.
    fn party_mon(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_02017E1C(mon, 5, 0)`: its species.
    fn species(&mut self, mon: u32) -> u32;
}

/// Save block 36.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Perap {
    /// +0x000: 1 when a recording exists.
    pub exists: u32,
    /// +0x004: the packed samples (1000 bytes).
    #[serde(with = "crate::save::hex")]
    pub data: Vec<u8>,
}

impl Default for Perap {
    fn default() -> Self {
        Perap {
            exists: 0,
            data: vec![0; DATA_SIZE],
        }
    }
}

/// C: `FUN_02007b68(dst, src)`: widen 1000 packed bytes to 8000 samples: each nibble `n` gives
/// the byte `((n + 8) & 0xF) << 4` four times (low nibble first); bytes 8000..0x1FF4 are zeroed.
pub fn unpack(src: &[u8], dst: &mut [u8]) {
    for (i, &s) in src[..DATA_SIZE].iter().enumerate() {
        let lo = (((s & 0xF) + 8) & 0xF) << 4;
        let hi = (((s >> 4) + 8) & 0xF) << 4;
        dst[i * 8..i * 8 + 4].fill(lo);
        dst[i * 8 + 4..i * 8 + 8].fill(hi);
    }
    dst[SAMPLES..UNPACKED_SIZE].fill(0);
}

/// Signed sample / 16 rounded towards zero, plus 8.
fn quantize(s: u8) -> u8 {
    let s = s as i8 as i32;
    (((s + ((s >> 3) as u32 >> 28) as i32) >> 4) + 8) as u8
}

impl Perap {
    /// Reads the original 0x3EC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        Perap {
            exists: r32(b, 0),
            data: b[4..SIZE].to_vec(),
        }
    }

    /// The original 0x3EC-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        w32(&mut b, 0, self.exists);
        b[4..].copy_from_slice(&self.data);
        b
    }

    /// C: `sub_02007B00`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `FUN_02007b08`: zero the block.
    pub fn init(&mut self) {
        *self = Perap::default();
    }

    /// C: `FUN_02007b50`: the "exists" word.
    pub fn exists(&self) -> u32 {
        self.exists
    }

    /// C: unlabeled routine at `0x02007B54`: no recording.
    pub fn clear(&mut self) {
        self.exists = 0;
    }

    /// C: unlabeled routine at `0x02007B5C`: a recording exists.
    pub fn set_exists(&mut self) {
        self.exists = 1;
    }

    /// C: `FUN_02007b64`: the packed samples (the original returns block + 4).
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// C: `FUN_02007bd0(block, src)`: mark the recording present and pack 8000 signed samples:
    /// byte `i` = `q(src[8i]) | (q(src[8i + 4]) & 0xF) << 4`, `q(s) = s / 16 (towards zero) + 8`.
    pub fn pack(&mut self, src: &[u8]) {
        self.exists = 1;
        for i in 0..DATA_SIZE {
            let lo = quantize(src[8 * i]);
            let hi = quantize(src[8 * i + 4]);
            self.data[i] = lo | ((hi & 0xF) << 4);
        }
    }

    /// C: unlabeled routine at `0x02007C20` (`(dst, src)`): copy a whole block.
    pub fn copy_from(&mut self, src: &Perap) {
        *self = src.clone();
    }

    /// C: `FUN_02007C34(block, party)`: no Chatot in the party drops the recording.
    pub fn check_party<H: PerapHooks>(&mut self, party: u32, h: &mut H) {
        let n = h.party_count(party) as i32;
        for i in 0..n.max(0) as u32 {
            let mon = h.party_mon(party, i);
            if h.species(mon) == CHATOT {
                return;
            }
        }
        self.clear();
    }

    /// C: `FUN_02007C70(block)`: the cry variant from three sample bytes (`data[0x64] ^ data[0x1F4]
    /// ^ data[0x2BC]`): 1 below 100, 2 from 150, else 0; 0 without a recording.
    pub fn cry_kind(&self) -> u32 {
        if self.exists != 1 {
            return 0;
        }
        let x = self.data[0x64] ^ self.data[0x1F4] ^ self.data[0x2BC];
        if x < 100 {
            1
        } else if x >= 150 {
            2
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests;
