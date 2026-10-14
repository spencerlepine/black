//! The trainer card save block 33 (S5b; 0x658 bytes), main ARM9 0x020089A8..0x02008AEC (callers:
//! overlay 10 `trcard_sys.c`, `btl_setup.c`). Verified per function with
//! `tools/check-fn.sh black-storage trcard` (spec `tools/rom-diff/specs/black-storage/trcard.spec`).
//!
//! Layout: +0x000 the signature (0x600 bytes, "signed" when any word is nonzero), +0x600 `u16`,
//! +0x602 `u8` (only ever set to 1), +0x603 `u8`, +0x604 eight 4-byte entries (three bytes
//! written together with a time stamp, UNVERIFIED: badge records), +0x624 `u64` seconds (the RTC
//! date and time of the last entry write), +0x62C 12 bytes not read here, +0x638 sixteen `u16`
//! (0xF0 written with an entry; UNVERIFIED: badge shine).
//!
//! Not ported: `FUN_020089C0` (`FUN_020071CC(sav, 33)`, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r16s, r32, w16s, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_89A8,
    0x0200_89B0,
    0x0200_89CC,
    0x0200_89D0,
    0x0200_89F4,
    0x0200_8A00,
    0x0200_8A0C,
    0x0200_8A14,
    0x0200_8A1C,
    0x0200_8A28,
    0x0200_8A34,
    0x0200_8A44,
    0x0200_8A54,
    0x0200_8A7C,
    0x0200_8ACC,
    0x0200_8ADC,
];

/// Save block number.
pub const BLOCK_ID: u32 = 33;
/// Bytes of the block.
pub const SIZE: usize = 0x658;
/// Bytes of the signature.
pub const SIGN_SIZE: usize = 0x600;
/// Number of 4-byte entries at +0x604.
pub const ENTRIES: usize = 8;
/// Value [`TrCard::set_entry`] writes to the entry's halfword.
pub const ENTRY_HALF: u16 = 0xF0;

/// The real-time clock (`gf_rtc` / NitroSDK RTC).
pub trait RtcHooks {
    /// C: `FUN_0203F49C(&date, &time)`: the current RTC date (16 bytes) and time (12 bytes).
    fn date_time(&mut self, date: &mut [u8; 16], time: &mut [u8; 12]);
    /// C: `FUN_02089004(&date, &time)` (`RTC_ConvertDateTimeToSecond`): seconds since 2000.
    fn to_seconds(&mut self, date: &[u8; 16], time: &[u8; 12]) -> u64;
}

/// Save block 33.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrCard {
    /// +0x000: the signature bitmap.
    #[serde(with = "crate::save::hex")]
    pub sign: Vec<u8>,
    /// +0x600.
    pub w600: u16,
    /// +0x602.
    pub b602: u8,
    /// +0x603.
    pub b603: u8,
    /// +0x604: the entries.
    pub entries: [[u8; 4]; ENTRIES],
    /// +0x624: the time stamp (seconds).
    pub time: u64,
    /// +0x62C: not read here.
    pub pad62c: [u8; 12],
    /// +0x638.
    pub halves: [u16; 16],
}

impl Default for TrCard {
    fn default() -> Self {
        TrCard::from_bytes(&[0; SIZE])
    }
}

impl TrCard {
    /// Reads the original 0x658-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        TrCard {
            sign: b[..SIGN_SIZE].to_vec(),
            w600: u16::from_le_bytes([b[0x600], b[0x601]]),
            b602: b[0x602],
            b603: b[0x603],
            entries: std::array::from_fn(|i| b[0x604 + 4 * i..0x608 + 4 * i].try_into().unwrap()),
            time: r32(b, 0x624) as u64 | (r32(b, 0x628) as u64) << 32,
            pad62c: b[0x62C..0x638].try_into().unwrap(),
            halves: r16s(b, 0x638),
        }
    }

    /// The original 0x658-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        b[..SIGN_SIZE].copy_from_slice(&self.sign);
        b[0x600..0x602].copy_from_slice(&self.w600.to_le_bytes());
        b[0x602] = self.b602;
        b[0x603] = self.b603;
        for (i, e) in self.entries.iter().enumerate() {
            b[0x604 + 4 * i..0x608 + 4 * i].copy_from_slice(e);
        }
        w32(&mut b, 0x624, self.time as u32);
        w32(&mut b, 0x628, (self.time >> 32) as u32);
        b[0x62C..0x638].copy_from_slice(&self.pad62c);
        w16s(&mut b, 0x638, &self.halves);
        b
    }

    /// C: `sub_020089A8`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_020089B0`: zero the block.
    pub fn init(&mut self) {
        *self = TrCard::default();
    }

    /// C: `FUN_020089CC`: the signature (the original returns the block address).
    pub fn sign(&self) -> &[u8] {
        &self.sign
    }

    /// C: unlabeled routine at `0x020089D0`: the signature is not blank (the sum of its 0x180
    /// words is nonzero).
    pub fn is_signed(&self) -> bool {
        self.sign.chunks(4).fold(0u32, |a, w| {
            a.wrapping_add(u32::from_le_bytes(w.try_into().unwrap()))
        }) != 0
    }

    /// C: `FUN_020089F4`: +0x602.
    pub fn b602(&self) -> u8 {
        self.b602
    }

    /// C: `FUN_02008a00`: +0x602 = 1.
    pub fn set_b602(&mut self) {
        self.b602 = 1;
    }

    /// C: `FUN_02008A0C`: +0x600.
    pub fn w600(&self) -> u16 {
        self.w600
    }

    /// C: unlabeled routine at `0x02008A14`: +0x600 = `v`.
    pub fn set_w600(&mut self, v: u16) {
        self.w600 = v;
    }

    /// C: unlabeled routine at `0x02008A1C`: +0x603 = `v`.
    pub fn set_b603(&mut self, v: u8) {
        self.b603 = v;
    }

    /// C: `FUN_02008A28`: +0x603.
    pub fn b603(&self) -> u8 {
        self.b603
    }

    /// C: `FUN_02008A34(block, lo, hi)`: the time stamp.
    pub fn set_time(&mut self, t: u64) {
        self.time = t;
    }

    /// C: unlabeled routine at `0x02008A44`: the time stamp.
    pub fn time(&self) -> u64 {
        self.time
    }

    /// C: unlabeled routine at `0x02008A54` (`(block, i)`): entry `i` as a little-endian word.
    pub fn entry(&self, i: usize) -> u32 {
        u32::from_le_bytes(self.entries[i])
    }

    /// C: `FUN_02008a7c(block, i, a, b, c)`: entry `i` bytes 0..3 = `a`, `b`, `c`, halfword `i` =
    /// 0xF0, time stamp = now.
    pub fn set_entry<H: RtcHooks>(&mut self, i: usize, a: u8, b: u8, c: u8, h: &mut H) {
        let mut date = [0u8; 16];
        let mut time = [0u8; 12];
        h.date_time(&mut date, &mut time);
        let t = h.to_seconds(&date, &time);
        self.entries[i][0] = a;
        self.entries[i][1] = b;
        self.entries[i][2] = c;
        self.halves[i] = ENTRY_HALF;
        self.set_time(t);
    }

    /// C: unlabeled routine at `0x02008ACC` (`(block, i)`): halfword `i`.
    pub fn half(&self, i: usize) -> u16 {
        self.halves[i]
    }

    /// C: unlabeled routine at `0x02008ADC` (`(block, i, v)`): halfword `i` = `v`.
    pub fn set_half(&mut self, i: usize, v: u16) {
        self.halves[i] = v;
    }
}

#[cfg(test)]
mod tests;
