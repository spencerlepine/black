//! The game records save block 38 (S5b; 0x1E0 bytes: the counters of the trainer card / records
//! screen, e.g. steps, battles, caught Pokemon), main ARM9 0x02008C20..0x02008EAC. Verified per
//! function with `tools/check-fn.sh black-storage record` (spec
//! `tools/rom-diff/specs/black-storage/record.spec`).
//!
//! Layout: records 0..71 are `u32` at `4 * id`, records 71..167 `u16` at `0x11C + 2 * (id - 71)`;
//! +0x1DC `u16` checksum, +0x1DE `u16` key. Every record has a cap: the byte table at rodata
//! 0x020A70C4 ([`LIMIT_INDEX`]) picks one of six caps (rodata 0x0209DDB4, [`LIMITS`]).
//!
//! The original keeps +0x04..+0x1DC enciphered in RAM (`gf_standard.c` checksum `FUN_0203F2FC`,
//! cipher `FUN_0203F328` / `FUN_0203F37C`, seeded with the checksum and the key); every update
//! opens it, changes one record and seals it again. That is DS save plumbing: the port keeps
//! plaintext and the three calls are the [`RecordCrypt`] hooks ([`Plain`] in the game). Quirk kept:
//! the open / seal flag the updates pass is the record id, so record 0 is changed without them.
//!
//! Not ported: `FUN_02008C58` (`FUN_020071CC(sav, 38)`, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r16, r16s, r32s, w16, w16s, w32s};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8C20,
    0x0200_8C28,
    0x0200_8C64,
    0x0200_8C94,
    0x0200_8CB4,
    0x0200_8CD4,
    0x0200_8CF8,
    0x0200_8D1C,
    0x0200_8D58,
    0x0200_8DAC,
    0x0200_8DF0,
    0x0200_8DFC,
    0x0200_8E30,
    0x0200_8E68,
    0x0200_8E78,
    0x0200_8E88,
    0x0200_8EA0,
];

/// Save block number.
pub const BLOCK_ID: u32 = 38;
/// Bytes of the block.
pub const SIZE: usize = 0x1E0;
/// Number of `u32` records.
pub const WORDS: usize = 71;
/// Number of records.
pub const RECORDS: usize = 167;
/// Offset and length of the enciphered part.
pub const CRYPT_OFS: usize = 4;
/// Bytes of the enciphered part.
pub const CRYPT_LEN: usize = 0x1D8;
/// Record whose maximum [`Record::set_max_123`] keeps (below 7).
pub const ID_123: u32 = 123;
/// Record whose maximum [`Record::set_max_124`] keeps (below 9999).
pub const ID_124: u32 = 124;

/// The cap of each record: an index into [`LIMITS`] (rodata 0x020A70C4).
pub const LIMIT_INDEX: [u8; RECORDS] = [
    0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 2, 3, 4, 3, 3, 5, 3, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, 2, 2, 2, 2, 2, 2,
];

/// The caps (rodata 0x0209DDB4).
pub const LIMITS: [u32; 6] = [999_999_999, 999_999, 0xFFFF, 9999, 999, 6];

/// `(maximum record, current record)` pairs that [`Record::fold_current`] folds (rodata
/// 0x0209DDA0).
pub const MAX_PAIRS: [(u16, u16); 5] = [(26, 27), (81, 82), (83, 84), (85, 86), (87, 88)];

/// The at-rest cipher of the original (replaced: plaintext in the port).
pub trait RecordCrypt {
    /// C: `FUN_0203F2FC(data, 0x1D8)`: the checksum of the enciphered part.
    fn checksum(&mut self, _data: &[u8]) -> u16 {
        0
    }
    /// C: `FUN_0203F328(data, 0x1D8, seed)`: encipher.
    fn encrypt(&mut self, _seed: u32) {}
    /// C: `FUN_0203F37C(data, 0x1D8, seed)`: decipher.
    fn decrypt(&mut self, _seed: u32) {}
    /// The V-count word at 0x02FFFC3C the init reads (twice) for the key.
    fn vcount(&mut self) -> u32 {
        0
    }
}

/// The game's cipher hooks: nothing to do on plaintext.
pub struct Plain;

impl RecordCrypt for Plain {}

/// Save block 38.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// +0x000: records 0..71.
    pub words: Vec<u32>,
    /// +0x11C: records 71..167.
    pub halves: Vec<u16>,
    /// +0x1DC: cipher checksum (kept for a lossless image).
    pub checksum: u16,
    /// +0x1DE: cipher key (idem).
    pub key: u16,
}

impl Default for Record {
    fn default() -> Self {
        Record::from_bytes(&[0; SIZE])
    }
}

impl Record {
    /// Reads the original 0x1E0-byte image (plaintext).
    pub fn from_bytes(b: &[u8]) -> Self {
        Record {
            words: r32s::<WORDS>(b, 0).to_vec(),
            halves: r16s::<{ RECORDS - WORDS }>(b, 0x11C).to_vec(),
            checksum: r16(b, 0x1DC),
            key: r16(b, 0x1DE),
        }
    }

