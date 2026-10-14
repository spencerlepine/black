//! `battle_rec.c` (S5): the battle-recorder save block 48 (0x94 bytes) and the pure helpers of
//! the battle-video header (main ARM9 0x0200B364..0x0200BA78). Verified per function with
//! `tools/check-fn.sh black-storage battle_rec` (spec `tools/rom-diff/specs/black-storage/battle_rec.spec`).
//!
//! Block 48 layout: +0x00 `i32` (-1 when empty; UNVERIFIED role, a recording id), +0x04 32 `u32`
//! words, +0x84 16 bytes the original only hands out (`0x0200B3A8`).
//!
//! The battle video itself (0x18A4 bytes, held in a global at 0x0214616C and stored in the extra
//! save areas with a CRC and the LCG cipher) is DS save plumbing and is not ported here:
//! `FUN_0200B3DC`..`FUN_0200B4B8` (allocate / free / clear / checksum the global video),
//! `FUN_0200B4E0`, `FUN_0200B570`, `0x0200B5A4`, `FUN_0200B5CC`, `FUN_0200B67C`, `FUN_0200B6E0`
//! (load / save through the extra areas), `0x0200B7D8` (header from the recorded parties),
//! `FUN_0200B888` / `FUN_0200B8AC` / `FUN_0200BA44` (magic 0xE281 and CRC checks),
//! `FUN_0200B900` / `FUN_0200B908` (cipher veneers), `FUN_0200B910`..`FUN_0200B938` (global
//! accessors), `FUN_0200BA58` / `FUN_0200BA68` (overlay 0xBC load / unload).

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_B364,
    0x0200_B368,
    0x0200_B37C,
    0x0200_B394,
    0x0200_B3A4,
    0x0200_B3A8,
    0x0200_B3AC,
    0x0200_B3B0,
    0x0200_B3C8,
    0x0200_B784,
    0x0200_B98C,
    0x0200_BA38,
];

/// Save block number of the battle-recorder block.
pub const BLOCK_ID: u32 = 48;
/// Bytes of the block.
pub const SIZE: usize = 0x94;
/// Bytes of a battle-video header.
pub const HEADER_SIZE: usize = 0x44;

/// The battle mode table (rodata 0x0209DDCC, 41 `u16`).
pub const MODES: [u16; 41] = [
    24, 536, 152, 664, 25, 537, 153, 665, 26, 538, 154, 666, 27, 539, 155, 667, 28, 540, 156, 668,
    0, 1, 4, 40, 41, 42, 43, 170, 104, 105, 106, 107, 234, 56, 184, 57, 185, 58, 186, 59, 187,
];

/// Save block 48.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleRecBlock {
    /// +0x00: -1 when empty.
    pub id: i32,
    /// +0x04.
    pub words: Vec<u32>,
    /// +0x84.
    pub tail: [u8; 16],
}

impl Default for BattleRecBlock {
    fn default() -> Self {
        BattleRecBlock::from_bytes(&[0; SIZE])
    }
}

impl BattleRecBlock {
    /// Reads the original 0x94-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut tail = [0; 16];
        tail.copy_from_slice(&b[0x84..0x94]);
        BattleRecBlock {
            id: crate::le::r32(b, 0) as i32,
            words: (0..32).map(|i| crate::le::r32(b, 4 + 4 * i)).collect(),
            tail,
        }
    }

    /// The original 0x94-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0; SIZE];
        crate::le::w32(&mut b, 0, self.id as u32);
        crate::le::w32s(&mut b, 4, &self.words);
        b[0x84..].copy_from_slice(&self.tail);
        b
    }

    /// A new-game block (`sub_0200B368`).
    pub fn new_game() -> Self {
        let mut b = BattleRecBlock::default();
        b.init();
        b
    }

    /// C: `sub_0200B364`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200B368`: zero the block, id -1.
    pub fn init(&mut self) {
        *self = BattleRecBlock::default();
        self.id = -1;
    }

    /// C: `FUN_0200B37C`: id -1, the 32 words 0 (the tail is kept).
    pub fn reset(&mut self) {
        self.id = -1;
        self.words.iter_mut().for_each(|w| *w = 0);
    }

    /// C: `FUN_0200B394(block, id)`: reset, then store `id`; returns the words (the original
    /// returns their address, block + 4).
    pub fn start(&mut self, id: u32) -> &mut [u32] {
        self.reset();
        self.id = id as i32;
        &mut self.words
    }

    /// C: `FUN_0200B3A4`: the words (block + 4).
    pub fn words_mut(&mut self) -> &mut [u32] {
        &mut self.words
    }

    /// C: unlabeled routine at `0x0200B3A8`: the tail (block + 0x84).
    pub fn tail_mut(&mut self) -> &mut [u8; 16] {
        &mut self.tail
    }

    /// C: `FUN_0200B3AC`: the id word.
    pub fn id(&self) -> u32 {
        self.id as u32
    }

    /// C: `FUN_0200B3B0(src, sav)`: block = `src` (`MIi_CpuCopy32`).
    pub fn load(&mut self, src: &BattleRecBlock) {
        self.clone_from(src);
    }

    /// C: `FUN_0200B3C8(sav, dst)`: `dst` = block (`MIi_CpuCopy32`).
    pub fn store(&self, dst: &mut BattleRecBlock) {
        dst.clone_from(self);
    }
}

