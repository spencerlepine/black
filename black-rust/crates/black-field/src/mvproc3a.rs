//! `fldmmdl_acmd.c`, part 3a (overlay 21, `0x0219A098..0x0219A87C`; F1 row 21.09, F4b job 3): the movement-type procs for looking
//! around (move codes 2, 6..0xD, 0x2D, 0x2E), the random walk (3, 4, 5, 0x43), fixed facing (0xE..0x11) and spinning (0x12, 0x13,
//! 0x31).
//!
//! F4b-spec3 specced all 43 functions (`tools/rom-diff/specs/black-field/mvproc3a.spec`; the header has the proc contract and the
//! conventions shared by `mvproc3a`..`mvproc3d`); F4b-port2 ported all of them. This module also holds the callee addresses the four
//! modules share. Every callee outside the area (also the other `mvproc3*` areas, whose specs stub each other `force`) is reached
//! through [`FieldEnv::call`] by address; in-area callees are Rust calls. The state procs are reached through ROM tables of code
//! addresses ([`SPIN_STATES`], [`SPIN2_STATES`]); the ports dispatch on the state byte directly (a state past the table panics; the
//! original jumps through whatever follows the table, UNVERIFIED).

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_A098,
    0x0219_A0CC,
    0x0219_A0D8,
    0x0219_A0E4,
    0x0219_A0F0,
    0x0219_A0FC,
    0x0219_A108,
    0x0219_A114,
    0x0219_A120,
    0x0219_A12C,
    0x0219_A138,
    0x0219_A144,
    0x0219_A150,
    0x0219_A1E8,
    0x0219_A24C,
    0x0219_A250,
    0x0219_A260,
    0x0219_A270,
    0x0219_A280,
    0x0219_A290,
    0x0219_A2B8,
    0x0219_A3D8,
    0x0219_A4CC,
    0x0219_A528,
    0x0219_A54C,
    0x0219_A574,
    0x0219_A580,
    0x0219_A58C,
    0x0219_A598,
    0x0219_A5A4,
    0x0219_A5C8,
    0x0219_A5D4,
    0x0219_A5E0,
    0x0219_A604,
    0x0219_A638,
    0x0219_A660,
    0x0219_A6CC,
    0x0219_A750,
    0x0219_A75C,
    0x0219_A780,
    0x0219_A7A8,
    0x0219_A7C4,
    0x0219_A7DC,
];

