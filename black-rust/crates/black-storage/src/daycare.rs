//! The daycare save block 50 (S5b; 0x1CC bytes), main ARM9 0x0200BC10..0x0200BDBC (the daycare
//! logic itself is overlay 21 `sodateya.c`; these are the block's accessors). Verified per function
//! with `tools/check-fn.sh black-storage daycare` (spec `tools/rom-diff/specs/black-storage/daycare.spec`).
//!
//! Layout: two 0xE4-byte slots (+0x00 `u32` occupied, 1 = a Pokemon is there; +0x04 the 0xDC-byte
//! party Pokemon; +0xE0 `u32` experience gained), then +0x1C8 `u32` egg flag (1 = an egg is
//! waiting). Slot indices are checked: an index outside 0..2 is ignored (or reads 0).
//!
//! Not ported: `FUN_0200BDB0` (`FUN_020071CC(sav, 50)`, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r32, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_BC10,
    0x0200_BC20,
    0x0200_BC2C,
    0x0200_BC4C,
    0x0200_BC88,
    0x0200_BCA0,
    0x0200_BCA8,
    0x0200_BCB8,
    0x0200_BCD8,
    0x0200_BCEC,
    0x0200_BD48,
    0x0200_BD98,
    0x0200_BDA0,
];

/// Save block number.
pub const BLOCK_ID: u32 = 50;
/// Bytes of the block.
pub const SIZE: usize = 0x1CC;
/// Bytes of one slot.
pub const SLOT_SIZE: usize = 0xE4;
/// Bytes of the stored party Pokemon.
pub const MON_SIZE: usize = 0xDC;
/// Number of slots.
pub const SLOTS: usize = 2;

/// One daycare slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaycareSlot {
    /// +0x00: 1 when a Pokemon is stored.
    pub occupied: u32,
    /// +0x04: the party Pokemon image (0xDC bytes).
    #[serde(with = "crate::save::hex")]
    pub mon: Vec<u8>,
    /// +0xE0: experience gained in the daycare.
    pub exp: u32,
}

impl Default for DaycareSlot {
    fn default() -> Self {
        DaycareSlot {
            occupied: 0,
            mon: vec![0; MON_SIZE],
            exp: 0,
        }
    }
}

/// Save block 50.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Daycare {
    /// +0x000 / +0x0E4: the slots.
    pub slots: [DaycareSlot; SLOTS],
    /// +0x1C8: 1 when an egg is waiting.
    pub egg: u32,
}

impl Daycare {
    /// Reads the original 0x1CC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let slot = |i: usize| {
            let o = i * SLOT_SIZE;
            DaycareSlot {
                occupied: r32(b, o),
                mon: b[o + 4..o + 4 + MON_SIZE].to_vec(),
                exp: r32(b, o + 0xE0),
            }
        };
        Daycare {
            slots: [slot(0), slot(1)],
            egg: r32(b, 0x1C8),
        }
    }

    /// The original 0x1CC-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        for (i, s) in self.slots.iter().enumerate() {
            let o = i * SLOT_SIZE;
            w32(&mut b, o, s.occupied);
            b[o + 4..o + 4 + MON_SIZE].copy_from_slice(&s.mon);
            w32(&mut b, o + 0xE0, s.exp);
        }
        w32(&mut b, 0x1C8, self.egg);
        b
    }

    /// C: `sub_0200BD98`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200BDA0`: zero the block.
    pub fn init(&mut self) {
        *self = Daycare::default();
    }

    /// `i` as a slot index when it is one (the signed `0 <= i < 2` checks).
    fn slot_index(i: u32) -> Option<usize> {
        ((i as i32) >= 0 && (i as i32) < SLOTS as i32).then_some(i as usize)
    }

    /// C: `FUN_0200BC10`: set the egg flag (written only when it is not already 1).
    pub fn set_egg(&mut self) {
        if self.egg != 1 {
            self.egg = 1;
        }
    }

    /// C: `FUN_0200bc20`: clear the egg flag.
    pub fn clear_egg(&mut self) {
        self.egg = 0;
    }

    /// C: `FUN_0200BC2C(block, i)`: the slot whose Pokemon is returned: `i` when it is a valid,
    /// occupied slot, else slot 0 (the original returns the address `block + 0xE4 * slot + 4`).
    pub fn mon_slot(&self, i: u32) -> usize {
        match Self::slot_index(i) {
            Some(k) if self.slots[k].occupied == 1 => k,
            _ => 0,
        }
    }

    /// The stored Pokemon image of [`Daycare::mon_slot`].
    pub fn mon(&self, i: u32) -> &[u8] {
        &self.slots[self.mon_slot(i)].mon
    }

    /// C: `FUN_0200bc4c(block, i, mon)`: store a party Pokemon in a free valid slot (occupied = 1,
    /// experience 0); an occupied or invalid slot is left alone.
    pub fn put(&mut self, i: u32, mon: &[u8]) {
        if let Some(k) = Self::slot_index(i) {
            let s = &mut self.slots[k];
            if s.occupied != 1 {
                s.mon.copy_from_slice(&mon[..MON_SIZE]);
                s.occupied = 1;
                s.exp = 0;
            }
        }
    }

    /// C: `FUN_0200bc88(block, i)`: free an occupied valid slot.
    pub fn take(&mut self, i: u32) {
        if let Some(k) = Self::slot_index(i) {
            if self.slots[k].occupied == 1 {
                self.slots[k].occupied = 0;
            }
        }
    }

    /// C: `sub_0200bca0`: the egg flag.
    pub fn egg(&self) -> u32 {
        self.egg
    }

    /// C: `FUN_0200BCA8(block, i)`: the occupied word of slot `i` (0 for `i >= 2`, unsigned).
    pub fn occupied(&self, i: u32) -> u32 {
        if i < SLOTS as u32 {
            self.slots[i as usize].occupied
        } else {
            0
        }
    }

    /// C: `FUN_0200BCB8`: the number of slots whose occupied word is 1.
    pub fn count(&self) -> u32 {
        self.slots.iter().filter(|s| s.occupied == 1).count() as u32
    }

    /// C: unlabeled routine at `0x0200BCD8` (`(block, i)`): the experience of slot `i` (0 for
    /// `i >= 2`, unsigned).
    pub fn exp(&self, i: u32) -> u32 {
        if i < SLOTS as u32 {
            self.slots[i as usize].exp
        } else {
            0
        }
    }

    /// C: `FUN_0200BCEC(block, i, j)`: swap two slots (both valid, signed checks).
    pub fn swap(&mut self, i: u32, j: u32) {
        if let (Some(a), Some(b)) = (Self::slot_index(i), Self::slot_index(j)) {
            self.slots.swap(a, b);
        }
    }

    /// C: `FUN_0200BD48(block, n)`: add `n` experience to every occupied slot, saturating at
    /// 0xFFFFFFFF.
    pub fn add_exp(&mut self, n: u32) {
        for s in self.slots.iter_mut().filter(|s| s.occupied == 1) {
            s.exp = s.exp.saturating_add(n);
        }
    }
}

#[cfg(test)]
mod tests;
