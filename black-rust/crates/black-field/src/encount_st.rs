//! `field_encount_st.c` (overlay 10, `0x0215E2B8..0x0215E440`, F1 row 10.18): the zone's wild encounter table
//! load (ARC 126, one or four seasonal 0xE8-byte tables, see `black_data::encounters`), the FIELD ENCOUNT ST
//! work alloc, the roamer (move_pokemon) hooks of a zone change and the swarm zone table.
//!
//! F5a-spec specced all 10 functions (`tools/rom-diff/specs/black-field/encount_st.spec`, whose header documents
//! the ENCOUNT TABLE and the ENC SAVE block); F5a ported all of them. Callees outside the area are
//! reached through [`FieldEnv::call`] by address.

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_E2B8,
    0x0215_E31C,
    0x0215_E33C,
    0x0215_E34C,
    0x0215_E358,
    0x0215_E364,
    0x0215_E3A0,
    0x0215_E3E4,
    0x0215_E404,
    0x0215_E428,
];

/// `zonedata`: zone -> ARC 126 member (the encount id; 0xFFFF none).
const ZONE_ENCOUNT_ID: u32 = 0x0201_3EC0;
/// Archive member size `(handle, member)`.
const ARC_MEMBER_SIZE: u32 = 0x0204_93D4;
/// Archive read `(handle, member, offset, size, dst)`.
const ARC_READ: u32 = 0x0204_944C;
/// Address of the string `"field_encount_st.c"`.
pub const FILE_FIELD_ENCOUNT_ST: u32 = 0x0217_2928;
/// FIELDMAP -> FIELD ENCOUNT.
const FIELDMAP_ENCOUNT: u32 = 0x0218_8CE8;
/// effect_encount `021AA7DC(enc, effect)`: interval and probability from the table's kind.
const EFFECT_SET_PARAMS: u32 = 0x021A_A7DC;
/// GAMEDATA -> ENCOUNT WORK (0x20 bytes).
const ENCOUNT_WORK: u32 = 0x0201_2A38;
/// GAMESYS -> GAMEDATA.
const GAMEDATA: u32 = 0x0201_2EBC;
/// GAMEDATA -> ENC SAVE (save block 0x38; a thunk to `020071CC`).
const ENC_SAVE: u32 = 0x0200_D418;
/// GAMEDATA byte getter (the season).
const SEASON: u32 = 0x0201_2984;
/// ENC SAVE roamer record `i` (result unused by `0215E3A0`).
const SV_ROAMER: u32 = 0x0200_D4AC;

/// `move_pokemon.c`: move roamer `i` (`(sv, season, i, 0xFF)`).
const MOVE_ROAMER: u32 = 0x0215_E740;
/// ENC SAVE `+0x2C[i]` (byte).
const SV_BYTE_2C: u32 = 0x0200_D478;
/// ENC SAVE: roamer `i` active.
const SV_ROAMER_ACTIVE: u32 = 0x0200_D488;
/// ENC SAVE: push a location (`(sv, loc)`).
const SV_PUSH_LOCATION: u32 = 0x0200_D45C;

/// ROM `0x021711C6`: the 17 swarm zones (u16), indexed by ENC SAVE `+0x2C[i]`.
pub const SWARM_ZONES: [u16; 17] = [
    319, 321, 326, 329, 331, 337, 345, 348, 355, 365, 368, 370, 374, 378, 383, 387, 0xFFFF,
];

/// C: `FUN_overlay_d_10__0215e2b8` (unlabeled routine at `0x0215E2B8`): load the ENCOUNT TABLE of `zone` into
/// the 0xE8 bytes at `tab` from the archive `handle`: zero `tab`; `m = FUN_02013EC0(zone)` (0xFFFF: none, 0); the
/// member holds `size / 0xE8` tables (unsigned divide); other than 1 or 4: 0; one table serves every season
/// (`season = 0`); read table `season` (`FUN_0204944C(handle, m, season * 0xE8, 0xE8, tab)`), set bit 7 of
/// `tab[7]` (loaded) and return 1.
pub fn load_table<E: FieldEnv>(env: &mut E, tab: u32, handle: u32, zone: u32, season: u32) -> u32 {
    env.mem().fill(tab, 0xE8, 0);
    let m = env.callv(ZONE_ENCOUNT_ID, &[zone]);
    if m == 0xFFFF {
        return 0;
    }
    let n = env.callv(ARC_MEMBER_SIZE, &[handle, m]) / 0xE8;
    if n != 1 && n != 4 {
        return 0;
    }
    let season = if n == 1 { 0 } else { season };
    env.call(
        ARC_READ,
        &mut [
            CArg::V(handle),
            CArg::V(m),
            CArg::V(season.wrapping_mul(0xE8)),
            CArg::V(0xE8),
            CArg::Out(tab, 0xE8),
        ],
    );
    let b = env.r8(tab.wrapping_add(7));
    env.w8(tab.wrapping_add(7), b | 0x80);
    1
}

