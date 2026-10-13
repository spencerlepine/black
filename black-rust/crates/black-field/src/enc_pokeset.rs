//! The encounter Pokemon set-up of overlay 21 (`0x021A9350..0x021A9ED8`; the original enc_pokeset.c, counted by
//! F1 in row 21.28 "field_menu.c"): the encounter context (EFP) of the lead Pokemon, the rate modifiers, the slot
//! group of an encounter type, the slot pick, the level / form roll, the ability effects, the swarm and WFBC
//! encounters and the POKEPARAM set-up of a wild or roaming Pokemon.
//!
//! F5a-spec specced all 25 functions (`tools/rom-diff/specs/black-field/enc_pokeset.spec`, whose header documents
//! the EFP (0x2C bytes) with its FLAGS word and the MON descriptor (0x14 bytes)); F5a ported all of
//! them. The encounter rolls use the global 64-bit LCG of `black_core::rng` (`FUN_020056EC`, `next(max)`),
//! reached through [`FieldEnv::call`].

use crate::hooks::{FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_9350,
    0x021A_94B0,
    0x021A_9508,
    0x021A_954C,
    0x021A_957C,
    0x021A_9604,
    0x021A_96B8,
    0x021A_9780,
    0x021A_9810,
    0x021A_9828,
    0x021A_996C,
    0x021A_99A8,
    0x021A_99E4,
    0x021A_9A5C,
    0x021A_9B00,
    0x021A_9B68,
    0x021A_9BE8,
    0x021A_9C58,
    0x021A_9CAC,
    0x021A_9D30,
    0x021A_9DC8,
    0x021A_9DE0,
    0x021A_9E64,
    0x021A_9E98,
    0x021A_9EC4,
];

/// `FUN_020056EC(max)`: the global LCG's next value below `max` (`black_core::rng::MathRandContext32::next`).
const RAND: u32 = 0x0200_56EC;
/// `FUN_02005728(max)`: the global LCG's next value scaled below `max` (`MathRandContext32::next_scaled`).
const RAND_SCALED: u32 = 0x0200_5728;
/// `FUN_02010FB0(rate)`: the encounter-rate modifier of the pass powers.
const PASS_POWER_RATE: u32 = 0x0201_0FB0;
/// GAMESYS -> GAMEDATA.
const GAMEDATA: u32 = 0x0201_2EBC;
/// GAMESYS -> party (POKEPARTY).
const PARTY: u32 = 0x0201_2934;
/// (GAMEDATA) -> ENC SAVE (save block 0x38).
const ENC_SAVE: u32 = 0x0200_D418;
/// GAMESYS -> MYSTATUS.
const MYSTATUS: u32 = 0x0201_2944;
/// MYSTATUS -> trainer id.
const TRAINER_ID: u32 = 0x0200_8530;
/// POKEPARTY mon `i`.
const PARTY_MON: u32 = 0x0201_AB28;
/// POKEPARTY: count of the Pokemon able to battle.
const PARTY_ABLE: u32 = 0x0201_AA50;
/// POKEPARTY: index of the first Pokemon able to battle.
const PARTY_FIRST_ABLE: u32 = 0x0201_AB08;
/// POKEPARAM get `(pp, id, 0)`.
const PP_GET: u32 = 0x0201_7E1C;
/// POKEPARAM put `(pp, id, value)`.
const PP_PUT: u32 = 0x0201_7E40;
/// POKEPARAM nature.
const PP_NATURE: u32 = 0x0201_8598;
/// POKEPARAM init `(pp, species, level, tid, 0, iv word, pid, 0)`.
const PP_INIT: u32 = 0x0201_7588;
/// POKEPARAM recalculation.
const PP_RECALC: u32 = 0x0201_85C4;
/// POKEPARAM `(pp, species, form)`: the hidden ability / special set-up.
const PP_SPECIAL: u32 = 0x0201_8A00;
/// PID for `(tid, species, form, gender, 2, shiny mode)`.
const PID_FOR: u32 = 0x0201_86C8;
/// GAMESYS -> EVENT WORK.
const EVENT_WORK: u32 = 0x0201_296C;
/// `event_work::check_flag(ew, flag)`.
const CHECK_FLAG: u32 = 0x0201_42E8;
/// ENC SAVE: repel steps are 0.
const SV_NO_REPEL: u32 = 0x0200_D58C;
/// ENC SAVE `+0x30` (swarm index).
const SV_SWARM: u32 = 0x0200_D454;
/// ENC SAVE: roamer `i` active.
const SV_ROAMER_ACTIVE: u32 = 0x0200_D488;
/// ENC SAVE: roamer record `i`.
const SV_ROAMER: u32 = 0x0200_D4AC;
/// Roamer record field `(rec, id)`.
const ROAMER_FIELD: u32 = 0x0200_D4B8;
/// Personal field `(species, form, id)`: 0x20 form count, 6 / 7 types.
const PERSONAL_FIELD: u32 = 0x0201_9C90;
/// Personal data open `(species, form, heap)`.
const PERSONAL_OPEN: u32 = 0x0201_ADE4;
/// Personal data close.
const PERSONAL_CLOSE: u32 = 0x0201_AE14;
/// Personal field of an open handle `(handle, id)`: 0x11..0x13 items, 0x14 gender ratio.
const PERSONAL_GET: u32 = 0x0201_AE1C;
/// Gender ratio is fixed (genderless / one gender only).
const GENDER_FIXED: u32 = 0x0201_AF6C;
/// MYSTATUS getter copied to POKEPARAM 0x9A.
const MYSTATUS_9A: u32 = 0x0200_8550;
/// MYSTATUS getter copied to POKEPARAM 0x8E.
const MYSTATUS_8E: u32 = 0x0200_84F4;
/// field_wfbc: person `i` present `(wfbc, i)`.
const WFBC_PRESENT: u32 = 0x0219_390C;
/// field_wfbc: the person record (no arguments the spec models).
const WFBC_RECORD: u32 = 0x0219_3C00;

/// Event flag of the swarms.
pub const FLAG_SWARM: u32 = 0x960;

