//! `playtime.c` (S5): play time and the date/time of the last save stamp, an 8-byte block (main
//! ARM9 0x0200857C..0x02008700). In the save it sits at +0x24 of the basic-data block (block 27,
//! [`crate::save_info`]). Verified per function with `tools/check-fn.sh black-storage playtime`
//! (spec `tools/rom-diff/specs/black-storage/playtime.spec`).
//!
//! Layout: +0 `u16` hours, +2 `u8` minutes, +3 `u8` seconds (the counter stops at 999:59:59),
//! +4 `u32` stamp: bits 0-6 year, 7-10 month, 11-15 day, 16-20 hour, 21-26 minute (from the RTC,
//! `FUN_02008658`), bits 27-31 unused. `FUN_0200857C` (allocation) is replaced by ownership.

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_85A0,
    0x0200_85D4,
    0x0200_863C,
    0x0200_864C,
    0x0200_8650,
    0x0200_8654,
    0x0200_8658,
    0x0200_86C8,
    0x0200_86D0,
    0x0200_86D8,
    0x0200_86E0,
    0x0200_86E8,
    0x0200_86F0,
    0x0200_86F8,
];

/// Bytes of the block.
pub const SIZE: usize = 8;
/// The counter's ceiling: 999 hours (then 59 minutes, 59 seconds).
pub const MAX_HOURS: u32 = 999;

/// The real-time clock (SDK `RTC_GetDate` / `RTC_GetTime`, undecompiled).
pub trait PlayTimeHooks {
    /// C: `FUN_02088864(&date)`: `[year, month, day, weekday]`.
    fn rtc_date(&mut self) -> [u32; 4];
    /// C: `FUN_0208890C(&time)`: `[hour, minute, second]`.
    fn rtc_time(&mut self) -> [u32; 3];
}

/// The play-time block.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayTime {
    /// +0: hours.
    pub hours: u16,
    /// +2: minutes.
    pub minutes: u8,
    /// +3: seconds.
    pub seconds: u8,
    /// +4: the stamp word (see the module doc for its bit fields).
    pub stamp: u32,
}

impl PlayTime {
    /// Reads the original 8-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        PlayTime {
            hours: crate::le::r16(b, 0),
            minutes: b[2],
            seconds: b[3],
            stamp: crate::le::r32(b, 4),
        }
    }

    /// The original 8-byte image.
    pub fn to_bytes(&self) -> [u8; SIZE] {
        let mut b = [0; SIZE];
        crate::le::w16(&mut b, 0, self.hours);
        b[2] = self.minutes;
        b[3] = self.seconds;
        crate::le::w32(&mut b, 4, self.stamp);
        b
    }

    /// A new-game block (`FUN_020085A0` on a zeroed block).
    pub fn new_game() -> Self {
        let mut p = PlayTime::default();
        p.init();
        p
    }

    /// C: `FUN_020085A0`: time 0:00:00, stamp bits 0-26 cleared (27-31 kept).
    pub fn init(&mut self) {
        self.hours = 0;
        self.minutes = 0;
        self.seconds = 0;
        self.stamp &= 0xF800_0000;
    }

    /// C: `FUN_020085D4(pt, secs)`: add `secs` seconds, carrying into minutes and hours; at
    /// 999:59:59 it stops (and an overflow past 999 hours clamps to it).
    pub fn add_seconds(&mut self, secs: u32) {
        let mut h = self.hours as u32;
        if h == MAX_HOURS && self.minutes == 59 && self.seconds == 59 {
            return;
        }
        let mut s = (self.seconds as u32).wrapping_add(secs);
        let mut m = self.minutes as u32;
        if s > 59 {
            m = m.wrapping_add(s / 60);
            s %= 60;
            if m > 59 {
                h = h.wrapping_add(m / 60);
                m %= 60;
                if h >= MAX_HOURS {
                    h = MAX_HOURS;
                    m = 59;
                    s = 59;
                }
            }
        }
        self.hours = h as u16;
        self.minutes = m as u8;
        self.seconds = s as u8;
    }

    /// C: `FUN_0200863C(dst, src)`: copy the block (`MI_CpuCopy8(src, dst, 8)`).
    pub fn copy(dst: &mut PlayTime, src: &PlayTime) {
        dst.clone_from(src);
    }

    /// C: `FUN_0200864C`.
    pub fn hours(&self) -> u32 {
        self.hours as u32
    }

    /// C: `FUN_02008650`.
    pub fn minutes(&self) -> u32 {
        self.minutes as u32
    }

    /// C: `FUN_02008654`.
    pub fn seconds(&self) -> u32 {
        self.seconds as u32
    }

    /// C: `FUN_02008658`: stamp the current RTC date and time (year, month, day, hour, minute).
    pub fn stamp_now<H: PlayTimeHooks>(&mut self, h: &mut H) {
        let d = h.rtc_date();
        let t = h.rtc_time();
        let mut w = self.stamp & !0x7F | d[0] & 0x7F;
        w = w & 0xFFFF_F87F | (d[1] & 0xF) << 7;
        w = w & 0xFFFF_07FF | (d[2] & 0x1F) << 11;
        w = w & 0xFFE0_FFFF | (t[0] & 0x1F) << 16;
        w = w & 0xF81F_FFFF | (t[1] & 0x3F) << 21;
        self.stamp = w;
    }

    /// C: `FUN_020086C8`: stamp year (bits 0-6).
    pub fn stamp_year(&self) -> u32 {
        self.stamp & 0x7F
    }

    /// C: `FUN_020086D0`: stamp month (bits 7-10).
    pub fn stamp_month(&self) -> u32 {
        self.stamp >> 7 & 0xF
    }

    /// C: `FUN_020086D8`: stamp day (bits 11-15).
    pub fn stamp_day(&self) -> u32 {
        self.stamp >> 11 & 0x1F
    }

    /// C: `FUN_020086E0`: stamp hour (bits 16-20).
    pub fn stamp_hour(&self) -> u32 {
        self.stamp >> 16 & 0x1F
    }

    /// C: `FUN_020086E8`: stamp minute (bits 21-26).
    pub fn stamp_minute(&self) -> u32 {
        self.stamp >> 21 & 0x3F
    }

    /// C: `FUN_020086F0(pt, &out)`: the raw stamp word.
    pub fn stamp_word(&self) -> u32 {
        self.stamp
    }

    /// C: `FUN_020086F8(pt, &src)`: set the raw stamp word.
    pub fn set_stamp_word(&mut self, w: u32) {
        self.stamp = w;
    }
}

#[cfg(test)]
mod tests;
