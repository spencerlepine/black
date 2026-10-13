//! `field_encount.c` (overlay 21, `0x021A9ED8..0x021AA77C`, F1 row 21.29): the FIELD ENCOUNT object, the wild
//! encounter check of a step, the fixed (scripted) wild battle, the fishing check, the WFBC encounter, the battle
//! field situation, the trainer battle setup and the walk counter.
//!
//! F5a-spec specced all 19 functions (`tools/rom-diff/specs/black-field/encount.spec`, whose header documents the
//! encounter roll step by step, the FIELD ENCOUNT object (0x14), the ENCOUNT WORK (0x20) and the FIELD
//! SITUATION (0x10)); F5a ported all 19 (15 verified, see `PORTED`).
//!
//! Encounter types (the index of the rate byte and of the slot group in an ENCOUNT TABLE): 0 grass, 1 dark
//! grass (doubles), 2 shaking grass, 3 surf, 4 rippling water, 5 fishing, 6 fishing on a ripple.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported and verified (read by `tools/rom-diff/svflow.py --ported-src`). Ported but
/// not listed (UNVERIFIED, see the crate README): `021A9F40` [`step_check`], `021AA0A0` [`fixed_battle`], `021AA180`
/// [`fishing`], `021AA248` [`wfbc_step`] (they pass the address of their EFP local to `FUN_021A96B8` / `FUN_021A9780`,
/// whose spec stubs log it as a value).
pub const PORTED: &[u32] = &[
    0x021A_9ED8,
    0x021A_9F28,
    0x021A_A304,
    0x021A_A35C,
    0x021A_A3E0,
    0x021A_A3FC,
    0x021A_A42C,
    0x021A_A490,
    0x021A_A548,
    0x021A_A5CC,
    0x021A_A69C,
    0x021A_A6E0,
    0x021A_A6F8,
    0x021A_A728,
    0x021A_A778,
];

/// `FUN_020056EC(max)`: the global LCG's next value below `max`.
const RAND: u32 = 0x0200_56EC;
/// FIELDMAP -> weather system.
const FIELDMAP_WEATHER: u32 = 0x0218_8C6C;
/// Weather id of the weather system.
const WEATHER_ID: u32 = 0x021A_1BF4;
/// FIELDMAP heap id.
const FIELDMAP_HEAP: u32 = 0x0218_8C84;
/// FIELDMAP -> GAMESYS.
const FIELDMAP_GAMESYS: u32 = 0x0218_8C80;
/// FIELDMAP zone id.
const FIELDMAP_ZONE: u32 = 0x0218_8C9C;
/// FIELDMAP +0x8C (the field player).
const FIELDMAP_PLAYER: u32 = 0x0218_8CB4;
/// FIELDMAP -> map mapper (result unused by `021AA35C`).
const FIELDMAP_MAPPER: u32 = 0x0218_8C98;
/// GAMESYS -> GAMEDATA.
const GAMEDATA: u32 = 0x0201_20F4;
/// GAMEDATA -> EVENTDATA_SYSTEM.
const EVENTDATA: u32 = 0x0201_2844;
/// eventdata: EVENTDATA_SYSTEM -> ENCOUNT TABLE (`sys + 0x38`).
const EVENTDATA_ENCOUNT: u32 = 0x0216_255C;
/// GAMEDATA -> ENCOUNT WORK (0x20 bytes).
const ENCOUNT_WORK: u32 = 0x0201_2A38;
/// GAMEDATA byte getter (the season).
const SEASON: u32 = 0x0201_2984;
/// GAMEDATA -> record counters.
const RECORDS: u32 = 0x0201_2F2C;
/// Record counter add `(rec, id)`.
const RECORD_ADD: u32 = 0x0200_8DF0;
/// effect_encount: allocate the EFFECT ENCOUNT work `(heap)`.
const EFFECT_NEW: u32 = 0x021A_A77C;
/// effect_encount: set up `(enc, effect)`.
const EFFECT_INIT: u32 = 0x021A_A7AC;
/// effect_encount: tear down `(enc, effect)`.
const EFFECT_EXIT: u32 = 0x021A_A7CC;
/// effect_encount: free the EFFECT ENCOUNT work.
const EFFECT_FREE: u32 = 0x021A_A794;
/// zonedata: zone -> battle background.
const ZONE_BATTLE_BG: u32 = 0x0201_3ED8;
/// The field player -> attribute word under it.
const PLAYER_ATTR: u32 = 0x021A_30C8;
/// The field player -> its MMDL.
const PLAYER_MMDL: u32 = 0x021A_3074;
/// map_attr `021AB0F0` (attribute value).
const ATTR_VALUE: u32 = 0x021A_B0F0;
/// map_attr `021AB0F8` (attribute flags).
const ATTR_FLAGS: u32 = 0x021A_B0F8;
/// map_attr `021AB100` (valid attribute).
const ATTR_VALID: u32 = 0x021A_B100;
/// map_attr `021AB140` (value 8 / 9).
const ATTR_8_OR_9: u32 = 0x021A_B140;
/// map_attr `021AB29C` (long grass).
const ATTR_LONG_GRASS: u32 = 0x021A_B29C;
/// map_attr `021AB580` (battle ground kind).
const ATTR_GROUND: u32 = 0x021A_B580;
/// RTC time `(&out)` (hour, minute, second words).
const RTC_TIME: u32 = 0x0203_F4CC;
/// MMDL grid x / y / z.
const MMDL_GX: u32 = 0x0216_D83C;
const MMDL_GY: u32 = 0x0216_D854;
const MMDL_GZ: u32 = 0x0216_D86C;
/// enc_pokeset: build the EFP `(efp, gamedata, type, kind, weather)`.
const EFP_INIT: u32 = 0x021A_9350;
/// enc_pokeset: the rate after the lead's modifiers `(efp, gamedata, rate)`.
const RATE_MODIFIER: u32 = 0x021A_94B0;
/// enc_pokeset: the wild Pokemon of the table `(table, efp, zone, mons)`.
const WILD_MONS: u32 = 0x021A_954C;
/// enc_pokeset: the roamer met `(table, efp, zone)`.
const ROAMER: u32 = 0x021A_957C;
/// enc_pokeset: the WFBC encounter `(wfbc, efp, mon)`.
const WFBC_MON: u32 = 0x021A_9604;
/// enc_pokeset: POKEPARAM of a wild Pokemon `(pp, efp, mon)`.
const WILD_POKEPARAM: u32 = 0x021A_96B8;
/// enc_pokeset: POKEPARAM of a roamer `(pp, efp, rec)`.
const ROAMER_POKEPARAM: u32 = 0x021A_9780;
/// btl_setup alloc `(heap)`.
const BTL_SETUP_NEW: u32 = 0x0201_3090;
/// btl_setup or_flags `(setup, mask)`.
const BTL_SETUP_FLAGS: u32 = 0x0201_3144;
/// btl_setup init `(setup)`.
const BTL_SETUP_INIT: u32 = 0x0201_30C4;
/// btl_setup setup_with_party `(setup, gamedata, party, &field, double, heap)`.
const BTL_SETUP_PARTY: u32 = 0x0201_347C;
/// btl_setup trainer set-ups `(setup, gamedata, &field, trainer, heap)`.
const BTL_SETUP_SINGLE_0: u32 = 0x0201_3610;
const BTL_SETUP_SINGLE_1: u32 = 0x0201_3644;
const BTL_SETUP_SINGLE_2: u32 = 0x0201_3678;
const BTL_SETUP_SINGLE_3: u32 = 0x0201_36AC;
/// btl_setup `(setup, gamedata, &field, trainer, trainer 2, heap)`.
const BTL_SETUP_DOUBLE: u32 = 0x0201_36E0;
/// btl_setup `(setup, gamedata, &field, partner, trainer, trainer 2, heap)`.
const BTL_SETUP_TRIPLE: u32 = 0x0201_3734;
/// (trainer data, 1) -> trainer record.
const TRAINER_RECORD: u32 = 0x0202_A230;
/// Trainer record: class / kind (0x11 = none).
const TRAINER_KIND: u32 = 0x0202_A420;
/// POKEPARTY alloc `(heap)`.
const PARTY_NEW: u32 = 0x0201_A920;
/// POKEPARTY add `(party, pp)`.
const PARTY_ADD: u32 = 0x0201_A98C;
/// `FUN_020173EC()`: the POKEPARAM size (library code that returns 0xDC; runs inside the originals).
const POKEPARAM_SIZE: u32 = 0xDC;
/// event_battle: the wild battle event `(gsys, fieldmap, setup, kind, encount kind)`.
const BATTLE_EVENT: u32 = 0x0216_E7C8;
/// field_wfbc: encounters off `(wfbc)`.
const WFBC_OFF: u32 = 0x0219_3920;
/// field_wfbc: encounter level factor `(wfbc)`.
const WFBC_FACTOR: u32 = 0x0219_38F4;
/// FX square root (fx32).
const FX_SQRT: u32 = 0x0207_C7B4;
/// `FUN_02010FB0(rate)`: the encounter-rate modifier of the pass powers.
const PASS_POWER_RATE: u32 = 0x0201_0FB0;

