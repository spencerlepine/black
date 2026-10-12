//! Personal data (ARC 016): base stats, types, abilities, EV yield, catch rate, growth (A7a).
//!
//! The NARC (`a/0/1/6`) has 669 members: 668 personal records (index 0 is an empty dummy, 1..=649
//! are species by national dex number, 650..=667 are alternate forms) and a final 1300-byte
//! table of 650 `u16` (see [`PersonalTable::trailer_from_narc`]). Records are 60 bytes, except
//! member 0 which is 56 bytes (it has no tutor word).
//!
//! Record layout (little-endian; stat order is HP, Atk, Def, Spe, SpA, SpD as in
//! [`black_core::ids::stat`]). Confirmed against the main-ARM9 accessor `FUN_0201AE1C`, which
//! switches on a field number and reads these offsets:
//!
//! ```text
//! 00 u8*6  base stats            06 u8*2 types              08 u8 catch rate    09 u8 stage
//! 0A u16   EV yield (2 bits per stat in stat order, bits 12..16 = unk)
//! 0C u16*3 held items            12 u8 gender ratio         13 u8 egg cycles    14 u8 friendship
//! 15 u8 growth rate              16 u8*2 egg groups         18 u8*3 abilities   1B u8 flee rate
//! 1C u16 form stats index        1E u16 form sprite index   20 u8 form count    21 u8 color+flags
//! 22 u16 base exp                24 u16 height              26 u16 weight
//! 28 u32*4 TM/HM bits (101 used) 38 u32 tutor bits (absent in the 56-byte record)
//! ```

use black_core::ids::{AbilityId, ItemId};
use serde::{Deserialize, Serialize};

/// Size of a full record.
pub const RECORD_LEN: usize = 60;
/// Size of the short record (member 0): everything but the tutor word.
pub const SHORT_RECORD_LEN: usize = 56;
/// Gender ratio value meaning "genderless".
pub const GENDERLESS: u8 = 255;

/// One personal-data record (one species or alternate form).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Personal {
    /// HP, Atk, Def, Spe, SpA, SpD.
    pub base_stats: [u8; 6],
    /// Primary and secondary type (`black_core::ids::types`); a mono-type mon repeats the type.
    pub types: [u8; 2],
    pub catch_rate: u8,
    /// Evolution stage (1..=3). Guess from the values (Bulbasaur line 1, 2, 3); meaning of 3 on
    /// legendaries unverified.
    pub stage: u8,
    /// EV yield per stat, in stat order (2 bits each in the file).
    pub ev_yield: [u8; 6],
    /// Bits 12..16 of the EV word; set on 2 records, meaning unknown.
    pub unk_0a_hi: u8,
    /// Wild held items (guess: 50% / 5% / 1% slots); 0 = none.
    pub held_items: [ItemId; 3],
    /// Female chance in 1/254 steps (0 = always male, 254 = always female, 255 = genderless).
    pub gender_ratio: u8,
    /// Egg cycles to hatch.
    pub egg_cycles: u8,
    pub base_friendship: u8,
    /// Index into the EXP growth tables (ARC 017).
    pub growth_rate: u8,
    pub egg_groups: [u8; 2],
    /// Ability 1, ability 2 (0 = none), hidden ability (0 = none).
    pub abilities: [AbilityId; 3],
    /// Escape rate (0, 60, 90, 120 or 150); guess: the Safari/flee value.
    pub flee_rate: u8,
    /// Index of the first alternate-form record when `form_count > 1` (0 = none).
    pub form_stats_index: u16,
    /// First alternate-form sprite index (0 = none); guess from the values.
    pub form_sprite_index: u16,
    pub form_count: u8,
    /// Pokedex color (low 6 bits of byte 0x21).
    pub color: u8,
    /// Bit 6 of byte 0x21; set on 47 records (not gender-specific), meaning unknown.
    pub unk_21_bit6: bool,
    /// Bit 7 of byte 0x21; set only on record 493 (Arceus), meaning unknown.
    pub unk_21_bit7: bool,
    pub base_exp: u16,
    /// Height in decimetres (guess from Bulbasaur = 7).
    pub height: u16,
    /// Weight in hectograms (guess from Bulbasaur = 69).
    pub weight: u16,
    /// TM/HM compatibility bits, little-endian across the four words; 101 bits are used.
    pub tm_hm: [u32; 4],
    /// Move tutor compatibility bits; `None` for the 56-byte record (member 0).
    pub tutors: Option<u32>,
}