    /// The original 0x1E0-byte image (plaintext).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        w32s(&mut b, 0, &self.words);
        w16s(&mut b, 0x11C, &self.halves);
        w16(&mut b, 0x1DC, self.checksum);
        w16(&mut b, 0x1DE, self.key);
        b
    }

    fn seed(&self) -> u32 {
        self.checksum as u32 | (self.key as u32) << 16
    }

    /// C: `sub_02008C20`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_02008C28`: zero, key = the V-count read twice (`v2 | v1 << 8`), then seal.
    pub fn init<H: RecordCrypt>(&mut self, h: &mut H) {
        *self = Record::default();
        let a = h.vcount();
        let b = h.vcount();
        self.key = (b | a << 8) as u16;
        self.seal(1, h);
    }

    /// C: `FUN_02008C64(block, flag)`: with a nonzero flag: checksum the enciphered part, then
    /// encipher it.
    pub fn seal<H: RecordCrypt>(&mut self, flag: u32, h: &mut H) {
        if flag != 0 {
            self.checksum = h.checksum(&self.to_bytes()[CRYPT_OFS..CRYPT_OFS + CRYPT_LEN]);
            h.encrypt(self.seed());
        }
    }

    /// C: `FUN_02008C94(block, flag)`: with a nonzero flag: decipher.
    pub fn open<H: RecordCrypt>(&self, flag: u32, h: &mut H) {
        if flag != 0 {
            h.decrypt(self.seed());
        }
    }

    /// C: `FUN_02008CB4(block, id)`: record `id` as stored (0 for `id >= 167`; `id` must not be
    /// negative).
    pub fn get_raw(&self, id: u32) -> u32 {
        let id = id as usize;
        if id < WORDS {
            self.words[id]
        } else if id < RECORDS {
            self.halves[id - WORDS] as u32
        } else {
            0
        }
    }

    /// C: `FUN_02008CD4(block, id, v)`: store `v` (truncated for a `u16` record), then return the
    /// stored value ([`Record::get_raw`]).
    pub fn set_raw(&mut self, id: u32, v: u32) -> u32 {
        let i = id as usize;
        if i < WORDS {
            self.words[i] = v;
        } else if i < RECORDS {
            self.halves[i - WORDS] = v as u16;
        }
        self.get_raw(id)
    }

    /// C: `FUN_02008CF8(id)`: the cap of record `id` (`id < 167`).
    pub fn limit(id: u32) -> u32 {
        let k = LIMIT_INDEX[id as usize] as usize;
        if k >= RECORDS || k >= LIMITS.len() {
            0
        } else {
            LIMITS[k]
        }
    }

    /// C: `FUN_02008D1C(block, id, v)`: set record `id` to `min(v, cap)`; returns the stored value.
    pub fn set<H: RecordCrypt>(&mut self, id: u32, v: u32, h: &mut H) -> u32 {
        let cap = Self::limit(id);
        self.open(id, h);
        let r = self.set_raw(id, v.min(cap));
        self.seal(id, h);
        r
    }

    /// C: `FUN_02008d58(block, id, v)`: keep the maximum: `v` (capped) replaces a smaller record;
    /// a record above the cap is lowered to it. Returns the record afterwards.
    pub fn set_max<H: RecordCrypt>(&mut self, id: u32, v: u32, h: &mut H) -> u32 {
        let cap = Self::limit(id);
        self.open(id, h);
        let mut r = self.get_raw(id);
        let v = v.min(cap);
        if r < v {
            r = self.set_raw(id, v);
        } else if r > cap {
            r = self.set_raw(id, cap);
        }
        self.seal(id, h);
        r
    }

    /// C: `sub_02008DAC(block, id, n)`: add `n` (wrapping) and cap; returns the stored value.
    pub fn add<H: RecordCrypt>(&mut self, id: u32, n: u32, h: &mut H) -> u32 {
        let cap = Self::limit(id);
        self.open(id, h);
        let s = self.get_raw(id).wrapping_add(n);
        let r = self.set_raw(id, s.min(cap));
        self.seal(id, h);
        r
    }

    /// C: `FUN_02008DF0(block, id)`: [`Record::add`] 1.
    pub fn inc<H: RecordCrypt>(&mut self, id: u32, h: &mut H) -> u32 {
        self.add(id, 1, h)
    }

    /// C: `sub_02008DFC(block, id)`: record `id`, capped.
    pub fn get<H: RecordCrypt>(&mut self, id: u32, h: &mut H) -> u32 {
        let cap = Self::limit(id);
        self.open(id, h);
        let v = self.get_raw(id);
        self.seal(id, h);
        v.min(cap)
    }

    /// C: `FUN_02008E30(block)`: for the five [`MAX_PAIRS`]: the maximum record keeps the larger
    /// value, the current record is reset to 0.
    pub fn fold_current<H: RecordCrypt>(&mut self, h: &mut H) {
        for &(max, cur) in &MAX_PAIRS {
            let v = self.get(cur as u32, h);
            self.set_max(max as u32, v, h);
            self.set(cur as u32, 0, h);
        }
    }

    /// C: `FUN_02008e68(block, v)`: [`Record::set_max`] of record 123 for `v < 7`.
    pub fn set_max_123<H: RecordCrypt>(&mut self, v: u32, h: &mut H) {
        if v < 7 {
            self.set_max(ID_123, v, h);
        }
    }

    /// C: `FUN_02008E78(block)`: record 123 (low byte).
    pub fn get_123<H: RecordCrypt>(&mut self, h: &mut H) -> u8 {
        self.get(ID_123, h) as u8
    }

    /// C: `FUN_02008e88(block, v)`: [`Record::set_max`] of record 124 for `v < 9999`.
    pub fn set_max_124<H: RecordCrypt>(&mut self, v: u32, h: &mut H) {
        if v < 9999 {
            self.set_max(ID_124, v, h);
        }
    }

    /// C: unlabeled routine at `0x02008EA0` (`(block)`): record 124.
    pub fn get_124<H: RecordCrypt>(&mut self, h: &mut H) -> u32 {
        self.get(ID_124, h)
    }
}

#[cfg(test)]
mod tests;
