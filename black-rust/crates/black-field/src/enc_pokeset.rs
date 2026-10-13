//! The encounter Pokemon set-up of overlay 21 (`0x021A9350..0x021A9ED8`; the original enc_pokeset.c, counted by
//! F1 in row 21.28 "field_menu.c"): the encounter context (EFP) of the lead Pokemon, the rate modifiers, the slot
//! group of an encounter type, the slot pick, the level / form roll, the ability effects, the swarm and WFBC
//! encounters and the POKEPARAM set-up of a wild or roaming Pokemon.
//!
//! F5a-spec specced all 25 functions (`tools/rom-diff/specs/black-field/enc_pokeset.spec`, whose header documents
//! the EFP (0x2C bytes) with its FLAGS word and the MON descriptor (0x14 bytes)); this module holds the smoke
//! ports. The encounter rolls use the global 64-bit LCG of `black_core::rng` (`FUN_020056EC`, `next(max)`),
//! reached through [`FieldEnv::call`].

use crate::hooks::{FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_94B0,
    0x021A_996C,
    0x021A_9DC8,
    0x021A_9DE0,
    0x021A_9E64,
    0x021A_9E98,
];

/// `FUN_020056EC(max)`: the global LCG's next value below `max` (`black_core::rng::MathRandContext32::next`).
const RAND: u32 = 0x0200_56EC;
/// `FUN_02010FB0(rate)`: the encounter-rate modifier of the pass powers.
const PASS_POWER_RATE: u32 = 0x0201_0FB0;

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

#[cfg(test)]
mod tests;
