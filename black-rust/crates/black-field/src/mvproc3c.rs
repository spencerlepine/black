//! `fldmmdl_acmd.c`, part 3c (overlay 21, `0x0219B41C..0x0219BDC4`; F4b job 3): the event-type procs (init / busy / spin by MMDL
//! event type, called from mmdl_move), following the player (move code 0x30) and walking behind the partner trainer (0x32).
//!
//! F4b-spec3 specced all 38 functions (`tools/rom-diff/specs/black-field/mvproc3c.spec`); this module holds the smoke ports.

use crate::hooks::FieldEnv;
use crate::mvproc3a::{PARAM, SUB_WORK_INIT};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_B41C,
    0x0219_B494,
    0x0219_B4C8,
    0x0219_B508,
    0x0219_B9B0,
    0x0219_BA2C,
];

/// `FUN_0219B45C(m)`: the event-type busy query (ROM table 0x021D52CC by event type).
const EV_BUSY: u32 = 0x0219_B45C;
/// `FUN_0219B478(m)`: the event-type spin query (ROM table 0x021D5320 by event type).
const EV_SPIN: u32 = 0x0219_B478;
/// Grid x / z (`FUN_0216D83C` / `FUN_0216D86C`) and old grid x / z (`FUN_0216D804` / `FUN_0216D81C`).
const GRID_X: u32 = 0x0216_D83C;
const GRID_Z: u32 = 0x0216_D86C;
const OLD_X: u32 = 0x0216_D804;
const OLD_Z: u32 = 0x0216_D81C;
/// `FUN_0216D67C(m)`: the MMDLSYS of an MMDL.
const SYS_OF: u32 = 0x0216_D67C;
/// `FUN_0216DEE0(sys)`: the player MMDL (0 = none).
const FIND_PLAYER: u32 = 0x0216_DEE0;
/// `FUN_0216D7C4(m)`: the acmd code (`u16 +0x26`).
const ACMD_CODE: u32 = 0x0216_D7C4;

/// C: `FUN_0219B41C` (unlabeled routine at `0x0219B41C`): mmdl_move's "event-type proc busy" query: 1 when both the busy query
/// (`0219B45C`) and the spin query (`0219B478`) answer nonzero, else 0.
pub fn ev_busy<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if env.callv(EV_BUSY, &[m]) == 0 {
        return 0;
    }
    u32::from(env.callv(EV_SPIN, &[m]) != 0)
}

/// C: `FUN_0219B494` (unlabeled routine at `0x0219B494`): 1 when the grid x or z differs from the old grid (the MMDL moved), else 0.
pub fn moved<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let x = env.callv(GRID_X, &[m]);
    if x != env.callv(OLD_X, &[m]) {
        return 1;
    }
    let z = env.callv(GRID_Z, &[m]);
    u32::from(z != env.callv(OLD_Z, &[m]))
}

/// C: `FUN_0219B4C8` (unlabeled routine at `0x0219B4C8`): 1 when grid x and z equal the old grid (standing), else 0.
pub fn standing<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let x = env.callv(GRID_X, &[m]);
    if x != env.callv(OLD_X, &[m]) {
        return 0;
    }
    let z = env.callv(GRID_Z, &[m]);
    u32::from(z == env.callv(OLD_Z, &[m]))
}

/// C: `FUN_0219B508` (unlabeled routine at `0x0219B508`): event-type init of types 4 / 0xE: sub work (9 bytes cleared) `u8 +3` =
/// header param 1.
pub fn ev_steps_init<E: FieldEnv>(env: &mut E, m: u32) {
    let s = env.callv(SUB_WORK_INIT, &[m, 9]);
    let p = env.callv(PARAM, &[m, 1]);
    env.w8(s.wrapping_add(3), p as u8);
}

/// C: `FUN_0219B9B0` (unlabeled routine at `0x0219B9B0`): start following the player: `u8 w[1]` = 1, `s16 w[2]` / `w[4]` = his grid
/// x / z, `u16 w[6]` = 0xFF.
pub fn follow_start<E: FieldEnv>(env: &mut E, m: u32, w: u32) {
    let sys = env.callv(SYS_OF, &[m]);
    let p = env.callv(FIND_PLAYER, &[sys]);
    env.w8(w.wrapping_add(1), 1);
    let x = env.callv(GRID_X, &[p]);
    env.w16(w.wrapping_add(2), x as u16);
    let z = env.callv(GRID_Z, &[p]);
    env.w16(w.wrapping_add(4), z as u16);
    env.w16(w.wrapping_add(6), 0xFF);
}

/// C: `FUN_0219BA2C` (unlabeled routine at `0x0219BA2C`): the acmd base a follower walks with: its own acmd code, 0x58..0x5B mapped
/// to 0x10..0x13 (the player lookup it makes first is unused).
pub fn follower_base<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    env.callv(FIND_PLAYER, &[sys]);
    let c = env.callv(ACMD_CODE, &[m]);
    match c.wrapping_sub(0x58) {
        k @ 0..=3 => 0x10 + k,
        _ => c,
    }
}

#[cfg(test)]
mod tests;
