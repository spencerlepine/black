//! The day care (F13): `sodateya.c`, overlay 21 of the field set, `0x021C31D4-0x021C38F4`, plus the block
//! getter `FUN_0200BDB0` of the day care save block.
//!
//! * [`DaycareObj`] is the 0x10-byte object the field map creates ([`create`], `FUN_021C31D4`): heap id,
//!   field map, the day care save block (block 50, a handle) and the step counter of the egg roll.
//! * Deposit / withdraw ([`deposit`], [`withdraw`]), the per-step day care EXP and the egg roll ([`step`],
//!   called by the field step hook, `black_pokemon::hatch::egg_step`), the compatibility check
//!   ([`compatibility`], [`compat_pair`], [`compat_score`]), the egg pickup glue ([`take_egg`],
//!   [`refuse_egg`], [`make_egg`], which calls the P7 egg generator `egg::generate`), and the level / fee
//!   values the scripts show ([`level_now`], [`levels_gained`], [`fee`]).
//!
//! The save block (two 0xE4-byte slots: `+0` occupied, `+4` the 0xDC-byte party Pokemon, `+0xE0` EXP gained;
//! `+0x1C8` egg flag) and its main-ARM9 accessors `0x0200BC10-0x0200BDA0` are `black_storage::daycare` (S5b);
//! this crate cannot depend on black-storage (black-storage depends on black-pokemon), so they are
//! [`DaycareHooks`] methods (`save_*`, one per accessor) that the game implements with
//! `black_storage::daycare::Daycare`.
//!
//! Every callee outside the area is a [`DaycareHooks`] method (the save accessors, the Pokemon accessors, the
//! party, the growth tables, the field / game-data getters, the event-work vars, the Pokedex, the egg
//! generator); the ROM differential test stubs them identically on both sides (`tools/check-fn.sh
//! black-pokemon daycare --per-function`, spec `tools/rom-diff/specs/black-pokemon/daycare.spec`).
//!
//! Deviations: Pokemon, parties, the save block, the field map and game data are `u32` handles. The object
//! is a Rust value: [`create`] still calls the allocator hook and returns its handle with the value.
//! [`DaycareHooks::set_work_var`] is `*FUN_020143CC(ev, id) = value`.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_BDB0,
    0x021C_31D4,
    0x021C_31FC,
    0x021C_3204,
    0x021C_3248,
    0x021C_3250,
    0x021C_3270,
    0x021C_32BC,
    0x021C_32F4,
    0x021C_335C,
    0x021C_3378,
    0x021C_33A4,
    0x021C_33E0,
    0x021C_3400,
    0x021C_340C,
    0x021C_341C,
    0x021C_3490,
    0x021C_34B4,
    0x021C_34C4,
    0x021C_34F8,
    0x021C_3510,
    0x021C_3530,
    0x021C_360C,
    0x021C_3670,
    0x021C_36B0,
    0x021C_36EC,
    0x021C_37F8,
    0x021C_385C,
    0x021C_3898,
    0x021C_38DC,
];

/// Save block id of the day care (`FUN_0200BDB0` passes it to the block getter).
pub const SAVE_BLOCK_ID: u32 = 0x32;
/// Number of slots.
pub const SLOTS: u32 = 2;

/// `Pokemon` field ids used here.
mod fid {
    pub const SPECIES: u32 = 5;
    pub const OT_ID: u32 = 7;
    pub const EXP: u32 = 8;
    /// The four move ids are 0x36..=0x39 (tables at 0x021DA8C8 / 0x021DA8D8).
    pub const MOVES: [u32; 4] = [0x36, 0x37, 0x38, 0x39];
    pub const GENDER: u32 = 0x6E;
    pub const FORM: u32 = 0x6F;
    pub const LEVEL: u32 = 0x9E;
}

