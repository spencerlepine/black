//! `btl_client.c` part 2 (0x021CDDDC-0x021CF5B0): request step functions and their helpers (B13-spec1).
//!
//! Specs: `tools/rom-diff/svflow_specs/client2.spec` (`nowork`, the client object is `arg buf 0x1C0`). Smoke ports only.

use crate::calc::u32_div;
use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021C_DEE8];

const OFS_HP_REF: usize = 0x19C;

/// C: `FUN_overlay_d_93__021cdee8`: how far the active mon's HP is below the reference HP (`c.u16[0x19C]`), in percent of the
/// current HP: 0 without an active mon (`021cdf1c`, not ported) or when HP >= reference, else `100 * (ref - HP) / HP`
/// (`BattleMon_Get(mon, 0xD)`; division by zero answers the dividend like the original runtime routine).
pub fn hp_drop_percent<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    let mon = env.call(0x021C_DF1C, &mut [Arg::P]);
    if mon == 0 {
        return 0;
    }
    let hp = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)]);
    let reference = super::rd16(c, OFS_HP_REF) as u32;
    if hp >= reference {
        return 0;
    }
    u32_div(100u32.wrapping_mul(reference - hp), hp)
}

#[cfg(test)]
mod tests;
