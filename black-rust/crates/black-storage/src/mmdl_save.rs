//! The field-object save block 41 (S5b; 0x1400 bytes: 64 records of 0x50 bytes, one per field
//! object (MMDL) alive when the game was saved), main ARM9 0x0200EE48..0x0200F128. Verified per
//! function with `tools/check-fn.sh black-storage mmdl_save` (spec
//! `tools/rom-diff/specs/black-storage/mmdl_save.spec`).
//!
//! A record is the save form of a 0x100-byte MMDL (`black_field::fldmmdl`): [`pack`] writes it from
//! an MMDL image, [`unpack`] rebuilds the MMDL image from it. Record layout (MMDL offsets in
//! brackets): +0x00 `u32` status [+0x00], +0x04 / +0x05 `u8` [+0x7C / +0x7D], +0x08 `u8` OBJ id
//! [+0x08], +0x09 `u8` [+0x0E], +0x0A / +0x0B `s8` [+0x2C / +0x2E], +0x0C..+0x0E `s8` [+0x16 /
//! +0x18 / +0x1A], +0x10 `u16` [+0x0A], +0x12 `u16` OBJ code [+0x0C], +0x14..+0x18 `u16` [+0x10 /
//! +0x12 / +0x14], +0x1A..+0x1E `u16` [+0x20 / +0x22 / +0x24], +0x20..+0x24 `s16` [+0x30 / +0x32 /
//! +0x34], +0x26..+0x2A `s16` grid x / y / z [+0x3C / +0x3E / +0x40], +0x2C `u32` fx32 height
//! [+0x48], +0x30 16 bytes [+0x94], +0x40 16 bytes [+0xA4]; +0x06, +0x07, +0x0F are not written.
//!
//! Not ported: `0x0200EE64` (save every live MMDL of the system into the block) /
//! `0x0200EEAC` (rebuild them) / `FUN_0200F0CC` / `FUN_0200F104` (the MMDL-system iteration, which
//! belongs with `black_field::fldmmdl`'s MMDLSYS), `FUN_0200F128` (OBJ code -> ARC record index,
//! not save data), `FUN_02012C18` / `FUN_02012CE0` (the game-data save / load that fetch the block).

use serde::{Deserialize, Serialize};

use crate::le::{r16, r32, w16, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0200_EE48, 0x0200_EE50, 0x0200_EED8, 0x0200_EF78];

/// Save block number.
pub const BLOCK_ID: u32 = 41;
/// Bytes of the block.
pub const SIZE: usize = 0x1400;
/// Bytes of a record.
pub const RECORD_SIZE: usize = 0x50;
/// Records in the block.
pub const RECORDS: usize = 64;
/// Bytes of an MMDL.
pub const MMDL_SIZE: usize = 0x100;

/// The OBJ-data load [`unpack`] calls.
pub trait MmdlHooks {
    /// C: `FUN_0216E2E8(sys, code, dst)` (overlay 10, fldmmdl): the 16-byte OBJ record of `code`
    /// into `dst` (MMDL + 0xE4).
    fn obj_data(&mut self, sys: u32, code: u32, dst: &mut [u8]);
}

/// Save block 41: the raw records.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MmdlSave {
    /// The 64 records.
    pub records: Vec<Record>,
}

/// One 0x50-byte record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record(#[serde(with = "crate::save::hex")] pub Vec<u8>);

impl Default for MmdlSave {
    fn default() -> Self {
        MmdlSave::from_bytes(&[0; SIZE])
    }
}

impl MmdlSave {
    /// Reads the original 0x1400-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        MmdlSave {
            records: b[..SIZE]
                .chunks(RECORD_SIZE)
                .map(|c| Record(c.to_vec()))
                .collect(),
        }
    }

    /// The original 0x1400-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(|r| r.0.clone()).collect()
    }

    /// C: `FUN_0200EE48`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `FUN_0200EE50`: zero the block.
    pub fn init(&mut self) {
        *self = MmdlSave::default();
    }
}