impl Personal {
    pub fn is_genderless(&self) -> bool {
        self.gender_ratio == GENDERLESS
    }

    /// Whether the TM/HM bit `index` (0-based over the 101-bit field) is set.
    pub fn can_learn_tm_hm(&self, index: usize) -> bool {
        index < 128 && (self.tm_hm[index / 32] >> (index % 32)) & 1 != 0
    }
}

/// All personal records, indexed by record number (species number for 1..=649).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PersonalTable(pub Vec<Personal>);

impl PersonalTable {
    /// Record number `index` (species number for 1..=649, alternate forms after).
    pub fn get(&self, index: usize) -> Option<&Personal> {
        self.0.get(index)
    }
}

#[cfg(feature = "rom")]
mod rom {
    use super::*;
    use nds_rom::narc::Narc;
    use nds_rom::{Error, Result};

    fn bad(detail: String) -> Error {
        Error::Invalid {
            what: "personal data",
            detail,
        }
    }

    fn u16_at(b: &[u8], o: usize) -> u16 {
        u16::from_le_bytes([b[o], b[o + 1]])
    }

    fn u32_at(b: &[u8], o: usize) -> u32 {
        u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
    }

    impl Personal {
        /// Decodes a 56- or 60-byte record.
        pub fn from_bytes(b: &[u8]) -> Result<Self> {
            if b.len() != RECORD_LEN && b.len() != SHORT_RECORD_LEN {
                return Err(bad(format!(
                    "record is {} bytes, expected 56 or 60",
                    b.len()
                )));
            }
            let ev = u16_at(b, 0x0A);
            let mut ev_yield = [0u8; 6];
            for (i, e) in ev_yield.iter_mut().enumerate() {
                *e = ((ev >> (2 * i)) & 3) as u8;
            }
            Ok(Personal {
                base_stats: b[0..6].try_into().unwrap(),
                types: [b[6], b[7]],
                catch_rate: b[8],
                stage: b[9],
                ev_yield,
                unk_0a_hi: (ev >> 12) as u8,
                held_items: [u16_at(b, 0x0C), u16_at(b, 0x0E), u16_at(b, 0x10)],
                gender_ratio: b[0x12],
                egg_cycles: b[0x13],
                base_friendship: b[0x14],
                growth_rate: b[0x15],
                egg_groups: [b[0x16], b[0x17]],
                abilities: [b[0x18].into(), b[0x19].into(), b[0x1A].into()],
                flee_rate: b[0x1B],
                form_stats_index: u16_at(b, 0x1C),
                form_sprite_index: u16_at(b, 0x1E),
                form_count: b[0x20],
                color: b[0x21] & 0x3F,
                unk_21_bit6: b[0x21] & 0x40 != 0,
                unk_21_bit7: b[0x21] & 0x80 != 0,
                base_exp: u16_at(b, 0x22),
                height: u16_at(b, 0x24),
                weight: u16_at(b, 0x26),
                tm_hm: [
                    u32_at(b, 0x28),
                    u32_at(b, 0x2C),
                    u32_at(b, 0x30),
                    u32_at(b, 0x34),
                ],
                tutors: (b.len() == RECORD_LEN).then(|| u32_at(b, 0x38)),
            })
        }

