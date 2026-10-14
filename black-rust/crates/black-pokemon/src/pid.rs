//! PID, gender, shininess and ability slot (P6; `poke_tool.c` 0x02017C50-0x0201868C-0x02018790).
//!
//! The gender of a Pokemon is decided by its PID against the species' gender ratio (personal field
//! 0x14); the ability slot is bit 16 (origin game 20 and later) or bit 0 (earlier) of the PID; a
//! Pokemon is shiny when the four halfwords of its trainer ID and PID xor to less than 8. The nature
//! is not derived from the PID in this game: it is a stored field (`FUN_020185A0`, id 0x70, in
//! [`crate::personal::nature_of`]; `rand(25)` at creation in [`crate::create::init_core`]).
//!
//! Callees outside this area (personal data, the RNG, the `poke_tool` accessors) come in through
//! [`PidHooks`]; the accessors `FUN_02017E88`/`FUN_02017E64` run as real code (`poke_tool`). In-area
//! callees (`FUN_02017ED8`, `FUN_0201AF6C`, `FUN_020185F8`, `FUN_02017C50`, `FUN_02017CC8`) are called
//! directly; `FUN_02017F50` is reached through [`PokeToolHooks::gender`] (as `poke_tool`'s put of id 0x6E
//! does), so the ROM differential test keeps it stubbed (`force`) where it is called from a wrapper.

use crate::hooks::PokeToolHooks;
use crate::poke_tool::{box_get_value, box_set_value, decrypt_on_access, reencrypt};

pub use crate::create::is_shiny;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_79B8,
    0x0201_7C50,
    0x0201_7CC8,
    0x0201_7EB4,
    0x0201_7ED8,
    0x0201_7F08,
    0x0201_7F50,
    0x0201_85F8,
    0x0201_862C,
    0x0201_86C8,
    0x0201_AF6C,
];

/// Callees of the PID routines that belong to other areas. The ROM differential test stubs them
/// identically on both sides, so only their arguments are verified here.
pub trait PidHooks: PokeToolHooks {
    /// Callee `FUN_02019C54(species, form)`: the personal-data record of a species/form (an opaque
    /// handle for [`PidHooks::personal_field`]).
    fn personal_record(&mut self, species: u32, form: u32) -> u32;

    /// Callee `FUN_0201AE1C(rec, field)`: field `field` of a personal record (0x14 = gender ratio,
    /// 0x1A/0x1B/0x1C = abilities 1, 2 and 3).
    fn personal_field(&mut self, rec: u32, field: u32) -> u32;

    /// Callee `FUN_02019C7C(rec)`: 2 if the record has a second ability, else 1.
    fn ability_count(&mut self, rec: u32) -> u32;

    /// Callee `FUN_020056EC(max)`: random number below `max` (0: the full 32-bit range).
    fn rand(&mut self, max: u32) -> u32;
}

/// Personal field of the gender ratio.
const FIELD_GENDER_RATIO: u32 = 0x14;

/// C: `FUN_0201AF6C(ratio)`: whether a gender ratio is fixed: 0 (always male), 0xFE (always female) or
/// 0xFF (genderless).
pub fn is_fixed_gender(ratio: u32) -> bool {
    ratio == 0 || ratio.wrapping_sub(0xFE) <= 1
}

/// C: `FUN_02017F50(species, form, pid)`: the gender a PID gives a species: 0 male, 1 female, 2
/// genderless. Ratio 0 is always male, 0xFE always female, 0xFF genderless; otherwise female when the
/// low byte of the PID is below the ratio.
pub fn gender_from_ratio(species: u32, form: u32, pid: u32, h: &mut impl PidHooks) -> u32 {
    let rec = h.personal_record(species, form);
    let ratio = h.personal_field(rec, FIELD_GENDER_RATIO) & 0xFF;
    match ratio {
        0 => 0,
        0xFE => 1,
        0xFF => 2,
        _ => ((pid & 0xFF) < ratio) as u32,
    }
}

/// C: `FUN_02017EB4(mon)`: whether the record is shiny, from its trainer ID (id 7) and PID (id 0).
pub fn is_shiny_mon(mon: &mut [u8], h: &mut impl PidHooks) -> bool {
    let ot_id = box_get_value(mon, 7, 0, h);
    let pid = box_get_value(mon, 0, 0, h);
    is_shiny(ot_id, pid)
}

