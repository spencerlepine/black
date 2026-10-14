//! Item use on a party Pokemon: the "can this item be used" checks and the effects, outside battle
//! (S4b; the party-summary overlay 91, `0x021BBE74..0x021BEA7C`). Logic only (D3): the menus,
//! messages and the party-screen work struct of the original are not here; the original passes the
//! pieces of that struct it needs (the heap id) as arguments.
//!
//! Every routine reads the item through [`record_param`] (`FUN_02021014`) on the record of
//! [`load_item_data`] (`FUN_02020F34`, kind 0), and the Pokemon through the `party_get_value` /
//! `party_set_value` field accessors of `black-pokemon`. The usage parameters are numbered as in
//! [`crate::item`]: 0x12-0x16 the status cures (sleep, poison, burn, freeze, paralysis), 0x1A/0x1B the
//! revive flags, 0x1C level up, 0x1D evolution, 0x25/0x26 PP Up/PP Max, 0x27/0x28 PP restore of one
//! move / all moves, 0x29 HP restore, 0x2A-0x2F the six EV flags with the amounts 0x34-0x39, 0x30 the
//! "EVs may pass 100" flag, 0x31-0x33 the friendship tiers with the deltas 0x3C-0x3E, 0x3A the HP
//! amount, 0x3B the PP amount.
//!
//! Callees that live in other areas (stat recalculation, the growth curve, the PP routines, the
//! friendship routine) are the methods of [`ItemUseHooks`]; the differential test stubs them
//! identically on both sides, so only their arguments are checked.

use crate::item::{record_param, ItemHooks};
use black_pokemon::hooks::PokeToolHooks;
use black_pokemon::poke_tool::{party_get_value, party_set_value, plate_type};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_BE74,
    0x021B_C17C,
    0x021B_C1B4,
    0x021B_C228,
    0x021B_DD78,
    0x021B_E2B4,
    0x021B_E874,
    0x021B_E914,
    0x021B_E948,
    0x021B_E9BC,
    0x021B_EA48,
    0x021B_C24C,
    0x021B_C288,
    0x0201_7F90,
    0x0201_9D00,
];

/// Heap id the use-kind classifier loads the item record with.
pub const KIND_HEAP: u32 = 0x17;
/// Species whose HP EVs an item never changes (Shedinja).
pub const SPECIES_NO_HP_EV: u32 = 0x124;
/// The EV total at which no more EVs are added.
pub const EV_TOTAL_MAX: i32 = 0x1FE;

/// Pokemon field ids read and written by the item routines.
mod fid {
    pub const SPECIES: u32 = 5;
    pub const EXP: u32 = 8;
    pub const FRIENDSHIP: u32 = 9;
    /// HP, Attack, Defense, Speed, Sp. Attack, Sp. Defense EVs.
    pub const EV: [u32; 6] = [0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12];
    pub const MOVE: u32 = 0x36;
    pub const PP: u32 = 0x3A;
    pub const PP_UPS: u32 = 0x3E;
    pub const FORM: u32 = 0x6F;
    pub const STATUS: u32 = 0x9D;
    pub const LEVEL: u32 = 0x9E;
    pub const HP: u32 = 0xA0;
    pub const MAX_HP: u32 = 0xA1;
}

/// Use kinds returned by [`use_kind`] (the party menu picks its behaviour from them).
pub mod kind {
    /// Not a "use on a Pokemon" item (usage kind byte != 1).
    pub const NONE: u32 = 0x1D;
    /// Any of the usage parameters 0x1E-0x24 is set.
    pub const FIELD_PARAMS: u32 = 0;
    pub const REVIVE: u32 = 1;
    pub const LEVEL_UP: u32 = 2;
    pub const STATUS_SLEEP: u32 = 3;
    pub const STATUS_POISON: u32 = 4;
    pub const STATUS_BURN: u32 = 5;
    pub const STATUS_FREEZE: u32 = 6;
    pub const STATUS_PARALYSIS: u32 = 7;
    pub const STATUS_ALL: u32 = 8;
    pub const STATUS_ALL_AND_HP: u32 = 0x0B;
    pub const STATUS_CONFUSION: u32 = 9;
    pub const PARAM_18: u32 = 0x0A;
    pub const HP_RESTORE: u32 = 0x0B;
    pub const PARAM_1A: u32 = 0x0C;
    pub const PP_UP: u32 = 0x1A;
    pub const PP_MAX: u32 = 0x1B;
    pub const PP_RESTORE: u32 = 0x1C;
}

