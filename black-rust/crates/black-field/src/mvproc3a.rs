//! `fldmmdl_acmd.c`, part 3a (overlay 21, `0x0219A098..0x0219A87C`; F1 row 21.09, F4b job 3): the movement-type procs for looking
//! around (move codes 2, 6..0xD, 0x2D, 0x2E), the random walk (3, 4, 5, 0x43), fixed facing (0xE..0x11) and spinning (0x12, 0x13,
//! 0x31).
//!
//! F4b-spec3 specced all 43 functions (`tools/rom-diff/specs/black-field/mvproc3a.spec`; the header has the proc contract and the
//! conventions shared by `mvproc3a`..`mvproc3d`); this module holds the smoke ports and the callee addresses the four modules share.
//! Every callee outside the area is reached through [`FieldEnv::call`] by address; in-area callees that are ported are Rust calls.

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_A098,
    0x0219_A0CC,
    0x0219_A1E8,
    0x0219_A528,
    0x0219_A54C,
    0x0219_A7C4,
];

/// `FUN_0216D684(m, n)`: clear `n` bytes of the move work (`m + 0x94`) and return it (`fldmmdl::move_work_init`).
pub(crate) const MOVE_WORK_INIT: u32 = 0x0216_D684;
/// `FUN_0216D69C(m)`: the move work (`m + 0x94`).
pub(crate) const MOVE_WORK: u32 = 0x0216_D69C;
/// `FUN_0216D6A0(m, n)`: clear `n` bytes of the sub work (`m + 0xA4`) and return it.
pub(crate) const SUB_WORK_INIT: u32 = 0x0216_D6A0;
/// `FUN_0216D674(m, v)`: draw animation code (`u16 +0x2A`) = v.
pub(crate) const SET_ANIM: u32 = 0x0216_D674;
/// `FUN_0216DA50(m)`: move bit 0x10 (walking) off.
pub(crate) const WALK_OFF: u32 = 0x0216_DA50;
/// `FUN_02163F10(m)`: old grid = grid.
pub(crate) const SAVE_OLD_GRID: u32 = 0x0216_3F10;
/// `FUN_0216D5E0(m, dir)`: display dir = dir unless the dir is locked.
pub(crate) const SET_DIR_DISP: u32 = 0x0216_D5E0;
/// `FUN_0216D5A8(m)`: the event type (`u16 +0x10`).
pub(crate) const EVENT_TYPE: u32 = 0x0216_D5A8;
/// `FUN_0216D63C(m, i)`: header param `i`.
pub(crate) const PARAM: u32 = 0x0216_D63C;
/// `FUN_0219B0F0(table, term)`: a random entry of a ROM table (mvproc3b).
const RANDOM_ENTRY: u32 = 0x0219_B0F0;
/// ROM `0x021D4FA4`: the look-around waits {0x10, 0x20, 0x30, 0x40}, ended by -1.
const WAITS: u32 = 0x021D_4FA4;

/// C: `FUN_0219A098` (unlabeled routine at `0x0219A098`): init of the look-around types: move work (8 bytes cleared): `s16 +2` = a
/// random wait of [`WAITS`], `u32 +4` = `kind` (the 0219B120 direction-set id); draw animation code 0; move bit 0x10 off.
pub fn look_init<E: FieldEnv>(env: &mut E, m: u32, kind: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 8]);
    let t = env.callv(RANDOM_ENTRY, &[WAITS, 0xFFFF_FFFF]);
    env.w16(w.wrapping_add(2), t as u16);
    env.callv(SET_ANIM, &[m, 0]);
    env.w32(w.wrapping_add(4), kind);
    env.callv(WALK_OFF, &[m]);
}

/// C: `FUN_0219A0CC` (unlabeled routine at `0x0219A0CC`): move code 2 init (thunk): [`look_init`] with direction set 0 (all four).
pub fn look_init_all<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 0);
}

/// ROM `0x021D5074`: per move code (4..0xD) the turn order of the look-around ([`next_dir`]): u16 code, then dirs ended by 9.
pub const TURN_ORDERS: [[u16; 6]; 10] = [
    [4, 0, 1, 9, 0, 0],
    [5, 2, 3, 9, 0, 0],
    [6, 0, 2, 9, 0, 0],
    [7, 0, 3, 9, 0, 0],
    [8, 1, 2, 9, 0, 0],
    [9, 1, 3, 9, 0, 0],
    [0xA, 0, 1, 2, 9, 0],
    [0xB, 0, 3, 1, 9, 0],
    [0xC, 0, 3, 2, 9, 0],
    [0xD, 1, 2, 3, 9, 0],
];

/// C: `FUN_0219A1E8` (unlabeled routine at `0x0219A1E8`): the dir after `dir` in the [`TURN_ORDERS`] row of move code `code` (the
/// first dir again after the last one); 9 when `dir` is not in the row or the code has none (whole-register compares).
pub fn next_dir(code: u32, dir: u32) -> u32 {
    let Some(row) = TURN_ORDERS.iter().find(|r| u32::from(r[0]) == code) else {
        return 9;
    };
    let mut i = 1;
    while row[i] != 9 {
        if u32::from(row[i]) == dir {
            i += 1;
            if row[i] == 9 {
                i = 1;
            }
            return u32::from(row[i]);
        }
        i += 1;
    }
    9
}

/// C: `FUN_0219A528` (unlabeled routine at `0x0219A528`): init of the fixed-facing codes 0xE..0x11: move work (8 bytes cleared)
/// `u32 +0` = `dir`; draw animation code 0; move bit 0x10 off; old grid = grid.
pub fn face_init<E: FieldEnv>(env: &mut E, m: u32, dir: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 8]);
    env.w32(w, dir);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(WALK_OFF, &[m]);
    env.callv(SAVE_OLD_GRID, &[m]);
}

/// C: `FUN_0219A54C` (unlabeled routine at `0x0219A54C`): move proc of codes 0xE..0x11: once (`u32 +4` == 0) display dir = the
/// stored dir, then `+4` = 1.
pub fn face_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    let n = env.r32(w.wrapping_add(4));
    if n != 0 {
        return;
    }
    let d = env.r32(w) & 0xFFFF;
    env.callv(SET_DIR_DISP, &[m, d]);
    env.w32(w.wrapping_add(4), n.wrapping_add(1));
}

/// C: `FUN_0219A7C4` (unlabeled routine at `0x0219A7C4`): state 2 of move code 0x31: count `u32 +4` up; below 0x18 returns 0, else
/// state (`u8 +2`) 3, returns 1.
pub fn spin_wait<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let n = env.r32(w.wrapping_add(4)).wrapping_add(1);
    env.w32(w.wrapping_add(4), n);
    if (n as i32) < 0x18 {
        return 0;
    }
    env.w8(w.wrapping_add(2), 3);
    1
}

#[cfg(test)]
mod tests;
