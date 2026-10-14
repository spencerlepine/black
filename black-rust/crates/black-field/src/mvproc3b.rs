//! `fldmmdl_acmd.c`, part 3b (overlay 21, `0x0219A87C..0x0219B41C`; F1 row 21.09, F4b job 3): the movement-type procs for walking
//! back and forth (move code 0x14), the 4-leg routes (0x15..0x24), the reversible routes (0x25..0x2C), and the helpers the other
//! `mvproc3*` areas share: pad bits, random table entries, the direction-set / route table lookup `0219B120`, the "toward the player
//! in sight" dir `0219B200` and the spin-look of the walking event types.
//!
//! F4b-spec3 specced all 55 functions (`tools/rom-diff/specs/black-field/mvproc3b.spec`); F4b-port2 ported all of them. The ROM
//! data the procs read (state tables, direction sets, routes, turn orders) is embedded in [`rom`] (`0x021D4D04..0x021D548C`).

use crate::hooks::FieldEnv;
use crate::mvproc3a::{
    start_acmd, ACMD_ACTION, DIR_DISP, EVENT_TYPE, GRID_X, GRID_Z, HEADER_DIR, HIT, INIT_X, INIT_Z,
    MOVE_CODE, MOVE_WORK, MOVE_WORK_INIT, OPPOSITE, PARAM, PLAYER_NEAR, SET_DIR_DISP, WALK_OFF,
    WALK_ON,
};

pub mod rom;
use rom::rom32;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_A87C,
    0x0219_A89C,
    0x0219_A8C0,
    0x0219_A900,
    0x0219_AA08,
    0x0219_AA38,
    0x0219_AA68,
    0x0219_AA78,
    0x0219_AA88,
    0x0219_AA98,
    0x0219_AAA8,
    0x0219_AAB8,
    0x0219_AAC8,
    0x0219_AAD8,
    0x0219_AAE8,
    0x0219_AAF8,
    0x0219_AB08,
    0x0219_AB18,
    0x0219_AB28,
    0x0219_AB38,
    0x0219_AB48,
    0x0219_AB58,
    0x0219_AB68,
    0x0219_AB8C,
    0x0219_ACE8,
    0x0219_AD18,
    0x0219_AD48,
    0x0219_AD58,
    0x0219_AD68,
    0x0219_AD78,
    0x0219_AD88,
    0x0219_AD98,
    0x0219_ADA8,
    0x0219_ADB8,
    0x0219_ADC8,
    0x0219_ADEC,
    0x0219_AE10,
    0x0219_AE68,
    0x0219_AEC8,
    0x0219_B030,
    0x0219_B060,
    0x0219_B074,
    0x0219_B0C4,
    0x0219_B0D8,
    0x0219_B0F0,
    0x0219_B104,
    0x0219_B120,
    0x0219_B13C,
    0x0219_B200,
    0x0219_B2E8,
    0x0219_B334,
    0x0219_B358,
    0x0219_B374,
    0x0219_B390,
    0x0219_B404,
];

/// `FUN_0216D548(m, v)`: status |= v.
const SET_STATUS: u32 = 0x0216_D548;
/// `FUN_0216D550(m, v)`: status &= !v.
const CLEAR_STATUS: u32 = 0x0216_D550;
/// `FUN_0216D55C(m, mask)`: status & mask.
const STATUS: u32 = 0x0216_D55C;
/// Status bit 8: the display dir is locked.
const DIR_LOCK: u32 = 8;
/// `FUN_0216D600(m, dir)`: move dir = dir.
const SET_MOVE_DIR: u32 = 0x0216_D600;
/// `FUN_0216D608(m)`: the move dir.
pub(crate) const MOVE_DIR: u32 = 0x0216_D608;
/// `FUN_0216D67C(m)`: the MMDLSYS of `m`.
pub(crate) const SYS_OF: u32 = 0x0216_D67C;
/// `FUN_0216DEE0(sys)`: the player MMDL or 0.
pub(crate) const FIND_PLAYER: u32 = 0x0216_DEE0;
/// `FUN_0216D7C4(m)`: the acmd code.
pub(crate) const ACMD_CODE: u32 = 0x0216_D7C4;
/// `FUN_0216D854(m)`: grid y (s16).
pub(crate) const GRID_Y: u32 = 0x0216_D854;
/// `FUN_021642B4(x0, z0, x1, z1)`: the dir from (x0, z0) to (x1, z1) (mmdl_move).
const DIR_TOWARD: u32 = 0x0216_42B4;
/// `FUN_0216E0E8(m, code)`: change the move code.
const CHANGE_MOVE_CODE: u32 = 0x0216_E0E8;
/// `FUN_021AE198(m, dir, range, 0)`: distance at which the trainer sees the player, or -1 (trainer_eye).
const TRAINER_SEES: u32 = 0x021A_E198;
/// `FUN_0203630C()`: ARM9 pad trigger bits.
const PAD_TRIGGER_BITS: u32 = 0x0203_630C;
/// `FUN_020362DC()`: ARM9 pad held bits.
const PAD_HELD_BITS: u32 = 0x0203_62DC;
/// `FUN_020056EC(max)`: a random number in `[0, max)`.
const RNG: u32 = 0x0200_56EC;

