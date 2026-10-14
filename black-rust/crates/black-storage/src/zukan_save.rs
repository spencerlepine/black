//! The Pokedex save block 55 (S5b; 0x4D4 bytes), main ARM9 0x0200C720..0x0200D414 (`zukan_savedata.c`;
//! callers: overlay 21 scripts, the Pokedex overlays 206..212, battle and the evolution / egg demos).
//! Verified per function with `tools/check-fn.sh black-storage zukan_save` (spec
//! `tools/rom-diff/specs/black-storage/zukan_save.spec`).
//!
//! Layout: +0x000 `u32` magic 0xBEEFCAFE, +0x004 `u32` flags (bit 0 national Pokedex; bits 1..10,
//! bit 11 and bits 12..21 are three more fields, UNVERIFIED roles), +0x008 caught bits (0x54 bytes,
//! bit `species - 1`), +0x05C / +0x0B0 / +0x104 / +0x158 seen bits (male, female, shiny male, shiny
//! female), +0x1AC / +0x200 / +0x254 / +0x2A8 the gender / colour shown on the Pokedex page (same
//! order; one bit per species), +0x2FC / +0x305 form seen bits (normal / shiny; 9 bytes each, one
//! bit per form of the species of [`FORMS`]), +0x30E / +0x317 the form shown, +0x320 language bits
//! (species 1..=493 x 7 languages), +0x4D0 `u32` the personality of the first Spinda.
//!
//! Gender values: 0 male, 1 female, 2 genderless (treated as male for the bits). Shiny: 1 = shiny.
//! Species outside 1..=649 are rejected by the checks that call [`Zukan::is_invalid_species`].
//!
//! Not ported: `FUN_0200CA10` / `FUN_0200CA1C` (`FUN_020071CC(sav, 55)` directly / through the game
//! data, ownership).

use serde::{Deserialize, Serialize};

use crate::le::{r32, w32};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_C720,
    0x0200_C738,
    0x0200_C780,
    0x0200_C7A0,
    0x0200_C7C0,
    0x0200_C7F4,
    0x0200_C818,
    0x0200_C914,
    0x0200_C9EC,
    0x0200_C9F4,
    0x0200_CA28,
    0x0200_CA38,
    0x0200_CA40,
    0x0200_CA54,
    0x0200_CA5C,
    0x0200_CA70,
    0x0200_CA78,
    0x0200_CA8C,
    0x0200_CA98,
    0x0200_CC20,
    0x0200_CC40,
    0x0200_CDE0,
    0x0200_CED4,
    0x0200_CF08,
    0x0200_CF28,
    0x0200_CF54,
    0x0200_CFA0,
    0x0200_D064,
    0x0200_D0AC,
    0x0200_D0CC,
    0x0200_D0F8,
    0x0200_D144,
    0x0200_D288,
    0x0200_D298,
    0x0200_D2B0,
    0x0200_D2C4,
    0x0200_D2D8,
    0x0200_D310,
    0x0200_D368,
    0x0200_D3C0,
];

/// Save block number.
pub const BLOCK_ID: u32 = 55;
/// Bytes of the block.
pub const SIZE: usize = 0x4D4;
/// The magic at +0.
pub const MAGIC: u32 = 0xBEEF_CAFE;
/// Bytes of a species bit set.
pub const BITS_SIZE: usize = 0x54;
/// Bytes of a form bit set.
pub const FORM_BITS_SIZE: usize = 9;
/// Bytes of the language bits.
pub const LANG_SIZE: usize = 0x1B0;
/// Highest species number.
pub const MAX_SPECIES: u32 = 649;
/// Highest species with language bits.
pub const MAX_LANG_SPECIES: u32 = 493;
/// Spinda (its personality is kept at +0x4D0).
pub const SPINDA: u32 = 327;
/// A regional Pokedex number meaning "not in the regional Pokedex".
pub const NOT_REGIONAL: u16 = 999;
/// The caught count (outside [`NOT_NATIONAL`]) that completes the national Pokedex.
pub const NATIONAL_COMPLETE: u32 = 634;
/// The regional count that completes the regional Pokedex.
pub const REGIONAL_COMPLETE: u32 = 150;

