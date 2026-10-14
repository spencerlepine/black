//! Save block 67 (S5b; 0x40 bytes: two 0x20-byte Pokemon records), main ARM9 0x0200E730..0x0200E874
//! (callers: overlay 10 `event_battle.c` / `trcard_sys.c`, overlay 21 `scrcmd_pokemon_fld.c`).
//! Verified per function with `tools/check-fn.sh black-storage poke_records` (spec
//! `tools/rom-diff/specs/black-storage/poke_records.spec`).
//!
//! Record (UNVERIFIED roles beyond the parameter ids it is filled from): +0x00 `u32` parameter 0
//! (personality), +0x04 the nickname (11 `u16`), +0x1A `u8` bit 0 = filled, bits 1..7 = parameter 158
//! (UNVERIFIED: level), +0x1B `u8` parameter 10 or 0xFF (UNVERIFIED: held item / ball), +0x1C `u32`
//! parameter 7 (trainer id).
//!
//! Not ported: `FUN_02012FD8` (the block through the game data, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r16s, r32, w16s, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_E730,
    0x0200_E734,
    0x0200_E740,
    0x0200_E74C,
    0x0200_E76C,
    0x0200_E780,
    0x0200_E788,
    0x0200_E7A4,
    0x0200_E7AC,
    0x0200_E7B4,
    0x0200_E7BC,
];

/// Save block number.
pub const BLOCK_ID: u32 = 67;
/// Bytes of the block.
pub const SIZE: usize = 0x40;
/// Bytes of a record.
pub const RECORD_SIZE: usize = 0x20;
/// Characters of the nickname.
pub const NAME_LEN: usize = 11;

/// The Pokemon queries of [`PokeRecords::fill`].
pub trait PokeRecordHooks {
    /// C: `FUN_02017E1C(mon, id, 0)`: a party Pokemon parameter.
    fn param(&mut self, mon: u32, id: u32) -> u32;
    /// C: `FUN_02017E1C(mon, 116, dst)`: the nickname into `dst` (11 characters).
    fn nickname(&mut self, mon: u32, dst: &mut [u16; NAME_LEN]);
    /// C: `FUN_02045F5C(*0x020A7370, species, buf, 11)` + `FUN_0201F41C(buf, dst, 11)`: the
    /// species name (message bank of the global handle 0x020A7370) into `dst`.
    fn species_name(&mut self, species: u32, dst: &mut [u16; NAME_LEN]);
}

/// One record.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PokeRecord {
    /// +0x00: parameter 0.
    pub pid: u32,
    /// +0x04: the nickname.
    pub name: [u16; NAME_LEN],
    /// +0x1A: bit 0 filled, bits 1..7 parameter 158.
    pub b1a: u8,
    /// +0x1B: parameter 10 or 0xFF.
    pub b1b: u8,
    /// +0x1C: parameter 7.
    pub id: u32,
}

impl PokeRecord {
    /// Reads the 0x20-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        PokeRecord {
            pid: r32(b, 0),
            name: r16s(b, 4),
            b1a: b[0x1A],
            b1b: b[0x1B],
            id: r32(b, 0x1C),
        }
    }

    /// The 0x20-byte image.
    pub fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut b = [0u8; RECORD_SIZE];
        w32(&mut b, 0, self.pid);
        w16s(&mut b, 4, &self.name);
        b[0x1A] = self.b1a;
        b[0x1B] = self.b1b;
        w32(&mut b, 0x1C, self.id);
        b
    }
}

/// Save block 67.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PokeRecords {
    /// The two records.
    pub records: [PokeRecord; 2],
}

impl PokeRecords {
    /// Reads the original 0x40-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        PokeRecords {
            records: [
                PokeRecord::from_bytes(&b[..RECORD_SIZE]),
                PokeRecord::from_bytes(&b[RECORD_SIZE..SIZE]),
            ],
        }
    }

    /// The original 0x40-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.records.iter().flat_map(|r| r.to_bytes()).collect()
    }

    /// C: `sub_0200E730`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200E734`: zero the block.
    pub fn init(&mut self) {
        *self = PokeRecords::default();
    }

    /// C: `FUN_0200E740(block, i)`: record `i` is filled (bit 0 of +0x1A).
    pub fn filled(&self, i: usize) -> u32 {
        (self.records[i].b1a & 1) as u32
    }

    /// C: `FUN_0200e74c(block, i)`: bits 1..7 of +0x1A, raised to at least 65 (else + 5), at most
    /// 100.
    pub fn level(&self, i: usize) -> u32 {
        let v = (self.records[i].b1a >> 1) as u32;
        let v = if v < 65 { 65 } else { (v + 5) & 0xFF };
        v.min(100)
    }

    /// C: unlabeled routine at `0x0200E76C` (`(block, i)`): +0x1B is not 0xFF.
    pub fn has_1b(&self, i: usize) -> bool {
        self.records[i].b1b != 0xFF
    }

    /// C: `FUN_0200E780(block, i)`: +0x1B.
    pub fn b1b(&self, i: usize) -> u8 {
        self.records[i].b1b
    }

    /// C: `FUN_0200E788(block, a, b)`: a message id: 0x23C for `a == 1`, else 0x222 for `b == 21`,
    /// else 0x224.
    pub fn message(a: u32, b: u32) -> u32 {
        if a == 1 {
            0x23C
        } else if b == 21 {
            0x222
        } else {
            0x224
        }
    }

    /// C: `FUN_0200E7A4(block, i)`: +0x00.
    pub fn pid(&self, i: usize) -> u32 {
        self.records[i].pid
    }

    /// C: `FUN_0200E7AC(block, i)`: the nickname (the original returns its address, +0x20 i + 4).
    pub fn name(&self, i: usize) -> &[u16; NAME_LEN] {
        &self.records[i].name
    }

    /// C: `FUN_0200E7B4(block, i)`: +0x1C.
    pub fn id(&self, i: usize) -> u32 {
        self.records[i].id
    }

    /// C: `FUN_0200E7BC(block, i, mon)`: fill record `i` from a party Pokemon (+0x1A bits 1..7,
    /// +0x1B, +0x00, +0x1C, the nickname or else the species name), then mark it filled.
    pub fn fill<H: PokeRecordHooks>(&mut self, i: usize, mon: u32, h: &mut H) {
        let r = &mut self.records[i];
        let v = h.param(mon, 158);
        r.b1a = (r.b1a & 1) | (((v & 0x7F) << 1) as u8);
        r.b1b = if h.param(mon, 113) != 0 {
            h.param(mon, 10) as u8
        } else {
            0xFF
        };
        r.pid = h.param(mon, 0);
        r.id = h.param(mon, 7);
        if h.param(mon, 117) != 0 {
            h.nickname(mon, &mut r.name);
        } else {
            let species = h.param(mon, 5);
            h.species_name(species, &mut r.name);
        }
        r.b1a |= 1;
    }
}

#[cfg(test)]
mod tests;
