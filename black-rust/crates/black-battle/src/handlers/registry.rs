//! Ability registration (asm/battle_calc.s 0x021D8484-0x021D85B0): ties a Pokemon's ability
//! to an event factor of type 4.
//!
//! The original searches the registry at 0x021F0E74 (ability ID, handler-table getter) and
//! calls the getter, which returns the handler table's address and writes a count word
//! (low halfword: number of handlers, high halfword: sub-priority, default 5). The registry
//! and the 156 getters that only return a constant pair are ROM data and stay behind
//! [`AbilityRegistry`]; the two getters that compute their result (abilities 100 and 143) are
//! ported here.

use crate::battle_mon::{value, BattleMon, BppEnv};
use crate::event_factor::{EventSystem, FactorId, TYPE_ABILITY};

/// The registry lookup: the table address and count word the ability's getter returns, or
/// `None` when the ability has no registry entry.
pub trait AbilityRegistry {
    fn getter(&self, ability: u32) -> Option<(u32, u32)>;
}

/// C: `FUN_overlay_d_93__021d84f0`: packs two halfwords into a word.
pub fn pack_u16_pair(hi: u32, lo: u32) -> u32 {
    (hi << 16) | lo
}

/// C: `FUN_overlay_d_93__021d84f8`: splits a getter's count word into
/// `(sub_priority, count)`; the sub-priority is the high halfword, or 5 when that is 0.
pub fn split_getter_word(word: u32) -> (u32, u32) {
    let priority = match word >> 16 {
        0 => 5,
        p => p,
    };
    (priority, word & 0xFFFF)
}

/// Handler table of ability 100.
const TABLE_ABILITY_100: u32 = 0x021F_05A0;
/// Handler table of ability 143.
const TABLE_ABILITY_143: u32 = 0x021F_0600;

/// C: unlabeled getter at `0x021D88D0` (ability 100): one handler, sub-priority 7. Returns
/// `(table, count word)`.
pub fn getter_021d88d0() -> (u32, u32) {
    (TABLE_ABILITY_100, pack_u16_pair(7, 1))
}

/// C: unlabeled getter at `0x021DC564` (ability 143): one handler, sub-priority 4. Returns
/// `(table, count word)`.
pub fn getter_021dc564() -> (u32, u32) {
    (TABLE_ABILITY_143, pack_u16_pair(4, 1))
}

/// The ported getters by ability ID (`None` for the table-constant getters).
pub fn custom_getter(ability: u32) -> Option<(u32, u32)> {
    match ability {
        100 => Some(getter_021d88d0()),
        143 => Some(getter_021dc564()),
        _ => None,
    }
}

/// C: `FUN_overlay_d_93__021d8484`: registers `mon`'s ability as a type-4 factor owned by it,
/// ordered by its speed. `None` when the ability has no registry entry or the pool is full.
pub fn register_ability<E: BppEnv, R: AbilityRegistry>(
    sys: &mut EventSystem,
    reg: &R,
    mon: &BattleMon,
    env: &mut E,
) -> Option<FactorId> {
    let ability = mon.get(value::ABILITY_RAW, env) as u16 as u32;
    let (table, word) = reg.getter(ability)?;
    // C: `FUN_overlay_d_93__021d85b0`: speed without stages, as a `u16`.
    let speed = mon.get_unstaged(value::SPEED, env) as u16 as u32;
    let owner = mon.poke_id();
    let (sub_priority, count) = split_getter_word(word);
    sys.add(
        TYPE_ABILITY,
        ability as u16,
        sub_priority,
        speed,
        owner,
        table,
        count as u16,
    )
}

/// C: `FUN_overlay_d_93__021d8510`: removes every type-4 factor owned by `mon`.
pub fn unregister_ability(sys: &mut EventSystem, mon: &BattleMon) {
    let owner = mon.poke_id() as u32;
    while let Some(f) = sys.find(TYPE_ABILITY, owner) {
        sys.remove(f);
    }
}

/// C: `FUN_overlay_d_93__021d8538`: disables `mon`'s type-4 factors.
pub fn disable_ability(sys: &mut EventSystem, mon: &BattleMon) {
    sys.disable(mon.poke_id() as u32, TYPE_ABILITY);
}

/// C: `FUN_overlay_d_93__021d8548`: re-enables `mon`'s type-4 factors, registering its
/// ability if it has none.
pub fn enable_ability<E: BppEnv, R: AbilityRegistry>(
    sys: &mut EventSystem,
    reg: &R,
    mon: &BattleMon,
    env: &mut E,
) {
    if !sys.enable(mon.poke_id() as u32, TYPE_ABILITY) {
        register_ability(sys, reg, mon, env);
    }
}

/// C: `FUN_overlay_d_93__021d8564`: swaps the abilities of two Pokemon: removes one type-4
/// factor of each, then registers both again.
pub fn swap_abilities<E: BppEnv, R: AbilityRegistry>(
    sys: &mut EventSystem,
    reg: &R,
    a: &BattleMon,
    b: &BattleMon,
    env: &mut E,
) {
    let fa = sys.find(TYPE_ABILITY, a.poke_id() as u32);
    let fb = sys.find(TYPE_ABILITY, b.poke_id() as u32);
    if let Some(f) = fa {
        sys.remove(f);
    }
    if let Some(f) = fb {
        sys.remove(f);
    }
    register_ability(sys, reg, a, env);
    register_ability(sys, reg, b, env);
}

#[cfg(test)]
mod tests;
