//! `bsubway_savedata.c` (S5): the Battle Subway play data (save block 57, 0x3C bytes), score
//! data (block 58, 0x1AC bytes) and the 11 x 3 record counters kept at +0x5A of block 60, main
//! ARM9 0x0200D858..0x0200E024. Verified per function with
//! `tools/check-fn.sh black-storage bsubway` (spec `tools/rom-diff/specs/black-storage/bsubway.spec`).
//!
//! Not ported: block 59 (0xB90 bytes, `sub_0200DD10`..`FUN_0200DF58`: the Wi-Fi rooms and leader
//! data downloaded for the online Battle Subway; D9, out of scope), `FUN_0200DF94` /
//! `FUN_0200DFB4` (helpers over those downloaded records).
//!
//! Field roles are named after the accessors; their game meaning is UNVERIFIED unless stated.

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_D858,
    0x0200_D85C,
    0x0200_D868,
    0x0200_D87C,
    0x0200_D884,
    0x0200_D914,
    0x0200_D9E8,
    0x0200_DA14,
    0x0200_DA28,
    0x0200_DA54,
    0x0200_DA64,
    0x0200_DA6C,
    0x0200_DA80,
    0x0200_DA98,
    0x0200_DABC,
    0x0200_DAC0,
    0x0200_DAC4,
    0x0200_DAD8,
    0x0200_DAEC,
    0x0200_DB08,
    0x0200_DB1C,
    0x0200_DB30,
    0x0200_DB44,
    0x0200_DB60,
    0x0200_DB80,
    0x0200_DBA0,
    0x0200_DBF0,
    0x0200_DBFC,
    0x0200_DC08,
    0x0200_DC0C,
    0x0200_DC50,
    0x0200_DC74,
    0x0200_DC94,
    0x0200_DCF8,
    0x0200_DCFC,
    0x0200_DFCC,
    0x0200_DFD8,
    0x0200_DFE4,
    0x0200_E008,
];

/// Save block number of the play data.
pub const PLAY_BLOCK_ID: u32 = 57;
/// Save block number of the score data.
pub const SCORE_BLOCK_ID: u32 = 58;
/// Bytes of the play data.
pub const PLAY_SIZE: usize = 0x3C;
/// Bytes of the score data.
pub const SCORE_SIZE: usize = 0x1AC;
/// Bytes of the record counters.
pub const COUNTERS_SIZE: usize = 0x42;
/// Number of per-mode entries in the score arrays.
pub const MODES: usize = 9;
/// Battle Points ceiling.
pub const BP_MAX: u32 = 9999;

/// Block 57: the play data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BsubwayPlay {
    /// +0x00: bit 0 (id 7), bit 1 (`FUN_0200D868`), bits 2-5 (id 0), bits 6-8 (id 9), bit 9 (id
    /// 10), bit 10 (id 11), bits 11-15 unused.
    pub bits: u16,
    /// +0x02 (id 1).
    pub b2: u8,
    /// +0x03 (id 2).
    pub b3: u8,
    /// +0x04 (id 3).
    pub h4: u16,
    /// +0x06 (id 4).
    pub h6: u16,
    /// +0x08 (id 5).
    pub data_08: [u8; 4],
    /// +0x0C (id 8).
    pub data_0c: [u8; 28],
    /// +0x28 (id 6).
    pub data_28: [u8; 20],
}

impl Default for BsubwayPlay {
    fn default() -> Self {
        BsubwayPlay::from_bytes(&[0; PLAY_SIZE])
    }
}