/// Address of the string `"field_encount.c"`.
pub const FILE_FIELD_ENCOUNT: u32 = 0x021D_DFA4;

/// C: `FUN_overlay_d_21__021a9ed8` (unlabeled routine at `0x021A9ED8`): allocate the FIELD ENCOUNT object (0x14 bytes,
/// `Heap_AllocDebug(FUN_02188C84(fieldmap), 0x14, 1, "field_encount.c", 0x62)`): `+0` fieldmap, `+4` GAMESYS, `+8`
/// GAMEDATA, `+0xC` the ENCOUNT TABLE of the EVENTDATA_SYSTEM, `+0x10` the EFFECT ENCOUNT work (`FUN_021AA77C(heap)`),
/// then `FUN_021AA7AC(enc, +0x10)`; returns the object.
pub fn create<E: FieldEnv>(env: &mut E, fieldmap: u32) -> u32 {
    let heap = env.callv(FIELDMAP_HEAP, &[fieldmap]);
    let enc = env.callv(HEAP_ALLOC, &[heap, 0x14, 1, FILE_FIELD_ENCOUNT, 0x62]);
    env.w32(enc, fieldmap);
    let gsys = env.callv(FIELDMAP_GAMESYS, &[fieldmap]);
    env.w32(enc.wrapping_add(4), gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(enc.wrapping_add(8), gd);
    let sys = env.callv(EVENTDATA, &[gd]);
    let tab = env.callv(EVENTDATA_ENCOUNT, &[sys]);
    env.w32(enc.wrapping_add(0xC), tab);
    let eff = env.callv(EFFECT_NEW, &[heap]);
    env.w32(enc.wrapping_add(0x10), eff);
    env.callv(EFFECT_INIT, &[enc, eff]);
    enc
}

/// C: `FUN_overlay_d_21__021a9f28` (unlabeled routine at `0x021A9F28`): delete the FIELD ENCOUNT object:
/// `FUN_021AA7CC(enc, +0x10)`, `FUN_021AA794(+0x10)`, `Heap_Free(enc)`.
pub fn delete<E: FieldEnv>(env: &mut E, enc: u32) {
    let eff = env.r32(enc.wrapping_add(0x10));
    env.callv(EFFECT_EXIT, &[enc, eff]);
    let eff = env.r32(enc.wrapping_add(0x10));
    env.callv(EFFECT_FREE, &[eff]);
    env.callv(HEAP_FREE, &[enc]);
}

/// The weather id of the FIELDMAP (`FUN_021A1BF4(FUN_02188C6C(fieldmap))`).
fn weather<E: FieldEnv>(env: &mut E, fieldmap: u32) -> u32 {
    let w = env.callv(FIELDMAP_WEATHER, &[fieldmap]);
    env.callv(WEATHER_ID, &[w])
}

/// `FUN_021A9350(efp, gamedata, type, kind, weather)`: the EFP of the lead (enc_pokeset, behind the seam).
fn efp_init<E: FieldEnv>(env: &mut E, efp: u32, gd: u32, ty: u32, kind: u32, weather: u32) {
    env.call(
        EFP_INIT,
        &mut [
            CArg::Out(efp, 0x2C),
            CArg::V(gd),
            CArg::V(ty),
            CArg::V(kind),
            CArg::V(weather),
        ],
    );
}

/// `FUN_021A954C(table, efp, zone, mons)`: the accepted wild Pokemon (enc_pokeset).
fn wild_mons<E: FieldEnv>(env: &mut E, table: u32, efp: u32, zone: u32, mons: u32) -> u32 {
    env.call(
        WILD_MONS,
        &mut [
            CArg::V(table),
            CArg::In(efp, 0x2C),
            CArg::V(zone),
            CArg::Out(mons, 0x28),
        ],
    )
}

/// C: `FUN_overlay_d_21__021a9f40` (unlabeled routine at `0x021A9F40`): the wild encounter check of a step, `kind` 0
/// walking, 1 effect encounter, 2 forced; returns the battle event or 0. Kind 0 counts the step ([`count_step`]); the
/// table must be loaded (`enc +0xC` set, bit 7 of its byte 7); [`step_type`] and [`rate_of`] (type 0xFF or rate 0: none);
/// the EFP of the lead (`FUN_021A9350`, with the weather); kind 0 only: [`rate_check`] of the modified rate
/// (`FUN_021A94B0`); kind 1 adds record 0x70, the other kinds look for a roamer (`FUN_021A957C`); a roamer battle is set
/// up by [`setup_roamer`], else the wild Pokemon (`FUN_021A954C`; none: 0; a double with one accepted Pokemon becomes a
/// single) by [`setup_wild`]; then [`reset_steps`] and the battle event `FUN_0216E7C8(gsys, fieldmap, setup, 0, kind)`.
///
/// UNVERIFIED port of `021A9F40`: the EFP is a local of this function, and the spec's stub of `FUN_021A96B8` (and of
/// `FUN_021A9780`) logs its address as a value (`v`), so the hash holds the emulator's stack address. Spec fix: those
/// stubs `args=v,in:0x2C,in:0x14` / `args=v,in:0x2C,v`, and `0201347c` `in:14` (the original never writes the two
/// padding bytes `+0xE` of the FIELD SITUATION and hands stale stack bytes to it), with `CArg::In(efp, 0x2C)` /
/// `CArg::In(sit, 14)` in [`setup_wild`] / [`setup_roamer`]: with that fix (tried, reverted) this port matches.
pub fn step_check<E: FieldEnv>(env: &mut E, enc: u32, kind: u32) -> u32 {
    let fieldmap = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fieldmap]);
    let gd = env.r32(enc.wrapping_add(8));
    let ewk = env.callv(ENCOUNT_WORK, &[gd]);
    let no_roll = kind.wrapping_sub(1) <= 1;
    if !no_roll {
        count_step(env, ewk, player);
    }
    let tab = env.r32(enc.wrapping_add(0xC));
    if tab == 0 || env.r8(tab.wrapping_add(7)) & 0x80 == 0 {
        return 0;
    }
    let bonus = env.mem().push_local(4);
    let ty = step_type(env, enc, kind, bonus);
    let b = env.r8(bonus) as u32;
    env.mem().pop_local(bonus);
    let rate = rate_of(env.mem(), enc, ty, b);
    if ty == 0xFF || rate == 0 {
        return 0;
    }
    let fieldmap = env.r32(enc);
    let w = weather(env, fieldmap);
    let efp = env.mem().push_local(0x2C);
    let gd = env.r32(enc.wrapping_add(8));
    efp_init(env, efp, gd, ty, kind, w);
    let r = step_encounter(env, enc, kind, ewk, player, efp, rate, no_roll);
    env.mem().pop_local(efp);
    r
}

