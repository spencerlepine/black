//! Egg generation (P7): the Pokemon-side logic of the day care, overlay 21 `0x021C3A00-0x021C4570`.
//!
//! The day care itself (`sodateya.c`, F13) owns the two parents and the "egg receive" flow; it calls
//! [`generate`] (`FUN_021C38F4`) with the parents and the profile. [`generate`] fills a 0x8C-byte work
//! struct ([`Egg`], laid out below) step by step and then builds the egg Pokemon from it
//! ([`build_mon`]). The steps, in the order the driver runs them:
//!
//! 1. parent order: male first ([`order_by_sex`]), Ditto second ([`order_ditto`]); the "base" parent is
//!    the second unless it is a Ditto ([`base_parent`]);
//! 2. species: the baby form from ARC 20 ([`baby_species`]), Nidoran / Volbeat / Illumise gender coin
//!    flips, Manaphy -> Phione, the incense babies ([`incense_baby`]); form for Shellos / Gastrodon /
//!    Burmy / Wormadam / Basculin from the base parent;
//! 3. nature (random, or an Everstone parent's), ability slot (hidden ability roll, Ditto), egg cycles
//!    (personal field 0x15), IVs (Power items, then random up to three), OT id, PID (Masuda: five
//!    re-rolls when the parents' languages differ);
//! 4. moves: the level-1 learnset, egg moves (ARC 123, the first parent's moves), moves both parents
//!    know, TM moves the first parent knows, Volt Tackle (Pichu + Light Ball).
//!
//! Every Pokemon accessor, allocator, archive reader and message routine is a callee outside the
//! area and comes in through [`EggHooks`]; the ROM differential test stubs them identically on both
//! sides (`tools/check-fn.sh black-pokemon egg --per-function`, spec
//! `tools/rom-diff/specs/black-pokemon/egg.spec`).
//!
//! Not here: `FUN_021C3898` (reads the day care struct and calls the driver) and the other
//! `sodateya.c` functions around it (F13).

use crate::create::is_shiny;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_38F4,
    0x021C_3A00,
    0x021C_3A44,
    0x021C_3A70,
    0x021C_3A7C,
    0x021C_3A90,
    0x021C_3AAC,
    0x021C_3ADC,
    0x021C_3B0C,
    0x021C_3B2C,
    0x021C_3BB0,
    0x021C_3BCC,
    0x021C_3BD4,
    0x021C_3C04,
    0x021C_3C30,
    0x021C_3C5C,
    0x021C_3C78,
    0x021C_3C88,
    0x021C_3D1C,
    0x021C_3DA8,
    0x021C_3DE4,
    0x021C_3E38,
    0x021C_3E64,
    0x021C_3F60,
    0x021C_3FC0,
    0x021C_4018,
    0x021C_4024,
    0x021C_4030,
    0x021C_4078,
    0x021C_4084,
    0x021C_40F8,
    0x021C_4160,
    0x021C_4238,
    0x021C_42BC,
    0x021C_42F4,
    0x021C_4340,
    0x021C_447C,
    0x021C_4504,
    0x021C_452C,
];

/// The egg work struct (`auStack_a4` of `FUN_021C38F4`, 0x8C bytes) as 35 words. Names are inferred
/// from the code that fills them.
pub type Egg = [u32; 35];
/// The two parents (`local_ac`, `local_a8` of the driver): `Pokemon` handles.
pub type Parents = [u32; 2];

/// Word indices of [`Egg`] (byte offset / 4).
pub mod w {
    /// Baby species (offset 0).
    pub const SPECIES: usize = 0;
    /// Form (4).
    pub const FORM: usize = 1;
    /// Level, 1 (8).
    pub const LEVEL: usize = 2;
    /// Nature (0xC).
    pub const NATURE: usize = 3;
    /// Ability id from the personal data slot (0x10).
    pub const ABILITY: usize = 4;
    /// Egg cycles = initial friendship, personal field 0x15 (0x14).
    pub const EGG_CYCLES: usize = 5;
    /// First of the six inherited / random IVs, in the order of [`super::IV_IDS`] (0x18).
    pub const IV: usize = 6;
    /// OT id (0x30).
    pub const OT_ID: usize = 12;
    /// PID (0x34).
    pub const PID: usize = 13;
    /// Number of moves in the list (0x38).
    pub const MOVE_COUNT: usize = 14;
    /// First of four moves (0x3C).
    pub const MOVES: usize = 15;
    /// Ability slot roll 0 / 1 / 2 (2 = hidden) (0x4C).
    pub const ABILITY_ROLL: usize = 19;
    /// Nature taken from the first parent's Everstone (0x50).
    pub const NATURE_FROM_P0: usize = 20;
    /// Nature taken from the second parent's Everstone (0x54).
    pub const NATURE_FROM_P1: usize = 21;
    /// First of six "IV comes from parent 0" flags (0x58).
    pub const IV_FROM_P0: usize = 22;
    /// First of six "IV comes from parent 1" flags (0x70).
    pub const IV_FROM_P1: usize = 28;
    /// Number of inherited IVs (0x88).
    pub const IV_COUNT: usize = 34;
}