/// C: `FUN_02017F08(mon)`: the gender of a record, from its species, form and PID (decrypt-on-access
/// around the reads).
pub fn gender_of(mon: &mut [u8], h: &mut impl PidHooks) -> u32 {
    let enter = decrypt_on_access(mon);
    let species = box_get_value(mon, 5, 0, h) & 0xFFFF;
    let form = box_get_value(mon, 0x6F, 0, h);
    let pid = box_get_value(mon, 0, 0, h);
    reencrypt(mon, enter as u32);
    h.gender(species, form, pid)
}

/// C: `FUN_020185F8(species, form, flag)`: the low byte of a PID for the gender ratio: 8 for a fixed
/// ratio ([`is_fixed_gender`]); otherwise the ratio, minus one if `flag` is nonzero.
pub fn gender_threshold(species: u32, form: u32, flag: u32, h: &mut impl PidHooks) -> u32 {
    let rec = h.personal_record(species, form);
    let ratio = h.personal_field(rec, FIELD_GENDER_RATIO) & 0xFF;
    if is_fixed_gender(ratio) {
        return 8;
    }
    // the original also clamps `ratio - 1` at 0, but a ratio of 0 is fixed and returned above
    if flag != 0 {
        ratio - 1
    } else {
        ratio
    }
}

/// The halfwords of the trainer ID xored: `(ot_id >> 16) ^ (ot_id & 0xFFFF)` (the original masks with
/// the constant 0xFFFF0000 loaded from the literal pool).
fn ot_xor(ot_id: u32) -> u32 {
    ((ot_id & 0xFFFF_0000) >> 16) ^ (ot_id & 0xFFFF)
}

/// C: `FUN_0201862C(ot_id, species, form, adjust, ability_bit, flag)`: a PID built without the RNG.
/// `ability_bit` is a byte (the stack word's low byte). With `flag == 0` the PID is not shiny: the high
/// halfword is `((ot xor) & 0xFF00) ^ 0xFF00` with bit 16 set to `ability_bit != 0`, and the low byte is
/// the gender ratio (minus one if `adjust == 1`) unless the ratio is fixed. With `flag != 0` it is shiny
/// for `ot_id`: the low byte is [`gender_threshold`] (`adjust` as its flag) and the high halfword is
/// `ot xor ^ low halfword`, bit 16 then flipped if it differs from `ability_bit << 16`.
pub fn pid_without_rng(
    ot_id: u32,
    species: u32,
    form: u32,
    adjust: u32,
    ability_bit: u32,
    flag: u32,
    h: &mut impl PidHooks,
) -> u32 {
    let ability = (ability_bit & 0xFF) << 16;
    if flag == 0 {
        let rec = h.personal_record(species, form);
        let ratio = h.personal_field(rec, FIELD_GENDER_RATIO) & 0xFF;
        let mut pid = ((ot_xor(ot_id) & 0xFF00) ^ 0xFF00) << 16;
        if pid & 0x10000 != ability {
            pid ^= 0x10000;
        }
        if !is_fixed_gender(ratio) {
            pid |= ratio;
            if adjust == 1 {
                pid = pid.wrapping_sub(1);
            }
        }
        pid
    } else {
        let low = gender_threshold(species, form, adjust, h);
        let mut pid = low | ((low & 0xFFFF ^ ot_xor(ot_id)) << 16);
        if pid & 0x10000 != ability {
            pid ^= 0x10000;
        }
        pid
    }
}

