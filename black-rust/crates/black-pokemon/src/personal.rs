//! Personal data access (P2; `poke_personal.c`: 0x0201AD88-0x0201B2B0 and the record cache
//! 0x02019C54-0x02019C90, T3-FINDINGS.md (a)).
//!
//! A personal record is 0x3C bytes. [`field`] reads one of its 44 fields. The record of a species/form is
//! found by [`form_record_index`] and kept in a one-entry cache ([`RecordCache`], 0x02146270 in RAM).
//!
//! The stat calculation (`FUN_02018A7C`, [`recalc_stats`]) and the nature modifier are ported too; specs:
//! `tools/rom-diff/specs/black-pokemon/personal.spec`. Verified per function with
//! `tools/check-fn.sh black-pokemon personal --per-function`.

use crate::hooks::{PersonalHooks, PokeToolHooks};
use crate::poke_tool::{box_get_value, party_get_value, party_set_value};

/// Functions of this area that are ported and recorded (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_85A0,
    0x0201_8A7C,
    0x0201_9B04,
    0x0201_9C54,
    0x0201_9C7C,
    0x0201_9C90,
    0x0201_AD88,
    0x0201_ADA4,
    0x0201_ADE4,
    0x0201_AE1C,
    0x0201_AFD4,
];

/// Size of a personal record.
pub const RECORD_SIZE: usize = 0x3C;
/// ARC id of the personal-data archive (as passed to `FUN_020490F4`).
pub const PERSONAL_ARC: u32 = 0x10;
/// Species whose record index is forced to 0 by [`form_record_index`] (0x28A and 0x28B).
pub const NO_RECORD_SPECIES: [u32; 2] = [0x28A, 0x28B];

fn rd16(b: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u32
}

fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// C: `FUN_0201AE1C`: field `id` of a personal record (`rec` is at least 0x3C bytes). Ids above 0x2B
/// read as 0.
///
/// Names (inferred, offsets ROM-verified): 0-5 base stats (HP, Atk, Def, Spe, SpA, SpD), 6/7 types,
/// 8 catch rate, 9 the halfword at 0x22, 0xA-0x10 two-bit EV yields (HP, Atk, Def, Spe, SpA, SpD) and
/// one bit, 0x11-0x13 wild held items, 0x14 gender ratio, 0x15 hatch cycles, 0x16 base friendship,
/// 0x17 growth curve id, 0x18/0x19 egg groups, 0x1A/0x1B/0x1C abilities, 0x1D flee rate, 0x1E/0x1F
/// halfwords at 0x1C/0x1E, 0x20 byte, 0x21 low 6 bits, 0x22/0x23 bits 6/7 of byte 0x21, 0x24 byte 9,
/// 0x25/0x26 halfwords at 0x24/0x26, 0x27-0x2A words at 0x28-0x34 (TM/HM bits), 0x2B word at 0x38
/// (tutor bits).
pub fn field(rec: &[u8], id: u32) -> u32 {
    let b = |o: usize| rec[o] as u32;
    // EV yield bits live in the halfword at 0x0A: two bits per stat from bit 0
    let ev = rd16(rec, 0x0A);
    match id {
        0..=8 => b(id as usize),
        9 => rd16(rec, 0x22),
        0xA..=0xF => (ev >> (2 * (id - 0xA))) & 3,
        0x10 => (ev >> 12) & 1,
        0x11 => rd16(rec, 0x0C),
        0x12 => rd16(rec, 0x0E),
        0x13 => rd16(rec, 0x10),
        0x14..=0x1D => b(0x12 + (id - 0x14) as usize),
        0x1E => rd16(rec, 0x1C),
        0x1F => rd16(rec, 0x1E),
        0x20 => b(0x20),
        0x21 => b(0x21) & 0x3F,
        0x22 => (b(0x21) >> 6) & 1,
        0x23 => b(0x21) >> 7,
        0x24 => b(9),
        0x25 => rd16(rec, 0x24),
        0x26 => rd16(rec, 0x26),
        0x27..=0x2B => rd32(rec, 0x28 + 4 * (id - 0x27) as usize),
        _ => 0,
    }
}