/// `FUN_0216D684(m, n)`: clear `n` bytes of the move work (`m + 0x94`) and return it (`fldmmdl::move_work_init`).
pub(crate) const MOVE_WORK_INIT: u32 = 0x0216_D684;
/// `FUN_0216D69C(m)`: the move work (`m + 0x94`).
pub(crate) const MOVE_WORK: u32 = 0x0216_D69C;
/// `FUN_0216D6A0(m, n)`: clear `n` bytes of the sub work (`m + 0xA4`) and return it.
pub(crate) const SUB_WORK_INIT: u32 = 0x0216_D6A0;
/// `FUN_0216D6B8(m)`: the sub work (`m + 0xA4`).
pub(crate) const SUB_WORK: u32 = 0x0216_D6B8;
/// `FUN_0216D674(m, v)`: draw animation code (`u16 +0x2A`) = v.
pub(crate) const SET_ANIM: u32 = 0x0216_D674;
/// `FUN_0216DA44(m)`: move bit 0x10 (walking) on.
pub(crate) const WALK_ON: u32 = 0x0216_DA44;
/// `FUN_0216DA50(m)`: move bit 0x10 (walking) off.
pub(crate) const WALK_OFF: u32 = 0x0216_DA50;
/// `FUN_0216DA7C(m)`: move bit 0x40 off.
pub(crate) const BIT40_OFF: u32 = 0x0216_DA7C;
/// `FUN_02163F10(m)`: old grid = grid.
pub(crate) const SAVE_OLD_GRID: u32 = 0x0216_3F10;
/// `FUN_0216D5E0(m, dir)`: display dir = dir unless the dir is locked.
pub(crate) const SET_DIR_DISP: u32 = 0x0216_D5E0;
/// `FUN_0216D5F8(m)`: the display dir.
pub(crate) const DIR_DISP: u32 = 0x0216_D5F8;
/// `FUN_0216D60C(m, dir)`: display dir and move dir = dir.
pub(crate) const SET_DIRS: u32 = 0x0216_D60C;
/// `FUN_0216D5D4(m)`: the header dir.
pub(crate) const HEADER_DIR: u32 = 0x0216_D5D4;
/// `FUN_0216D5A0(m)`: the move code (`u16 +0x0E`).
pub(crate) const MOVE_CODE: u32 = 0x0216_D5A0;
/// `FUN_0216D5A8(m)`: the event type (`u16 +0x10`).
pub(crate) const EVENT_TYPE: u32 = 0x0216_D5A8;
/// `FUN_0216D63C(m, i)`: header param `i`.
pub(crate) const PARAM: u32 = 0x0216_D63C;
/// `FUN_0216D438(m, code)`: acmd change.
pub(crate) const ACMD_CHANGE: u32 = 0x0216_D438;
/// `FUN_02197E7C(dir, base)`: the acmd code of the kind `base` for `dir` (acmd1).
pub(crate) const ACMD_CODE_FOR: u32 = 0x0219_7E7C;
/// `FUN_02197EDC(m)`: run the acmd one frame (acmd1 `action`): 1 = done.
pub(crate) const ACMD_ACTION: u32 = 0x0219_7EDC;
/// `FUN_02164168(m)`: 1 when the player is within 0x12 tiles of `m` (mmdl_move).
pub(crate) const PLAYER_NEAR: u32 = 0x0216_4168;
/// `FUN_021639C0(m, dir)`: hit bits of the step from `m` in `dir` (0 = free; mmdl_move).
pub(crate) const HIT: u32 = 0x0216_39C0;
/// `FUN_021641E8(dir)`: dx of a dir (mmdl_move).
pub(crate) const DIR_DX: u32 = 0x0216_41E8;
/// `FUN_021641F8(dir)`: dz of a dir (mmdl_move).
pub(crate) const DIR_DZ: u32 = 0x0216_41F8;
/// `FUN_021642A4(dir)`: the opposite dir (mmdl_move).
pub(crate) const OPPOSITE: u32 = 0x0216_42A4;
/// `FUN_0216D660(m)`: move limit x (s16).
pub(crate) const LIMIT_X: u32 = 0x0216_D660;
/// `FUN_0216D66C(m)`: move limit z (s16).
pub(crate) const LIMIT_Z: u32 = 0x0216_D66C;
/// `FUN_0216D7E8(m)`: initial grid x (s16).
pub(crate) const INIT_X: u32 = 0x0216_D7E8;
/// `FUN_0216D7F8(m)`: initial grid z (s16).
pub(crate) const INIT_Z: u32 = 0x0216_D7F8;
/// `FUN_0216D83C(m)`: grid x (s16).
pub(crate) const GRID_X: u32 = 0x0216_D83C;
/// `FUN_0216D86C(m)`: grid z (s16).
pub(crate) const GRID_Z: u32 = 0x0216_D86C;
/// `FUN_0219B0C4()`: pad bit 2 held (mvproc3b).
pub(crate) const PAD_HELD: u32 = 0x0219_B0C4;
/// `FUN_0219B0F0(table, term)`: a random entry of a ROM table (mvproc3b).
pub(crate) const RANDOM_ENTRY: u32 = 0x0219_B0F0;
/// `FUN_0219B104(set, term)`: a random dir of a direction set (mvproc3b).
pub(crate) const RANDOM_DIR: u32 = 0x0219_B104;
/// `FUN_0219B200(m, set, term)`: a dir toward the player when he is in sight, else `term` (mvproc3b).
pub(crate) const SIGHT_DIR: u32 = 0x0219_B200;
/// ROM `0x021D4FA4`: the look-around waits {0x10, 0x20, 0x30, 0x40}, ended by -1.
pub(crate) const WAITS: u32 = 0x021D_4FA4;

/// Starts the acmd `02197E7C(dir, base)` (`0216D438(m, (u16)code)`); the caller masks `dir` / `base` as the original does.
pub(crate) fn start_acmd<E: FieldEnv>(env: &mut E, m: u32, dir: u32, base: u32) {
    let code = env.callv(ACMD_CODE_FOR, &[dir, base]);
    env.callv(ACMD_CHANGE, &[m, code & 0xFFFF]);
}

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

