//! The easy-chat (PMS) save blocks (S5b): block 39 (0xA8 bytes, main ARM9 0x02024910..0x020249B0,
//! `pms_word.c`: unlocked word groups and twenty registered sentences) and block 62 (0x28 bytes,
//! main ARM9 0x0200E6B0..0x0200E730: five sentences the player set, read by overlay 10
//! `trcard_sys.c` / `event_research_radar.c`). Verified per function with
//! `tools/check-fn.sh black-storage pms_save` (spec `tools/rom-diff/specs/black-storage/pms_save.spec`).
//!
//! Block 39: +0x00 `u32` unlock bits, +0x04 `u32`, +0x08 twenty 8-byte sentences ([`Pms`]).
//! Block 62: five sentences; the init writes the defaults below (the `pms_word.c` setters
//! `FUN_020242A4` / `FUN_02024344` / `FUN_020242D8` are inlined as field writes, as in `mail`).
//!
//! Not ported: `FUN_02024988` / `FUN_0200E6F0` (`FUN_020071CC(sav, 39 / 62)`) and `FUN_0200E6E8`
//! (the same through the game data): ownership. `FUN_020249B0` (are two words in one synonym
//! group) is `pms_word.c` word logic, not save data.

use serde::{Deserialize, Serialize};

use crate::le::{r32, w32};
use crate::mail::Pms;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_E6B0,
    0x0200_E6B4,
    0x0200_E6FC,
    0x0200_E714,
    0x0202_4910,
    0x0202_4918,
    0x0202_4934,
    0x0202_4938,
    0x0202_4994,
    0x0202_49A0,
];

/// Save block number of [`PmsSave`].
pub const BLOCK_ID: u32 = 39;
/// Bytes of [`PmsSave`].
pub const SIZE: usize = 0xA8;
/// Sentences in [`PmsSave`].
pub const SENTENCES: usize = 20;
/// Save block number of [`PmsMessages`].
pub const MESSAGES_BLOCK_ID: u32 = 62;
/// Bytes of [`PmsMessages`].
pub const MESSAGES_SIZE: usize = 0x28;
/// Sentences in [`PmsMessages`].
pub const MESSAGES: usize = 5;

/// `(group kind, unlock bit)` pairs (rodata 0x0209F784); the init unlocks the bit of kind 2.
pub const INIT_UNLOCKS: [(u8, u8); 7] = [(1, 0), (2, 1), (3, 2), (4, 3), (5, 4), (7, 5), (8, 6)];

/// Default `(kind, id, word 0)` of the five [`PmsMessages`] (`FUN_02024344` for sentence 0,
/// `FUN_020242D8(i)` for the others; word 1 = 0xFFFF).
pub const MESSAGE_DEFAULTS: [(u16, u16, u16); 5] = [
    (0, 1, 0x58F),
    (1, 0, 0x5D2),
    (2, 0, 0x5CD),
    (3, 0, 0x667),
    (2, 13, 0x573),
];

/// Save block 39.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmsSave {
    /// +0x00: unlocked word groups (bit per group).
    pub unlocked: u32,
    /// +0x04.
    pub d04: u32,
    /// +0x08: the sentences.
    pub sentences: Vec<Pms>,
}

impl Default for PmsSave {
    fn default() -> Self {
        PmsSave::from_bytes(&[0; SIZE])
    }
}

impl PmsSave {
    /// Reads the original 0xA8-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        PmsSave {
            unlocked: r32(b, 0),
            d04: r32(b, 4),
            sentences: (0..SENTENCES)
                .map(|i| Pms::from_bytes(&b[8 + 8 * i..16 + 8 * i]))
                .collect(),
        }
    }

    /// The original 0xA8-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        w32(&mut b, 0, self.unlocked);
        w32(&mut b, 4, self.d04);
        for (i, p) in self.sentences.iter().enumerate() {
            b[8 + 8 * i..16 + 8 * i].copy_from_slice(&p.to_bytes());
        }
        b
    }

    /// C: `sub_02024934`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_02024938`: clear the two words and the words of every sentence (type and id are
    /// kept), then unlock the group of kind 2 ([`INIT_UNLOCKS`]).
    pub fn init(&mut self) {
        self.unlocked = 0;
        self.d04 = 0;
        for p in &mut self.sentences {
            p.words = [0xFFFF; 2];
        }
        if let Some(&(_, bit)) = INIT_UNLOCKS.iter().find(|&&(k, _)| k == 2) {
            self.unlock(bit as u32);
        }
    }

    /// C: unlabeled routine at `0x02024910` (`(block, i)`): sentence `i` (the original returns its
    /// address, block + 8 + 8i).
    pub fn sentence(&self, i: usize) -> &Pms {
        &self.sentences[i]
    }

    /// C: `FUN_02024918(block, i, src)`: sentence `i` = `src` (nothing for `i` outside 0..20,
    /// signed).
    pub fn set_sentence(&mut self, i: u32, src: &Pms) {
        if (i as i32) >= 0 && (i as i32) < SENTENCES as i32 {
            self.sentences[i as usize] = *src;
        }
    }

    /// C: unlabeled routine at `0x02024994` (`(block, bit)`): unlock bit `bit` (a register shift: the low byte of `bit` counts, 32 and up give 0).
    pub fn is_unlocked(&self, bit: u32) -> bool {
        self.unlocked.checked_shr(bit & 0xFF).unwrap_or(0) & 1 != 0
    }

    /// C: `FUN_020249A0(block, bit)`: set unlock bit `bit`.
    pub fn unlock(&mut self, bit: u32) {
        self.unlocked |= 1u32.checked_shl(bit & 0xFF).unwrap_or(0);
    }
}

/// Save block 62.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PmsMessages {
    /// The five sentences.
    pub sentences: [Pms; MESSAGES],
}

impl Default for PmsMessages {
    fn default() -> Self {
        PmsMessages::from_bytes(&[0; MESSAGES_SIZE])
    }
}

impl PmsMessages {
    /// Reads the original 0x28-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        PmsMessages {
            sentences: std::array::from_fn(|i| Pms::from_bytes(&b[8 * i..8 * i + 8])),
        }
    }

    /// The original 0x28-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.sentences.iter().flat_map(|p| p.to_bytes()).collect()
    }

    /// C: `sub_0200E6B0`: block size.
    pub fn size() -> u32 {
        MESSAGES_SIZE as u32
    }

    /// C: `sub_0200E6B4`: the default sentences ([`MESSAGE_DEFAULTS`]).
    pub fn init(&mut self) {
        for (p, &(kind, id, w0)) in self.sentences.iter_mut().zip(&MESSAGE_DEFAULTS) {
            *p = Pms {
                kind,
                id,
                words: [w0, 0xFFFF],
            };
        }
    }

    /// C: `FUN_0200E6FC(block, i, dst)`: sentence `i`.
    pub fn get(&self, i: usize) -> Pms {
        self.sentences[i]
    }

    /// C: `FUN_0200E714(block, i, src)`: sentence `i` = `src`.
    pub fn set(&mut self, i: usize, p: &Pms) {
        self.sentences[i] = *p;
    }
}

#[cfg(test)]
mod tests;