/// C: `_u32_div_f` (`FUN_0209C2B0`, runs as library code inside the originals) returning `(quotient,
/// remainder)`; division by zero gives `(n, 0)` (as `black_battle::calc::u32_divmod`).
pub fn u32_divmod(n: u32, d: u32) -> (u32, u32) {
    match (n.checked_div(d), n.checked_rem(d)) {
        (Some(q), Some(r)) => (q, r),
        _ => (n, 0),
    }
}

/// EFP `+0x28` flag: fishing type (5 / 6).
pub const F_FISHING: u32 = 2;
/// EFP flag: repel active.
pub const F_REPEL: u32 = 4;
/// EFP flag: kind 1 / 2 (no level filter).
pub const F_NO_FILTER: u32 = 8;
/// EFP flag: rate x2 (Illuminate, Arena Trap, No Guard).
pub const F_RATE_X2: u32 = 0x40;
/// EFP flag: rate / 2 (Stench, White Smoke, Quick Feet, Sand Veil / Snow Cloak in their weather).
pub const F_RATE_HALF: u32 = 0x80;
/// EFP flag: fishing rate x2 (Suction Cups, Sticky Hold).
pub const F_FISH_X2: u32 = 0x100;
/// EFP flag: Intimidate / Keen Eye (skip Pokemon 5 levels below the lead).
pub const F_LOW_LEVEL_SKIP: u32 = 0x8000;

/// Cleanse Tag and Pure Incense (`2 * rate / 3`).
const RATE_ITEMS: [u16; 2] = [0xE0, 0x140];

/// C: `FUN_overlay_d_21__021a94b0` (unlabeled routine at `0x021A94B0`): the encounter rate after the lead's
/// modifiers. Fishing (flag 2): Suction Cups / Sticky Hold double it, at most 100, nothing else. Otherwise flag
/// 0x40 doubles, else flag 0x80 halves it; a lead holding Cleanse Tag (0xE0) or Pure Incense (0x140) gives
/// `2 * rate / 3` of the unmodified rate instead; at most 100; then `FUN_02010FB0` (pass powers).
/// Shifts wrap at 32 bits and the division is unsigned, as in the original.
pub fn rate_modifier<E: FieldEnv>(env: &mut E, efp: u32, rate: u32) -> u32 {
    let flags = env.r32(efp.wrapping_add(0x28));
    if flags & F_FISHING != 0 {
        let r = if flags & F_FISH_X2 != 0 {
            rate.wrapping_shl(1)
        } else {
            rate
        };
        return r.min(100);
    }
    let mut r = if flags & F_RATE_X2 != 0 {
        rate.wrapping_shl(1)
    } else if flags & F_RATE_HALF != 0 {
        rate >> 1
    } else {
        rate
    };
    if RATE_ITEMS.contains(&env.r16(efp.wrapping_add(0x18))) {
        r = rate.wrapping_shl(1) / 3;
    }
    env.callv(PASS_POWER_RATE, &[r.min(100)])
}

/// C: `FUN_overlay_d_21__021a996c` (unlabeled routine at `0x021A996C`): is a wild Pokemon of `level` filtered
/// out? Never with flag 8; Intimidate / Keen Eye (flag 0x8000): when `level + 5 <= lead level` (signed); repel
/// (flag 4, not fishing): when `level < lead level` (unsigned). The lead level is EFP `+0x1E`.
pub fn rejected(mem: &Mem, efp: u32, level: u32) -> bool {
    let flags = mem.r32(efp.wrapping_add(0x28));
    if flags & F_NO_FILTER != 0 {
        return false;
    }
    let lead = mem.r8(efp.wrapping_add(0x1E)) as u32;
    if flags & F_LOW_LEVEL_SKIP != 0 && (lead as i32) >= (level.wrapping_add(5) as i32) {
        return true;
    }
    flags & F_FISHING == 0 && flags & F_REPEL != 0 && level < lead
}

/// C: `FUN_overlay_d_21__021a9dc8` (unlabeled routine at `0x021A9DC8`): the percent roll of the encounter code,
/// `FUN_020056EC(0xFFFF) / 0x290` (0..99).
pub fn roll100<E: FieldEnv>(env: &mut E) -> u32 {
    env.callv(RAND, &[0xFFFF]) / 0x290
}

/// The first index whose threshold is above `r`, else `thresholds.len()`.
fn slot_by(r: u32, thresholds: &[u32]) -> u32 {
    thresholds
        .iter()
        .position(|&t| r < t)
        .unwrap_or(thresholds.len()) as u32
}

/// C: `sub_021a9de0` (unlabeled routine at `0x021A9DE0`, the land entry of the slot-pick table ROM `0x021D8DD8`):
/// the land slot of a [`roll100`]: 20 / 20 / 10 / 10 / 10 / 10 / 5 / 5 / 4 / 4 / 1 / 1 percent (`r == 98` is
/// slot 10, anything else from 98 up slot 11). The slot count argument is not read.
pub fn land_slot<E: FieldEnv>(env: &mut E) -> u32 {
    let r = roll100(env);
    if r < 98 {
        slot_by(r, &[20, 40, 50, 60, 70, 80, 85, 90, 94, 98])
    } else if r == 98 {
        10
    } else {
        11
    }
}

/// C: unlabeled routine at `0x021A9E64` (the water entry of ROM `0x021D8DD8`): 60 / 30 / 5 / 4 / 1 percent.
pub fn water_slot<E: FieldEnv>(env: &mut E) -> u32 {
    slot_by(roll100(env), &[60, 90, 95, 99])
}

/// C: unlabeled routine at `0x021A9E98` (the fishing entry of ROM `0x021D8DD8`): 40 / 40 / 15 / 4 / 1 percent.
pub fn fishing_slot<E: FieldEnv>(env: &mut E) -> u32 {
    slot_by(roll100(env), &[40, 80, 95, 99])
}

