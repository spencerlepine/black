//! The situation save block 28 (S5b; 0x9C bytes: where the player is and where warps / escapes
//! lead), main ARM9 0x02008828..0x020089A8. Verified per function with
//! `tools/check-fn.sh black-storage situation` (spec `tools/rom-diff/specs/black-storage/situation.spec`).
//!
//! Layout (roles from the callers; the location records are opaque here, the field code reads
//! them): +0x00 0x38 bytes (the game data keeps two pointers to it), +0x38 and +0x54 two 0x1C-byte
//! location records, +0x70 `u16` (the game data's `FUN_02012900` / `FUN_02012914` value), +0x72
//! `u8` copied to / from field status +6 (UNVERIFIED: weather), +0x73 / +0x74 `u8`, +0x75 pad,
//! +0x76 `u16` (0x1AB after init), +0x78 `u32`, +0x7C `u16` (egg step counter), +0x7E `u16`
//! (a script counter), +0x80 the saved player position (0x1C bytes: `u16` zone at +0x80, +0x82
//! pad, 12 bytes at +0x84 (UNVERIFIED: x / y / z), `u16` +0x90 / +0x92 / +0x94 / +0x96, `s16`
//! +0x98, `u8` +0x9A / +0x9B).
//!
//! Not ported: `FUN_02008854` (`FUN_020071CC(sav, 28)`, ownership; the functions that fetch the
//! block through it take `&mut self` and the tests call the fetch stub where the original does).

use serde::{Deserialize, Serialize};

use crate::le::{r16, r32, w16, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8828,
    0x0200_882C,
    0x0200_8844,
    0x0200_8848,
    0x0200_884C,
    0x0200_8850,
    0x0200_8860,
    0x0200_88B8,
    0x0200_890C,
    0x0200_892C,
    0x0200_893C,
    0x0200_894C,
    0x0200_8954,
    0x0200_895C,
    0x0200_8964,
    0x0200_896C,
    0x0200_8970,
    0x0200_8978,
    0x0200_8980,
    0x0200_8988,
    0x0200_8990,
    0x0200_8998,
    0x0200_89A0,
];

/// Save block number.
pub const BLOCK_ID: u32 = 28;
/// Bytes of the block.
pub const SIZE: usize = 0x9C;
/// Bytes of the saved player position (+0x80).
pub const POS_SIZE: usize = 0x1C;
/// +0x76 after init.
pub const INIT_76: u16 = 0x1AB;

/// The field-status byte the block mirrors (`field_status.c`, ported in `black_field::event_work`).
pub trait FieldStatusHooks {
    /// C: `FUN_02014468(fs)`: field status byte +6.
    fn status_6(&mut self, fs: u32) -> u8;
    /// C: `FUN_02014464(fs, v)`: field status byte +6 = `v`.
    fn set_status_6(&mut self, fs: u32, v: u8);
}

/// Save block 28.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Situation {
    /// +0x00 (0x38 bytes).
    #[serde(with = "crate::save::hex")]
    pub loc_00: Vec<u8>,
    /// +0x38 (0x1C bytes).
    #[serde(with = "crate::save::hex")]
    pub loc_38: Vec<u8>,
    /// +0x54 (0x1C bytes).
    #[serde(with = "crate::save::hex")]
    pub loc_54: Vec<u8>,
    /// +0x70.
    pub w70: u16,
    /// +0x72 (mirror of field status +6).
    pub status_6: u8,
    /// +0x73.
    pub b73: u8,
    /// +0x74.
    pub b74: u8,
    /// +0x75 (padding).
    pub pad75: u8,
    /// +0x76.
    pub w76: u16,
    /// +0x78.
    pub d78: u32,
    /// +0x7C: egg step counter.
    pub egg_steps: u16,
    /// +0x7E.
    pub w7e: u16,
    /// +0x80: the saved player position (0x1C bytes).
    #[serde(with = "crate::save::hex")]
    pub pos: Vec<u8>,
}

impl Default for Situation {
    fn default() -> Self {
        Situation::from_bytes(&[0; SIZE])
    }
}

