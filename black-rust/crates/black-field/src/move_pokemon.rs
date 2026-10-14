//! `move_pokemon.c` (overlay 10, `0x0215E440..0x0215E840`, the first part of F1 row 10.19; F5b): the roaming Pokemon
//! (Thundurus / Tornadus): a roamer's creation, the respawn of defeated roamers, the update after a battle with one, the
//! location move on a zone change, and the zone / active queries.
//!
//! Spec: `tools/rom-diff/specs/black-field/move_pokemon.spec` (its header has the ENC SAVE roamer layout). The zone-change
//! hooks that call [`move_roamer`] and the location -> zone lookup live in `field_encount_st.c` ([`crate::encount_st`]).
//! Every callee outside this area is reached through [`FieldEnv::call`] by address; callees inside it are Rust calls.
//!
//! Roamer `i` 0 is species 0x282 (Thundurus), `i` 1 is 0x281 (Tornadus), both level 40. On every zone change a roaming
//! roamer hides (location 16, zone 0xFFFF) with 20 % probability unless already hidden; otherwise it moves to a uniformly
//! picked location of its group ([`ROAM_GROUPS`]`[FUN_020113D8(season)]`) other than its current one and `exclude`.

use crate::encount_st::SWARM_ZONES;
use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_E440,
    0x0215_E580,
    0x0215_E5B0,
    0x0215_E6A8,
    0x0215_E6F8,
    0x0215_E724,
    0x0215_E740,
    0x0215_E7DC,
    0x0215_E804,
];

/// Address of the string `"move_pokemon.c"` (the file argument of the allocation in [`create_roamer`]).
pub const FILE_MOVE_POKEMON: u32 = 0x0217_293C;
/// Species of roamer 0 (Thundurus) and roamer 1 (Tornadus).
pub const ROAMER_SPECIES: [u32; 2] = [0x282, 0x281];
/// The level a roamer is created at.
pub const ROAMER_LEVEL: u32 = 40;
/// The location index of a hidden roamer.
pub const HIDDEN: u32 = 16;

/// ROM table `0x021711A8`: the roamer location groups (`u16` count, up to 4 location indices), indexed by
/// `FUN_020113D8(season)`.
pub const ROAM_GROUPS: [(u16, [u8; 4]); 5] = [
    (3, [0, 1, 15, 0]),
    (3, [2, 3, 14, 0]),
    (3, [4, 5, 6, 0]),
    (3, [7, 8, 9, 0]),
    (4, [10, 11, 12, 13]),
];

/// `FUN_02012EBC(gd)`: the save data of the GAMEDATA.
const GD_SAVE: u32 = 0x0201_2EBC;
/// `FUN_0200D418(save)` (thunk to `FUN_020071CC(save, 0x38)`): the ENC SAVE block.
const ENC_SAVE: u32 = 0x0200_D418;
/// `FUN_0200D468(sv, i)`: state of roamer `i` (2 = defeated, 3 = caught).
const SV_STATE: u32 = 0x0200_D468;
/// `FUN_0200D470(sv, i, state)`: set the state of roamer `i`.
const SV_SET_STATE: u32 = 0x0200_D470;
/// `FUN_0200D478(sv, i)`: location index of roamer `i`.
const SV_LOC: u32 = 0x0200_D478;
/// `FUN_0200D480(sv, i, loc)`: set the location index of roamer `i`.
const SV_SET_LOC: u32 = 0x0200_D480;
/// `FUN_0200D488(sv, i)`: is roamer `i` active.
const SV_ACTIVE: u32 = 0x0200_D488;
/// `FUN_0200D494(rec)`: clear a roamer record.
const REC_CLEAR: u32 = 0x0200_D494;
/// `FUN_0200D4AC(sv, i)`: the record of roamer `i`.
const SV_REC: u32 = 0x0200_D4AC;
/// `FUN_0200D4B8(rec, id)`: roamer record field `id`.
const REC_GET: u32 = 0x0200_D4B8;
/// `FUN_0200D50C(rec, id, v)`: set roamer record field `id`.
const REC_SET: u32 = 0x0200_D50C;
/// `FUN_02017E1C(mon, id, x)`: PokeParam get.
const POKE_GET: u32 = 0x0201_7E1C;
/// `FUN_02005728(n)` (black-core `rng::next_scaled`): a random number below `n`.
const RAND: u32 = 0x0200_5728;

/// The ENC SAVE of GAMEDATA `gd` (`FUN_0200D418(FUN_02012EBC(gd))`).
fn enc_save<E: FieldEnv>(env: &mut E, gd: u32) -> u32 {
    let save = env.callv(GD_SAVE, &[gd]);
    env.callv(ENC_SAVE, &[save])
}

