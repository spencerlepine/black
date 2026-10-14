//! `battle_examination.c` (S5): the battle-examination data (0x5C8 bytes, kept in extra save
//! area 5, not in a regular block), main ARM9 0x0200E588..0x0200E6B0. Verified per function with
//! `tools/check-fn.sh black-storage battle_examination` (spec
//! `tools/rom-diff/specs/black-storage/battle_examination.spec`).
//!
//! Layout: +0x000 five 0x120-byte entries (UNVERIFIED content: trainer data), +0x5A0..+0x5C1
//! (0x22 bytes, not read by these functions), +0x5C2 `u16` state (0xD68 = loaded / valid, set to
//! 0x7DA by [`BattleExamination::mark_done`]), +0x5C4 `u16` (bit 15 and bits 0-6 read), +0x5C6
//! `u16` CRC-16 of bytes 0..0x5C6.
//!
//! Not ported: `0x0200E608` (load the data from extra save area 5 through `FUN_020072CC`, then
//! wait for the card: save plumbing).

use serde::{Deserialize, Serialize};

use crate::regulation::RegulationHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_E588,
    0x0200_E590,
    0x0200_E5A0,
    0x0200_E5CC,
    0x0200_E5E4,
    0x0200_E5F8,
    0x0200_E684,
];

/// Bytes of the data.
pub const SIZE: usize = 0x5C8;
/// Bytes of one entry.
pub const ENTRY_SIZE: usize = 0x120;
/// Number of entries.
pub const ENTRIES: usize = 5;
/// Bytes the CRC covers.
pub const CRC_LEN: usize = 0x5C6;
/// State of valid data.
pub const STATE_VALID: u16 = 0xD68;
/// State written by [`BattleExamination::mark_done`].
pub const STATE_DONE: u16 = 0x7DA;

/// The battle-examination data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleExamination {
    /// +0x000: the five entries, 0x120 bytes each.
    pub entries: Vec<u8>,
    /// +0x5A0: 0x22 bytes.
    pub extra: Vec<u8>,
    /// +0x5C2.
    pub state: u16,
    /// +0x5C4.
    pub flags: u16,
    /// +0x5C6.
    pub crc: u16,
}

impl Default for BattleExamination {
    fn default() -> Self {
        BattleExamination::from_bytes(&[0; SIZE])
    }
}

impl BattleExamination {
    /// Reads the original 0x5C8-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        BattleExamination {
            entries: b[..0x5A0].to_vec(),
            extra: b[0x5A0..0x5C2].to_vec(),
            state: crate::le::r16(b, 0x5C2),
            flags: crate::le::r16(b, 0x5C4),
            crc: crate::le::r16(b, 0x5C6),
        }
    }

    /// The original 0x5C8-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0; SIZE];
        b[..0x5A0].copy_from_slice(&self.entries);
        b[0x5A0..0x5C2].copy_from_slice(&self.extra);
        crate::le::w16(&mut b, 0x5C2, self.state);
        crate::le::w16(&mut b, 0x5C4, self.flags);
        crate::le::w16(&mut b, 0x5C6, self.crc);
        b
    }

    /// C: `FUN_0200E588`: size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200E590`: zero everything.
    pub fn init(&mut self) {
        *self = BattleExamination::default();
    }

    fn crc_of<H: RegulationHooks>(&self, h: &mut H) -> u16 {
        h.crc16(&self.to_bytes()[..CRC_LEN])
    }

    /// C: `FUN_0200E5A0`: the CRC matches and the state is 0xD68.
    pub fn is_valid<H: RegulationHooks>(&self, h: &mut H) -> bool {
        self.crc == self.crc_of(h) && self.state == STATE_VALID
    }

    /// C: `FUN_0200E5CC`: bit 15 of +0x5C4 clear.
    pub fn flag15_clear(&self) -> bool {
        self.flags & 0x8000 == 0
    }

    /// C: unlabeled routine at `0x0200E5E4`: bits 0-6 of +0x5C4.
    pub fn count(&self) -> u32 {
        (self.flags & 0x7F) as u32
    }

    /// C: unlabeled routine at `0x0200E5F8` (`(data, i)`): entry `i`, or entry 0 for `i >= 5`
    /// (the original returns its address, data + 0x120 * i).
    pub fn entry_index(i: u32) -> usize {
        if i < ENTRIES as u32 {
            i as usize
        } else {
            0
        }
    }

    /// The bytes of an entry.
    pub fn entry(&self, i: u32) -> &[u8] {
        let k = Self::entry_index(i) * ENTRY_SIZE;
        &self.entries[k..k + ENTRY_SIZE]
    }

    /// C: `FUN_0200E684`: when the CRC matches, state = 0x7DA and the CRC is recomputed.
    pub fn mark_done<H: RegulationHooks>(&mut self, h: &mut H) {
        if self.crc == self.crc_of(h) {
            self.state = STATE_DONE;
            self.crc = self.crc_of(h);
        }
    }
}

#[cfg(test)]
mod tests;
