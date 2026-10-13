//! `battle_server.s` part 3 (0x021B902C-0x021BAA94): main-module helpers, pokecon and party container, player records,
//! battle-end results, teardown and the `btl_server.c` create 021ba9c8 (B11).
//!
//! Specs: `tools/rom-diff/svflow_specs/server3.spec` (`nowork`). Smoke ports only (B11-spec); the B11 port job ports the rest.

use super::{rd32, wr32};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021B_9C24, 0x021B_A710];

/// Party container: offset of the u8 mon count (after the 6 BattleMon pointers).
const PARTY_COUNT: usize = 0x18;

/// C: `FUN_overlay_d_93__021b9c24`: moves the mon pointer of slot `i` of a party container to slot `count - 1`, shifting the
/// slots after it down by one. The loop compares signed (`count - 1` is -1 for an empty party: nothing moves, slot `i` is
/// written back unchanged) and keeps the index as a byte.
pub fn party_move_to_end(party: &mut [u8], i: u32) {
    let mut i = i;
    let moved = rd32(party, (i * 4) as usize);
    let last = i32::from(party[PARTY_COUNT]) - 1;
    while (i as i32) < last {
        let next = rd32(party, (i * 4 + 4) as usize);
        wr32(party, (i * 4) as usize, next);
        i = (i + 1) & 0xFF;
    }
    wr32(party, (i * 4) as usize, moved);
}

/// C: `FUN_overlay_d_93__021ba710`: relation of client `b` to client `a`: 0 the same client, 2 the same side (equal bit 0),
/// otherwise 1 when `b <= 1` and 3 when `b > 1`.
pub fn client_relation(a: u32, b: u32) -> u8 {
    if a == b {
        0
    } else if a & 1 == b & 1 {
        2
    } else if b <= 1 {
        1
    } else {
        3
    }
}

#[cfg(test)]
mod tests;
