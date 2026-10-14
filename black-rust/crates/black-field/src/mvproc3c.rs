//! `fldmmdl_acmd.c`, part 3c (overlay 21, `0x0219B41C..0x0219BDC4`; F1 row 21.09, F4b job 3): the event-type procs (looking around
//! after some steps, event types 4 / 5 / 6 / 0xE / 0xF / 0x10 / 0x14; reached from mmdl_move through `0219B414` / `0219B41C`), the
//! follow-the-player type (move code 0x30) and the walk-behind-the-partner-trainer type (0x32) with its partner search (also used by
//! scrcmd_obj).
//!
//! F4b-spec3 specced all 38 functions (`tools/rom-diff/specs/black-field/mvproc3c.spec`); F4b-port2 ported all of them. The ROM
//! tables (event-type procs, turn orders) are read through [`crate::mvproc3b::rom`].

use crate::hooks::{CArg, FieldEnv};
use crate::mvproc3a::{
    start_acmd, ACMD_ACTION, ACMD_CHANGE, ACMD_CODE_FOR, BIT40_OFF, DIR_DISP, DIR_DX, DIR_DZ,
    EVENT_TYPE, GRID_X, GRID_Z, MOVE_WORK, MOVE_WORK_INIT, PARAM, PLAYER_NEAR, SET_ANIM,
    SET_DIR_DISP, SUB_WORK, SUB_WORK_INIT, WALK_OFF, WALK_ON,
};
use crate::mvproc3b::rom::rom32;
use crate::mvproc3b::{ACMD_CODE, FIND_PLAYER, SYS_OF};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_B41C,
    0x0219_B440,
    0x0219_B45C,
    0x0219_B478,
    0x0219_B494,
    0x0219_B4C8,
    0x0219_B4FC,
    0x0219_B500,
    0x0219_B504,
    0x0219_B508,
    0x0219_B520,
    0x0219_B59C,
    0x0219_B6B0,
    0x0219_B6E0,
    0x0219_B75C,
    0x0219_B8A8,
    0x0219_B8BC,
    0x0219_B8E8,
    0x0219_B920,
    0x0219_B924,
    0x0219_B968,
    0x0219_B984,
    0x0219_B9B0,
    0x0219_B9D8,
    0x0219_BA0C,
    0x0219_BA2C,
    0x0219_BA6C,
    0x0219_BAE0,
    0x0219_BB08,
    0x0219_BB38,
    0x0219_BB3C,
    0x0219_BB48,
    0x0219_BB84,
    0x0219_BBA4,
    0x0219_BC48,
    0x0219_BCC0,
    0x0219_BCE4,
    0x0219_BD30,
];

/// `FUN_0216D804(m)`: old grid x (s16).
const OLD_X: u32 = 0x0216_D804;
/// `FUN_0216D81C(m)`: old grid z (s16).
const OLD_Z: u32 = 0x0216_D81C;
/// `FUN_0216D590(m)`: the zone id.
const ZONE: u32 = 0x0216_D590;
/// `FUN_0216D57C(m, mask)`: move bits & mask.
const MOVE_BITS: u32 = 0x0216_D57C;
/// `FUN_0216DA5C(m)`: move bit 0x10 (walking) set.
const WALKING: u32 = 0x0216_DA5C;
/// `FUN_0216DAE0(m, on)`: status 0x80 off when `on` == 1, else on.
const STATUS_80: u32 = 0x0216_DAE0;
/// `FUN_0216DD1C(sys, &m, &i)`: the next MMDL in use (1 = `*m` set).
const NEXT_IN_USE: u32 = 0x0216_DD1C;
/// `FUN_021642B4(x0, z0, x1, z1)`: the dir from (x0, z0) to (x1, z1) (mmdl_move).
const DIR_TOWARD: u32 = 0x0216_42B4;
/// `FUN_021AE884(m)`: the trainer id (trainer_eye).
const TRAINER_ID: u32 = 0x021A_E884;
/// `FUN_0203630C()`: ARM9 pad trigger bits.
const PAD_TRIGGER_BITS: u32 = 0x0203_630C;