/// ROM `0x021D4D60`: the states of [`walk_move`].
pub const WALK_STATES: [u32; 3] = [0x0219_A8C1, 0x0219_A901, 0x0219_AA09];
/// ROM `0x021D4D04`: the states of [`route_move`].
pub const ROUTE_STATES: [u32; 2] = [0x0219_AB8D, 0x0219_ACE9];
/// ROM `0x021D4D78`: the states of [`rroute_move`].
pub const RROUTE_STATES: [u32; 4] = [0x0219_AE11, 0x0219_AE69, 0x0219_AEC9, 0x0219_B031];
/// ROM `0x021D50F8`: pairs {id, table} of [`dir_table`] (id 0x27 ends the list).
const DIR_TABLES: u32 = 0x021D_50F8;
/// ROM `0x021D4FCC`: the spin-look turn orders, rows {0, 2, 1, 3} / {0, 3, 1, 2} (u32).
const SPIN_LOOK_ORDERS: u32 = 0x021D_4FCC;
/// ROM `0x021D4FEC`: the move codes of the types that turn toward a fast-walking player (0xFF ends).
const SIGHT_CODES: u32 = 0x021D_4FEC;
/// ROM `0x021D4F90`: the reversible route pairs (u16 move codes; 0xFF ends).
const ROUTE_PAIRS: u32 = 0x021D_4F90;

/// The u8 at `w + o`.
fn b<E: FieldEnv>(env: &mut E, w: u32, o: u32) -> u32 {
    u32::from(env.r8(w.wrapping_add(o)))
}

/// 1 when `m` stands on its initial tile (initial x, initial z, grid x, grid z read in that order).
fn on_initial_tile<E: FieldEnv>(env: &mut E, m: u32) -> bool {
    let ix = env.callv(INIT_X, &[m]);
    let iz = env.callv(INIT_Z, &[m]);
    let x = env.callv(GRID_X, &[m]);
    let z = env.callv(GRID_Z, &[m]);
    ix == x && iz == z
}

/// 1 when `m` stands on its initial line along `axis` (0: x, else z).
fn on_initial_line<E: FieldEnv>(env: &mut E, m: u32, axis: u32) -> bool {
    if axis == 0 {
        let ix = env.callv(INIT_X, &[m]);
        ix == env.callv(GRID_X, &[m])
    } else {
        let iz = env.callv(INIT_Z, &[m]);
        iz == env.callv(GRID_Z, &[m])
    }
}

/// Move dir = `d`, display dir = `d` unless the type spin-looks, then the hit bits of the step in `d`.
fn face_and_hit<E: FieldEnv>(env: &mut E, m: u32, d: u32) -> u32 {
    env.callv(SET_MOVE_DIR, &[m, d & 0xFFFF]);
    if spins_while_walking(env, m) == 0 {
        env.callv(SET_DIR_DISP, &[m, d & 0xFFFF]);
    }
    env.callv(HIT, &[m, d & 0xFFFF])
}

/// The common end of the walk steps: acmd base 0xC (walk), 0x10 for the fast types when the player is near and pressed, 0x20 when
/// `hit` (walk in place); start it; spin-look start at `look`; move bit 0x10 on.
fn walk_tail<E: FieldEnv>(env: &mut E, m: u32, d: u32, hit: u32, look: u32) {
    let mut base = 0xC;
    if walks_fast_near(env, m) != 0 && pad_trigger(env) == 1 && env.callv(PLAYER_NEAR, &[m]) == 1 {
        base = 0x10;
    }
    if hit != 0 {
        base = 0x20;
    }
    start_acmd(env, m, d & 0xFFFF, base);
    if spins_while_walking(env, m) == 1 {
        spin_look_start(env, m, look);
    }
    env.callv(WALK_ON, &[m]);
}