/// C: `FUN_0219A0D8` (unlabeled routine at `0x0219A0D8`): move code 6 init (thunk): [`look_init`] with direction set 1 (up / left).
pub fn look_init_06<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 1);
}

/// C: `FUN_0219A0E4` (unlabeled routine at `0x0219A0E4`): move code 7 init (thunk): [`look_init`] with direction set 2 (up / right).
pub fn look_init_07<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 2);
}

/// C: `FUN_0219A0F0` (unlabeled routine at `0x0219A0F0`): move code 8 init (thunk): [`look_init`] with direction set 3 (down / left).
pub fn look_init_08<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 3);
}

/// C: `FUN_0219A0FC` (unlabeled routine at `0x0219A0FC`): move code 9 init (thunk): [`look_init`] with direction set 4 (down /
/// right).
pub fn look_init_09<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 4);
}

/// C: `FUN_0219A108` (unlabeled routine at `0x0219A108`): move code 0xA init (thunk): [`look_init`] with direction set 5 (up / down
/// / left).
pub fn look_init_0a<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 5);
}

/// C: `FUN_0219A114` (unlabeled routine at `0x0219A114`): move code 0xB init (thunk): [`look_init`] with direction set 6 (up / down
/// / right).
pub fn look_init_0b<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 6);
}

/// C: `FUN_0219A120` (unlabeled routine at `0x0219A120`): move code 0xC init (thunk): [`look_init`] with direction set 7 (up / left
/// / right).
pub fn look_init_0c<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 7);
}

/// C: `FUN_0219A12C` (unlabeled routine at `0x0219A12C`): move code 0xD init (thunk): [`look_init`] with direction set 8 (down /
/// left / right).
pub fn look_init_0d<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 8);
}

/// C: `FUN_0219A138` (unlabeled routine at `0x0219A138`): move code 0x2D init (thunk): [`look_init`] with direction set 9 (up /
/// down).
pub fn look_init_2d<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 9);
}

/// C: `FUN_0219A144` (unlabeled routine at `0x0219A144`): move code 0x2E init (thunk): [`look_init`] with direction set 10 (left /
/// right).
pub fn look_init_2e<E: FieldEnv>(env: &mut E, m: u32) {
    look_init(env, m, 10);
}

/// C: `FUN_0219A150` (unlabeled routine at `0x0219A150`): move proc of the look-around codes. Event type 0xB: only while the player
/// is near and pad bit 2 is held, turn to [`next_dir`] of the move code. Other types: turn toward the player in sight; else, with
/// `u16 +0` == 0, count the wait `s16 +2` down and at <= 0 draw a new wait and a random dir of the set `u32 +4`. Old grid = grid.
pub fn look_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    let d = if env.callv(EVENT_TYPE, &[m]) == 0xB {
        if env.callv(PLAYER_NEAR, &[m]) != 1 || env.callv(PAD_HELD, &[]) != 1 {
            None
        } else {
            let dir = env.callv(DIR_DISP, &[m]);
            let code = env.callv(MOVE_CODE, &[m]);
            let d = next_dir(code, dir);
            (d != 9).then_some(d)
        }
    } else {
        let set = env.r32(w.wrapping_add(4));
        let d = env.callv(SIGHT_DIR, &[m, set, 9]);
        if d != 9 {
            Some(d & 0xFFFF)
        } else if env.r16(w) != 0 {
            None
        } else {
            let t = (env.r16(w.wrapping_add(2)) as i16).wrapping_sub(1);
            env.w16(w.wrapping_add(2), t as u16);
            if t > 0 {
                None
            } else {
                let t = env.callv(RANDOM_ENTRY, &[WAITS, 0xFFFF_FFFF]);
                env.w16(w.wrapping_add(2), t as u16);
                let set = env.r32(w.wrapping_add(4));
                Some(env.callv(RANDOM_DIR, &[set, 9]) & 0xFFFF)
            }
        }
    };
    if let Some(d) = d {
        env.callv(SET_DIR_DISP, &[m, d]);
    }
    env.callv(SAVE_OLD_GRID, &[m]);
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

