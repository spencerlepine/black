//! `effect_encount.c` (overlay 21, `0x021AA77C..0x021AB0D4`, the logic part of F1 row 21.30): the effect encounters
//! (shaking grass, dust clouds, rippling water, bird shadows): the step trigger, the candidate search around the
//! player, the placement, the stepped-on check and the items of dust clouds and bird shadows. The effect objects
//! themselves (fldeff_encount.c, F1 row 21.38) are drawn by U2; this code reaches them only by address (D3).
//!
//! F5a-spec specced all 29 functions (`tools/rom-diff/specs/black-field/effect_encount.spec`, whose header
//! documents the trigger, the search window, the EFFECT ENCOUNT work (0x5C0) and the effect state in the ENCOUNT
//! WORK); this module holds the smoke ports.

use crate::hooks::{FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_A7DC,
    0x021A_AB48,
    0x021A_ABC4,
    0x021A_ABEC,
    0x021A_B070,
];

/// `FUN_02005728(max)`: the global LCG's next value scaled below `max` (`MathRandContext32::next_scaled`).
const RAND_SCALED: u32 = 0x0200_5728;

/// EFFECT ENCOUNT `+0x5B8 u16`: the step interval.
pub const EFF_INTERVAL: u32 = 0x5B8;
/// EFFECT ENCOUNT `+0x5BA u16`: the probability (per mille / 10).
pub const EFF_PROBABILITY: u32 = 0x5BA;
/// EFFECT ENCOUNT `+0x5BC u16`: the item of the last dust cloud / bird shadow.
pub const EFF_ITEM: u32 = 0x5BC;

/// ROM `0x021D8E70`: (interval, probability) per effect kind 0..2 (the table's byte 7 & 0x7F).
pub const KIND_PARAMS: [[u8; 2]; 3] = [[20, 10], [20, 10], [5, 15]];

/// C: `FUN_021AA7DC` (unlabeled routine at `0x021AA7DC`): the interval and probability of the zone's effect kind:
/// `k = table byte 7 & 0x7F` (`enc +0xC`), more than 2 counts as 0; `eff +0x5B8 / +0x5BA` = [`KIND_PARAMS`]`[k]`.
pub fn set_params(mem: &mut Mem, enc: u32, eff: u32) {
    let table = mem.r32(enc.wrapping_add(0xC));
    let mut k = (mem.r8(table.wrapping_add(7)) & 0x7F) as usize;
    if k > 2 {
        k = 0;
    }
    mem.w16(eff.wrapping_add(EFF_INTERVAL), KIND_PARAMS[k][0] as u16);
    mem.w16(eff.wrapping_add(EFF_PROBABILITY), KIND_PARAMS[k][1] as u16);
}

/// C: `FUN_overlay_d_21__021aab48` (unlabeled routine at `0x021AAB48`): the item of the last dust cloud / bird
/// shadow (`u16 +0x5BC` of the EFFECT ENCOUNT work `enc +0x10`).
pub fn item(mem: &Mem, enc: u32) -> u16 {
    mem.r16(mem.r32(enc.wrapping_add(0x10)).wrapping_add(EFF_ITEM))
}

/// C: `FUN_overlay_d_21__021aabc4` (unlabeled routine at `0x021AABC4`): count a step of the effect counter
/// (`ewk +0xC`, unless it is 0xFFFFFFFF) and tell whether it reached the interval `eff +0x5B8`.
pub fn step_due(mem: &mut Mem, ewk: u32, eff: u32) -> bool {
    let a = ewk.wrapping_add(0xC);
    let c = mem.r32(a);
    if c != u32::MAX {
        mem.w32(a, c + 1);
    }
    mem.r32(a) >= mem.r16(eff.wrapping_add(EFF_INTERVAL)) as u32
}

/// C: `FUN_overlay_d_21__021aabec` (unlabeled routine at `0x021AABEC`): restart the effect counter (`ewk +0xC = 0`).
pub fn reset_counter(mem: &mut Mem, ewk: u32) {
    mem.w32(ewk.wrapping_add(0xC), 0);
}

/// C: `FUN_overlay_d_21__021ab070` (unlabeled routine at `0x021AB070`): the item of a bird shadow:
/// `FUN_02005728(1000) < 900` (as u16): a wing `565 + FUN_02005728(600) / 100` (565..570), else 571.
pub fn shadow_item<E: FieldEnv>(env: &mut E) -> u32 {
    if env.callv(RAND_SCALED, &[1000]) & 0xFFFF < 900 {
        let w = (env.callv(RAND_SCALED, &[600]) / 100) & 0xFFFF;
        (w + 0x235) & 0xFFFF
    } else {
        0x23B
    }
}

#[cfg(test)]
mod tests;