/// ROM `0x021D5278` / `0x021D52CC` / `0x021D5320`: the event-type init / busy / spin procs (21 entries each).
const EV_INIT: u32 = 0x021D_5278;
const EV_BUSY_PROCS: u32 = 0x021D_52CC;
const EV_SPIN_PROCS: u32 = 0x021D_5320;
/// ROM `0x021D5238`: per display dir the turn-order row of [`ev_steps_spin`] (u32).
const STEPS_ROW: u32 = 0x021D_5238;
/// ROM `0x021D5248`: the 2-dir turn orders of [`ev_steps_spin`] (2 rows of 2 u32).
const STEPS_ORDERS: u32 = 0x021D_5248;
/// ROM `0x021D5258`: the 4-dir turn orders of [`ev_look_spin`] (2 rows of 4 u32).
const LOOK_ORDERS: u32 = 0x021D_5258;
/// ROM `0x021D5374`: the states of [`follow_move`].
pub const FOLLOW_STATES: [u32; 2] = [0x0219_B925, 0x0219_B969];
/// ROM `0x021D538C`: the states of [`partner_move`].
pub const PARTNER_STATES: [u32; 2] = [0x0219_BB49, 0x0219_BB85];

/// Runs the event-type proc at the ROM address `a` (a target of the three event-type tables).
fn ev_proc<E: FieldEnv>(env: &mut E, a: u32, m: u32) -> u32 {
    match a {
        0x0219_B4FD => {
            ev_init_none(env, m);
            0
        }
        0x0219_B509 => {
            ev_steps_init(env, m);
            0
        }
        0x0219_B6B1 => {
            ev_look_init(env, m);
            0
        }
        0x0219_B501 => ev_busy_none(env, m),
        0x0219_B521 => ev_steps_busy(env, m),
        0x0219_B6E1 => ev_look_busy(env, m),
        0x0219_B505 => ev_spin_none(env, m),
        0x0219_B59D => ev_steps_spin(env, m),
        0x0219_B75D => ev_look_spin(env, m),
        _ => panic!("UNVERIFIED: event-type proc {a:#010x} (event type past the ROM tables)"),
    }
}

/// The proc of the event type of `m` in the ROM table `table` (the type is not range-checked).
fn ev_dispatch<E: FieldEnv>(env: &mut E, table: u32, m: u32) -> u32 {
    let t = env.callv(EVENT_TYPE, &[m]);
    let a = rom32(table.wrapping_add(t.wrapping_mul(4)));
    ev_proc(env, a, m)
}

/// C: `FUN_0219B41C` (unlabeled routine at `0x0219B41C`): 1 when the event-type busy query ([`ev_busy_query`]) and spin query
/// ([`ev_spin_query`]) both answer nonzero, else 0 (mmdl_move `02163074` then keeps the move proc from running).
pub fn ev_busy<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if ev_busy_query(env, m) == 0 {
        return 0;
    }
    u32::from(ev_spin_query(env, m) != 0)
}

/// C: `FUN_0219B440` (unlabeled routine at `0x0219B440`): event-type init: ROM `0x021D5278`[event type](m).
pub fn ev_init<E: FieldEnv>(env: &mut E, m: u32) {
    ev_dispatch(env, EV_INIT, m);
}

/// C: `FUN_0219B45C` (unlabeled routine at `0x0219B45C`): event-type busy query: ROM `0x021D52CC`[event type](m).
pub fn ev_busy_query<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    ev_dispatch(env, EV_BUSY_PROCS, m)
}

/// C: `FUN_0219B478` (unlabeled routine at `0x0219B478`): event-type spin query: ROM `0x021D5320`[event type](m).
pub fn ev_spin_query<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    ev_dispatch(env, EV_SPIN_PROCS, m)
}

/// C: `FUN_0219B494` (unlabeled routine at `0x0219B494`): 1 when the grid x or z differs from the old grid x / z (moved), else 0.
pub fn moved<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let x = env.callv(GRID_X, &[m]);
    if x != env.callv(OLD_X, &[m]) {
        return 1;
    }
    let z = env.callv(GRID_Z, &[m]);
    u32::from(z != env.callv(OLD_Z, &[m]))
}