/// C: `FUN_0219A24C` (unlabeled routine at `0x0219A24C`): delete proc of the look-around codes: nothing.
pub fn look_delete<E: FieldEnv>(_env: &mut E, _m: u32) {}

/// C: `FUN_0219A250` (unlabeled routine at `0x0219A250`): move code 3 init: [`rand_walk_init`] with acmd base 0xC, direction set
/// 0xB, mode 0.
pub fn rand_walk_init_03<E: FieldEnv>(env: &mut E, m: u32) {
    rand_walk_init(env, m, 0xC, 0xB, 0);
}

/// C: `FUN_0219A260` (unlabeled routine at `0x0219A260`): move code 4 init: [`rand_walk_init`] with acmd base 0xC, direction set
/// 0xC, mode 0.
pub fn rand_walk_init_04<E: FieldEnv>(env: &mut E, m: u32) {
    rand_walk_init(env, m, 0xC, 0xC, 0);
}

/// C: `FUN_0219A270` (unlabeled routine at `0x0219A270`): move code 5 init: [`rand_walk_init`] with acmd base 0xC, direction set
/// 0xD, mode 0.
pub fn rand_walk_init_05<E: FieldEnv>(env: &mut E, m: u32) {
    rand_walk_init(env, m, 0xC, 0xD, 0);
}

/// C: `FUN_0219A280` (unlabeled routine at `0x0219A280`): move code 0x43 init: [`rand_walk_init`] with acmd base 0xC, direction
/// set 0xD, mode 2.
pub fn rand_walk_init_43<E: FieldEnv>(env: &mut E, m: u32) {
    rand_walk_init(env, m, 0xC, 0xD, 2);
}

/// C: `FUN_0219A290` (unlabeled routine at `0x0219A290`): random-walk init: move work (0x10 bytes cleared) `+4` = mode, `+8` = acmd
/// base code, `+0xC` = direction-set id; draw animation code 0; move bit 0x10 off.
pub fn rand_walk_init<E: FieldEnv>(env: &mut E, m: u32, code: u32, set: u32, mode: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 0x10]);
    env.w32(w.wrapping_add(4), mode);
    env.w32(w.wrapping_add(8), code);
    env.w32(w.wrapping_add(0xC), set);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(WALK_OFF, &[m]);
}

/// C: `FUN_0219A2B8` (unlabeled routine at `0x0219A2B8`): move proc of the random-walk codes (state `s16 +0`, 0..4; other states do
/// nothing): 0 face, 1 wait for the acmd and draw a wait, 2 count it down, 3 pick a random dir and start a walk (back to 0 when it
/// leaves the move limits in mode 1 or is blocked; mode 2 walks on when hit bit 0 is clear), 4 wait for the walk.
pub fn rand_walk_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    let state = |env: &mut E| env.r16(w) as i16;
    let next = |env: &mut E| {
        let s = (env.r16(w) as i16).wrapping_add(1);
        env.w16(w, s as u16);
    };
    let mut s = state(env);
    if s == 0 {
        env.callv(WALK_OFF, &[m]);
        env.callv(BIT40_OFF, &[m]);
        let d = env.callv(DIR_DISP, &[m]) & 0xFFFF;
        start_acmd(env, m, d, 0);
        next(env);
        return;
    }
    if s == 1 {
        if env.callv(ACMD_ACTION, &[m]) == 0 {
            return;
        }
        let t = env.callv(RANDOM_ENTRY, &[WAITS, 0xFFFF_FFFF]);
        env.w16(w.wrapping_add(2), t as u16);
        next(env);
        s = 2;
    }
    if s == 2 {
        let t = (env.r16(w.wrapping_add(2)) as i16).wrapping_sub(1);
        env.w16(w.wrapping_add(2), t as u16);
        if t != 0 {
            return;
        }
        next(env);
        s = 3;
    }
    if s == 3 {
        let set = env.r32(w.wrapping_add(0xC));
        let d = env.callv(RANDOM_DIR, &[set, 9]);
        env.callv(SET_DIRS, &[m, d & 0xFFFF]);
        let mode = env.r32(w.wrapping_add(4));
        if mode == 1 && in_limits(env, m, d) == 0 {
            env.w16(w, 0);
            return;
        }
        let hit = env.callv(HIT, &[m, d & 0xFFFF]);
        if hit != 0 && (env.r32(w.wrapping_add(4)) != 2 || hit & 1 != 0) {
            env.w16(w, 0);
            return;
        }
        let base = env.r32(w.wrapping_add(8)) & 0xFFFF;
        start_acmd(env, m, d & 0xFFFF, base);
        env.callv(WALK_ON, &[m]);
        next(env);
        s = 4;
    }
    if s == 4 && env.callv(ACMD_ACTION, &[m]) != 0 {
        env.callv(WALK_OFF, &[m]);
        env.w16(w, 0);
    }
}