/// Species not counted for national completion (rodata 0x0209DE2C).
pub const NOT_NATIONAL: [u16; 15] = [
    151, 251, 385, 386, 489, 490, 491, 492, 493, 645, 646, 647, 648, 649, 494,
];
/// Species not counted for regional completion (rodata 0x0209DE20).
pub const NOT_REGIONAL_COUNT: [u16; 6] = [645, 646, 647, 648, 649, 494];
/// Species with forms and their form counts (rodata 0x0209DE4C; the bit of form `f` of the
/// `k`-th species is the sum of the earlier counts plus `f`).
pub const FORMS: [(u16, u16); 16] = [
    (201, 28),
    (386, 4),
    (492, 2),
    (487, 2),
    (479, 6),
    (422, 2),
    (423, 2),
    (412, 3),
    (413, 3),
    (351, 4),
    (421, 2),
    (585, 4),
    (586, 4),
    (648, 2),
    (555, 2),
    (550, 2),
];

/// The Pokemon, personal-data and heap callees.
pub trait ZukanHooks {
    /// C: `FUN_02017D30(mon)`: open the Pokemon's data for fast reads (returns the old state).
    fn lock(&mut self, mon: u32) -> u32;
    /// C: `FUN_02017D70(mon, state)`: restore it.
    fn unlock(&mut self, mon: u32, state: u32);
    /// C: `FUN_02017E1C(mon, id, 0)`: a party Pokemon parameter.
    fn param(&mut self, mon: u32, id: u32) -> u32;
    /// C: `FUN_02017F00(mon)`: its gender (0, 1, 2).
    fn gender(&mut self, mon: u32) -> u32;
    /// C: `FUN_02017EAC(mon)`: 1 when shiny.
    fn shiny(&mut self, mon: u32) -> u32;
    /// C: `FUN_0201AF80(heap, 0)`: the regional Pokedex numbers (indexed by species, 999 = not
    /// regional), allocated on `heap`; the table and its address.
    fn regional_table(&mut self, heap: u32) -> (Vec<u16>, u32);
    /// C: `FUN_020307B0(p)` (`Heap_Free`).
    fn free(&mut self, p: u32);
    /// C: `FUN_0201ADE4(species, form, heap)`: open the personal data.
    fn personal_open(&mut self, species: u32, form: u32, heap: u32) -> u32;
    /// C: `FUN_0201AE1C(p, id)`: a personal-data value (20 = gender ratio, 255 = genderless).
    fn personal_param(&mut self, p: u32, id: u32) -> u32;
    /// C: `FUN_0201AE14(p)`: close it.
    fn personal_close(&mut self, p: u32);
}

fn bit_of(i: u32) -> (usize, u8) {
    let i = (i & 0xFFFF) as usize;
    (i >> 3, 1 << (i & 7))
}

fn test_bit(b: &[u8], i: u32) -> bool {
    let (k, m) = bit_of(i);
    b[k] & m != 0
}

fn set_bit(b: &mut [u8], i: u32) {
    let (k, m) = bit_of(i);
    b[k] |= m;
}

fn clear_bit(b: &mut [u8], i: u32) {
    let (k, m) = bit_of(i);
    b[k] &= !m;
}

/// The (gender, shiny) bit set index: male 0, female 1, shiny male 2, shiny female 3 (any
/// gender but 1 counts as male; shiny only when exactly 1).
fn kind_index(gender: u32, shiny: u32) -> usize {
    (gender == 1) as usize + 2 * (shiny == 1) as usize
}

