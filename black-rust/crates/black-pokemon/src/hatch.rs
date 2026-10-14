//! Egg hatching logic (P7b): the per-step egg-cycle countdown, the "is an egg ready to hatch" check and
//! the hatch conversion of the egg Pokemon.
//!
//! Where it lives in the ROM:
//!
//! * overlay 21 (field set), `0x0218A9B0-0x0218AB9C`: the field step hook. [`step_hook`]
//!   (`FUN_0218A9B0`) is called on every player step; when the step flag (bit 2 of word 0x3C of its
//!   config) is set it calls [`egg_step`] (`FUN_0218AB50`), which runs the day care step, then
//!   [`step_counter`] (`FUN_0218AAD0`) and then [`hatch_ready`] (`FUN_0218AB0C`); a ready egg starts
//!   the hatch event script 0x8CA.
//! * [`step_counter`] adds `0x100` (scaled by the game's step-rate global) to a counter stored in
//!   the save-side struct (+0x78); above `0x10000` it runs [`decrement_eggs`] (`FUN_0218AA74`) and
//!   resets the counter. [`decrement_eggs`] lowers the friendship field (id 9, the egg cycle counter
//!   of an egg) of every egg in the party by [`step_decrement`] (`FUN_0218AA2C`): 2 if a party member
//!   that is not an egg has Magma Armor (0x28) or Flame Body (0x31), else 1; it stops at 0.
//! * [`hatch_ready`]: any party Pokemon with the egg flag (id 0x4C) set and friendship 0.
//! * main ARM9 `0x02017B24` / `0x02017B4C` (in a data-looking region Ghidra left undisassembled, so the
//!   decompiled sets have no C for them): the hatch conversion, called by the egg demo (overlay 215
//!   `FUN_021F598A`, case 7 of its state machine, after the name is shown): [`hatch`] /
//!   [`hatch_body`]. It clears the egg flag, sets friendship to 120, copies the species name message into
//!   the nickname (put id 0x74, which also clears the nickname flag state), and records the hatch place
//!   and date (met kind 2).
//!
//! Not here: overlay 215 itself (the scene), the script that picks the egg and starts the scene, the
//! day care step (`FUN_021C3378`, F13) and the friendship step bonus (`FUN_0218AB9C`).
//!
//! Every callee outside the area is a [`HatchHooks`] method; the ROM differential test stubs them
//! identically on both sides (`tools/check-fn.sh black-pokemon hatch --per-function`, spec
//! `tools/rom-diff/specs/black-pokemon/hatch.spec`).

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_A9B0,
    0x0218_AA2C,
    0x0218_AA74,
    0x0218_AAD0,
    0x0218_AB0C,
    0x0218_AB50,
    0x0201_7B24,
    0x0201_7B4C,
];

/// `Pokemon` field ids used here.
mod fid {
    pub const SPECIES: u32 = 5;
    pub const FRIENDSHIP: u32 = 9;
    pub const ABILITY: u32 = 0xA;
    pub const IS_EGG: u32 = 0x4C;
    pub const FORM: u32 = 0x6F;
    /// Nickname from a string buffer (the setter that also records the nickname).
    pub const NICKNAME_PUT: u32 = 0x74;
}

/// Magma Armor.
const MAGMA_ARMOR: u32 = 0x28;
/// Flame Body.
const FLAME_BODY: u32 = 0x31;
/// The step counter threshold (`> 0x10000` runs one egg-cycle step).
const STEP_LIMIT: u32 = 0x10000;
/// The step increment before scaling by the step-rate global.
const STEP_UNIT: u32 = 0x100;
/// Hatch event script number (`FUN_02158768(fsys, 0x8CA, 0, ..)`).
pub const HATCH_EVENT: u32 = 0x8CA;
/// Friendship of a freshly hatched Pokemon.
const HATCHED_FRIENDSHIP: u32 = 0x78;
/// Met kind passed to the met-data setter for a hatch.
const MET_HATCHED: u32 = 2;
/// The shared string buffer the species name is copied into (a global at 0x021462F0).
pub const NAME_BUF: u32 = 0x0214_62F0;
/// Size of the name copy (characters).
const NAME_LEN: u32 = 0x20;

/// The words of the step hook's config struct that [`step_hook`] reads (`r5` of `FUN_0218A9B0`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StepCfg {
    /// Word at +0x08 (passed to the friendship step routine).
    pub word08: u32,
    /// Halfword at +0x10 (passed to the poison-step routine).
    pub half10: u32,
    /// Word at +0x3C: bit 2 = run the step routines, bit 9 = also run the "fast" handler.
    pub flags: u32,
}