/// The common end of the walk waits: 0 until the acmd is done (`02197EDC` == 1); then move bit 0x10 off, spin-look end at `look`,
/// state 0; returns 0.
fn walk_wait_tail<E: FieldEnv>(env: &mut E, m: u32, w: u32, look: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 1 {
        env.callv(WALK_OFF, &[m]);
        if spins_while_walking(env, m) == 1 {
            spin_unlock(env, m, look);
        }
        env.w8(w, 0);
    }
    0
}

/// C: `FUN_0219A87C` (unlabeled routine at `0x0219A87C`): move code 0x14 init: move work (8 bytes cleared); for the spin-look
/// event types [`spin_order`] at `+4`.
pub fn walk_init<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 8]);
    if spins_while_walking(env, m) == 1 {
        spin_order(env, m, w.wrapping_add(4));
    }
}

/// C: `FUN_0219A89C` (unlabeled routine at `0x0219A89C`): move proc of code 0x14: runs [`WALK_STATES`]`[u8 +0]` while it answers 1.
pub fn walk_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w) {
            0 => walk_start(env, m, w),
            1 => walk_step(env, m, w),
            2 => walk_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219A89C state {s} past the ROM table 0x021D4D60"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219A8C0` (unlabeled routine at `0x0219A8C0`): state 0 of [`walk_move`]: move dir = the header dir (its opposite when
/// `u8 +1` == 1), display dir too unless the type spin-looks; state 1; returns 1.
pub fn walk_start<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let mut d = env.callv(HEADER_DIR, &[m]);
    if b(env, w, 1) == 1 {
        d = env.callv(OPPOSITE, &[d & 0xFFFF]);
    }
    env.callv(SET_MOVE_DIR, &[m, d & 0xFFFF]);
    if spins_while_walking(env, m) == 0 {
        env.callv(SET_DIR_DISP, &[m, d & 0xFFFF]);
    }
    env.w8(w, 1);
    1
}

/// C: `FUN_0219A900` (unlabeled routine at `0x0219A900`): state 1 of [`walk_move`]: held by the sight counter `+2` ([`may_move`]:
/// 0); reversed (`u8 +1`) and back on the initial tile -> turn around and clear `+1`; walk in the move dir, turning around (`+1` =
/// 1) when the step is blocked; state 2; returns 1.
pub fn walk_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if may_move(env, m, w.wrapping_add(2)) == 0 {
        return 0;
    }
    if b(env, w, 1) != 0 && on_initial_tile(env, m) {
        let md = env.callv(MOVE_DIR, &[m]);
        let d = env.callv(OPPOSITE, &[md]);
        env.callv(SET_MOVE_DIR, &[m, d & 0xFFFF]);
        if spins_while_walking(env, m) == 0 {
            env.callv(SET_DIR_DISP, &[m, d & 0xFFFF]);
        }
        env.w8(w.wrapping_add(1), 0);
    }
    let mut d = env.callv(MOVE_DIR, &[m]);
    let mut hit = env.callv(HIT, &[m, d & 0xFFFF]);
    if hit & 1 != 0 {
        env.w8(w.wrapping_add(1), 1);
        d = env.callv(OPPOSITE, &[d & 0xFFFF]);
        hit = env.callv(HIT, &[m, d & 0xFFFF]);
    }
    walk_tail(env, m, d, hit, w.wrapping_add(4));
    env.w8(w, 2);
    1
}

/// C: `FUN_0219AA08` (unlabeled routine at `0x0219AA08`): state 2 of [`walk_move`]: when the acmd is done, move bit 0x10 off,
/// spin-look end (`+4`), state 0; returns 0.
pub fn walk_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    walk_wait_tail(env, m, w, w.wrapping_add(4))
}

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

/// C: `FUN_0219AA78` (unlabeled routine at `0x0219AA78`): move code 0x16 init: [`route_init`]`(m, 2, 0, 0xF)`.
pub fn route_init_16<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0xF);
}

/// C: `FUN_0219AA88` (unlabeled routine at `0x0219AA88`): move code 0x17 init: [`route_init`]`(m, 2, 1, 0x10)`.
pub fn route_init_17<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x10);
}

/// C: `FUN_0219AA98` (unlabeled routine at `0x0219AA98`): move code 0x18 init: [`route_init`]`(m, 2, 1, 0x11)`.
pub fn route_init_18<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x11);
}

