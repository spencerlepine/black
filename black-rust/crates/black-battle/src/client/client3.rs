//! `btl_client.c` part 3 (0x021CF5B0-0x021D10B8): selection checks and party/target helpers (B13-spec1).
//!
//! Specs: `tools/rom-diff/svflow_specs/client3.spec` (`nowork`, the client object is `arg buf 0x1C0`). Smoke ports only.

use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021C_FBBC, 0x021D_0B10];

const OFS_PARTY: usize = 0x118;
const OFS_PARTY_COUNT: usize = 0x11C;

const OFS_SLOTS: usize = 0x10C;
const SLOT_COUNT: usize = 6;

/// C: `FUN_overlay_d_93__021cfbbc`: is `id` one of the six signed bytes at `+0x10C` (0xFF = empty slot, i.e. -1).
pub fn slot_holds(c: &[u8], id: i32) -> bool {
    (0..SLOT_COUNT).any(|i| c[OFS_SLOTS + i] as i8 as i32 == id)
}

/// C: `FUN_overlay_d_93__021d0b10`: sets up the party-list view block `block` through three Thumb-to-ARM veneers (0x0689xxxx code,
/// not ported): `021d0b3c(block, c+0x118 party, b, a)`, `021d0b4c(block, c[0x11C] count)`, `021d0b5c(extra, block)` (`extra` is the
/// first stack argument).
pub fn setup_party_block<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    a: u32,
    b: u32,
    block: u32,
    extra: u32,
) {
    let party = super::rd32(c, OFS_PARTY);
    env.call(
        0x021D_0B3C,
        &mut [Arg::V(block), Arg::V(party), Arg::V(b), Arg::V(a)],
    );
    env.call(
        0x021D_0B4C,
        &mut [Arg::V(block), Arg::V(c[OFS_PARTY_COUNT] as u32)],
    );
    env.call(0x021D_0B5C, &mut [Arg::V(extra), Arg::V(block)]);
}

#[cfg(test)]
mod tests;