/// C: `FUN_0219B4C8` (unlabeled routine at `0x0219B4C8`): 1 when the grid x and z equal the old grid x / z, else 0.
pub fn standing<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let x = env.callv(GRID_X, &[m]);
    if x != env.callv(OLD_X, &[m]) {
        return 0;
    }
    let z = env.callv(GRID_Z, &[m]);
    u32::from(z == env.callv(OLD_Z, &[m]))
}

/// C: `FUN_0219B4FC` (unlabeled routine at `0x0219B4FC`): event-type init of the plain types: nothing.
pub fn ev_init_none<E: FieldEnv>(_env: &mut E, _m: u32) {}

/// C: `FUN_0219B500` (unlabeled routine at `0x0219B500`): event-type busy query of the plain types: 0.
pub fn ev_busy_none<E: FieldEnv>(_env: &mut E, _m: u32) -> u32 {
    0
}

/// C: `FUN_0219B504` (unlabeled routine at `0x0219B504`): event-type spin query of the plain types: 0.
pub fn ev_spin_none<E: FieldEnv>(_env: &mut E, _m: u32) -> u32 {
    0
}

/// C: `FUN_0219B508` (unlabeled routine at `0x0219B508`): event-type init of types 4 / 0xE: sub work (9 bytes cleared) `u8 +3` =
/// header param 1 (the steps before a look-around).
pub fn ev_steps_init<E: FieldEnv>(env: &mut E, m: u32) {
    let s = env.callv(SUB_WORK_INIT, &[m, 9]);
    let n = env.callv(PARAM, &[m, 1]);
    env.w8(s.wrapping_add(3), n as u8);
}

/// The busy query of the step-counting event types (`0219B520` = `0219B6E0`): state `u8 sub+0`: 0 wait for a step (moved -> 1);
/// 1 count standing frames after it (`s8 +2` up to `s8 +3`, else back to 0); 2 wait until the walk bit is off, then state 3 with
/// `+1` / `+2` cleared; 3 busy (1).
fn steps_busy<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let s = env.callv(SUB_WORK, &[m]);
    let inc = |env: &mut E| {
        let v = env.r8(s).wrapping_add(1);
        env.w8(s, v);
    };
    let mut st = env.r8(s);
    if st == 0 {
        if moved(env, m) == 1 {
            inc(env);
        }
        return 0;
    }
    if st == 1 {
        if standing(env, m) == 0 {
            return 0;
        }
        let c = (env.r8(s.wrapping_add(2)) as i8).wrapping_add(1);
        env.w8(s.wrapping_add(2), c as u8);
        if c < env.r8(s.wrapping_add(3)) as i8 {
            env.w8(s, 0);
            return 0;
        }
        inc(env);
        st = 2;
    }
    if st == 2 {
        if env.callv(WALKING, &[m]) == 1 {
            return 0;
        }
        inc(env);
        env.w8(s.wrapping_add(2), 0);
        env.w8(s.wrapping_add(1), 0);
        return 1;
    }
    u32::from(st == 3)
}

/// C: `FUN_0219B520` (unlabeled routine at `0x0219B520`): event-type busy query of types 4 / 0xE (see `steps_busy`).
pub fn ev_steps_busy<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    steps_busy(env, m)
}

