//! Save block 52 (S5), the player's money, badges and miscellaneous counters (0xEC bytes; an
//! unnamed file of the main ARM9, 0x0200C054..0x0200C3F4). In Black the money and badges are
//! here, not in [`crate::mystatus`]. Verified per function with
//! `tools/check-fn.sh black-storage misc` (spec `tools/rom-diff/specs/black-storage/misc.spec`).
//!
//! Layout: +0x00 `u32` money (at most 9,999,999), +0x04 `u32` badge bits, +0x08 `u16` and +0x0A a
//! byte (bits 0-5, bit 6, bit 7) set together, +0x0C 8 bytes, +0x14 `u32` (bits 0-27 and 28-31),
//! +0x18 / +0x1C two `u32` counters (at most 99,999), +0x20 and +0x32 two 9-unit names (filled
//! from message bank 0x2F entries 0x151 / 0x159 at new game; UNVERIFIED: the rivals' names),
//! +0x44 / +0x45 (four 2-bit fields) / +0x46 bytes, +0x47 three bytes (0xFF at new game), +0x4A
//! three `u16` counters, +0x50 a `u64`, +0x58 eight records of six `u16`, +0xB8 eight `u32`,
//! +0xDC `u32`, +0xE0 `u16`. Roles other than money and badges are UNVERIFIED.
//!
//! Not ported: `FUN_0200C0E8` / `FUN_0200C0F0` (`FUN_020071CC(sav, 52)`; ownership),
//! `FUN_0200C1F8` / `FUN_0200C210` (a name from a string buffer, `FUN_02045A38`; text glue).

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_C054,
    0x0200_C058,
    0x0200_C0FC,
    0x0200_C104,
    0x0200_C10C,
    0x0200_C11C,
    0x0200_C164,
    0x0200_C184,
    0x0200_C18C,
    0x0200_C1A4,
    0x0200_C1B0,
    0x0200_C1C4,
    0x0200_C1C8,
    0x0200_C1DC,
    0x0200_C1E0,
    0x0200_C1F4,
    0x0200_C20C,
    0x0200_C224,
    0x0200_C22C,
    0x0200_C234,
    0x0200_C248,
    0x0200_C258,
    0x0200_C274,
    0x0200_C278,
    0x0200_C29C,
    0x0200_C2B0,
    0x0200_C2D0,
    0x0200_C2F0,
    0x0200_C308,
    0x0200_C31C,
    0x0200_C324,
    0x0200_C32C,
    0x0200_C334,
    0x0200_C33C,
    0x0200_C344,
    0x0200_C34C,
    0x0200_C358,
    0x0200_C364,
    0x0200_C37C,
    0x0200_C384,
    0x0200_C38C,
    0x0200_C39C,
    0x0200_C3B0,
    0x0200_C3B8,
    0x0200_C3C0,
    0x0200_C3E0,
    0x0200_C3EC,
];

/// Save block number.
pub const BLOCK_ID: u32 = 52;
/// Bytes of the block.
pub const SIZE: usize = 0xEC;
/// Money ceiling.
pub const MONEY_MAX: u32 = 9_999_999;
/// Ceiling of the two counters at +0x18 / +0x1C.
pub const COUNTER_MAX: u32 = 99_999;

/// The message-bank calls of the new-game init (text system, undecompiled).
pub trait MiscTextHooks {
    /// C: `FUN_02045B38(0, 3, 0x2F, 0x8007)`: open message bank 0x2F.
    fn msg_open(&mut self) -> u32;
    /// C: `FUN_02045EC0(msg, id)`: a new string with entry `id`.
    fn msg_string(&mut self, msg: u32, id: u32) -> u32;
    /// C: `FUN_02045A38(s, dst, 9)`: copy the string into a 9-unit name.
    fn str_to_name(&mut self, s: u32, dst: &mut [u16; 9]);
    /// C: `FUN_02045808(s)`: free the string.
    fn str_free(&mut self, s: u32);
    /// C: `FUN_02045C04(msg)`: close the bank.
    fn msg_close(&mut self, msg: u32);
}

/// Save block 52.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Misc {
    /// +0x00.
    pub money: u32,
    /// +0x04: bit `n` = badge `n`.
    pub badges: u32,
    /// +0x08.
    pub h08: u16,
    /// +0x0A: bits 0-5, bit 6, bit 7.
    pub b0a: u8,
    /// +0x0B: unused.
    pub b0b: u8,
    /// +0x0C.
    pub bytes_0c: [u8; 8],
    /// +0x14: bits 0-27 and 28-31.
    pub w14: u32,
    /// +0x18.
    pub counter_18: u32,
    /// +0x1C.
    pub counter_1c: u32,
    /// +0x20.
    pub name_20: [u16; 9],
    /// +0x32.
    pub name_32: [u16; 9],
    /// +0x44.
    pub b44: u8,
    /// +0x45: four 2-bit fields.
    pub b45: u8,
    /// +0x46.
    pub b46: u8,
    /// +0x47.
    pub b47: [u8; 3],
    /// +0x4A.
    pub h4a: [u16; 3],
    /// +0x50.
    pub q50: u64,
    /// +0x58: eight records of six `u16`.
    pub records: [[u16; 6]; 8],
    /// +0xB8.
    pub w_b8: [u32; 8],
    /// +0xD8: unused.
    pub w_d8: u32,
    /// +0xDC.
    pub w_dc: u32,
    /// +0xE0.
    pub h_e0: u16,
    /// +0xE2..+0xEB: unused.
    pub tail: [u8; 10],
}