/// `Pokemon` field ids used here (see [`crate::poke_tool::id`]).
mod fid {
    pub const SPECIES: u32 = 5;
    pub const HELD_ITEM: u32 = 6;
    pub const FRIENDSHIP: u32 = 9;
    pub const ABILITY: u32 = 10;
    pub const LANGUAGE: u32 = 12;
    pub const MOVE1: u32 = 0x36;
    pub const IS_EGG: u32 = 0x4C;
    pub const GENDER: u32 = 0x6E;
    pub const FORM: u32 = 0x6F;
    pub const NATURE: u32 = 0x70;
    pub const NICKNAME_STRBUF: u32 = 0x73;
    pub const OT_NAME_STRBUF: u32 = 0x8D;
    /// Put id 0xB2 is not a field of `box_put` (178 ids, 0..=177): the put is a no-op in this ROM.
    pub const ABILITY_PUT: u32 = 0xB2;
}

/// IV field ids in the order HP, Atk, Def, Spe, SpA, SpD (table at 0x021DA918 and 0x021DA948).
pub const IV_IDS: [u32; 6] = [0x46, 0x47, 0x48, 0x4A, 0x4B, 0x49];
/// Power items for the same six stats (table at 0x021DA930).
pub const POWER_ITEMS: [u32; 6] = [0x126, 0x121, 0x122, 0x123, 0x124, 0x125];
/// Move field ids of a parent (tables at 0x021DA8E8 / 0x021DA8F8 / 0x021DA908).
const MOVE_IDS: [u32; 4] = [0x36, 0x37, 0x38, 0x39];
/// Everstone.
const EVERSTONE: u32 = 0xE5;
/// Light Ball.
const LIGHT_BALL: u32 = 0xEC;
/// Ditto.
const DITTO: u32 = 0x84;
/// Volt Tackle.
const VOLT_TACKLE: u32 = 0x158;
/// Messages: the egg name (message 0x28A, also the bad-egg name).
const MSG_EGG_NAME: u32 = 0x28A;
/// "Species count": ids from here on are not species (0x289).
const SPECIES_LIMIT: u32 = 0x289;
/// Incense baby table (0x021DA960): baby species, incense item, species without the incense.
pub const INCENSE_BABIES: [[u32; 3]; 12] = [
    [0x12A, 0xFE, 0xB7],
    [0x168, 0xFF, 0xCA],
    [0x1B7, 0x13A, 0x7A],
    [0x1B6, 0x13B, 0xB9],
    [0x1BE, 0x13C, 0x8F],
    [0x1CA, 0x13D, 0xE2],
    [0x196, 0x13E, 0x13B],
    [0x1B8, 0x13F, 0x71],
    [0x1B1, 0x140, 0x166],
    [0, 0, 0],
    [0, 0, 0],
    [0, 0, 0],
];