/// C: `FUN_0219B59C` (unlabeled routine at `0x0219B59C`): event-type spin query of types 4 / 0xE: state `u8 sub+1`: 0 store the
/// display dir (`+4`) and its row (ROM `0x021D5238`[dir], `+5`); 1 face the dir of the 2-dir order `0x021D5248`[`s8 +5`][`s8 +6`];
/// 2 wait for the acmd (1 while busy); 3 wait 8 frames (4 for type 0xE with the pad pressed and the player near; counter `s8 +8`),
/// then the next turn (`+6` flips, 4 turns counted in `s8 +7`) or the end: display dir = `s8 +4`, state 4, `+7` = `+0` = 0,
/// returns 0. States above 3 return 0.
pub fn ev_steps_spin<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let s = env.callv(SUB_WORK, &[m]);
    let rd = |env: &mut E, o: u32| env.r8(s.wrapping_add(o)) as i8 as i32;
    let inc_state = |env: &mut E| {
        let v = env.r8(s.wrapping_add(1)).wrapping_add(1);
        env.w8(s.wrapping_add(1), v);
    };
    let mut st = env.r8(s.wrapping_add(1));
    if st > 3 {
        return 0;
    }
    if st == 0 {
        let d = env.callv(DIR_DISP, &[m]);
        env.w8(s.wrapping_add(4), d as u8);
        assert!(
            d < 4,
            "UNVERIFIED: 0219B59C reads past its 4-entry stack copy (display dir {d})"
        );
        let row = rom32(STEPS_ROW + 4 * d);
        env.w8(s.wrapping_add(5), row as u8);
        inc_state(env);
        st = 1;
    }
    if st == 1 {
        let i = rd(env, 5) * 2 + rd(env, 6);
        assert!(
            (0..4).contains(&i),
            "UNVERIFIED: 0219B59C reads past its 4-entry stack copy (index {i})"
        );
        let d = rom32(STEPS_ORDERS + 4 * i as u32);
        start_acmd(env, m, d & 0xFFFF, 0);
        inc_state(env);
        st = 2;
    }
    if st == 2 {
        if env.callv(ACMD_ACTION, &[m]) == 0 {
            return 1;
        }
        inc_state(env);
    }
    let mut limit = 8;
    if env.callv(EVENT_TYPE, &[m]) == 0xE
        && pad_trigger(env) == 1
        && env.callv(PLAYER_NEAR, &[m]) == 1
    {
        limit = 4;
    }
    spin_wait_tail(env, m, s, limit, 2)
}

/// The end of the spin queries' state 3: count `s8 sub+8` to `limit` (1 below it); then `+8` = 0 and `s8 +7` += 1: below 4 the
/// next turn (`+6` = (`s8 +6` + 1) mod `turns`, state 1, returns 1); else display dir = `s8 +4`, state + 1, `+7` = `+0` = 0,
/// returns 0.
fn spin_wait_tail<E: FieldEnv>(env: &mut E, m: u32, s: u32, limit: i32, turns: i32) -> u32 {
    let c = (env.r8(s.wrapping_add(8)) as i8).wrapping_add(1);
    env.w8(s.wrapping_add(8), c as u8);
    if i32::from(c) < limit {
        return 1;
    }
    env.w8(s.wrapping_add(8), 0);
    let k = (env.r8(s.wrapping_add(7)) as i8).wrapping_add(1);
    env.w8(s.wrapping_add(7), k as u8);
    if k < 4 {
        let n = i32::from(env.r8(s.wrapping_add(6)) as i8) + 1;
        let n = if turns == 2 { n & 1 } else { n % 4 };
        env.w8(s.wrapping_add(6), n as u8);
        env.w8(s.wrapping_add(1), 1);
        return 1;
    }
    let d = env.r8(s.wrapping_add(4)) as i8 as i32 as u32;
    env.callv(SET_DIR_DISP, &[m, d]);
    let v = env.r8(s.wrapping_add(1)).wrapping_add(1);
    env.w8(s.wrapping_add(1), v);
    env.w8(s.wrapping_add(7), 0);
    env.w8(s, 0);
    0
}

/// C: `FUN_0219B6B0` (unlabeled routine at `0x0219B6B0`): event-type init of types 5 / 6 / 0xF / 0x10 / 0x14: sub work (9 bytes
/// cleared) `+3` = header param 1; `u8 +5` = 0 for types 5 / 0x14 / 0xF, else 1 (the turn-order row).
pub fn ev_look_init<E: FieldEnv>(env: &mut E, m: u32) {
    let s = env.callv(SUB_WORK_INIT, &[m, 9]);
    let n = env.callv(PARAM, &[m, 1]);
    env.w8(s.wrapping_add(3), n as u8);
    let t = env.callv(EVENT_TYPE, &[m]);
    env.w8(
        s.wrapping_add(5),
        u8::from(!(t == 5 || t == 0x14 || t == 0xF)),
    );
}

/// C: `FUN_0219B6E0` (unlabeled routine at `0x0219B6E0`): event-type busy query of types 5 / 6 / 0xF / 0x10 / 0x14 (the same code
/// as [`ev_steps_busy`]).
pub fn ev_look_busy<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    steps_busy(env, m)
}

