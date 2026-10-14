//! EXP of a Pokemon record at its current level (P3; `poke_tool.c`, 0x0201854C).
//!
//! The rest of the EXP code is elsewhere: the curve lookups and level-from-EXP are [`crate::growth`],
//! storing the level after an EXP change is [`crate::create::recalc`] (`FUN_020185C4`), the level-up
//! learnset scan is [`crate::create::next_level_up_move`] (`FUN_02018328`). The battle EXP formula is
//! in `black-battle` (`calc::exp_yield` 0x021D7EB4, `server_flow::events::exp_scale` 0x021CB55C /
//! `exp_distribute` 0x021CB2D4, `BattleMon::gain_exp` 0x021D69B8, `exp_for_max_level` 0x021D6B6C).
//!
//! Verified per function with `tools/check-fn.sh black-pokemon exp --per-function`
//! (spec `tools/rom-diff/specs/black-pokemon/exp.spec`).

use crate::hooks::InitHooks;
use crate::poke_tool::box_get_value;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0201_854C];

/// C: `sub_0201854C(mon)`: the EXP a record needs to be at the level it has now: `FUN_02018584` (the
/// species' growth curve, [`crate::growth::exp_for_species_level`], behind
/// [`InitHooks::exp_for_level`]) for its species (id 5), form (id 0x6F) and level (id 0x9E), each read
/// through the box accessor and truncated to 16 bits.
pub fn exp_at_level(mon: &mut [u8], h: &mut impl InitHooks) -> u32 {
    let species = box_get_value(mon, 5, 0, h);
    let form = box_get_value(mon, 0x6F, 0, h);
    let level = box_get_value(mon, 0x9E, 0, h);
    h.exp_for_level(species & 0xFFFF, form & 0xFFFF, level & 0xFFFF)
}

#[cfg(test)]
mod tests;