/// Callees outside the area. Pointers into the game (`Pokemon`, personal records, string buffers,
/// archive blocks) are opaque `u32` handles; the ROM differential test stubs every one of these
/// identically on both sides, so only the arguments are verified, not the callees' behaviour.
pub trait EggHooks {
    /// `FUN_02017E1C(mon, id, 0)`: get a field of a `Pokemon` ([`crate::poke_tool::party_get_value`]).
    fn get(&mut self, mon: u32, id: u32) -> u32;
    /// `FUN_02017E40(mon, id, value)`: set a field of a `Pokemon`.
    fn put(&mut self, mon: u32, id: u32, value: u32);
    /// `FUN_02005728(max)`: random number below `max`.
    fn rand(&mut self, max: u32) -> u32;
    /// `FUN_0201ADE4(species, form, heap)`: allocate and read a personal record.
    fn alloc_record(&mut self, species: u32, form: u32, heap: u32) -> u32;
    /// `FUN_0201AE1C(record, field)`: a field of a personal record.
    fn record_field(&mut self, record: u32, field: u32) -> u32;
    /// `FUN_020307B0(block)` (reached through the thunk `FUN_0201AE14` for a record): free a heap block.
    fn free_block(&mut self, block: u32);
    /// `FUN_0201ADD0(species, form, dst)`: the learnset of a species/form into `dst` (0x68 bytes of
    /// `(move, level)` halfword pairs ending in a pair of [`EggHooks::learnset_end`]).
    fn read_learnset(&mut self, species: u32, form: u32, dst: &mut [u8]);
    /// The halfword that ends a learnset: 0xFFFF in the game (a literal in `FUN_021C4084` and
    /// `FUN_021C4160`; the differential test moves it so random learnsets end).
    fn learnset_end(&mut self) -> u32 {
        0xFFFF
    }
    /// `FUN_02048F60(arc, member, heap)`: load an archive member into a heap block.
    fn arc_alloc(&mut self, arc: u32, member: u32, heap: u32) -> u32;
    /// A halfword of a block returned by [`EggHooks::arc_alloc`] (plain memory read).
    fn block_u16(&mut self, block: u32, index: u32) -> u32;
    /// `FUN_02048F74(dst, arc, member, offset, 2)`: one halfword of an archive member.
    fn arc_u16(&mut self, arc: u32, member: u32, offset: u32) -> u32;
    /// `FUN_02021288(item)`: whether the item is a TM or HM.
    fn item_is_tm(&mut self, item: u32) -> u32;
    /// `FUN_0202130C(item)`: the TM/HM index of an item.
    fn tm_index(&mut self, item: u32) -> u32;
    /// `FUN_020212B0(item)`: the move a TM/HM item teaches.
    fn tm_move(&mut self, item: u32) -> u32;
    /// `FUN_02008530(profile)`: the trainer id of a profile (a field read).
    fn trainer_id(&mut self, profile: u32) -> u32;
    /// `FUN_02017588(mon, species, level, a3, a4, a5, a6, a7)`: initialise a `Pokemon`
    /// ([`crate::create::init_pokemon_full`]).
    #[allow(clippy::too_many_arguments)]
    fn init_pokemon(
        &mut self,
        mon: u32,
        species: u32,
        level: u32,
        a3: u32,
        a4: u32,
        a5: u32,
        a6: u32,
        a7: u32,
    );
    /// `FUN_02018A00(mon, species, form)` (a thunk to `0x02018A08`): give the Pokemon its hidden ability.
    fn set_hidden_ability(&mut self, mon: u32, species: u32, form: u32);
    /// `FUN_02008510(profile, heap)`: a string buffer with the profile's trainer name.
    fn trainer_name(&mut self, profile: u32, heap: u32) -> u32;
    /// `FUN_02045808(strbuf)`: free a string buffer.
    fn free_strbuf(&mut self, strbuf: u32);
    /// `FUN_02045EC0(*0x020A7370, message)`: a message of the global message manager as a string buffer.
    fn message(&mut self, message: u32) -> u32;
    /// `FUN_0202F190(mon, kind, profile, place, heap)`: record where and by whom the Pokemon was
    /// received (kind 5 = egg).
    fn set_met_data(&mut self, mon: u32, kind: u32, profile: u32, place: u32, heap: u32);
    /// `FUN_02018198(mon)` (a thunk to `0x020181A0`): restore all PP ([`crate::waza`]).
    fn restore_pp(&mut self, mon: u32);
    /// `FUN_020185C4(mon)`: recalculate level and stats ([`crate::create::recalc`]).
    fn recalc(&mut self, mon: u32);
    /// `FUN_0201B9F0(mon)`: restore HP, status and PP ([`crate::waza::restore_mon`]).
    fn restore_mon(&mut self, mon: u32);
}

/// C: `FUN_021C3A70(parents)`: swap the two parents.
pub fn swap_parents(p: &mut Parents) {
    p.swap(0, 1);
}

/// C: `FUN_021C3A00(parents)`: put the male parent first. Swaps if the first is female, then swaps
/// again if the (new) second is male. The extra reads are in the original.
pub fn order_by_sex(p: &mut Parents, h: &mut impl EggHooks) {
    let g0 = h.get(p[0], fid::GENDER);
    h.get(p[1], fid::GENDER);
    if g0 == 1 {
        swap_parents(p);
    }
    h.get(p[0], fid::GENDER);
    if h.get(p[1], fid::GENDER) == 0 {
        swap_parents(p);
    }
}

