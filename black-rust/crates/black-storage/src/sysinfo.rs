//! The system-info save block 37 (S5b; 0x5C bytes: which DS made the save, and the game clock that
//! guards time-based events against RTC changes), main ARM9 0x02008AEC..0x02008C20. Verified per
//! function with `tools/check-fn.sh black-storage sysinfo` (spec
//! `tools/rom-diff/specs/black-storage/sysinfo.spec`).
//!
//! Layout: +0x00 `u64` the owner's RTC offset, +0x08 the DS MAC address (6 bytes), +0x0E / +0x0F
//! the owner's birthday month / day, +0x10 the game clock ([`GameTime`], 0x38 bytes), +0x48 0x14
//! bytes not read here. The game clock: +0x00 `u32` 1 once started, +0x04 RTC date (16 bytes),
//! +0x14 RTC time (12 bytes), +0x20 `u32` the date as a day number, +0x24 `u64` seconds, +0x2C /
//! +0x30 `u32` (cleared at start), +0x34 `u32` penalty minutes (set to 1440 = 24 h when the clock
//! is changed, counted down by [`GameTime::count_down`]).
//!
//! Not ported: `FUN_02008B08` / `FUN_02008b14` (`FUN_020071CC(sav, 37)` and its +0x10, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r32, w32};
use crate::trcard::RtcHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_8AEC,
    0x0200_8AF0,
    0x0200_8B20,
    0x0200_8B4C,
    0x0200_8B78,
    0x0200_8B94,
    0x0200_8B98,
    0x0200_8B9C,
    0x0200_8BD0,
    0x0200_8BE0,
    0x0200_8C00,
];

/// Save block number.
pub const BLOCK_ID: u32 = 37;
/// Bytes of the block.
pub const SIZE: usize = 0x5C;
/// Bytes of the game clock.
pub const GAME_TIME_SIZE: usize = 0x38;
/// The penalty after a clock change (minutes).
pub const PENALTY: u32 = 0x5A0;

/// The DS firmware / OS queries.
pub trait SysHooks: RtcHooks {
    /// C: `FUN_02087D08` (`OS_GetOwnerRtcOffset`): the owner's RTC offset.
    fn rtc_offset(&mut self) -> u64;
    /// C: `FUN_02087C6C(buf)` (`OS_GetMacAddress`): the 6-byte MAC address.
    fn mac(&mut self, buf: &mut [u8; 6]);
    /// C: `FUN_02087C88(buf)` (`OS_GetOwnerInfo`): the 0x54-byte owner info (byte 2 = birthday
    /// month, byte 3 = day).
    fn owner_info(&mut self, buf: &mut [u8; 0x54]);
    /// C: `FUN_02088F58(&date)` (`RTC_ConvertDateToDay`): the date as a day number.
    fn date_to_days(&mut self, date: &[u8; 16]) -> u32;
}

/// The game clock (+0x10 of the block).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameTime {
    /// +0x00: 1 once started.
    pub started: u32,
    /// +0x04: RTC date.
    pub date: [u8; 16],
    /// +0x14: RTC time.
    pub time: [u8; 12],
    /// +0x20: the date as a day number.
    pub days: u32,
    /// +0x24: seconds.
    pub seconds: u64,
    /// +0x2C.
    pub d2c: u32,
    /// +0x30.
    pub d30: u32,
    /// +0x34: penalty minutes.
    pub penalty: u32,
}

