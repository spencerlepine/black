//! The Entralink save block 53 (S5b; 0x1B0 bytes), main ARM9 0x0200BDBC..0x0200C054 (callers:
//! overlays 11 `intrude_comm.c`, 18 `intrude_field.c`, 33 `dowsing.c`, 71 `monolith_tool.c`).
//! Verified per function with `tools/check-fn.sh black-storage intrude_save` (spec
//! `tools/rom-diff/specs/black-storage/intrude_save.spec`).
//!
//! Layout: +0x000 a 0x10-byte record the overlay-11 routine `FUN_021763BC` fills (UNVERIFIED: Pass
//! Power levels; +0x0C / +0x0E `u16` point counters capped at 999), +0x010 twenty 0x14-byte entries
//! (name: eight `u16` with an EOS, `u16` +0x10 nonzero = used, `u8` +0x12 kind below 18, `u8` +0x13),
//! +0x1A0 `u8` (0x30 at init), +0x1A1 two bytes of flags (ids 33..=38), +0x1A3 pad, +0x1A4 `u32`
//! counter capped at 99999, +0x1A8 `u64` seconds of play spent (see [`IntrudeSave::play_seconds`]).
//!
//! Not ported: `FUN_0200BE48` (`FUN_020071CC(sav, 53)`, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r16, r16s, r32, w16, w16s, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_BDBC,
    0x0200_BDC4,
    0x0200_BDE4,
    0x0200_BDF8,
    0x0200_BDFC,
    0x0200_BE00,
    0x0200_BE24,
    0x0200_BE54,
    0x0200_BE6C,
    0x0200_BE84,
    0x0200_BF08,
    0x0200_BF10,
    0x0200_BF18,
    0x0200_BF54,
    0x0200_BF70,
    0x0200_BF78,
    0x0200_BF94,
    0x0200_BFA4,
];

/// Save block number.
pub const BLOCK_ID: u32 = 53;
/// Bytes of the block.
pub const SIZE: usize = 0x1B0;
/// Number of list entries.
pub const ENTRIES: usize = 20;
/// The string terminator `FUN_02045A98` returns (`black_text::strbuf::EOS`).
pub const EOS: u16 = 0xFFFF;
/// Cap of the point counters.
pub const POINTS_MAX: i32 = 999;
/// Cap of the +0x1A4 counter.
pub const COUNTER_MAX: u32 = 99_999;
/// +0x1A0 after init.
pub const INIT_1A0: u8 = 0x30;

/// The callees outside the block code.
pub trait IntrudeHooks {
    /// C: `FUN_021763BC(rec)` (overlay 11): fill the 0x10-byte record at +0 (after it was zeroed).
    fn power_init(&mut self, rec: &mut [u8; 16]);
    /// C: `FUN_0200864C` / `FUN_02008650` / `FUN_02008654(playtime)`: hours, minutes, seconds of the
    /// play time (`black_storage::playtime`).
    fn play_time(&mut self, pt: u32) -> (u32, u32, u32);
}

/// Save block 53.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntrudeSave {
    /// +0x000: the record `FUN_021763BC` fills (bytes 0..12).
    pub power: [u8; 12],
    /// +0x00C: points.
    pub points_c: u16,
    /// +0x00E: points.
    pub points_e: u16,
    /// +0x010: the entries (ten `u16` each).
    pub entries: Vec<[u16; 10]>,
    /// +0x1A0.
    pub b1a0: u8,
    /// +0x1A1: flags of ids 33..=38.
    pub flags: [u8; 2],
    /// +0x1A3: pad.
    pub pad: u8,
    /// +0x1A4: counter.
    pub counter: u32,
    /// +0x1A8: seconds.
    pub seconds: u64,
}

impl Default for IntrudeSave {
    fn default() -> Self {
        IntrudeSave::from_bytes(&[0; SIZE])
    }
}

