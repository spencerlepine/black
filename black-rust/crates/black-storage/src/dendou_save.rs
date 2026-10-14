//! Save block 66 (S5b; 0x16C bytes: one Hall of Fame record, a team of six and a date), main ARM9
//! 0x0200EA14..0x0200ECB4 (callers: overlay 48 `field_gimmick_gate.c` and overlays 171 / 205).
//! Verified per function with `tools/check-fn.sh black-storage dendou_save` (spec
//! `tools/rom-diff/specs/black-storage/dendou_save.spec`).
//!
//! Record: six 0x3C-byte entries (+0x00 `u16` species, +0x02 `u8` parameter 158 (UNVERIFIED: form),
//! +0x03 bits 0..5 parameter 111 and bits 6..7 parameter 110 (UNVERIFIED: level / gender), +0x04
//! `u32` personality, +0x08 `u32` trainer id, +0x0C nickname (11 `u16`), +0x22 trainer name (8
//! `u16`), +0x32 four moves, +0x3A pad), +0x168 `u16` year, +0x16A `u8` month, +0x16B `u8` day.
//!
//! Not ported: the 15-record Hall of Fame ring of the extra save area (0x155C bytes; its init
//! `sub_0200EA1C` and `FUN_0200EBD0` / `FUN_0200ECB4` / `0x0200ECC4` / `0x0200ECD0` /
//! `FUN_0200ECF0` / `0x0200ED14`): extra-area data, not a block.

use serde::{Deserialize, Serialize};

use crate::le::{r16, r16s, r32, w16, w16s, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_EA14,
    0x0200_EA2C,
    0x0200_EA3C,
    0x0200_EC14,
    0x0200_EC2C,
    0x0200_EC98,
];

/// Save block number.
pub const BLOCK_ID: u32 = 66;
/// Bytes of the block.
pub const SIZE: usize = 0x16C;
/// Bytes of one entry.
pub const MON_SIZE: usize = 0x3C;
/// Entries.
pub const MONS: usize = 6;
/// The string terminator (`FUN_02045A98`, `black_text::strbuf::EOS`).
pub const EOS: u16 = 0xFFFF;