/// Shaymin: deposited in its Sky form it goes back to form 0.
const SHAYMIN: u32 = 0x1EC;
/// Ditto.
const DITTO: u32 = 0x84;
/// The Undiscovered egg group.
const EGG_GROUP_NONE: u32 = 15;
/// Genderless.
const GENDERLESS: u32 = 2;
/// Personal-record fields of the two egg groups.
const PERSONAL_EGG_GROUP_1: u32 = 0x18;
const PERSONAL_EGG_GROUP_2: u32 = 0x19;
/// The event-work var that says "an egg is waiting and the party has room".
pub const EGG_VAR: u32 = 0x4139;
/// Met place of an egg from the day care couple (`FUN_021C3898`'s third argument to the generator).
pub const EGG_PLACE: u32 = 0xEA62;
/// The `__FILE__` string of `sodateya.c` passed to `Heap_AllocDebug` (0x021DE458).
pub const FILE_NAME: u32 = 0x021D_E458;
/// Bytes of the learnset buffer of [`apply_exp`] (26 (move, level) halfword pairs).
pub const LEARNSET_BYTES: usize = 0x68;

/// The day care object (`FUN_021C31D4`, 0x10 bytes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DaycareObj {
    /// +0 (u16): heap id (the egg's heap).
    pub heap: u16,
    /// +4: the field map.
    pub field: u32,
    /// +8: the day care save block (block 50, a handle).
    pub save: u32,
    /// +0xC (u16): the egg-roll step counter ([`egg_roll`]).
    pub counter: u16,
}

