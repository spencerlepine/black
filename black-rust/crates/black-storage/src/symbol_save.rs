//! `symbol_save.c` (S5): the symbol-encounter save block 61 (0x850 bytes; Pokemon that appear as
//! map symbols), main ARM9 0x0200E024..0x0200E368. Verified per function with
//! `tools/check-fn.sh black-storage symbol_save` (spec `tools/rom-diff/specs/black-storage/symbol_save.spec`).
//!
//! Layout: +0x000 530 `u32` records (bits 0-10 species, 0 = free slot; 11-20, 21-22, 23-28,
//! 29-31 four more fields, UNVERIFIED roles), grouped in four zones by index (rodata 0x0209DE90:
//! `[0, 10)`, `[10, 20)`, `[20, 50)`, `[50, 530)`); +0x848 a byte set to 1 by the first zone-2
//! record; +0x849 a byte derived from the slot of the last zone-3 record; +0x84A unused;
//! +0x84C `u16` checksum and +0x84E `u16` key of the at-rest cipher.
//!
//! The original keeps the block enciphered in RAM: `FUN_0200E024` seals it (key from the
//! V-count, `gf_standard.c` checksum and LCG cipher), `FUN_0200E06C` opens it. That is DS save
//! plumbing: the port keeps plaintext and the two calls are the [`SymbolCrypt`] hooks
//! (no-ops in the game; the differential test routes them to stubs). Not ported:
//! `FUN_0200E0B0` (`FUN_020071CC(sav, 61)`, ownership), `FUN_0200E2BC` (search a record in the
//! copy held in the extra save area: save plumbing).

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_E090,
    0x0200_E098,
    0x0200_E0BC,
    0x0200_E0E8,
    0x0200_E124,
    0x0200_E164,
    0x0200_E184,
    0x0200_E18C,
    0x0200_E1A8,
    0x0200_E27C,
];

/// Save block number of the symbol block.
pub const BLOCK_ID: u32 = 61;
/// Bytes of the block.
pub const SIZE: usize = 0x850;
/// Number of records.
pub const RECORDS: usize = 0x212;
/// The zones (rodata 0x0209DE90): `(start, end)` record indices.
pub const ZONES: [(u16, u16); 4] = [(0, 10), (10, 20), (20, 50), (50, 530)];
/// "No free slot" from [`SymbolSave::find_free`].
pub const NO_SLOT: u32 = 0xFFFF;

/// The at-rest cipher of the original (replaced: plaintext in the port).
pub trait SymbolCrypt {
    /// C: `FUN_0200E024(block)`: encipher (and checksum) the block.
    fn seal(&mut self) {}
    /// C: `FUN_0200E06C(block)`: decipher the block.
    fn open(&mut self) {}
}

/// The game's cipher hooks: nothing to do on plaintext.
pub struct Plain;

impl SymbolCrypt for Plain {}

/// Save block 61.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSave {
    /// +0x000: the records.
    pub records: Vec<u32>,
    /// +0x848.
    pub zone2_seen: u8,
    /// +0x849.
    pub zone3_level: u8,
    /// +0x84A: unused.
    pub pad: u16,
    /// +0x84C: cipher checksum (meaningless in plaintext; kept for a lossless image).
    pub checksum: u16,
    /// +0x84E: cipher key (idem).
    pub key: u16,
}

impl Default for SymbolSave {
    fn default() -> Self {
        SymbolSave::from_bytes(&[0; SIZE])
    }
}

/// The packed record of [`SymbolSave::add`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SymbolRecord {
    /// Bits 0-10.
    pub species: u32,
    /// Bits 11-20.
    pub a: u32,
    /// Bits 21-22.
    pub b: u32,
    /// Bits 23-28 (the low byte of the argument is used).
    pub c: u32,
    /// Bits 29-31 (the low byte of the argument is used).
    pub d: u32,
}

