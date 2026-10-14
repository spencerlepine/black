//! The White Forest / Black City save block 43 (S5b; 0x2DC bytes), main ARM9 0x0200A840..0x0200A8A0
//! (the data's logic is overlay 10 / 20 `field_wfbc_data*.c`, see `black_field::wfbc_data`).
//! Verified per function with `tools/check-fn.sh black-storage wfbc_save` (spec
//! `tools/rom-diff/specs/black-storage/wfbc_save.spec`).
//!
//! Layout: +0x000 the town data (0x1E8 bytes: `u16` +0x00, `u16` +0x02 (2 at init), 4 bytes, then
//! two lists of ten 0x18-byte residents at +0x08 and +0xF8), +0x1E8 ten `u16` (0xFFFF at init),
//! +0x1FC 0xE0 bytes (the first 4 cleared at init with `u16` +0x1FE = 10). The init writes of the
//! overlay-10 routines `FUN_02161D98` / `FUN_02161EEC` / `FUN_02161F00` are inlined.
//!
//! Not ported: `FUN_0200A864` (`FUN_020071CC(sav, 43)`, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r16s, w16, w16s};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_A840,
    0x0200_A848,
    0x0200_A870,
    0x0200_A884,
    0x0200_A890,
    0x0200_A898,
];

/// Save block number.
pub const BLOCK_ID: u32 = 43;
/// Bytes of the block.
pub const SIZE: usize = 0x2DC;
/// Bytes of the town data.
pub const TOWN_SIZE: usize = 0x1E8;
/// Offset of the ten halfwords.
pub const IDS_OFS: usize = 0x1E8;
/// Offset of the tail.
pub const TAIL_OFS: usize = 0x1FC;
/// Bytes of a resident.
pub const RESIDENT_SIZE: usize = 0x18;

/// Save block 43.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WfbcSave {
    /// +0x000: the town data.
    #[serde(with = "crate::save::hex")]
    pub town: Vec<u8>,
    /// +0x1E8.
    pub ids: [u16; 10],
    /// +0x1FC.
    #[serde(with = "crate::save::hex")]
    pub tail: Vec<u8>,
}

impl Default for WfbcSave {
    fn default() -> Self {
        WfbcSave::from_bytes(&[0; SIZE])
    }
}

impl WfbcSave {
    /// Reads the original 0x2DC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        WfbcSave {
            town: b[..TOWN_SIZE].to_vec(),
            ids: r16s(b, IDS_OFS),
            tail: b[TAIL_OFS..SIZE].to_vec(),
        }
    }

    /// The original 0x2DC-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        b[..TOWN_SIZE].copy_from_slice(&self.town);
        w16s(&mut b, IDS_OFS, &self.ids);
        b[TAIL_OFS..].copy_from_slice(&self.tail);
        b
    }

    /// C: `sub_0200A840`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200A848`: town data `u16` +0 = 0, +2 = 2, the twenty residents zeroed
    /// (`FUN_02161D98`); the ten halfwords 0xFFFF (`FUN_02161EEC`); tail bytes 0..4 zeroed, `u16`
    /// +2 = 10 (`FUN_02161F00`). Other bytes are kept.
    pub fn init(&mut self) {
        w16(&mut self.town, 0, 0);
        w16(&mut self.town, 2, 2);
        self.town[8..TOWN_SIZE].fill(0);
        self.ids = [0xFFFF; 10];
        self.tail[..4].fill(0);
        w16(&mut self.tail, 2, 10);
    }

    /// C: `FUN_0200A870(block, src)`: the town data = `src` (0x1E8 bytes).
    pub fn load_town(&mut self, src: &[u8]) {
        self.town.copy_from_slice(&src[..TOWN_SIZE]);
    }

    /// C: `FUN_0200A884(block, dst)`: copy the town data out.
    pub fn store_town(&self, dst: &mut [u8]) {
        dst[..TOWN_SIZE].copy_from_slice(&self.town);
    }

    /// C: unlabeled routine at `0x0200A890`: the ten halfwords (the original returns block + 0x1E8).
    pub fn ids(&mut self) -> &mut [u16; 10] {
        &mut self.ids
    }

    /// C: `FUN_0200a898`: the tail (the original returns block + 0x1FC).
    pub fn tail(&mut self) -> &mut [u8] {
        &mut self.tail
    }
}

#[cfg(test)]
mod tests;
