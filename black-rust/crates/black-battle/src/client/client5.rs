//! `btl_client.c` part 5 (0x021D2FD8-0x021D3F04): playback-table command handlers and client getters (B13-spec1).
//!
//! Specs: `tools/rom-diff/svflow_specs/client5.spec` (`nowork`, the client object is `arg buf 0x1C0`, the argument record
//! `arg buf 0x40`). Smoke ports only.

use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021D_3CC8];

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

#[cfg(test)]
mod tests;