impl SymbolSave {
    /// Reads the original 0x850-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        SymbolSave {
            records: (0..RECORDS).map(|i| crate::le::r32(b, 4 * i)).collect(),
            zone2_seen: b[0x848],
            zone3_level: b[0x849],
            pad: crate::le::r16(b, 0x84A),
            checksum: crate::le::r16(b, 0x84C),
            key: crate::le::r16(b, 0x84E),
        }
    }

    /// The original 0x850-byte image (plaintext).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0; SIZE];
        crate::le::w32s(&mut b, 0, &self.records);
        b[0x848] = self.zone2_seen;
        b[0x849] = self.zone3_level;
        crate::le::w16(&mut b, 0x84A, self.pad);
        crate::le::w16(&mut b, 0x84C, self.checksum);
        crate::le::w16(&mut b, 0x84E, self.key);
        b
    }

    /// C: `FUN_0200E090`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200E098`: zero the block, then seal it.
    pub fn init<H: SymbolCrypt>(&mut self, h: &mut H) {
        *self = SymbolSave::default();
        h.seal();
    }

    /// C: `FUN_0200E0BC(index)`: the zone of a record index (3 past the last zone).
    pub fn zone_of(index: u32) -> u32 {
        ZONES
            .iter()
            .position(|&(_, end)| index < end as u32)
            .map_or(3, |k| k as u32)
    }

    /// C: unlabeled routine at `0x0200E0E8` (`(block, zone)`): first free record of a zone (species
    /// bits 0), [`NO_SLOT`] if none.
    pub fn find_free_raw(&self, zone: u32) -> u32 {
        let (start, end) = ZONES[zone as usize];
        (start..end)
            .find(|&i| self.records[i as usize] & 0x7FF == 0)
            .map_or(NO_SLOT, |i| i as u32)
    }

    /// C: `FUN_0200E124(block, index)`: drop record `index` from its zone: the later records of
    /// the zone move down one slot and the zone's last record is freed (its species cleared).
    pub fn remove_raw(&mut self, index: u32) {
        let end = ZONES[Self::zone_of(index) as usize].1 as u32;
        if index < end {
            self.records
                .copy_within(index as usize + 1..end as usize, index as usize);
        }
        self.records[end as usize - 1] &= 0xFFFF_F800;
    }

    /// C: `FUN_0200E164(block, zone)`: [`SymbolSave::find_free_raw`] between open and seal.
    pub fn find_free<H: SymbolCrypt>(&self, zone: u32, h: &mut H) -> u32 {
        h.open();
        let r = self.find_free_raw(zone);
        h.seal();
        r
    }

    /// C: `FUN_0200E184(index)`: [`SymbolSave::zone_of`] (tail call).
    pub fn zone_of_index(index: u32) -> u32 {
        Self::zone_of(index)
    }

    /// C: `FUN_0200E18C(block, index)`: [`SymbolSave::remove_raw`] between open and seal.
    pub fn remove<H: SymbolCrypt>(&mut self, index: u32, h: &mut H) {
        h.open();
        self.remove_raw(index);
        h.seal();
    }

    /// C: `FUN_0200E1A8(block, species, a, b, c, d, zone)`: store a record in the first free slot
    /// of the zone; a full zone drops its first record and takes the last slot. Zone 2 sets the
    /// +0x848 byte (once); zone 3 updates +0x849 from the slot ([`SymbolSave::set_zone3_level`]).
    pub fn add<H: SymbolCrypt>(&mut self, r: SymbolRecord, zone: u32, h: &mut H) {
        h.open();
        let (start, end) = ZONES[zone as usize];
        let slot = self.find_free_raw(zone);
        let i = if slot == NO_SLOT {
            self.remove_raw(start as u32);
            end as usize - 1
        } else {
            slot as usize
        };
        let mut w = self.records[i];
        w = w & 0xFFFF_F800 | r.species & 0x7FF;
        w = w & 0xFFE0_07FF | (r.a & 0x3FF) << 11;
        w = w & 0xFF9F_FFFF | (r.b & 3) << 21;
        w = w & 0xE07F_FFFF | (r.c & 0x3F) << 23;
        w = w & 0x1FFF_FFFF | (r.d & 0xFF) << 29;
        self.records[i] = w;
        if zone == 2 {
            if self.zone2_seen < 1 {
                self.zone2_seen += 1;
            }
        } else if zone == 3 {
            self.set_zone3_level(slot & 0xFFFF);
        }
        h.seal();
    }

    /// C: `FUN_0200E27C(block, slot)`: for a slot in 170..=530, +0x849 = `min((slot - 50) / 60 -
    /// 1, 6)`.
    pub fn set_zone3_level(&mut self, slot: u32) {
        let q = (((slot as i32 - 50) / 60) - 1) as u8;
        if slot != 0xFFFF && (170..=0x212).contains(&slot) {
            self.zone3_level = q.min(6);
        }
    }
}

#[cfg(test)]
mod tests;