/// C: `FUN_021C3A44(parents)`: swap if the first parent is a Ditto and the second is genderless.
pub fn order_ditto(p: &mut Parents, h: &mut impl EggHooks) {
    let s0 = h.get(p[0], fid::SPECIES);
    let g1 = h.get(p[1], fid::GENDER);
    if s0 == DITTO && g1 == 2 {
        swap_parents(p);
    }
}

/// C: `FUN_021C3A7C(egg)`: clear the work struct and set the level to 1.
pub fn init_egg(e: &mut Egg) {
    *e = [0; 35];
    e[w::LEVEL] = 1;
}

/// C: `FUN_021C3BB0(parents)` (identical copy: `FUN_021C3C5C`): the parent whose traits the baby takes: the
/// second parent, or the first if the second is a Ditto.
pub fn base_parent(p: &Parents, h: &mut impl EggHooks) -> u32 {
    if h.get(p[1], fid::SPECIES) == DITTO {
        p[0]
    } else {
        p[1]
    }
}

/// C: `FUN_021C4504(species)`: the baby form of a species (ARC 20, halfword at `species * 2`); ids from
/// 0x289 are returned unchanged.
pub fn baby_species(species: u32, h: &mut impl EggHooks) -> u32 {
    if species < SPECIES_LIMIT {
        h.arc_u16(0x14, species, 0)
    } else {
        species
    }
}

/// C: `FUN_021C3A90(parents, egg)`: the baby species of the base parent.
pub fn set_baby_species(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let base = base_parent(p, h);
    let species = h.get(base, fid::SPECIES) & 0xFFFF;
    e[w::SPECIES] = baby_species(species, h);
}

/// C: `FUN_021C3AAC(parents, egg)`: Nidoran female / male are equally likely for a baby of the Nidoran
/// line (parent species 29, or 32..=34).
pub fn nidoran_gender(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let base = base_parent(p, h);
    let s = h.get(base, fid::SPECIES);
    if s == 29 || s.wrapping_sub(32) <= 2 {
        e[w::SPECIES] = if h.rand(2) == 0 { 29 } else { 32 };
    }
}

/// C: `FUN_021C3ADC(parents, egg)`: Volbeat (313) / Illumise (314) are equally likely.
pub fn volbeat_illumise(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let base = base_parent(p, h);
    let s = h.get(base, fid::SPECIES);
    if s.wrapping_sub(313) <= 1 {
        e[w::SPECIES] = if h.rand(2) == 0 { 313 } else { 314 };
    }
}

/// C: `FUN_021C3B0C(parents, egg)`: a Manaphy (490) parent gives Phione (489).
pub fn phione(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let base = base_parent(p, h);
    if h.get(base, fid::SPECIES) == 490 {
        e[w::SPECIES] = 489;
    }
}

/// C: `FUN_021C3B2C(parents, egg)`: the incense babies: if the baby is one of the table's baby
/// species and neither parent holds the matching incense, the baby is the table's unbaby species.
pub fn incense_baby(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let base = base_parent(p, h);
    h.get(base, fid::SPECIES);
    h.get(base, fid::HELD_ITEM);
    let item0 = h.get(p[0], fid::HELD_ITEM);
    let item1 = h.get(p[1], fid::HELD_ITEM);
    let baby = e[w::SPECIES];
    for row in INCENSE_BABIES {
        if baby == row[0] && item0 != row[1] && item1 != row[1] {
            e[w::SPECIES] = row[2];
            return;
        }
    }
}

/// C: `FUN_021C3BCC(_, egg)`: clear the form.
pub fn clear_form(e: &mut Egg) {
    e[w::FORM] = 0;
}

/// Copy the base parent's form when its species is in `check`.
fn form_from_base(p: &Parents, e: &mut Egg, h: &mut impl EggHooks, check: impl Fn(u32) -> bool) {
    let base = base_parent(p, h);
    let s = h.get(base, fid::SPECIES);
    if check(s) {
        e[w::FORM] = h.get(base, fid::FORM);
    }
}

/// C: `FUN_021C3BD4(parents, egg)`: the form of a Shellos / Gastrodon (422, 423) parent.
pub fn form_shellos(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    form_from_base(p, e, h, |s| s.wrapping_sub(422) <= 1);
}

/// C: `FUN_021C3C04(parents, egg)`: the form of a Burmy / Wormadam (412, 413) parent.
pub fn form_burmy(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    form_from_base(p, e, h, |s| s.wrapping_sub(412) <= 1);
}

/// C: `FUN_021C3C30(parents, egg)`: the form of a Basculin (550) parent.
pub fn form_basculin(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    form_from_base(p, e, h, |s| s == 550);
}