/// C: `FUN_020186C8(ot_id, species, form, kind, ability_bit, shiny)`: a random PID. `kind` 2 (genderless)
/// takes a random word. Otherwise the high 24 bits are random and the low byte comes from
/// [`gender_threshold`] (flag `kind & 0xFF`), spread over the gender: for `kind` 0 (male) a random byte
/// at or above the threshold (`threshold + rand(0xFE - threshold)`), for `kind` 1 (female) a random byte
/// in `1..=threshold` (`rand(threshold) + 1`), unless the ratio is fixed ([`is_fixed_gender`] of the
/// threshold). `shiny` 0 makes it not shiny (`^ 0x10000000` if it is), 1 shiny (the high halfword
/// becomes `ot xor ^ low byte`), 2 leaves it; any other value returns the PID as is. Unless that early
/// return happens, bit 16 is forced to `ability_bit` (flipped if different).
pub fn random_pid(
    ot_id: u32,
    species: u32,
    form: u32,
    kind: u32,
    ability_bit: u32,
    shiny: u32,
    h: &mut impl PidHooks,
) -> u32 {
    let mut pid;
    if kind == 2 {
        pid = h.rand(0);
    } else {
        let high = h.rand(0) & !0xFF;
        let mut low = gender_threshold(species, form, kind & 0xFF, h);
        if kind == 0 && !is_fixed_gender(low) {
            low = low.wrapping_add(h.rand(0xFEu32.wrapping_sub(low)));
        } else if kind == 1 && !is_fixed_gender(low) {
            low = h.rand(low).wrapping_add(1);
        }
        pid = low | high;
    }
    let want = ability_bit << 16;
    match shiny {
        0 => {
            if is_shiny(ot_id, pid) {
                pid ^= 0x1000_0000;
            }
        }
        1 => {
            let low = pid & 0xFF;
            pid = low | ((ot_xor(ot_id) ^ low) << 16);
            return if pid & 0x10000 == want {
                pid
            } else {
                pid ^ 0x10000
            };
        }
        2 => {}
        _ => return pid,
    }
    if pid & 0x10000 == want {
        pid
    } else {
        pid ^ 0x10000
    }
}

/// C: `FUN_02017C50(mon, species)`: set the ability slot (id 10) of a record from its PID after a form
/// change. The slot is the third ability (field 0x1C) if id 0x71 is set; otherwise the second
/// (field 0x1B) if the form has two abilities and the PID has the ability bit (bit 0 for an origin
/// game below 20, else bit 16), otherwise the first (field 0x1A).
pub fn update_ability_slot(mon: &mut [u8], species: u32, h: &mut impl PidHooks) {
    let form = box_get_value(mon, 0x6F, 0, h) & 0xFFFF;
    let rec = h.personal_record(species, form);
    let pid = box_get_value(mon, 0, 0, h);
    let origin = box_get_value(mon, 0x77, 0, h);
    let bit = if origin < 0x14 { 1 } else { 0x10000 };
    let mut field = 0x1A;
    if box_get_value(mon, 0x71, 0, h) != 0 {
        field = 0x1C;
    } else if h.ability_count(rec) == 2 && pid & bit != 0 {
        field = 0x1B;
    }
    let ability = h.personal_field(rec, field);
    box_set_value(mon, 10, ability, h);
}

/// C: `FUN_02017CC8(mon, old_species, new_species)`: recompute the stored gender (id 0x6E) after a form
/// change, unless the old ratio is fixed and the new one is not genderless (0xFF).
pub fn update_gender(mon: &mut [u8], old_species: u32, new_species: u32, h: &mut impl PidHooks) {
    let form = box_get_value(mon, 0x6F, 0, h) & 0xFFFF;
    let rec = h.personal_record(old_species, form);
    let old_ratio = h.personal_field(rec, FIELD_GENDER_RATIO);
    let rec = h.personal_record(new_species, form);
    let new_ratio = h.personal_field(rec, FIELD_GENDER_RATIO);
    if !is_fixed_gender(old_ratio) || new_ratio == 0xFF {
        let pid = box_get_value(mon, 0, 0, h);
        let gender = h.gender(old_species, form, pid);
        box_set_value(mon, 0x6E, gender, h);
    }
}

/// C: `FUN_020179B8(mon, form)`: change the form of a record (id 0x6F). If it differs, store it and
/// refresh the ability slot and gender ([`update_ability_slot`], [`update_gender`], both with the
/// record's species). Returns whether the form changed.
pub fn set_form(mon: &mut [u8], form: u32, h: &mut impl PidHooks) -> bool {
    let enter = decrypt_on_access(mon);
    let changed = box_get_value(mon, 0x6F, 0, h) & 0xFFFF != form;
    if changed {
        box_get_value(mon, 0x6F, 0, h);
        let species = box_get_value(mon, 5, 0, h) & 0xFFFF;
        box_set_value(mon, 0x6F, form, h);
        update_ability_slot(mon, species, h);
        update_gender(mon, species, species, h);
    }
    reencrypt(mon, enter as u32);
    changed
}

#[cfg(test)]
mod tests;