/// C: `FUN_0201AFD4`: index of the personal record of a species/form: the species itself, or for a
/// species with alternate forms (halfword at 0x1C of its own record = first form record, byte at 0x20 =
/// form count) and `0 < form < count` (signed compare) the form's record, `(first + form - 1) & 0xFFFF`.
/// 0 for species 0x28A/0x28B (compared on the low 16 bits). `handle` is the open archive (0x0214652C).
pub fn form_record_index(species: u32, form: u32, handle: u32, h: &mut impl PersonalHooks) -> u32 {
    if (species.wrapping_add(0xFD76) & 0xFFFF) < 2 {
        return 0;
    }
    let mut rec = [0u8; RECORD_SIZE];
    let off = h.arc_member_offset(handle, species);
    h.arc_seek(handle, off);
    h.arc_read(handle, RECORD_SIZE as u32, &mut rec);
    let first = rd16(&rec, 0x1C);
    let count = rec[0x20] as i32;
    if first == 0 || count <= form as i32 || form == 0 {
        return species;
    }
    first.wrapping_add(form).wrapping_sub(1) & 0xFFFF
}

/// C: `FUN_0201AD88`: open the personal-data archive if `handle` is still 0 (the argument is passed on
/// to the opener, `FUN_020490F4(0x10, arg)`).
pub fn open_archive_once(arg: u32, handle: &mut u32, h: &mut impl PersonalHooks) {
    if *handle == 0 {
        *handle = h.open_archive(PERSONAL_ARC, arg);
    }
}

/// The record cache at 0x02146270: halfword form, halfword species, then the 0x3C-byte record.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RecordCache {
    pub form: u16,
    pub species: u16,
    pub record: [u8; RECORD_SIZE],
}

impl RecordCache {
    /// Size of the cache in RAM.
    pub const SIZE: usize = 4 + RECORD_SIZE;
    /// RAM address of the cache.
    pub const ADDR: u32 = 0x0214_6270;
    /// RAM address of the record inside it (what `FUN_02019C54` returns).
    pub const RECORD_ADDR: u32 = Self::ADDR + 4;

    /// Reads the cache from its RAM image (at least [`RecordCache::SIZE`] bytes).
    pub fn from_bytes(b: &[u8]) -> Self {
        let mut record = [0u8; RECORD_SIZE];
        record.copy_from_slice(&b[4..Self::SIZE]);
        RecordCache {
            form: rd16(b, 0) as u16,
            species: rd16(b, 2) as u16,
            record,
        }
    }

    /// Writes the cache back into its RAM image.
    pub fn write_to(&self, b: &mut [u8]) {
        b[0..2].copy_from_slice(&self.form.to_le_bytes());
        b[2..4].copy_from_slice(&self.species.to_le_bytes());
        b[4..Self::SIZE].copy_from_slice(&self.record);
    }
}

/// C: `FUN_02019C54`: make `cache.record` the record of `species`/`form`: if the pair differs from the
/// cached one (the 32-bit arguments are compared with the stored halfwords) it is stored (truncated to
/// 16 bits) and the record is loaded with `FUN_0201ADA4`. The original returns the record's address.
pub fn load_cached_record(
    cache: &mut RecordCache,
    species: u32,
    form: u32,
    h: &mut impl PersonalHooks,
) {
    if species != cache.species as u32 || form != cache.form as u32 {
        cache.form = form as u16;
        cache.species = species as u16;
        h.load_record(species, form, &mut cache.record);
    }
}

/// Line number `FUN_0201ADE4` passes to the allocator.
pub const ALLOC_RECORD_LINE: u32 = 0x76;

/// The nature table at 0x0209E324 (`poke_personal.c` data, 25 rows of 5 signed bytes, copied from
/// `main.sbin` file offset 0x9A324): row `nature`, entry `stat - 1` (stats 1 Atk, 2 Def, 3 SpA, 4 SpD,
/// 5 Spe) is 1 if the nature raises the stat, -1 if it lowers it, else 0.
pub const NATURE_TABLE: [i8; 125] = [
    0, 0, 0, 0, 0, 1, -1, 0, 0, 0, 1, 0, 0, 0, -1, 1, 0, -1, 0, 0, 1, 0, 0, -1, 0, -1, 1, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, -1, 0, 1, -1, 0, 0, 0, 1, 0, -1, 0, -1, 0, 0, 0, 1, 0, -1, 0, 0, 1,
    0, 0, 0, 0, 0, 0, 0, -1, 0, 1, 0, 0, 0, -1, 1, -1, 0, 1, 0, 0, 0, -1, 1, 0, 0, 0, 0, 1, 0, -1,
    0, 0, 0, 0, 0, 0, 0, 1, -1, 0, -1, 0, 0, 1, 0, 0, -1, 0, 1, 0, 0, 0, 0, 1, -1, 0, 0, -1, 1, 0,
    0, 0, 0, 0, 0,
];

