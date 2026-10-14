//! Save block 27 (S5), the basic-data block: config, player status and play time plus the
//! save-system bookkeeping (main ARM9 0x02008700..0x02008828, an unnamed file between
//! `playtime.c` and the block-28 code). Verified per function with
//! `tools/check-fn.sh black-storage save_info` (spec `tools/rom-diff/specs/black-storage/save_info.spec`).
//!
//! Layout (0x68 bytes): +0x00 [`Config`] (4), +0x04 [`MyStatus`] (0x20), +0x24 [`PlayTime`] (8),
//! +0x2C a flag byte (set / cleared by the save code: `FUN_02007150` stores it, `FUN_02007168` /
//! `FUN_020071A8` test it), +0x2D..+0x2F unused, +0x30 ten `u16` "extra save area valid" marks
//! (0xC21E when area `i` holds data), +0x44 nine `u32` per-area words (the original hands out
//! their addresses; UNVERIFIED use).
//!
//! The original functions take the save control and fetch the block with `FUN_020071CC(sav, 27)`;
//! here they are methods of the block itself (the [`crate::save::SaveData`] owns it).

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::mystatus::MyStatus;
use crate::playtime::PlayTime;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8700,
    0x0200_8704,
    0x0200_8730,
    0x0200_873C,
    0x0200_8748,
    0x0200_8754,
    0x0200_8764,
    0x0200_8774,
    0x0200_8784,
    0x0200_87A8,
    0x0200_87C0,
    0x0200_87D4,
    0x0200_87E8,
    0x0200_8808,
];

/// Save block number of the basic-data block.
pub const BLOCK_ID: u32 = 27;
/// Bytes of the block.
pub const SIZE: usize = 0x68;
/// Number of extra-area marks at +0x30.
pub const EXTRA_MARKS: usize = 10;
/// Number of extra-area words at +0x44.
pub const EXTRA_WORDS: usize = 9;
/// The "extra area holds data" mark.
pub const EXTRA_MAGIC: u16 = 0xC21E;

/// Offset of the player status in the block.
pub const MYSTATUS_OFS: u32 = 4;
/// Offset of the play time in the block.
pub const PLAYTIME_OFS: u32 = 0x24;

/// The basic-data block.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveInfo {
    /// +0x00.
    pub config: Config,
    /// +0x04.
    pub my_status: MyStatus,
    /// +0x24.
    pub play_time: PlayTime,
    /// +0x2C: save-system flag.
    pub flag: u8,
    /// +0x2D..+0x2F: unused.
    pub pad: [u8; 3],
    /// +0x30: extra-area marks.
    pub extra_marks: [u16; EXTRA_MARKS],
    /// +0x44: extra-area words.
    pub extra_words: [u32; EXTRA_WORDS],
}

impl SaveInfo {
    /// Reads the original 0x68-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        SaveInfo {
            config: Config::from_bytes(&b[0..4]),
            my_status: MyStatus::from_bytes(&b[4..0x24]),
            play_time: PlayTime::from_bytes(&b[0x24..0x2C]),
            flag: b[0x2C],
            pad: [b[0x2D], b[0x2E], b[0x2F]],
            extra_marks: crate::le::r16s(b, 0x30),
            extra_words: crate::le::r32s(b, 0x44),
        }
    }

    /// The original 0x68-byte image.
    pub fn to_bytes(&self) -> [u8; SIZE] {
        let mut b = [0; SIZE];
        b[0..4].copy_from_slice(&self.config.to_bytes());
        b[4..0x24].copy_from_slice(&self.my_status.to_bytes());
        b[0x24..0x2C].copy_from_slice(&self.play_time.to_bytes());
        b[0x2C] = self.flag;
        b[0x2D..0x30].copy_from_slice(&self.pad);
        crate::le::w16s(&mut b, 0x30, &self.extra_marks);
        crate::le::w32s(&mut b, 0x44, &self.extra_words);
        b
    }

    /// A new-game block (`FUN_02008704`).
    pub fn new_game() -> Self {
        let mut s = SaveInfo::default();
        s.init();
        s
    }

    /// C: `FUN_02008700`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `FUN_02008704`: zero the block (`MIi_CpuClear32`), init config, status and play time,
    /// flag 0.
    pub fn init(&mut self) {
        *self = SaveInfo::default();
        self.config.init();
        self.my_status.init();
        self.play_time.init();
        self.flag = 0;
    }

    /// C: `FUN_02008730(sav)`: the player status (block + 4).
    pub fn my_status_mut(&mut self) -> &mut MyStatus {
        &mut self.my_status
    }

    /// C: `FUN_0200873C(sav)`: the config (block + 0).
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// C: `FUN_02008748(sav)`: the play time (block + 0x24).
    pub fn play_time_mut(&mut self) -> &mut PlayTime {
        &mut self.play_time
    }

    /// C: `FUN_02008754(sav)`: the flag byte.
    pub fn flag(&self) -> u32 {
        self.flag as u32
    }

    /// C: `FUN_02008764(sav)`: flag = 1.
    pub fn set_flag(&mut self) {
        self.flag = 1;
    }

    /// C: `FUN_02008774(sav)`: flag = 0.
    pub fn clear_flag(&mut self) {
        self.flag = 0;
    }

    /// C: `FUN_02008784(sav, i)`: does extra area `i` carry the 0xC21E mark.
    pub fn extra_marked(&self, i: usize) -> bool {
        self.extra_marks[i] == EXTRA_MAGIC
    }

    /// C: `FUN_020087A8(sav, i)`: mark extra area `i`.
    pub fn mark_extra(&mut self, i: usize) {
        self.extra_marks[i] = EXTRA_MAGIC;
    }

    /// C: unlabeled routine at `0x020087C0` (`(sav, i)`): clear the mark of extra area `i`.
    pub fn unmark_extra(&mut self, i: usize) {
        self.extra_marks[i] = 0;
    }

    /// C: `FUN_020087D4(sav, i)` (the decomp's `thunk_FUN_02008216`): the word of extra area `i`
    /// (the original returns its address, block + 0x44 + 4i).
    pub fn extra_word_mut(&mut self, i: usize) -> &mut u32 {
        &mut self.extra_words[i]
    }

    /// C: `FUN_020087E8(sav, src)`: the player status = the 0x20 bytes at `src + 0x20`.
    pub fn load_my_status(&mut self, src: &MyStatus) {
        self.my_status = src.clone();
    }

    /// C: `FUN_02008808(sav, dst)`: copy the player status to `dst + 0x20`.
    pub fn store_my_status(&self, dst: &mut MyStatus) {
        *dst = self.my_status.clone();
    }
}

#[cfg(test)]
mod tests;