/// The part of [`step_check`] after the EFP is built (its locals live in the caller's windows).
#[allow(clippy::too_many_arguments)]
fn step_encounter<E: FieldEnv>(
    env: &mut E,
    enc: u32,
    kind: u32,
    ewk: u32,
    player: u32,
    efp: u32,
    rate: u32,
    no_roll: bool,
) -> u32 {
    if !no_roll {
        let gd = env.r32(enc.wrapping_add(8));
        let r = env.call(
            RATE_MODIFIER,
            &mut [CArg::In(efp, 0x2C), CArg::V(gd), CArg::V(rate)],
        );
        if !rate_check(env, ewk, r) {
            return 0;
        }
    }
    let mut rec = 0;
    if kind == 1 {
        let gd = env.r32(enc.wrapping_add(8));
        let r = env.callv(RECORDS, &[gd]);
        env.callv(RECORD_ADD, &[r, 0x70]);
    } else {
        let fieldmap = env.r32(enc);
        let zone = env.callv(FIELDMAP_ZONE, &[fieldmap]);
        let tab = env.r32(enc.wrapping_add(0xC));
        rec = env.call(
            ROAMER,
            &mut [CArg::V(tab), CArg::In(efp, 0x2C), CArg::V(zone)],
        );
    }
    let setup;
    if rec != 0 {
        setup = env.callv(BTL_SETUP_NEW, &[4]);
        setup_roamer(env, enc, efp, setup, 4, rec);
    } else {
        let mons = env.mem().push_local(0x28);
        env.mem().fill(mons, 0x28, 0);
        let fieldmap = env.r32(enc);
        let zone = env.callv(FIELDMAP_ZONE, &[fieldmap]);
        let tab = env.r32(enc.wrapping_add(0xC));
        let n = wild_mons(env, tab, efp, zone, mons);
        if n == 0 {
            env.mem().pop_local(mons);
            return 0;
        }
        let flags = env.r32(efp.wrapping_add(0x28));
        if flags & 0x20 != 0 && n == 1 {
            env.w32(efp.wrapping_add(0x28), flags & !0x20);
            env.w8(efp.wrapping_add(0x15), n as u8);
        }
        setup = env.callv(BTL_SETUP_NEW, &[4]);
        setup_wild(env, enc, efp, setup, 4, mons);
        env.mem().pop_local(mons);
    }
    reset_steps(env, ewk, player);
    let gsys = env.r32(enc.wrapping_add(4));
    let fieldmap = env.r32(enc);
    env.callv(BATTLE_EVENT, &[gsys, fieldmap, setup, 0, kind])
}