/// EFP flag: event flag 0x960 (swarms on).
pub const F_SWARM: u32 = 1;
/// EFP flag: a dark-grass double battle.
pub const F_DOUBLE: u32 = 0x20;
/// EFP flag: held-item odds of Compound Eyes.
pub const F_ITEM_ODDS: u32 = 0x200;
/// EFP flag: Synchronize (the lead's nature).
pub const F_SYNCHRONIZE: u32 = 0x400;
/// EFP flag: Cute Charm (the opposite gender of the lead).
pub const F_CUTE_CHARM: u32 = 0x800;
/// EFP flag: Magnet Pull (prefer a steel slot).
pub const F_PREFER_STEEL: u32 = 0x1000;
/// EFP flag: Static (prefer an electric slot).
pub const F_PREFER_ELECTRIC: u32 = 0x2000;
/// EFP flag: Pressure / Hustle / Vital Spirit (the highest level of the species).
pub const F_MAX_LEVEL: u32 = 0x4000;

fn set_flag<E: FieldEnv>(env: &mut E, efp: u32, f: u32) {
    let v = env.r32(efp.wrapping_add(0x28));
    env.w32(efp.wrapping_add(0x28), v | f);
}

/// C: `FUN_overlay_d_21__021a9350` (unlabeled routine at `0x021A9350`): build the encounter context (EFP, 0x2C bytes at
/// `efp`) of `type` / `kind` (the fifth argument `weather` is a stack word; its low half goes to [`abilities`]): zero it;
/// `+0` GAMESYS, `+4` ENC SAVE, `+8` type, `+0xC` kind; types 5 / 6 set flag 2; `+0x24` MYSTATUS, `+0x20` its trainer id;
/// the lead (party mon 0): `+0x1A` egg (POKEPARAM 0x4C), and when it is not an egg `+0x16` species (5), `+0x18` item (6),
/// `+0x1B` ability (0xA), `+0x1C` gender (0x6E), `+0x1D` nature and [`abilities`]; flag 1 = event flag 0x960; flag 4 when
/// `FUN_0200D58C(sv)` is 0; kind 2 sets flag 8 and becomes 0, kind 1 sets flag 8; flags 0x10 / 0x20 cleared; type 1 with at
/// least 2 Pokemon able to battle and a [`roll100`] below 40 sets flag 0x20; with flag 0x8000 or 4 `+0x1E` is the level
/// (0x9E) of the first Pokemon able to battle; `+0x15` = 1 + (flag 0x20).
pub fn efp_init<E: FieldEnv>(env: &mut E, efp: u32, gsys: u32, ty: u32, kind: u32, weather: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let party = env.callv(PARTY, &[gsys]);
    env.mem().fill(efp, 0x2C, 0);
    env.w32(efp.wrapping_add(8), ty);
    env.w32(efp.wrapping_add(0xC), kind);
    env.w32(efp, gsys);
    let sv = env.callv(ENC_SAVE, &[gd]);
    env.w32(efp.wrapping_add(4), sv);
    if ty.wrapping_sub(5) <= 1 {
        set_flag(env, efp, F_FISHING);
    }
    let my = env.callv(MYSTATUS, &[gsys]);
    env.w32(efp.wrapping_add(0x24), my);
    let tid = env.callv(TRAINER_ID, &[my]);
    env.w32(efp.wrapping_add(0x20), tid);
    let lead = env.callv(PARTY_MON, &[party, 0]);
    let egg = env.callv(PP_GET, &[lead, 0x4C, 0]);
    env.w8(efp.wrapping_add(0x1A), egg as u8);
    if env.r8(efp.wrapping_add(0x1A)) == 0 {
        let v = env.callv(PP_GET, &[lead, 5, 0]);
        env.w16(efp.wrapping_add(0x16), v as u16);
        let v = env.callv(PP_GET, &[lead, 6, 0]);
        env.w16(efp.wrapping_add(0x18), v as u16);
        let v = env.callv(PP_GET, &[lead, 0xA, 0]);
        env.w8(efp.wrapping_add(0x1B), v as u8);
        let v = env.callv(PP_GET, &[lead, 0x6E, 0]);
        env.w8(efp.wrapping_add(0x1C), v as u8);
        let v = env.callv(PP_NATURE, &[lead]);
        env.w8(efp.wrapping_add(0x1D), v as u8);
        abilities(env, efp, weather & 0xFFFF);
    }
    let ew = env.callv(EVENT_WORK, &[gsys]);
    let f = env.callv(CHECK_FLAG, &[ew, FLAG_SWARM]);
    let flags = env.r32(efp.wrapping_add(0x28));
    env.w32(efp.wrapping_add(0x28), (flags & !1) | (f & 1));
    let sv = env.callv(ENC_SAVE, &[gd]);
    if env.callv(SV_NO_REPEL, &[sv]) == 0 {
        set_flag(env, efp, F_REPEL);
    }
    if env.r32(efp.wrapping_add(0xC)) == 2 {
        set_flag(env, efp, F_NO_FILTER);
        env.w32(efp.wrapping_add(0xC), 0);
    }
    if env.r32(efp.wrapping_add(0xC)) == 1 {
        set_flag(env, efp, F_NO_FILTER);
    }
    let flags = env.r32(efp.wrapping_add(0x28)) & !0x10 & !0x20;
    env.w32(efp.wrapping_add(0x28), flags);
    if env.r32(efp.wrapping_add(8)) == 1
        && env.callv(PARTY_ABLE, &[party]) as i32 >= 2
        && roll100(env) < 40
    {
        set_flag(env, efp, F_DOUBLE);
    }
    let flags = env.r32(efp.wrapping_add(0x28));
    if flags & F_LOW_LEVEL_SKIP != 0 || flags & F_REPEL != 0 {
        let i = env.callv(PARTY_FIRST_ABLE, &[party]);
        let m = env.callv(PARTY_MON, &[party, i]);
        let lv = env.callv(PP_GET, &[m, 0x9E, 0]);
        env.w8(efp.wrapping_add(0x1E), lv as u8);
    }
    let flags = env.r32(efp.wrapping_add(0x28));
    env.w8(efp.wrapping_add(0x15), (((flags >> 5) & 1) + 1) as u8);
}

