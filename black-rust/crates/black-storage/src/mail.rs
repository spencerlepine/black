//! `mail.c` (S5): mail records (0x38 bytes) and the mailbox save block 40 (20 records, 0x460
//! bytes), main ARM9 0x02008EAC..0x020090B0. Verified per function with
//! `tools/check-fn.sh black-storage mail` (spec `tools/rom-diff/specs/black-storage/mail.spec`).
//!
//! Mail layout: +0x00 `u32` writer's trainer ID, +0x04 gender, +0x05 language, +0x06 ROM version,
//! +0x07 design (0..11; 0xFF = empty slot), +0x08 writer's name (8 code units), +0x1E `u16`
//! (0xFFFF when cleared; UNVERIFIED role), +0x20 three 8-byte easy-chat sentences ([`Pms`]).
//!
//! Not ported: `FUN_02008F08` / `FUN_02009050` (allocate a mail; replaced by ownership) and
//! `FUN_02008FEC` (`FUN_020071CC(FUN_02012EBC(gamedata), 40)`: the mailbox of the save; the
//! [`crate::save::SaveData`] owns it). The mailbox functions take a location argument (`r1`)
//! that the original ignores; it is dropped here.

use serde::{Deserialize, Serialize};

use crate::mystatus::MyStatus;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8EAC,
    0x0200_8EF8,
    0x0200_8F3C,
    0x0200_8F48,
    0x0200_8F80,
    0x0200_8F84,
    0x0200_8F88,
    0x0200_8F9C,
    0x0200_8FA0,
    0x0200_8FA4,
    0x0200_8FA8,
    0x0200_8FB0,
    0x0200_8FB4,
    0x0200_8FB8,
    0x0200_8FBC,
    0x0200_8FC0,
    0x0200_8FC4,
    0x0200_8FC8,
    0x0200_8FD8,
    0x0200_8FFC,
    0x0200_9004,
    0x0200_9020,
    0x0200_902C,
    0x0200_903C,
    0x0200_9074,
    0x0200_90A0,
];

/// Save block number of the mailbox.
pub const BLOCK_ID: u32 = 40;
/// Bytes of one mail.
pub const MAIL_SIZE: usize = 0x38;
/// Mails in the mailbox block.
pub const MAILBOX_SLOTS: usize = 20;
/// Bytes of the mailbox block.
pub const MAILBOX_SIZE: usize = MAIL_SIZE * MAILBOX_SLOTS;
/// Design byte of an empty slot.
pub const DESIGN_NONE: u8 = 0xFF;
/// Number of mail designs (valid designs are `0..DESIGNS`).
pub const DESIGNS: u32 = 12;
/// Easy-chat sentences per mail.
pub const MAIL_PMS: usize = 3;

/// The ROM's run-time constants a cleared mail is stamped with: the string terminator (`u16` at
/// 0x020A7E7C, read through `FUN_02045A98`), the language byte (0x0209DF0C) and the version byte
/// (0x0209DF08) of the cartridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameConsts {
    /// String terminator code unit.
    pub eos: u16,
    /// Cartridge language.
    pub language: u8,
    /// Cartridge version.
    pub version: u8,
}

impl GameConsts {
    /// The values in the US Black ROM.
    pub const BLACK_US: GameConsts = GameConsts {
        eos: 0xFFFF,
        language: 2,
        version: 0x15,
    };
}

/// An easy-chat sentence (`pms_word.c`, 8 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pms {
    /// +0: sentence type (0xFFFF = none).
    pub kind: u16,
    /// +2: sentence id (not touched by the clear).
    pub id: u16,
    /// +4: the two words (0xFFFF = none).
    pub words: [u16; 2],
}

impl Pms {
    /// Reads the 8-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        Pms {
            kind: crate::le::r16(b, 0),
            id: crate::le::r16(b, 2),
            words: crate::le::r16s(b, 4),
        }
    }

    /// The 8-byte image.
    pub fn to_bytes(&self) -> [u8; 8] {
        let mut b = [0; 8];
        crate::le::w16(&mut b, 0, self.kind);
        crate::le::w16(&mut b, 2, self.id);
        crate::le::w16s(&mut b, 4, &self.words);
        b
    }

    /// `FUN_0202428C` (`pms_word.c`, inlined): type and both words 0xFFFF, the id is kept.
    pub fn clear(&mut self) {
        self.kind = 0xFFFF;
        self.words = [0xFFFF; 2];
    }
}