/// The move-limit rectangle `[x0, z0, x1, z1]` of [`limit_rect`] (`None` for the codes it leaves unwritten).
fn limit_rect_of<E: FieldEnv>(env: &mut E, m: u32) -> Option<[u32; 4]> {
    let x = env.callv(INIT_X, &[m]);
    let z = env.callv(INIT_Z, &[m]);
    let lx = env.callv(LIMIT_X, &[m]);
    let lz = env.callv(LIMIT_Z, &[m]);
    let (xl, xh, zl, zh) = (
        x.wrapping_sub(lx),
        x.wrapping_add(lx),
        z.wrapping_sub(lz),
        z.wrapping_add(lz),
    );
    Some(match env.callv(MOVE_CODE, &[m]) {
        6 => [xl, zl, x, z],
        7 => [x, zl, xh, z],
        8 => [xl, z, x, zh],
        9 => [x, z, xh, zh],
        0xA => [xl, zl, x, zh],
        0xB => [x, zl, xh, zh],
        0xC => [xl, zl, xh, z],
        0xD => [xl, z, xh, zh],
        _ => return None,
    })
}

/// C: `FUN_0219A3D8` (unlabeled routine at `0x0219A3D8`): the move-limit rectangle `{s32 x0, z0, x1, z1}` at `out` of the move code
/// 6..0xD (quarter / half planes of the limit box around the initial tile); other codes write nothing.
pub fn limit_rect<E: FieldEnv>(env: &mut E, m: u32, out: u32) {
    if let Some(r) = limit_rect_of(env, m) {
        for (i, v) in r.into_iter().enumerate() {
            env.w32(out.wrapping_add(4 * i as u32), v);
        }
    }
}

/// C: `FUN_0219A4CC` (unlabeled routine at `0x0219A4CC`): 1 when the tile next to `m` in `dir` lies inside [`limit_rect`]'s
/// rectangle, else 0. UNVERIFIED for move codes outside 6..0xD (the original compares uninitialised stack; the port panics; only
/// random-walk mode 1 reaches it and no record sets that mode).
pub fn in_limits<E: FieldEnv>(env: &mut E, m: u32, dir: u32) -> u32 {
    let r = limit_rect_of(env, m);
    let gx = env.callv(GRID_X, &[m]);
    let x = gx.wrapping_add(env.callv(DIR_DX, &[dir & 0xFFFF])) as i32;
    let gz = env.callv(GRID_Z, &[m]);
    let z = gz.wrapping_add(env.callv(DIR_DZ, &[dir & 0xFFFF])) as i32;
    let r = r.expect(
        "UNVERIFIED: 0219A4CC with a move code outside 6..0xD reads an unwritten rectangle",
    );
    let [x0, z0, x1, z1] = r.map(|v| v as i32);
    if x0 > x || x1 < x {
        return 0;
    }
    u32::from(z0 <= z && z1 >= z)
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

/// C: `FUN_0219A574` (unlabeled routine at `0x0219A574`): move code 0xE init (thunk): [`face_init`] facing up.
pub fn face_init_up<E: FieldEnv>(env: &mut E, m: u32) {
    face_init(env, m, 0);
}

/// C: `FUN_0219A580` (unlabeled routine at `0x0219A580`): move code 0xF init (thunk): [`face_init`] facing down.
pub fn face_init_down<E: FieldEnv>(env: &mut E, m: u32) {
    face_init(env, m, 1);
}

/// C: `FUN_0219A58C` (unlabeled routine at `0x0219A58C`): move code 0x10 init (thunk): [`face_init`] facing left.
pub fn face_init_left<E: FieldEnv>(env: &mut E, m: u32) {
    face_init(env, m, 2);
}

/// C: `FUN_0219A598` (unlabeled routine at `0x0219A598`): move code 0x11 init (thunk): [`face_init`] facing right.
pub fn face_init_right<E: FieldEnv>(env: &mut E, m: u32) {
    face_init(env, m, 3);
}

/// C: `FUN_0219A5A4` (unlabeled routine at `0x0219A5A4`): init of the spin codes 0x12 / 0x13 / 0x31: move work (8 bytes cleared)
/// `u8 +0` = `k` (the turn order: 2 or other); draw animation code 0; move bit 0x10 off; old grid = grid.
pub fn spin_init<E: FieldEnv>(env: &mut E, m: u32, k: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 8]);
    env.w8(w, k as u8);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(WALK_OFF, &[m]);
    env.callv(SAVE_OLD_GRID, &[m]);
}