/// Save block 55.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Zukan {
    /// +0x000.
    pub magic: u32,
    /// +0x004.
    pub flags: u32,
    /// +0x008: caught bits.
    #[serde(with = "crate::save::hex")]
    pub caught: Vec<u8>,
    /// +0x05C / +0x0B0 / +0x104 / +0x158: seen bits by [`kind_index`].
    pub seen: Vec<Bits>,
    /// +0x1AC / +0x200 / +0x254 / +0x2A8: the kind shown.
    pub shown: Vec<Bits>,
    /// +0x2FC / +0x305: forms seen (normal, shiny).
    pub form_seen: [[u8; FORM_BITS_SIZE]; 2],
    /// +0x30E / +0x317: the form shown (normal, shiny).
    pub form_shown: [[u8; FORM_BITS_SIZE]; 2],
    /// +0x320: language bits.
    #[serde(with = "crate::save::hex")]
    pub lang: Vec<u8>,
    /// +0x4D0: the first Spinda's personality.
    pub spinda_pid: u32,
}

/// One 0x54-byte species bit set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bits(#[serde(with = "crate::save::hex")] pub Vec<u8>);

impl Default for Zukan {
    fn default() -> Self {
        Zukan::from_bytes(&[0; SIZE])
    }
}

impl Zukan {
    /// Reads the original 0x4D4-byte image.
    pub fn from_bytes(b: &[u8]) -> Self {
        let bits = |o: usize| Bits(b[o..o + BITS_SIZE].to_vec());
        let form =
            |o: usize| -> [u8; FORM_BITS_SIZE] { b[o..o + FORM_BITS_SIZE].try_into().unwrap() };
        Zukan {
            magic: r32(b, 0),
            flags: r32(b, 4),
            caught: b[8..8 + BITS_SIZE].to_vec(),
            seen: (0..4).map(|k| bits(0x5C + BITS_SIZE * k)).collect(),
            shown: (0..4).map(|k| bits(0x1AC + BITS_SIZE * k)).collect(),
            form_seen: [form(0x2FC), form(0x305)],
            form_shown: [form(0x30E), form(0x317)],
            lang: b[0x320..0x320 + LANG_SIZE].to_vec(),
            spinda_pid: r32(b, 0x4D0),
        }
    }

