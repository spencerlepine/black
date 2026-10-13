//! `field_encount.c` (overlay 21, `0x021A9ED8..0x021AA77C`, F1 row 21.29): the FIELD ENCOUNT object, the wild
//! encounter check of a step, the fixed (scripted) wild battle, the fishing check, the WFBC encounter, the battle
//! field situation, the trainer battle setup and the walk counter.
//!
//! F5a-spec specced all 19 functions (`tools/rom-diff/specs/black-field/encount.spec`, whose header documents the
//! encounter roll step by step, the FIELD ENCOUNT object (0x14), the ENCOUNT WORK (0x20) and the FIELD
//! SITUATION (0x10)); this module holds the smoke ports.
//!
//! Encounter types (the index of the rate byte and of the slot group in an ENCOUNT TABLE): 0 grass, 1 dark
//! grass (doubles), 2 shaking grass, 3 surf, 4 rippling water, 5 fishing, 6 fishing on a ripple.

use crate::hooks::{FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_A3E0,
    0x021A_A3FC,
    0x021A_A69C,
    0x021A_A6E0,
    0x021A_A778,
];

/// `FUN_020056EC(max)`: the global LCG's next value below `max`.
const RAND: u32 = 0x0200_56EC;
/// FIELDMAP -> weather system.
const FIELDMAP_WEATHER: u32 = 0x0218_8C6C;
/// Weather id of the weather system.
const WEATHER_ID: u32 = 0x021A_1BF4;

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

#[cfg(test)]
mod tests;
