//! Flow API used by the event handlers (0x021C7CBC-0x021C8810; B10c replaces the `FlowHost`
//! methods with these). Specs: `tools/rom-diff/svflow_specs/api.spec` (69 functions; the overlay-95
//! veneers and the five tail calls into overlay 95 that cannot run are listed in its header).
//!
//! B10bc-spec smoke ports: a getter (`7cbc`), the pokeID -> BattleMon lookup (`7cf4`), the
//! iterator-based id list (`7d54`), the action-record search (`804c`) and the 6-entry list append
//! (`831c`). Callees outside this area go through [`FlowEnv`].

use super::env::{Arg, FlowEnv};
use super::SvFlowWork;

/// Work offset of the pokecon pointer handed to the BattleMon lookups.
const OFS_POKECON: usize = 8;

/// C: `FUN_overlay_d_93__021c7cbc`: the u16 at `work + 0x1F7C`.
pub fn word_1f7c(w: &SvFlowWork) -> u16 {
    w.u16_at(0x1F7C)
}

/// C: `FUN_overlay_d_93__021c7cf4`: the BattleMon of `poke_id` (tail call
/// `021b9934(work[8], poke_id)`).
pub fn mon_by_id<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32) -> u32 {
    env.call(
        0x021B_9934,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(poke_id)],
    )
}

/// C: `FUN_021c7d54`: walks the mons of the work (core2 iterator `021bd2b4` / `021bd34c`) and
/// writes the pokeID of every mon that is not on the side of `poke_id` (`021b9a24`) to `out`;
/// returns how many (u8 count).
pub fn list_foes<E: FlowEnv>(env: &mut E, poke_id: u32, out: &mut [u8]) -> u32 {
    env.call(0x021B_D2B4, &mut [Arg::P, Arg::P]);
    let mut n = 0u32;
    loop {
        let mut m = [0u8; 4];
        if env.call(0x021B_D34C, &mut [Arg::P, Arg::P, Arg::Out(&mut m)]) == 0 {
            break;
        }
        let mon = u32::from_le_bytes(m);
        let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
        if env.call(0x021B_9A24, &mut [Arg::V(poke_id), Arg::V(id)]) == 0 {
            let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
            out[n as usize] = id as u8;
            n = (n + 1) & 0xFF;
        }
    }
    n
}

/// C: `FUN_overlay_d_93__021c804c`: searches the `work[0x782]` action records (0x10 bytes each,
/// BattleMon at +0x7E0, action word at +0x7E4) for one of `poke_id` whose kind (`word & 0xF`) is
/// not 6; writes its word to `out` and returns 1, or returns 0.
pub fn find_action<E: FlowEnv>(env: &mut E, w: &SvFlowWork, poke_id: u32, out: &mut [u8]) -> u32 {
    let mut i = 0usize;
    while i < w.u8_at(0x782) as usize {
        let mon = w.u32_at(0x7E0 + 16 * i);
        if poke_id == env.call(0x021D_5440, &mut [Arg::V(mon)]) {
            let word = w.u32_at(0x7E4 + 16 * i);
            if word & 0xF != 6 {
                out[0..4].copy_from_slice(&word.to_le_bytes());
                return 1;
            }
        }
        i += 1;
    }
    0
}

/// C: unlabeled routine at `0x021C831C`: appends the byte `v` to the 6-entry list at
/// `work + 0x78B` (count `work[0x788]`); no-op when it is full.
pub fn list6_push(w: &mut SvFlowWork, v: u32) {
    let n = w.u8_at(0x788);
    if n < 6 {
        w.set_u8_at(0x788, n + 1);
        w.set_u8_at(0x78B + n as usize, v as u8);
    }
}

/// Addresses of ported functions; stubs at these addresses become inactive.
pub const PORTED: &[u32] = &[0x021C7CBC, 0x021C7CF4, 0x021C7D54, 0x021C804C, 0x021C831C];

#[cfg(test)]
mod tests;