/// C: `FUN_0219A5C8` (unlabeled routine at `0x0219A5C8`): move code 0x12 init (thunk): [`spin_init`] with order 2.
pub fn spin_init_12<E: FieldEnv>(env: &mut E, m: u32) {
    spin_init(env, m, 2);
}

/// C: `FUN_0219A5D4` (unlabeled routine at `0x0219A5D4`): move code 0x13 init (thunk): [`spin_init`] with order 3.
pub fn spin_init_13<E: FieldEnv>(env: &mut E, m: u32) {
    spin_init(env, m, 3);
}

/// ROM `0x021D4DE8`: the states of [`spin_move`].
pub const SPIN_STATES: [u32; 4] = [0x0219_A605, 0x0219_A639, 0x0219_A661, 0x0219_A6CD];
/// ROM `0x021D4DB8`: the states of [`spin2_move`].
pub const SPIN2_STATES: [u32; 4] = [0x0219_A781, 0x0219_A7A9, 0x0219_A7C5, 0x0219_A7DD];

/// C: `FUN_0219A5E0` (unlabeled routine at `0x0219A5E0`): move proc of codes 0x12 / 0x13: runs [`SPIN_STATES`]`[s8 +2]` while it
/// answers 1.
pub fn spin_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w.wrapping_add(2)) as i8 {
            0 => spin_face(env, m, w),
            1 => spin_face_wait(env, m, w),
            2 => spin_watch(env, m, w),
            3 => spin_turn(env, m, w),
            s => panic!("UNVERIFIED: 0219A5E0 state {s} past the ROM table 0x021D4DE8"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219A604` (unlabeled routine at `0x0219A604`): state 0 of [`spin_move`]: face toward the player in sight (else the
/// display dir); state 1; returns 1.
pub fn spin_face<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let mut d = env.callv(SIGHT_DIR, &[m, 0x26, 9]);
    if d == 9 {
        d = env.callv(DIR_DISP, &[m]);
    }
    start_acmd(env, m, d & 0xFFFF, 0);
    env.w8(w.wrapping_add(2), 1);
    1
}

/// C: `FUN_0219A638` (unlabeled routine at `0x0219A638`): state 1 of [`spin_move`]: 0 until the acmd is done; then (event type not
/// 0xB) `u32 +4` = 0; state 2; returns 1.
pub fn spin_face_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 0 {
        return 0;
    }
    if env.callv(EVENT_TYPE, &[m]) != 0xB {
        env.w32(w.wrapping_add(4), 0);
    }
    env.w8(w.wrapping_add(2), 2);
    1
}

/// C: `FUN_0219A660` (unlabeled routine at `0x0219A660`): state 2 of [`spin_move`]. Event type 0xB: a set counter `u32 +4` is
/// cleared (returns 0); else the player near and pad bit 2 held count it up and go to state 3. Other types: the player in sight
/// (counter set) goes back to state 0; else the counter goes up and reaches state 3 at 0x18.
pub fn spin_watch<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let ty = env.callv(EVENT_TYPE, &[m]);
    let n = env.r32(w.wrapping_add(4));
    if ty == 0xB {
        if n != 0 {
            env.w32(w.wrapping_add(4), 0);
            return 0;
        }
        if env.callv(PLAYER_NEAR, &[m]) == 0 || env.callv(PAD_HELD, &[]) == 0 {
            return 0;
        }
        let n = env.r32(w.wrapping_add(4)).wrapping_add(1);
        env.w32(w.wrapping_add(4), n);
    } else {
        if n != 0 && env.callv(SIGHT_DIR, &[m, 0x26, 9]) != 9 {
            env.w8(w.wrapping_add(2), 0);
            return 1;
        }
        let n = env.r32(w.wrapping_add(4)).wrapping_add(1);
        env.w32(w.wrapping_add(4), n);
        if (n as i32) < 0x18 {
            return 0;
        }
    }
    env.w8(w.wrapping_add(2), 3);
    1
}

/// ROM `0x021D5024` / `0x021D5038` (and the equal `0x021D504C` / `0x021D5060`): the spin turn orders (u32 dirs ended by 9) for
/// order 2 and the others.
pub const SPIN_ORDERS: [[u32; 5]; 2] = [[0, 2, 1, 3, 9], [0, 3, 1, 2, 9]];

/// Turns `m` to the dir after its display dir in the turn order of `s8 w+0` (index 0 is compared too; a dir not in the order reads
/// past the copied table: panics, UNVERIFIED).
fn spin_turn_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) {
    let order = &SPIN_ORDERS[usize::from(env.r8(w) as i8 != 2)];
    let d = env.callv(DIR_DISP, &[m]);
    let mut i = 0;
    while order[i] != 9 && d != order[i] {
        i += 1;
    }
    i += 1;
    if order[i] == 9 {
        i = 0;
    }
    env.callv(SET_DIR_DISP, &[m, order[i] & 0xFFFF]);
}

/// C: `FUN_0219A6CC` (unlabeled routine at `0x0219A6CC`): state 3 of [`spin_move`]: turn to the next dir of the [`SPIN_ORDERS`] row
/// (wrapping at the 9); state 0; returns 1.
pub fn spin_turn<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    spin_turn_step(env, m, w);
    env.w8(w.wrapping_add(2), 0);
    1
}

/// C: `FUN_0219A750` (unlabeled routine at `0x0219A750`): move code 0x31 init (thunk): [`spin_init`] with order 3.
pub fn spin_init_31<E: FieldEnv>(env: &mut E, m: u32) {
    spin_init(env, m, 3);
}

/// C: `FUN_0219A75C` (unlabeled routine at `0x0219A75C`): move proc of code 0x31: runs [`SPIN2_STATES`]`[s8 +2]` while it answers
/// 1.
pub fn spin2_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w.wrapping_add(2)) as i8 {
            0 => spin2_face(env, m, w),
            1 => spin2_face_wait(env, m, w),
            2 => spin_wait(env, w),
            3 => spin2_turn(env, m, w),
            s => panic!("UNVERIFIED: 0219A75C state {s} past the ROM table 0x021D4DB8"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219A780` (unlabeled routine at `0x0219A780`): state 0 of [`spin2_move`]: face the display dir; state 1; returns 1.
pub fn spin2_face<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let d = env.callv(DIR_DISP, &[m]) & 0xFFFF;
    start_acmd(env, m, d, 0);
    env.w8(w.wrapping_add(2), 1);
    1
}

/// C: `FUN_0219A7A8` (unlabeled routine at `0x0219A7A8`): state 1 of [`spin2_move`]: 0 until the acmd is done; then `u32 +4` = 0,
/// state 2, returns 1.
pub fn spin2_face_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 0 {
        return 0;
    }
    env.w32(w.wrapping_add(4), 0);
    env.w8(w.wrapping_add(2), 2);
    1
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

/// C: `FUN_0219A7DC` (unlabeled routine at `0x0219A7DC`): state 3 of [`spin2_move`]: as [`spin_turn`]; then, when the display dir is
/// the header dir, the order byte `+0` = the opposite of itself (2 <-> 3); state 0; returns 1.
pub fn spin2_turn<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    spin_turn_step(env, m, w);
    let d = env.callv(DIR_DISP, &[m]);
    let h = env.callv(HEADER_DIR, &[m]);
    if d == h {
        let k = env.r8(w) as i8 as i32 as u32;
        let o = env.callv(OPPOSITE, &[k]);
        env.w8(w, o as u8);
    }
    env.w8(w.wrapping_add(2), 0);
    1
}

#[cfg(test)]
mod tests;