/// C: `FUN_0200EED8(mmdl, rec)`: write the save record of an MMDL image (the bytes listed in the
/// module doc; the others are left as they are).
pub fn pack(m: &[u8], rec: &mut [u8]) {
    w32(rec, 0, r32(m, 0));
    rec[4] = m[0x7C];
    rec[5] = m[0x7D];
    rec[8] = m[0x08];
    w16(rec, 0x10, r16(m, 0x0A));
    w16(rec, 0x12, r16(m, 0x0C));
    rec[9] = m[0x0E];
    for (d, s) in [(0x14, 0x10), (0x16, 0x12), (0x18, 0x14)] {
        w16(rec, d, r16(m, s));
    }
    rec[0x0C] = m[0x16];
    rec[0x0D] = m[0x18];
    rec[0x0E] = m[0x1A];
    for (d, s) in [(0x1A, 0x20), (0x1C, 0x22), (0x1E, 0x24)] {
        w16(rec, d, r16(m, s));
    }
    rec[0x0A] = m[0x2C];
    rec[0x0B] = m[0x2E];
    for (d, s) in [
        (0x20, 0x30),
        (0x22, 0x32),
        (0x24, 0x34),
        (0x26, 0x3C),
        (0x28, 0x3E),
        (0x2A, 0x40),
    ] {
        w16(rec, d, r16(m, s));
    }
    w32(rec, 0x2C, r32(m, 0x48));
    rec[0x30..0x40].copy_from_slice(&m[0x94..0xA4]);
    rec[0x40..0x50].copy_from_slice(&m[0xA4..0xB4]);
}

/// Sign-extended byte as a halfword.
fn sx8(b: u8) -> u16 {
    b as i8 as i16 as u16
}

/// C: `FUN_0200EF78(mmdl, rec, sys)`: rebuild an MMDL image from a save record: zero it, copy the
/// fields back (bytes widened with sign), system = `sys`, the previous grid = the grid, the fx32
/// position from the grid (+ half a tile) and the saved height, the OBJ data of its code at +0xE4
/// (loaded with status bit 0 clear), +0x7E..+0x80 from that data's bytes 13..15, status bit 0 set,
/// +0x04 bit 11 set (and bit 0 when status bit 1 is set).
pub fn unpack<H: MmdlHooks>(m: &mut [u8], rec: &[u8], sys: u32, h: &mut H) {
    m[..MMDL_SIZE].fill(0);
    w32(m, 0, r32(rec, 0));
    m[0x7C] = rec[4];
    m[0x7D] = rec[5];
    w16(m, 0x08, rec[8] as u16);
    w16(m, 0x0A, r16(rec, 0x10));
    w16(m, 0x0C, r16(rec, 0x12));
    w16(m, 0x0E, rec[9] as u16);
    for (d, s) in [(0x10, 0x14), (0x12, 0x16), (0x14, 0x18)] {
        w16(m, d, r16(rec, s));
    }
    w16(m, 0x16, sx8(rec[0x0C]));
    w16(m, 0x18, sx8(rec[0x0D]));
    w16(m, 0x1A, sx8(rec[0x0E]));
    for (d, s) in [(0x20, 0x1A), (0x22, 0x1C), (0x24, 0x1E)] {
        w16(m, d, r16(rec, s));
    }
    w16(m, 0x2C, sx8(rec[0x0A]));
    w16(m, 0x2E, sx8(rec[0x0B]));
    for (d, s) in [
        (0x30, 0x20),
        (0x32, 0x22),
        (0x34, 0x24),
        (0x3C, 0x26),
        (0x3E, 0x28),
        (0x40, 0x2A),
    ] {
        w16(m, d, r16(rec, s));
    }
    m[0x94..0xA4].copy_from_slice(&rec[0x30..0x40]);
    m[0xA4..0xB4].copy_from_slice(&rec[0x40..0x50]);
    w32(m, 0x88, sys);
    let gx = r16(m, 0x3C);
    let gy = r16(m, 0x3E);
    let gz = r16(m, 0x40);
    w16(m, 0x36, gx);
    w16(m, 0x38, gy);
    w16(m, 0x3A, gz);
    w32(
        m,
        0x44,
        ((gx as i16 as i32) << 16).wrapping_add(0x8000) as u32,
    );
    w32(m, 0x48, r32(rec, 0x2C));
    w32(
        m,
        0x4C,
        ((gz as i16 as i32) << 16).wrapping_add(0x8000) as u32,
    );
    let st = r32(m, 0) & !1;
    w32(m, 0, st);
    let code = r16(m, 0x0C) as u32;
    h.obj_data(sys, code, &mut m[0xE4..0xF4]);
    let st = r32(m, 0) | 1;
    m[0x7E] = m[0xE4 + 13];
    m[0x7F] = m[0xE4 + 14];
    m[0x80] = m[0xE4 + 15];
    let mut w4 = r32(m, 4) | 0x800;
    if st & 2 != 0 {
        w4 |= 1;
    }
    w32(m, 0, st);
    w32(m, 4, w4);
}

#[cfg(test)]
mod tests;