/// The party, Pokemon and string callees of [`HofRecord::fill`] / [`HofRecord::mon_view`].
pub trait HofHooks {
    /// C: `FUN_0201AA18(party)`: the number of Pokemon.
    fn party_count(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: the Pokemon at `i`.
    fn party_mon(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_02017D30(mon)`: open the Pokemon's data for fast reads (returns the old state).
    fn lock(&mut self, mon: u32) -> u32;
    /// C: `FUN_02017D70(mon, state)`: restore it.
    fn unlock(&mut self, mon: u32, state: u32);
    /// C: `FUN_02017E1C(mon, id, arg)`: a party Pokemon parameter (`arg` = a string buffer for
    /// parameters 115 / 141).
    fn param(&mut self, mon: u32, id: u32, arg: u32) -> u32;
    /// C: `FUN_020457B0(11, heap)`: a new 11-character string buffer (0 = none).
    fn str_alloc(&mut self, heap: u32) -> u32;
    /// C: `FUN_02045A38(s, dst, n)`: copy the string out as `n` characters.
    fn str_read(&mut self, s: u32, dst: &mut [u16]);
    /// C: `FUN_02045994(s, src, n)`: set the string from `n` characters.
    fn str_write(&mut self, s: u32, src: &[u16]);
    /// C: `FUN_02045808(s)`: free it.
    fn str_free(&mut self, s: u32);
}

/// One Pokemon of the record.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HofMon {
    /// +0x00.
    pub species: u16,
    /// +0x02.
    pub form: u8,
    /// +0x03.
    pub b3: u8,
    /// +0x04.
    pub pid: u32,
    /// +0x08.
    pub id: u32,
    /// +0x0C.
    pub nickname: [u16; 11],
    /// +0x22.
    pub ot_name: [u16; 8],
    /// +0x32.
    pub moves: [u16; 4],
    /// +0x3A.
    pub pad: u16,
}

impl HofMon {
    fn from_bytes(b: &[u8]) -> Self {
        HofMon {
            species: r16(b, 0),
            form: b[2],
            b3: b[3],
            pid: r32(b, 4),
            id: r32(b, 8),
            nickname: r16s(b, 0x0C),
            ot_name: r16s(b, 0x22),
            moves: r16s(b, 0x32),
            pad: r16(b, 0x3A),
        }
    }

    fn write(&self, b: &mut [u8]) {
        w16(b, 0, self.species);
        b[2] = self.form;
        b[3] = self.b3;
        w32(b, 4, self.pid);
        w32(b, 8, self.id);
        w16s(b, 0x0C, &self.nickname);
        w16s(b, 0x22, &self.ot_name);
        w16s(b, 0x32, &self.moves);
        w16(b, 0x3A, self.pad);
    }
}

/// What [`HofRecord::mon_view`] fills (0x1C bytes in the original: two string buffers, then
/// these fields).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HofMonView {
    /// +0x08.
    pub pid: u32,
    /// +0x0C.
    pub id: u32,
    /// +0x10.
    pub species: u16,
    /// +0x12.
    pub form: u8,
    /// +0x13.
    pub b3: u8,
    /// +0x14.
    pub moves: [u16; 4],
}

/// Save block 66.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HofRecord {
    /// The six Pokemon.
    pub mons: [HofMon; MONS],
    /// +0x168.
    pub year: u16,
    /// +0x16A.
    pub month: u8,
    /// +0x16B.
    pub day: u8,
}

impl HofRecord {
    /// Reads the original 0x16C-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        HofRecord {
            mons: std::array::from_fn(|i| HofMon::from_bytes(&b[MON_SIZE * i..])),
            year: r16(b, 0x168),
            month: b[0x16A],
            day: b[0x16B],
        }
    }

    /// The original 0x16C-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        for (i, m) in self.mons.iter().enumerate() {
            m.write(&mut b[MON_SIZE * i..]);
        }
        w16(&mut b, 0x168, self.year);
        b[0x16A] = self.month;
        b[0x16B] = self.day;
        b
    }

    /// C: `sub_0200EA14`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200EA2C`: zero the block.
    pub fn init(&mut self) {
        *self = HofRecord::default();
    }

    /// C: `FUN_0200EA3C(rec, party, date, heap)`: clear the six entries, store the party's
    /// non-egg Pokemon (parameter 76 = 0) in order, then the date (`date` = year, month, day
    /// words; year as `u16`, month / day as `u8`).
    pub fn fill<H: HofHooks>(&mut self, party: u32, date: [u32; 3], heap: u32, h: &mut H) {
        let n = h.party_count(party) as i32;
        let s = h.str_alloc(heap);
        self.mons = Default::default();
        let mut k = 0;
        for i in 0..n.max(0) as u32 {
            let mon = h.party_mon(party, i);
            let state = h.lock(mon);
            if h.param(mon, 76, 0) == 0 {
                let e = &mut self.mons[k];
                e.species = h.param(mon, 5, 0) as u16;
                e.form = h.param(mon, 158, 0) as u8;
                e.b3 = (e.b3 & !0x3F) | (h.param(mon, 111, 0) as u8 & 0x3F);
                e.b3 = (e.b3 & !0xC0) | ((h.param(mon, 110, 0) as u8 & 3) << 6);
                e.pid = h.param(mon, 0, 0);
                e.id = h.param(mon, 7, 0);
                for (j, id) in (54..58).enumerate() {
                    e.moves[j] = h.param(mon, id, 0) as u16;
                }
                if s != 0 {
                    h.param(mon, 115, s);
                    h.str_read(s, &mut e.nickname);
                    h.param(mon, 141, s);
                    h.str_read(s, &mut e.ot_name);
                } else {
                    e.nickname[0] = EOS;
                    e.ot_name[0] = EOS;
                }
                k += 1;
            }
            h.unlock(mon, state);
        }
        self.year = date[0] as u16;
        self.month = date[1] as u8;
        self.day = date[2] as u8;
        if s != 0 {
            h.str_free(s);
        }
    }

    /// C: `sub_0200EC14(rec)`: the number of entries before the first with species 0.
    pub fn count(&self) -> u32 {
        self.mons.iter().take_while(|m| m.species != 0).count() as u32
    }

    /// C: `FUN_0200EC2C(rec, i, out)`: entry `i` into the view (fields) and the two string buffers
    /// `nickname_buf` / `ot_buf` (the view's +0x00 / +0x04).
    pub fn mon_view<H: HofHooks>(
        &self,
        i: usize,
        out: &mut HofMonView,
        nickname_buf: u32,
        ot_buf: u32,
        h: &mut H,
    ) {
        let m = &self.mons[i];
        out.species = m.species;
        out.form = m.form;
        out.pid = m.pid;
        out.id = m.id;
        out.b3 = m.b3;
        h.str_write(nickname_buf, &m.nickname);
        h.str_write(ot_buf, &m.ot_name);
        out.moves = m.moves;
    }

    /// C: `sub_0200EC98(rec, out)`: the date as four words (year, month, day, 0).
    pub fn date(&self) -> [u32; 4] {
        [self.year as u32, self.month as u32, self.day as u32, 0]
    }
}

#[cfg(test)]
mod tests;
