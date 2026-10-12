//! EXP growth curves and level-from-EXP (P3; `unk_02019A50.s` loaders 0x02019B94/0x02019BA4 and the
//! level routines 0x0201850C/0x02018584, T3-FINDINGS.md (a)).
//!
//! The curve of a species is its personal-data field 0x17; the table for a curve is ARC 17 member
//! `curve` (101 little-endian `u32`: EXP needed to *be* level 0..=100). The original loads it into one
//! shared buffer (0x02146398) on every call; the port takes that buffer as `table` and leaves the loaded
//! curve in it, as the original does.
//!
//! Verified per function with `tools/check-fn.sh black-pokemon growth --per-function`
//! (spec `tools/rom-diff/specs/black-pokemon/growth.spec`). The EXP-gain formula is not in this area
//! (see the open questions of T3).

use crate::hooks::GrowthHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0201_8584, 0x0201_850C, 0x0201_9B94, 0x0201_9BA4];

/// Entries of a growth table (levels 0..=100).
pub const LEVELS: usize = 101;
/// ARC id of the growth tables.
pub const GROWTH_ARC: u32 = 0x11;
/// Personal-data field holding the growth curve id.
pub const PERSONAL_GROWTH_FIELD: u32 = 0x17;

/// C: `FUN_02019B94`: load growth curve `curve` into the table buffer (tail call of `ArcLoadMember`).
pub fn load_curve(curve: u32, table: &mut [u32; LEVELS], h: &mut impl GrowthHooks) {
    h.load_growth_curve(curve, table);
}

/// C: `FUN_02019BA4`: load growth curve `curve` and return the EXP needed for `level`.
///
/// The original indexes the buffer with the raw 32-bit `level` and reads past the table for 101 and up;
/// the port panics there (callers pass levels 0..=100).
pub fn exp_for_level(
    curve: u32,
    level: u32,
    table: &mut [u32; LEVELS],
    h: &mut impl GrowthHooks,
) -> u32 {
    load_curve(curve, table, h);
    table[level as usize]
}

/// C: `FUN_0201850C`: level of a Pokemon of `species`/`form` with `exp` experience points: the number
/// of levels 1..=100 whose table entry is `<= exp`, i.e. `(first level with exp < table[level]) - 1`,
/// at most 100.
pub fn level_from_exp(
    species: u32,
    form: u32,
    exp: u32,
    table: &mut [u32; LEVELS],
    h: &mut impl GrowthHooks,
) -> u32 {
    let rec = h.personal_record(species, form);
    // the original truncates the field to 16 bits before using it as the member index
    let curve = h.personal_field(rec, PERSONAL_GROWTH_FIELD) & 0xFFFF;
    load_curve(curve, table, h);
    let mut level = 1usize;
    loop {
        if exp < table[level] {
            break;
        }
        level += 1;
        if level >= LEVELS {
            break;
        }
    }
    level as u32 - 1
}

/// C: `FUN_02018584`: EXP needed for `level` by a Pokemon of `species`/`form` (its personal growth
/// curve, then [`exp_for_level`]).
pub fn exp_for_species_level(
    species: u32,
    form: u32,
    level: u32,
    table: &mut [u32; LEVELS],
    h: &mut impl GrowthHooks,
) -> u32 {
    let curve = h.personal_param(species, form, PERSONAL_GROWTH_FIELD);
    exp_for_level(curve, level, table, h)
}

#[cfg(test)]
mod tests;