impl BsubwayPlay {
    /// Reads the original 0x3C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut data_08 = [0; 4];
        data_08.copy_from_slice(&b[8..0xC]);
        let mut data_0c = [0; 28];
        data_0c.copy_from_slice(&b[0xC..0x28]);
        let mut data_28 = [0; 20];
        data_28.copy_from_slice(&b[0x28..0x3C]);
        BsubwayPlay {
            bits: crate::le::r16(b, 0),
            b2: b[2],
            b3: b[3],
            h4: crate::le::r16(b, 4),
            h6: crate::le::r16(b, 6),
            data_08,
            data_0c,
            data_28,
        }
    }

    /// The original 0x3C-byte image.
    pub fn to_bytes(&self) -> [u8; PLAY_SIZE] {
        let mut b = [0; PLAY_SIZE];
        crate::le::w16(&mut b, 0, self.bits);
        b[2] = self.b2;
        b[3] = self.b3;
        crate::le::w16(&mut b, 4, self.h4);
        crate::le::w16(&mut b, 6, self.h6);
        b[8..0xC].copy_from_slice(&self.data_08);
        b[0xC..0x28].copy_from_slice(&self.data_0c);
        b[0x28..].copy_from_slice(&self.data_28);
        b
    }

    /// C: `sub_0200D858`: block size.
    pub fn size() -> u32 {
        PLAY_SIZE as u32
    }

    /// C: `FUN_0200D85C`: zero the block (the new-game state).
    pub fn init(&mut self) {
        *self = BsubwayPlay::default();
    }

    /// C: `FUN_0200D868(play, v)`: bit 1 = `v & 1`.
    pub fn set_bit1(&mut self, v: u32) {
        self.bits = self.bits & !2 | (v as u16 & 1) << 1;
    }

    /// C: `FUN_0200D87C`: bit 1.
    pub fn bit1(&self) -> u32 {
        self.bits as u32 >> 1 & 1
    }

    /// C: `FUN_0200D884(play, id, out)`: field `id` (0..11): scalar ids return the value, the
    /// byte-array ids 5 (4 bytes), 6 (20) and 8 (28) copy into `out` and return 0; ids > 11
    /// return 0. `out` must hold the copied bytes.
    pub fn get(&self, id: u32, out: &mut [u8]) -> u32 {
        let w = self.bits as u32;
        match id {
            0 => w >> 2 & 0xF,
            1 => self.b2 as u32,
            2 => self.b3 as u32,
            3 => self.h4 as u32,
            4 => self.h6 as u32,
            5 => {
                out[..4].copy_from_slice(&self.data_08);
                0
            }
            6 => {
                out[..20].copy_from_slice(&self.data_28);
                0
            }
            7 => w & 1,
            8 => {
                out[..28].copy_from_slice(&self.data_0c);
                0
            }
            9 => w >> 6 & 7,
            10 => w >> 9 & 1,
            11 => w >> 10 & 1,
            _ => 0,
        }
    }

    /// C: `FUN_0200D914(play, id, src)`: set field `id` (0..11) from the value at `src` (a byte,
    /// a halfword for ids 3 and 4, or the byte array of ids 5, 6, 8); bit fields take the low
    /// bits of the byte. Ids > 11 do nothing.
    pub fn set(&mut self, id: u32, src: &[u8]) {
        let b = src[0] as u16;
        match id {
            0 => self.bits = self.bits & !0x3C | (b & 0xF) << 2,
            1 => self.b2 = src[0],
            2 => self.b3 = src[0],
            3 => self.h4 = crate::le::r16(src, 0),
            4 => self.h6 = crate::le::r16(src, 0),
            5 => self.data_08.copy_from_slice(&src[..4]),
            6 => self.data_28.copy_from_slice(&src[..20]),
            7 => self.bits = self.bits & !1 | b & 1,
            8 => self.data_0c.copy_from_slice(&src[..28]),
            9 => self.bits = self.bits & 0xFE3F | (b & 7) << 6,
            10 => self.bits = self.bits & 0xFDFF | (b & 1) << 9,
            11 => self.bits = self.bits & 0xFBFF | (b & 1) << 10,
            _ => {}
        }
    }

    /// C: `FUN_0200D9E8(play, a, b, c)`: add to the three counters, each only if the sum stays
    /// below its ceiling (signed compares): `b3 + a < 255`, `h4 + b < 0xFFFF`, `h6 + c < 0xFFFF`.
    pub fn add_counters(&mut self, a: u32, b: u32, c: u32) {
        let s = (self.b3 as u32).wrapping_add(a);
        if (s as i32) < 255 {
            self.b3 = s as u8;
        }
        let s = (self.h4 as u32).wrapping_add(b);
        if (s as i32) < 0xFFFF {
            self.h4 = s as u16;
        }
        let s = (self.h6 as u32).wrapping_add(c);
        if (s as i32) < 0xFFFF {
            self.h6 = s as u16;
        }
    }

    /// C: `FUN_0200DA14`: field 1 = 0.
    pub fn clear_b2(&mut self) {
        self.set(1, &[0]);
    }

    /// C: `FUN_0200DA28`: field 1 += 1, stopping at 255.
    pub fn inc_b2(&mut self) {
        let mut v = self.get(1, &mut []) as u8;
        v = v.saturating_add(1);
        self.set(1, &[v]);
    }

    /// C: `FUN_0200DA54`: field 1.
    pub fn b2(&self) -> u32 {
        self.get(1, &mut []) & 0xFFFF
    }
}