/// Callees of the item-use routines that belong to other areas (stubbed by the differential test).
/// The Pokemon is passed along where the original passes it; a production implementation changes it.
pub trait ItemUseHooks: ItemHooks + PokeToolHooks {
    /// Callee `FUN_020185C4(mon)`: recalculate level and stats after the EXP or an EV changed.
    fn recalc(&mut self, mon: &mut [u8]);
    /// Callee `FUN_02018584(species, form, level)`: the EXP a Pokemon of that species needs for `level`.
    fn exp_for_species_level(&mut self, species: u32, form: u32, level: u32) -> u32;
    /// Callee `FUN_0201B920(mon, slot)`: 1 if the PP of the slot is below its maximum.
    fn slot_below_max(&mut self, mon: &mut [u8], slot: u32) -> u32;
    /// Callee `FUN_0201B970(mon, slot, amount)`: raise the PP of a slot; 1 if it changed.
    fn raise_pp(&mut self, mon: &mut [u8], slot: u32, amount: u32) -> u32;
    /// Callee `FUN_0201B7B4(mon, item, location, heap)`: apply the item's friendship deltas.
    fn friendship_by_item(&mut self, mon: &mut [u8], item: u32, location: u32, heap: u32);
    /// Callee `FUN_02017988(mon, form)`: change the form of a party Pokemon (decrypt, set the form, recalculate
    /// the stats if it changed, encrypt).
    fn set_form(&mut self, mon: &mut [u8], form: u32);
}

/// The item record of `item` (heap `heap`), run through `f`, and released afterwards
/// (`FUN_02020F34` kind 0 ... `FUN_020307B0`).
fn with_record<H: ItemHooks, T>(
    h: &mut H,
    item: u32,
    heap: u32,
    f: impl FnOnce(&mut H, &[u8]) -> T,
) -> T {
    let handle = crate::item::load_item_data(h, item, heap);
    let rec = h.block(handle);
    let r = f(h, &rec);
    h.free(handle);
    r
}

fn get<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], id: u32) -> u32 {
    party_get_value(mon, id, 0, h)
}

fn set<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], id: u32, val: u32) {
    party_set_value(mon, id, val, h)
}

/// C: `FUN_02017F90(mon)`: the status word of a party Pokemon (field 0x9D; 0 = healthy).
pub fn status_of<H: ItemUseHooks>(h: &mut H, mon: &mut [u8]) -> u32 {
    get(h, mon, fid::STATUS)
}

/// C: `FUN_02019D00(item)`: the Genesect form a Drive item selects (items 0x74-0x77: forms 1-4), else 0.
pub fn drive_form(item: u32) -> u32 {
    match item.wrapping_sub(0x74) {
        i @ 0..=3 => i + 1,
        _ => 0,
    }
}

/// Species the held-item form routines act on: Arceus and Genesect.
pub const SPECIES_ARCEUS: u32 = 0x1ED;
pub const SPECIES_GENESECT: u32 = 0x289;

/// C: `FUN_overlay_d_91__021bc24c(work, mon, item)`: an Arceus takes the form of the plate `item`
/// (`FUN_02019CF8`, `black_pokemon::poke_tool::plate_type`) if it differs from its current form.
pub fn apply_plate_form<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], item: u32) {
    if get(h, mon, fid::SPECIES) == SPECIES_ARCEUS {
        let form = get(h, mon, fid::FORM);
        let target = plate_type(item);
        if form != target {
            h.set_form(mon, target);
        }
    }
}

/// C: `FUN_overlay_d_91__021bc288(work, mon, item)`: a Genesect takes the form of the Drive `item`
/// ([`drive_form`]) if it differs from its current form.
pub fn apply_drive_form<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], item: u32) {
    if get(h, mon, fid::SPECIES) == SPECIES_GENESECT {
        let form = get(h, mon, fid::FORM);
        let target = drive_form(item);
        if form != target {
            h.set_form(mon, target & 0xFFFF);
        }
    }
}