/// C: `FUN_02019B04(nature, value, stat)`: the stat `value` scaled by the nature: 110% if the table
/// raises `stat`, 90% if it lowers it (`(value * 110 or 90) & 0xFFFF`, signed division by 100, low 16
/// bits), unchanged otherwise or for `stat > 5`.
///
/// The original indexes the table at `nature * 5 + stat - 1` without a bound: for `stat == 0` that is the
/// byte before the table (0), and past its 125 bytes it reads the shuffle table that follows it in ROM
/// (after 3 zero bytes), which is reproduced for the 128 + 128 bytes after the start of the table.
pub fn nature_modifier(nature: u32, value: u32, stat: u32) -> u32 {
    if stat >= 6 {
        return value;
    }
    let idx = nature.wrapping_mul(5).wrapping_add(stat).wrapping_sub(1);
    let entry: i8 = match idx {
        0..=124 => NATURE_TABLE[idx as usize],
        128..=255 => crate::poke_tool::SHUFFLE[(idx - 128) as usize] as i8,
        _ => 0,
    };
    let percent = match entry {
        -1 => 90,
        1 => 110,
        _ => return value,
    };
    let scaled = value.wrapping_mul(percent) & 0xFFFF;
    (scaled / 100) & 0xFFFF
}

/// C: `FUN_02019C7C(rec)`: 2 if the second ability slot (field 0x1B) is used, else 1.
pub fn ability_count(rec: &[u8]) -> u32 {
    if field(rec, 0x1B) != 0 {
        2
    } else {
        1
    }
}

/// C: `FUN_020185A0` (also reached through the thunk `FUN_02018598`): the nature of a `BoxPokemon`
/// (get id 0x70, low byte).
pub fn nature_of(mon: &mut [u8], h: &mut impl PokeToolHooks) -> u32 {
    box_get_value(mon, 0x70, 0, h) & 0xFF
}

/// C: `FUN_0201ADA4(species, form, dst)`: read the 0x3C-byte record of a species/form into `dst`:
/// seek the archive (open handle `handle`, the word at 0x0214652C) to the member of
/// [`form_record_index`] and read.
pub fn read_record(
    species: u32,
    form: u32,
    handle: u32,
    dst: &mut [u8],
    h: &mut impl PersonalHooks,
) {
    let index = form_record_index(species, form, handle, h);
    let offset = h.arc_member_offset(handle, index);
    h.arc_seek(handle, offset);
    h.arc_read(handle, RECORD_SIZE as u32, dst);
}

/// C: `FUN_0201ADE4(species, form, heap)`: allocate a 0x3C-byte block on `heap` (alignment 0,
/// `poke_personal.c` line 0x76) and read the record into it with [`read_record`]. `block` is the memory
/// of the block the allocator returns; returns its handle. An allocation failure is not modelled (the
/// original would read into address 0).
pub fn alloc_record(
    species: u32,
    form: u32,
    heap: u32,
    handle: u32,
    block: &mut [u8],
    h: &mut impl PersonalHooks,
) -> u32 {
    let addr = h.heap_alloc(heap, RECORD_SIZE as u32, 0, ALLOC_RECORD_LINE);
    assert_ne!(addr, 0, "allocation failure is not modelled");
    read_record(species, form, handle, block, h);
    addr
}

/// C: `FUN_02019C90(species, form, field_id)`: field `field_id` of the personal record of a
/// species/form, through the record cache ([`load_cached_record`]).
pub fn cached_field(
    cache: &mut RecordCache,
    species: u32,
    form: u32,
    field_id: u32,
    h: &mut impl PersonalHooks,
) -> u32 {
    load_cached_record(cache, species, form, h);
    field(&cache.record, field_id)
}