/// Block 58: the score data.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BsubwayScore {
    /// +0x00: Battle Points (at most 9999).
    pub bp: u16,
    /// +0x02: a streak counter (`FUN_0200DC0C`).
    pub b2: u8,
    /// +0x03: 1..10 (1 at new game).
    pub b3: u8,
    /// +0x04: 11 flag bits (`FUN_0200DBA0`).
    pub flags: u32,
    /// +0x08: per mode.
    pub h08: [u16; MODES],
    /// +0x1A: per-mode maximum.
    pub h1a: [u16; MODES],
    /// +0x2C: per-mode counter.
    pub h2c: [u16; MODES],
    /// +0x3E: computed points (`FUN_0200DC94`).
    pub points: u16,
    /// +0x40..+0x43: unused.
    pub pad: [u8; 4],
    /// +0x44 (0xB4 bytes, slot 1 of `FUN_0200DC50` / `FUN_0200DC74`).
    pub data_44: Vec<u8>,
    /// +0xF8 (0xB4 bytes, slot 0).
    pub data_f8: Vec<u8>,
}

impl Default for BsubwayScore {
    fn default() -> Self {
        BsubwayScore::from_bytes(&[0; SCORE_SIZE])
    }
}

impl BsubwayScore {
    /// Reads the original 0x1AC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut pad = [0; 4];
        pad.copy_from_slice(&b[0x40..0x44]);
        BsubwayScore {
            bp: crate::le::r16(b, 0),
            b2: b[2],
            b3: b[3],
            flags: crate::le::r32(b, 4),
            h08: crate::le::r16s(b, 8),
            h1a: crate::le::r16s(b, 0x1A),
            h2c: crate::le::r16s(b, 0x2C),
            points: crate::le::r16(b, 0x3E),
            pad,
            data_44: b[0x44..0xF8].to_vec(),
            data_f8: b[0xF8..0x1AC].to_vec(),
        }
    }

    /// The original 0x1AC-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0; SCORE_SIZE];
        crate::le::w16(&mut b, 0, self.bp);
        b[2] = self.b2;
        b[3] = self.b3;
        crate::le::w32(&mut b, 4, self.flags);
        crate::le::w16s(&mut b, 8, &self.h08);
        crate::le::w16s(&mut b, 0x1A, &self.h1a);
        crate::le::w16s(&mut b, 0x2C, &self.h2c);
        crate::le::w16(&mut b, 0x3E, self.points);
        b[0x40..0x44].copy_from_slice(&self.pad);
        b[0x44..0xF8].copy_from_slice(&self.data_44);
        b[0xF8..].copy_from_slice(&self.data_f8);
        b
    }

    /// A new-game block (`sub_0200DA6C`).
    pub fn new_game() -> Self {
        let mut s = BsubwayScore::default();
        s.init();
        s
    }

    /// C: `sub_0200DA64`: block size.
    pub fn size() -> u32 {
        SCORE_SIZE as u32
    }

    /// C: `sub_0200DA6C`: zero the block, +3 = 1.
    pub fn init(&mut self) {
        *self = BsubwayScore::default();
        self.b3 = 1;
    }

    /// C: `FUN_0200DA80(score, n)`: Battle Points += `n`, capped at 9999 (signed compare).
    pub fn add_bp(&mut self, n: u32) {
        let v = (self.bp as u32).wrapping_add(n);
        self.bp = if v as i32 > BP_MAX as i32 {
            BP_MAX as u16
        } else {
            v as u16
        };
    }

    /// C: `FUN_0200DA98(score, n)`: Battle Points -= `n`, 0 when the 16-bit signed difference is
    /// negative.
    pub fn sub_bp(&mut self, n: u32) {
        let d = (self.bp as i16).wrapping_sub(n as i16);
        self.bp = if d < 0 {
            0
        } else {
            self.bp.wrapping_sub(n as u16)
        };
    }

    /// C: `FUN_0200DABC`: Battle Points.
    pub fn bp(&self) -> u32 {
        self.bp as u32
    }

    /// C: unlabeled routine at `0x0200DAC0`: the per-mode array index of a mode (the identity).
    pub fn mode_index(mode: u32) -> u32 {
        mode
    }

    /// C: `FUN_0200DAC4(score, mode)`.
    pub fn h08(&self, mode: u32) -> u32 {
        self.h08[Self::mode_index(mode) as usize] as u32
    }

    /// C: `FUN_0200DAD8(score, mode)`.
    pub fn h1a(&self, mode: u32) -> u32 {
        self.h1a[Self::mode_index(mode) as usize] as u32
    }

    /// C: `FUN_0200DAEC(score, mode, v)`: raise the per-mode maximum to `v` (stores the low 16
    /// bits when the stored value is below the full `v`).
    pub fn raise_h1a(&mut self, mode: u32, v: u32) {
        let i = Self::mode_index(mode) as usize;
        if (self.h1a[i] as u32) < v {
            self.h1a[i] = v as u16;
        }
    }

    /// C: `FUN_0200DB08(score, mode, v)`.
    pub fn set_h08(&mut self, mode: u32, v: u32) {
        self.h08[Self::mode_index(mode) as usize] = v as u16;
    }

    /// C: `FUN_0200DB1C(score, mode)`: counter = 0.
    pub fn clear_h2c(&mut self, mode: u32) {
        self.h2c[Self::mode_index(mode) as usize] = 0;
    }

    /// C: `FUN_0200DB30(score, mode)`: counter = 1.
    pub fn start_h2c(&mut self, mode: u32) {
        self.h2c[Self::mode_index(mode) as usize] = 1;
    }

    /// C: `FUN_0200DB44(score, mode)`: counter non-zero.
    pub fn has_h2c(&self, mode: u32) -> bool {
        self.h2c[Self::mode_index(mode) as usize] != 0
    }

    /// C: `FUN_0200DB60(score, mode)`: counter += 1 while below 0xFFFE.
    pub fn inc_h2c(&mut self, mode: u32) {
        let i = Self::mode_index(mode) as usize;
        if self.h2c[i] < 0xFFFE {
            self.h2c[i] += 1;
        }
    }

    /// C: `FUN_0200DB80(score, mode)`: counter - 1, or 0 when that is negative as a 16-bit signed
    /// value (counter 0 or above 0x8000).
    pub fn h2c_minus_one(&self, mode: u32) -> u32 {
        let x = self.h2c[Self::mode_index(mode) as usize].wrapping_sub(1);
        if (x as i16) < 0 {
            0
        } else {
            x as u32
        }
    }

    /// C: `FUN_0200DBA0(score, bit, op)`: op 0 reads flag `bit`, 1 sets it, 2 clears it; returns
    /// the read bit (op 0) or 0. `bit` 11 or more (signed) does nothing and returns 0; `bit` must
    /// not be negative (the original then loops ~2^32 times building the mask).
    pub fn flag(&mut self, bit: u32, op: u32) -> u32 {
        if bit as i32 >= 11 {
            return 0;
        }
        let mask = 1u32 << bit;
        match op {
            0 => return self.flags >> bit & 1,
            1 => self.flags |= mask,
            2 => self.flags &= !mask,
            _ => {}
        }
        0
    }

    /// C: `FUN_0200DBF0`: +3 += 1 while below 10.
    pub fn inc_b3(&mut self) {
        if self.b3 < 10 {
            self.b3 += 1;
        }
    }

    /// C: `FUN_0200DBFC`: +3 -= 1 while above 1.
    pub fn dec_b3(&mut self) {
        if self.b3 > 1 {
            self.b3 -= 1;
        }
    }

    /// C: `FUN_0200DC08`.
    pub fn b3(&self) -> u32 {
        self.b3 as u32
    }

    /// C: `FUN_0200DC0C(score, kind)`: kind 2 clears flag 0 and the streak; kind 3 counts the
    /// streak up (to 255) when flag 0 is set, else sets flag 0 and starts it at 1; returns the
    /// streak.
    pub fn update_streak(&mut self, kind: u32) -> u32 {
        match kind {
            2 => {
                self.flag(0, 2);
                self.b2 = 0;
            }
            3 => {
                if self.flag(0, 0) != 0 {
                    self.b2 = self.b2.saturating_add(1);
                } else {
                    self.flag(0, 1);
                    self.b2 = 1;
                }
            }
            _ => {}
        }
        self.b2 as u32
    }

    /// C: `FUN_0200DC74(score, which, dst)`: copy 0xB4 bytes out (+0xF8 for `which == 0`, else
    /// +0x44).
    pub fn read_data(&self, which: u32, dst: &mut [u8]) {
        let src = if which == 0 {
            &self.data_f8
        } else {
            &self.data_44
        };
        dst[..0xB4].copy_from_slice(src);
    }

    /// C: `FUN_0200DC50(score, which, src)`: copy 0xB4 bytes in.
    pub fn write_data(&mut self, which: u32, src: &[u8]) {
        let dst = if which == 0 {
            &mut self.data_f8
        } else {
            &mut self.data_44
        };
        dst.copy_from_slice(&src[..0xB4]);
    }

    /// C: `FUN_0200DC94(score, play)`: points = `(play.b2 - 1) * 1000 + max(950 - (play.h4 * 10 +
    /// play.b3 * 20), 0) + (play.h6 <= 970 ? (1000 - play.h6) / 30 : 0)`, every term truncated to
    /// 16 bits as in the original.
    pub fn compute_points(&mut self, play: &BsubwayPlay) {
        let a = ((play.b2 as u32).wrapping_sub(1).wrapping_mul(1000)) & 0xFFFF;
        let t = ((play.h4 as u32 * 10) & 0xFFFF) + ((play.b3 as u32 * 20) & 0xFFFF);
        let b = if t as i32 > 950 {
            0
        } else {
            (950 - t) & 0xFFFF
        };
        let c = if play.h6 > 0x3CA {
            0
        } else {
            ((1000 - play.h6 as u32) / 30) & 0xFFFF
        };
        self.points = a.wrapping_add(b).wrapping_add(c) as u16;
    }

    /// C: `FUN_0200DCF8`.
    pub fn points(&self) -> u32 {
        self.points as u32
    }

    /// C: `FUN_0200DCFC`: points / 1000 (low byte).
    pub fn points_thousands(&self) -> u32 {
        (self.points as u32 / 1000) & 0xFF
    }
}