/// C: `FUN_overlay_d_21__021aa0a0` (unlabeled routine at `0x021AA0A0`): a fixed (scripted) wild battle against `species`
/// at `level`: EFP of type 0xFF, kind 0; one MON (zeroed): species, level; `flags` 4: item 0xFFFF (none); 2: shiny mode 1,
/// else 0x10: 2, else 0; 8: `+7` = 1; 0x20: gender request 1, else 0x40: 2; battle setup `FUN_02013090(4)` by
/// [`setup_wild`]; `flags` 1: setup flag 4, else 0x80: setup flag 0x80; 0x100: setup flag 0x200; returns the battle event
/// `FUN_0216E7C8(gsys, fieldmap, setup, 1, 0)`.
///
/// UNVERIFIED port of `021AA0A0`: the EFP is a local of this function, and the spec's stub of `FUN_021A96B8` (and of
/// `FUN_021A9780`) logs its address as a value (`v`), so the hash holds the emulator's stack address. Spec fix: those
/// stubs `args=v,in:0x2C,in:0x14` / `args=v,in:0x2C,v`, and `0201347c` `in:14` (the original never writes the two
/// padding bytes `+0xE` of the FIELD SITUATION and hands stale stack bytes to it), with `CArg::In(efp, 0x2C)` /
/// `CArg::In(sit, 14)` in [`setup_wild`] / [`setup_roamer`]: with that fix (tried, reverted) this port matches.
pub fn fixed_battle<E: FieldEnv>(
    env: &mut E,
    enc: u32,
    species: u32,
    level: u32,
    flags: u32,
) -> u32 {
    let fieldmap = env.r32(enc);
    env.callv(FIELDMAP_PLAYER, &[fieldmap]);
    let fieldmap = env.r32(enc);
    let w = weather(env, fieldmap);
    let efp = env.mem().push_local(0x2C);
    let gd = env.r32(enc.wrapping_add(8));
    efp_init(env, efp, gd, 0xFF, 0, w);
    let mons = env.mem().push_local(0x28);
    env.mem().fill(mons, 0x28, 0);
    env.w16(mons, species as u16);
    env.w8(mons.wrapping_add(4), level as u8);
    if flags & 4 != 0 {
        env.w16(mons.wrapping_add(2), 0xFFFF);
    }
    if flags & 2 != 0 {
        env.w8(mons.wrapping_add(6), 1);
    } else if flags & 0x10 != 0 {
        env.w8(mons.wrapping_add(6), 2);
    } else {
        env.w8(mons.wrapping_add(6), 0);
    }
    if flags & 8 != 0 {
        env.w8(mons.wrapping_add(7), 1);
    }
    if flags & 0x20 != 0 {
        env.w8(mons.wrapping_add(0x10), 1);
    } else if flags & 0x40 != 0 {
        env.w8(mons.wrapping_add(0x10), 2);
    }
    let setup = env.callv(BTL_SETUP_NEW, &[4]);
    setup_wild(env, enc, efp, setup, 4, mons);
    env.mem().pop_local(mons);
    env.mem().pop_local(efp);
    if flags & 1 != 0 {
        env.callv(BTL_SETUP_FLAGS, &[setup, 4]);
    } else if flags & 0x80 != 0 {
        env.callv(BTL_SETUP_FLAGS, &[setup, 0x80]);
    }
    if flags & 0x100 != 0 {
        env.callv(BTL_SETUP_FLAGS, &[setup, 0x200]);
    }
    let gsys = env.r32(enc.wrapping_add(4));
    let fieldmap = env.r32(enc);
    env.callv(BATTLE_EVENT, &[gsys, fieldmap, setup, 1, 0])
}