/// The classification of a record by [`use_kind`] (no loading).
fn kind_of(rec: &[u8]) -> u32 {
    let p = |n: u32| record_param(rec, n);
    if p(0x0E) != 1 {
        return kind::NONE;
    }
    if (0x1E..=0x24).any(|n| p(n) != 0) {
        return kind::FIELD_PARAMS;
    }
    if p(0x1B) != 0 {
        return kind::REVIVE;
    }
    if p(0x1C) != 0 {
        return kind::LEVEL_UP;
    }
    let sum = p(0x12)
        .wrapping_add(p(0x13).wrapping_mul(2))
        .wrapping_add(p(0x14).wrapping_mul(4))
        .wrapping_add(p(0x15).wrapping_mul(8))
        .wrapping_add(p(0x16).wrapping_mul(0x10))
        .wrapping_add(p(0x17).wrapping_mul(0x20)) as i32;
    match sum {
        1 => return kind::STATUS_SLEEP,
        2 => return kind::STATUS_POISON,
        4 => return kind::STATUS_BURN,
        8 => return kind::STATUS_FREEZE,
        0x10 => return kind::STATUS_PARALYSIS,
        0x20 => return kind::STATUS_ALL,
        0x3F => {
            return if p(0x29) != 0 {
                kind::STATUS_ALL_AND_HP
            } else {
                kind::STATUS_CONFUSION
            }
        }
        _ => {}
    }
    if p(0x18) != 0 {
        return kind::PARAM_18;
    }
    if p(0x29) != 0 {
        return kind::HP_RESTORE;
    }
    if p(0x1A) != 0 {
        return kind::PARAM_1A;
    }
    // the six EV amounts: positive 0x0D.., negative 0x13..
    for (i, param) in (0x34..=0x39).enumerate() {
        let v = p(param) as i32;
        if v > 0 {
            return 0x0D + i as u32;
        }
        if v < 0 {
            return 0x13 + i as u32;
        }
    }
    if p(0x1D) != 0 {
        return 0x19;
    }
    if p(0x25) != 0 {
        return kind::PP_UP;
    }
    if p(0x26) != 0 {
        return kind::PP_MAX;
    }
    if p(0x27) != 0 || p(0x28) != 0 {
        return kind::PP_RESTORE;
    }
    kind::NONE
}

/// C: `FUN_overlay_d_91__021bbe74(item)`: classify an item by its usage parameters (`0x1D` = no use on a
/// Pokemon; `0` the field-use parameters 0x1E-0x24; 1 revive; 2 level up; 3-8 the status cures by the
/// bit pattern of parameters 0x12-0x17; ...). The record is loaded with heap [`KIND_HEAP`].
pub fn use_kind<H: ItemHooks>(h: &mut H, item: u32) -> u32 {
    with_record(h, item, KIND_HEAP, |_, rec| kind_of(rec))
}