/// C: `FUN_0219B75C` (unlabeled routine at `0x0219B75C`): event-type spin query of types 5 / 6 / 0xF / 0x10 / 0x14: as
/// [`ev_steps_spin`] with the 4-dir rows of ROM `0x021D5258` (row `s8 sub+5`): state 0 stores the display dir and `+6` = (its index
/// in the row + 1) % 4 (not found: 4 -> 1); state 3 shortens the wait to 4 for types 0xF / 0x10 and steps `+6` modulo 4.
pub fn ev_look_spin<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let s = env.callv(SUB_WORK, &[m]);
    let rd = |env: &mut E, o: u32| env.r8(s.wrapping_add(o)) as i8 as i32;
    let inc_state = |env: &mut E| {
        let v = env.r8(s.wrapping_add(1)).wrapping_add(1);
        env.w8(s.wrapping_add(1), v);
    };
    let entry = |row: i32, i: i32| {
        let k = row * 4 + i;
        assert!(
            (0..8).contains(&k),
            "UNVERIFIED: 0219B75C reads past its 8-entry stack copy (index {k})"
        );
        rom32(LOOK_ORDERS + 4 * k as u32)
    };
    let mut st = env.r8(s.wrapping_add(1));
    if st > 3 {
        return 0;
    }
    if st == 0 {
        let d = env.callv(DIR_DISP, &[m]);
        let mut i = 0;
        while i < 4 {
            let row = rd(env, 5);
            if d == entry(row, i) {
                break;
            }
            i += 1;
        }
        env.w8(s.wrapping_add(4), d as u8);
        env.w8(s.wrapping_add(6), ((i + 1) % 4) as u8);
        inc_state(env);
        st = 1;
    }
    if st == 1 {
        let d = entry(rd(env, 5), rd(env, 6));
        start_acmd(env, m, d & 0xFFFF, 0);
        inc_state(env);
        st = 2;
    }
    if st == 2 {
        if env.callv(ACMD_ACTION, &[m]) == 0 {
            return 1;
        }
        inc_state(env);
    }
    let mut limit = 8;
    let t = env.callv(EVENT_TYPE, &[m]);
    if (t.wrapping_add(0xFFF1) & 0xFFFF) <= 1
        && pad_trigger(env) == 1
        && env.callv(PLAYER_NEAR, &[m]) == 1
    {
        limit = 4;
    }
    spin_wait_tail(env, m, s, limit, 4)
}

/// C: `FUN_0219B8A8` (unlabeled routine at `0x0219B8A8`): 1 when the pad trigger bits (`0203630C`) have bit 2, else 0.
pub fn pad_trigger<E: FieldEnv>(env: &mut E) -> u32 {
    u32::from(env.callv(PAD_TRIGGER_BITS, &[]) & 2 != 0)
}

/// C: `FUN_0219B8BC` (unlabeled routine at `0x0219B8BC`): move code 0x30 init (follow the player): move work (8 bytes cleared),
/// [`follow_check`]; draw animation code 0; move bit 0x10 off; status 0x80 on (`0216DAE0(m, 0)`).
pub fn follow_init<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 8]);
    follow_check(env, m, w);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(WALK_OFF, &[m]);
    env.callv(STATUS_80, &[m, 0]);
}

/// C: `FUN_0219B8E8` (unlabeled routine at `0x0219B8E8`): move proc of code 0x30: with a player ([`follow_check`]) status 0x80 on
/// and [`FOLLOW_STATES`]`[u8 +0]` while it answers 1.
pub fn follow_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    if follow_check(env, m, w) == 0 {
        return;
    }
    env.callv(STATUS_80, &[m, 0]);
    loop {
        let r = match env.r8(w) {
            0 => follow_step(env, m, w),
            1 => follow_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219B8E8 state {s} past the ROM table 0x021D5374"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219B920` (unlabeled routine at `0x0219B920`): delete proc of code 0x30: nothing.
pub fn follow_delete<E: FieldEnv>(_env: &mut E, _m: u32) {}

/// C: `FUN_0219B924` (unlabeled routine at `0x0219B924`): state 0 of [`follow_move`]: move bits 0x10 / 0x40 off; when the player
/// left the recorded tile, record his tile again and, if a step toward his old tile starts, move bit 0x10 on, state + 1, returns 1;
/// else 0.
pub fn follow_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    env.callv(WALK_OFF, &[m]);
    env.callv(BIT40_OFF, &[m]);
    if player_moved(env, m, w) == 1 {
        follow_record(env, m, w);
        if follow_start_step(env, m) == 1 {
            env.callv(WALK_ON, &[m]);
            let s = env.r8(w).wrapping_add(1);
            env.w8(w, s);
            return 1;
        }
    }
    0
}

/// C: `FUN_0219B968` (unlabeled routine at `0x0219B968`): state 1 of [`follow_move`]: when the acmd is done (1), move bit 0x10 off
/// and state 0; returns 0.
pub fn follow_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 1 {
        env.callv(WALK_OFF, &[m]);
        env.w8(w, 0);
    }
    0
}