impl IntrudeSave {
    /// Reads the original 0x1B0-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        IntrudeSave {
            power: b[..12].try_into().unwrap(),
            points_c: r16(b, 0x0C),
            points_e: r16(b, 0x0E),
            entries: (0..ENTRIES).map(|i| r16s(b, 0x10 + 0x14 * i)).collect(),
            b1a0: b[0x1A0],
            flags: [b[0x1A1], b[0x1A2]],
            pad: b[0x1A3],
            counter: r32(b, 0x1A4),
            seconds: r32(b, 0x1A8) as u64 | (r32(b, 0x1AC) as u64) << 32,
        }
    }

    /// The original 0x1B0-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        self.write_power(&mut b[..16]);
        for (i, e) in self.entries.iter().enumerate() {
            w16s(&mut b, 0x10 + 0x14 * i, e);
        }
        b[0x1A0] = self.b1a0;
        b[0x1A1] = self.flags[0];
        b[0x1A2] = self.flags[1];
        b[0x1A3] = self.pad;
        w32(&mut b, 0x1A4, self.counter);
        w32(&mut b, 0x1A8, self.seconds as u32);
        w32(&mut b, 0x1AC, (self.seconds >> 32) as u32);
        b
    }

    fn write_power(&self, b: &mut [u8]) {
        b[..12].copy_from_slice(&self.power);
        w16(b, 0x0C, self.points_c);
        w16(b, 0x0E, self.points_e);
    }

    /// The 0x10-byte record at +0.
    pub fn power_record(&self) -> [u8; 16] {
        let mut b = [0u8; 16];
        self.write_power(&mut b);
        b
    }

    fn set_power_record(&mut self, b: &[u8]) {
        self.power = b[..12].try_into().unwrap();
        self.points_c = r16(b, 0x0C);
        self.points_e = r16(b, 0x0E);
    }

    /// C: `sub_0200BDBC`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200BDC4`: zero, [`IntrudeSave::reset_power`], +0x1A0 = 0x30.
    pub fn init<H: IntrudeHooks>(&mut self, h: &mut H) {
        *self = IntrudeSave::default();
        self.reset_power(h);
        self.b1a0 = INIT_1A0;
    }

    /// C: `FUN_0200BDE4(block)`: zero the record at +0, then `FUN_021763BC` on it.
    pub fn reset_power<H: IntrudeHooks>(&mut self, h: &mut H) {
        let mut r = [0u8; 16];
        h.power_init(&mut r);
        self.set_power_record(&r);
    }

    /// C: unlabeled routine at `0x0200BDF8`: +0x0C.
    pub fn points_c(&self) -> u16 {
        self.points_c
    }

    /// C: `FUN_0200BDFC`: +0x0E.
    pub fn points_e(&self) -> u16 {
        self.points_e
    }

    fn add_capped(v: u16, n: u32) -> u16 {
        if (v as i32).wrapping_add(n as i32) > POINTS_MAX {
            POINTS_MAX as u16
        } else {
            v.wrapping_add(n as u16)
        }
    }

    /// C: `FUN_0200BE00(block, n)`: +0x0C += `n`, 999 when the (signed) sum is larger.
    pub fn add_points_c(&mut self, n: u32) {
        self.points_c = Self::add_capped(self.points_c, n);
    }

    /// C: `FUN_0200BE24(block, n)`: +0x0E += `n`, 999 when the (signed) sum is larger.
    pub fn add_points_e(&mut self, n: u32) {
        self.points_e = Self::add_capped(self.points_e, n);
    }

    /// C: `FUN_0200BE54(sav, src)`: the record at +0 = `src` (eight halfwords).
    pub fn load_power(&mut self, src: &[u8]) {
        self.set_power_record(&src[..16]);
    }

    /// C: `FUN_0200BE6C(sav, dst)`: copy the record at +0 out (eight halfwords).
    pub fn store_power(&self, dst: &mut [u8]) {
        dst[..16].copy_from_slice(&self.power_record());
    }

    /// C: unlabeled routine at `0x0200BE84` (`(block, rec)`): add a 0x14-byte entry: ignored
    /// unless its name has an EOS among its eight halfwords and its byte +0x12 is below 18; stored
    /// in the first unused entry (+0x10 halfword 0), or, with all 20 used, the list moves up one
    /// and it becomes the last.
    pub fn add_entry(&mut self, rec: &[u16; 10]) {
        if !rec[..8].contains(&EOS) || (rec[9] & 0xFF) >= 18 {
            return;
        }
        if let Some(e) = self.entries.iter_mut().find(|e| e[8] == 0) {
            *e = *rec;
            return;
        }
        self.entries.remove(0);
        self.entries.push(*rec);
    }

    /// C: unlabeled routine at `0x0200BF08`: +0x1A0 = `v`.
    pub fn set_b1a0(&mut self, v: u8) {
        self.b1a0 = v;
    }

    /// C: `FUN_0200BF10`: +0x1A0.
    pub fn b1a0(&self) -> u8 {
        self.b1a0
    }

    /// C: `FUN_0200bf18(block, id)`: set the flag of `id` (33..=38; others ignored, except ids from
    /// 0x80000021 up, which pass both signed checks and write far outside the block in the
    /// original: they panic here).
    pub fn set_flag(&mut self, id: u32) {
        let k = (id as i32).wrapping_sub(33);
        if k < 0 || (id as i32) > 38 {
            return;
        }
        self.flags[(k >> 3) as usize] |= 1 << (k & 7);
    }

    /// C: `FUN_0200BF54(block, out)`: the two flag bytes.
    pub fn flags(&self) -> [u8; 2] {
        self.flags
    }

    /// C: `FUN_0200BF70`: the +0x1A4 counter.
    pub fn counter(&self) -> u32 {
        self.counter
    }

    /// C: unlabeled routine at `0x0200BF78`: increment the counter below 99999; returns it.
    pub fn inc_counter(&mut self) -> u32 {
        if self.counter < COUNTER_MAX {
            self.counter += 1;
        }
        self.counter
    }

    /// C: `FUN_0200BF94`: the +0x1A8 seconds.
    pub fn seconds(&self) -> u64 {
        self.seconds
    }

    /// C: `FUN_0200BFA4(block, playtime, mode)`: `now` = the play time in seconds; mode 0 starts a
    /// session (`*start = now`), mode 1 ends it (+0x1A8 += now - start), mode 2 returns the total
    /// with the running session (+0x1A8 + now - start, not stored); every other mode returns
    /// +0x1A8. `start` is the RAM word 0x02146170 (not saved).
    pub fn play_seconds<H: IntrudeHooks>(
        &mut self,
        pt: u32,
        mode: u32,
        start: &mut u64,
        h: &mut H,
    ) -> u64 {
        let (hh, mm, ss) = h.play_time(pt);
        let now = (mm as u64 * 60)
            .wrapping_add(hh as u64 * 3600)
            .wrapping_add(ss as u64);
        match mode {
            0 => *start = now,
            1 => self.seconds = self.seconds.wrapping_add(now.wrapping_sub(*start)),
            2 => return self.seconds.wrapping_add(now.wrapping_sub(*start)),
            _ => {}
        }
        self.seconds
    }
}

#[cfg(test)]
mod tests;