/// C: `FUN_0219AAA8` (unlabeled routine at `0x0219AAA8`): move code 0x19 init: [`route_init`]`(m, 2, 0, 0x12)`.
pub fn route_init_19<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0x12);
}

/// C: `FUN_0219AAB8` (unlabeled routine at `0x0219AAB8`): move code 0x1A init: [`route_init`]`(m, 2, 0, 0x13)`.
pub fn route_init_1a<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0x13);
}

/// C: `FUN_0219AAC8` (unlabeled routine at `0x0219AAC8`): move code 0x1B init: [`route_init`]`(m, 2, 1, 0x14)`.
pub fn route_init_1b<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x14);
}

/// C: `FUN_0219AAD8` (unlabeled routine at `0x0219AAD8`): move code 0x1C init: [`route_init`]`(m, 2, 1, 0x15)`.
pub fn route_init_1c<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x15);
}

/// C: `FUN_0219AAE8` (unlabeled routine at `0x0219AAE8`): move code 0x1D init: [`route_init`]`(m, 2, 1, 0x16)`.
pub fn route_init_1d<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x16);
}

/// C: `FUN_0219AAF8` (unlabeled routine at `0x0219AAF8`): move code 0x1E init: [`route_init`]`(m, 2, 1, 0x17)`.
pub fn route_init_1e<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x17);
}

/// C: `FUN_0219AB08` (unlabeled routine at `0x0219AB08`): move code 0x1F init: [`route_init`]`(m, 2, 0, 0x18)`.
pub fn route_init_1f<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0x18);
}

/// C: `FUN_0219AB18` (unlabeled routine at `0x0219AB18`): move code 0x20 init: [`route_init`]`(m, 2, 0, 0x19)`.
pub fn route_init_20<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0x19);
}

/// C: `FUN_0219AB28` (unlabeled routine at `0x0219AB28`): move code 0x21 init: [`route_init`]`(m, 2, 1, 0x1A)`.
pub fn route_init_21<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x1A);
}

/// C: `FUN_0219AB38` (unlabeled routine at `0x0219AB38`): move code 0x22 init: [`route_init`]`(m, 2, 1, 0x1B)`.
pub fn route_init_22<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 1, 0x1B);
}

/// C: `FUN_0219AB48` (unlabeled routine at `0x0219AB48`): move code 0x23 init: [`route_init`]`(m, 2, 0, 0x1C)`.
pub fn route_init_23<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0x1C);
}

/// C: `FUN_0219AB58` (unlabeled routine at `0x0219AB58`): move code 0x24 init: [`route_init`]`(m, 2, 0, 0x1D)`.
pub fn route_init_24<E: FieldEnv>(env: &mut E, m: u32) {
    route_init(env, m, 2, 0, 0x1D);
}

/// C: `FUN_0219AB68` (unlabeled routine at `0x0219AB68`): move proc of the route codes: runs [`ROUTE_STATES`]`[u8 +0]` while it
/// answers 1.
pub fn route_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w) {
            0 => route_step(env, m, w),
            1 => route_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219AB68 state {s} past the ROM table 0x021D4D04"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219AB8C` (unlabeled routine at `0x0219AB8C`): state 0 of [`route_move`]: held by the sight counter `+5` (0). At the turn
/// leg (`u8 +1` == `+2`) on the initial line of the axis `+3` the leg goes on; leg 3 on the initial tile starts over (leg 0). Walk
/// in the leg's dir of the route `+4` (blocked: next leg, unchecked index into the ROM route); state 1; returns 1.
pub fn route_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if may_move(env, m, w.wrapping_add(5)) == 0 {
        return 0;
    }
    if b(env, w, 1) == b(env, w, 2) {
        let axis = b(env, w, 3);
        if on_initial_line(env, m, axis) {
            let l = env.r8(w.wrapping_add(1)).wrapping_add(1);
            env.w8(w.wrapping_add(1), l);
        }
    }
    if b(env, w, 1) == 3 && on_initial_tile(env, m) {
        env.w8(w.wrapping_add(1), 0);
    }
    let id = b(env, w, 4);
    let t = dir_table(id);
    let mut d = rom32(t.wrapping_add(4 * b(env, w, 1)));
    let mut hit = face_and_hit(env, m, d);
    if hit & 1 != 0 {
        let l = env.r8(w.wrapping_add(1)).wrapping_add(1);
        env.w8(w.wrapping_add(1), l);
        d = rom32(t.wrapping_add(4 * b(env, w, 1)));
        hit = face_and_hit(env, m, d);
    }
    walk_tail(env, m, d, hit, w.wrapping_add(8));
    env.w8(w, 1);
    1
}