/// The player MMDL of `m`'s MMDLSYS, or 0.
fn player_of<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    env.callv(FIND_PLAYER, &[sys])
}

/// C: `FUN_0219B984` (unlabeled routine at `0x0219B984`): 0 (with `u8 w+1` = 0) when there is no player; else, the first time
/// (`+1` == 0), [`follow_start`]; returns 1.
pub fn follow_check<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if player_of(env, m) == 0 {
        env.w8(w.wrapping_add(1), 0);
        return 0;
    }
    if env.r8(w.wrapping_add(1)) == 0 {
        follow_start(env, m, w);
    }
    1
}

/// C: `FUN_0219B9B0` (unlabeled routine at `0x0219B9B0`): `u8 w+1` = 1, `s16 w+2` / `w+4` = the player's grid x / z, `u16 w+6` =
/// 0xFF.
pub fn follow_start<E: FieldEnv>(env: &mut E, m: u32, w: u32) {
    let p = player_of(env, m);
    env.w8(w.wrapping_add(1), 1);
    let x = env.callv(GRID_X, &[p]);
    env.w16(w.wrapping_add(2), x as u16);
    let z = env.callv(GRID_Z, &[p]);
    env.w16(w.wrapping_add(4), z as u16);
    env.w16(w.wrapping_add(6), 0xFF);
}

/// C: `FUN_0219B9D8` (unlabeled routine at `0x0219B9D8`): 1 when the player exists and his grid x / z differ from `s16 w+2` /
/// `w+4`, else 0.
pub fn player_moved<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let p = player_of(env, m);
    if p == 0 {
        return 0;
    }
    let x = env.callv(GRID_X, &[p]);
    let z = env.callv(GRID_Z, &[p]);
    let rx = env.r16(w.wrapping_add(2)) as i16 as i32 as u32;
    if x != rx {
        return 1;
    }
    let rz = env.r16(w.wrapping_add(4)) as i16 as i32 as u32;
    u32::from(z != rz)
}

/// C: `FUN_0219BA0C` (unlabeled routine at `0x0219BA0C`): `s16 w+2` / `w+4` = the player's grid x / z (no null check).
pub fn follow_record<E: FieldEnv>(env: &mut E, m: u32, w: u32) {
    let p = player_of(env, m);
    let x = env.callv(GRID_X, &[p]);
    env.w16(w.wrapping_add(2), x as u16);
    let z = env.callv(GRID_Z, &[p]);
    env.w16(w.wrapping_add(4), z as u16);
}

/// C: `FUN_0219BA2C` (unlabeled routine at `0x0219BA2C`): the acmd base of the follower: its own acmd code mapped 0x58..0x5B ->
/// 0x10..0x13, any other code as it is (the player lookup it makes is discarded).
pub fn follower_base<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    env.callv(FIND_PLAYER, &[sys]);
    let c = env.callv(ACMD_CODE, &[m]);
    match c.wrapping_sub(0x58) {
        0..=3 => c - 0x48,
        _ => c,
    }
}

