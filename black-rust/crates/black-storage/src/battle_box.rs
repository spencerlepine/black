//! The Battle Box save block 49 (S5b; 0x35C bytes: a team of six stored for link / facility
//! battles), main ARM9 0x0200BA78..0x0200BC10. Verified per function with
//! `tools/check-fn.sh black-storage battle_box` (spec `tools/rom-diff/specs/black-storage/battle_box.spec`).
//!
//! Layout: +0x000 six 0x88-byte box Pokemon, +0x330 the box name (0x28 bytes, 0x14 characters; message bank 9,
//! entry 6 at init), +0x358 `u16` flags, +0x35A 2 bytes not read here. The addressing takes a
//! box number (`0x330 * box + 0x88 * slot`); only box 0 exists in the block.
//!
//! Not ported: `FUN_0200BAC8` (`FUN_020071CC(sav, 49)`, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r16, r16s, w16, w16s};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_BA78,
    0x0200_BA80,
    0x0200_BAD4,
    0x0200_BAF4,
    0x0200_BB58,
    0x0200_BBA8,
    0x0200_BBC8,
    0x0200_BBD8,
    0x0200_BBEC,
    0x0200_BBFC,
];

/// Save block number.
pub const BLOCK_ID: u32 = 49;
/// Bytes of the block.
pub const SIZE: usize = 0x35C;
/// Bytes of a box Pokemon.
pub const MON_SIZE: usize = 0x88;
/// Number of slots.
pub const SLOTS: usize = 6;
/// Offset of the name.
pub const NAME_OFS: usize = 0x330;
/// Bytes of the name (0x14 characters).
pub const NAME_SIZE: usize = 0x28;
/// Bytes between two boxes in the addressing.
pub const BOX_STRIDE: usize = 0x330;
/// The box-Pokemon parameter "species present" (`FUN_02017E88(mon, 169, 0)`).
pub const PARAM_PRESENT: u32 = 169;