/// C: `FUN_0219ACE8` (unlabeled routine at `0x0219ACE8`): state 1 of [`route_move`]: when the acmd is done, move bit 0x10 off,
/// spin-look end (`+8`), state 0; returns 0.
pub fn route_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    walk_wait_tail(env, m, w, w.wrapping_add(8))
}

/// C: `FUN_0219AD18` (unlabeled routine at `0x0219AD18`): init of the reversible route codes 0x25..0x2C: as [`route_init`].
pub fn rroute_init<E: FieldEnv>(env: &mut E, m: u32, turn: u32, axis: u32, route: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 0xC]);
    env.w8(w.wrapping_add(2), turn as u8);
    env.w8(w.wrapping_add(3), axis as u8);
    env.w8(w.wrapping_add(4), route as u8);
    if spins_while_walking(env, m) == 1 {
        spin_order(env, m, w.wrapping_add(8));
    }
}

/// C: `FUN_0219AD48` (unlabeled routine at `0x0219AD48`): move code 0x25 init: [`rroute_init`]`(m, 2, 1, 0x1E)`.
pub fn rroute_init_25<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 1, 0x1E);
}

/// C: `FUN_0219AD58` (unlabeled routine at `0x0219AD58`): move code 0x26 init: [`rroute_init`]`(m, 2, 1, 0x1F)`.
pub fn rroute_init_26<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 1, 0x1F);
}

/// C: `FUN_0219AD68` (unlabeled routine at `0x0219AD68`): move code 0x27 init: [`rroute_init`]`(m, 2, 0, 0x20)`.
pub fn rroute_init_27<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 0, 0x20);
}

/// C: `FUN_0219AD78` (unlabeled routine at `0x0219AD78`): move code 0x28 init: [`rroute_init`]`(m, 2, 0, 0x21)`.
pub fn rroute_init_28<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 0, 0x21);
}

/// C: `FUN_0219AD88` (unlabeled routine at `0x0219AD88`): move code 0x29 init: [`rroute_init`]`(m, 2, 1, 0x22)`.
pub fn rroute_init_29<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 1, 0x22);
}

/// C: `FUN_0219AD98` (unlabeled routine at `0x0219AD98`): move code 0x2A init: [`rroute_init`]`(m, 2, 1, 0x23)`.
pub fn rroute_init_2a<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 1, 0x23);
}

/// C: `FUN_0219ADA8` (unlabeled routine at `0x0219ADA8`): move code 0x2B init: [`rroute_init`]`(m, 2, 0, 0x24)`.
pub fn rroute_init_2b<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 0, 0x24);
}

/// C: `FUN_0219ADB8` (unlabeled routine at `0x0219ADB8`): move code 0x2C init: [`rroute_init`]`(m, 2, 0, 0x25)`.
pub fn rroute_init_2c<E: FieldEnv>(env: &mut E, m: u32) {
    rroute_init(env, m, 2, 0, 0x25);
}

/// C: `FUN_0219ADC8` (unlabeled routine at `0x0219ADC8`): move proc of the reversible route codes: runs
/// [`RROUTE_STATES`]`[u8 +0]` while it answers 1.
pub fn rroute_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w) {
            0 => rroute_sight(env, m, w),
            1 => rroute_reverse(env, m, w),
            2 => rroute_step(env, m, w),
            3 => rroute_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219ADC8 state {s} past the ROM table 0x021D4D78"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219ADEC` (unlabeled routine at `0x0219ADEC`): step the leg `s8 w+1` of a reversible route: backwards (`u8 w+5` == 1)
/// with a wrap from -1 to 3, else forwards (no wrap).
pub fn rroute_next_leg<E: FieldEnv>(env: &mut E, w: u32) {
    if env.r8(w.wrapping_add(5)) == 1 {
        let l = (env.r8(w.wrapping_add(1)) as i8).wrapping_sub(1);
        env.w8(w.wrapping_add(1), l as u8);
        if l < 0 {
            env.w8(w.wrapping_add(1), 3);
        }
    } else {
        let l = env.r8(w.wrapping_add(1)).wrapping_add(1);
        env.w8(w.wrapping_add(1), l);
    }
}

/// C: `FUN_0219AE10` (unlabeled routine at `0x0219AE10`): state 0 of [`rroute_move`]: a trainer (event type 0xA) that sees the
/// player in its display dir (`021AE198(m, dir, header param 0, 0)` != -1) starts the acmd `02197E7C(dir, 0x30)` (neither value
/// masked), move bit 0x10 on, state 1; otherwise state 2. Returns 1.
pub fn rroute_sight<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(EVENT_TYPE, &[m]) == 0xA {
        let d = env.callv(DIR_DISP, &[m]);
        let range = env.callv(PARAM, &[m, 0]);
        if env.callv(TRAINER_SEES, &[m, d, range, 0]) != 0xFFFF_FFFF {
            let code = env.callv(crate::mvproc3a::ACMD_CODE_FOR, &[d, 0x30]);
            env.callv(crate::mvproc3a::ACMD_CHANGE, &[m, code]);
            env.callv(WALK_ON, &[m]);
            env.w8(w, 1);
            return 1;
        }
    }
    env.w8(w, 2);
    1
}