/// C: `FUN_021C3C78(_, egg)`: a random nature.
pub fn roll_nature(e: &mut Egg, h: &mut impl EggHooks) {
    e[w::NATURE] = h.rand(25);
}

/// C: `FUN_021C3C88(parents, egg)`: an Everstone parent passes its nature on (half of the time when
/// one holds it, a coin flip between the two when both do).
pub fn everstone_nature(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let s0 = h.get(p[0], fid::HELD_ITEM) == EVERSTONE;
    let s1 = h.get(p[1], fid::HELD_ITEM) == EVERSTONE;
    let n0 = h.get(p[0], fid::NATURE);
    let n1 = h.get(p[1], fid::NATURE);
    if (s0 || s1) && h.rand(2) == 0 {
        return;
    }
    if s0 && s1 {
        if h.rand(2) == 0 {
            e[w::NATURE] = n0;
            e[w::NATURE_FROM_P0] = 1;
        } else {
            e[w::NATURE] = n1;
            e[w::NATURE_FROM_P1] = 1;
        }
    } else if s0 {
        e[w::NATURE] = n0;
        e[w::NATURE_FROM_P0] = 1;
    } else if s1 {
        e[w::NATURE] = n1;
        e[w::NATURE_FROM_P1] = 1;
    }
}

/// C: `FUN_021C452C(species, form, ability, heap)`: the ability slot (0, 1 or 2 = hidden) of an ability
/// id in the personal record's slots (fields 0x1A, 0x1B, 0x1C); 0 if it is in none.
pub fn ability_slot(
    species: u32,
    form: u32,
    ability: u32,
    heap: u32,
    h: &mut impl EggHooks,
) -> u32 {
    let rec = h.alloc_record(species, form, heap);
    let a = [
        h.record_field(rec, 0x1A),
        h.record_field(rec, 0x1B),
        h.record_field(rec, 0x1C),
    ];
    h.free_block(rec);
    let mut i = 0;
    while i < 3 && ability != a[i] {
        i += 1;
    }
    if i >= 3 {
        i = 0;
    }
    i as u32 & 0xFF
}

/// C: `FUN_021C3D1C(parents, egg, heap)`: roll the ability slot of the baby from the second parent's
/// slot: slot 0 keeps it 80%, slot 1 keeps it 20%, slot 2 becomes slot 0 (20%), 1 (20%) or 2 (60%).
pub fn roll_ability(p: &Parents, e: &mut Egg, heap: u32, h: &mut impl EggHooks) {
    let species = h.get(p[1], fid::SPECIES);
    let form = h.get(p[1], fid::FORM);
    let ability = h.get(p[1], fid::ABILITY);
    let slot = ability_slot(species & 0xFFFF, form & 0xFFFF, ability & 0xFFFF, heap, h);
    let r = h.rand(100);
    e[w::ABILITY_ROLL] = match slot {
        1 => u32::from(r >= 0x14),
        2 => {
            if r < 0x14 {
                0
            } else if r < 0x28 {
                1
            } else {
                2
            }
        }
        _ => u32::from(r >= 0x50),
    };
}

/// C: `FUN_021C3DA8(parents, egg)`: with a Ditto parent the ability slot is 0 or 1.
pub fn ditto_ability(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let s0 = h.get(p[0], fid::SPECIES);
    let s1 = h.get(p[1], fid::SPECIES);
    if s0 == DITTO || s1 == DITTO {
        e[w::ABILITY_ROLL] = u32::from(h.rand(2) != 0);
    }
}

/// C: `FUN_021C3DE4(_, egg, heap)`: the ability id for slot 0 / 1 (slot 0 if empty); slot 2 is left to
/// [`build_mon`].
pub fn pick_ability(e: &mut Egg, heap: u32, h: &mut impl EggHooks) {
    let rec = h.alloc_record(e[w::SPECIES] & 0xFFFF, e[w::FORM] & 0xFFFF, heap);
    let a = [
        h.record_field(rec, 0x1A),
        h.record_field(rec, 0x1B),
        h.record_field(rec, 0x1C),
    ];
    h.free_block(rec);
    let roll = e[w::ABILITY_ROLL];
    if roll < 2 {
        e[w::ABILITY] = a[roll as usize];
        if e[w::ABILITY] == 0 {
            e[w::ABILITY] = a[0];
        }
    }
}