/// C: `FUN_overlay_d_10__0215e31c` (unlabeled routine at `0x0215E31C`): allocate the 0x20-byte FIELD ENCOUNT ST
/// work (`Heap_AllocDebug(heap, 0x20, 1, "field_encount_st.c", 0x2D)`).
pub fn work_new<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    env.callv(HEAP_ALLOC, &[heap, 0x20, 1, FILE_FIELD_ENCOUNT_ST, 0x2D])
}

/// C: `FUN_overlay_d_10__0215e33c` (unlabeled routine at `0x0215E33C`): `enc = FUN_02188CE8(fieldmap)` (the FIELD
/// ENCOUNT); `FUN_021AA7DC(enc, enc +0x10)` (effect_encount: interval and probability from the table's kind).
pub fn effect_setup<E: FieldEnv>(env: &mut E, fieldmap: u32) {
    let enc = env.callv(FIELDMAP_ENCOUNT, &[fieldmap]);
    let eff = env.r32(enc.wrapping_add(0x10));
    env.callv(EFFECT_SET_PARAMS, &[enc, eff]);
}

/// C: `FUN_overlay_d_10__0215e34c` (unlabeled routine at `0x0215E34C`): clear the effect-encounter step counter
/// (`FUN_02012A38(gamedata) +0xC = 0`; the zone-change hook).
pub fn reset_effect_steps<E: FieldEnv>(env: &mut E, gamedata: u32) {
    let w = env.callv(ENCOUNT_WORK, &[gamedata]);
    env.w32(w.wrapping_add(0xC), 0);
}

/// C: `FUN_overlay_d_10__0215e358` (unlabeled routine at `0x0215E358`): `FUN_0215E740(sv, season, i, 0xFF)`
/// (move roamer `i`); returns its result.
pub fn move_roamer<E: FieldEnv>(env: &mut E, sv: u32, season: u32, i: u32) -> u32 {
    env.callv(MOVE_ROAMER, &[sv, season, i, 0xFF])
}

/// C: `FUN_overlay_d_10__0215e364` (unlabeled routine at `0x0215E364`): every active roamer moves:
/// `sv = FUN_0200D418(FUN_02012EBC(gsys))`, `season = FUN_02012984(gsys)`; for `i` = 0, 1: when
/// `FUN_0200D488(sv, i) != 0`, [`move_roamer`]`(sv, season, i)`.
pub fn move_roamers<E: FieldEnv>(env: &mut E, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let sv = env.callv(ENC_SAVE, &[gd]);
    let season = env.callv(SEASON, &[gsys]);
    for i in 0..2 {
        if env.callv(SV_ROAMER_ACTIVE, &[sv, i]) != 0 {
            move_roamer(env, sv, season, i);
        }
    }
}

/// C: `FUN_overlay_d_10__0215e3a0` (unlabeled routine at `0x0215E3A0`): as [`move_roamers`], with a
/// `FUN_0200D4AC(sv, i)` call (result unused) before each test.
pub fn move_roamers_b<E: FieldEnv>(env: &mut E, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let sv = env.callv(ENC_SAVE, &[gd]);
    let season = env.callv(SEASON, &[gsys]);
    for i in 0..2 {
        env.callv(SV_ROAMER, &[sv, i]);
        if env.callv(SV_ROAMER_ACTIVE, &[sv, i]) != 0 {
            move_roamer(env, sv, season, i);
        }
    }
}

/// C: `FUN_overlay_d_10__0215e3e4` (unlabeled routine at `0x0215E3E4`): the swarm zone of slot `i`:
/// `k = FUN_0200D478(sv, i)` (the original passes its own r1 through); `k > 16`: 319, else [`SWARM_ZONES`]`[k]`.
pub fn swarm_zone<E: FieldEnv>(env: &mut E, sv: u32, i: u32) -> u32 {
    let k = env.callv(SV_BYTE_2C, &[sv, i]);
    if k > 16 {
        319
    } else {
        SWARM_ZONES[k as usize] as u32
    }
}

/// C: `FUN_overlay_d_10__0215e404` (unlabeled routine at `0x0215E404`): is roamer 0 or 1 active
/// (`FUN_0200D488(sv, i) != 0`, tested in order)?
pub fn any_roamer_active<E: FieldEnv>(env: &mut E, sv: u32) -> bool {
    (0..2).any(|i| env.callv(SV_ROAMER_ACTIVE, &[sv, i]) != 0)
}

/// C: `FUN_overlay_d_10__0215e428` (unlabeled routine at `0x0215E428`): when a roamer is active
/// ([`any_roamer_active`]), push the location `loc` (`FUN_0200D45C(sv, loc)`).
pub fn push_location<E: FieldEnv>(env: &mut E, sv: u32, loc: u32) {
    if any_roamer_active(env, sv) {
        env.callv(SV_PUSH_LOCATION, &[sv, loc]);
    }
}

#[cfg(test)]
mod tests;