/// C: `FUN_overlay_d_91__021bc17c(work, item)`: 1 if the item needs a move chosen (PP Up, PP Max, or the
/// PP restore of a single move: parameter 0x27), else 0. `heap` is the first halfword of the party
/// screen's work struct, the heap `FUN_02020FE4` loads the record with.
pub fn needs_move_slot<H: ItemHooks>(h: &mut H, heap: u32, item: u32) -> u32 {
    match use_kind(h, item) {
        kind::PP_UP | kind::PP_MAX => 1,
        kind::PP_RESTORE => (crate::item::item_param(h, item, 0x27, heap as u16) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_91__021bc1b4(work, item)`: 1 if usage parameter 0x1B of the item is set.
pub fn has_param_1b<H: ItemHooks>(h: &mut H, heap: u32, item: u32) -> u32 {
    (crate::item::item_param(h, item, 0x1B, heap as u16) != 0) as u32
}

/// C: `FUN_overlay_d_91__021bc228(work, item)`: the prompt id of the move-choice screen: 0x16 for the PP
/// Up / PP Max items, 0x15 for everything else.
pub fn move_prompt_id<H: ItemHooks>(h: &mut H, item: u32) -> u32 {
    match use_kind(h, item) {
        kind::PP_UP | kind::PP_MAX => 0x16,
        _ => 0x15,
    }
}

/// C: `FUN_overlay_d_91__021be9bc(mon, rec)`: whether the item record (`rec`) would raise the friendship
/// of the Pokemon: the parameter of its tier (below 100, below 200, below 255) is set and the delta is
/// positive; 0 at friendship 255 or more.
pub fn friendship_raises<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], rec: &[u8]) -> u32 {
    let f = get(h, mon, fid::FRIENDSHIP) as i32;
    let (flag, delta) = if f >= 0xFF {
        return 0;
    } else if f < 100 {
        (0x31, 0x3C)
    } else if f < 200 {
        (0x32, 0x3D)
    } else {
        (0x33, 0x3E)
    };
    (record_param(rec, flag) != 0 && (record_param(rec, delta) as i32) > 0) as u32
}

/// C: `FUN_overlay_d_91__021be948(ev, others, delta, flag)`: the new value of one EV after adding
/// `delta` (-1 if the item cannot change it): `others` is the sum of the other five EVs, `flag` the
/// "may pass 100" parameter 0x30. Capped at 255 EVs by the caller's check, at 100 without the flag and
/// at [`EV_TOTAL_MAX`] in total; never below 0.
pub fn ev_after(ev: i32, others: i32, delta: i32, flag: i32) -> i32 {
    if ev == 0 && delta < 0 {
        return -1;
    }
    if ev == 0xFF && delta > 0 {
        return -1;
    }
    if ev >= 100 && flag == 0 && delta > 0 {
        return -1;
    }
    if ev.wrapping_add(others) >= EV_TOTAL_MAX && delta > 0 {
        return -1;
    }
    let mut v = ev.wrapping_add(delta);
    if delta > 0 && v > 100 && flag == 0 {
        v = 100;
    } else if v < 0 {
        v = 0;
    }
    if v.wrapping_add(others) > EV_TOTAL_MAX {
        v = EV_TOTAL_MAX.wrapping_sub(others);
    }
    v
}

/// C: `FUN_overlay_d_91__021be914(mon, hp, max, amount)`: raise the HP by `amount` (0xFF = to the
/// maximum, 0xFE half of it, 0xFD a quarter of it; 1 for a maximum of 1), capped at the maximum.
pub fn heal<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], hp: u32, max: u32, amount: u32) {
    let amount = if max == 1 {
        1
    } else {
        match amount {
            0xFF => max,
            0xFE => max >> 1,
            0xFD => max >> 2,
            a => a,
        }
    };
    let mut new = hp.wrapping_add(amount);
    if new > max {
        new = max;
    }
    set(h, mon, fid::HP, new);
}

/// C: `FUN_overlay_d_91__021be874(mon, slot, ups)`: add `ups` PP Ups to the move in `slot`: 0 if it
/// already has 3 or its base PP is below 5; else the PP gain follows the maximum, and 1.
pub fn add_pp_ups<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], slot: u32, ups: u32) -> u32 {
    let cur = get(h, mon, fid::PP_UPS + slot) & 0xFF;
    if cur == 3 {
        return 0;
    }
    let mv = get(h, mon, fid::MOVE + slot) & 0xFFFF;
    if h.move_max_pp(mv, 0) < 5 {
        return 0;
    }
    let pp = get(h, mon, fid::PP + slot) & 0xFF;
    let old_max = h.move_max_pp(mv, cur);
    let sum = cur.wrapping_add(ups); // the original wraps (`adds` + unsigned `bls`), V-0032
    let new_ups = if sum > 3 { 3 } else { sum & 0xFF };
    let new_max = h.move_max_pp(mv, new_ups);
    let pp = pp.wrapping_add(new_max).wrapping_sub(old_max) & 0xFF;
    set(h, mon, fid::PP_UPS + slot, new_ups);
    set(h, mon, fid::PP + slot, pp);
    1
}

/// C: `FUN_overlay_d_91__021bea48(mon, friendship, delta, item, location, heap)`: apply the item's
/// friendship deltas (`FUN_0201B7B4`) unless the friendship is already at its limit in the direction
/// of `delta`; 1 if applied.
pub fn apply_friendship<H: ItemUseHooks>(
    h: &mut H,
    mon: &mut [u8],
    friendship: u32,
    delta: i32,
    item: u32,
    location: u32,
    heap: u32,
) -> u32 {
    if friendship == 0xFF && delta > 0 {
        return 0;
    }
    if friendship == 0 && delta < 0 {
        return 0;
    }
    h.friendship_by_item(mon, item, location & 0xFFFF, heap & 0xFFFF);
    1
}

/// The six EV values (HP, Attack, Defense, Speed, Sp. Attack, Sp. Defense), signed as the original compares.
fn evs<H: ItemUseHooks>(h: &mut H, mon: &mut [u8]) -> [i32; 6] {
    let mut e = [0i32; 6];
    for (v, id) in e.iter_mut().zip(fid::EV) {
        *v = get(h, mon, id) as i32;
    }
    e
}

/// C: `FUN_overlay_d_91__021bdd78(mon, item, slot, heap)`: can the item be used on the Pokemon (1) or not
/// (0)? `slot` is the move slot the item is aimed at (PP items). `heap` is the heap id the item record is
/// loaded with (16 bits).
pub fn can_use<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], item: u32, slot: u32, heap: u32) -> u32 {
    with_record(h, item, heap & 0xFFFF, |h, rec| {
        can_use_with(h, mon, rec, slot)
    }) as u32
}

fn can_use_with<H: ItemUseHooks>(h: &mut H, mon: &mut [u8], rec: &[u8], slot: u32) -> bool {
    let p = |n: u32| record_param(rec, n);
    if p(0x0E) != 1 {
        return false;
    }
    let status = status_of(h, mon);
    for (param, value) in [(0x12, 2), (0x13, 5), (0x14, 4), (0x15, 3), (0x16, 1)] {
        if p(param) != 0 && status == value {
            return true;
        }
    }
    let hp = get(h, mon, fid::HP);
    let revive = (p(0x1A) != 0 || p(0x1B) != 0) && p(0x1C) == 0;
    if revive {
        if hp == 0 {
            return true;
        }
    } else if p(0x29) != 0 && hp != 0 && hp < get(h, mon, fid::MAX_HP) {
        return true;
    }
    if p(0x1C) != 0 && get(h, mon, fid::LEVEL) < 100 {
        return true;
    }
    if (p(0x25) != 0 || p(0x26) != 0) && get(h, mon, fid::PP_UPS + slot) < 3 {
        let mv = get(h, mon, fid::MOVE + slot) & 0xFFFF;
        if h.move_max_pp(mv, 0) >= 5 {
            return true;
        }
    }
    if p(0x27) != 0 && h.slot_below_max(mon, slot) == 1 {
        return true;
    }
    if p(0x28) != 0 {
        for i in 0..4 {
            if h.slot_below_max(mon, i) == 1 {
                return true;
            }
        }
    }
    let ev = evs(h, mon);
    let flag = p(0x30) as i32;
    let species = get(h, mon, fid::SPECIES);
    for (i, (&e, (on, amount))) in ev
        .iter()
        .zip([
            (0x2A, 0x34),
            (0x2B, 0x35),
            (0x2C, 0x36),
            (0x2D, 0x37),
            (0x2E, 0x38),
            (0x2F, 0x39),
        ])
        .enumerate()
    {
        if i == 0 && species == SPECIES_NO_HP_EV {
            continue;
        }
        if p(on) == 0 {
            continue;
        }
        let delta = p(amount) as i32;
        if delta > 0 {
            if e >= 100 && flag == 0 {
                continue;
            }
            if e >= 0xFF {
                continue;
            }
            let total = ev.iter().fold(0i32, |a, &b| a.wrapping_add(b));
            if total < EV_TOTAL_MAX {
                return true;
            }
        } else if delta < 0 && (e > 0 || friendship_raises(h, mon, rec) == 1) {
            return true;
        }
    }
    false
}

/// C: `FUN_overlay_d_91__021be2b4(mon, item, slot, location, heap)`: use the item on the Pokemon: cure
/// the status, restore HP / revive, level up, raise PP, add or remove EVs, then the friendship deltas.
/// Returns 1 if the Pokemon was changed by anything but the friendship (0 if the item had an effect but
/// changed nothing). `slot` is the move slot (PP items), `location` the met-location argument of the
/// friendship routine, `heap` the heap id (16 bits).
pub fn apply_item<H: ItemUseHooks>(
    h: &mut H,
    mon: &mut [u8],
    item: u32,
    slot: u32,
    location: u32,
    heap: u32,
) -> u32 {
    with_record(h, item, heap & 0xFFFF, |h, rec| {
        apply_with(h, mon, rec, item, slot, location, heap)
    }) as u32
}

fn apply_with<H: ItemUseHooks>(
    h: &mut H,
    mon: &mut [u8],
    rec: &[u8],
    item: u32,
    slot: u32,
    location: u32,
    heap: u32,
) -> bool {
    let p = |n: u32| record_param(rec, n);
    if p(0x0E) != 1 {
        return false;
    }
    let mut changed = false;
    let mut used = false;
    let status = status_of(h, mon);
    let mut new_status = status;
    for param in 0x12..=0x16 {
        if p(param) != 0 {
            new_status = 0;
            used = true;
        }
    }
    if status != new_status {
        set(h, mon, fid::STATUS, new_status);
        changed = true;
    }
    let hp = get(h, mon, fid::HP);
    let max_hp = get(h, mon, fid::MAX_HP);
    if (p(0x1A) != 0 || p(0x1B) != 0) && p(0x1C) == 0 {
        if hp == 0 {
            heal(h, mon, hp, max_hp, p(0x3A));
            changed = true;
        }
        used = true;
    } else if p(0x29) != 0 {
        if (hp as i32) < (max_hp as i32) {
            heal(h, mon, hp, max_hp, p(0x3A));
            changed = true;
        }
        used = true;
    }
    let level = get(h, mon, fid::LEVEL);
    if p(0x1C) != 0 {
        if (level as i32) < 100 {
            let species = get(h, mon, fid::SPECIES);
            let form = get(h, mon, fid::FORM);
            let exp = h.exp_for_species_level(
                species & 0xFFFF,
                form & 0xFFFF,
                level.wrapping_add(1) & 0xFFFF,
            );
            set(h, mon, fid::EXP, exp);
            h.recalc(mon);
            if hp == 0 {
                let new_max = get(h, mon, fid::MAX_HP);
                heal(h, mon, hp, new_max, new_max.wrapping_sub(max_hp));
            }
            changed = true;
        }
        used = true;
    }
    if p(0x1D) != 0 {
        used = true;
    }
    if p(0x25) != 0 {
        used = true;
        if add_pp_ups(h, mon, slot, 1) == 1 {
            changed = true;
        }
    } else if p(0x26) != 0 {
        if add_pp_ups(h, mon, slot, 3) == 1 {
            changed = true;
        }
        used = true;
    }
    if p(0x27) != 0 {
        if h.raise_pp(mon, slot, p(0x3B)) == 1 {
            changed = true;
        }
        used = true;
    } else if p(0x28) != 0 {
        for i in 0..4 {
            if h.raise_pp(mon, i, p(0x3B)) == 1 {
                changed = true;
            }
        }
        used = true;
    }
    let mut ev = evs(h, mon);
    let flag = p(0x30) as i32;
    let species = get(h, mon, fid::SPECIES);
    let blocks = [
        (0x2A, 0x34),
        (0x2B, 0x35),
        (0x2C, 0x36),
        (0x2D, 0x37),
        (0x2E, 0x38),
        (0x2F, 0x39),
    ];
    for (i, (on, amount)) in blocks.into_iter().enumerate() {
        if i == 0 && species == SPECIES_NO_HP_EV {
            continue;
        }
        if p(on) == 0 {
            continue;
        }
        let delta = p(amount) as i32;
        let others = ev
            .iter()
            .enumerate()
            .filter(|&(j, _)| j != i)
            .fold(0i32, |a, (_, &b)| a.wrapping_add(b));
        let new = ev_after(ev[i], others, delta, flag);
        if new != -1 {
            set(h, mon, fid::EV[i], new as u32);
            h.recalc(mon);
            ev[i] = new;
            changed = true;
        }
        if delta > 0 {
            used = true;
        }
    }
    if !changed && used {
        return false;
    }
    let friendship = get(h, mon, fid::FRIENDSHIP) as i32;
    let tier = if friendship < 100 {
        Some((0x31, 0x3C))
    } else if friendship < 200 {
        Some((0x32, 0x3D))
    } else if friendship <= 255 {
        Some((0x33, 0x3E))
    } else {
        None
    };
    if let Some((flag, delta)) = tier {
        if p(flag) != 0 {
            apply_friendship(
                h,
                mon,
                friendship as u32,
                p(delta) as i32,
                item,
                location,
                heap,
            );
        }
    }
    changed
}
#[cfg(test)]
mod tests;
