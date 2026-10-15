//! The field PROCLINK event, the first part of `event_field_proclink.c` (overlay 10, `0x0216075C..0x02160A92`; F14b).
//!
//! F1 gave this range to row 10.21 (`event_shortcut_menu.c`), but it is the code that reads the PROCLINK table
//! ([`TABLE`], ROM `0x021712CC`, which `proclink.rs` (F14) ports the callbacks of): `__FILE__` strings put the shortcut
//! menu's only allocation (`"event_shortcut_menu.c"`, string `0x02172960`) in `0215FF54`, the callers of the shared
//! helpers `0216070C` / `021606DC` and the shortcut-menu calls (`021C4F7C`, `021C51A0`, `021C5334`, `021C5380`) end at
//! `0x02160758`, and the proclink file's first allocation is `02160A94`; the event constructor at `0x0216075C` is
//! the first function that builds the event the table's callbacks run in. The file boundary is therefore `0x0216075C`
//! (row 10.21 = `0215FF54..0216075A`, row 10.22 = `0216075C..02161D96`).
//!
//! The event launches the app named by the table entry `pl+4` (a sub-proc through `020120B4`), reads the result back
//! through the entry's `return` callback and loops (kind 4 = switch to the app `pl+0xC`). The apps themselves (U-phase)
//! and every system callee are hooks reached through [`FieldEnv::call`] by address (D3).
//!
//! Layouts (as read here). `S` (the argument of [`new`]): `+0` GAMESYS, `+4` FIELDMAP, `+8` parent event, `+0xC` first
//! app index, `+0x10` mode, `+0x14` / `+0x18` / `+0x1C` callbacks A / B / C of the callback record, `+0x20` its
//! argument, `+0x24` the event's result (0..3). `PL` (the 0x74-byte GMEVENT work): `+0` u16 id, `+4` current app index,
//! `+8` previous app index, `+0xC` next app index, `+0x10` kind (4 at the start), `+0x14` the GMEVENT, `+0x18` `S`,
//! `+0x1C` the running app's parameter block, `+0x20` the callback record (0x14 bytes `{cbA, cbB, cbC, arg, owner}`),
//! `+0x34` 0x18 bytes filled by `02164480`, `+0x4C` 0x14 bytes filled by `0215E840`, `+0x60` u8, `+0x62` u16, `+0x68`
//! u32 (the fields `proclink.rs` reads).
//!
//! Table entry (0x18 bytes): `+0` overlay, `+4` proc data (0 = no app to run), `+8` `init(pl, mode, prev kind, prev
//! param) -> param`, `+0xC` `return(pl, param) -> kind`, `+0x10` `special(pl, mode)` (0 = none), `+0x14`
//! `free(param)`.
//!
//! Verified by `tools/check-fn.sh black-field proclink_event --per-function` (spec
//! `tools/rom-diff/specs/black-field/proclink_event.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_075C,
    0x0216_07E0,
    0x0216_0A28,
    0x0216_0A4C,
    0x0216_0A64,
    0x0216_0A7C,
];

/// The PROCLINK table (ROM, 12 entries of [`ENTRY`] bytes).
pub const TABLE: u32 = 0x0217_12CC;
/// Size of a table entry.
pub const ENTRY: u32 = 0x18;
/// The event proc ([`proc_`], Thumb bit set) handed to the GMEVENT.
const PROC: u32 = 0x0216_07E1;

/// C: `FUN_020122C0(gsys, parent, proc, work size)` / `FUN_020124E8(event)`: GMEVENT create / work.
const EVENT_NEW: u32 = 0x0201_22C0;
const EVENT_WORK: u32 = 0x0201_24E8;
/// C: `FUN_02012374(event, sub)`: run the GMEVENT `sub` as a child of `event`.
const EVENT_CALL: u32 = 0x0201_2374;
/// C: `FUN_020120B4(gs, overlay, proc data, param)` / `FUN_020120F0(gs)`: run an app sub-proc / it is still running.
const APP_RUN: u32 = 0x0201_20B4;
const APP_BUSY: u32 = 0x0201_20F0;
/// C: `FUN_02164480(out 0x18, gs, fm)` / `FUN_0215E840(fm, out 0x14)`: field state snapshots stored in `PL`.
const SNAP_A: u32 = 0x0216_4480;
const SNAP_B: u32 = 0x0215_E840;

/// The word `off` of the table entry `idx`.
fn entry<E: FieldEnv>(env: &mut E, idx: u32, off: u32) -> u32 {
    env.r32(
        TABLE
            .wrapping_add(idx.wrapping_mul(ENTRY))
            .wrapping_add(off),
    )
}

/// Sets the next state.
fn goto<E: FieldEnv>(env: &mut E, seq: u32, state: u32) -> u32 {
    env.w32(seq, state);
    0
}