/// C: `FUN_overlay_d_21__021a9508` (unlabeled routine at `0x021A9508`): pick the Pokemon of the encounter: for each of
/// the EFP `+0x15` Pokemon (re-read every step) [`slot_pick`] into `mons + 0x14 * accepted` (a rejected Pokemon is
/// overwritten by the next one), counting those the level filter ([`rejected`]) accepts; returns the count.
pub fn pick_mons<E: FieldEnv>(env: &mut E, efp: u32, slots: u32, mons: u32) -> u32 {
    let mut accepted = 0u32;
    let mut i = 0u32;
    while (i as i32) < env.r8(efp.wrapping_add(0x15)) as i32 {
        let m = mons.wrapping_add(0x14u32.wrapping_mul(accepted));
        slot_pick(env, efp, slots, m);
        let lv = env.r8(m.wrapping_add(4)) as u32;
        if !rejected(env.mem(), efp, lv) {
            accepted += 1;
        }
        i += 1;
    }
    accepted
}

/// C: `FUN_overlay_d_21__021a954c` (unlabeled routine at `0x021A954C`): the wild Pokemon of the ENCOUNT TABLE `table`:
/// the slot group ([`slot_group`]) into a zeroed 0x30-byte local (`MI_CpuFill32`), then [`pick_mons`]; returns the
/// accepted count.
pub fn wild_mons<E: FieldEnv>(env: &mut E, table: u32, efp: u32, zone: u32, mons: u32) -> u32 {
    let slots = env.mem().push_local(0x30);
    env.mem().fill(slots, 0x30, 0);
    slot_group(env, table, efp, zone, slots);
    let n = pick_mons(env, efp, slots, mons);
    env.mem().pop_local(slots);
    n
}

/// C: `FUN_overlay_d_21__021a957c` (unlabeled routine at `0x021A957C`): the roamer met in `zone`, or 0: the active roamer
/// records `i` = 0, 1 (ENC SAVE at EFP `+4`) whose field 1 is `zone` and whose level (field 6, low byte) passes the
/// level filter ([`rejected`]) are candidates; with any, `FUN_02005728(1000) >= 500` picks candidate
/// `FUN_02005728(count)`, else 0. `table` is not read.
pub fn roamer<E: FieldEnv>(env: &mut E, _table: u32, efp: u32, zone: u32) -> u32 {
    let mut cand = [0u32; 2];
    let mut n = 0u32;
    for i in 0..2 {
        let sv = env.r32(efp.wrapping_add(4));
        if env.callv(SV_ROAMER_ACTIVE, &[sv, i]) == 0 {
            continue;
        }
        let sv = env.r32(efp.wrapping_add(4));
        let rec = env.callv(SV_ROAMER, &[sv, i]);
        if zone != env.callv(ROAMER_FIELD, &[rec, 1]) {
            continue;
        }
        let lv = env.callv(ROAMER_FIELD, &[rec, 6]) & 0xFF;
        if !rejected(env.mem(), efp, lv) {
            cand[n as usize] = rec;
            n += 1;
        }
    }
    if n == 0 || env.callv(RAND_SCALED, &[1000]) < 500 {
        return 0;
    }
    let k = env.callv(RAND_SCALED, &[n]);
    cand[k as usize]
}