/// C: `FUN_0200B784(mode, &count, &per_side)`: the party layout of a battle mode: 4 parties of 3
/// for the modes at [`MODES`] indices 16..=19 and 22, else 2 parties of 6.
pub fn party_layout(mode: u32) -> (u32, u32) {
    for (i, &m) in MODES.iter().enumerate() {
        if mode == m as u32 {
            let k = i.wrapping_sub(16);
            if k <= 6 && k != 4 && k != 5 {
                return (4, 3);
            }
        }
    }
    (2, 6)
}

/// A battle-video header (0x44 bytes; built by `0x0200B7D8`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoHeader {
    /// +0x00: species per slot.
    pub species: [u16; 12],
    /// +0x18: per slot, bits 0-6 level, bit 7 a flag.
    pub levels: [u8; 12],
    /// +0x24.
    pub field_24: u16,
    /// +0x26: battle mode (a [`MODES`] value).
    pub mode: u16,
    /// +0x28: 0xE281 when valid.
    pub magic: u16,
    /// +0x2A.
    pub field_2a: u8,
    /// +0x2B (100 when built).
    pub field_2b: u8,
    /// +0x2C..+0x37.
    pub pad: [u8; 12],
    /// +0x38.
    pub id: u64,
    /// +0x40: CRC-16 of bytes 0..0x38 (`FUN_0203F1FC`).
    pub crc: u16,
    /// +0x42.
    pub pad2: u16,
}

impl VideoHeader {
    /// Reads the original 0x44-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut levels = [0; 12];
        levels.copy_from_slice(&b[0x18..0x24]);
        let mut pad = [0; 12];
        pad.copy_from_slice(&b[0x2C..0x38]);
        VideoHeader {
            species: crate::le::r16s(b, 0),
            levels,
            field_24: crate::le::r16(b, 0x24),
            mode: crate::le::r16(b, 0x26),
            magic: crate::le::r16(b, 0x28),
            field_2a: b[0x2A],
            field_2b: b[0x2B],
            pad,
            id: crate::le::r32(b, 0x38) as u64 | (crate::le::r32(b, 0x3C) as u64) << 32,
            crc: crate::le::r16(b, 0x40),
            pad2: crate::le::r16(b, 0x42),
        }
    }

    /// The original 0x44-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0; HEADER_SIZE];
        crate::le::w16s(&mut b, 0, &self.species);
        b[0x18..0x24].copy_from_slice(&self.levels);
        crate::le::w16(&mut b, 0x24, self.field_24);
        crate::le::w16(&mut b, 0x26, self.mode);
        crate::le::w16(&mut b, 0x28, self.magic);
        b[0x2A] = self.field_2a;
        b[0x2B] = self.field_2b;
        b[0x2C..0x38].copy_from_slice(&self.pad);
        crate::le::w32(&mut b, 0x38, self.id as u32);
        crate::le::w32(&mut b, 0x3C, (self.id >> 32) as u32);
        crate::le::w16(&mut b, 0x40, self.crc);
        crate::le::w16(&mut b, 0x42, self.pad2);
        b
    }

    /// C: `FUN_0200B98C(hdr, which, idx)` (64-bit result): 0 species of slot `idx` (0 above
    /// 0x289), 1 level of slot `idx`, 2 `field_24` (9999 above 9999), 3 the [`MODES`] index of the
    /// mode (0 if absent), 4 the id, 5 `field_2a`, 6 the flag bit of slot `idx`, 7 `field_2b`,
    /// anything else 0. `idx` must stay below 12.
    pub fn param(&self, which: u32, idx: u32) -> u64 {
        let i = idx as usize;
        match which {
            0 => {
                let s = self.species[i] as u64;
                if s > 0x289 {
                    0
                } else {
                    s
                }
            }
            1 => (self.levels[i] & 0x7F) as u64,
            2 => {
                // above 9999 the original returns the compare constant (9999) still in r0
                (self.field_24 as u64).min(9999)
            }
            3 => MODES
                .iter()
                .position(|&m| m == self.mode)
                .map_or(0, |p| p as u64),
            4 => self.id,
            5 => self.field_2a as u64,
            6 => (self.levels[i] >> 7) as u64,
            7 => self.field_2b as u64,
            _ => 0,
        }
    }
}

/// C: `FUN_0200BA38(level)`: `level <= 100`.
pub fn level_ok(level: u32) -> bool {
    level <= 100
}

#[cfg(test)]
mod tests;