/// C: `FUN_0216075C(S, id)`: the event: a GMEVENT (proc [`proc_`], work `PL` of 0x74 bytes) with kind 4, all three app
/// indices `S+0xC`, the callback record from `S+0x14..0x20` and the two snapshots; returns the GMEVENT.
pub fn new<E: FieldEnv>(env: &mut E, s: u32, id: u32) -> u32 {
    let gs = env.r32(s);
    let parent = env.r32(s + 8);
    let ev = env.callv(EVENT_NEW, &[gs, parent, PROC, 0x74]);
    let pl = env.callv(EVENT_WORK, &[ev]);
    env.mem().fill(pl, 0x74, 0);
    env.w16(pl, id as u16);
    env.w32(pl + 0x18, s);
    env.w32(pl + 0x10, 4);
    env.w32(pl + 0x14, ev);
    let first = env.r32(s + 0xC);
    env.w32(pl + 4, first);
    let first = env.r32(s + 0xC);
    env.w32(pl + 8, first);
    let first = env.r32(s + 0xC);
    env.w32(pl + 0xC, first);
    env.w8(pl + 0x60, 0);
    env.w16(pl + 0x62, 0);
    env.w32(pl + 0x68, 0);
    let s = env.r32(pl + 0x18);
    let (a, b) = (env.r32(s + 0x14), env.r32(s + 0x18));
    let (c, d) = (env.r32(s + 0x1C), env.r32(s + 0x20));
    rec_init(env, pl + 0x20, s, a, b, c, d);
    let s = env.r32(pl + 0x18);
    let (gs, fm) = (env.r32(s), env.r32(s + 4));
    env.call(
        SNAP_A,
        &mut [CArg::Out(pl + 0x34, 0x18), CArg::V(gs), CArg::V(fm)],
    );
    let s = env.r32(pl + 0x18);
    let fm = env.r32(s + 4);
    env.call(SNAP_B, &mut [CArg::V(fm), CArg::Out(pl + 0x4C, 0x14)]);
    ev
}

/// C: `sub_021607E0(event, seq, pl)`: the event proc, a state machine on `*seq`. 0 start, 1 dispatch on the kind
/// (`pl+0x10`: 0 / 1 plain results, 2 / 3 results 2 / 3, 4 = switch: entries with a `special` callback go to 3, the
/// others to 6), 2 write the result and finish, 3 run `special`, 4 / 5 wait for the record (callbacks B / A), 6 / 7
/// child events (`021BE8D4`, `020144DC`), 8 / 9 / 0xA the field teardown (`020145CC`, `0201210C` +
/// `02188DA0` / `02188DB0`, `021BE92C`), 0xB build the app's parameter (`init`) and run its proc, 0xC wait for it, 0xD
/// read the result (`return`) and either switch apps (kind 4: `pl+8 = pl+4`, `pl+4 = pl+0xC`) or free the parameter
/// and tear down (8). Returns 1 when the event is done.
pub fn proc_<E: FieldEnv>(env: &mut E, ev: u32, seq: u32, pl: u32) -> u32 {
    let state = env.r32(seq);
    match state {
        0 => goto(env, seq, 1),
        1 => {
            let kind = env.r32(pl + 0x10);
            if kind > 1 {
                if kind == 4 {
                    let idx = env.r32(pl + 4);
                    let special = entry(env, idx, 0x10);
                    return goto(env, seq, if special == 0 { 6 } else { 3 });
                }
                if kind != 3 {
                    if kind != 2 {
                        return 0;
                    }
                    let s = env.r32(pl + 0x18);
                    env.w32(s + 0x24, 2);
                    return goto(env, seq, 2);
                }
                let s = env.r32(pl + 0x18);
                env.w32(s + 0x24, 3);
            }
            goto(env, seq, 2)
        }
        2 => {
            let kind = env.r32(pl + 0x10);
            if kind == 0 || kind == 1 {
                let s = env.r32(pl + 0x18);
                env.w32(s + 0x24, kind);
            }
            1
        }
        3 => {
            if rec_b(env, pl + 0x20) == 0 {
                return 0;
            }
            let idx = env.r32(pl + 4);
            let f = entry(env, idx, 0x10);
            let s = env.r32(pl + 0x18);
            let mode = env.r32(s + 0x10);
            env.callv(f & !1, &[pl, mode]);
            goto(env, seq, 4)
        }
        4 => goto(env, seq, 5),
        5 => {
            if env.r32(pl + 0x10) == 0 && rec_a(env, pl + 0x20) == 0 {
                return 0;
            }
            goto(env, seq, 1)
        }
        6 => {
            let s = env.r32(pl + 0x18);
            let (gs, fm) = (env.r32(s), env.r32(s + 4));
            let sub = env.callv(0x021B_E8D4, &[gs, fm, 0, 0]);
            env.callv(EVENT_CALL, &[ev, sub]);
            goto(env, seq, 7)
        }
        7 => {
            if rec_b(env, pl + 0x20) == 0 {
                return 0;
            }
            let s = env.r32(pl + 0x18);
            let (gs, fm) = (env.r32(s), env.r32(s + 4));
            let sub = env.callv(0x0201_44DC, &[gs, fm]);
            env.callv(EVENT_CALL, &[ev, sub]);
            goto(env, seq, 0xB)
        }
        8 => {
            rec_c(env, pl + 0x20);
            let s = env.r32(pl + 0x18);
            let gs = env.r32(s);
            let sub = env.callv(0x0201_45CC, &[gs]);
            env.callv(EVENT_CALL, &[ev, sub]);
            goto(env, seq, 9)
        }
        9 => {
            let s = env.r32(pl + 0x18);
            let gs = env.r32(s);
            let fm = env.callv(0x0201_210C, &[gs]);
            let s = env.r32(pl + 0x18);
            env.w32(s + 4, fm);
            env.callv(0x0218_8DA0, &[]);
            let s = env.r32(pl + 0x18);
            let fm = env.r32(s + 4);
            env.callv(0x0218_8DB0, &[fm]);
            goto(env, seq, 0xA)
        }
        0xA => {
            if rec_a(env, pl + 0x20) == 0 {
                return 0;
            }
            let s = env.r32(pl + 0x18);
            let (gs, fm) = (env.r32(s), env.r32(s + 4));
            let sub = env.callv(0x021B_E92C, &[gs, fm, 0, 0, 0, 0, 0]);
            env.callv(EVENT_CALL, &[ev, sub]);
            goto(env, seq, 1)
        }
        0xB => {
            let idx = env.r32(pl + 4);
            let f = entry(env, idx, 0x08);
            let s = env.r32(pl + 0x18);
            let mode = env.r32(s + 0x10);
            let prev = env.r32(pl + 8);
            let old = env.r32(pl + 0x1C);
            let param = env.callv(f & !1, &[pl, mode, prev, old]);
            let old = env.r32(pl + 0x1C);
            if old != 0 {
                let prev = env.r32(pl + 8);
                let free = entry(env, prev, 0x14);
                env.callv(free & !1, &[old]);
                env.w32(pl + 0x1C, 0);
            }
            let idx = env.r32(pl + 4);
            env.w32(pl + 0x1C, param);
            let procdata = entry(env, idx, 0x04);
            if procdata == 0 {
                return 0;
            }
            let s = env.r32(pl + 0x18);
            let gs = env.r32(s);
            let overlay = entry(env, idx, 0);
            env.callv(APP_RUN, &[gs, overlay, procdata, param]);
            goto(env, seq, 0xC)
        }
        0xC => {
            let s = env.r32(pl + 0x18);
            let gs = env.r32(s);
            if env.callv(APP_BUSY, &[gs]) != 0 {
                return 0;
            }
            goto(env, seq, 0xD)
        }
        0xD => {
            let idx = env.r32(pl + 4);
            let f = entry(env, idx, 0x0C);
            let param = env.r32(pl + 0x1C);
            let res = env.callv(f & !1, &[pl, param]);
            env.w32(pl + 0x10, res);
            if res <= 3 {
                let s = env.r32(pl + 0x18);
                env.w32(s + 0x24, res);
            }
            if env.r32(pl + 0x10) == 4 {
                let cur = env.r32(pl + 4);
                env.w32(pl + 8, cur);
                let next = env.r32(pl + 0xC);
                env.w32(pl + 4, next);
                return goto(env, seq, 0xB);
            }
            let param = env.r32(pl + 0x1C);
            if param != 0 {
                env.callv(HEAP_FREE, &[param]);
                env.w32(pl + 0x1C, 0);
            }
            goto(env, seq, 8)
        }
        _ => 0,
    }
}