/// C: `FUN_overlay_d_21__021aa180` (unlabeled routine at `0x021AA180`): the fishing encounter (`kind` 1: on a ripple,
/// type 6; else type 5); returns the battle setup or 0: the table must be loaded; rate [`rate_of`]`(type, 0)` (0: none);
/// the EFP; kind != 1: none when [`rate_roll`] exceeds the modified rate (`FUN_021A94B0`); the wild Pokemon
/// (`FUN_021A954C`) must reach the EFP's count `+0x15`; battle setup `FUN_02013090(4)` by [`setup_wild`], setup flag 1,
/// `setup +0xC` = 6.
///
/// UNVERIFIED port of `021AA180`: the EFP is a local of this function, and the spec's stub of `FUN_021A96B8` (and of
/// `FUN_021A9780`) logs its address as a value (`v`), so the hash holds the emulator's stack address. Spec fix: those
/// stubs `args=v,in:0x2C,in:0x14` / `args=v,in:0x2C,v`, and `0201347c` `in:14` (the original never writes the two
/// padding bytes `+0xE` of the FIELD SITUATION and hands stale stack bytes to it), with `CArg::In(efp, 0x2C)` /
/// `CArg::In(sit, 14)` in [`setup_wild`] / [`setup_roamer`]: with that fix (tried, reverted) this port matches.
pub fn fishing<E: FieldEnv>(env: &mut E, enc: u32, kind: u32) -> u32 {
    let tab = env.r32(enc.wrapping_add(0xC));
    if tab == 0 || env.r8(tab.wrapping_add(7)) & 0x80 == 0 {
        return 0;
    }
    let ty = if kind == 1 { 6 } else { 5 };
    let rate = rate_of(env.mem(), enc, ty, 0);
    if rate == 0 {
        return 0;
    }
    let fieldmap = env.r32(enc);
    let w = weather(env, fieldmap);
    let efp = env.mem().push_local(0x2C);
    let gd = env.r32(enc.wrapping_add(8));
    efp_init(env, efp, gd, ty, kind, w);
    let r = fishing_tail(env, enc, kind, efp, rate);
    env.mem().pop_local(efp);
    r
}

fn fishing_tail<E: FieldEnv>(env: &mut E, enc: u32, kind: u32, efp: u32, rate: u32) -> u32 {
    if kind != 1 {
        let gd = env.r32(enc.wrapping_add(8));
        let r = env.call(
            RATE_MODIFIER,
            &mut [CArg::In(efp, 0x2C), CArg::V(gd), CArg::V(rate)],
        );
        if rate_roll(env) > r {
            return 0;
        }
    }
    let mons = env.mem().push_local(0x28);
    env.mem().fill(mons, 0x28, 0);
    let fieldmap = env.r32(enc);
    let zone = env.callv(FIELDMAP_ZONE, &[fieldmap]);
    let tab = env.r32(enc.wrapping_add(0xC));
    let n = wild_mons(env, tab, efp, zone, mons);
    if n < env.r8(efp.wrapping_add(0x15)) as u32 {
        env.mem().pop_local(mons);
        return 0;
    }
    let setup = env.callv(BTL_SETUP_NEW, &[4]);
    setup_wild(env, enc, efp, setup, 4, mons);
    env.mem().pop_local(mons);
    env.callv(BTL_SETUP_FLAGS, &[setup, 1]);
    env.w32(setup.wrapping_add(0xC), 6);
    setup
}

/// C: `FUN_overlay_d_21__021aa248` (unlabeled routine at `0x021AA248`): the WFBC encounter of a step; returns the battle
/// event or 0: [`count_step`]; `FUN_02193920(wfbc) == 1` (encounters off): none; [`wfbc_check`] fails: none; type
/// [`step_type`]`(enc, 3)`; EFP of kind 3; `FUN_021A9604(wfbc, efp, mon)` (none: 0); battle setup by [`setup_wild`],
/// [`reset_steps`], `FUN_0216E7C8(gsys, fieldmap, setup, 0, 3)`.
///
/// UNVERIFIED port of `021AA248`: the EFP is a local of this function, and the spec's stub of `FUN_021A96B8` (and of
/// `FUN_021A9780`) logs its address as a value (`v`), so the hash holds the emulator's stack address. Spec fix: those
/// stubs `args=v,in:0x2C,in:0x14` / `args=v,in:0x2C,v`, and `0201347c` `in:14` (the original never writes the two
/// padding bytes `+0xE` of the FIELD SITUATION and hands stale stack bytes to it), with `CArg::In(efp, 0x2C)` /
/// `CArg::In(sit, 14)` in [`setup_wild`] / [`setup_roamer`]: with that fix (tried, reverted) this port matches.
pub fn wfbc_step<E: FieldEnv>(env: &mut E, enc: u32, wfbc: u32) -> u32 {
    let fieldmap = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fieldmap]);
    let gd = env.r32(enc.wrapping_add(8));
    let ewk = env.callv(ENCOUNT_WORK, &[gd]);
    count_step(env, ewk, player);
    if env.callv(WFBC_OFF, &[wfbc]) == 1 {
        return 0;
    }
    if !wfbc_check(env, ewk, wfbc) {
        return 0;
    }
    let bonus = env.mem().push_local(4);
    let ty = step_type(env, enc, 3, bonus);
    env.mem().pop_local(bonus);
    let fieldmap = env.r32(enc);
    let w = weather(env, fieldmap);
    let efp = env.mem().push_local(0x2C);
    let gd = env.r32(enc.wrapping_add(8));
    efp_init(env, efp, gd, ty, 3, w);
    let mons = env.mem().push_local(0x28);
    env.mem().fill(mons, 0x28, 0);
    let found = env.call(
        WFBC_MON,
        &mut [CArg::V(wfbc), CArg::In(efp, 0x2C), CArg::Out(mons, 0x14)],
    );
    if found == 0 {
        env.mem().pop_local(mons);
        env.mem().pop_local(efp);
        return 0;
    }
    let setup = env.callv(BTL_SETUP_NEW, &[4]);
    setup_wild(env, enc, efp, setup, 4, mons);
    env.mem().pop_local(mons);
    env.mem().pop_local(efp);
    reset_steps(env, ewk, player);
    let gsys = env.r32(enc.wrapping_add(4));
    let fieldmap = env.r32(enc);
    env.callv(BATTLE_EVENT, &[gsys, fieldmap, setup, 0, 3])
}

