//! `field_encount_st.c` (overlay 10, `0x0215E2B8..0x0215E440`, F1 row 10.18): the zone's wild encounter table
//! load (ARC 126, one or four seasonal 0xE8-byte tables, see `black_data::encounters`), the FIELD ENCOUNT ST
//! work alloc, the roamer (move_pokemon) hooks of a zone change and the swarm zone table.
//!
//! F5a-spec specced all 10 functions (`tools/rom-diff/specs/black-field/encount_st.spec`, whose header documents
//! the ENCOUNT TABLE and the ENC SAVE block); this module holds the smoke ports. Callees outside the area are
//! reached through [`FieldEnv::call`] by address.

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0215_E358, 0x0215_E3E4, 0x0215_E404, 0x0215_E428];

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

/// C: `FUN_overlay_d_10__0215e358` (unlabeled routine at `0x0215E358`): `FUN_0215E740(sv, season, i, 0xFF)`
/// (move roamer `i`); returns its result.
pub fn move_roamer<E: FieldEnv>(env: &mut E, sv: u32, season: u32, i: u32) -> u32 {
    env.callv(MOVE_ROAMER, &[sv, season, i, 0xFF])
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