/// C: `FUN_overlay_d_21__021a9604` (unlabeled routine at `0x021A9604`): the WFBC encounter of the people of `wfbc` into
/// `mon`: the weight column `c` by the EFP type: 0 -> 0, 1 -> 1, 3 -> 2, 0xFF -> none (0); for any other type the
/// original keeps its caller's r5, passed here as `r5` (0 under the harness entry state). `total` = the sum of the bytes
/// `rec + 0x14 + c` of the present people (`FUN_0219390C(wfbc, i)`, `i < 30`, `rec = FUN_02193C00()`); `r =
/// FUN_020056EC(total)`; the first present person whose running sum exceeds `r` fills `mon` (zeroed: `+0` species = u16
/// `rec + 8 + 2c`, `+4` level = `rec + 0x10 + c`) and gives 1; else 0.
pub fn wfbc_mon<E: FieldEnv>(env: &mut E, wfbc: u32, efp: u32, mon: u32, r5: u32) -> u32 {
    let ty = env.r32(efp.wrapping_add(8)) as i32;
    let c = match ty {
        0 => 0,
        1 => 1,
        3 => 2,
        0xFF => return 0,
        _ => r5,
    };
    let mut total = 0u32;
    for i in 0..30 {
        if env.callv(WFBC_PRESENT, &[wfbc, i]) != 0 {
            let rec = env.callv(WFBC_RECORD, &[]);
            total = total.wrapping_add(env.r8(c.wrapping_add(rec).wrapping_add(0x14)) as u32);
        }
    }
    let r = env.callv(RAND, &[total]);
    let mut sum = 0u32;
    for i in 0..30 {
        if env.callv(WFBC_PRESENT, &[wfbc, i]) == 0 {
            continue;
        }
        let rec = env.callv(WFBC_RECORD, &[]);
        sum = sum.wrapping_add(env.r8(c.wrapping_add(rec).wrapping_add(0x14)) as u32);
        if r < sum {
            env.mem().fill(mon, 0x14, 0);
            let sp = env.r16(rec.wrapping_add(c.wrapping_mul(2)).wrapping_add(8));
            env.w16(mon, sp);
            let lv = env.r8(rec.wrapping_add(c).wrapping_add(0x10));
            env.w8(mon.wrapping_add(4), lv);
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_21__021a96b8` (unlabeled routine at `0x021A96B8`): set up the POKEPARAM `pp` of the wild Pokemon
/// `mon`: open its personal data (`FUN_0201ADE4(species, form, 4)`), PID by [`pid`], `FUN_02017588(pp, species, level, tid,
/// 0, -1, pid, 0)`, form (0x6F), the special set-up `FUN_02018A00` when `mon +7` is set, the held item (`mon +2`: 1..0x272
/// that item, 0 the personal items by [`held_item`] (Compound Eyes flag 0x200, dark grass type 1), above 0x272 none),
/// 0x9A / 0x8E from the MYSTATUS, the lead's nature (0x70) with Synchronize (flag 0x400), recalculation, personal close.
pub fn wild_pokeparam<E: FieldEnv>(env: &mut E, pp: u32, efp: u32, mon: u32) {
    let species = env.r16(mon) as u32;
    let form = env.r8(mon.wrapping_add(5)) as u32;
    let per = env.callv(PERSONAL_OPEN, &[species, form, 4]);
    let pid = pid(env, efp, per, mon);
    let species = env.r16(mon) as u32;
    let level = env.r8(mon.wrapping_add(4)) as u32;
    let tid = env.r32(efp.wrapping_add(0x20));
    env.callv(PP_INIT, &[pp, species, level, tid, 0, u32::MAX, pid, 0]);
    let form = env.r8(mon.wrapping_add(5)) as u32;
    env.callv(PP_PUT, &[pp, 0x6F, form]);
    if env.r8(mon.wrapping_add(7)) != 0 {
        let species = env.r16(mon) as u32;
        let form = env.r8(mon.wrapping_add(5)) as u32;
        env.callv(PP_SPECIAL, &[pp, species, form]);
    }
    let item = env.r16(mon.wrapping_add(2)) as u32;
    if item <= 0x272 {
        if item != 0 {
            env.callv(PP_PUT, &[pp, 6, item]);
        } else {
            let dark = (env.r32(efp.wrapping_add(8)) == 1) as u32;
            let eyes = (env.r32(efp.wrapping_add(0x28)) >> 9) & 1;
            held_item(env, pp, per, eyes, dark);
        }
    }
    let my = env.r32(efp.wrapping_add(0x24));
    let v = env.callv(MYSTATUS_9A, &[my]);
    env.callv(PP_PUT, &[pp, 0x9A, v]);
    let my = env.r32(efp.wrapping_add(0x24));
    let v = env.callv(MYSTATUS_8E, &[my]);
    env.callv(PP_PUT, &[pp, 0x8E, v]);
    if env.r32(efp.wrapping_add(0x28)) & F_SYNCHRONIZE != 0 {
        let nature = env.r8(efp.wrapping_add(0x1D)) as u32;
        env.callv(PP_PUT, &[pp, 0x70, nature]);
    }
    env.callv(PP_RECALC, &[pp]);
    env.callv(PERSONAL_CLOSE, &[per]);
}

/// C: `FUN_overlay_d_21__021a9780` (unlabeled routine at `0x021A9780`): set up the POKEPARAM `pp` of the roamer record
/// `rec`: `FUN_02017588(pp, field 4 (u16), field 6 (u8), tid, 0, field 2, field 3, 0)`, nature (0x70) = field 10, 0x9D =
/// field 7, 0xA0 = field 5 (`FUN_0200D4B8(rec, id)`), recalculation.
pub fn roamer_pokeparam<E: FieldEnv>(env: &mut E, pp: u32, efp: u32, rec: u32) {
    let species = env.callv(ROAMER_FIELD, &[rec, 4]) & 0xFFFF;
    let level = env.callv(ROAMER_FIELD, &[rec, 6]) & 0xFF;
    let a = env.callv(ROAMER_FIELD, &[rec, 2]);
    let b = env.callv(ROAMER_FIELD, &[rec, 3]);
    let tid = env.r32(efp.wrapping_add(0x20));
    env.callv(PP_INIT, &[pp, species, level, tid, 0, a, b, 0]);
    let v = env.callv(ROAMER_FIELD, &[rec, 10]);
    env.callv(PP_PUT, &[pp, 0x70, v]);
    let v = env.callv(ROAMER_FIELD, &[rec, 7]);
    env.callv(PP_PUT, &[pp, 0x9D, v]);
    let v = env.callv(ROAMER_FIELD, &[rec, 5]);
    env.callv(PP_PUT, &[pp, 0xA0, v]);
    env.callv(PP_RECALC, &[pp]);
}

/// C: unlabeled routine at `0x021A9810` (no caller in the field set): the swarm species: [`swarm_record`]`(gsys)`; none:
/// 0xFFFF, else its first halfword (the record's zone, as the original reads it).
pub fn swarm_species<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let s = swarm_record(env, gsys);
    if s == 0 {
        0xFFFF
    } else {
        rom_r16(s) as u32
    }
}

/// C: `FUN_overlay_d_21__021a9828` (unlabeled routine at `0x021A9828`): the lead's ability (EFP `+0x1B`) -> EFP flags:
/// Illuminate 0x23 / Arena Trap 0x47 / No Guard 0x63: 0x40; Stench 1 / White Smoke 0x49 / Quick Feet 0x5F: 0x80; Sand
/// Veil 8 in weather 3 / 0xC, Snow Cloak 0x51 in weather 1 or with a low half 4 / 5: 0x80; Suction Cups 0x15 / Sticky
/// Hold 0x3C with flag 2: 0x100; Compound Eyes 0xE: 0x200 (each returns at once, also when its condition fails). Any
/// other: Cute Charm 0x38 with a [`roll100`] below 67: 0x800; else (also Cute Charm with a higher roll) a second
/// [`roll100`] of at least 50 gives Magnet Pull 0x2A: 0x1000, Static 9: 0x2000, Pressure 0x2E / Hustle 0x37 / Vital
/// Spirit 0x48: 0x4000, Intimidate 0x16 / Keen Eye 0x33: 0x8000, Synchronize 0x1C: 0x400.
pub fn abilities<E: FieldEnv>(env: &mut E, efp: u32, weather: u32) {
    let a = env.r8(efp.wrapping_add(0x1B));
    match a {
        0x23 | 0x47 | 0x63 => return set_flag(env, efp, F_RATE_X2),
        1 | 0x49 | 0x5F => return set_flag(env, efp, F_RATE_HALF),
        8 => {
            if weather == 3 || weather == 0xC {
                set_flag(env, efp, F_RATE_HALF);
            }
            return;
        }
        0x51 => {
            if weather == 1 || weather.wrapping_add(0xFFFC) & 0xFFFF <= 1 {
                set_flag(env, efp, F_RATE_HALF);
            }
            return;
        }
        0x15 | 0x3C => {
            if env.r32(efp.wrapping_add(0x28)) & F_FISHING != 0 {
                set_flag(env, efp, F_FISH_X2);
            }
            return;
        }
        0xE => return set_flag(env, efp, F_ITEM_ODDS),
        _ => {}
    }
    if a == 0x38 && roll100(env) < 67 {
        return set_flag(env, efp, F_CUTE_CHARM);
    }
    if roll100(env) < 50 {
        return;
    }
    match env.r8(efp.wrapping_add(0x1B)) {
        0x2A => set_flag(env, efp, F_PREFER_STEEL),
        9 => set_flag(env, efp, F_PREFER_ELECTRIC),
        0x2E | 0x37 | 0x48 => set_flag(env, efp, F_MAX_LEVEL),
        0x16 | 0x33 => set_flag(env, efp, F_LOW_LEVEL_SKIP),
        0x1C => set_flag(env, efp, F_SYNCHRONIZE),
        _ => {}
    }
}

/// ROM `0x021D8DE8`: the 17 swarm records (u16 zone, u16 species, u8 form, u8 min level, u8 max level, u8 0).
pub const SWARM_ROM: u32 = 0x021D_8DE8;
/// The bytes of [`SWARM_ROM`].
pub const SWARM_DATA: [u8; 17 * 8] = [
    0x3d, 0x01, 0x53, 0x00, 0x00, 0x0f, 0x37, 0x00, 0x3f, 0x01, 0x68, 0x01, 0x00, 0x0f, 0x37, 0x00,
    0x41, 0x01, 0x39, 0x01, 0x00, 0x0f, 0x37, 0x00, 0x46, 0x01, 0xc1, 0x01, 0x00, 0x0f, 0x37, 0x00,
    0x49, 0x01, 0xeb, 0x00, 0x00, 0x0f, 0x37, 0x00, 0x4b, 0x01, 0x37, 0x01, 0x00, 0x0f, 0x37, 0x00,
    0x51, 0x01, 0xa1, 0x00, 0x00, 0x0f, 0x37, 0x00, 0x59, 0x01, 0xc5, 0x01, 0x00, 0x0f, 0x37, 0x00,
    0x5c, 0x01, 0xe4, 0x00, 0x00, 0x0f, 0x37, 0x00, 0x63, 0x01, 0xec, 0x00, 0x00, 0x0f, 0x37, 0x00,
    0x6d, 0x01, 0x1d, 0x01, 0x00, 0x0f, 0x37, 0x00, 0x70, 0x01, 0x54, 0x00, 0x00, 0x0f, 0x37, 0x00,
    0x72, 0x01, 0x61, 0x01, 0x00, 0x0f, 0x37, 0x00, 0x76, 0x01, 0xc1, 0x00, 0x00, 0x0f, 0x37, 0x00,
    0x7a, 0x01, 0x38, 0x00, 0x00, 0x0f, 0x37, 0x00, 0x7f, 0x01, 0xcc, 0x00, 0x00, 0x0f, 0x37, 0x00,
    0x83, 0x01, 0x66, 0x00, 0x00, 0x0f, 0x37, 0x00,
];

/// A byte of the swarm table by its ROM address (panics outside [`SWARM_DATA`]).
fn rom_r8(addr: u32) -> u8 {
    SWARM_DATA[addr.wrapping_sub(SWARM_ROM) as usize]
}

fn rom_r16(addr: u32) -> u16 {
    rom_r8(addr) as u16 | (rom_r8(addr.wrapping_add(1)) as u16) << 8
}

/// C: `FUN_overlay_d_21__021a99a8` (unlabeled routine at `0x021A99A8`): the ROM address of the current swarm record:
/// 0 without event flag 0x960; else index `k = FUN_0200D454(sv)` (`k >= 17` -> 0): `0x021D8DE8 + 8k`.
pub fn swarm_record<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let ew = env.callv(EVENT_WORK, &[gsys]);
    if env.callv(CHECK_FLAG, &[ew, FLAG_SWARM]) == 0 {
        return 0;
    }
    let sv = env.callv(ENC_SAVE, &[gd]);
    let mut k = env.callv(SV_SWARM, &[sv]);
    if k >= 17 {
        k = 0;
    }
    SWARM_ROM.wrapping_add(k << 3)
}

/// C: `FUN_overlay_d_21__021a99e4` (unlabeled routine at `0x021A99E4`): the swarm slot: the record of [`swarm_record`]`(EFP
/// +0)` must exist, be of `zone` and the type be 0 (grass), and `FUN_02005728(1000) <= 400`; then `slot` = (species | form
/// << 11, min, max), EFP `+0x10` = 3 (uniform), `+0x14` = 1; returns 1 (else 0). `table` is not read.
pub fn swarm_slot<E: FieldEnv>(env: &mut E, _table: u32, efp: u32, zone: u32, slot: u32) -> u32 {
    let gsys = env.r32(efp);
    let s = swarm_record(env, gsys);
    if s == 0 || rom_r16(s) as u32 != zone {
        return 0;
    }
    if env.r32(efp.wrapping_add(8)) != 0 {
        return 0;
    }
    if env.callv(RAND_SCALED, &[1000]) > 400 {
        return 0;
    }
    let old = env.r16(slot);
    env.w16(slot, (old & 0xF800) | (rom_r16(s.wrapping_add(2)) & 0x7FF));
    env.w8(slot.wrapping_add(2), rom_r8(s.wrapping_add(5)));
    env.w8(slot.wrapping_add(3), rom_r8(s.wrapping_add(6)));
    let cur = env.r16(slot);
    let form = (rom_r8(s.wrapping_add(4)) as u16 & 0x1F) << 11;
    env.w16(slot, (cur & 0x07FF) | form);
    env.w32(efp.wrapping_add(0x10), 3);
    env.w8(efp.wrapping_add(0x14), 1);
    1
}

/// C: `FUN_overlay_d_21__021a9a5c` (unlabeled routine at `0x021A9A5C`): the slot group of the encounter into `slots`;
/// returns the slot count: the swarm ([`swarm_slot`]) gives EFP `+0x14`; a type of 7 or more (signed) becomes 0; types 0 /
/// 1 / 2: 12 slots at `table` +8 / +0x38 / +0x68, mode 0; 3 / 4: 5 at +0x98 / +0xAC, mode 1; 5 / 6: 5 at +0xC0 / +0xD4,
/// mode 2; EFP `+0x10` = mode, `+0x14` = count; the 4-byte slots are copied (`MI_CpuCopy8`). A negative type copies 0
/// bytes (from the caller's r4) with mode and count 0.
pub fn slot_group<E: FieldEnv>(env: &mut E, table: u32, efp: u32, zone: u32, slots: u32) -> u32 {
    if swarm_slot(env, table, efp, zone, slots) != 0 {
        return env.r8(efp.wrapping_add(0x14)) as u32;
    }
    if env.r32(efp.wrapping_add(8)) as i32 >= 7 {
        env.w32(efp.wrapping_add(8), 0);
    }
    let (ofs, n, mode) = match env.r32(efp.wrapping_add(8)) {
        0 => (8, 12, 0),
        1 => (0x38, 12, 0),
        2 => (0x68, 12, 0),
        3 => (0x98, 5, 1),
        4 => (0xAC, 5, 1),
        5 => (0xC0, 5, 2),
        6 => (0xD4, 5, 2),
        _ => (0, 0, 0),
    };
    env.w32(efp.wrapping_add(0x10), mode);
    env.w8(efp.wrapping_add(0x14), n as u8);
    if n != 0 {
        env.mem().copy(slots, table.wrapping_add(ofs), n * 4);
    }
    n
}

/// C: `FUN_overlay_d_21__021a9b00` (unlabeled routine at `0x021A9B00`): roll the Pokemon of `slot` into `mon` (zeroed, 0x14
/// bytes): level = min + [`roll100`] % (max(max - min, 0) + 1) (16-bit signed difference), species = `slot & 0x7FF`;
/// `n = FUN_02019C90(species, 0, 0x20)` (form count, low byte); form `slot >> 11`: 0x1F -> `FUN_02005728(n)`, `>= n`
/// (signed) -> 0, else the form.
pub fn roll_mon<E: FieldEnv>(env: &mut E, slot: u32, mon: u32) {
    let min = env.r8(slot.wrapping_add(2)) as u32;
    let max = env.r8(slot.wrapping_add(3)) as u32;
    let mut d = max.wrapping_sub(min) as u16 as i16 as i32;
    env.mem().fill(mon, 0x14, 0);
    if d < 0 {
        d = 0;
    }
    let r = roll100(env);
    let (_, rem) = u32_divmod(r, (d + 1) as u32);
    let min = env.r8(slot.wrapping_add(2)) as u32;
    env.w8(mon.wrapping_add(4), min.wrapping_add(rem) as u8);
    let species = env.r16(slot) as u32 & 0x7FF;
    env.w16(mon, species as u16);
    let n = env.callv(PERSONAL_FIELD, &[species, 0, 0x20]) & 0xFF;
    let form = (env.r16(slot) >> 11) as u32 & 0x1F;
    if form == 0x1F {
        let f = env.callv(RAND_SCALED, &[n]);
        env.w8(mon.wrapping_add(5), f as u8);
    } else if (form as i32) < n as i32 {
        env.w8(mon.wrapping_add(5), form as u8);
    } else {
        env.w8(mon.wrapping_add(5), 0);
    }
}

/// C: `FUN_overlay_d_21__021a9b68` (unlabeled routine at `0x021A9B68`): a slot whose species has the type `ty`: the
/// slots `i` < EFP `+0x14` (not form 0x1F) whose personal type 1 or 2 (`FUN_02019C90(species, form, 6 / 7)`, low byte) is
/// `ty`; none: 0xFF, else candidate [`roll100`] % count. UNVERIFIED: more than 12 candidates overflow the original's
/// 12-byte local (not modelled).
pub fn typed_slot<E: FieldEnv>(env: &mut E, efp: u32, slots: u32, ty: u32) -> u32 {
    let mut cand: Vec<u8> = Vec::new();
    let mut i = 0u32;
    while (i as i32) < env.r8(efp.wrapping_add(0x14)) as i32 {
        let s = env.r16(slots.wrapping_add(i * 4)) as u32;
        let form = (s >> 11) & 0x1F;
        if form != 0x1F {
            let t1 = env.callv(PERSONAL_FIELD, &[s & 0x7FF, form, 6]) & 0xFF;
            let s = env.r16(slots.wrapping_add(i * 4)) as u32;
            let t2 = env.callv(PERSONAL_FIELD, &[s & 0x7FF, (s >> 11) & 0x1F, 7]) & 0xFF;
            if t1 == ty || t2 == ty {
                cand.push(i as u8);
            }
        }
        i += 1;
    }
    if cand.is_empty() {
        return 0xFF;
    }
    let r = roll100(env);
    let (_, k) = u32_divmod(r, cand.len() as u32);
    cand[k as usize] as u32
}

/// C: `FUN_overlay_d_21__021a9be8` (unlabeled routine at `0x021A9BE8`): Pressure / Hustle / Vital Spirit (flag 0x4000):
/// `m` = the highest of `mon`'s level and the max levels of the slots `i` < EFP `+0x14` of `mon`'s species; when some slot's
/// min and max differ and `level + 5 <= m`: level + 5; else level = `m`.
pub fn max_level<E: FieldEnv>(env: &mut E, efp: u32, slots: u32, mon: u32) {
    let lv = env.r8(mon.wrapping_add(4)) as u32;
    if env.r32(efp.wrapping_add(0x28)) & F_MAX_LEVEL == 0 {
        return;
    }
    let mut m = lv;
    let mut varied = false;
    let species = env.r16(mon) as u32;
    let mut i = 0u32;
    while (i as i32) < env.r8(efp.wrapping_add(0x14)) as i32 {
        let s = slots.wrapping_add(i * 4);
        if !varied && env.r8(s.wrapping_add(2)) != env.r8(s.wrapping_add(3)) {
            varied = true;
        }
        if species == env.r16(s) as u32 & 0x7FF {
            let mx = env.r8(s.wrapping_add(3)) as u32;
            if m < mx {
                m = mx;
            }
        }
        i += 1;
    }
    if (lv + 5) as i32 > m as i32 || !varied {
        env.w8(mon.wrapping_add(4), m as u8);
    } else {
        let l = env.r8(mon.wrapping_add(4));
        env.w8(mon.wrapping_add(4), l.wrapping_add(5));
    }
}

/// ROM `0x021D8DD8`: the slot-pick functions by EFP `+0x10` mode (Thumb addresses): [`land_slot`], [`water_slot`],
/// [`fishing_slot`], [`uniform_slot`].
pub const SLOT_PICK: [u32; 4] = [0x021A_9DE1, 0x021A_9E65, 0x021A_9E99, 0x021A_9EC5];

/// C: `FUN_overlay_d_21__021a9c58` (unlabeled routine at `0x021A9C58`): pick the slot of `mon` and roll it: Magnet Pull
/// (flag 0x1000) a steel (8) slot, else Static (flag 0x2000) an electric (0xC) slot ([`typed_slot`]); none (0xFF): the
/// function of [`SLOT_PICK`]`[mode]` with the slot count (low byte of its result); then [`roll_mon`]`(slots + 4i, mon)` and
/// [`max_level`]. A mode of 4 or more makes the original jump through the next ROM words (the swarm table); the port
/// panics there (UNVERIFIED).
pub fn slot_pick<E: FieldEnv>(env: &mut E, efp: u32, slots: u32, mon: u32) {
    let flags = env.r32(efp.wrapping_add(0x28));
    let mut i = 0xFF;
    if flags & F_PREFER_STEEL != 0 {
        i = typed_slot(env, efp, slots, 8);
    } else if flags & F_PREFER_ELECTRIC != 0 {
        i = typed_slot(env, efp, slots, 0xC);
    }
    if i == 0xFF {
        let mode = env.r32(efp.wrapping_add(0x10));
        let n = env.r8(efp.wrapping_add(0x14)) as u32;
        let r = match SLOT_PICK.get(mode as usize).copied() {
            Some(0x021A_9DE1) => land_slot(env),
            Some(0x021A_9E65) => water_slot(env),
            Some(0x021A_9E99) => fishing_slot(env),
            Some(0x021A_9EC5) => uniform_slot(env, n),
            _ => panic!("slot-pick mode {mode} reads past ROM 0x021D8DD8"),
        };
        i = r & 0xFF;
    }
    roll_mon(env, slots.wrapping_add(i << 2), mon);
    max_level(env, efp, slots, mon);
}

/// ROM `0x021D8DCC`: the held-item odds (percent of the three personal items) by `eyes + 2 * dark` (Compound Eyes,
/// encounter type 1).
pub const ITEM_ODDS: [[u8; 3]; 4] = [[50, 5, 0], [60, 20, 0], [50, 5, 1], [60, 20, 5]];

/// C: `FUN_overlay_d_21__021a9cac` (unlabeled routine at `0x021A9CAC`): the held item of a wild Pokemon from its personal
/// items (`FUN_0201AE1C(per, 0x11 / 0x12 / 0x13)`, u16): the first two equal: that one (POKEPARAM put 6); else the
/// first item whose running sum of [`ITEM_ODDS`]`[eyes + 2 dark]` (bytes, wrapping) exceeds the low byte of a
/// [`roll100`], or none.
pub fn held_item<E: FieldEnv>(env: &mut E, pp: u32, per: u32, eyes: u32, dark: u32) {
    let mut items = [0u16; 3];
    for (i, it) in items.iter_mut().enumerate() {
        *it = env.callv(PERSONAL_GET, &[per, 0x11 + i as u32]) as u16;
    }
    if items[0] == items[1] {
        env.callv(PP_PUT, &[pp, 6, items[0] as u32]);
        return;
    }
    let odds = ITEM_ODDS[eyes.wrapping_add(dark.wrapping_mul(2)) as usize];
    let r = roll100(env) & 0xFF;
    let mut sum = 0u8;
    for i in 0..3 {
        sum = sum.wrapping_add(odds[i]);
        if r < sum as u32 {
            env.callv(PP_PUT, &[pp, 6, items[i] as u32]);
            return;
        }
    }
}

/// C: `FUN_overlay_d_21__021a9d30` (unlabeled routine at `0x021A9D30`): the PID of the wild Pokemon `mon`: gender request
/// 2 (random); when the gender ratio (`FUN_0201AE1C(per, 0x14)`, low byte) is not fixed (`FUN_0201AF6C != 1`): Cute Charm
/// (flag 0x800) asks for the lead's opposite gender (0 -> 1, 1 -> 0), then `mon +0x10` 1 -> 0, 2 -> 1; `pid =
/// FUN_020186C8(tid, species, form, gender, 2, mon +6 == 2 ? 0 : 2)`; unless `mon +6` is 2, bit 31 is set to the parity
/// bit `(pid ^ tid ^ tid >> 16) & 1` of the low halves.
pub fn pid<E: FieldEnv>(env: &mut E, efp: u32, per: u32, mon: u32) -> u32 {
    let mut g = 2;
    let ratio = env.callv(PERSONAL_GET, &[per, 0x14]) & 0xFF;
    if env.callv(GENDER_FIXED, &[ratio]) != 1 {
        if env.r32(efp.wrapping_add(0x28)) & F_CUTE_CHARM != 0 {
            match env.r8(efp.wrapping_add(0x1C)) {
                0 => g = 1,
                1 => g = 0,
                _ => {}
            }
        }
        match env.r8(mon.wrapping_add(0x10)) {
            1 => g = 0,
            2 => g = 1,
            _ => {}
        }
    }
    let shiny = if env.r8(mon.wrapping_add(6)) == 2 {
        0
    } else {
        2
    };
    let species = env.r16(mon) as u32;
    let form = env.r8(mon.wrapping_add(5)) as u32;
    let tid = env.r32(efp.wrapping_add(0x20));
    let pid = env.callv(PID_FOR, &[tid, species, form, g, 2, shiny]);
    if env.r8(mon.wrapping_add(6)) == 2 {
        return pid;
    }
    let tid = env.r32(efp.wrapping_add(0x20));
    let x = (pid & 0xFFFF) ^ ((tid & 0xFFFF) ^ (tid >> 16));
    if x & 1 != 0 {
        pid | 0x8000_0000
    } else {
        pid & 0x7FFF_FFFF
    }
}

/// C: `FUN_overlay_d_21__021a9ec4` (unlabeled routine at `0x021A9EC4`, the uniform entry of ROM `0x021D8DD8`, mode 3):
/// [`roll100`] % `n` (unsigned).
pub fn uniform_slot<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    let r = roll100(env);
    u32_divmod(r, n).1
}

#[cfg(test)]
mod tests;