/// Callees outside the area. Pointers into the game are opaque `u32` handles; the ROM differential
/// test stubs every one of these identically on both sides.
pub trait HatchHooks {
    /// `FUN_02017E1C(mon, id, 0)`: get a field of a `Pokemon`.
    fn get(&mut self, mon: u32, id: u32) -> u32;
    /// `FUN_02017E40(mon, id, value)`: set a field of a `Pokemon`.
    fn put(&mut self, mon: u32, id: u32, value: u32);
    /// `FUN_0201AA18(party)`: number of Pokemon in a party.
    fn party_count(&mut self, party: u32) -> u32;
    /// `FUN_0201AB28(party, index)`: a party member.
    fn party_mon(&mut self, party: u32, index: u32) -> u32;
    /// `FUN_02012EBC(fsys)`: a field system sub-object (feeds `FUN_02008854`).
    fn fsys_save(&mut self, fsys: u32) -> u32;
    /// `FUN_02008854(save)`: the save block the step counter lives in.
    fn step_block(&mut self, save: u32) -> u32;
    /// `FUN_02008970(block, &out)`: read the step counter (word at +0x78).
    fn counter_get(&mut self, block: u32) -> u32;
    /// `FUN_0200896C(block, value)`: write the step counter.
    fn counter_set(&mut self, block: u32, value: u32);
    /// `FUN_02010FF4(0x100)`: the step increment scaled by the step-rate global.
    fn scaled_step(&mut self, unit: u32) -> u32;
    /// `FUN_02188C84(work)`: halfword at +0 of the step work.
    fn work_half(&mut self, work: u32) -> u32;
    /// `FUN_02188D50(work)`: word at +0x128 of the step work (the day care).
    fn work_daycare(&mut self, work: u32) -> u32;
    /// `FUN_021C3378(daycare)`: the day care step (F13).
    fn daycare_step(&mut self, daycare: u32);
    /// `FUN_02012934(fsys)`: the party of the field system.
    fn fsys_party(&mut self, fsys: u32) -> u32;
    /// `FUN_02158768(fsys, event, 0, arg)`: start an event script; returns its result.
    fn start_event(&mut self, fsys: u32, event: u32, zero: u32, arg: u32) -> u32;
    /// `FUN_02188C80(work)`: the step work's second field (feeds `FUN_020120F4`).
    fn work_field(&mut self, work: u32) -> u32;
    /// `FUN_020120F4(x)`: the field system of a step work field.
    fn field_fsys(&mut self, x: u32) -> u32;
    /// `FUN_0218AC84(work, a, fsys)`: the fast handler (bit 9 / bit 2); nonzero ends the step.
    fn fast_handler(&mut self, work: u32, a: u32, fsys: u32) -> u32;
    /// `FUN_0218AB9C(word08, work)`: the friendship step bonus.
    fn friendship_step(&mut self, word08: u32, work: u32);
    /// `FUN_0218AC08(a, work, half)`: the poison step handler; nonzero ends the step.
    fn poison_step(&mut self, a: u32, work: u32, half: u32) -> u32;
    /// `FUN_0218AC4C(work, a, fsys)`: the last step handler.
    fn last_handler(&mut self, work: u32, a: u32, fsys: u32) -> u32;
    /// `FUN_02017D30(mon)`: decrypt the box part (returns whether it was changed).
    fn unlock_box(&mut self, mon: u32) -> u32;
    /// `FUN_02017D70(mon, flag)`: undo [`HatchHooks::unlock_box`].
    fn relock_box(&mut self, mon: u32, flag: u32);
    /// `FUN_02017DBC(mon)`: decrypt the party part (returns whether it was changed).
    fn unlock_party(&mut self, mon: u32) -> u32;
    /// `FUN_02017DE4(mon, flag)`: undo [`HatchHooks::unlock_party`].
    fn relock_party(&mut self, mon: u32, flag: u32);
    /// `FUN_02017E88(mon, id, 0)`: get a field of the decrypted box part.
    fn box_get(&mut self, mon: u32, id: u32) -> u32;
    /// `FUN_02017E64(mon, id, value)`: set a field of the decrypted box part.
    fn box_put(&mut self, mon: u32, id: u32, value: u32);
    /// `FUN_02045F5C(*0x020A7370, message, buf, len)`: copy a message (the species name) into a buffer.
    fn message_into(&mut self, message: u32, buf: u32, len: u32);
    /// `FUN_0202F1B4(mon, kind, a, b, c)`: record where and when the Pokemon was received / hatched.
    fn set_met(&mut self, mon: u32, kind: u32, a: u32, b: u32, c: u32);
}

/// C: `FUN_0218AA2C(party)`: the egg-cycle step: 2 if a Pokemon that is not an egg has Magma Armor or
/// Flame Body, else 1.
pub fn step_decrement(party: u32, h: &mut impl HatchHooks) -> u32 {
    let count = h.party_count(party) as i32;
    let mut i = 0;
    while i < count {
        let mon = h.party_mon(party, i as u32);
        let ability = h.get(mon, fid::ABILITY);
        if h.get(mon, fid::IS_EGG) == 0 && (ability == FLAME_BODY || ability == MAGMA_ARMOR) {
            return 2;
        }
        i += 1;
    }
    1
}

