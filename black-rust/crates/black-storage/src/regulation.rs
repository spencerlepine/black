//! `regulation.c` (S5): battle regulations (0xBC bytes), regulation cards (0x160 bytes: a
//! regulation plus card data and a CRC) and the save block 47 (0x2FC bytes: two cards and two
//! 30-byte party summaries), main ARM9 0x0200ADE8..0x0200B364. Verified per function with
//! `tools/check-fn.sh black-storage regulation` (spec `tools/rom-diff/specs/black-storage/regulation.spec`).
//!
//! The regulation is read and written through numbered parameters (`FUN_0200AE54` /
//! `FUN_0200AEE8`, ids 0..22); the meaning of each id is not established (UNVERIFIED), so the
//! fields are named after their offsets and the setters' bounds are documented on
//! [`Regulation::set`]. Ids 9, 10 and 14 are bit sets (0x52, 0x4C and 7 bytes at +0x0A, +0x5C,
//! +0xAC) read with [`Regulation::flag`].
//!
//! Not ported: `FUN_0200ADF0` (allocate a regulation; ownership), `FUN_0200AE18` /
//! `FUN_0200AE3C` (cup name into a string buffer: message bank 0xA6 of ARC 2, text glue),
//! `FUN_0200B358` (`FUN_020071CC(sav, 47)`; the [`crate::save::SaveData`] owns the block).

use serde::{Deserialize, Serialize};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_ADE8,
    0x0200_AE08,
    0x0200_AE14,
    0x0200_AE54,
    0x0200_AEE8,
    0x0200_AFD4,
    0x0200_B040,
    0x0200_B074,
    0x0200_B078,
    0x0200_B110,
    0x0200_B1A8,
    0x0200_B1C4,
    0x0200_B1D4,
    0x0200_B1E0,
    0x0200_B1F8,
    0x0200_B204,
    0x0200_B214,
    0x0200_B254,
    0x0200_B2A4,
    0x0200_B348,
];

/// Save block number of the regulation block.
pub const BLOCK_ID: u32 = 47;
/// Bytes of a regulation.
pub const REGULATION_SIZE: usize = 0xBC;
/// Bytes of a regulation card.
pub const CARD_SIZE: usize = 0x160;
/// Bytes the card CRC covers (everything before the CRC field).
pub const CARD_CRC_LEN: usize = 0x15E;
/// Bytes of a party summary slot.
pub const SUMMARY_SIZE: usize = 30;
/// Bytes of the save block.
pub const BLOCK_SIZE: usize = 2 * CARD_SIZE + 2 * SUMMARY_SIZE;
/// Offset of the summaries in the block.
pub const SUMMARY_OFS: usize = 2 * CARD_SIZE;
/// Offset of the card text (`FUN_0200B074`).
pub const CARD_TEXT_OFS: u32 = 0xC0;

/// The undecompiled routines the regulation code calls.
pub trait RegulationHooks {
    /// C: `FUN_0203F1FC(data, len)` (`gf_standard.c`): CRC-16 of `data` (SDK
    /// `MATH_CalcCRC16CCITT` with the game's table).
    fn crc16(&mut self, data: &[u8]) -> u16;
}