/// C: `FUN_021C3E38(egg, heap)`: egg cycles from personal field 0x15.
pub fn egg_cycles(e: &mut Egg, heap: u32, h: &mut impl EggHooks) {
    let rec = h.alloc_record(e[w::SPECIES] & 0xFFFF, e[w::FORM] & 0xFFFF, heap);
    let cycles = h.record_field(rec, 0x15);
    h.free_block(rec);
    e[w::EGG_CYCLES] = cycles;
}

/// C: `FUN_021C3E64(parents, egg)`: a parent holding a Power item passes the matching IV (a coin flip
/// when both hold one).
pub fn power_item_ivs(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let item0 = h.get(p[0], fid::HELD_ITEM);
    let item1 = h.get(p[1], fid::HELD_ITEM);
    let i0 = POWER_ITEMS.iter().position(|&x| x == item0);
    let i1 = POWER_ITEMS.iter().position(|&x| x == item1);
    match (i0, i1) {
        (Some(a), Some(b)) => {
            if h.rand(2) == 0 {
                e[w::IV_FROM_P0 + a] = 1;
            } else {
                e[w::IV_FROM_P1 + b] = 1;
            }
            e[w::IV_COUNT] += 1;
        }
        (Some(a), None) => {
            e[w::IV_FROM_P0 + a] = 1;
            e[w::IV_COUNT] += 1;
        }
        (None, Some(b)) => {
            e[w::IV_FROM_P1 + b] = 1;
            e[w::IV_COUNT] += 1;
        }
        (None, None) => {}
    }
}

/// C: `FUN_021C3F60(_, egg)`: choose random IVs of random parents until three are inherited.
pub fn random_ivs(e: &mut Egg, h: &mut impl EggHooks) {
    while e[w::IV_COUNT] < 3 {
        let stat = h.rand(6) as usize;
        let from_second = h.rand(2) != 0;
        let (f0, f1) = (w::IV_FROM_P0 + stat, w::IV_FROM_P1 + stat);
        if e[f0] == 0 && e[f1] == 0 {
            e[if from_second { f1 } else { f0 }] = 1;
            e[w::IV_COUNT] += 1;
        }
    }
}

/// C: `FUN_021C3FC0(parents, egg)`: read the flagged IVs from the parents.
pub fn inherit_ivs(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    for (i, id) in IV_IDS.iter().enumerate() {
        if e[w::IV_FROM_P0 + i] != 0 {
            e[w::IV + i] = h.get(p[0], *id);
        }
        if e[w::IV_FROM_P1 + i] != 0 {
            e[w::IV + i] = h.get(p[1], *id);
        }
    }
}

/// C: `FUN_021C4018(profile, egg)`: the OT id is the profile's trainer id.
pub fn set_ot_id(profile: u32, e: &mut Egg, h: &mut impl EggHooks) {
    e[w::OT_ID] = h.trainer_id(profile);
}

/// C: `FUN_021C4078()`: a random 32-bit number (`FUN_02005728(0xFFFFFFFF)`).
pub fn rand32(h: &mut impl EggHooks) -> u32 {
    h.rand(0xFFFF_FFFF)
}

/// C: `FUN_021C4024(_, egg)`: a random PID.
pub fn roll_pid(e: &mut Egg, h: &mut impl EggHooks) {
    e[w::PID] = rand32(h);
}

/// C: `FUN_021C4030(parents, egg)`: Masuda method: when the parents' languages differ and the PID is
/// not shiny, re-roll it up to five times until it is.
pub fn masuda(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let l0 = h.get(p[0], fid::LANGUAGE);
    let l1 = h.get(p[1], fid::LANGUAGE);
    if l0 == l1 || is_shiny(e[w::OT_ID], e[w::PID]) {
        return;
    }
    let mut n = 0;
    loop {
        e[w::PID] = rand32(h);
        if is_shiny(e[w::OT_ID], e[w::PID]) {
            return;
        }
        n += 1;
        if n >= 5 {
            return;
        }
    }
}

/// C: `FUN_021C42F4(egg, move)`: add a move to the list of four, unless it is 0 or already there; a
/// full list drops its first move.
pub fn add_move(e: &mut Egg, mv: u32) {
    if mv == 0 {
        return;
    }
    let n = e[w::MOVE_COUNT];
    for i in 0..n as usize {
        if e[w::MOVES + i] == mv {
            return;
        }
    }
    if n < 4 {
        e[w::MOVES + n as usize] = mv;
        e[w::MOVE_COUNT] += 1;
        return;
    }
    for i in 0..3 {
        e[w::MOVES + i] = e[w::MOVES + i + 1];
    }
    e[w::MOVES + 3] = mv;
}