        /// Exact inverse of [`Personal::from_bytes`]. Fails on values that do not fit the
        /// bit fields (EV yield above 3, color above 63, high EV bits above 15, ability above 255).
        pub fn to_bytes(&self) -> Result<Vec<u8>> {
            if self.ev_yield.iter().any(|&e| e > 3) || self.unk_0a_hi > 15 || self.color > 0x3F {
                return Err(bad("EV yield, EV high bits or color out of range".into()));
            }
            let mut b = Vec::with_capacity(RECORD_LEN);
            b.extend_from_slice(&self.base_stats);
            b.extend_from_slice(&self.types);
            b.push(self.catch_rate);
            b.push(self.stage);
            let ev = self
                .ev_yield
                .iter()
                .enumerate()
                .fold(u16::from(self.unk_0a_hi) << 12, |acc, (i, &e)| {
                    acc | (u16::from(e) << (2 * i))
                });
            b.extend_from_slice(&ev.to_le_bytes());
            for item in self.held_items {
                b.extend_from_slice(&item.to_le_bytes());
            }
            b.extend_from_slice(&[
                self.gender_ratio,
                self.egg_cycles,
                self.base_friendship,
                self.growth_rate,
                self.egg_groups[0],
                self.egg_groups[1],
            ]);
            for a in self.abilities {
                b.push(u8::try_from(a).map_err(|_| bad(format!("ability {a} above 255")))?);
            }
            b.push(self.flee_rate);
            b.extend_from_slice(&self.form_stats_index.to_le_bytes());
            b.extend_from_slice(&self.form_sprite_index.to_le_bytes());
            b.push(self.form_count);
            b.push(
                self.color | (u8::from(self.unk_21_bit6) << 6) | (u8::from(self.unk_21_bit7) << 7),
            );
            b.extend_from_slice(&self.base_exp.to_le_bytes());
            b.extend_from_slice(&self.height.to_le_bytes());
            b.extend_from_slice(&self.weight.to_le_bytes());
            for w in self.tm_hm {
                b.extend_from_slice(&w.to_le_bytes());
            }
            if let Some(t) = self.tutors {
                b.extend_from_slice(&t.to_le_bytes());
            }
            Ok(b)
        }
    }

    impl PersonalTable {
        /// Decodes every member except the last (the trailer, see [`Self::trailer_from_narc`]).
        pub fn from_narc(bytes: &[u8]) -> Result<Self> {
            let narc = Narc::parse(bytes)?;
            let records = (0..narc.len().saturating_sub(1))
                .map(|i| Personal::from_bytes(narc.file(i)))
                .collect::<Result<Vec<_>>>()?;
            Ok(PersonalTable(records))
        }

        /// The last NARC member: 650 `u16`, `999` for species 0..=493 and `0..=155` for species
        /// 494..=649 (guess: index among the Gen 5 species). Meaning unverified.
        pub fn trailer_from_narc(bytes: &[u8]) -> Result<Vec<u16>> {
            let narc = Narc::parse(bytes)?;
            let last = narc
                .len()
                .checked_sub(1)
                .ok_or_else(|| bad("empty NARC".into()))?;
            let t = narc.file(last);
            if t.len() % 2 != 0 {
                return Err(bad(format!(
                    "trailer is {} bytes, not a u16 table",
                    t.len()
                )));
            }
            Ok((0..t.len() / 2).map(|i| u16_at(t, 2 * i)).collect())
        }

        /// Rebuilds the NARC from the records and the trailer.
        pub fn to_narc(&self, trailer: &[u16]) -> Result<Vec<u8>> {
            let mut files = self
                .0
                .iter()
                .map(Personal::to_bytes)
                .collect::<Result<Vec<_>>>()?;
            files.push(trailer.iter().flat_map(|v| v.to_le_bytes()).collect());
            Ok(Narc::build(&files).to_bytes())
        }
    }
}

#[cfg(test)]
mod tests;