    /// The original 0x4D4-byte image.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = vec![0u8; SIZE];
        w32(&mut b, 0, self.magic);
        w32(&mut b, 4, self.flags);
        b[8..8 + BITS_SIZE].copy_from_slice(&self.caught);
        for k in 0..4 {
            let o = 0x5C + BITS_SIZE * k;
            b[o..o + BITS_SIZE].copy_from_slice(&self.seen[k].0);
            let o = 0x1AC + BITS_SIZE * k;
            b[o..o + BITS_SIZE].copy_from_slice(&self.shown[k].0);
        }
        b[0x2FC..0x305].copy_from_slice(&self.form_seen[0]);
        b[0x305..0x30E].copy_from_slice(&self.form_seen[1]);
        b[0x30E..0x317].copy_from_slice(&self.form_shown[0]);
        b[0x317..0x320].copy_from_slice(&self.form_shown[1]);
        b[0x320..0x320 + LANG_SIZE].copy_from_slice(&self.lang);
        w32(&mut b, 0x4D0, self.spinda_pid);
        b
    }

    /// C: `FUN_0200C720(species)`: 0 or above 649.
    pub fn is_invalid_species(species: u32) -> bool {
        species == 0 || species > MAX_SPECIES
    }

    /// The language bit of `(species, lang)`, `None` for a language without one (above 8, or 6)
    /// or a species above 493. Quirk kept: language 0 passes the checks and gives bit
    /// `7 * (species - 1) - 1`, the last bit of the previous species; with species 0 or 1 that
    /// bit is past the set (the original reaches outside the block): those panic here.
    fn lang_bit(species: u32, lang: u32) -> Option<u32> {
        if lang > 8 || lang == 6 || species > MAX_LANG_SPECIES {
            return None;
        }
        let l = if lang >= 7 { lang - 1 } else { lang };
        Some((species.wrapping_sub(1).wrapping_mul(7)).wrapping_add(l.wrapping_sub(1)) & 0xFFFF)
    }

    /// C: unlabeled routine at `0x0200C738` (`(zukan, species, lang)`): set the language bit
    /// (languages 1..=8 except 6; 7 and 8 use bits 6 and 7).
    pub fn set_lang(&mut self, species: u32, lang: u32) {
        if let Some(i) = Self::lang_bit(species, lang) {
            set_bit(&mut self.lang, i);
        }
    }

    /// C: unlabeled routine at `0x0200D3C0` (`(zukan, species, lang)`): the
    /// language bit (0 for a language or species without one).
    pub fn lang_seen(&self, species: u32, lang: u32) -> bool {
        Self::lang_bit(species, lang).is_some_and(|i| test_bit(&self.lang, i))
    }

    /// C: `FUN_0200c780(species)`: 1 unless the species is one of [`NOT_NATIONAL`].
    pub fn counts_national(species: u32) -> bool {
        !NOT_NATIONAL.iter().any(|&s| s as u32 == species)
    }

    /// C: `FUN_0200c7a0(species)`: 1 unless the species is one of [`NOT_REGIONAL_COUNT`].
    pub fn counts_regional(species: u32) -> bool {
        !NOT_REGIONAL_COUNT.iter().any(|&s| s as u32 == species)
    }

    /// C: `FUN_0200C7C0(species)`: the first form bit of the species, -1 without forms.
    pub fn form_start(species: u32) -> u32 {
        let mut start = 0u32;
        for &(s, n) in &FORMS {
            if s as u32 == species {
                return start;
            }
            start += n as u32;
        }
        u32::MAX
    }

    /// C: `FUN_0200c7f4(species)`: the index of the species in [`FORMS`], -1 if absent.
    pub fn form_index(species: u32) -> u32 {
        FORMS
            .iter()
            .position(|&(s, _)| s as u32 == species)
            .map_or(u32::MAX, |i| i as u32)
    }

    /// C: `FUN_0200CC20(species)`: the number of forms (1 without forms).
    pub fn form_count(species: u32) -> u32 {
        match Self::form_index(species) {
            u32::MAX => 1,
            i => FORMS[i as usize].1 as u32,
        }
    }

    /// C: unlabeled routine at `0x0200C818` (`(zukan, species, gender, shiny)`): set the seen bit.
    pub fn set_seen(&mut self, species: u32, gender: u32, shiny: u32) {
        set_bit(
            &mut self.seen[kind_index(gender, shiny)].0,
            species.wrapping_sub(1),
        );
    }

    /// C: `FUN_0200c914(zukan, species, _, shiny, form)`: set the form-seen bits: Castform all
    /// four, Cherrim / Darmanitan / Meloetta both, others form `form` (form 0 when out of range).
    pub fn set_form_seen(&mut self, species: u32, shiny: u32, form: u32) {
        let start = Self::form_start(species & 0xFFFF);
        if start == u32::MAX {
            return;
        }
        let (form, n) = match species {
            351 => (0, 4),
            421 | 555 | 648 => (0, 2),
            _ => (
                if form < Self::form_count(species & 0xFFFF) {
                    form
                } else {
                    0
                },
                1,
            ),
        };
        let set = &mut self.form_seen[(shiny == 1) as usize];
        for j in 0..n {
            set_bit(set, start.wrapping_add(form).wrapping_add(j));
        }
    }

    /// C: `sub_0200C9EC`: block size.
    pub fn size() -> u32 {
        SIZE as u32
    }

    /// C: `sub_0200c9f4`: zero, magic.
    pub fn init(&mut self) {
        *self = Zukan::default();
        self.magic = MAGIC;
    }

    /// C: unlabeled routine at `0x0200CA28`: flags bit 0 = 1 (national Pokedex).
    pub fn set_national(&mut self) {
        self.flags |= 1;
    }

    /// C: `FUN_0200CA38`: flags bit 0.
    pub fn national(&self) -> u32 {
        self.flags & 1
    }

    /// C: `FUN_0200ca40`: flags bits 1..10 = 1.
    pub fn set_field_1(&mut self) {
        self.flags = (self.flags & 0xFFFF_F801) | 2;
    }

    /// C: unlabeled routine at `0x0200CA54`: flags bits 1..10.
    pub fn field_1(&self) -> u32 {
        (self.flags << 21) >> 22
    }

    /// C: unlabeled routine at `0x0200CA5C` (`(zukan, v)`): flags bit 11 = bit 0 of `v`.
    pub fn set_bit_11(&mut self, v: u32) {
        self.flags = (self.flags & 0xFFFF_F7FF) | ((v & 1) << 11);
    }

    /// C: unlabeled routine at `0x0200CA70`: flags bit 11.
    pub fn bit_11(&self) -> u32 {
        (self.flags >> 11) & 1
    }

    /// C: unlabeled routine at `0x0200CA78` (`(zukan, v)`): flags bits 12..21 = `v` (10 bits).
    pub fn set_field_12(&mut self, v: u32) {
        self.flags = (self.flags & 0xFFC0_0FFF) | ((v & 0x3FF) << 12);
    }

    /// C: unlabeled routine at `0x0200CA8C`: flags bits 12..21.
    pub fn field_12(&self) -> u16 {
        ((self.flags << 10) >> 22) as u16
    }

    /// C: `FUN_0200CA98(zukan, species, gender, shiny, form)`: make `(gender, shiny, form)` the
    /// one shown: a species with forms gets only form `form` (form 0 when out of range) in the
    /// normal or shiny shown-form bits; every species gets only its `(gender, shiny)` shown bit.
    pub fn set_shown(&mut self, species: u32, gender: u32, shiny: u32, form: u32) {
        let mut start = 0u32;
        for &(s, n) in &FORMS {
            if s as u32 == species {
                let n = n as u32;
                for j in 0..n {
                    clear_bit(&mut self.form_shown[0], start + j);
                    clear_bit(&mut self.form_shown[1], start + j);
                }
                let f = if form < n { form } else { 0 };
                set_bit(
                    &mut self.form_shown[(shiny == 1) as usize],
                    start.wrapping_add(f),
                );
                break;
            }
            start += n as u32;
        }
        let i = species.wrapping_sub(1);
        for s in &mut self.shown {
            clear_bit(&mut s.0, i);
        }
        set_bit(&mut self.shown[kind_index(gender, shiny)].0, i);
    }

    /// C: `FUN_0200CC40(zukan, species, &gender, &shiny, &form, heap)`: the `(gender, shiny,
    /// form)` shown. Starting from `cur` (the values the caller's variables hold; each is written
    /// only where a bit says so), a genderless species (personal value 20 = 255) answers gender 2.
    pub fn shown_kind<H: ZukanHooks>(
        &self,
        species: u32,
        heap: u32,
        cur: [u32; 3],
        h: &mut H,
    ) -> [u32; 3] {
        let [mut gender, mut shiny, mut form] = cur;
        let i = species.wrapping_sub(1);
        let sh = |k: usize| test_bit(&self.shown[k].0, i);
        match Self::form_index(species) {
            u32::MAX => {
                form = 0;
                if sh(0) {
                    gender = 0;
                    shiny = 0;
                }
                if sh(1) {
                    gender = 1;
                    shiny = 0;
                }
                if sh(2) {
                    gender = 0;
                    shiny = 1;
                }
                if sh(3) {
                    gender = 1;
                    shiny = 1;
                }
            }
            idx => {
                let start = Self::form_start(species);
                for j in 0..FORMS[idx as usize].1 as u32 {
                    let b = start.wrapping_add(j);
                    if test_bit(&self.form_shown[0], b) {
                        form = j;
                        shiny = 0;
                        break;
                    }
                    if test_bit(&self.form_shown[1], b) {
                        form = j;
                        shiny = 1;
                        break;
                    }
                }
                if sh(0) || sh(2) {
                    gender = 0;
                }
                if sh(1) || sh(3) {
                    gender = 1;
                }
            }
        }
        let p = h.personal_open(species, form & 0xFFFF, heap & 0xFFFF);
        if h.personal_param(p, 20) == 255 {
            gender = 2;
        }
        h.personal_close(p);
        [gender, shiny, form]
    }

    /// The common part of [`Zukan::register_caught`] / [`Zukan::register_seen`]: read the
    /// Pokemon (eggs are skipped: `None`).
    fn read_mon<H: ZukanHooks>(mon: u32, caught: bool, h: &mut H) -> Option<[u32; 6]> {
        let state = h.lock(mon);
        if h.param(mon, 76) == 1 {
            h.unlock(mon, state);
            return None;
        }
        let species = h.param(mon, 5) & 0xFFFF;
        let pid = h.param(mon, 0);
        let form = h.param(mon, 111);
        let lang = if caught { h.param(mon, 12) } else { 0 };
        let gender = h.gender(mon);
        let shiny = h.shiny(mon);
        h.unlock(mon, state);
        Some([species, pid, form, lang, gender, shiny])
    }

    fn register(&mut self, m: [u32; 6]) {
        let [species, pid, form, _, gender, shiny] = m;
        if !self.seen(species) {
            self.set_shown(species, gender, shiny, form);
        }
        if species == SPINDA && self.spinda_pid == 0 {
            self.spinda_pid = pid;
        }
        self.set_seen(species, gender, shiny);
        self.set_form_seen(species, shiny, form);
    }

    /// C: `FUN_0200CDE0(zukan, mon)`: register a caught Pokemon (not an egg, valid species):
    /// shown kind if it was not seen yet, Spinda's personality, seen / form / language bits,
    /// caught bit.
    pub fn register_caught<H: ZukanHooks>(&mut self, mon: u32, h: &mut H) {
        let Some(m) = Self::read_mon(mon, true, h) else {
            return;
        };
        let species = m[0];
        if Self::is_invalid_species(species) {
            return;
        }
        self.register(m);
        self.set_lang(species, m[3]);
        set_bit(&mut self.caught, species - 1);
    }

    /// C: `FUN_0200CFA0(zukan, mon)`: register a seen Pokemon ([`Zukan::register_caught`] without
    /// the language and caught bits).
    pub fn register_seen<H: ZukanHooks>(&mut self, mon: u32, h: &mut H) {
        let Some(m) = Self::read_mon(mon, false, h) else {
            return;
        };
        if Self::is_invalid_species(m[0]) {
            return;
        }
        self.register(m);
    }

    /// C: `FUN_0200CED4(zukan, species)`: the caught bit (0 for an invalid species).
    pub fn caught(&self, species: u32) -> bool {
        !Self::is_invalid_species(species) && test_bit(&self.caught, species - 1)
    }

    /// C: `FUN_0200D064(zukan, species)`: any seen bit (0 for an invalid species).
    pub fn seen(&self, species: u32) -> bool {
        !Self::is_invalid_species(species) && self.seen.iter().any(|s| test_bit(&s.0, species - 1))
    }

    /// C: `FUN_0200CF28`: caught species 1..=649.
    pub fn caught_count_national(&self) -> u32 {
        (1..=MAX_SPECIES).filter(|&s| self.caught(s)).count() as u32
    }

    /// C: `FUN_0200D0CC`: seen species 1..=649.
    pub fn seen_count_national(&self) -> u32 {
        (1..=MAX_SPECIES).filter(|&s| self.seen(s)).count() as u32
    }

    fn regional_count<H: ZukanHooks>(
        &self,
        heap: u32,
        h: &mut H,
        f: impl Fn(&Self, u32) -> bool,
    ) -> u32 {
        let (table, p) = h.regional_table(heap);
        let n = (1..=MAX_SPECIES)
            .filter(|&s| f(self, s) && table[s as usize] != NOT_REGIONAL)
            .count() as u32;
        h.free(p);
        n
    }

    /// C: `FUN_0200CF54(zukan, heap)`: caught species in the regional Pokedex.
    pub fn caught_count_regional<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> u32 {
        self.regional_count(heap, h, |z, s| z.caught(s))
    }

    /// C: `FUN_0200D0F8(zukan, heap)`: seen species in the regional Pokedex.
    pub fn seen_count_regional<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> u32 {
        self.regional_count(heap, h, |z, s| z.seen(s))
    }

    /// C: `FUN_0200CF08(zukan, heap)`: the caught count of the Pokedex in use.
    pub fn caught_count<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> u32 {
        if self.national() != 0 {
            self.caught_count_national()
        } else {
            self.caught_count_regional(heap, h)
        }
    }

    /// C: unlabeled routine at `0x0200D0AC` (`(zukan, heap)`): the seen count of the Pokedex in use.
    pub fn seen_count<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> u32 {
        if self.national() != 0 {
            self.seen_count_national()
        } else {
            self.seen_count_regional(heap, h)
        }
    }

    /// C: unlabeled routine at `0x0200D144` (`(zukan, species, gender,
    /// shiny, form)`): seen with this kind: a species with forms tests its form bit (normal /
    /// shiny set by `shiny`, which must be 0 or 1); others test the `(gender, shiny)` seen bit,
    /// a gender other than 0 / 1 either gender's. -1 for an invalid species.
    pub fn seen_kind(&self, species: u32, gender: u32, shiny: u32, form: u32) -> u32 {
        if Self::is_invalid_species(species) {
            return u32::MAX;
        }
        let start = Self::form_start(species);
        if start != u32::MAX {
            return test_bit(&self.form_seen[shiny as usize], start.wrapping_add(form)) as u32;
        }
        let i = species - 1;
        let t = |k: usize| test_bit(&self.seen[k].0, i);
        let s = 2 * (shiny == 1) as usize;
        (match gender {
            0 => t(s),
            1 => t(s + 1),
            _ => t(s) || t(s + 1),
        }) as u32
    }

    /// C: unlabeled routine at `0x0200D288` (`(zukan, k)`): the Spinda personality for `k == 0`,
    /// else 0.
    pub fn spinda(&self, k: u32) -> u32 {
        if k == 0 {
            self.spinda_pid
        } else {
            0
        }
    }

    /// C: `FUN_0200d2d8(zukan)`: caught species counted for national completion.
    pub fn national_caught(&self) -> u32 {
        (1..=MAX_SPECIES)
            .filter(|&s| self.caught(s) && Self::counts_national(s))
            .count() as u32
    }

    /// C: `FUN_0200D298(zukan)`: at least 634 of those.
    pub fn national_complete(&self) -> bool {
        self.national_caught() >= NATIONAL_COMPLETE
    }

    /// C: `FUN_0200d310(zukan, heap)`: caught regional species counted for completion.
    pub fn regional_caught<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> u32 {
        self.regional_count(heap, h, |z, s| z.caught(s) && Self::counts_regional(s))
    }

    /// C: `FUN_0200d368(zukan, heap)`: seen regional species counted for completion.
    pub fn regional_seen<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> u32 {
        self.regional_count(heap, h, |z, s| z.seen(s) && Self::counts_regional(s))
    }

    /// C: `FUN_0200D2B0(zukan, heap)`: at least 150 regional caught.
    pub fn regional_caught_complete<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> bool {
        self.regional_caught(heap, h) >= REGIONAL_COMPLETE
    }

    /// C: `FUN_0200D2C4(zukan, heap)`: at least 150 regional seen.
    pub fn regional_seen_complete<H: ZukanHooks>(&self, heap: u32, h: &mut H) -> bool {
        self.regional_seen(heap, h) >= REGIONAL_COMPLETE
    }
}

#[cfg(test)]
mod tests;
