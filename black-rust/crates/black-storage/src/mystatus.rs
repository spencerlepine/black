//! `mystatus.c` (S5): the player's identity, a 0x20-byte block (main ARM9 0x02008468..0x0200857C).
//! In the save it sits at +4 of the basic-data block (block 27, [`crate::save_info`]). Verified per
//! function with `tools/check-fn.sh black-storage mystatus` (spec
//! `tools/rom-diff/specs/black-storage/mystatus.spec`).
//!
//! Layout: +0x00 name (8 UTF-16 code units, the game's own character codes), +0x10 trainer ID
//! (`u32`: low half = the public ID, high half = the secret ID), +0x14 a second `u32` that is set
//! only once (UNVERIFIED role; a profile / sync ID), +0x18 / +0x19 two bytes set together
//! (UNVERIFIED: country and region), +0x1A language (2 = English at new game), +0x1B ROM version
//! (0x15 = Black at new game), +0x1C a byte (UNVERIFIED role), +0x1D gender (mail copies it),
//! +0x1E..+0x1F unused. Money and badges are not here in Black (they live in block 52, not
//! modelled yet).
//!
//! Not ported (text glue, belongs with the string buffers of `black-text`): `FUN_020084D4`
//! (`FUN_0201F41C(src, status, 8)`, name from a string), `FUN_020084E4` (`FUN_02045A38`),
//! `FUN_020084F8` (tail call to `FUN_0201F41C` with the caller's r2), `FUN_02008500` /
//! `FUN_02008510` (name to a new string buffer). `FUN_0200846C` (allocation) is replaced by
//! ownership.

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8468,
    0x0200_8494,
    0x0200_84A0,
    0x0200_84BC,
    0x0200_84F4,
    0x0200_852C,
    0x0200_8530,
    0x0200_8534,
    0x0200_853C,
    0x0200_8540,
    0x0200_854C,
    0x0200_8550,
    0x0200_8554,
    0x0200_8558,
    0x0200_855C,
    0x0200_8560,
    0x0200_8564,
    0x0200_8568,
    0x0200_856C,
    0x0200_8570,
    0x0200_8574,
];

/// Bytes of the block.
pub const SIZE: usize = 0x20;
/// Language byte of a new game (English).
pub const LANG_NEW_GAME: u8 = 2;
/// ROM version byte of a new game (Black).
pub const VERSION_BLACK: u8 = 0x15;

/// The player-status block.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MyStatus {
    /// +0x00: name, 8 code units.
    pub name: [u16; 8],
    /// +0x10: trainer ID (low half public, high half secret).
    pub id: u32,
    /// +0x14: set-once ID (UNVERIFIED role).
    pub sync_id: u32,
    /// +0x18 (UNVERIFIED: country).
    pub country: u8,
    /// +0x19 (UNVERIFIED: region).
    pub region: u8,
    /// +0x1A: language.
    pub language: u8,
    /// +0x1B: ROM version.
    pub version: u8,
    /// +0x1C (UNVERIFIED role).
    pub field_1c: u8,
    /// +0x1D: gender.
    pub gender: u8,
    /// +0x1E: unused.
    pub pad: u16,
}

impl MyStatus {
    /// Reads the original 0x20-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        use crate::le::*;
        MyStatus {
            name: r16s(b, 0),
            id: r32(b, 0x10),
            sync_id: r32(b, 0x14),
            country: b[0x18],
            region: b[0x19],
            language: b[0x1A],
            version: b[0x1B],
            field_1c: b[0x1C],
            gender: b[0x1D],
            pad: r16(b, 0x1E),
        }
    }

    /// The original 0x20-byte image.
    pub fn to_bytes(&self) -> [u8; SIZE] {
        use crate::le::*;
        let mut b = [0; SIZE];
        w16s(&mut b, 0, &self.name);
        w32(&mut b, 0x10, self.id);
        w32(&mut b, 0x14, self.sync_id);
        b[0x18] = self.country;
        b[0x19] = self.region;
        b[0x1A] = self.language;
        b[0x1B] = self.version;
        b[0x1C] = self.field_1c;
        b[0x1D] = self.gender;
        w16(&mut b, 0x1E, self.pad);
        b
    }

    /// A new-game block (`FUN_020084A0` on a fresh block).
    pub fn new_game() -> Self {
        let mut s = MyStatus::default();
        s.init();
        s
    }

    /// C: `FUN_02008468`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `FUN_02008494(src, dst)`: copy the block (`MI_CpuCopy8(src, dst, 0x20)`).
    pub fn copy(src: &MyStatus, dst: &mut MyStatus) {
        dst.clone_from(src);
    }

    /// C: `FUN_020084A0`: clear, language 2, version 0x15 (through `FUN_02008560`).
    pub fn init(&mut self) {
        *self = MyStatus::default();
        self.language = LANG_NEW_GAME;
        self.set_version(VERSION_BLACK as u32);
    }

    /// C: `FUN_020084BC`: 1 if all 8 name code units are 0 (no name set yet).
    pub fn name_is_empty(&self) -> bool {
        self.name.iter().all(|&c| c == 0)
    }

    /// C: `FUN_020084F4`: the name (the original returns the block address, the name being at +0).
    pub fn name(&self) -> &[u16; 8] {
        &self.name
    }

    /// C: `FUN_0200852C(status, id)`.
    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    /// C: `FUN_02008530`.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// C: `FUN_02008534`: the public half of the trainer ID.
    pub fn public_id(&self) -> u32 {
        self.id & 0xFFFF
    }

    /// C: `FUN_0200853C`.
    pub fn sync_id(&self) -> u32 {
        self.sync_id
    }

    /// C: `FUN_02008540(status, v)`: stores `v` only while the field is still 0.
    pub fn set_sync_id_once(&mut self, v: u32) {
        if self.sync_id == 0 {
            self.sync_id = v;
        }
    }

    /// C: `FUN_0200854C(status, v)`: gender = low byte of `v`.
    pub fn set_gender(&mut self, v: u32) {
        self.gender = v as u8;
    }

    /// C: `FUN_02008550`.
    pub fn gender(&self) -> u32 {
        self.gender as u32
    }

    /// C: `FUN_02008554`.
    pub fn field_1c(&self) -> u32 {
        self.field_1c as u32
    }

    /// C: `FUN_02008558(status, v)`.
    pub fn set_field_1c(&mut self, v: u32) {
        self.field_1c = v as u8;
    }

    /// C: `FUN_0200855C`.
    pub fn version(&self) -> u32 {
        self.version as u32
    }

    /// C: `FUN_02008560(status, v)`.
    pub fn set_version(&mut self, v: u32) {
        self.version = v as u8;
    }

    /// C: `FUN_02008564`.
    pub fn language(&self) -> u32 {
        self.language as u32
    }

    /// C: `FUN_02008568(status, v)`.
    pub fn set_language(&mut self, v: u32) {
        self.language = v as u8;
    }

    /// C: `FUN_0200856C`.
    pub fn country(&self) -> u32 {
        self.country as u32
    }

    /// C: `FUN_02008570`.
    pub fn region(&self) -> u32 {
        self.region as u32
    }

    /// C: `FUN_02008574(status, country, region)`.
    pub fn set_country_region(&mut self, country: u32, region: u32) {
        self.country = country as u8;
        self.region = region as u8;
    }
}

#[cfg(test)]
mod tests;
