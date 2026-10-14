//! `config.c` (S5): the game options, one 16-bit bit field in a 4-byte block (main ARM9
//! 0x020082DC..0x02008468). In the save it is the first field of the basic-data block (block 27,
//! [`crate::save_info`]). Verified per function with `tools/check-fn.sh black-storage config`
//! (spec `tools/rom-diff/specs/black-storage/config.spec`).
//!
//! Bit layout (`u16` at +0, +2..+3 unused): bits 0-3 text speed (new game 1), bits 4-5 button mode
//! (0 at new game; the boot code `FUN_0200530C` remaps keys when it is non-zero), bit 6 and bit 7 two
//! on/off options (both 0 at new game; UNVERIFIED which one is the battle scene and which the
//! battle style), bit 8 sound mode (its setter tells the sound system, `FUN_020461D4`), bit 9 (1 at
//! new game) and bit 10 (0) two more options (UNVERIFIED roles), bits 11-15 unused.

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8314,
    0x0200_8320,
    0x0200_8374,
    0x0200_837C,
    0x0200_8390,
    0x0200_8398,
    0x0200_83AC,
    0x0200_83B4,
    0x0200_83C8,
    0x0200_83D0,
    0x0200_83E4,
    0x0200_83EC,
    0x0200_8414,
    0x0200_8428,
    0x0200_8430,
    0x0200_8448,
    0x0200_8450,
];

/// Bytes of the config block (`FUN_020082EC` allocates this many).
pub const SIZE: usize = 4;

/// Undecompiled routines the config code calls.
pub trait ConfigHooks {
    /// C: `FUN_020461D4(mode)`: hand the sound-mode bit (as a byte) to the sound system.
    fn apply_sound_mode(&mut self, mode: u32);
}

/// The options block. Field widths are the bit-field widths; [`Config::from_bytes`] /
/// [`Config::to_bytes`] convert from / to the original 4-byte image losslessly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// Bits 0-3: text speed.
    pub text_speed: u8,
    /// Bits 4-5: button mode.
    pub button_mode: u8,
    /// Bit 6 (UNVERIFIED role: battle scene or battle style).
    pub flag6: bool,
    /// Bit 7 (UNVERIFIED role: battle scene or battle style).
    pub flag7: bool,
    /// Bit 8: sound mode.
    pub sound_mode: bool,
    /// Bit 9 (UNVERIFIED role; 1 at new game).
    pub flag9: bool,
    /// Bit 10 (UNVERIFIED role).
    pub flag10: bool,
    /// Bits 11-15 (unused by the accessors, kept for a lossless image).
    pub high_bits: u8,
    /// Bytes +2..+3 (unused, kept for a lossless image).
    pub pad: u16,
}

impl Config {
    /// The 16-bit field at +0.
    pub fn bits(&self) -> u16 {
        (self.text_speed as u16 & 0xF)
            | (self.button_mode as u16 & 3) << 4
            | (self.flag6 as u16) << 6
            | (self.flag7 as u16) << 7
            | (self.sound_mode as u16) << 8
            | (self.flag9 as u16) << 9
            | (self.flag10 as u16) << 10
            | (self.high_bits as u16 & 0x1F) << 11
    }

    fn set_bits(&mut self, v: u16) {
        self.text_speed = (v & 0xF) as u8;
        self.button_mode = (v >> 4 & 3) as u8;
        self.flag6 = v >> 6 & 1 != 0;
        self.flag7 = v >> 7 & 1 != 0;
        self.sound_mode = v >> 8 & 1 != 0;
        self.flag9 = v >> 9 & 1 != 0;
        self.flag10 = v >> 10 & 1 != 0;
        self.high_bits = (v >> 11) as u8;
    }