/// A learnset entry `i` of a 0x68-byte buffer: `(move, level)`.
fn learnset_entry(buf: &[u8], i: usize) -> (u32, u32) {
    let r = |o: usize| u32::from(u16::from_le_bytes([buf[o], buf[o + 1]]));
    (r(i * 4), r(i * 4 + 2))
}

/// C: `FUN_021C4084(egg)`: the moves the baby species learns up to its level (1).
pub fn level1_moves(e: &mut Egg, h: &mut impl EggHooks) {
    let mut buf = [0u8; 0x68];
    h.read_learnset(e[w::SPECIES] & 0xFFFF, e[w::FORM] & 0xFFFF, &mut buf);
    let end = h.learnset_end();
    let mut i = 0;
    loop {
        let (mv, lv) = learnset_entry(&buf, i);
        if mv == end && lv == end {
            return;
        }
        if e[w::LEVEL] < lv {
            return;
        }
        add_move(e, mv);
        i += 1;
    }
}

/// C: `FUN_021C40F8(parents, egg, heap)`: egg moves (ARC 123, member = baby species: a count then that
/// many moves) that the first parent knows.
pub fn egg_moves(p: &Parents, e: &mut Egg, heap: u32, h: &mut impl EggHooks) {
    let block = h.arc_alloc(0x7B, e[w::SPECIES], heap);
    let n = h.block_u16(block, 0);
    for id in MOVE_IDS {
        let mv = h.get(p[0], id);
        if n >= 1 {
            let mut k = 1;
            loop {
                if mv == h.block_u16(block, k) {
                    add_move(e, mv);
                    break;
                }
                k += 1;
                if k > n {
                    break;
                }
            }
        }
    }
    h.free_block(block);
}

/// Whether a parent knows the move.
fn knows(mon: u32, mv: u32, h: &mut impl EggHooks) -> bool {
    for id in MOVE_IDS {
        if h.get(mon, id) == mv {
            return true;
        }
    }
    false
}

/// C: `FUN_021C4160(parents, egg)`: moves in the baby's learnset that both parents know.
pub fn common_moves(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    let mut buf = [0u8; 0x68];
    h.read_learnset(e[w::SPECIES] & 0xFFFF, e[w::FORM] & 0xFFFF, &mut buf);
    let end = h.learnset_end();
    let mut i = 0;
    loop {
        let (mv, lv) = learnset_entry(&buf, i);
        if mv == end && lv == end {
            return;
        }
        let a = knows(p[0], mv, h);
        let b = knows(p[1], mv, h);
        if a && b {
            add_move(e, mv);
        }
        i += 1;
    }
}

/// C: `FUN_021C447C(heap, species, form, item)`: whether the species can learn the TM/HM item (a bit
/// of personal fields 0x27..0x2A); 0 for a non-TM item or an id from 0x289.
pub fn can_learn_tm(heap: u32, species: u32, form: u32, item: u32, h: &mut impl EggHooks) -> u32 {
    if species >= SPECIES_LIMIT {
        return 0;
    }
    if h.item_is_tm(item) == 0 {
        return 0;
    }
    let rec = h.alloc_record(species, form, heap);
    let mut bit = h.tm_index(item);
    let word = if bit < 0x20 {
        h.record_field(rec, 0x27)
    } else if bit < 0x40 {
        let v = h.record_field(rec, 0x28);
        bit -= 0x20;
        v
    } else if bit < 0x60 {
        let v = h.record_field(rec, 0x29);
        bit -= 0x40;
        v
    } else {
        let v = h.record_field(rec, 0x2A);
        bit -= 0x60;
        v
    };
    h.free_block(rec);
    // `lsls` by a register: 32 and above give 0
    1u32.checked_shl(bit).unwrap_or(0) & word
}

/// C: `FUN_021C4238(parents, egg, heap)`: TM and HM moves the first parent knows and the baby can learn.
pub fn tm_moves(p: &Parents, e: &mut Egg, heap: u32, h: &mut impl EggHooks) {
    for id in MOVE_IDS {
        let mv = h.get(p[0], id);
        let mut item = 0x148u32;
        loop {
            if mv == (h.tm_move(item & 0xFFFF) & 0xFFFF)
                && can_learn_tm(heap, e[w::SPECIES] & 0xFFFF, e[w::FORM] & 0xFFFF, item, h) != 0
            {
                add_move(e, mv);
                break;
            }
            item += 1;
            if item > 0x272 {
                break;
            }
        }
    }
}