impl GameTime {
    /// Reads the 0x38-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        GameTime {
            started: r32(b, 0),
            date: b[4..0x14].try_into().unwrap(),
            time: b[0x14..0x20].try_into().unwrap(),
            days: r32(b, 0x20),
            seconds: r32(b, 0x24) as u64 | (r32(b, 0x28) as u64) << 32,
            d2c: r32(b, 0x2C),
            d30: r32(b, 0x30),
            penalty: r32(b, 0x34),
        }
    }

    /// The 0x38-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; GAME_TIME_SIZE];
        w32(&mut b, 0, self.started);
        b[4..0x14].copy_from_slice(&self.date);
        b[0x14..0x20].copy_from_slice(&self.time);
        w32(&mut b, 0x20, self.days);
        w32(&mut b, 0x24, self.seconds as u32);
        w32(&mut b, 0x28, (self.seconds >> 32) as u32);
        w32(&mut b, 0x2C, self.d2c);
        w32(&mut b, 0x30, self.d30);
        w32(&mut b, 0x34, self.penalty);
        b
    }

    /// C: `FUN_02008b9c(gt)`: start the clock: started = 1, date / time = now, day number and
    /// seconds of now, +0x2C / +0x30 / penalty = 0.
    pub fn start<H: SysHooks>(&mut self, h: &mut H) {
        self.started = 1;
        h.date_time(&mut self.date, &mut self.time);
        self.days = h.date_to_days(&self.date);
        self.seconds = h.to_seconds(&self.date, &self.time);
        self.d2c = 0;
        self.d30 = 0;
        self.penalty = 0;
    }

    /// C: `FUN_02008BD0(gt)`: a penalty is running.
    pub fn has_penalty(&self) -> bool {
        self.penalty != 0
    }

    /// C: `FUN_02008be0(gt, n)`: clamp the penalty to 1440, then take `n` minutes off (not below
    /// 0).
    pub fn count_down(&mut self, n: u32) {
        if self.penalty > PENALTY {
            self.penalty = PENALTY;
        }
        self.penalty = self.penalty.saturating_sub(n);
    }

    /// C: `FUN_02008C00(gt)`: the clock was changed: penalty = 1440, date / time = now, day
    /// number of now (the seconds stay).
    pub fn clock_changed<H: SysHooks>(&mut self, h: &mut H) {
        self.penalty = PENALTY;
        h.date_time(&mut self.date, &mut self.time);
        self.days = h.date_to_days(&self.date);
    }
}

/// Save block 37.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SysInfo {
    /// +0x00: the owner's RTC offset.
    pub rtc_offset: u64,
    /// +0x08: the MAC address.
    pub mac: [u8; 6],
    /// +0x0E: birthday month.
    pub birth_month: u8,
    /// +0x0F: birthday day.
    pub birth_day: u8,
    /// +0x10: the game clock.
    pub game_time: GameTime,
    /// +0x48: not read here.
    pub tail: [u8; 0x14],
}

impl Default for SysInfo {
    fn default() -> Self {
        SysInfo::from_bytes(&[0; SIZE])
    }
}

impl SysInfo {
    /// Reads the original 0x5C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        SysInfo {
            rtc_offset: r32(b, 0) as u64 | (r32(b, 4) as u64) << 32,
            mac: b[8..14].try_into().unwrap(),
            birth_month: b[14],
            birth_day: b[15],
            game_time: GameTime::from_bytes(&b[0x10..0x48]),
            tail: b[0x48..SIZE].try_into().unwrap(),
        }
    }

    /// The original 0x5C-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        w32(&mut b, 0, self.rtc_offset as u32);
        w32(&mut b, 4, (self.rtc_offset >> 32) as u32);
        b[8..14].copy_from_slice(&self.mac);
        b[14] = self.birth_month;
        b[15] = self.birth_day;
        b[0x10..0x48].copy_from_slice(&self.game_time.to_bytes());
        b[0x48..].copy_from_slice(&self.tail);
        b
    }

    /// C: `sub_02008AEC`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_02008AF0`: zero the block, then start the game clock.
    pub fn init<H: SysHooks>(&mut self, h: &mut H) {
        *self = SysInfo::default();
        self.game_time.start(h);
    }

    /// C: unlabeled routine at `0x02008B20` (`(block)`): record this DS: RTC offset, MAC address,
    /// birthday.
    pub fn record_owner<H: SysHooks>(&mut self, h: &mut H) {
        self.rtc_offset = h.rtc_offset();
        h.mac(&mut self.mac);
        let mut info = [0u8; 0x54];
        h.owner_info(&mut info);
        self.birth_month = info[2];
        self.birth_day = info[3];
    }

    /// C: `FUN_02008B4C(block)`: the stored MAC address is this DS's.
    pub fn same_mac<H: SysHooks>(&self, h: &mut H) -> bool {
        let mut m = [0u8; 6];
        h.mac(&mut m);
        m == self.mac
    }

    /// C: `FUN_02008B78(block)`: the stored RTC offset is the current one.
    pub fn same_rtc_offset<H: SysHooks>(&self, h: &mut H) -> bool {
        h.rtc_offset() == self.rtc_offset
    }

    /// C: `FUN_02008B94`: birthday month.
    pub fn birth_month(&self) -> u8 {
        self.birth_month
    }

    /// C: `FUN_02008B98`: birthday day.
    pub fn birth_day(&self) -> u8 {
        self.birth_day
    }
}

#[cfg(test)]
mod tests;