    /// Reads the original 4-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut c = Config {
            pad: crate::le::r16(b, 2),
            ..Default::default()
        };
        c.set_bits(crate::le::r16(b, 0));
        c
    }

    /// The original 4-byte image.
    pub fn to_bytes(&self) -> [u8; SIZE] {
        let mut b = [0; SIZE];
        crate::le::w16(&mut b, 0, self.bits());
        crate::le::w16(&mut b, 2, self.pad);
        b
    }

    /// A new-game config (`FUN_02008320` on a fresh block).
    pub fn new_game() -> Self {
        let mut c = Config::default();
        c.init();
        c
    }

    /// C: `FUN_02008314(src, dst)`: copy the 4-byte block (`MI_CpuCopy8(src, dst, 4)`).
    pub fn copy(src: &Config, dst: &mut Config) {
        dst.clone_from(src);
    }

    /// C: `FUN_02008320`: clear the block, text speed 1, bit 9 set, every other option 0.
    pub fn init(&mut self) {
        *self = Config::default();
        let mut v = self.bits();
        v = v & 0xFFF0 | 1;
        v &= 0xFFCF;
        v &= 0xFFBF;
        v &= 0xFF7F;
        v &= 0xFEFF;
        v |= 0x200;
        v &= 0xFBFF;
        self.set_bits(v);
    }

    /// C: `FUN_02008374`: text speed (bits 0-3).
    pub fn text_speed(&self) -> u32 {
        self.bits() as u32 & 0xF
    }

    /// C: `FUN_0200837C(cfg, v)`: text speed = `v & 0xF`.
    pub fn set_text_speed(&mut self, v: u32) {
        self.set_bits(self.bits() & !0xF | (v as u16 & 0xF));
    }

    /// C: `FUN_02008390`: button mode (bits 4-5).
    pub fn button_mode(&self) -> u32 {
        (self.bits() as u32 & 0x3F) >> 4
    }

    /// C: `FUN_02008398(cfg, v)`: button mode = `v & 3`.
    pub fn set_button_mode(&mut self, v: u32) {
        self.set_bits(self.bits() & !0x30 | (v as u16 & 3) << 4);
    }

    /// C: `FUN_020083AC`: bit 7.
    pub fn flag7(&self) -> u32 {
        self.bits() as u32 >> 7 & 1
    }

    /// C: `FUN_020083B4(cfg, v)`: bit 7 = `v & 1`.
    pub fn set_flag7(&mut self, v: u32) {
        self.set_bits(self.bits() & !0x80 | (v as u16 & 1) << 7);
    }

    /// C: `FUN_020083C8`: bit 6.
    pub fn flag6(&self) -> u32 {
        self.bits() as u32 >> 6 & 1
    }

    /// C: `FUN_020083D0(cfg, v)`: bit 6 = `v & 1`.
    pub fn set_flag6(&mut self, v: u32) {
        self.set_bits(self.bits() & !0x40 | (v as u16 & 1) << 6);
    }

    /// C: `FUN_020083E4`: sound mode (bit 8).
    pub fn sound_mode(&self) -> u32 {
        self.bits() as u32 >> 8 & 1
    }

    /// C: `FUN_020083EC(cfg, v)`: sound mode = `v & 1`, then hand it to the sound system.
    pub fn set_sound_mode<H: ConfigHooks>(&mut self, v: u32, h: &mut H) {
        self.set_bits(self.bits() & !0x100 | (v as u16 & 1) << 8);
        h.apply_sound_mode(self.sound_mode());
    }

    /// C: `FUN_02008414`: hand the stored sound mode to the sound system.
    pub fn apply_sound_mode<H: ConfigHooks>(&self, h: &mut H) {
        h.apply_sound_mode(self.sound_mode());
    }

    /// C: `FUN_02008428`: bit 9.
    pub fn flag9(&self) -> u32 {
        self.bits() as u32 >> 9 & 1
    }

    /// C: `FUN_02008430(cfg, v)`: bit 9 = `v & 1`.
    pub fn set_flag9(&mut self, v: u32) {
        self.set_bits(self.bits() & !0x200 | (v as u16 & 1) << 9);
    }

    /// C: `FUN_02008448`: bit 10.
    pub fn flag10(&self) -> u32 {
        self.bits() as u32 >> 10 & 1
    }

    /// C: `FUN_02008450(cfg, v)`: bit 10 = `v & 1`.
    pub fn set_flag10(&mut self, v: u32) {
        self.set_bits(self.bits() & !0x400 | (v as u16 & 1) << 10);
    }
}

#[cfg(test)]
mod tests;