impl Default for Misc {
    fn default() -> Self {
        Misc::from_bytes(&[0; SIZE])
    }
}

impl Misc {
    /// Reads the original 0xEC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        use crate::le::*;
        let mut bytes_0c = [0; 8];
        bytes_0c.copy_from_slice(&b[0xC..0x14]);
        let mut tail = [0; 10];
        tail.copy_from_slice(&b[0xE2..0xEC]);
        Misc {
            money: r32(b, 0),
            badges: r32(b, 4),
            h08: r16(b, 8),
            b0a: b[0xA],
            b0b: b[0xB],
            bytes_0c,
            w14: r32(b, 0x14),
            counter_18: r32(b, 0x18),
            counter_1c: r32(b, 0x1C),
            name_20: r16s(b, 0x20),
            name_32: r16s(b, 0x32),
            b44: b[0x44],
            b45: b[0x45],
            b46: b[0x46],
            b47: [b[0x47], b[0x48], b[0x49]],
            h4a: r16s(b, 0x4A),
            q50: r32(b, 0x50) as u64 | (r32(b, 0x54) as u64) << 32,
            records: std::array::from_fn(|i| r16s(b, 0x58 + 12 * i)),
            w_b8: r32s(b, 0xB8),
            w_d8: r32(b, 0xD8),
            w_dc: r32(b, 0xDC),
            h_e0: r16(b, 0xE0),
            tail,
        }
    }

    /// The original 0xEC-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        use crate::le::*;
        let mut b = vec![0; SIZE];
        w32(&mut b, 0, self.money);
        w32(&mut b, 4, self.badges);
        w16(&mut b, 8, self.h08);
        b[0xA] = self.b0a;
        b[0xB] = self.b0b;
        b[0xC..0x14].copy_from_slice(&self.bytes_0c);
        w32(&mut b, 0x14, self.w14);
        w32(&mut b, 0x18, self.counter_18);
        w32(&mut b, 0x1C, self.counter_1c);
        w16s(&mut b, 0x20, &self.name_20);
        w16s(&mut b, 0x32, &self.name_32);
        b[0x44] = self.b44;
        b[0x45] = self.b45;
        b[0x46] = self.b46;
        b[0x47..0x4A].copy_from_slice(&self.b47);
        w16s(&mut b, 0x4A, &self.h4a);
        w32(&mut b, 0x50, self.q50 as u32);
        w32(&mut b, 0x54, (self.q50 >> 32) as u32);
        for (i, r) in self.records.iter().enumerate() {
            w16s(&mut b, 0x58 + 12 * i, r);
        }
        w32s(&mut b, 0xB8, &self.w_b8);
        w32(&mut b, 0xD8, self.w_d8);
        w32(&mut b, 0xDC, self.w_dc);
        w16(&mut b, 0xE0, self.h_e0);
        b[0xE2..].copy_from_slice(&self.tail);
        b
    }

    /// C: `sub_0200C054`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200C058`: zero the block, the two default names from message bank 0x2F (entries
    /// 0x151 and 0x159), +0x44 / +0x46 = 0, +0x47..+0x49 = 0xFF.
    pub fn init<H: MiscTextHooks>(&mut self, h: &mut H) {
        *self = Misc::default();
        self.clear_0c();
        self.w14 &= 0xF000_0000;
        let msg = h.msg_open();
        let s = h.msg_string(msg, 0x151);
        h.str_to_name(s, &mut self.name_20);
        h.str_free(s);
        let s = h.msg_string(msg, 0x159);
        h.str_to_name(s, &mut self.name_32);
        h.str_free(s);
        h.msg_close(msg);
        self.b44 = 0;
        self.b46 = 0;
        self.b47 = [0xFF; 3];
    }

    /// C: unlabeled routine at `0x0200C0FC` (`(m, i, v)`): byte `0x0C + i` = `v` (`i` < 8).
    pub fn set_byte_0c(&mut self, i: usize, v: u32) {
        self.bytes_0c[i] = v as u8;
    }

    /// C: unlabeled routine at `0x0200C104` (`(m, i)`).
    pub fn byte_0c(&self, i: usize) -> u32 {
        self.bytes_0c[i] as u32
    }

    /// C: unlabeled routine at `0x0200C10C`: clear the 8 bytes at +0x0C.
    pub fn clear_0c(&mut self) {
        self.bytes_0c = [0; 8];
    }

    /// C: unlabeled routine at `0x0200C11C` (`(m, a, b, c, d)`): +0x08 = `a`, bits 0-5 of +0x0A =
    /// `b`, bit 6 = `c & 1`, bit 7 = `d == 1`.
    pub fn set_08(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.h08 = a as u16;
        self.b0a = self.b0a & !0x3F | (b as u8 & 0x3F);
        self.b0a = self.b0a & !0x40 | (c as u8 & 1) << 6;
        self.b0a = self.b0a & !0x80 | ((d == 1) as u8) << 7;
    }

    /// C: unlabeled routine at `0x0200C164` (`(m, &a, &b, &c, &d)`): the four values of
    /// [`Misc::set_08`].
    pub fn get_08(&self) -> [u32; 4] {
        let x = self.b0a as u32;
        [self.h08 as u32, x & 0x3F, x >> 6 & 1, x >> 7 & 1]
    }

    /// C: `FUN_0200C184`: bits 0-27 of +0x14.
    pub fn w14_low(&self) -> u32 {
        self.w14 & 0x0FFF_FFFF
    }

    /// C: `FUN_0200C18C(m, v)`: bits 0-27 of +0x14 = `v`.
    pub fn set_w14_low(&mut self, v: u32) {
        self.w14 = self.w14 & 0xF000_0000 | v & 0x0FFF_FFFF;
    }

    /// C: `FUN_0200C1A4`: bits 28-31 of +0x14.
    pub fn w14_high(&self) -> u32 {
        self.w14 >> 28
    }

    /// C: `FUN_0200C1B0(m, v)`: bits 28-31 of +0x14 = `v`.
    pub fn set_w14_high(&mut self, v: u32) {
        self.w14 = v << 28 | self.w14 & 0x0FFF_FFFF;
    }

    /// C: `FUN_0200C1C4`.
    pub fn counter_18(&self) -> u32 {
        self.counter_18
    }

    /// C: `FUN_0200C1C8`: += 1 below 99,999; returns the counter.
    pub fn inc_counter_18(&mut self) -> u32 {
        if self.counter_18 < COUNTER_MAX {
            self.counter_18 += 1;
        }
        self.counter_18
    }

    /// C: `FUN_0200C1DC`.
    pub fn counter_1c(&self) -> u32 {
        self.counter_1c
    }

    /// C: `FUN_0200C1E0`: += 1 below 99,999; returns the counter.
    pub fn inc_counter_1c(&mut self) -> u32 {
        if self.counter_1c < COUNTER_MAX {
            self.counter_1c += 1;
        }
        self.counter_1c
    }

    /// C: `FUN_0200C1F4`: the name at +0x20 (the original returns its address).
    pub fn name_20(&self) -> &[u16; 9] {
        &self.name_20
    }

    /// C: `FUN_0200C20C`: the name at +0x32 (the original returns its address).
    pub fn name_32(&self) -> &[u16; 9] {
        &self.name_32
    }

    /// C: `FUN_0200C224`.
    pub fn b44(&self) -> u32 {
        self.b44 as u32
    }

    /// C: `FUN_0200C22C(m, v)`.
    pub fn set_b44(&mut self, v: u32) {
        self.b44 = v as u8;
    }

    /// C: `FUN_0200C234(m, n)`: has badge `n` (register shift: `n` of 32 or more gives false).
    pub fn has_badge(&self, n: u32) -> bool {
        self.badges & 1u32.checked_shl(n & 0xFF).unwrap_or(0) != 0
    }

    /// C: unlabeled routine at `0x0200C248` (`(m, n)`): give badge `n`.
    pub fn give_badge(&mut self, n: u32) {
        self.badges |= 1u32.checked_shl(n & 0xFF).unwrap_or(0);
    }

    /// C: `FUN_0200C258`: number of badges.
    pub fn badge_count(&self) -> u32 {
        self.badges.count_ones()
    }

    /// C: `FUN_0200C274`.
    pub fn money(&self) -> u32 {
        self.money
    }

    /// C: `FUN_0200C278(m, n)`: money += `n`, at most 9,999,999 (an `n` above the ceiling sets
    /// the ceiling); returns the money.
    pub fn add_money(&mut self, n: u32) -> u32 {
        if n > MONEY_MAX {
            self.money = MONEY_MAX;
        } else {
            self.money = self.money.wrapping_add(n);
        }
        if self.money > MONEY_MAX {
            self.money = MONEY_MAX;
        }
        self.money
    }

    /// C: `FUN_0200C29C(m, n)`: money -= `n`, 0 when short; returns the money.
    pub fn sub_money(&mut self, n: u32) -> u32 {
        self.money = self.money.saturating_sub(n);
        self.money
    }

    /// C: unlabeled routine at `0x0200C2B0` (`(m, i, src)`): record `i` = six `u16` from `src`.
    pub fn set_record(&mut self, i: usize, src: &[u16; 6]) {
        self.records[i] = *src;
    }

    /// C: `FUN_0200C2D0(m, i, dst)`: six `u16` of record `i` to `dst`.
    pub fn record(&self, i: usize) -> [u16; 6] {
        self.records[i]
    }

    /// C: unlabeled routine at `0x0200C2F0` (`(m, i, v)`): +0x45 |= `v << 2i` (low byte).
    pub fn or_b45(&mut self, i: u32, v: u32) {
        self.b45 |= v.checked_shl(i.wrapping_mul(2) & 0xFF).unwrap_or(0) as u8;
    }

    /// C: `FUN_0200C308(m, i)`: 2-bit field `i` of +0x45.
    pub fn b45_field(&self, i: u32) -> u32 {
        (self.b45 as u32)
            .checked_shr(i.wrapping_mul(2) & 0xFF)
            .unwrap_or(0)
            & 3
    }

    /// C: unlabeled routine at `0x0200C31C`.
    pub fn b45(&self) -> u32 {
        self.b45 as u32
    }

    /// C: unlabeled routine at `0x0200C324` (`(m, v)`).
    pub fn set_b45(&mut self, v: u32) {
        self.b45 = v as u8;
    }

    /// C: `FUN_0200C32C`.
    pub fn b46(&self) -> u32 {
        self.b46 as u32
    }

    /// C: unlabeled routine at `0x0200C334` (`(m, v)`).
    pub fn set_b46(&mut self, v: u32) {
        self.b46 = v as u8;
    }

    /// C: `FUN_0200C33C(m, i)`: byte `0x47 + i`.
    pub fn b47(&self, i: usize) -> u32 {
        self.b47[i] as u32
    }

    /// C: `FUN_0200C344(m, i, v)`.
    pub fn set_b47(&mut self, i: usize, v: u32) {
        self.b47[i] = v as u8;
    }

    /// C: `FUN_0200C34C(m, i)`: `u16` at `0x4A + 2i`.
    pub fn h4a(&self, i: usize) -> u32 {
        self.h4a[i] as u32
    }

    /// C: `FUN_0200C358(m, i, v)`.
    pub fn set_h4a(&mut self, i: usize, v: u32) {
        self.h4a[i] = v as u16;
    }

    /// C: `FUN_0200C364(m, i, n)`: += `n`, at most 0xFFFF.
    pub fn add_h4a(&mut self, i: usize, n: u32) {
        let v = (self.h4a[i] as u32).wrapping_add(n);
        self.h4a[i] = v.min(0xFFFF) as u16;
    }

    /// C: `FUN_0200C37C`: the `u64` at +0x50.
    pub fn q50(&self) -> u64 {
        self.q50
    }

    /// C: `FUN_0200C384(m, lo, hi)`.
    pub fn set_q50(&mut self, v: u64) {
        self.q50 = v;
    }

    /// C: `FUN_0200C38C(m, i, v)`: word `i` of +0xB8 = `v` for `i < 8` (signed).
    pub fn set_w_b8(&mut self, i: u32, v: u32) {
        if (i as i32) < 8 {
            self.w_b8[i as usize] = v;
        }
    }

    /// C: `FUN_0200C39C(m, i)`: word `i` of +0xB8, 0 for `i >= 8` (signed). `i` must not be negative.
    pub fn w_b8(&self, i: u32) -> u32 {
        if (i as i32) < 8 {
            self.w_b8[i as usize]
        } else {
            0
        }
    }

    /// C: `FUN_0200C3B0`.
    pub fn h_e0(&self) -> u32 {
        self.h_e0 as u32
    }

    /// C: `FUN_0200C3B8(m, v)`.
    pub fn set_h_e0(&mut self, v: u32) {
        self.h_e0 = v as u16;
    }

    /// C: `FUN_0200C3C0(m, n)`: -= `n`, 0 when short.
    pub fn sub_h_e0(&mut self, n: u32) {
        let h = self.h_e0 as u32;
        self.h_e0 = if n > h { 0 } else { (h - n) as u16 };
    }

    /// C: `FUN_0200C3E0`: low byte of the word at +0xDC.
    pub fn w_dc(&self) -> u32 {
        self.w_dc & 0xFF
    }

    /// C: `FUN_0200C3EC(m, v)`: the word at +0xDC = `v`.
    pub fn set_w_dc(&mut self, v: u32) {
        self.w_dc = v;
    }
}

#[cfg(test)]
mod tests;