/// One mail.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mail {
    /// +0x00: writer's trainer ID.
    pub trainer_id: u32,
    /// +0x04: writer's gender.
    pub gender: u8,
    /// +0x05: language.
    pub language: u8,
    /// +0x06: ROM version.
    pub version: u8,
    /// +0x07: design (0xFF = empty).
    pub design: u8,
    /// +0x08: writer's name.
    pub name: [u16; 8],
    /// +0x18..+0x1D: not touched by any accessor.
    pub pad: [u8; 6],
    /// +0x1E (UNVERIFIED role).
    pub field_1e: u16,
    /// +0x20: the sentences.
    pub pms: [Pms; MAIL_PMS],
}

impl Mail {
    /// Reads the original 0x38-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut pad = [0; 6];
        pad.copy_from_slice(&b[0x18..0x1E]);
        Mail {
            trainer_id: crate::le::r32(b, 0),
            gender: b[4],
            language: b[5],
            version: b[6],
            design: b[7],
            name: crate::le::r16s(b, 8),
            pad,
            field_1e: crate::le::r16(b, 0x1E),
            pms: std::array::from_fn(|i| Pms::from_bytes(&b[0x20 + 8 * i..])),
        }
    }

    /// The original 0x38-byte image.
    pub fn to_bytes(&self) -> [u8; MAIL_SIZE] {
        let mut b = [0; MAIL_SIZE];
        crate::le::w32(&mut b, 0, self.trainer_id);
        b[4] = self.gender;
        b[5] = self.language;
        b[6] = self.version;
        b[7] = self.design;
        crate::le::w16s(&mut b, 8, &self.name);
        b[0x18..0x1E].copy_from_slice(&self.pad);
        crate::le::w16(&mut b, 0x1E, self.field_1e);
        for (i, p) in self.pms.iter().enumerate() {
            b[0x20 + 8 * i..0x28 + 8 * i].copy_from_slice(&p.to_bytes());
        }
        b
    }

    /// A cleared mail (`FUN_02008EAC` on a zeroed record).
    pub fn empty(k: GameConsts) -> Self {
        let mut m = Mail::default();
        m.init(k);
        m
    }

    /// C: `FUN_02008EAC`: clear the mail: ID 0, gender 0, the cartridge language and version,
    /// design 0xFF, name filled with the terminator, +0x1E = 0xFFFF, the three sentences cleared.
    /// Bytes +0x18..+0x1D and each sentence's id are left as they are.
    pub fn init(&mut self, k: GameConsts) {
        self.trainer_id = 0;
        self.gender = 0;
        self.language = k.language;
        self.version = k.version;
        self.design = DESIGN_NONE;
        self.name = [k.eos; 8];
        self.field_1e = 0xFFFF;
        for p in &mut self.pms {
            p.clear();
        }
    }

    /// C: `FUN_02008EF8`: does the slot hold a mail (design 0..11).
    pub fn is_valid(&self) -> bool {
        self.design <= 11
    }

    /// C: `FUN_02008F3C(src, dst)`: copy a mail (`MI_CpuCopy8(src, dst, 0x38)`).
    pub fn copy(src: &Mail, dst: &mut Mail) {
        dst.clone_from(src);
    }

    /// C: `FUN_02008F48(mail, design, _, gamedata)`: clear the mail, set the design (low byte),
    /// sign it with the player's name, gender and ID. The original fetches the player status from
    /// the game data (`FUN_02012944`); here it is passed in.
    pub fn create(&mut self, k: GameConsts, design: u32, status: &MyStatus) {
        self.init(k);
        self.design = design as u8;
        self.name = *status.name();
        self.gender = status.gender() as u8;
        self.trainer_id = status.id();
    }

    /// C: `FUN_02008F80`.
    pub fn trainer_id(&self) -> u32 {
        self.trainer_id
    }

    /// C: `FUN_02008F84`: the name (the original returns its address, mail + 8).
    pub fn name(&self) -> &[u16; 8] {
        &self.name
    }

    /// C: `FUN_02008F88(mail, src)`: name = the 16 bytes at `src` (`MI_CpuCopy8(src, mail + 8, 0x10)`).
    pub fn set_name(&mut self, src: &[u16; 8]) {
        self.name = *src;
    }

    /// C: `FUN_02008F9C`.
    pub fn gender(&self) -> u32 {
        self.gender as u32
    }

    /// C: `FUN_02008FA0(mail, v)`.
    pub fn set_gender(&mut self, v: u32) {
        self.gender = v as u8;
    }

    /// C: `FUN_02008FA4`.
    pub fn design(&self) -> u32 {
        self.design as u32
    }

    /// C: `FUN_02008FA8(mail, v)`: only designs below 12 are stored.
    pub fn set_design(&mut self, v: u32) {
        if v < DESIGNS {
            self.design = v as u8;
        }
    }

    /// C: `FUN_02008FB0`.
    pub fn language(&self) -> u32 {
        self.language as u32
    }

    /// C: `FUN_02008FB4(mail, v)`.
    pub fn set_language(&mut self, v: u32) {
        self.language = v as u8;
    }

    /// C: `FUN_02008FB8`.
    pub fn version(&self) -> u32 {
        self.version as u32
    }

    /// C: `FUN_02008FBC(mail, v)`.
    pub fn set_version(&mut self, v: u32) {
        self.version = v as u8;
    }

    /// C: `FUN_02008FC0`.
    pub fn field_1e(&self) -> u32 {
        self.field_1e as u32
    }

    /// C: `FUN_02008FC4(mail, v)`.
    pub fn set_field_1e(&mut self, v: u32) {
        self.field_1e = v as u16;
    }

    /// C: `FUN_02008FC8(mail, i)`: sentence `i`; an index of 3 or more gives sentence 0. Returns
    /// the index used (the original returns the address, mail + 0x20 + 8 * index).
    pub fn pms_index(i: u32) -> usize {
        if i < MAIL_PMS as u32 {
            i as usize
        } else {
            0
        }
    }

    /// C: `FUN_02008FD8(mail, src, i)`: sentence `i` = `src` (`FUN_02024570`, a field copy) when
    /// `i < 3`.
    pub fn set_pms(&mut self, src: &Pms, i: u32) {
        if i < MAIL_PMS as u32 {
            self.pms[i as usize] = *src;
        }
    }
}