/// C: `FUN_0215E440`: create roamer `i` (0 or 1; other values do nothing): the
/// record's species / level 40 / field 9 = 1, a temporary PokeParam (`FUN_02017588(mon, species, 40,
/// FUN_02008530(MYSTATUS), 0, -1, 0, -1)`, heap 0x8003) whose fields 0xAC / PID / 0x70 / max HP go into record fields
/// 2 / 3 / 10 / 5 (7 = 0, 8 = 1), the pokedex "seen" entry, then [`move_roamer`]`(sv, season, i, 5)` and state 1.
pub fn create_roamer<E: FieldEnv>(env: &mut E, gd: u32, i: u32) {
    let save = env.callv(GD_SAVE, &[gd]);
    let sv = env.callv(ENC_SAVE, &[save]);
    let rec = env.callv(SV_REC, &[sv, i]);
    let species = match i {
        0 => ROAMER_SPECIES[0],
        1 => ROAMER_SPECIES[1],
        _ => return,
    };
    env.callv(REC_SET, &[rec, 4, species]);
    env.callv(REC_SET, &[rec, 6, ROAMER_LEVEL]);
    env.callv(REC_SET, &[rec, 9, 1]);
    let my = env.callv(0x0201_2944, &[gd]);
    let mon = env.callv(0x0203_0734, &[0x8003, 0xDC, 1, FILE_MOVE_POKEMON, 0x12E]);
    let id = env.callv(0x0200_8530, &[my]);
    env.callv(
        0x0201_7588,
        &[mon, species, ROAMER_LEVEL, id, 0, u32::MAX, 0, u32::MAX],
    );
    env.callv(REC_SET, &[rec, 7, 0]);
    env.callv(REC_SET, &[rec, 8, 1]);
    for (param, field) in [(0xAC, 2), (0, 3), (0x70, 10), (0xA1, 5)] {
        let v = env.callv(POKE_GET, &[mon, param, 0]);
        env.callv(REC_SET, &[rec, field, v]);
    }
    let dex = env.callv(0x0200_CA10, &[save]);
    env.callv(0x0200_CFA0, &[dex, mon]);
    env.callv(0x0203_07B0, &[mon]);
    let season = env.callv(0x0201_2984, &[gd]);
    move_roamer(env, sv, season, i, 5);
    env.callv(SV_SET_STATE, &[sv, i, 1]);
}

/// C: `FUN_0215E580`: recreate every roamer in state 2 (defeated) with
/// [`create_roamer`].
pub fn respawn_roamers<E: FieldEnv>(env: &mut E, gd: u32) {
    let sv = enc_save(env, gd);
    for i in 0..2 {
        if env.callv(SV_STATE, &[sv, i]) == 2 {
            create_roamer(env, gd, i);
        }
    }
}

/// C: `FUN_0215E5B0`: after a wild battle (`setup +4 == 0`, `+0 == 0`) against
/// party member 0 of the setup: not a roamer -> 30 % chance (`FUN_02005728(100) < 30`) that every roamer moves
/// (`FUN_0215E364(gd)`); a roamer: result (`setup +0x8C`) 1 or 2 with 0 HP -> record cleared, hidden, state 2
/// (respawns); result 5 (caught) -> cleared, hidden, state 3; otherwise its HP and status go back into the record and
/// every roamer moves. (`FUN_02012AC8(FUN_020127A4(gd))` is called first; its result is unused.)
pub fn after_battle<E: FieldEnv>(env: &mut E, gd: u32, setup: u32) {
    if env.r32(setup + 4) != 0 || env.r32(setup) != 0 {
        return;
    }
    let p = env.callv(0x0201_27A4, &[gd]);
    env.callv(0x0201_2AC8, &[p]);
    let party = env.callv(0x0201_313C, &[setup, 1]);
    let mon = env.callv(0x0201_AB28, &[party, 0]);
    let species = env.callv(POKE_GET, &[mon, 5, 0]) & 0xFFFF;
    let hp = env.callv(POKE_GET, &[mon, 0xA0, 0]) & 0xFFFF;
    let status = env.callv(POKE_GET, &[mon, 0x9D, 0]) & 0xFF;
    let sv = enc_save(env, gd);
    let rec = roamer_record(env, sv, species);
    if rec == 0 {
        if env.callv(RAND, &[100]) < 30 {
            env.callv(0x0215_E364, &[gd]);
        }
        return;
    }
    let i = roamer_index(species);
    let result = env.r32(setup + 0x8C);
    if result.wrapping_sub(1) <= 1 && hp == 0 {
        env.callv(REC_CLEAR, &[rec]);
        env.callv(SV_SET_LOC, &[sv, i, HIDDEN]);
        env.callv(SV_SET_STATE, &[sv, i, 2]);
    } else if result == 5 {
        env.callv(REC_CLEAR, &[rec]);
        env.callv(SV_SET_LOC, &[sv, i, HIDDEN]);
        env.callv(SV_SET_STATE, &[sv, i, 3]);
    } else {
        env.callv(REC_SET, &[rec, 5, hp]);
        env.callv(REC_SET, &[rec, 7, status]);
        env.callv(0x0215_E364, &[gd]);
    }
}

