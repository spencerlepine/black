//! `fldmmdl_acmd.c`, part 2c (overlay 21, `0x02198960..0x02198E44`; F4b job 2): waits (02198960, update 02198978), rise / fall
//! (codes 0x43 / 0x44), the status toggles 0x45..0x4A, the balloon marks (02198B4C, update 02198B80) and the walks with a per-frame
//! speed table (02198C08, update 02198C4C).
//!
//! F4b-spec2 specced all 53 functions (`tools/rom-diff/specs/black-field/acmd2c.spec`); this module holds the smoke ports.

use crate::acmd2a::{
    ACMD_WORK, ACMD_WORK_INIT, INC_SEQ, POS_HEIGHT, POS_MOVE, SAVE_OLD_GRID, SET_ANIM,
    SET_DRAW_OFFSET, SET_MOVE_BITS, WALK_DONE,
};
use crate::acmd2b::vec_bytes;
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_8960,
    0x0219_8978,
    0x0219_89BC,
    0x0219_8A8C,
    0x0219_8C4C,
    0x0219_8D7C,
];

/// The ROM speed tables of the speed-table walks (fx32 per frame), by address: codes 0x4C..0x4F / 0xA3..0xA6 (6 frames),
/// 0x50..0x53 (3), 0x60..0x63 (7), 0xA7..0xAA (12).
pub const SPEED_TABLES: [(u32, &[u32]); 4] = [
    (
        0x021D_4A68,
        &[0x2000, 0x3000, 0x3000, 0x2000, 0x3000, 0x3000],
    ),
    (0x021D_4A44, &[0x5000, 0x6000, 0x5000]),
    (
        0x021D_4A80,
        &[0x2500, 0x2480, 0x2480, 0x2480, 0x2480, 0x2480, 0x2480],
    ),
    (
        0x021D_4A9C,
        &[
            0x1000, 0x1000, 0x2000, 0x1000, 0x1000, 0x2000, 0x1000, 0x1000, 0x2000, 0x1000, 0x1000,
            0x2000,
        ],
    ),
];

/// C: `FUN_02198960` (unlabeled routine at `0x02198960`): start a wait of `frames` frames (acmd work `u32 +0`); acmd seq + 1.
pub fn wait_start<E: FieldEnv>(env: &mut E, m: u32, frames: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.w32(w, frames);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02198978` (unlabeled routine at `0x02198978`): wait update: frames left -> one less, returns 0; none -> acmd seq + 1,
/// returns 1.
pub fn wait_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let n = env.r32(w);
    if n != 0 {
        env.w32(w, n.wrapping_sub(1));
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_021989BC` (unlabeled routine at `0x021989BC`): acmd code 0x3F (wait 8 frames): [`wait_start`]`(m, 8)`; returns 1.
pub fn wait_8<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 8);
    1
}

/// C: `FUN_02198A8C` (unlabeled routine at `0x02198A8C`): fall update: height += speed, clamped at 0 from below; draw offset =
/// (0, height, 0); still above 0 -> 0; else acmd seq + 1, returns 1.
pub fn fall_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut h = env.r32(w).wrapping_add(env.r32(w.wrapping_add(4)));
    if (h as i32) < 0 {
        h = 0;
    }
    env.w32(w, h);
    let v = vec_bytes(0, h, 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    if (h as i32) > 0 {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198C4C` (unlabeled routine at `0x02198C4C`): speed-table walk update: position += `table[index]` in the u16 dir
/// (+0), map height; index (s16 +6) + 1; below the frame count (s16 +4) -> 0; else move bits 0x440, old grid = grid, draw
/// animation code 0, acmd seq + 1, returns 1. `table` is the ROM address of one of [`SPEED_TABLES`]. UNVERIFIED: another
/// address, or an index past the table, panics (the original reads whatever follows).
pub fn table_walk_update<E: FieldEnv>(env: &mut E, m: u32, table: u32) -> u32 {
    let t = SPEED_TABLES
        .iter()
        .find(|t| t.0 == table)
        .map(|t| t.1)
        .unwrap_or_else(|| panic!("02198C4C: unknown speed table {table:#010x}"));
    let w = env.callv(ACMD_WORK, &[m]);
    let i = env.r16(w.wrapping_add(6)) as i16;
    let dir = u32::from(env.r16(w));
    env.callv(POS_MOVE, &[m, dir, t[i as usize]]);
    env.callv(POS_HEIGHT, &[m]);
    let i = i.wrapping_add(1);
    env.w16(w.wrapping_add(6), i as u16);
    if i < env.r16(w.wrapping_add(4)) as i16 {
        return 0;
    }
    env.callv(SET_MOVE_BITS, &[m, WALK_DONE]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198D7C` (unlabeled routine at `0x02198D7C`): update of acmd codes 0x50..0x53: [`table_walk_update`] with the 3-frame
/// table `0x021D4A44`; 1 when it returned 1, else 0.
pub fn table_walk_update_3<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    u32::from(table_walk_update(env, m, 0x021D_4A44) == 1)
}

#[cfg(test)]
mod tests;