/// The 11 x 3 record counters at +0x5A of block 60.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BsubwayCounters {
    /// `[i][j]` at `6 * i + 2 * j`.
    pub counts: [[u16; 3]; 11],
}

impl BsubwayCounters {
    /// Reads the original 0x42-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        BsubwayCounters {
            counts: std::array::from_fn(|i| crate::le::r16s(b, 6 * i)),
        }
    }

    /// The original 0x42-byte image.
    pub fn to_bytes(&self) -> [u8; COUNTERS_SIZE] {
        let mut b = [0; COUNTERS_SIZE];
        for (i, row) in self.counts.iter().enumerate() {
            crate::le::w16s(&mut b, 6 * i, row);
        }
        b
    }

    /// C: unlabeled routine at `0x0200DFCC`: zero the counters.
    pub fn clear(&mut self) {
        *self = BsubwayCounters::default();
    }

    /// C: `FUN_0200DFD8(c, i, j)`.
    pub fn get(&self, i: usize, j: usize) -> u32 {
        self.counts[i][j] as u32
    }

    /// C: `FUN_0200DFE4(c, i, j, v)`: store `v` clamped to 0..=0xFFFF (signed).
    pub fn set(&mut self, i: usize, j: usize, v: u32) {
        self.counts[i][j] = (v as i32).clamp(0, 0xFFFF) as u16;
    }

    /// C: `FUN_0200E008(c, i, j)`: += 1 while below 0xFFFF.
    pub fn inc(&mut self, i: usize, j: usize) {
        self.counts[i][j] = self.counts[i][j].saturating_add(1);
    }
}

#[cfg(test)]
mod tests;