/// C: `FUN_0219BA6C` (unlabeled routine at `0x0219BA6C`): 1 when a step toward the player's old tile started (acmd change to
/// `02197E7C(021642B4(m.x, m.z, p.oldx, p.oldz), `[`follower_base`]`)`); 0 when `m` already stands there.
pub fn follow_start_step<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let p = player_of(env, m);
    let x = env.callv(GRID_X, &[m]);
    let z = env.callv(GRID_Z, &[m]);
    let ox = env.callv(OLD_X, &[p]);
    let oz = env.callv(OLD_Z, &[p]);
    if x == ox && z == oz {
        return 0;
    }
    let base = follower_base(env, m);
    let d = env.callv(DIR_TOWARD, &[x, z, ox, oz]);
    start_acmd(env, m, d & 0xFFFF, base & 0xFFFF);
    1
}

/// C: `FUN_0219BAE0` (unlabeled routine at `0x0219BAE0`): move code 0x32 init (walk behind the partner trainer): move work (0xC
/// bytes cleared), [`partner_check`]; draw animation code 0; move bit 0x10 off; `u8 +1` = 0.
pub fn partner_init<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 0xC]);
    partner_check(env, m, w);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(WALK_OFF, &[m]);
    env.w8(w.wrapping_add(1), 0);
}

/// C: `FUN_0219BB08` (unlabeled routine at `0x0219BB08`): move proc of code 0x32: with a partner ([`partner_check`])
/// [`PARTNER_STATES`]`[u8 +0]` while it answers 1.
pub fn partner_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    if partner_check(env, m, w) == 0 {
        return;
    }
    loop {
        let r = match env.r8(w) {
            0 => partner_step(env, m, w),
            1 => partner_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219BB08 state {s} past the ROM table 0x021D538C"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219BB38` (unlabeled routine at `0x0219BB38`): delete proc of code 0x32: nothing.
pub fn partner_delete<E: FieldEnv>(_env: &mut E, _m: u32) {}

/// C: `FUN_0219BB3C` (unlabeled routine at `0x0219BB3C`): return proc of code 0x32: `u8 (move work)+1` = 0 (look for the partner
/// again).
pub fn partner_return<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    env.w8(w.wrapping_add(1), 0);
}

/// C: `FUN_0219BB48` (unlabeled routine at `0x0219BB48`): state 0 of [`partner_move`]: move bits 0x10 / 0x40 off; when it is time
/// to follow and a step started, move bit 0x10 on, state + 1, returns 1; else 0.
pub fn partner_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    env.callv(WALK_OFF, &[m]);
    env.callv(BIT40_OFF, &[m]);
    if partner_due(env, m, w) == 1 && partner_start_step(env, m, w) == 1 {
        env.callv(WALK_ON, &[m]);
        let s = env.r8(w).wrapping_add(1);
        env.w8(w, s);
        return 1;
    }
    0
}

/// C: `FUN_0219BB84` (unlabeled routine at `0x0219BB84`): state 1 of [`partner_move`]: when the acmd is done (nonzero), move bit
/// 0x10 off and state 0; returns 0.
pub fn partner_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 0 {
        return 0;
    }
    env.callv(WALK_OFF, &[m]);
    env.w8(w, 0);
    0
}

/// The `while (FUN_0216DD1C(sys, &o, &i))` search of [`find_partner`] / [`partner_check`]: the first MMDL in use other than `m`
/// with zone id `zone` and trainer id `tid`, or `None`. The original's stack words `o` / `i` are a [`crate::hooks::Mem`] local.
fn partner_search<E: FieldEnv>(env: &mut E, m: u32, sys: u32, zone: u32, tid: u32) -> Option<u32> {
    let loc = env.mem().push_local(8);
    let (o_at, i_at) = (loc, loc + 4);
    let mut found = None;
    while env.call(
        NEXT_IN_USE,
        &mut [CArg::V(sys), CArg::Out(o_at, 4), CArg::P(i_at)],
    ) == 1
    {
        let o = env.r32(o_at);
        if m != o && zone == env.callv(ZONE, &[o]) && tid == env.callv(TRAINER_ID, &[o]) {
            found = Some(o);
            break;
        }
    }
    env.mem().pop_local(loc);
    found
}

