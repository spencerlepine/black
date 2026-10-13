//! `battle_server.s` part 2 (0x021B75A8-0x021B902C): sub-proc steps, per-frame loops, main-module getters, position maps (B11).
//!
//! Specs: `tools/rom-diff/svflow_specs/server2.spec` (`nowork`; M is `arg buf 0x1000` with the BATTLE_SETUP at arena 0x480).
//! Smoke ports only (B11-spec); the B11 port job ports the rest.

use super::rd32;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021B_8564, 0x021B_8670];

/// M: the 4 client object pointers.
const M_CLIENTS: usize = 0x10;
/// M: u8 per-client "done" flags.
const M_CLIENT_DONE: usize = 0xC4;
/// BATTLE_SETUP: the rule word (0..3).
const SETUP_RULE: usize = 4;

/// C: `FUN_overlay_d_93__021b8564`: 0 if some client slot `i` (0..3) has a client object (`M[0x10 + 4 i] != 0`) whose done
/// flag `M[0xC4 + i]` is still 0, else 1.
pub fn all_clients_done(m: &[u8]) -> u32 {
    let pending = (0..4).any(|i| rd32(m, M_CLIENTS + 4 * i) != 0 && m[M_CLIENT_DONE + i] == 0);
    u32::from(!pending)
}

/// C: `sub_021b8670`: a per-rule count from the rule word of the BATTLE_SETUP (`*M`): rule 0 -> 1, 1 -> 2, 2 -> 3, 3 -> 1,
/// anything else 1 (a jump table in the original). Probably the number of battle positions per side (single / double /
/// triple / rotation), UNVERIFIED meaning.
pub fn rule_positions_per_side(setup: &[u8]) -> u32 {
    match rd32(setup, SETUP_RULE) {
        1 => 2,
        2 => 3,
        _ => 1,
    }
}

#[cfg(test)]
mod tests;