/// C: `FUN_0215E6A8`: 6 when roamer 0 is active in `zone`, 7 when roamer 1 is,
/// else 0xFFFF (the zone of a roamer is `FUN_0215E3E4(sv, i)`, [`crate::encount_st::swarm_zone`]).
pub fn roamer_in_zone<E: FieldEnv>(env: &mut E, gd: u32, zone: u32) -> u32 {
    let sv = enc_save(env, gd);
    for (i, r) in [(0, 6), (1, 7)] {
        if env.callv(SV_ACTIVE, &[sv, i]) != 0 && env.callv(0x0215_E3E4, &[sv, i]) == zone {
            return r;
        }
    }
    0xFFFF
}

/// C: `FUN_0215E6F8`: 1 when roamer 0 or 1 is active (tested in order), else 0.
pub fn any_active<E: FieldEnv>(env: &mut E, gd: u32) -> u32 {
    let sv = enc_save(env, gd);
    (env.callv(SV_ACTIVE, &[sv, 0]) != 0 || env.callv(SV_ACTIVE, &[sv, 1]) != 0) as u32
}

/// C: `FUN_0215E724`: the roamer index of `species` (0x282 -> 0, 0x281 -> 1,
/// other -> 2).
pub fn roamer_index(species: u32) -> u32 {
    match species {
        0x281 => 1,
        0x282 => 0,
        _ => 2,
    }
}

/// C: `FUN_0215E740`: move roamer `i`: unless it is hidden, 20 % chance
/// (`FUN_02005728(100) < 20`) to hide ([`set_location`] 16, zone 0xFFFF); otherwise pick `FUN_02005728(n)` among the
/// `n` locations of group [`ROAM_GROUPS`]`[(u16) FUN_020113D8(season)]` that are neither the current one nor
/// `exclude`; its zone is `SWARM_ZONES[loc]` (0x13F past 16).
pub fn move_roamer<E: FieldEnv>(env: &mut E, sv: u32, season: u32, i: u32, exclude: u32) {
    let cur = env.callv(SV_LOC, &[sv, i]);
    let g = env.callv(0x0201_13D8, &[season]) & 0xFFFF;
    let (count, locs) = ROAM_GROUPS[g as usize];
    if cur != HIDDEN && env.callv(RAND, &[100]) < 20 {
        set_location(env, sv, i, HIDDEN, 0xFFFF);
        return;
    }
    let mut list = [0u8; 4];
    let mut n = 0;
    for &b in locs.iter().take(count as usize) {
        let b32 = b as u32;
        if b32 != cur && b32 != exclude {
            list[n] = b;
            n += 1;
        }
    }
    let k = env.callv(RAND, &[n as u32]);
    let loc = list[k as usize] as u32;
    let zone = if loc > 16 {
        0x13F
    } else {
        SWARM_ZONES[loc as usize] as u32
    };
    set_location(env, sv, i, loc, zone);
}

/// C: `FUN_0215E7DC`: roamer `i` goes to location `loc` with zone `zone` (record
/// field 1).
pub fn set_location<E: FieldEnv>(env: &mut E, sv: u32, i: u32, loc: u32, zone: u32) {
    let rec = env.callv(SV_REC, &[sv, i]);
    env.callv(SV_SET_LOC, &[sv, i, loc]);
    env.callv(REC_SET, &[rec, 1, zone]);
}

/// C: `FUN_0215E804`: the record of the first active roamer whose species (record
/// field 4) is `species`, or 0.
pub fn roamer_record<E: FieldEnv>(env: &mut E, sv: u32, species: u32) -> u32 {
    for i in 0..2 {
        if env.callv(SV_ACTIVE, &[sv, i]) != 0 {
            let rec = env.callv(SV_REC, &[sv, i]);
            if env.callv(REC_GET, &[rec, 4]) == species {
                return rec;
            }
        }
    }
    0
}

#[cfg(test)]
mod tests;