/// C: `FUN_0219AE68` (unlabeled routine at `0x0219AE68`): state 1 of [`rroute_move`]: when the acmd is done: the move code's
/// partner in the ROM pairs `0x021D4F90` (either side) becomes the move code (`0216E0E8`, the reverse route) and the leg `s8 +1`
/// becomes `3 - leg` (read before the change, written after); move bit 0x10 off; state 2. Returns 0.
pub fn rroute_reverse<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) != 1 {
        return 0;
    }
    let code = env.callv(MOVE_CODE, &[m]);
    let mut p = ROUTE_PAIRS;
    let mut a = 0x25;
    loop {
        let bb = rom::rom16(p.wrapping_add(2));
        let partner = if a == code {
            bb
        } else if bb == code {
            a
        } else {
            0xFF
        };
        if partner != 0xFF {
            let leg = env.r8(w.wrapping_add(1)) as i8 as i32;
            let nl = 4i32.wrapping_sub(leg.wrapping_add(1));
            env.callv(CHANGE_MOVE_CODE, &[m, partner]);
            env.w8(w.wrapping_add(1), nl as u8);
            break;
        }
        p = p.wrapping_add(4);
        a = rom::rom16(p);
        if a == 0xFF {
            break;
        }
    }
    env.callv(WALK_OFF, &[m]);
    env.w8(w, 2);
    0
}

/// C: `FUN_0219AEC8` (unlabeled routine at `0x0219AEC8`): state 2 of [`rroute_move`]: as [`route_step`] with the signed leg `s8 +1`
/// stepped by [`rroute_next_leg`] (direction `+5`) and the sight counter at `+6`; state 3.
pub fn rroute_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if may_move(env, m, w.wrapping_add(6)) == 0 {
        return 0;
    }
    let leg = |env: &mut E| env.r8(w.wrapping_add(1)) as i8 as i32 as u32;
    if leg(env) == b(env, w, 2) {
        let axis = b(env, w, 3);
        if on_initial_line(env, m, axis) {
            rroute_next_leg(env, w);
        }
    }
    if leg(env) == 3 && on_initial_tile(env, m) {
        env.w8(w.wrapping_add(1), 0);
    }
    let id = b(env, w, 4);
    let t = dir_table(id);
    let mut d = rom32(t.wrapping_add(leg(env).wrapping_mul(4)));
    let mut hit = face_and_hit(env, m, d);
    if hit & 1 != 0 {
        rroute_next_leg(env, w);
        d = rom32(t.wrapping_add(leg(env).wrapping_mul(4)));
        hit = face_and_hit(env, m, d);
    }
    walk_tail(env, m, d, hit, w.wrapping_add(8));
    env.w8(w, 3);
    1
}

/// C: `FUN_0219B030` (unlabeled routine at `0x0219B030`): state 3 of [`rroute_move`]: when the acmd is done, move bit 0x10 off,
/// spin-look end (`+8`), state 0; returns 0.
pub fn rroute_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    walk_wait_tail(env, m, w, w.wrapping_add(8))
}

/// C: `FUN_0219B060` (unlabeled routine at `0x0219B060`): 1 when the pad trigger bits (`0203630C`) have bit 2, else 0.
pub fn pad_trigger<E: FieldEnv>(env: &mut E) -> u32 {
    u32::from(env.callv(PAD_TRIGGER_BITS, &[]) & 2 != 0)
}

