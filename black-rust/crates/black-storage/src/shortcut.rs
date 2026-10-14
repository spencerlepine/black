//! The shortcut (registered Y-button items / menu entries) save block 54 (S5b; 0x1A bytes), main
//! ARM9 0x0200C3F4..0x0200C520. Verified per function with
//! `tools/check-fn.sh black-storage shortcut` (spec `tools/rom-diff/specs/black-storage/shortcut.spec`).
//!
//! Layout: 0x1A bytes of shortcut ids, [`EMPTY`] (25) = free; the lists below scan the first 25
//! and keep the used ids packed at the front. A full list makes [`Shortcut::set`] store a 26th id
//! in the last byte, which the scans never read (kept as the original does).
//!
//! Not ported: `FUN_0200C404` / `FUN_0200C40C` (`FUN_020071CC(sav, 54)` through the game data:
//! ownership).

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_C3F4,
    0x0200_C3F8,
    0x0200_C418,
    0x0200_C44C,
    0x0200_C464,
    0x0200_C468,
    0x0200_C480,
    0x0200_C4E8,
];

/// Save block number.
pub const BLOCK_ID: u32 = 54;
/// Bytes of the block.
pub const SIZE: usize = 0x1A;
/// Number of list entries the accessors scan.
pub const LEN: usize = 25;
/// A free entry (also the "append" position answer of a full list).
pub const EMPTY: u8 = 25;

/// Save block 54.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    /// The ids, [`EMPTY`] when free.
    pub ids: [u8; SIZE],
}

impl Default for Shortcut {
    fn default() -> Self {
        Shortcut { ids: [EMPTY; SIZE] }
    }
}

impl Shortcut {
    /// Reads the original 0x1A-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        Shortcut {
            ids: b[..SIZE].try_into().unwrap(),
        }
    }

    /// The original 0x1A-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.ids.to_vec()
    }

    /// C: `sub_0200C3F4`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200C3F8`: every byte [`EMPTY`].
    pub fn init(&mut self) {
        *self = Shortcut::default();
    }

    /// C: `FUN_0200C418(list, id, on)`: `on`: add `id` at the first free entry (nothing when it is
    /// already listed; a full list stores it in byte 25); off: free every entry holding `id`.
    /// Both then [`Shortcut::pack`] the list (not when `id` was already listed).
    pub fn set(&mut self, id: u8, on: bool) {
        if on {
            let mut i = 0;
            while i < LEN {
                let e = self.ids[i];
                if e == id {
                    return;
                }
                if e == EMPTY {
                    break;
                }
                i += 1;
            }
            self.ids[i] = id;
        } else {
            for e in self.ids[..LEN].iter_mut().filter(|e| **e == id) {
                *e = EMPTY;
            }
        }
        self.pack();
    }

    /// C: `FUN_0200C44C(list, id)`: `id` is among the 25 entries.
    pub fn contains(&self, id: u8) -> bool {
        self.ids[..LEN].contains(&id)
    }

    /// C: `FUN_0200C464(list, i)`: entry `i`.
    pub fn get(&self, i: usize) -> u8 {
        self.ids[i]
    }

    /// C: `FUN_0200C468`: number of used entries among the 25.
    pub fn count(&self) -> u16 {
        self.ids[..LEN].iter().filter(|&&e| e != EMPTY).count() as u16
    }

    /// C: `FUN_0200c480(list, id, pos)`: move (or add) `id` to position `pos`; `pos` 0xFF appends
    /// it at the first free entry (returns 25 without storing when the list is full). Returns the
    /// position it was stored at. `pos` must be 0..=25 or 0xFF (larger positions write past the
    /// block in the original).
    pub fn insert(&mut self, id: u8, pos: u32) -> u32 {
        let found = self.ids[..LEN].iter().position(|&e| e == id).unwrap_or(LEN) as i32;
        self.set(id, false);
        let mut p = pos;
        if p == 0xFF {
            match self.ids[..LEN].iter().position(|&e| e == EMPTY) {
                Some(k) => p = k as u32,
                None => return LEN as u32,
            }
        } else {
            if found < p as i32 {
                p = (p - 1) & 0xFF;
            }
            if p as i32 <= 24 {
                for k in (p as usize..=24).rev() {
                    if k > 0 {
                        self.ids[k] = self.ids[k - 1];
                    }
                }
            }
        }
        self.ids[p as usize] = id;
        p
    }

    /// C: `FUN_0200C4E8(list)`: one bubble pass that moves free entries towards the end (entry
    /// `i` free: swap with entry `min(i + 1, 24)`).
    pub fn pack(&mut self) {
        for i in 0..LEN {
            let j = (i + 1).clamp(1, 24);
            if self.ids[i] == EMPTY {
                self.ids[i] = self.ids[j];
                self.ids[j] = EMPTY;
            }
        }
    }
}

#[cfg(test)]
mod tests;
