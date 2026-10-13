//! `battle_server.s` part 1 (0x021B6100-0x021B75A8): BattleProc init / main / exit, sub-proc selection, setup sub-procs (B11).
//!
//! Specs: `tools/rom-diff/svflow_specs/server1.spec` (`nowork`; M is `arg buf 0x1000`). Smoke ports only (B11-spec); the
//! B11 port job ports the rest.

use super::{rd32, wr32};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021B_6804, 0x021B_6818];

/// Thumb address of the exit sub-proc `021b6c08` (frees the battle objects).
const EXIT_SUBPROC: u32 = 0x021B_6C09;

/// C: `FUN_overlay_d_93__021b6804`: puts the exit sub-proc into a sub-proc slot `{+0 fn, +4 fn 2, +8 arg, +0xC state}`:
/// `{0x021B6C09, 0, arg, 0}`.
pub fn set_exit_subproc(slot: &mut [u8], arg: u32) {
    wr32(slot, 8, arg);
    wr32(slot, 0, EXIT_SUBPROC);
    wr32(slot, 4, 0);
    wr32(slot, 0xC, 0);
}

/// C: `FUN_overlay_d_93__021b6818`: 1 if the battle type word at `+0` of a BATTLE_SETUP is 3, else 0 (a 4-case jump table
/// in the original; types above 3 take the default 0).
pub fn setup_type_is_3(setup: &[u8]) -> u32 {
    u32::from(rd32(setup, 0) == 3)
}

#[cfg(test)]
mod tests;