/// C: `FUN_0219B074` (unlabeled routine at `0x0219B074`): 1 when the player exists and his acmd code is a fast walk (0x58..0x5B,
/// 0xA3..0xA6; signed compares), else 0.
pub fn player_walks_fast<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    let p = env.callv(FIND_PLAYER, &[sys]);
    if p == 0 {
        return 0;
    }
    let c = env.callv(ACMD_CODE, &[p]) as i32;
    u32::from((0x58..=0x5B).contains(&c) || (0xA3..=0xA6).contains(&c))
}

/// C: `FUN_0219B0C4` (unlabeled routine at `0x0219B0C4`): 1 when the pad held bits (`020362DC`) have bit 2, else 0.
pub fn pad_held<E: FieldEnv>(env: &mut E) -> u32 {
    u32::from(env.callv(PAD_HELD_BITS, &[]) & 2 != 0)
}

/// C: `FUN_0219B0D8` (unlabeled routine at `0x0219B0D8`): the number of ROM words at `table` before `term` (no bound: past the
/// embedded ROM block the port panics, UNVERIFIED).
pub fn table_len(table: u32, term: u32) -> u32 {
    let mut n = 0u32;
    while rom32(table.wrapping_add(n.wrapping_mul(4))) != term {
        n = n.wrapping_add(1);
    }
    n
}

/// C: `FUN_0219B0F0` (unlabeled routine at `0x0219B0F0`): a random entry of the ROM table `table` (ended by `term`).
pub fn random_entry<E: FieldEnv>(env: &mut E, table: u32, term: u32) -> u32 {
    let n = table_len(table, term);
    let i = env.callv(RNG, &[n]);
    rom32(table.wrapping_add(i.wrapping_mul(4)))
}

/// C: `FUN_0219B104` (unlabeled routine at `0x0219B104`): a random dir of the direction set `kind` ([`dir_table`], ended by
/// `term`).
pub fn random_dir<E: FieldEnv>(env: &mut E, kind: u32, term: u32) -> u32 {
    let t = dir_table(kind);
    let n = table_len(t, term);
    let i = env.callv(RNG, &[n]);
    rom32(t.wrapping_add(i.wrapping_mul(4)))
}

/// C: `FUN_0219B120` (unlabeled routine at `0x0219B120`): the ROM address of the direction set / route `kind` from the pairs
/// {id, table} at `0x021D50F8` (id 0x27 ends them); 0 for an unknown id.
pub fn dir_table(kind: u32) -> u32 {
    let mut p = DIR_TABLES;
    let mut id = 0;
    loop {
        if id == kind {
            return rom32(p.wrapping_add(4));
        }
        p = p.wrapping_add(8);
        id = rom32(p);
        if id == 0x27 {
            return 0;
        }
    }
}

/// C: `FUN_0219B13C` (unlabeled routine at `0x0219B13C`): the dir from `m` toward the player when `m` (event type 1 or 2, move code
/// in the ROM list `0x021D4FEC`) sees him walking fast at its grid y within header param 0 tiles (x and z, inclusive), else 9.
pub fn player_dir_in_sight<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let ty = env.callv(EVENT_TYPE, &[m]);
    if ty != 1 && ty != 2 {
        return 9;
    }
    if player_walks_fast(env, m) == 0 {
        return 9;
    }
    let code = env.callv(MOVE_CODE, &[m]);
    let mut i = 0u32;
    let mut c;
    loop {
        c = rom32(SIGHT_CODES.wrapping_add(4 * i));
        i += 1;
        if c == code || c == 0xFF {
            break;
        }
    }
    if code != c {
        return 9;
    }
    let sys = env.callv(SYS_OF, &[m]);
    let p = env.callv(FIND_PLAYER, &[sys]);
    if p == 0 {
        return 9;
    }
    let py = env.callv(GRID_Y, &[p]);
    if py != env.callv(GRID_Y, &[m]) {
        return 9;
    }
    let px = env.callv(GRID_X, &[p]) as i32;
    let pz = env.callv(GRID_Z, &[p]) as i32;
    let r = env.callv(PARAM, &[m, 0]) as i32;
    let x = env.callv(GRID_X, &[m]) as i32;
    let z = env.callv(GRID_Z, &[m]) as i32;
    if z.wrapping_sub(r) > pz
        || z.wrapping_add(r) < pz
        || x.wrapping_sub(r) > px
        || x.wrapping_add(r) < px
    {
        return 9;
    }
    env.callv(DIR_TOWARD, &[x as u32, z as u32, px as u32, pz as u32])
}