impl Situation {
    /// Reads the original 0x9C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        Situation {
            loc_00: b[0..0x38].to_vec(),
            loc_38: b[0x38..0x54].to_vec(),
            loc_54: b[0x54..0x70].to_vec(),
            w70: r16(b, 0x70),
            status_6: b[0x72],
            b73: b[0x73],
            b74: b[0x74],
            pad75: b[0x75],
            w76: r16(b, 0x76),
            d78: r32(b, 0x78),
            egg_steps: r16(b, 0x7C),
            w7e: r16(b, 0x7E),
            pos: b[0x80..SIZE].to_vec(),
        }
    }

    /// The original 0x9C-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        b[0..0x38].copy_from_slice(&self.loc_00);
        b[0x38..0x54].copy_from_slice(&self.loc_38);
        b[0x54..0x70].copy_from_slice(&self.loc_54);
        w16(&mut b, 0x70, self.w70);
        b[0x72] = self.status_6;
        b[0x73] = self.b73;
        b[0x74] = self.b74;
        b[0x75] = self.pad75;
        w16(&mut b, 0x76, self.w76);
        w32(&mut b, 0x78, self.d78);
        w16(&mut b, 0x7C, self.egg_steps);
        w16(&mut b, 0x7E, self.w7e);
        b[0x80..].copy_from_slice(&self.pos);
        b
    }

    /// C: `FUN_02008828`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200882C`: zero the block, +0x76 = 0x1AB.
    pub fn init(&mut self) {
        *self = Situation::default();
        self.w76 = INIT_76;
    }

    /// C: `FUN_02008844` / `FUN_02008848` / `FUN_0200884C` / `FUN_02008850`: the offsets of the
    /// records (+0x00, +0x38, +0x54, +0x70) the original returns pointers to.
    pub const RECORD_OFFSETS: [u32; 4] = [0x00, 0x38, 0x54, 0x70];

    /// C: `FUN_02008844`: the +0x00 record.
    pub fn loc_00(&self) -> &[u8] {
        &self.loc_00
    }

    /// C: `FUN_02008848`: the +0x38 record.
    pub fn loc_38(&self) -> &[u8] {
        &self.loc_38
    }

    /// C: `FUN_0200884C`: the +0x54 record.
    pub fn loc_54(&self) -> &[u8] {
        &self.loc_54
    }

    /// C: `FUN_02008850`: the +0x70 halfword (the original returns its address).
    pub fn w70(&self) -> u16 {
        self.w70
    }

    /// C: `FUN_02008860(sav, src)`: save the player position from the game data record `src`
    /// (fields +0x00, +0x04..+0x10, +0x10..+0x18 halfwords, +0x1B byte, the low byte of +0x40).
    pub fn store_pos(&mut self, src: &[u8]) {
        let p = &mut self.pos;
        p[0..2].copy_from_slice(&src[0..2]);
        p[4..0x10].copy_from_slice(&src[4..0x10]);
        p[0x10..0x1A].copy_from_slice(&src[0x10..0x1A]);
        p[0x1A] = src[0x1B];
        p[0x1B] = src[0x40];
    }

    /// C: `FUN_020088B8(sav, dst)`: the inverse of [`Situation::store_pos`] (+0x18 is written as the
    /// sign-extended halfword, +0x40 as the zero-extended byte).
    pub fn load_pos(&self, dst: &mut [u8]) {
        let p = &self.pos;
        dst[0..2].copy_from_slice(&p[0..2]);
        dst[4..0x10].copy_from_slice(&p[4..0x10]);
        dst[0x10..0x1A].copy_from_slice(&p[0x10..0x1A]);
        dst[0x1B] = p[0x1A];
        w32(dst, 0x40, p[0x1B] as u32);
    }

    /// C: `FUN_0200890C(sav, dst)`: copy the 0x1C-byte position record out.
    pub fn copy_pos(&self, dst: &mut [u8]) {
        dst[..POS_SIZE].copy_from_slice(&self.pos);
    }

    /// C: `FUN_0200892C(block, fs)`: +0x72 = field status +6.
    pub fn save_status<H: FieldStatusHooks>(&mut self, fs: u32, h: &mut H) {
        self.status_6 = h.status_6(fs);
    }

    /// C: `FUN_0200893C(block, fs)`: field status +6 = +0x72.
    pub fn restore_status<H: FieldStatusHooks>(&self, fs: u32, h: &mut H) {
        h.set_status_6(fs, self.status_6);
    }

    /// C: unlabeled routine at `0x0200894C`: +0x73 = `v`.
    pub fn set_b73(&mut self, v: u8) {
        self.b73 = v;
    }

    /// C: `FUN_02008954(block, out)`: +0x73 (the original stores it through `out`).
    pub fn b73(&self) -> u8 {
        self.b73
    }

    /// C: unlabeled routine at `0x0200895C`: +0x74 = `v`.
    pub fn set_b74(&mut self, v: u8) {
        self.b74 = v;
    }

    /// C: `FUN_02008964(block, out)`: +0x74 (stored through `out`).
    pub fn b74(&self) -> u8 {
        self.b74
    }

    /// C: unlabeled routine at `0x0200896C`: +0x78 = `v`.
    pub fn set_d78(&mut self, v: u32) {
        self.d78 = v;
    }

    /// C: `FUN_02008970(block, out)`: +0x78 (stored through `out`).
    pub fn d78(&self) -> u32 {
        self.d78
    }

    /// C: `FUN_02008978`: the egg step counter.
    pub fn egg_steps(&self) -> u16 {
        self.egg_steps
    }

    /// C: `FUN_02008980`: set the egg step counter.
    pub fn set_egg_steps(&mut self, v: u16) {
        self.egg_steps = v;
    }

    /// C: `FUN_02008988`: +0x7E = `v`.
    pub fn set_w7e(&mut self, v: u16) {
        self.w7e = v;
    }

    /// C: `FUN_02008990`: +0x7E.
    pub fn w7e(&self) -> u16 {
        self.w7e
    }

    /// C: `FUN_02008998`: +0x76.
    pub fn w76(&self) -> u16 {
        self.w76
    }

    /// C: `sub_020089A0`: +0x76 = `v`.
    pub fn set_w76(&mut self, v: u16) {
        self.w76 = v;
    }
}

#[cfg(test)]
mod tests;