/// C: `FUN_02160A28(rec, owner, cbA, cbB, cbC, arg)`: the 0x14-byte callback record `{cbA, cbB, cbC, arg, owner}`
/// (zeroed first; `cbC` and `arg` are stack words).
pub fn rec_init<E: FieldEnv>(
    env: &mut E,
    rec: u32,
    owner: u32,
    cb_a: u32,
    cb_b: u32,
    cb_c: u32,
    arg: u32,
) {
    env.mem().fill(rec, 0x14, 0);
    env.w32(rec, cb_a);
    env.w32(rec + 8, cb_c);
    env.w32(rec + 4, cb_b);
    env.w32(rec + 0xC, arg);
    env.w32(rec + 0x10, owner);
}

/// The callback `rec+ofs` with `(rec+0x10, rec+0xC)`, or 1 when there is none.
fn rec_call<E: FieldEnv>(env: &mut E, rec: u32, ofs: u32) -> u32 {
    let f = env.r32(rec + ofs);
    if f == 0 {
        return 1;
    }
    let (owner, arg) = (env.r32(rec + 0x10), env.r32(rec + 0xC));
    env.callv(f & !1, &[owner, arg])
}

/// C: `FUN_02160A4C(rec)`: callback A of the record (1 when none).
pub fn rec_a<E: FieldEnv>(env: &mut E, rec: u32) -> u32 {
    rec_call(env, rec, 0)
}

/// C: `FUN_02160A64(rec)`: callback B of the record (1 when none).
pub fn rec_b<E: FieldEnv>(env: &mut E, rec: u32) -> u32 {
    rec_call(env, rec, 4)
}

/// C: `FUN_02160A7C(rec)`: callback C of the record (1 when none).
pub fn rec_c<E: FieldEnv>(env: &mut E, rec: u32) -> u32 {
    rec_call(env, rec, 8)
}

#[cfg(test)]
mod tests;