/// C: `FUN_0219B200` (unlabeled routine at `0x0219B200`): a dir of the set `kind` toward the player in sight, or 9: sets of one entry
/// -> 9; [`player_dir_in_sight`] when it is in the set; else the x / z dirs from `m` to the player: the one that is set when it is
/// in the set, both set -> the first set entry equal to either.
pub fn sight_dir<E: FieldEnv>(env: &mut E, m: u32, kind: u32, term: u32) -> u32 {
    let t = dir_table(kind);
    let n = table_len(t, term) as i32;
    if n == 1 {
        return 9;
    }
    let d = player_dir_in_sight(env, m);
    if d == 9 {
        return 9;
    }
    let entry = |i: i32| rom32(t.wrapping_add((i as u32).wrapping_mul(4)));
    let mut i = 0;
    loop {
        if d == entry(i) {
            return d;
        }
        i += 1;
        if i >= n {
            break;
        }
    }
    let sys = env.callv(SYS_OF, &[m]);
    let p = env.callv(FIND_PLAYER, &[sys]);
    let x = env.callv(GRID_X, &[m]) as i32;
    let z = env.callv(GRID_Z, &[m]) as i32;
    let px = env.callv(GRID_X, &[p]) as i32;
    let pz = env.callv(GRID_Z, &[p]) as i32;
    let dx = if x > px {
        2
    } else if x < px {
        3
    } else {
        9
    };
    let dz = if z > pz {
        0
    } else if z < pz {
        1
    } else {
        9
    };
    let mut i = 0;
    loop {
        let e = entry(i);
        if dx == 9 {
            if dz == e {
                return dz;
            }
        } else if dz == 9 {
            if dx == e {
                return dx;
            }
        } else {
            if dx == e {
                return dx;
            }
            if dz == e {
                return dz;
            }
        }
        i += 1;
        if i >= n {
            return 9;
        }
    }
}

/// C: `FUN_0219B2E8` (unlabeled routine at `0x0219B2E8`): 1 when `m` may move: always for event types other than 0xC; a watcher
/// (0xC) with its counter `u8 *cnt` at 0 needs the player near and pad bit 2 held (else 0), then `*cnt` = header param 1; a set
/// counter counts down.
pub fn may_move<E: FieldEnv>(env: &mut E, m: u32, cnt: u32) -> u32 {
    if env.callv(EVENT_TYPE, &[m]) != 0xC {
        return 1;
    }
    if env.r8(cnt) == 0 {
        if env.callv(PLAYER_NEAR, &[m]) == 0 {
            return 0;
        }
        if pad_held(env) == 0 {
            return 0;
        }
        let v = env.callv(PARAM, &[m, 1]);
        env.w8(cnt, v as u8);
    }
    let c = env.r8(cnt);
    if c != 0 {
        env.w8(cnt, c.wrapping_sub(1));
    }
    1
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

/// C: `FUN_0219B390` (unlabeled routine at `0x0219B390`): spin-look start: `u8 p[0]` = the display dir; display dir = the next dir
/// after it in the ROM order row `s8 p[1]` (`0x021D4FCC`; not found -> index 4, `(i + 1) % 4`); `p[2]` = 1 when the dir was already
/// locked; lock the dir (status |= 8).
pub fn spin_look_start<E: FieldEnv>(env: &mut E, m: u32, p: u32) {
    let d = env.callv(DIR_DISP, &[m]);
    let row = |env: &mut E| {
        SPIN_LOOK_ORDERS.wrapping_add(((env.r8(p.wrapping_add(1)) as i8 as i32) << 4) as u32)
    };
    let mut i = 0i32;
    while i < 4 {
        let r = row(env);
        if d == rom32(r.wrapping_add(4 * i as u32)) {
            break;
        }
        i += 1;
    }
    env.w8(p, d as u8);
    let j = (i + 1) % 4;
    let r = row(env);
    let e = rom32(r.wrapping_add(4 * j as u32));
    let locked = env.callv(STATUS, &[m, DIR_LOCK]);
    env.w8(p.wrapping_add(2), u8::from(locked != 0));
    env.callv(SET_DIR_DISP, &[m, e & 0xFFFF]);
    env.callv(SET_STATUS, &[m, DIR_LOCK]);
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