/// C: `FUN_overlay_d_21__021aa304` (unlabeled routine at `0x021AA304`): the battle FIELD SITUATION (0x10 bytes at `sit`):
/// `+0` battle background of the zone (`FUN_02013ED8`), `+4` ground kind of the attribute value under the player
/// (`021AB580(021AB0F0(FUN_021A30C8(player)))`), `+0xA` zone, `+0xC` / `+0xD` RTC hour / minute (`FUN_0203F4CC` into a
/// 12-byte local), `+8` [`weather_class`], `+9` the season of the GAMEDATA. `+0xE..0x10` is not written: the callers'
/// locals are zeroed in the ports, the original leaves stale stack bytes there (passed on to the btl_setup calls).
pub fn field_situation<E: FieldEnv>(env: &mut E, sit: u32, gd: u32, fieldmap: u32) {
    let zone = env.callv(FIELDMAP_ZONE, &[fieldmap]);
    let player = env.callv(FIELDMAP_PLAYER, &[fieldmap]);
    let bg = env.callv(ZONE_BATTLE_BG, &[zone]);
    env.w32(sit, bg);
    let a = env.callv(PLAYER_ATTR, &[player]);
    let v = env.callv(ATTR_VALUE, &[a]);
    let k = env.callv(ATTR_GROUND, &[v]);
    env.w32(sit.wrapping_add(4), k);
    env.w16(sit.wrapping_add(0xA), zone as u16);
    let t = env.mem().push_local(12);
    env.call(RTC_TIME, &mut [CArg::Out(t, 12)]);
    let (h, m) = (env.r32(t), env.r32(t.wrapping_add(4)));
    env.mem().pop_local(t);
    env.w8(sit.wrapping_add(0xC), h as u8);
    env.w8(sit.wrapping_add(0xD), m as u8);
    let wc = weather_class(env, fieldmap);
    env.w8(sit.wrapping_add(8), wc as u8);
    let season = env.callv(SEASON, &[gd]);
    env.w8(sit.wrapping_add(9), season as u8);
}

/// C: `FUN_overlay_d_21__021aa35c` (unlabeled routine at `0x021AA35C`): the encounter type of the attribute under the
/// player, or 0xFF; `*bonus` (a byte) = 0, + 10 on attribute values 8 / 9 (`021AB140`): an invalid attribute (`021AB100`)
/// or, unless `kind` is 1, one without flag bit 2: 0xFF; water (flag bit 1): 4 for kind 1, else 3; land: kind 1: 2,
/// long grass (`021AB29C`): 1, else 0. (`FUN_02188C98(fieldmap)` is called, its result unused.)
pub fn step_type<E: FieldEnv>(env: &mut E, enc: u32, kind: u32, bonus: u32) -> u32 {
    let fieldmap = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fieldmap]);
    let fieldmap = env.r32(enc);
    env.callv(FIELDMAP_MAPPER, &[fieldmap]);
    env.w8(bonus, 0);
    let a = env.callv(PLAYER_ATTR, &[player]);
    if env.callv(ATTR_VALID, &[a]) == 0 {
        return 0xFF;
    }
    let f = env.callv(ATTR_FLAGS, &[a]);
    if kind != 1 && f & 4 == 0 {
        return 0xFF;
    }
    let v = env.callv(ATTR_VALUE, &[a]);
    if env.callv(ATTR_8_OR_9, &[v]) != 0 {
        let b = env.r8(bonus);
        env.w8(bonus, b.wrapping_add(10));
    }
    if f & 2 != 0 {
        return if kind == 1 { 4 } else { 3 };
    }
    if kind == 1 {
        return 2;
    }
    (env.callv(ATTR_LONG_GRASS, &[v]) != 0) as u32
}

/// C: `FUN_overlay_d_21__021aa3e0` (unlabeled routine at `0x021AA3E0`): the encounter rate of type `ty`:
/// `ty >= 7` (signed) gives 0; else the rate byte `table[ty]` of the ENCOUNT TABLE (`enc +0xC`; a negative type
/// reads before the table, as the original does) plus `bonus` when the byte is nonzero, 0 if that is negative.
pub fn rate_of(mem: &Mem, enc: u32, ty: u32, bonus: u32) -> u32 {
    if ty as i32 >= 7 {
        return 0;
    }
    let table = mem.r32(enc.wrapping_add(0xC));
    let mut r = mem.r8(table.wrapping_add(ty)) as i32;
    if r > 0 {
        r = r.wrapping_add(bonus as i32);
    }
    r.max(0) as u32
}

/// C: `FUN_overlay_d_21__021aa778` (unlabeled routine at `0x021AA778`): the steps of the ENCOUNT WORK `ewk`
/// (`+8`).
pub fn steps(mem: &Mem, ewk: u32) -> u32 {
    mem.r32(ewk.wrapping_add(8))
}

