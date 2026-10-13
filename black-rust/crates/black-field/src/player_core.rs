//! `field_player_core.c` (overlay 21, `0x021A36E4..0x021A4140`): PLAYER CORE, the 0x34-byte state of the player
//! MMDL shared by the grid and no-grid players: the move form (walk / cycle / surf, saved in the PLAYER record of the
//! GAMEDATA), the OBJ code changes that go with it, the pad -> direction mapping and the request state machine
//! (`+0x24` request bits, procs in ROM `0x021D89C4`).
//!
//! F3a-spec specced all 64 functions (`tools/rom-diff/specs/black-field/player_core.spec`, header: the CORE layout,
//! requests and forms); this module holds the smoke ports. CORE: `+4` GAMESYS, `+8` FIELDMAP, `+0xC` PLAYER record,
//! `+0x10` move state, `+0x14` move value, `+0x18` sex (form-table row), `+0x1C` the player MMDL, `+0x20` / `+0x22`
//! last pressed x / z direction, `+0x24` request bits (bit i = request i, 0..9), `+0x28` surf effect task, `+0x2C` /
//! `+0x2E` counters, `+0x30 s16` request step, `+0x32` request done.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_3C24,
    0x021A_3C3C,
    0x021A_3C54,
    0x021A_3C94,
    0x021A_3CB0,
];

/// No direction (the pad helpers' answer when no key of the axis is held).
pub const DIR_NONE: u32 = 9;

/// C: `FUN_021A3C24` (unlabeled routine at `0x021A3C24`): remember the pressed directions: `+0x20` = the x direction,
/// `+0x22` = the z direction of `keys` (u16 stores).
pub fn remember_keys(mem: &mut Mem, core: u32, keys: u32) {
    mem.w16(core.wrapping_add(0x20), key_dir_x(keys) as u16);
    mem.w16(core.wrapping_add(0x22), key_dir_z(keys) as u16);
}

/// C: `FUN_021A3C3C` (unlabeled routine at `0x021A3C3C`): the x direction of the pad bits: 0x20 (left) -> 2,
/// 0x10 (right) -> 3, else [`DIR_NONE`]; left wins.
pub fn key_dir_x(keys: u32) -> u32 {
    if keys & 0x20 != 0 {
        2
    } else if keys & 0x10 != 0 {
        3
    } else {
        DIR_NONE
    }
}

/// C: `FUN_021A3C54` (unlabeled routine at `0x021A3C54`): the z direction of the pad bits: 0x40 (up) -> 0,
/// 0x80 (down) -> 1, else [`DIR_NONE`]; up wins.
pub fn key_dir_z(keys: u32) -> u32 {
    if keys & 0x40 != 0 {
        0
    } else if keys & 0x80 != 0 {
        1
    } else {
        DIR_NONE
    }
}

/// C: `FUN_021A3C94` (unlabeled routine at `0x021A3C94`): the index of the lowest set bit among bits 0..9 of the
/// request bits, 10 when none is set.
pub fn first_request(bits: u32) -> u32 {
    (0..10).find(|&i| bits >> i & 1 != 0).unwrap_or(10)
}

/// C: `FUN_021A3CB0` (unlabeled routine at `0x021A3CB0`): start a request set: `+0x24` = bits; the step `+0x30`,
/// done `+0x32` and counters `+0x2C` / `+0x2E` = 0.
pub fn set_requests(mem: &mut Mem, core: u32, bits: u32) {
    mem.w32(core.wrapping_add(0x24), bits);
    for o in [0x30, 0x32, 0x2C, 0x2E] {
        mem.w16(core.wrapping_add(o), 0);
    }
}

#[cfg(test)]
mod tests;