/// The callees of `sodateya.c` outside the area (one method per original routine).
pub trait DaycareHooks {
    /// `FUN_0200BC10(save)`: egg flag = 1 (`black_storage::daycare::Daycare::set_egg`).
    fn save_set_egg(&mut self, save: u32);
    /// `FUN_0200BC20(save)`: egg flag = 0 (`Daycare::clear_egg`).
    fn save_clear_egg(&mut self, save: u32);
    /// `FUN_0200BC2C(save, slot)`: the Pokemon of a slot (slot 0 unless valid and occupied; `Daycare::mon`).
    fn save_mon(&mut self, save: u32, slot: u32) -> u32;
    /// `FUN_0200BC4C(save, slot, mon)`: store a party Pokemon (`Daycare::put`).
    fn save_put(&mut self, save: u32, slot: u32, mon: u32);
    /// `FUN_0200BC88(save, slot)`: free a slot (`Daycare::take`).
    fn save_take(&mut self, save: u32, slot: u32);
    /// `FUN_0200BCA0(save)`: the egg flag (`Daycare::egg`).
    fn save_egg(&mut self, save: u32) -> u32;
    /// `FUN_0200BCA8(save, slot)`: the occupied word (`Daycare::occupied`).
    fn save_occupied(&mut self, save: u32, slot: u32) -> u32;
    /// `FUN_0200BCB8(save)`: number of occupied slots (`Daycare::count`).
    fn save_count(&mut self, save: u32) -> u32;
    /// `FUN_0200BCD8(save, slot)`: EXP gained in the day care (`Daycare::exp`).
    fn save_exp(&mut self, save: u32, slot: u32) -> u32;
    /// `FUN_0200BCEC(save, a, b)`: swap two slots (`Daycare::swap`).
    fn save_swap(&mut self, save: u32, a: u32, b: u32);
    /// `FUN_0200BD48(save, n)`: add EXP to every occupied slot, saturating (`Daycare::add_exp`).
    fn save_add_exp(&mut self, save: u32, n: u32);
    /// `FUN_02017E1C(mon, id, 0)`: a Pokemon field.
    fn get(&mut self, mon: u32, id: u32) -> u32;
    /// `FUN_02017E40(mon, id, value)`: set a Pokemon field.
    fn put(&mut self, mon: u32, id: u32, value: u32);
    /// `FUN_0201AB28(party, index)`: a party member.
    fn party_mon(&mut self, party: u32, index: u32) -> u32;
    /// `FUN_0201A9C8(party, index)`: remove a party member.
    fn party_remove(&mut self, party: u32, index: u32);
    /// `FUN_0201A98C(party, mon)`: add a Pokemon to the party.
    fn party_add(&mut self, party: u32, mon: u32);
    /// `FUN_0201AA18(party)`: the party count.
    fn party_count(&mut self, party: u32) -> u32;
    /// `FUN_0201AA14(party)`: the party capacity.
    fn party_max(&mut self, party: u32) -> u32;
    /// `FUN_0201B9F0(mon)` (through the veneer `FUN_021C3248`): restore the PP.
    fn restore_pp(&mut self, mon: u32);
    /// `FUN_02017988(mon, form)`: set the form.
    fn set_form(&mut self, mon: u32, form: u32);
    /// `FUN_02018584(species, form, level)`: the EXP of a level.
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32;
    /// `FUN_0201850C(species, form, exp)`: the level of an EXP total.
    fn level_from_exp(&mut self, species: u32, form: u32, exp: u32) -> u32;
    /// `FUN_020185C4(mon)`: recalculate level and stats.
    fn recalc(&mut self, mon: u32);
    /// `FUN_0201ADD0(species, form, buf)`: the level-up learnset as (move, level) halfword pairs ending
    /// with (0xFFFF, 0xFFFF).
    fn load_learnset(&mut self, species: u32, form: u32, out: &mut [u8; LEARNSET_BYTES]);
    /// `FUN_020181F4(mon, move, slot)` (thunk to `FUN_020181FC`): put a move into a slot.
    fn set_move(&mut self, mon: u32, mv: u32, slot: u32);
    /// `FUN_020180F4(mon, move)` (thunk to `FUN_020180FC`): learn a move, pushing out the first.
    fn push_move(&mut self, mon: u32, mv: u32);
    /// `FUN_02019C90(species, form, field)`: a personal-record field.
    fn personal(&mut self, species: u32, form: u32, field: u32) -> u32;
    /// `FUN_02005728(n)`: a random number below `n`.
    fn rand(&mut self, n: u32) -> u32;
    /// `FUN_02188C80(field map)`: the game system.
    fn field_gamesys(&mut self, fm: u32) -> u32;
    /// `FUN_020120F4(gamesys)`: the game data.
    fn gamedata(&mut self, gs: u32) -> u32;
    /// `FUN_0201296C(gamedata)`: the event work.
    fn event_work(&mut self, gd: u32) -> u32;
    /// `FUN_02012934(gamedata)`: the party.
    fn party(&mut self, gd: u32) -> u32;
    /// `FUN_0200CA1C(gamedata)`: the Pokedex.
    fn pokedex(&mut self, gd: u32) -> u32;
    /// `FUN_02012944(gamedata)`: the player profile (the egg's OT).
    fn profile(&mut self, gd: u32) -> u32;
    /// `FUN_02012F98(gamedata)` (UNVERIFIED: the record that `FUN_02007C34` updates).
    fn gamedata_02012f98(&mut self, gd: u32) -> u32;
    /// `FUN_02012984(gamedata)`: the season (a byte).
    fn season(&mut self, gd: u32) -> u32;
    /// `FUN_02007C34(x, party)` (UNVERIFIED: a record updated after the party changed).
    fn party_changed(&mut self, x: u32, party: u32);
    /// `*(u16 *)FUN_020143CC(event work, id) = value`: set an event-work var.
    fn set_work_var(&mut self, ev: u32, id: u32, value: u16);
    /// `FUN_0200CED4(pokedex, species)`: species registered.
    fn dex_has(&mut self, dex: u32, species: u32) -> u32;
    /// `FUN_0200CDE0(pokedex, mon)`: register a Pokemon.
    fn dex_register(&mut self, dex: u32, mon: u32);
    /// `FUN_020173F0(1, 1, 1, 0, heap)`: a new Pokemon (the egg).
    fn new_pokemon(&mut self, a: u32, b: u32, c: u32, d: u32, heap: u32) -> u32;
    /// `Heap_AllocDebug(heap, size, flag, file, line)` (`FUN_02030734`).
    fn alloc(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// `Heap_Free(block)` (`FUN_020307B0`).
    fn free(&mut self, block: u32);
    /// `FUN_0216556C()` (overlay 10; UNVERIFIED: an effect of the egg handover).
    fn egg_received(&mut self);
    /// `FUN_0216AAE4(gamedata, party, season)`: the seasonal forms of the party.
    fn seasonal_forms(&mut self, gd: u32, party: u32, season: u32);
    /// `FUN_021C38F4(heap, profile, place, parent0, parent1, egg)`: the egg generator (P7 `egg::generate`).
    fn generate_egg(&mut self, heap: u32, profile: u32, place: u32, p0: u32, p1: u32, egg: u32);
    /// `FUN_020071CC(save, id)`: a save block.
    fn save_block(&mut self, save: u32, id: u32) -> u32;
}

/// C: `FUN_0200BDB0(save data)`: the day care save block (`FUN_020071CC(save data, 0x32)`).
pub fn save_block<H: DaycareHooks>(savedata: u32, h: &mut H) -> u32 {
    h.save_block(savedata, SAVE_BLOCK_ID)
}

/// C: `FUN_021C3510(a, b)`: `a + b`, saturating at 0xFFFF_FFFF.
pub fn sat_add(a: u32, b: u32) -> u32 {
    a.saturating_add(b)
}

/// C: `FUN_021C31D4(heap, field map, save)`: allocate the object (`Heap_AllocDebug(heap, 0x10, 0,
/// "sodateya.c", 0x6A)`) and fill it; returns the block handle and the object.
pub fn create<H: DaycareHooks>(heap: u32, field: u32, save: u32, h: &mut H) -> (u32, DaycareObj) {
    let block = h.alloc(heap, 0x10, 0, FILE_NAME, 0x6A);
    (
        block,
        DaycareObj {
            heap: heap as u16,
            field,
            save,
            counter: 0,
        },
    )
}

/// C: `FUN_021C31FC(obj)`: free the object.
pub fn delete<H: DaycareHooks>(obj: u32, h: &mut H) {
    h.free(obj);
}

/// C: `FUN_021C3248(mon)`: veneer to `FUN_0201B9F0` (restore PP).
pub fn restore_pp<H: DaycareHooks>(mon: u32, h: &mut H) {
    h.restore_pp(mon);
}

/// C: `FUN_021C3250(mon)`: Shaymin goes back to form 0.
pub fn reset_form<H: DaycareHooks>(mon: u32, h: &mut H) {
    if h.get(mon, fid::SPECIES) == SHAYMIN {
        h.set_form(mon, 0);
    }
}

/// C: `FUN_021C3204(obj, party, index)`: deposit party member `index` into the next free slot (only with
/// fewer than two occupied): PP restore, Shaymin form, store, remove from the party, [`party_changed`].
pub fn deposit<H: DaycareHooks>(dc: &DaycareObj, party: u32, index: u32, h: &mut H) {
    let n = h.save_count(dc.save);
    let mon = h.party_mon(party, index);
    if (n as i32) < 2 {
        restore_pp(mon, h);
        reset_form(mon, h);
        h.save_put(dc.save, n, mon);
        h.party_remove(party, index);
        party_changed(dc.field, party, h);
    }
}

/// C: `FUN_021C3270(obj, slot, party)`: withdraw an occupied slot: day care EXP and moves
/// ([`apply_exp`]), Pokedex ([`register_dex`]), add to the party, free the slot, [`compact`].
pub fn withdraw<H: DaycareHooks>(dc: &DaycareObj, slot: u32, party: u32, h: &mut H) {
    if h.save_occupied(dc.save, slot) != 1 {
        return;
    }
    let mon = h.save_mon(dc.save, slot);
    let exp = h.save_exp(dc.save, slot);
    apply_exp(mon, exp, h);
    register_dex(dc, mon, h);
    h.party_add(party, mon);
    h.save_take(dc.save, slot);
    compact(dc.save, h);
}

/// C: `FUN_021C32BC(obj, mon)`: register the species in the Pokedex unless it already is.
pub fn register_dex<H: DaycareHooks>(dc: &DaycareObj, mon: u32, h: &mut H) {
    let gs = h.field_gamesys(dc.field);
    let gd = h.gamedata(gs);
    let dex = h.pokedex(gd);
    let species = h.get(mon, fid::SPECIES) & 0xFFFF;
    if h.dex_has(dex, species) == 0 {
        h.dex_register(dex, mon);
    }
}

/// C: `FUN_021C32F4(obj, party)`: take the egg (if the egg flag is 1): a new Pokemon filled by [`make_egg`],
/// added to the party; the egg flag and [`EGG_VAR`] cleared; the seasonal forms refreshed. The new Pokemon is
/// allocated before the test and only freed when an egg was handed over.
pub fn take_egg<H: DaycareHooks>(dc: &DaycareObj, party: u32, h: &mut H) {
    let egg = h.new_pokemon(1, 1, 1, 0, u32::from(dc.heap));
    if h.save_egg(dc.save) != 1 {
        return;
    }
    h.egg_received();
    clear_egg_var(dc.field, h);
    make_egg(dc, egg, h);
    h.party_add(party, egg);
    h.save_clear_egg(dc.save);
    h.free(egg);
    let gs = h.field_gamesys(dc.field);
    let gd = h.gamedata(gs);
    let season = h.season(gd);
    h.seasonal_forms(gd, party, season);
}

/// C: `FUN_021C335C(obj)`: refuse the egg (if the egg flag is 1): clear it and [`EGG_VAR`].
pub fn refuse_egg<H: DaycareHooks>(dc: &DaycareObj, h: &mut H) {
    if h.save_egg(dc.save) == 1 {
        h.save_clear_egg(dc.save);
        clear_egg_var(dc.field, h);
    }
}

/// C: `FUN_021C3378(obj)`: the day care step (from the field step hook): one EXP point for every occupied
/// slot, then [`egg_roll`]; an egg sets the egg flag and [`set_egg_var`]. Returns 1 on an egg.
pub fn step<H: DaycareHooks>(dc: &mut DaycareObj, h: &mut H) -> u32 {
    h.save_add_exp(dc.save, 1);
    if egg_roll(dc, h) == 1 {
        h.save_set_egg(dc.save);
        set_egg_var(dc.field, h);
        return 1;
    }
    0
}

/// C: `FUN_021C33A4(field map)`: [`EGG_VAR`] = 1 if the party is not full.
pub fn set_egg_var<H: DaycareHooks>(fm: u32, h: &mut H) {
    let gs = h.field_gamesys(fm);
    let gd = h.gamedata(gs);
    let ev = h.event_work(gd);
    let party = h.party(gd);
    let n = h.party_count(party) as i32;
    if n < h.party_max(party) as i32 {
        h.set_work_var(ev, EGG_VAR, 1);
    }
}

/// C: `FUN_021C33E0(field map)`: [`EGG_VAR`] = 0.
pub fn clear_egg_var<H: DaycareHooks>(fm: u32, h: &mut H) {
    let gs = h.field_gamesys(fm);
    let gd = h.gamedata(gs);
    let ev = h.event_work(gd);
    h.set_work_var(ev, EGG_VAR, 0);
}

/// C: `FUN_021C3400(obj)`: the egg flag.
pub fn egg_ready<H: DaycareHooks>(dc: &DaycareObj, h: &mut H) -> u32 {
    h.save_egg(dc.save)
}

/// C: `FUN_021C340C(obj)`: number of occupied slots.
pub fn count<H: DaycareHooks>(dc: &DaycareObj, h: &mut H) -> u8 {
    h.save_count(dc.save) as u8
}

/// C: `FUN_021C341C(obj, slot)`: the level the slot's Pokemon has with its day care EXP (1 for a slot that is
/// not occupied).
pub fn level_now<H: DaycareHooks>(dc: &DaycareObj, slot: u32, h: &mut H) -> u32 {
    if h.save_occupied(dc.save, slot) != 1 {
        return 1;
    }
    let mon = h.save_mon(dc.save, slot);
    let species = h.get(mon, fid::SPECIES);
    let form = h.get(mon, fid::FORM);
    let exp = h.get(mon, fid::EXP);
    h.get(mon, fid::LEVEL);
    let gained = h.save_exp(dc.save, slot);
    let total = sat_add(exp, gained);
    h.level_from_exp(species & 0xFFFF, form & 0xFFFF, total)
}

/// C: `FUN_021C3490(obj, slot)`: levels gained, [`level_now`] minus the stored level.
pub fn levels_gained<H: DaycareHooks>(dc: &DaycareObj, slot: u32, h: &mut H) -> u32 {
    let mon = h.save_mon(dc.save, slot);
    let level = h.get(mon, fid::LEVEL);
    level_now(dc, slot, h).wrapping_sub(level)
}

/// C: `FUN_021C34B4(obj, slot)`: the fee, (levels gained + 1) * 100.
pub fn fee<H: DaycareHooks>(dc: &DaycareObj, slot: u32, h: &mut H) -> u32 {
    levels_gained(dc, slot, h).wrapping_add(1).wrapping_mul(100)
}

/// C: `FUN_021C34C4(obj)`: the compatibility of the two slots ([`compat_pair`]; 0 unless both are occupied).
pub fn compatibility<H: DaycareHooks>(dc: &DaycareObj, h: &mut H) -> u8 {
    if (h.save_count(dc.save) as i32) < 2 {
        return 0;
    }
    let a = h.save_mon(dc.save, 0);
    let b = h.save_mon(dc.save, 1);
    compat_pair(a, b, h) as u8
}

/// C: `FUN_021C34F8(obj, slot)`: the Pokemon of a slot (the original also reads the occupied word and ignores
/// it).
pub fn slot_mon<H: DaycareHooks>(dc: &DaycareObj, slot: u32, h: &mut H) -> u32 {
    h.save_occupied(dc.save, slot);
    h.save_mon(dc.save, slot)
}

/// C: `FUN_021C3530(mon, exp)`: the day care EXP: `exp = min(exp + gained (saturating), EXP of level 100)`,
/// recalc, then [`learn_move`] every level-up move whose level is above the old level and not above the new
/// one.
pub fn apply_exp<H: DaycareHooks>(mon: u32, gained: u32, h: &mut H) {
    let species = h.get(mon, fid::SPECIES);
    let form = h.get(mon, fid::FORM);
    let max = h.exp_for_level(species & 0xFFFF, form & 0xFFFF, 100);
    let old = h.get(mon, fid::LEVEL) as i32;
    let exp = h.get(mon, fid::EXP);
    let total = sat_add(exp, gained);
    h.put(mon, fid::EXP, total.min(max));
    h.recalc(mon);
    let new = h.get(mon, fid::LEVEL) as i32;
    let mut buf = [0u8; LEARNSET_BYTES];
    h.load_learnset(species & 0xFFFF, form & 0xFFFF, &mut buf);
    // Deviation: the original walks past the 26 pairs on the stack when no terminator is found; a real
    // learnset always ends inside the buffer.
    for e in buf.as_chunks::<4>().0 {
        let mv = u32::from(u16::from_le_bytes([e[0], e[1]]));
        let level = u32::from(u16::from_le_bytes([e[2], e[3]]));
        if mv == 0xFFFF && level == 0xFFFF {
            break;
        }
        if new < level as i32 {
            break;
        }
        if old < level as i32 {
            learn_move(mon, mv, h);
        }
    }
}

/// C: `FUN_021C360C(mon, move)`: learn `move` unless it is 0 or known ([`knows_move`]): into the first empty
/// move slot, else push out the first move.
pub fn learn_move<H: DaycareHooks>(mon: u32, mv: u32, h: &mut H) {
    if mv == 0 || knows_move(mon, mv, h) != 0 {
        return;
    }
    for (slot, &id) in fid::MOVES.iter().enumerate() {
        if h.get(mon, id) == 0 {
            h.set_move(mon, mv & 0xFFFF, slot as u32);
            return;
        }
    }
    h.push_move(mon, mv & 0xFFFF);
}

/// C: `FUN_021C3670(mon, move)`: 1 if one of the four moves is `move`.
pub fn knows_move<H: DaycareHooks>(mon: u32, mv: u32, h: &mut H) -> u32 {
    for &id in &fid::MOVES {
        if h.get(mon, id) == mv {
            return 1;
        }
    }
    0
}

/// C: `FUN_021C36B0(save)`: move an occupied slot down into an empty one before it.
pub fn compact<H: DaycareHooks>(save: u32, h: &mut H) {
    for i in 0..SLOTS {
        if h.save_occupied(save, i) == 1 {
            continue;
        }
        for j in i + 1..SLOTS {
            if h.save_occupied(save, j) == 1 {
                h.save_swap(save, j, j - 1);
            }
        }
    }
}

/// C: `FUN_021C36EC(a, b)`: compatibility of two Pokemon: 0 if either is in the Undiscovered egg group, both
/// are Ditto, the genders are equal or one is genderless, or no egg group is shared; else [`compat_score`] (a
/// Ditto pairs with anything breedable).
pub fn compat_pair<H: DaycareHooks>(a: u32, b: u32, h: &mut H) -> u32 {
    let mut species = [0u32; 2];
    let mut gender = [0u32; 2];
    let mut group1 = [0u32; 2];
    let mut group2 = [0u32; 2];
    for (i, &m) in [a, b].iter().enumerate() {
        species[i] = h.get(m, fid::SPECIES);
        h.get(m, fid::OT_ID);
        let form = h.get(m, fid::FORM);
        gender[i] = h.get(m, fid::GENDER);
        let (s, f) = (species[i] & 0xFFFF, form & 0xFFFF);
        group1[i] = h.personal(s, f, PERSONAL_EGG_GROUP_1);
        group2[i] = h.personal(s, f, PERSONAL_EGG_GROUP_2);
    }
    if group1.contains(&EGG_GROUP_NONE) || group2.contains(&EGG_GROUP_NONE) {
        return 0;
    }
    if species[0] == DITTO && species[1] == DITTO {
        return 0;
    }
    if species[0] == DITTO || species[1] == DITTO {
        return compat_score(a, b, h);
    }
    if gender[0] == gender[1] || gender[0] == GENDERLESS || gender[1] == GENDERLESS {
        return 0;
    }
    if group1[0] == group1[1]
        || group1[0] == group2[1]
        || group2[0] == group1[1]
        || group2[0] == group2[1]
    {
        return compat_score(a, b, h);
    }
    0
}

/// C: `FUN_021C37F8(a, b)`: same species: 50 (same OT) / 70; different species: 20 (same OT) / 50.
pub fn compat_score<H: DaycareHooks>(a: u32, b: u32, h: &mut H) -> u32 {
    let mut species = [0u32; 2];
    let mut ot = [0u32; 2];
    for (i, &m) in [a, b].iter().enumerate() {
        species[i] = h.get(m, fid::SPECIES);
        ot[i] = h.get(m, fid::OT_ID);
    }
    match (species[0] == species[1], ot[0] == ot[1]) {
        (true, true) => 50,
        (true, false) => 70,
        (false, true) => 20,
        (false, false) => 50,
    }
}

/// C: `FUN_021C385C(obj)`: the egg roll: the u16 counter goes up by 0x10 per step; past 0xFF it restarts, and
/// with both slots occupied an egg comes when `rand(1600)` is below [`compatibility`]. Returns 1 on an egg.
pub fn egg_roll<H: DaycareHooks>(dc: &mut DaycareObj, h: &mut H) -> u32 {
    let old = dc.counter;
    dc.counter = old.wrapping_add(0x10);
    if old <= 0xFF {
        return 0;
    }
    dc.counter = 0;
    if (h.save_count(dc.save) as i32) < 2 {
        return 0;
    }
    let r = h.rand(0x640);
    if r < u32::from(compatibility(dc, h)) {
        return 1;
    }
    0
}

/// C: `FUN_021C3898(obj, egg)`: fill `egg` from the two slots with the egg generator
/// (`FUN_021C38F4(heap, profile, 0xEA62, parent 0, parent 1, egg)`).
pub fn make_egg<H: DaycareHooks>(dc: &DaycareObj, egg: u32, h: &mut H) {
    let gs = h.field_gamesys(dc.field);
    let gd = h.gamedata(gs);
    let profile = h.profile(gd);
    let p0 = h.save_mon(dc.save, 0);
    let p1 = h.save_mon(dc.save, 1);
    h.generate_egg(u32::from(dc.heap), profile, EGG_PLACE, p0, p1, egg);
}

/// C: `FUN_021C38DC(field map, party)`: `FUN_02007C34(FUN_02012F98(game data), party)` (UNVERIFIED role).
pub fn party_changed<H: DaycareHooks>(fm: u32, party: u32, h: &mut H) {
    let gs = h.field_gamesys(fm);
    let gd = h.gamedata(gs);
    let x = h.gamedata_02012f98(gd);
    h.party_changed(x, party);
}

#[cfg(test)]
mod tests;