/// The mailbox, save block 40.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailBox {
    /// The 20 slots.
    pub mails: Vec<Mail>,
}

impl Default for MailBox {
    fn default() -> Self {
        MailBox {
            mails: vec![Mail::default(); MAILBOX_SLOTS],
        }
    }
}

impl MailBox {
    /// Reads the original 0x460-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        MailBox {
            mails: (0..MAILBOX_SLOTS)
                .map(|i| Mail::from_bytes(&b[i * MAIL_SIZE..]))
                .collect(),
        }
    }

    /// The original 0x460-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.mails.iter().flat_map(|m| m.to_bytes()).collect()
    }

    /// A new-game mailbox (`FUN_02009004`).
    pub fn new_game(k: GameConsts) -> Self {
        let mut b = MailBox::default();
        b.init(k);
        b
    }

    /// C: `FUN_02008FFC`: block size.
    pub fn size() -> u32 {
        MAILBOX_SIZE as u32
    }

    /// C: `FUN_02009004`: clear all 20 mails.
    pub fn init(&mut self, k: GameConsts) {
        for m in &mut self.mails {
            m.init(k);
        }
    }

    /// C: `FUN_02009074(mails, n)`: index of the first of the first `n` slots without a mail,
    /// -1 if none (or `n <= 0`).
    pub fn find_empty_in(&self, n: i32) -> i32 {
        for i in 0..n.max(0) {
            if !self.mails[i as usize].is_valid() {
                return i;
            }
        }
        -1
    }

    /// C: `FUN_02009020(mailbox)`: [`MailBox::find_empty_in`] over all 20 slots.
    pub fn find_empty(&self) -> i32 {
        self.find_empty_in(MAILBOX_SLOTS as i32)
    }

    /// C: unlabeled routine at `0x020090A0` (`(mailbox, location, idx)`): the slot index if
    /// `idx < 20` (signed), else none.
    pub fn slot(idx: i32) -> Option<usize> {
        (idx < MAILBOX_SLOTS as i32).then_some(idx as usize)
    }

    /// C: `FUN_0200902C(mailbox, location, idx)`: clear slot `idx`.
    pub fn clear(&mut self, idx: i32, k: GameConsts) {
        if let Some(i) = Self::slot(idx) {
            self.mails[i].init(k);
        }
    }

    /// C: `FUN_0200903C(mailbox, location, idx, src)`: slot `idx` = `src`.
    pub fn set(&mut self, idx: i32, src: &Mail) {
        if let Some(i) = Self::slot(idx) {
            Mail::copy(src, &mut self.mails[i]);
        }
    }
}

#[cfg(test)]
mod tests;