/// C: `FUN_overlay_d_21__021aa69c` (unlabeled routine at `0x021AA69C`): the weather class of the battle field
/// situation: weather 2 / 6 / 7 -> 2, 3 / 0xC -> 4, 4 / 5 -> 3, anything else 0.
pub fn weather_class<E: FieldEnv>(env: &mut E, fieldmap: u32) -> u32 {
    let w = env.callv(FIELDMAP_WEATHER, &[fieldmap]);
    match env.callv(WEATHER_ID, &[w]) {
        2 | 6 | 7 => 2,
        3 | 0xC => 4,
        4 | 5 => 3,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021aa6e0` (unlabeled routine at `0x021AA6E0`): the rate roll, `FUN_020056EC(0xFFFF) /
/// 0x290` (0..99).
pub fn rate_roll<E: FieldEnv>(env: &mut E) -> u32 {
    env.callv(RAND, &[0xFFFF]) / 0x290
}

/// C: `FUN_overlay_d_21__021aa3fc` (unlabeled routine at `0x021AA3FC`): does this step meet a Pokemon?
/// `rate = min(rate, 100)`; no steps since the last reset: no; exactly one: the rate is 1; then yes when
/// [`rate_roll`] `<= rate` (so a rate `r` hits `r + 1` rolls of 100). `enc` is not read.
pub fn rate_check<E: FieldEnv>(env: &mut E, ewk: u32, rate: u32) -> bool {
    let mut rate = rate.min(100);
    match steps(env.mem(), ewk) {
        0 => return false,
        1 => rate = 1,
        _ => {}
    }
    rate_roll(env) <= rate
}

/// C: `FUN_overlay_d_21__021aa42c` (unlabeled routine at `0x021AA42C`): the WFBC encounter roll: `s` = [`steps`], `f =
/// FUN_021938F4(wfbc)`, `r = FUN_020056EC(100)`; `p` = the fx32 product `((s - 1) << 12) * FX_Sqrt(f << 12)` (64-bit,
/// `+0x800 >> 12`), scaled again (`<< 14`, `+0x800 >> 12`, `>> 12`); encounter when `FUN_02010FB0(p) > r` (signed).
pub fn wfbc_check<E: FieldEnv>(env: &mut E, ewk: u32, wfbc: u32) -> bool {
    let s = steps(env.mem(), ewk);
    let f = env.callv(WFBC_FACTOR, &[wfbc]);
    let r = env.callv(RAND, &[100]);
    let q = env.callv(FX_SQRT, &[f << 12]);
    let a = (s.wrapping_sub(1) << 12) as i32 as i64;
    let prod = a.wrapping_mul(q as i32 as i64);
    let x = (prod.wrapping_add(0x800) >> 12) as u32 as i32;
    let y = (((x as i64) << 14).wrapping_add(0x800) >> 12) as u32 as i32;
    let p = (y >> 12) as u32;
    env.callv(PASS_POWER_RATE, &[p]) as i32 > r as i32
}

/// C: `FUN_overlay_d_21__021aa490` (unlabeled routine at `0x021AA490`): set up the wild battle `setup` with the Pokemon
/// `mons` (the fifth argument is a stack word): a party (`FUN_0201A920(heap)`), a 0xDC-byte POKEPARAM
/// (`Heap_AllocDebug(heap, 0xDC, 1, "field_encount.c", 0x285)`), for each of the 1 + (EFP flag 0x20) Pokemon
/// `FUN_021A96B8(pp, efp, mons + 0x14 i)` and `FUN_0201A98C(party, pp)`; the [`field_situation`] (a 0x10-byte local);
/// `FUN_0201347C(setup, gamedata, party, &sit, double, heap)`; type 1: setup flag 0x20; both blocks freed.
pub fn setup_wild<E: FieldEnv>(env: &mut E, enc: u32, efp: u32, setup: u32, heap: u32, mons: u32) {
    let party = env.callv(PARTY_NEW, &[heap]);
    let pp = env.callv(
        HEAP_ALLOC,
        &[heap, POKEPARAM_SIZE, 1, FILE_FIELD_ENCOUNT, 0x285],
    );
    let mut i = 0u32;
    while (i as i32) < (((env.r32(efp.wrapping_add(0x28)) >> 5) & 1) + 1) as i32 {
        let m = mons.wrapping_add(0x14 * i);
        env.call(
            WILD_POKEPARAM,
            &mut [CArg::V(pp), CArg::V(efp), CArg::In(m, 0x14)],
        );
        env.callv(PARTY_ADD, &[party, pp]);
        i += 1;
    }
    let sit = env.mem().push_local(0x10);
    let gd = env.r32(enc.wrapping_add(8));
    let fieldmap = env.r32(enc);
    field_situation(env, sit, gd, fieldmap);
    let double = (env.r32(efp.wrapping_add(0x28)) >> 5) & 1;
    let gd = env.r32(enc.wrapping_add(8));
    env.call(
        BTL_SETUP_PARTY,
        &mut [
            CArg::V(setup),
            CArg::V(gd),
            CArg::V(party),
            CArg::In(sit, 16),
            CArg::V(double),
            CArg::V(heap),
        ],
    );
    env.mem().pop_local(sit);
    if env.r32(efp.wrapping_add(8)) == 1 {
        env.callv(BTL_SETUP_FLAGS, &[setup, 0x20]);
    }
    env.callv(HEAP_FREE, &[pp]);
    env.callv(HEAP_FREE, &[party]);
}

/// C: `FUN_overlay_d_21__021aa548` (unlabeled routine at `0x021AA548`): set up the roamer battle `setup` with the roamer
/// record `rec` (a stack word): a party, a POKEPARAM from heap `(heap & 0x7FFF) | 0x8000` (`"field_encount.c"`, 0x2A4),
/// `FUN_021A9780(pp, efp, rec)`, added to the party; [`field_situation`]; `FUN_0201347C(setup, gamedata, party, &sit, 0,
/// heap)`; setup flag 8; both blocks freed.
pub fn setup_roamer<E: FieldEnv>(env: &mut E, enc: u32, efp: u32, setup: u32, heap: u32, rec: u32) {
    let party = env.callv(PARTY_NEW, &[heap]);
    let h = ((heap & 0x7FFF) | 0x8000) & 0xFFFF;
    let pp = env.callv(
        HEAP_ALLOC,
        &[h, POKEPARAM_SIZE, 1, FILE_FIELD_ENCOUNT, 0x2A4],
    );
    env.callv(ROAMER_POKEPARAM, &[pp, efp, rec]);
    env.callv(PARTY_ADD, &[party, pp]);
    let sit = env.mem().push_local(0x10);
    let gd = env.r32(enc.wrapping_add(8));
    let fieldmap = env.r32(enc);
    field_situation(env, sit, gd, fieldmap);
    let gd = env.r32(enc.wrapping_add(8));
    env.call(
        BTL_SETUP_PARTY,
        &mut [
            CArg::V(setup),
            CArg::V(gd),
            CArg::V(party),
            CArg::In(sit, 16),
            CArg::V(0),
            CArg::V(heap),
        ],
    );
    env.mem().pop_local(sit);
    env.callv(BTL_SETUP_FLAGS, &[setup, 8]);
    env.callv(HEAP_FREE, &[pp]);
    env.callv(HEAP_FREE, &[party]);
}

/// C: `FUN_overlay_d_21__021aa5cc` (unlabeled routine at `0x021AA5CC`): set up the trainer battle `setup` (`tr`, `tr2`
/// and `heap` are stack words): `FUN_020130C4(setup)`; the [`field_situation`]; its ground kind `+4` becomes the trainer
/// record's kind (`FUN_0202A420(FUN_0202A230(tr, 1))`) unless that is 0x11; `kind` 0: `tr2` ? double
/// `FUN_020136E0(setup, gd, &sit, tr, tr2, heap)` : `FUN_02013610(setup, gd, &sit, tr, heap)`; 1: `partner` ?
/// `FUN_02013734(setup, gd, &sit, partner, tr, tr2, heap)` : `FUN_02013644(..)`; 2: `FUN_02013678(..)`; 3:
/// `FUN_020136AC(..)`; any other kind: nothing more.
#[allow(clippy::too_many_arguments)]
pub fn setup_trainer<E: FieldEnv>(
    env: &mut E,
    enc: u32,
    setup: u32,
    kind: u32,
    partner: u32,
    tr: u32,
    tr2: u32,
    heap: u32,
) {
    let gd = env.r32(enc.wrapping_add(8));
    env.callv(BTL_SETUP_INIT, &[setup]);
    let sit = env.mem().push_local(0x10);
    let gd2 = env.r32(enc.wrapping_add(8));
    let fieldmap = env.r32(enc);
    field_situation(env, sit, gd2, fieldmap);
    let rec = env.callv(TRAINER_RECORD, &[tr, 1]);
    let k = env.callv(TRAINER_KIND, &[rec]);
    if k != 0x11 {
        env.w32(sit.wrapping_add(4), k);
    }
    let single = |env: &mut E, addr: u32| {
        env.call(
            addr,
            &mut [
                CArg::V(setup),
                CArg::V(gd),
                CArg::In(sit, 16),
                CArg::V(tr),
                CArg::V(heap),
            ],
        );
    };
    match kind {
        0 => {
            if tr2 != 0 {
                env.call(
                    BTL_SETUP_DOUBLE,
                    &mut [
                        CArg::V(setup),
                        CArg::V(gd),
                        CArg::In(sit, 16),
                        CArg::V(tr),
                        CArg::V(tr2),
                        CArg::V(heap),
                    ],
                );
            } else {
                single(env, BTL_SETUP_SINGLE_0);
            }
        }
        1 => {
            if partner != 0 {
                env.call(
                    BTL_SETUP_TRIPLE,
                    &mut [
                        CArg::V(setup),
                        CArg::V(gd),
                        CArg::In(sit, 16),
                        CArg::V(partner),
                        CArg::V(tr),
                        CArg::V(tr2),
                        CArg::V(heap),
                    ],
                );
            } else {
                single(env, BTL_SETUP_SINGLE_1);
            }
        }
        2 => single(env, BTL_SETUP_SINGLE_2),
        3 => single(env, BTL_SETUP_SINGLE_3),
        _ => {}
    }
    env.mem().pop_local(sit);
}

/// C: `FUN_overlay_d_21__021aa6f8` (unlabeled routine at `0x021AA6F8`): reset the walk counter of the ENCOUNT WORK `ewk`:
/// zero its first 0xC bytes and store the player's MMDL grid x / y / z (`+0` / `+2` / `+4`).
pub fn reset_steps<E: FieldEnv>(env: &mut E, ewk: u32, player: u32) {
    let m = env.callv(PLAYER_MMDL, &[player]);
    env.mem().fill(ewk, 0xC, 0);
    let x = env.callv(MMDL_GX, &[m]);
    env.w16(ewk, x as u16);
    let y = env.callv(MMDL_GY, &[m]);
    env.w16(ewk.wrapping_add(2), y as u16);
    let z = env.callv(MMDL_GZ, &[m]);
    env.w16(ewk.wrapping_add(4), z as u16);
}

/// C: `FUN_overlay_d_21__021aa728` (unlabeled routine at `0x021AA728`): count a step: once moved (`+6`), the steps `+8`
/// count up to 0xA000; before that, a grid position other than the one stored at the reset (`+0` / `+2` / `+4`, compared
/// in that order) sets moved and counts the step.
pub fn count_step<E: FieldEnv>(env: &mut E, ewk: u32, player: u32) {
    let m = env.callv(PLAYER_MMDL, &[player]);
    if env.r16(ewk.wrapping_add(6)) != 0 {
        let s = env.r32(ewk.wrapping_add(8));
        if s < 0xA000 {
            env.w32(ewk.wrapping_add(8), s + 1);
        }
        return;
    }
    let same = env.r16(ewk) as u32 == env.callv(MMDL_GX, &[m])
        && env.r16(ewk.wrapping_add(2)) as u32 == env.callv(MMDL_GY, &[m])
        && env.r16(ewk.wrapping_add(4)) as u32 == env.callv(MMDL_GZ, &[m]);
    if !same {
        env.w16(ewk.wrapping_add(6), 1);
        let s = env.r32(ewk.wrapping_add(8));
        env.w32(ewk.wrapping_add(8), s.wrapping_add(1));
    }
}

#[cfg(test)]
mod tests;
