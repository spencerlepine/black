//! The DS-identity logs (S5b): save block 65 (0x5C bytes; overlay 21 `trial_house.c` reads it) and
//! save block 60 (0x9C bytes; the Wi-Fi / live battle-match overlays 176 / 186 / 199 read it, D9),
//! main ARM9 0x0200E8E8..0x0200EA08. Both start with the same 0x5A-byte [`MacLog`]; block 60 adds
//! the Battle Subway record counters S5 ported ([`BsubwayCounters`]). Verified per function with
//! `tools/check-fn.sh black-storage mac_log` (spec `tools/rom-diff/specs/black-storage/mac_log.spec`).
//!
//! [`MacLog`]: +0x00 this DS's MAC address, +0x06 four `u8` counters (counter 0 = entries in use,
//! saturating 0..=255), +0x0A ten 8-byte entries (a MAC address and two bytes), ring-indexed `i % 10`.
//!
//! Not ported: `FUN_0200E8F8` / `0x0200E9F4` (`FUN_020071CC(sav, 65 / 60)`, ownership).

use serde::{Deserialize, Serialize};

use crate::bsubway::BsubwayCounters;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_E8E8,
    0x0200_E8EC,
    0x0200_E904,
    0x0200_E918,
    0x0200_E920,
    0x0200_E938,
    0x0200_E958,
    0x0200_E978,
    0x0200_E994,
    0x0200_E9DC,
    0x0200_E9E0,
    0x0200_EA00,
    0x0200_EA04,
];

/// Save block number of [`TrialHouse`].
pub const BLOCK_ID: u32 = 65;
/// Bytes of [`TrialHouse`].
pub const SIZE: usize = 0x5C;
/// Save block number of [`MatchLog`].
pub const MATCH_BLOCK_ID: u32 = 60;
/// Bytes of [`MatchLog`].
pub const MATCH_SIZE: usize = 0x9C;
/// Bytes of [`MacLog`].
pub const LOG_SIZE: usize = 0x5A;
/// Entries of a [`MacLog`].
pub const ENTRIES: usize = 10;

/// The DS's own MAC address and the compare routine.
pub trait MacHooks {
    /// C: `FUN_02087C6C(buf)` (`OS_GetMacAddress`).
    fn mac(&mut self, buf: &mut [u8; 6]);
    /// C: `FUN_0203EFC8(a, b, 6)` (`memcmp`-like): 0 when equal.
    fn compare(&mut self, a: &[u8; 6], b: &[u8; 6]) -> u32;
}

/// The 0x5A-byte log at the start of blocks 60 and 65.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MacLog {
    /// +0x00: this DS's MAC address.
    pub mac: [u8; 6],
    /// +0x06: counters (0 = entries in use).
    pub counts: [u8; 4],
    /// +0x0A: the entries.
    pub entries: [[u8; 8]; ENTRIES],
}

impl MacLog {
    /// Reads the 0x5A-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        MacLog {
            mac: b[..6].try_into().unwrap(),
            counts: b[6..10].try_into().unwrap(),
            entries: std::array::from_fn(|i| b[10 + 8 * i..18 + 8 * i].try_into().unwrap()),
        }
    }

    /// The 0x5A-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; LOG_SIZE];
        b[..6].copy_from_slice(&self.mac);
        b[6..10].copy_from_slice(&self.counts);
        for (i, e) in self.entries.iter().enumerate() {
            b[10 + 8 * i..18 + 8 * i].copy_from_slice(e);
        }
        b
    }

    /// C: `FUN_0200e904(log)`: zero, then this DS's MAC address.
    pub fn init<H: MacHooks>(&mut self, h: &mut H) {
        *self = MacLog::default();
        h.mac(&mut self.mac);
    }

    /// C: `FUN_0200E918(log, i)`: counter `i`.
    pub fn count(&self, i: usize) -> u8 {
        self.counts[i]
    }

    /// C: `FUN_0200e920(log, i, n)`: counter `i` += `n` (signed), clamped to 0..=255.
    pub fn add_count(&mut self, i: usize, n: u32) {
        let v = (self.counts[i] as i32).wrapping_add(n as i32);
        self.counts[i] = v.clamp(0, 255) as u8;
    }

    /// C: `FUN_0200E938(log, i, dst)`: the MAC address of entry `i % 10`.
    pub fn entry_mac(&self, i: u32) -> [u8; 6] {
        self.entries[(i % ENTRIES as u32) as usize][..6]
            .try_into()
            .unwrap()
    }

    /// C: `FUN_0200E958(log, i, src)`: the MAC address of entry `i % 10` = `src`.
    pub fn set_entry_mac(&mut self, i: u32, src: &[u8]) {
        self.entries[(i % ENTRIES as u32) as usize][..6].copy_from_slice(&src[..6]);
    }

    /// C: `FUN_0200E978(log, i, j, v)`: byte `j` (0 or 1) after the MAC address of entry `i % 10`.
    pub fn set_entry_byte(&mut self, i: u32, j: usize, v: u8) {
        self.entries[(i % ENTRIES as u32) as usize][6 + j] = v;
    }

    /// C: `FUN_0200E994(log, mac)`: `mac` is among the first `min(counter 0, 10)` entries.
    pub fn contains<H: MacHooks>(&self, mac: &[u8; 6], h: &mut H) -> bool {
        let n = (self.count(0) as usize).min(ENTRIES);
        (0..n).any(|i| h.compare(mac, &self.entry_mac(i as u32)) == 0)
    }
}

/// Save block 65.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialHouse {
    /// +0x00: the log.
    pub log: MacLog,
    /// +0x5A: not read here.
    pub pad: [u8; 2],
}

impl TrialHouse {
    /// Reads the original 0x5C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        TrialHouse {
            log: MacLog::from_bytes(b),
            pad: [b[0x5A], b[0x5B]],
        }
    }

    /// The original 0x5C-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = self.log.to_bytes();
        b.extend_from_slice(&self.pad);
        b
    }

    /// C: `sub_0200E8E8`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200E8EC`: zero the block.
    pub fn init(&mut self) {
        *self = TrialHouse::default();
    }
}

/// Save block 60.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchLog {
    /// +0x00: the log.
    pub log: MacLog,
    /// +0x5A: the Battle Subway record counters.
    pub counters: BsubwayCounters,
}

impl MatchLog {
    /// Reads the original 0x9C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        MatchLog {
            log: MacLog::from_bytes(b),
            counters: BsubwayCounters::from_bytes(&b[LOG_SIZE..MATCH_SIZE]),
        }
    }

    /// The original 0x9C-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = self.log.to_bytes();
        b.extend_from_slice(&self.counters.to_bytes());
        b
    }

    /// C: `sub_0200E9DC`: block size.
    pub fn size() -> u32 {
        MATCH_SIZE as u32
    }

    /// C: `sub_0200E9E0`: [`MacLog::init`], then clear the counters (`0x0200DFCC`).
    pub fn init<H: MacHooks>(&mut self, h: &mut H) {
        self.log.init(h);
        self.counters.clear();
    }

    /// C: unlabeled routines at `0x0200EA00` and `0x0200EA04`: the counters (the original returns
    /// block + 0x5A).
    pub fn counters(&mut self) -> &mut BsubwayCounters {
        &mut self.counters
    }
}

#[cfg(test)]
mod tests;
