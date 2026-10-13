//! `fldmmdl_acmd.c`, part 3b (overlay 21, `0x0219A87C..0x0219B41C`; F4b job 3): the back-and-forth walk (move code 0x14), the
//! 4-leg routes (0x15..0x24), the reversible routes (0x25..0x2C) and the helpers the movement types share (pad bits, random entries of
//! the dir tables, the dir toward the player, the sight counter, event-type predicates, the spin-look while walking).
//!
//! F4b-spec3 specced all 55 functions (`tools/rom-diff/specs/black-field/mvproc3b.spec`); this module holds the smoke ports.

use crate::hooks::FieldEnv;
use crate::mvproc3a::{EVENT_TYPE, MOVE_WORK_INIT};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_AA38,
    0x0219_AA68,
    0x0219_B334,
    0x0219_B358,
    0x0219_B374,
    0x0219_B404,
];

/// `FUN_0216D550(m, v)`: status &= !v.
const CLEAR_STATUS: u32 = 0x0216_D550;
/// Status bit 8: the display dir is locked.
const DIR_LOCK: u32 = 8;

/// C: `FUN_0219AA38` (unlabeled routine at `0x0219AA38`): init of the route codes 0x15..0x24: move work (0xC bytes cleared): `u8 +2`
/// = `turn` (the leg at which the walker looks for the start line), `+3` = `axis` (0 x, 1 z), `+4` = `route` (0219B120 id); for the
/// spin-look event types ([`spins_while_walking`]) [`spin_order`] at `+8`.
pub fn route_init<E: FieldEnv>(env: &mut E, m: u32, turn: u32, axis: u32, route: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 0xC]);
    env.w8(w.wrapping_add(2), turn as u8);
    env.w8(w.wrapping_add(3), axis as u8);
    env.w8(w.wrapping_add(4), route as u8);
    if spins_while_walking(env, m) == 1 {
        spin_order(env, m, w.wrapping_add(8));
    }
}

/// C: `FUN_0219AA68` (unlabeled routine at `0x0219AA68`): move code 0x15 init: [`route_init`]`(m, 2, 0, 0xE)`.
pub fn route_init_15<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0xE);
}

/// C: `FUN_0219B334` (unlabeled routine at `0x0219B334`): 1 for event type 0xD or 0xF..0x13 (`(u16)(type - 0xF) <= 4`), else 0.
pub fn walks_fast_near<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let t = env.callv(EVENT_TYPE, &[m]);
    if t == 0xD || (t.wrapping_add(0xFFF1) & 0xFFFF) <= 4 {
        return 1;
    }
    0
}

/// C: `FUN_0219B358` (unlabeled routine at `0x0219B358`): 1 for event type 7, 8, 0x11 or 0x12 (the types that spin-look while they
/// walk), else 0.
pub fn spins_while_walking<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let t = env.callv(EVENT_TYPE, &[m]);
    u32::from(t == 7 || t == 8 || t.wrapping_sub(0x11) <= 1)
}

/// C: `FUN_0219B374` (unlabeled routine at `0x0219B374`): the spin order of the spin-look state at `p`: `u8 p[1]` = 0 for event type
/// 7 or 0x11, else 1.
pub fn spin_order<E: FieldEnv>(env: &mut E, m: u32, p: u32) {
    let t = env.callv(EVENT_TYPE, &[m]);
    env.w8(p.wrapping_add(1), u8::from(!(t == 7 || t == 0x11)));
}

/// C: `FUN_0219B404` (unlabeled routine at `0x0219B404`): end of a spin-look: unless `u8 p[2]` says the dir was locked before, unlock
/// it (status &= !8).
pub fn spin_unlock<E: FieldEnv>(env: &mut E, m: u32, p: u32) {
    if env.r8(p.wrapping_add(2)) == 0 {
        env.callv(CLEAR_STATUS, &[m, DIR_LOCK]);
    }
}

#[cfg(test)]
mod tests;