/// The party a summary is built from (`pokeparty.c` / `poke_tool.c` callees).
pub trait PartySource {
    /// C: `FUN_0201AA18(party)`: number of Pokemon.
    fn count(&mut self) -> u32;
    /// C: `FUN_0201AB28(party, i)`: the Pokemon in slot `i` (an opaque handle).
    fn mon(&mut self, i: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, 0)`: a Pokemon field.
    fn mon_param(&mut self, mon: u32, id: u32) -> u32;
}

/// A battle regulation (0xBC bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Regulation {
    /// +0x00..+0x05: parameters 0..5.
    pub b00: [u8; 6],
    /// +0x06: parameter 6 (at most 600).
    pub h06: u16,
    /// +0x08: parameter 7.
    pub b08: u8,
    /// +0x09: parameter 8.
    pub b09: u8,
    /// +0x0A: bit set of parameter 9 (0x52 bytes).
    pub bits_0a: Vec<u8>,
    /// +0x5C: bit set of parameter 10 (0x4C bytes).
    pub bits_5c: Vec<u8>,
    /// +0xA8: parameter 11.
    pub h_a8: u16,
    /// +0xAA: parameter 12.
    pub b_aa: u8,
    /// +0xAB: parameter 13.
    pub b_ab: u8,
    /// +0xAC: bit set of parameter 14 (7 bytes).
    pub bits_ac: [u8; 7],
    /// +0xB3..+0xBB: parameters 15 (+0xB3), 16, 17, 18 (+0xB6), unused +0xB7, 19 (+0xB8), 20, 21, 22.
    pub tail: [u8; 9],
}

impl Default for Regulation {
    fn default() -> Self {
        Regulation::from_bytes(&[0; REGULATION_SIZE])
    }
}

/// Offset, width and setter bound of a regulation parameter.
enum Param {
    None,
    /// `(offset, is_u16, bound)`: the setter stores `v` only when `bound(v)` holds (signed compare).
    Field(usize, bool, fn(i32) -> bool),
}

fn param(id: u32) -> Param {
    use Param::*;
    let any: fn(i32) -> bool = |_| true;
    match id {
        0 => Field(0, false, |v| v < 7),
        1..=3 => Field(id as usize, false, |v| v < 6),
        4 => Field(4, false, |v| v <= 100),
        5 => Field(5, false, |v| v <= 6),
        6 => Field(6, true, |v| v <= 600),
        7 => Field(8, false, |v| v < 2),
        8 => Field(9, false, |v| v < 2),
        11 => Field(0xA8, true, |v| v < 0x28B),
        12 => Field(0xAA, false, any),
        13 => Field(0xAB, false, |v| v < 2),
        15 => Field(0xB3, false, any),
        16 => Field(0xB4, false, any),
        17 => Field(0xB5, false, |v| v < 2),
        18 => Field(0xB6, false, any),
        19 => Field(0xB8, false, |v| v < 2),
        20 => Field(0xB9, false, |v| v <= 99),
        21 => Field(0xBA, false, |v| v < 6),
        22 => Field(0xBB, false, any),
        _ => None,
    }
}

impl Regulation {
    /// Reads the original 0xBC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut b00 = [0; 6];
        b00.copy_from_slice(&b[0..6]);
        let mut bits_ac = [0; 7];
        bits_ac.copy_from_slice(&b[0xAC..0xB3]);
        let mut tail = [0; 9];
        tail.copy_from_slice(&b[0xB3..0xBC]);
        Regulation {
            b00,
            h06: crate::le::r16(b, 6),
            b08: b[8],
            b09: b[9],
            bits_0a: b[0x0A..0x5C].to_vec(),
            bits_5c: b[0x5C..0xA8].to_vec(),
            h_a8: crate::le::r16(b, 0xA8),
            b_aa: b[0xAA],
            b_ab: b[0xAB],
            bits_ac,
            tail,
        }
    }

    /// The original 0xBC-byte image.
    pub fn to_bytes(&self) -> [u8; REGULATION_SIZE] {
        let mut b = [0; REGULATION_SIZE];
        b[0..6].copy_from_slice(&self.b00);
        crate::le::w16(&mut b, 6, self.h06);
        b[8] = self.b08;
        b[9] = self.b09;
        b[0x0A..0x5C].copy_from_slice(&self.bits_0a);
        b[0x5C..0xA8].copy_from_slice(&self.bits_5c);
        crate::le::w16(&mut b, 0xA8, self.h_a8);
        b[0xAA] = self.b_aa;
        b[0xAB] = self.b_ab;
        b[0xAC..0xB3].copy_from_slice(&self.bits_ac);
        b[0xB3..0xBC].copy_from_slice(&self.tail);
        b
    }

    /// C: `FUN_0200AE08(src, dst)`: copy (`MI_CpuCopy8(src, dst, 0xBC)`).
    pub fn copy(src: &Regulation, dst: &mut Regulation) {
        dst.clone_from(src);
    }

    /// C: `FUN_0200AE14(reg)`: an empty function in the original (does nothing).
    pub fn noop(&mut self) {}

    /// C: `FUN_0200AE54(reg, id)`: parameter `id` (0 for the bit-set ids 9, 10, 14 and ids > 22).
    pub fn get(&self, id: u32) -> u32 {
        match param(id) {
            Param::None => 0,
            Param::Field(o, wide, _) => {
                let b = self.to_bytes();
                if wide {
                    crate::le::r16(&b, o) as u32
                } else {
                    b[o] as u32
                }
            }
        }
    }

    /// C: `FUN_0200AEE8(reg, id, v)`: store parameter `id` if `v` passes its bound (signed
    /// compares): id 0 `< 7`; 1-3 `< 6`; 4 `<= 100`; 5 `<= 6`; 6 `<= 600`; 7, 8, 13, 17, 19 `< 2`;
    /// 11 `< 0x28B`; 20 `<= 99`; 21 `< 6`; 12, 15, 16, 18, 22 any. The low 8 (16 for ids 6 and 11)
    /// bits are stored. Ids 9, 10, 14 and > 22 store nothing.
    pub fn set(&mut self, id: u32, v: u32) {
        if let Param::Field(o, wide, ok) = param(id) {
            if ok(v as i32) {
                let mut b = self.to_bytes();
                if wide {
                    crate::le::w16(&mut b, o, v as u16);
                } else {
                    b[o] = v as u8;
                }
                *self = Regulation::from_bytes(&b);
            }
        }
    }

    /// C: `FUN_0200AFD4(reg, id, n)`: bit `n` of the bit set of parameter 9 (+0x0A), 10 (+0x5C)
    /// or 14 (+0xAC); 0 for any other id. `n` must not be negative (the original then reads
    /// before the set); the bit index may run past the set into the following fields, as in the
    /// original.
    pub fn flag(&self, id: u32, n: u32) -> bool {
        let base = match id {
            9 => 0x0A,
            10 => 0x5C,
            14 => 0xAC,
            _ => return false,
        };
        let b = self.to_bytes();
        b[base + (n >> 3) as usize] >> (n & 7) & 1 != 0
    }

    /// C: `FUN_0200B040(reg, out)`: 8 bytes: the 7-byte bit set of parameter 14, then parameter 13
    /// (clamped to 1 when 2 or more).
    pub fn rule_bytes(&self) -> [u8; 8] {
        let mut out = [0; 8];
        out[7] = self.get(13) as u8;
        if out[7] >= 2 {
            out[7] = 1;
        }
        out[..7].copy_from_slice(&self.bits_ac);
        out
    }
}

/// A card parameter as `FUN_0200B078` returns it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardParam {
    /// Id 0 (and ids > 12): the original returns the card's own address (its regulation is at +0).
    Card,
    /// A field value (sign-extended where the field is signed).
    Value(u32),
}

/// A regulation card (0x160 bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegulationCard {
    /// +0x000: the regulation.
    pub regulation: Regulation,
    /// +0x0BC: card parameter 1.
    pub p1: u32,
    /// +0x0C0: 73 code units (`FUN_0200B074` hands out their address; UNVERIFIED: card text).
    pub text: Vec<u16>,
    /// +0x152: card parameter 2.
    pub p2: i16,
    /// +0x154..+0x15B: card parameters 3..10.
    pub p3: [i8; 8],
    /// +0x15C: card parameter 11.
    pub p11: u8,
    /// +0x15D: card parameter 12.
    pub p12: u8,
    /// +0x15E: CRC-16 of bytes 0..0x15E.
    pub crc: u16,
}

impl Default for RegulationCard {
    fn default() -> Self {
        RegulationCard::from_bytes(&[0; CARD_SIZE])
    }
}

impl RegulationCard {
    /// Reads the original 0x160-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        RegulationCard {
            regulation: Regulation::from_bytes(&b[..REGULATION_SIZE]),
            p1: crate::le::r32(b, 0xBC),
            text: (0..73).map(|i| crate::le::r16(b, 0xC0 + 2 * i)).collect(),
            p2: crate::le::r16(b, 0x152) as i16,
            p3: std::array::from_fn(|i| b[0x154 + i] as i8),
            p11: b[0x15C],
            p12: b[0x15D],
            crc: crate::le::r16(b, 0x15E),
        }
    }

    /// The original 0x160-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0; CARD_SIZE];
        b[..REGULATION_SIZE].copy_from_slice(&self.regulation.to_bytes());
        crate::le::w32(&mut b, 0xBC, self.p1);
        crate::le::w16s(&mut b, 0xC0, &self.text);
        crate::le::w16(&mut b, 0x152, self.p2 as u16);
        for (i, &x) in self.p3.iter().enumerate() {
            b[0x154 + i] = x as u8;
        }
        b[0x15C] = self.p11;
        b[0x15D] = self.p12;
        crate::le::w16(&mut b, 0x15E, self.crc);
        b
    }

    /// C: `FUN_0200B074(card)`: the text (the original returns its address, card + 0xC0).
    pub fn text(&self) -> &[u16] {
        &self.text
    }

    /// C: `FUN_0200B078(card, id)`: id 1 `p1`, 2 `p2` (sign-extended), 3..10 `p3[id - 3]`
    /// (sign-extended), 11 `p11`, 12 `p12`; id 0 and ids > 12 give the card itself.
    pub fn param(&self, id: u32) -> CardParam {
        CardParam::Value(match id {
            1 => self.p1,
            2 => self.p2 as i32 as u32,
            3..=10 => self.p3[id as usize - 3] as i32 as u32,
            11 => self.p11 as u32,
            12 => self.p12 as u32,
            _ => return CardParam::Card,
        })
    }

    /// C: `FUN_0200B110(card, id, v)`: store the low bits of `v` into card parameter `id` (1..12;
    /// other ids do nothing).
    pub fn set_param(&mut self, id: u32, v: u32) {
        match id {
            1 => self.p1 = v,
            2 => self.p2 = v as i16,
            3..=10 => self.p3[id as usize - 3] = v as i8,
            11 => self.p11 = v as u8,
            12 => self.p12 = v as u8,
            _ => {}
        }
    }

    /// C: `FUN_0200B1C4(card)`: CRC-16 of the first 0x15E bytes.
    pub fn compute_crc<H: RegulationHooks>(&self, h: &mut H) -> u16 {
        h.crc16(&self.to_bytes()[..CARD_CRC_LEN])
    }

    /// C: `FUN_0200B1A8(card)`: does the stored CRC match.
    pub fn is_valid<H: RegulationHooks>(&self, h: &mut H) -> bool {
        self.crc == self.compute_crc(h)
    }
}

/// A 30-byte party summary (species, a per-slot byte and two per-slot bit sets).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartySummary {
    /// +0: kind 0, per slot (`FUN_02017E1C` id 5: species).
    pub species: [u16; 6],
    /// +12: kind 1, per slot (id 0x6F).
    pub field_6f: [u8; 6],
    /// +18: kind 2, bit per slot (has a held item, id 6).
    pub held_bits: u8,
    /// +19: kind 3, bit per slot (id 0x6E equal to 1).
    pub bits_6e: u8,
    /// +20..+29: unused.
    pub pad: [u8; 10],
}

impl PartySummary {
    /// Reads the original 30-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut field_6f = [0; 6];
        field_6f.copy_from_slice(&b[12..18]);
        let mut pad = [0; 10];
        pad.copy_from_slice(&b[20..30]);
        PartySummary {
            species: crate::le::r16s(b, 0),
            field_6f,
            held_bits: b[18],
            bits_6e: b[19],
            pad,
        }
    }

    /// The original 30-byte image.
    pub fn to_bytes(&self) -> [u8; SUMMARY_SIZE] {
        let mut b = [0; SUMMARY_SIZE];
        crate::le::w16s(&mut b, 0, &self.species);
        b[12..18].copy_from_slice(&self.field_6f);
        b[18] = self.held_bits;
        b[19] = self.bits_6e;
        b[20..30].copy_from_slice(&self.pad);
        b
    }

    /// C: `FUN_0200B1F8(sum)`: clear the 30 bytes.
    pub fn clear(&mut self) {
        *self = PartySummary::default();
    }

    /// C: `FUN_0200B214(sum, kind, idx)`: kind 0 species, 1 the per-slot byte, 2 / 3 bit `idx`
    /// of the bit sets (register shift: an amount of 32 or more gives 0); 0 for other kinds.
    /// For kinds 0 and 1 an `idx` of 6 or more reads the following bytes of the summary, as in the
    /// original (it must stay inside the 30 bytes here).
    pub fn get(&self, kind: u32, idx: u32) -> u32 {
        let shr = |b: u8| (b as u32).checked_shr(idx & 0xFF).unwrap_or(0) & 1;
        let img = self.to_bytes();
        match kind {
            0 => crate::le::r16(&img, 2 * idx as usize) as u32,
            1 => img[12 + idx as usize] as u32,
            2 => shr(self.held_bits),
            3 => shr(self.bits_6e),
            _ => 0,
        }
    }

    /// C: unlabeled routine at `0x0200B254` (`(sum, kind, idx, v)`): kind 0 species = `v`, 1 the
    /// per-slot byte = `v`, 2 / 3 OR in bit `idx` when `v == 1` (bits are never cleared).
    pub fn set(&mut self, kind: u32, idx: u32, v: u32) {
        let bit = || ((v == 1) as u32).checked_shl(idx & 0xFF).unwrap_or(0) as u8;
        let mut img = self.to_bytes();
        match kind {
            0 => crate::le::w16(&mut img, 2 * idx as usize, v as u16),
            1 => img[12 + idx as usize] = v as u8,
            2 => img[18] |= bit(),
            3 => img[19] |= bit(),
            _ => {}
        }
        *self = PartySummary::from_bytes(&img);
    }

    /// C: `FUN_0200B2A4(sum, party)`: clear, then for each Pokemon whose field 0xA9 is non-zero
    /// store its species (id 5), field 0x6F, held-item bit (id 6 non-zero) and field 0x6E (bit set
    /// when it is 1).
    pub fn build<P: PartySource>(&mut self, p: &mut P) {
        self.clear();
        let mut i = 0u32;
        while (i as i32) < p.count() as i32 {
            let mon = p.mon(i);
            if p.mon_param(mon, 0xA9) != 0 {
                let slot = i & 0xFF;
                let species = p.mon_param(mon, 5);
                self.set(0, slot, species);
                let f = p.mon_param(mon, 0x6F);
                self.set(1, slot, f);
                let held = p.mon_param(mon, 6);
                self.set(2, slot, (held != 0) as u32);
                let g = p.mon_param(mon, 0x6E);
                self.set(3, slot, g);
            }
            i += 1;
        }
    }
}

/// Save block 47: two regulation cards and two party summaries.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegulationBlock {
    /// +0x000, +0x160.
    pub cards: [RegulationCard; 2],
    /// +0x2C0, +0x2DE.
    pub summaries: [PartySummary; 2],
}

impl RegulationBlock {
    /// Reads the original 0x2FC-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        RegulationBlock {
            cards: std::array::from_fn(|i| RegulationCard::from_bytes(&b[i * CARD_SIZE..])),
            summaries: std::array::from_fn(|i| {
                PartySummary::from_bytes(&b[SUMMARY_OFS + i * SUMMARY_SIZE..])
            }),
        }
    }

    /// The original 0x2FC-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(BLOCK_SIZE);
        for c in &self.cards {
            b.extend_from_slice(&c.to_bytes());
        }
        for s in &self.summaries {
            b.extend_from_slice(&s.to_bytes());
        }
        b
    }

    /// C: `FUN_0200ADE8`: block size.
    pub fn size() -> u32 {
        BLOCK_SIZE as u32
    }

    /// C: `sub_0200B348`: zero the block (the new-game state).
    pub fn init(&mut self) {
        *self = RegulationBlock::default();
    }

    /// C: `FUN_0200B1D4(block, i)`: card `i` (the original returns block + i * 0x160).
    pub fn card_mut(&mut self, i: usize) -> &mut RegulationCard {
        &mut self.cards[i]
    }

    /// C: unlabeled routine at `0x0200B1E0` (`(block, i, src)`): card `i` = `src`.
    pub fn set_card(&mut self, i: usize, src: &RegulationCard) {
        self.cards[i] = src.clone();
    }

    /// C: `FUN_0200B204(block, i)`: summary `i` (the original returns block + 0x2C0 + 30 * i).
    pub fn summary_mut(&mut self, i: usize) -> &mut PartySummary {
        &mut self.summaries[i]
    }
}

#[cfg(test)]
mod tests;
