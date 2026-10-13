//! `btl_client.c` part 5 (0x021D2FD8-0x021D3F04): playback-table command handlers and client getters (B13-spec1).
//!
//! Specs: `tools/rom-diff/svflow_specs/client5.spec` (`nowork`, the client object is `arg buf 0x1C0`, the argument record
//! `arg buf 0x40`). Smoke ports only.

use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021D_3CC8, 0x021D_3EC8];

const OFS_POKECON: usize = 4;

/// C: `FUN_overlay_d_93__021d3cc8`: playback command (BattleMon status block): `mon = 021b9934(pokecon, u8 args[0])`, then
/// `021d6848(mon, &{u16 args[4], u16 args[5], u8 args[3], u8 args[1], u8 args[2], pad})`. Returns 1 ("done").
pub fn relay_mon_block<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let w = |i: usize| super::rd32(args, 4 * i);
    let mon = env.call(
        0x021B_9934,
        &mut [Arg::V(super::rd32(c, OFS_POKECON)), Arg::V(w(0) & 0xFF)],
    );
    let mut block = [0u8; 8];
    block[0..2].copy_from_slice(&(w(4) as u16).to_le_bytes());
    block[2..4].copy_from_slice(&(w(5) as u16).to_le_bytes());
    block[4] = w(3) as u8;
    block[5] = w(1) as u8;
    block[6] = w(2) as u8;
    env.call(0x021D_6848, &mut [Arg::V(mon), Arg::In(&block[..7])]);
    1
}

/// `{item kind, status flag}` pairs of the ROM table 0x021EFFA0 (`021d3ec8`): kind for `021d805c`, flag for `021d62c4`.
const ITEM_STATUS_CURES: [(u8, u8); 7] = [
    (18, 2),
    (19, 5),
    (20, 4),
    (21, 3),
    (22, 1),
    (23, 6),
    (24, 7),
];

/// C: `FUN_overlay_d_93__021d3ec8`: does `item` cure a status `mon` has: the first table pair whose kind the item has
/// (`021d805c(item, kind)`) and whose flag the mon has (`021d62c4(mon, flag)`) answers 1; else 0.
pub fn item_cures_status<E: FlowEnv>(env: &mut E, item: u32, mon: u32) -> u32 {
    for &(kind, flag) in &ITEM_STATUS_CURES {
        if env.call(0x021D_805C, &mut [Arg::V(item), Arg::V(kind as u32)]) != 0
            && env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(flag as u32)]) != 0
        {
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod tests;