fn sdiv(a: i32, b: i32) -> i32 {
    a.wrapping_div(b)
}

/// `(2 * base + iv + ev / 4) * level / 100`, the part of a stat that [`recalc_stats`] adds `level + 10`
/// (HP) or 5 (the others) to.
fn stat_core(base: u8, iv: u32, ev: u32, level: u32) -> u32 {
    let ev4 = sdiv(ev as i32, 4);
    let sum = ev4.wrapping_add((iv as i32).wrapping_add(2 * base as i32));
    sdiv(sum.wrapping_mul(level as i32), 100) as u32
}

/// C: `FUN_02018A7C(mon)`: recalculate the stats of a `Pokemon` from its level, base stats (the record
/// of its species and form through the cache `cache`), EVs, IVs and nature, and store them (ids 0xA1
/// max HP and 0xA2-0xA6 Atk, Def, Spe, SpA, SpD):
///
/// - HP = `(2 * base + iv + ev / 4) * level / 100 + level + 10`, 1 for species 0x124 (Shedinja);
/// - other = `((2 * base + iv + ev / 4) * level / 100 + 5)` through [`nature_modifier`] (the low 16
///   bits of the sum).
///
/// The current HP (id 0xA0) shifts by the change of max HP: 0 stays 0 (unless max HP was 0 too, then it
/// becomes the new max HP), a lowered max HP caps it, Shedinja has 1. The current HP is not written
/// when it ends up 0.
pub fn recalc_stats(
    mon: &mut [u8],
    cache: &mut RecordCache,
    h: &mut (impl PokeToolHooks + PersonalHooks),
) {
    fn get(mon: &mut [u8], id: u32, h: &mut dyn PokeToolHooks) -> u32 {
        party_get_value(mon, id, 0, h)
    }
    let level = get(mon, 0x9E, h);
    let old_max = get(mon, 0xA1, h);
    let mut cur = get(mon, 0xA0, h);
    let iv_hp = get(mon, 0x46, h);
    let ev_hp = get(mon, 0xD, h);
    let iv_atk = get(mon, 0x47, h);
    let ev_atk = get(mon, 0xE, h);
    let iv_def = get(mon, 0x48, h);
    let ev_def = get(mon, 0xF, h);
    let iv_spe = get(mon, 0x49, h);
    let ev_spe = get(mon, 0x10, h);
    let iv_spa = get(mon, 0x4A, h);
    let ev_spa = get(mon, 0x11, h);
    let iv_spd = get(mon, 0x4B, h);
    let ev_spd = get(mon, 0x12, h);
    let form = get(mon, 0x6F, h);
    let species = get(mon, 5, h);
    load_cached_record(cache, species & 0xFFFF, form & 0xFFFF, h);
    let base: [u8; 6] = cache.record[..6].try_into().unwrap();
    let new_max = if species == 0x124 {
        1
    } else {
        level
            .wrapping_add(stat_core(base[0], iv_hp, ev_hp, level))
            .wrapping_add(10)
    };
    party_set_value(mon, 0xA1, new_max, h);
    let stats = [
        (0xA2, 1, base[1], iv_atk, ev_atk),
        (0xA3, 2, base[2], iv_def, ev_def),
        (0xA4, 5, base[3], iv_spe, ev_spe),
        (0xA5, 3, base[4], iv_spa, ev_spa),
        (0xA6, 4, base[5], iv_spd, ev_spd),
    ];
    for (id, nature_stat, b, iv, ev) in stats {
        let raw = stat_core(b, iv, ev, level).wrapping_add(5) & 0xFFFF;
        let nature = nature_of(mon, h);
        let v = nature_modifier(nature, raw, nature_stat);
        party_set_value(mon, id, v, h);
    }
    if cur == 0 && old_max != 0 {
        // a fainted Pokemon stays fainted
    } else if species == 0x124 {
        cur = 1;
    } else if cur == 0 {
        cur = new_max;
    } else {
        let d = (new_max as i32).wrapping_sub(old_max as i32);
        if d >= 0 {
            cur = cur.wrapping_add(d as u32);
        } else if (cur as i32) > new_max as i32 {
            cur = new_max;
        }
    }
    if cur != 0 {
        party_set_value(mon, 0xA0, cur, h);
    }
}

#[cfg(test)]
mod tests;
