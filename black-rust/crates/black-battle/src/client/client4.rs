//! `btl_client.c` part 4 (0x021D10B8-0x021D2FD8): the request steps 021d10b8 .. 021d1a74 and the first playback-table handlers
//! (B13-spec1 / B13-spec2).
//!
//! Specs: `tools/rom-diff/svflow_specs/client4.spec` (`nowork`, the client object is `arg buf 0x1C0`, a step's state word
//! `arg buf 0x10`, a handler's argument words `arg buf 0x40`). Smoke ports only.

use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[0x021D_2990, 0x021D_299C, 0x021D_2C7C];

const OFS_VIEW: usize = 0x54;

/// The sound id the yes/no message window polls (`021d2c7c`).
const SE_WINDOW: u32 = 0x56B;

/// C: `FUN_overlay_d_93__021d2990`: playback handler (one shot): `021f878c(args[0])` (overlay 94 sound call, not ported); returns 1.
pub fn play_sound<E: FlowEnv>(env: &mut E, args: &[u8]) -> u32 {
    env.call(0x021F_878C, &mut [Arg::V(super::rd32(args, 0))]);
    1
}

/// C: `FUN_overlay_d_93__021d299c`: two-state playback handler: state 0 starts the view effect `021ea8e4(view)` and moves to state 1
/// (0 = not done); state 1 answers `021ea8f4(view) != 0`; any other state is 0.
pub fn view_effect_step<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8]) -> u32 {
    let view = super::rd32(c, OFS_VIEW);
    match super::rd32(state, 0) {
        0 => {
            env.call(0x021E_A8E4, &mut [Arg::V(view)]);
            super::wr32(state, 0, 1);
            0
        }
        1 => (env.call(0x021E_A8F4, &mut [Arg::V(view)]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2c7c`: message-window callback (`021ea638` in 021d2ca8): event 3 plays the window sound
/// (`020061e4(0x56B)`, answer 0), event 5 answers whether it has finished (`02006254(0x56B) != 0`), any other event 0.
pub fn window_sound_event<E: FlowEnv>(env: &mut E, event: u32) -> u32 {
    match event {
        3 => {
            env.call(0x0200_61E4, &mut [Arg::V(SE_WINDOW)]);
            0
        }
        5 => (env.call(0x0200_6254, &mut [Arg::V(SE_WINDOW)]) != 0) as u32,
        _ => 0,
    }
}

#[cfg(test)]
mod tests;