/// C: `FUN_0218AA74(party)`: lower the egg cycles (the friendship field) of every egg by the step,
/// down to 0.
pub fn decrement_eggs(party: u32, h: &mut impl HatchHooks) {
    let count = h.party_count(party) as i32;
    let dec = step_decrement(party, h);
    let mut i = 0;
    while i < count {
        let mon = h.party_mon(party, i as u32);
        let egg = h.get(mon, fid::IS_EGG);
        let cycles = h.get(mon, fid::FRIENDSHIP);
        if egg == 1 {
            let left = cycles.saturating_sub(dec);
            h.put(mon, fid::FRIENDSHIP, left);
        }
        i += 1;
    }
}

/// C: `FUN_0218AAD0(fsys, party, _, _)`: one player step: add the scaled step to the counter; above
/// `0x10000` run [`decrement_eggs`] and restart the counter at 0.
pub fn step_counter(fsys: u32, party: u32, h: &mut impl HatchHooks) {
    let save = h.fsys_save(fsys);
    let block = h.step_block(save);
    let mut counter = h.counter_get(block);
    let inc = h.scaled_step(STEP_UNIT);
    counter = counter.wrapping_add(inc);
    if counter > STEP_LIMIT {
        decrement_eggs(party, h);
        counter = 0;
    }
    h.counter_set(block, counter);
}

/// C: `FUN_0218AB0C(party)`: 1 if a party Pokemon is an egg with 0 cycles left, else 0.
pub fn hatch_ready(party: u32, h: &mut impl HatchHooks) -> u32 {
    let count = h.party_count(party) as i32;
    let mut i = 0;
    while i < count {
        let mon = h.party_mon(party, i as u32);
        let egg = h.get(mon, fid::IS_EGG);
        let cycles = h.get(mon, fid::FRIENDSHIP);
        if egg == 1 && cycles == 0 {
            return 1;
        }
        i += 1;
    }
    0
}

/// C: `FUN_0218AB50(_, work, field, fsys)`: the egg part of a step: day care step, egg-cycle step, and
/// the hatch event if an egg is ready. `field` (`r2`) is the first argument of the event start.
pub fn egg_step(work: u32, field: u32, fsys: u32, h: &mut impl HatchHooks) -> u32 {
    let half = h.work_half(work);
    let party = h.fsys_party(fsys);
    let daycare = h.work_daycare(work);
    h.daycare_step(daycare);
    step_counter(fsys, party, h);
    if hatch_ready(party, h) != 0 {
        return h.start_event(field, HATCH_EVENT, 0, half);
    }
    0
}

/// C: `FUN_0218A9B0(cfg, work)`: the field step hook. `work` is the step work (`r1`); the flags are
/// read once (the original re-reads word 0x3C after the fast handler, which the stubs never change).
pub fn step_hook(cfg: &StepCfg, work: u32, h: &mut impl HatchHooks) -> u32 {
    h.work_half(work);
    let field = h.work_field(work);
    let fsys = h.field_fsys(field);
    let bit2 = (cfg.flags >> 2) & 1;
    let bit9 = (cfg.flags >> 9) & 1;
    if bit2 != 0 || bit9 != 0 {
        let r = h.fast_handler(work, field, fsys);
        if r != 0 {
            return r;
        }
    }
    if bit2 == 0 {
        return 0;
    }
    h.friendship_step(cfg.word08, work);
    let r = h.poison_step(field, work, cfg.half10 & 0xFFFF);
    if r != 0 {
        return r;
    }
    let r = egg_step(work, field, fsys, h);
    if r != 0 {
        return r;
    }
    h.last_handler(work, field, fsys)
}

/// C: `FUN_02017B4C(mon, a, b, c)`: the hatch conversion proper.
pub fn hatch_body(mon: u32, a: u32, b: u32, c: u32, h: &mut impl HatchHooks) {
    let party_flag = h.unlock_party(mon) & 0xFF;
    let species = h.box_get(mon, fid::SPECIES) & 0xFFFF;
    h.box_get(mon, fid::FORM);
    h.box_put(mon, fid::IS_EGG, 0);
    h.box_put(mon, fid::FRIENDSHIP, HATCHED_FRIENDSHIP);
    h.message_into(species, NAME_BUF, NAME_LEN);
    h.box_put(mon, fid::NICKNAME_PUT, NAME_BUF);
    h.set_met(mon, MET_HATCHED, a, b, c);
    h.relock_party(mon, party_flag);
}

/// C: `FUN_02017B24(mon, a, b, c)`: hatch a Pokemon: [`hatch_body`] with the box part decrypted. `a` is
/// the profile, `b` the place, `c` the heap id.
pub fn hatch(mon: u32, a: u32, b: u32, c: u32, h: &mut impl HatchHooks) {
    let box_flag = h.unlock_box(mon) & 0xFF;
    hatch_body(mon, a, b, c, h);
    h.relock_box(mon, box_flag);
}

#[cfg(test)]
mod tests;