/// C: `FUN_0219BBA4` (unlabeled routine at `0x0219BBA4`): the partner MMDL of a double-battle trainer (event type 1..8 or 0x14):
/// another MMDL in use with the same zone id and trainer id; else 0.
pub fn find_partner<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let t = env.callv(EVENT_TYPE, &[m]);
    let zone = env.callv(ZONE, &[m]);
    let sys = env.callv(SYS_OF, &[m]);
    let tid = env.callv(TRAINER_ID, &[m]);
    if !((1..=8).contains(&t) || t == 0x14) {
        return 0;
    }
    partner_search(env, m, sys, zone, tid).unwrap_or(0)
}

/// C: `FUN_0219BC48` (unlabeled routine at `0x0219BC48`): 1 when the partner (the search of [`find_partner`] without the event-type
/// test) exists, with [`partner_record`] the first time (`u8 w+1` == 0); none: `w+1` = 0, returns 0.
pub fn partner_check<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    let zone = env.callv(ZONE, &[m]);
    let tid = env.callv(TRAINER_ID, &[m]);
    match partner_search(env, m, sys, zone, tid) {
        Some(o) => {
            if env.r8(w.wrapping_add(1)) == 0 {
                partner_record(env, m, w, o);
            }
            1
        }
        None => {
            env.w8(w.wrapping_add(1), 0);
            0
        }
    }
}

/// C: `FUN_0219BCC0` (unlabeled routine at `0x0219BCC0`): `u8 w+1` = 1, `s16 w+2` / `w+4` = `o`'s grid x / z, `u16 w+6` = 0xFF,
/// `u32 w+8` = `o`.
pub fn partner_record<E: FieldEnv>(env: &mut E, _m: u32, w: u32, o: u32) {
    env.w8(w.wrapping_add(1), 1);
    let x = env.callv(GRID_X, &[o]);
    env.w16(w.wrapping_add(2), x as u16);
    let z = env.callv(GRID_Z, &[o]);
    env.w16(w.wrapping_add(4), z as u16);
    env.w16(w.wrapping_add(6), 0xFF);
    env.w32(w.wrapping_add(8), o);
}

/// C: `FUN_0219BCE4` (unlabeled routine at `0x0219BCE4`): 1 when it is time to follow the partner `o` (`u32 w+8`): not when `m`
/// stands on `o`'s old tile; when `o` is walking or its move bits & 0x1808 are clear.
pub fn partner_due<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let o = env.r32(w.wrapping_add(8));
    let x = env.callv(GRID_X, &[m]);
    let z = env.callv(GRID_Z, &[m]);
    let ox = env.callv(OLD_X, &[o]);
    let oz = env.callv(OLD_Z, &[o]);
    if x == ox && z == oz {
        return 0;
    }
    if env.callv(WALKING, &[o]) == 1 {
        return 1;
    }
    u32::from(env.callv(MOVE_BITS, &[o, 0x1808]) == 0)
}

/// C: `FUN_0219BD30` (unlabeled routine at `0x0219BD30`): 1 when a step toward the partner `o` (`u32 w+8`) started: not when `m`
/// stands on `o`'s tile or the step toward `o`'s old tile would land on `o`'s tile; else acmd change to `02197E7C(d, 0xC)`.
pub fn partner_start_step<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let x = env.callv(GRID_X, &[m]);
    let z = env.callv(GRID_Z, &[m]);
    let o = env.r32(w.wrapping_add(8));
    let px = env.callv(GRID_X, &[o]);
    let o = env.r32(w.wrapping_add(8));
    let pz = env.callv(GRID_Z, &[o]);
    let o = env.r32(w.wrapping_add(8));
    let ox = env.callv(OLD_X, &[o]);
    let o = env.r32(w.wrapping_add(8));
    let oz = env.callv(OLD_Z, &[o]);
    if x == px && z == pz {
        return 0;
    }
    let d = env.callv(DIR_TOWARD, &[x, z, ox, oz]);
    let nx = x.wrapping_add(env.callv(DIR_DX, &[d & 0xFFFF]));
    let nz = z.wrapping_add(env.callv(DIR_DZ, &[d & 0xFFFF]));
    if nx == px && nz == pz {
        return 0;
    }
    let code = env.callv(ACMD_CODE_FOR, &[d & 0xFFFF, 0xC]);
    env.callv(ACMD_CHANGE, &[m, code & 0xFFFF]);
    1
}

#[cfg(test)]
mod tests;