/// The callees of the Battle Box functions (message bank, Pokemon and party routines).
pub trait BattleBoxHooks {
    /// C: `FUN_02045B38(0, 2, 9, 1)`: open message bank 9 (0 = none).
    fn msg_open(&mut self) -> u32;
    /// C: `FUN_02045F5C(msg, 6, name, 0x14)`: entry 6 as a 0x14-character name.
    fn msg_name(&mut self, msg: u32, name: &mut [u8; NAME_SIZE]);
    /// C: `FUN_02045C04(msg)`: close the bank.
    fn msg_close(&mut self, msg: u32);
    /// C: `FUN_02017E88(mon, id, 0)`: a box-Pokemon parameter.
    fn box_param(&mut self, mon: &[u8], id: u32) -> u32;
    /// C: `FUN_0201A920(heap)`: a new party.
    fn party_alloc(&mut self, heap: u32) -> u32;
    /// C: `FUN_0201A954(party, n)`: init it for `n` Pokemon.
    fn party_init(&mut self, party: u32, n: u32);
    /// C: `FUN_0201A98C(party, mon)`: append a party Pokemon.
    fn party_add(&mut self, party: u32, mon: u32);
    /// C: `FUN_0201AA18(party)`: the number of Pokemon.
    fn party_count(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: the Pokemon at `i`.
    fn party_mon(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_020185F4(mon)`: the box part (0x88 bytes) of a party Pokemon.
    fn box_part(&mut self, mon: u32) -> [u8; MON_SIZE];
    /// C: `FUN_0201747C(boxmon, heap)`: a new party Pokemon made from a box Pokemon.
    fn party_mon_from_box(&mut self, mon: &[u8], heap: u32) -> u32;
    /// C: `FUN_020307B0(p)` (`Heap_Free`).
    fn free(&mut self, p: u32);
}

/// Save block 49.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleBox {
    /// +0x000: the six box Pokemon (0x88 bytes each).
    pub mons: Vec<Mon>,
    /// +0x330: the name (0x14 characters).
    pub name: [u16; NAME_SIZE / 2],
    /// +0x358: flags.
    pub flags: u16,
    /// +0x35A: not read here.
    pub pad: u16,
}

/// One box Pokemon image (0x88 bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mon(#[serde(with = "crate::save::hex")] pub Vec<u8>);

impl Default for BattleBox {
    fn default() -> Self {
        BattleBox::from_bytes(&[0; SIZE])
    }
}

impl BattleBox {
    /// Reads the original 0x35C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        BattleBox {
            mons: (0..SLOTS)
                .map(|i| Mon(b[i * MON_SIZE..(i + 1) * MON_SIZE].to_vec()))
                .collect(),
            name: r16s(b, NAME_OFS),
            flags: r16(b, 0x358),
            pad: r16(b, 0x35A),
        }
    }

    /// The original 0x35C-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        for (i, m) in self.mons.iter().enumerate() {
            b[i * MON_SIZE..(i + 1) * MON_SIZE].copy_from_slice(&m.0);
        }
        w16s(&mut b, NAME_OFS, &self.name);
        w16(&mut b, 0x358, self.flags);
        w16(&mut b, 0x35A, self.pad);
        b
    }

    /// C: `sub_0200BA78`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200BA80`: zero, clear the six box Pokemon, the default name from message bank 9
    /// (left zero when the bank does not open).
    pub fn init<H: BattleBoxHooks>(&mut self, h: &mut H) {
        *self = BattleBox::default();
        for m in &mut self.mons {
            black_pokemon::create::clear_box_pokemon(&mut m.0);
        }
        let msg = h.msg_open();
        if msg != 0 {
            let mut name = [0u8; NAME_SIZE];
            h.msg_name(msg, &mut name);
            self.name = r16s(&name, 0);
            h.msg_close(msg);
        }
    }

    /// C: `FUN_0200BBEC(block, box, slot)`: the slot index the original addresses
    /// (`0x330 * box + 0x88 * slot`; box 0 only).
    pub fn slot_offset(bx: u32, slot: u32) -> usize {
        BOX_STRIDE * bx as usize + MON_SIZE * slot as usize
    }

    /// The box Pokemon of slot `slot` (box 0).
    pub fn mon(&self, slot: usize) -> &[u8] {
        &self.mons[slot].0
    }

    /// C: unlabeled routine at `0x0200BAD4` (`(block, box, slot, src)`): copy a box Pokemon in
    /// (box 0).
    pub fn set_mon(&mut self, slot: usize, src: &[u8]) {
        self.mons[slot].0.copy_from_slice(&src[..MON_SIZE]);
    }

    /// C: `FUN_0200BAF4(block, heap)`: a new party (6 places) with the stored Pokemon up to the
    /// first empty slot; returns the party.
    pub fn to_party<H: BattleBoxHooks>(&self, heap: u32, h: &mut H) -> u32 {
        let party = h.party_alloc(heap);
        h.party_init(party, SLOTS as u32);
        let heap16 = (heap & 0x7FFF) | 0x8000;
        for m in &self.mons {
            if h.box_param(&m.0, PARAM_PRESENT) == 0 {
                break;
            }
            let pp = h.party_mon_from_box(&m.0, heap16);
            h.party_add(party, pp);
            h.free(pp);
        }
        party
    }

    /// C: `FUN_0200bb58(block, party)`: clear the six slots, then store the box part of each party
    /// Pokemon (the count is asked again after each one; at most six).
    pub fn from_party<H: BattleBoxHooks>(&mut self, party: u32, h: &mut H) {
        for m in &mut self.mons {
            black_pokemon::create::clear_box_pokemon(&mut m.0);
        }
        let mut i = 0;
        while i < h.party_count(party) {
            let mon = h.party_mon(party, i);
            let bp = h.box_part(mon);
            self.set_mon(i as usize, &bp);
            i += 1;
        }
    }

    /// C: `FUN_0200BBA8(block)`: slot 0 holds a Pokemon.
    pub fn has_team<H: BattleBoxHooks>(&self, h: &mut H) -> bool {
        h.box_param(&self.mons[0].0, PARAM_PRESENT) != 0
    }

    /// C: unlabeled routine at `0x0200BBC8` (`(block, bits)`): set flag bits.
    pub fn set_flags(&mut self, bits: u16) {
        self.flags |= bits;
    }

    /// C: unlabeled routine at `0x0200BBD8` (`(block, bits)`): clear flag bits.
    pub fn clear_flags(&mut self, bits: u16) {
        self.flags &= !bits;
    }

    /// C: unlabeled routine at `0x0200BBFC` (`(block, bits)`): any of the bits set.
    pub fn test_flags(&self, bits: u32) -> bool {
        self.flags as u32 & bits != 0
    }
}

#[cfg(test)]
mod tests;