/// C: `FUN_021C42BC(parents, egg)`: a Pichu (172) baby gets Volt Tackle if a parent holds a Light Ball.
pub fn volt_tackle(p: &Parents, e: &mut Egg, h: &mut impl EggHooks) {
    if e[w::SPECIES] == 0xAC
        && (h.get(p[0], fid::HELD_ITEM) == LIGHT_BALL || h.get(p[1], fid::HELD_ITEM) == LIGHT_BALL)
    {
        add_move(e, VOLT_TACKLE);
    }
}

/// C: `FUN_021C4340(mon, egg, profile, place, heap)`: build the egg `Pokemon` from the work struct:
/// initialise it (level 1, the OT id and PID chosen), form, nature, egg cycles (as friendship),
/// ability, inherited IVs, moves, OT name and the egg name as nickname, the "received as an egg"
/// data, the egg flag; then recalculate stats and restore PP.
pub fn build_mon(mon: u32, e: &Egg, profile: u32, place: u32, heap: u32, h: &mut impl EggHooks) {
    h.init_pokemon(
        mon,
        e[w::SPECIES] & 0xFFFF,
        e[w::LEVEL] & 0xFFFF,
        e[w::OT_ID],
        0,
        0xFFFF_FFFF,
        e[w::PID],
        0,
    );
    h.put(mon, fid::FORM, e[w::FORM]);
    h.put(mon, fid::NATURE, e[w::NATURE]);
    h.put(mon, fid::FRIENDSHIP, e[w::EGG_CYCLES]);
    if e[w::ABILITY_ROLL] == 2 {
        h.set_hidden_ability(mon, e[w::SPECIES], e[w::FORM]);
    } else {
        h.put(mon, fid::ABILITY_PUT, e[w::ABILITY]);
    }
    for (i, id) in IV_IDS.iter().enumerate() {
        if e[w::IV_FROM_P0 + i] != 0 || e[w::IV_FROM_P1 + i] != 0 {
            h.put(mon, *id, e[w::IV + i]);
        }
    }
    for i in 0..4 {
        h.put(mon, fid::MOVE1 + i as u32, e[w::MOVES + i]);
    }
    let name = h.trainer_name(profile, heap & 0xFFFF);
    h.put(mon, fid::OT_NAME_STRBUF, name);
    h.free_strbuf(name);
    let egg_name = h.message(MSG_EGG_NAME);
    h.put(mon, fid::NICKNAME_STRBUF, egg_name);
    h.free_strbuf(egg_name);
    h.set_met_data(mon, 5, profile, place, heap & 0xFFFF);
    h.put(mon, fid::IS_EGG, 1);
    h.restore_pp(mon);
    h.recalc(mon);
    h.restore_mon(mon);
}

/// C: `FUN_021C38F4(heap, profile, place, parent0, parent1, mon)`: generate an egg from two parents
/// into the `Pokemon` at `mon`. The steps are listed in the module documentation; the order of the
/// calls (and so of the random numbers) is the original's.
pub fn generate(
    heap: u32,
    profile: u32,
    place: u32,
    parent0: u32,
    parent1: u32,
    mon: u32,
    h: &mut impl EggHooks,
) {
    let mut p: Parents = [parent0, parent1];
    let mut e: Egg = [0; 35];
    order_by_sex(&mut p, h);
    order_ditto(&mut p, h);
    init_egg(&mut e);
    set_baby_species(&p, &mut e, h);
    nidoran_gender(&p, &mut e, h);
    volbeat_illumise(&p, &mut e, h);
    phione(&p, &mut e, h);
    incense_baby(&p, &mut e, h);
    clear_form(&mut e);
    form_shellos(&p, &mut e, h);
    form_burmy(&p, &mut e, h);
    form_basculin(&p, &mut e, h);
    roll_nature(&mut e, h);
    everstone_nature(&p, &mut e, h);
    roll_ability(&p, &mut e, heap, h);
    ditto_ability(&p, &mut e, h);
    pick_ability(&mut e, heap, h);
    egg_cycles(&mut e, heap, h);
    power_item_ivs(&p, &mut e, h);
    random_ivs(&mut e, h);
    inherit_ivs(&p, &mut e, h);
    set_ot_id(profile, &mut e, h);
    roll_pid(&mut e, h);
    masuda(&p, &mut e, h);
    level1_moves(&mut e, h);
    egg_moves(&p, &mut e, heap, h);
    common_moves(&p, &mut e, h);
    tm_moves(&p, &mut e, heap, h);
    volt_tackle(&p, &mut e, h);
    build_mon(mon, &e, profile, place, heap, h);
}

#[cfg(test)]
mod tests;
