//! `fieldmap.c`, second part (overlay 21, `0x02189BD0..0x0218B752`): the field event checks (talk, step,
//! push, bump, signs, warps, hidden items, special tiles) and the 0x40-byte EVENT REQUEST they share.
//!
//! F2a-spec specced all 52 functions (`tools/rom-diff/specs/black-field/fieldmap_event.spec`, whose header
//! documents the EVENT REQUEST); F2a ported them. Records are read through the [`Mem`] window of the
//! environment at their original addresses; a request the original builds on its stack is a
//! [`Mem::push_local`] window. Every callee outside the area (the FIELDMAP accessors of `fieldmap`, the player,
//! MMDL, event-data, script, party, save and comm subsystems) goes through [`FieldEnv::call`]; what each one
//! does is only what the comment at the call says.
//!
//! EVENT REQUEST (built by [`request_init`]): `+0x00 u16` heap, `+0x04` gamesys, `+0x08` GAMEDATA, `+0x0C` zone
//! event data, `+0x10 u16` zone, `+0x14` FIELDMAP, `+0x18` player, `+0x1C` move state, `+0x20`, `+0x24 u16` facing,
//! `+0x26 u16` pending script, `+0x28` keys held, `+0x2C` keys triggered, `+0x30 u16` trigger direction, `+0x34` ->
//! position (fx32 x, y, z), `+0x38`, `+0x3C` FLAGS (bits in [`request_init`]).

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_9BD0,
    0x0218_A074,
    0x0218_A0E4,
    0x0218_A240,
    0x0218_A278,
    0x0218_A4BC,
    0x0218_A4E0,
    0x0218_A6E8,
    0x0218_A728,
    0x0218_A738,
    0x0218_A76C,
    0x0218_A784,
    0x0218_A7F0,
    0x0218_A824,
    0x0218_A878,
    0x0218_A8BC,
    0x0218_A9B0,
    0x0218_AA2C,
    0x0218_AA74,
    0x0218_AAD0,
    0x0218_AB0C,
    0x0218_AB50,
    0x0218_AB9C,
    0x0218_AC08,
    0x0218_AC4C,
    0x0218_AC84,
    0x0218_ACA8,
    0x0218_ACD0,
    0x0218_AD28,
    0x0218_AD80,
    0x0218_ADC8,
    0x0218_AE04,
    0x0218_AE48,
    0x0218_AEAC,
    0x0218_AED4,
    0x0218_AF90,
    0x0218_B0CC,
    0x0218_B13C,
    0x0218_B208,
    0x0218_B24C,
    0x0218_B2A8,
    0x0218_B2E8,
    0x0218_B310,
    0x0218_B32C,
    0x0218_B3DC,
    0x0218_B408,
    0x0218_B4B0,
    0x0218_B53C,
    0x0218_B56C,
    0x0218_B630,
    0x0218_B6AC,
    0x0218_B6F8,
];

/// Size of the EVENT REQUEST.
pub const REQ_SIZE: u32 = 0x40;
/// EVENT REQUEST: `+0x24 u16` facing direction (0 up, 1 down, 2 left, 3 right).
pub const REQ_FACING: u32 = 0x24;
/// EVENT REQUEST: `+0x34` pointer to the player position (`fx32[3]`).
pub const REQ_POS: u32 = 0x34;
/// EVENT REQUEST: `+0x3C` FLAGS.
pub const REQ_FLAGS: u32 = 0x3C;

/// C: `FUN_02158768(gamesys, script, 0, heap)` (overlay 10): a GMEVENT that runs `script`.
const SCRIPT_EVENT: u32 = 0x0215_8768;

/// Direction of a key trigger mask: up 0x40 -> 0, down 0x80 -> 1, left 0x20 -> 2, right 0x10 -> 3, else 9.
fn trigger_dir(t: u32) -> u32 {
    if t & 0x40 != 0 {
        0
    } else if t & 0x80 != 0 {
        1
    } else if t & 0x20 != 0 {
        2
    } else if t & 0x10 != 0 {
        3
    } else {
        9
    }
}

/// fx32 -> tile: `(v >> 12) / 16` (the second step a signed division rounding toward zero).
fn tile_of(v: u32) -> i32 {
    ((v as i32) >> 12) / 16
}

/// The request's FLAGS word.
fn flags<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    env.r32(req + REQ_FLAGS)
}

/// `FUN_02158768(gs, script, 0, heap)` with the request's game system and heap.
fn script_event<E: FieldEnv>(env: &mut E, req: u32, s: u32) -> u32 {
    let (heap, gs) = (env.r16(req) as u32, env.r32(req + 4));
    env.callv(SCRIPT_EVENT, &[gs, s, 0, heap])
}

/// C: `FUN_02189BD0` (unlabeled routine at `0x02189BD0`): the grid-map event check: build the EVENT REQUEST, then
/// try in order (the first GMEVENT wins; `*talked` = 1 for the talk-type ones, `*flag` = 1 for the menu one): the
/// pending script, `FUN_021C868C` / `FUN_021AE0CC`, on a step the talk / warp / tile checks, the step checks, the
/// facing / trigger talk and push checks, the step encounter, the walk-time check `FUN_021A2F20` (flag 0x964), the
/// comm checks of overlays 11 / 18 (when `FUN_02026454(..) == 2`), with A pressed the MMDL in front / BG / attribute
/// scripts / `FUN_021CFAC0` tile events / `FUN_021CD440` / `FUN_02162940`, the ledge jump, the bump checks, then the
/// comm and menu checks gated by `FUN_020221A4()`, and finally `FUN_0217B410`.
pub fn grid_check<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, talked: u32, flag: u32) -> u32 {
    env.w32(talked, 0);
    env.w32(flag, 0);
    let req = env.mem().push_local(REQ_SIZE);
    let pos = env.mem().push_local(12);
    let r = grid_check_at(env, gs, fm, talked, flag, req, pos);
    env.mem().pop_local(pos);
    env.mem().pop_local(req);
    r
}

#[allow(clippy::too_many_arguments)]
fn grid_check_at<E: FieldEnv>(
    env: &mut E,
    gs: u32,
    fm: u32,
    talked: u32,
    flag: u32,
    req: u32,
    pos: u32,
) -> u32 {
    request_init(env, req, gs, fm);
    let e = pending_script(env, req);
    if e != 0 {
        env.w32(talked, 1);
        return e;
    }
    let gd = env.r32(req + 8);
    // C: `FUN_02012ED0(gd)`: a record whose first word is handed on
    let rec = env.callv(0x0201_2ED0, &[gd]);
    let w = env.r32(rec);
    let e = env.callv(0x021C_868C, &[gs, w]);
    if e != 0 {
        return e;
    }
    let e = env.callv(0x021A_E0CC, &[fm]);
    if e != 0 {
        env.w32(talked, 1);
        return e;
    }
    if flags(env, req) & 4 != 0 {
        let gd = env.r32(req + 8);
        clear_02008df0(env, gd);
    }
    if flags(env, req) & 4 != 0 {
        let e = talk(env, req);
        if e != 0 {
            env.w32(talked, 1);
            return e;
        }
        let p = env.r32(req + REQ_POS);
        let e = warp_check(env, req, fm, p);
        if e != 0 {
            return e;
        }
        let e = tile_event(env, req, fm);
        if e != 0 {
            return e;
        }
    }
    let e = step_checks(env, req, fm);
    if e != 0 {
        return e;
    }
    if flags(env, req) & 8 != 0 {
        let e = talk_facing(env, req);
        if e != 0 {
            return e;
        }
    }
    let e = talk_trigger(env, req);
    if e != 0 {
        return e;
    }
    let e = push_check(env, req, gs, fm);
    if e != 0 {
        return e;
    }
    let e = step_encounter(env, req, gs, fm);
    if e != 0 {
        return e;
    }
    let gd = env.r32(req + 8);
    let ev = env.callv(0x0201_296C, &[gd]);
    let f964 = (env.callv(0x0201_42E8, &[ev, 0x964]) == 1) as u32;
    let (pl, held, trig) = (
        env.r32(req + 0x18),
        env.r32(req + 0x28),
        env.r32(req + 0x2C),
    );
    let e = env.callv(0x021A_2F20, &[pl, held, trig, f964]);
    if e != 0 {
        return e;
    }
    let x = env.callv(0x0201_2138, &[gs]);
    // C: `FUN_02158344(x)`: the comm state (0 = none)
    let c = env.callv(0x0215_8344, &[x]);
    let pl = env.r32(req + 0x18);
    let mm = env.callv(0x021A_3074, &[pl]);
    let zone = env.r16(req + 0x10) as u32;
    if env.callv(0x0217_BFD8, &[gs, zone]) == 1 {
        return env.callv(0x0217_CA84, &[gs, 0]);
    }
    if env.callv(0x0202_6454, &[x]) == 2 && c != 0 {
        let gd = env.r32(req + 8);
        if env.callv(0x0201_2B70, &[gd]) == 1 && env.callv(0x0217_C070, &[c]) == 1 {
            return env.call(0x0217_B810, &mut [CArg::V(gs), CArg::X(fm)]);
        }
        let zone = env.r16(req + 0x10) as u32;
        if env.callv(0x0217_689C, &[c, zone]) == 1 {
            let gd = env.r32(req + 8);
            let y = env.callv(0x0201_2FB0, &[gd]);
            return env.callv(0x0216_B3A8, &[gs, 9, y]);
        }
        let mut out = [0u8; 4];
        if flags(env, req) & 0x400 != 0
            && env.call(0x0217_54B4, &mut [CArg::V(c), CArg::OutBuf(&mut out)]) == 1
        {
            let pl = env.r32(req + 0x18);
            env.callv(0x021A_2F6C, &[pl]);
            let heap = env.r16(req) as u32;
            return env.call(
                0x0217_FC6C,
                &mut [
                    CArg::V(gs),
                    CArg::V(fm),
                    CArg::V(c),
                    CArg::V(mm),
                    CArg::V(u32::from_le_bytes(out)),
                    CArg::X(heap),
                ],
            );
        }
        if flags(env, req) & 8 != 0 {
            let pl = env.r32(req + 0x18);
            let e = env.callv(0x0217_BDE0, &[gs, c, pl]);
            if e != 0 {
                return e;
            }
        }
        if flags(env, req) & 1 != 0 {
            let pl = env.r32(req + 0x18);
            let mut out = [0u8; 4];
            if env.call(
                0x0217_AFC4,
                &mut [CArg::V(c), CArg::V(pl), CArg::OutBuf(&mut out)],
            ) == 1
            {
                let pl = env.r32(req + 0x18);
                env.callv(0x021A_2F6C, &[pl]);
                let heap = env.r16(req) as u32;
                return env.call(
                    0x0217_FB5C,
                    &mut [
                        CArg::V(gs),
                        CArg::V(fm),
                        CArg::P(c),
                        CArg::V(mm),
                        CArg::V(u32::from_le_bytes(out)),
                        CArg::X(heap),
                    ],
                );
            }
        }
    }
    if flags(env, req) & 1 != 0 {
        let pl = env.r32(req + 0x18);
        // C: `FUN_021A3050(player)`: its direction
        let d = env.callv(0x021A_3050, &[pl]);
        let pl = env.r32(req + 0x18);
        env.call(0x021A_3038, &mut [CArg::V(pl), CArg::Out(pos, 12)]);
        env.call(
            0x0216_4208,
            &mut [CArg::V(d), CArg::In(pos, 12), CArg::V(0x10000)],
        );
        let m = talk_target(env, req, fm);
        if m != 0 {
            let s = env.callv(0x0216_D5B8, &[m]);
            if env.callv(0x0215_8D90, &[s & 0xFFFF]) == 1 {
                let pl = env.r32(req + 0x18);
                let mm2 = env.callv(0x021A_3074, &[pl]);
                let pl = env.r32(req + 0x18);
                env.callv(0x021A_2F6C, &[pl]);
                env.w32(talked, 1);
                let heap = env.r16(req) as u32;
                return env.call(
                    0x0219_F01C,
                    &mut [
                        CArg::V(gs),
                        CArg::P(fm),
                        CArg::V(s),
                        CArg::P(mm2),
                        CArg::V(m),
                        CArg::X(heap),
                    ],
                );
            }
        }
        let gd = env.r32(req + 8);
        let ev = env.callv(0x0201_296C, &[gd]);
        let zev = env.r32(req + 0xC);
        let s = env.call(
            0x0216_2A54,
            &mut [CArg::V(zev), CArg::V(ev), CArg::In(pos, 12), CArg::V(d)],
        );
        if s != 0xFFFF {
            return script_event(env, req, s);
        }
        let s = attribute_script(env, req);
        if s != 0xFFFF {
            return script_event(env, req, s);
        }
        let px = (env.r32(pos) as i32) >> 12;
        let py = (env.r32(pos + 4) as i32) >> 12;
        let pz = (env.r32(pos + 8) as i32) >> 12;
        let gd = env.r32(req + 8);
        let save = env.callv(0x0201_2EBC, &[gd]);
        let b = env.callv(0x0200_BE48, &[save]);
        let zone = env.callv(0x0218_8C9C, &[fm]);
        // tile = fx >> 12 / 16 (rounding toward zero), as `s16`
        let t = |v: i32| ((v.wrapping_add(((v >> 3) as u32 >> 28) as i32) << 12) >> 16) as u32;
        let r = env.callv(0x021C_FAC0, &[b, zone, t(px), t(py), t(pz)]);
        if r != 0xFF {
            let heap = env.r16(req) as u32;
            return env.call(0x021C_FC14, &mut [CArg::V(gs), CArg::P(heap), CArg::V(r)]);
        }
        if env.callv(0x021C_D440, &[fm]) == 1 {
            return script_event(env, req, 0x271B);
        }
        let zone = env.r16(req + 0x10) as u32;
        if env.callv(0x0201_3FFC, &[zone]) != 0 {
            let gd = env.r32(req + 8);
            let ev = env.callv(0x0201_296C, &[gd]);
            let zev = env.r32(req + 0xC);
            let s = env.call(
                0x0216_2940,
                &mut [CArg::V(zev), CArg::V(ev), CArg::In(pos, 12)],
            );
            if s != 0xFFFF {
                return script_event(env, req, s);
            }
        }
    }
    let st = env.r32(req + 0x1C);
    if st == 3 || st == 0 {
        let e = ledge_check(env, req, gs, fm);
        if e != 0 {
            return e;
        }
    }
    if flags(env, req) & 0x20 != 0 {
        let e = bump_warp(env, req, gs, fm);
        if e != 0 {
            return e;
        }
        let e = bump_checks(env, req, gs, fm);
        if e != 0 {
            return e;
        }
    }
    comm_checks(env, req, gs, fm, flag, false)
}

/// The comm / menu tail shared by [`grid_check`] (`rail = false`) and [`rail_check`] (`rail = true`), from the
/// FLAGS bit 6 check on.
fn comm_checks<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32, flag: u32, rail: bool) -> u32 {
    // C: `FUN_020221A4()`: comm available
    if flags(env, req) & 0x40 != 0 && env.callv(0x0202_21A4, &[]) != 0 {
        let o = env.callv(0x0218_8CCC, &[fm]);
        if env.callv(0x0215_FF34, &[gs]) != 0 && env.callv(0x021A_0ED4, &[o]) != 0 {
            let heap = env.r16(req) as u32;
            let e = env.callv(0x0215_FF54, &[gs, fm, heap]);
            if e != 0 {
                return e;
            }
        }
    }
    if env.callv(0x0201_212C, &[gs]) == 1
        && env.callv(0x0202_21A4, &[]) != 0
        && flags(env, req) & 0x400 != 0
    {
        return env.call(0x021C_06CC, &mut [CArg::V(gs), CArg::X(fm)]);
    }
    if flags(env, req) & 2 != 0 && env.callv(0x0202_21A4, &[]) != 0 {
        let o = env.callv(0x0218_8CCC, &[fm]);
        if env.callv(0x021A_0F90, &[o]) == 6 {
            return 0;
        }
        if !rail {
            env.w32(flag, 1);
        }
        let heap = env.r16(req) as u32;
        return env.callv(0x0215_FC40, &[gs, fm, heap]);
    }
    if rail {
        let b7 = (flags(env, req) & 0x80 != 0 && env.callv(0x0202_21A4, &[]) != 0) as u32;
        let o = env.callv(0x0218_8CCC, &[fm]);
        let e = env.callv(0x021A_0EB0, &[o, b7]);
        if e != 0 {
            return e;
        }
    } else if env.callv(0x0202_21A4, &[]) != 0 {
        let o = env.callv(0x0218_8CCC, &[fm]);
        let b7 = (flags(env, req) >> 7) & 1;
        let e = env.callv(0x021A_0EB0, &[o, b7]);
        if e != 0 {
            return e;
        }
    }
    if flags(env, req) & 0x80 != 0 && env.callv(0x0202_21A4, &[]) != 0 {
        let e = event_0218aca8(env, gs);
        if e != 0 {
            return e;
        }
    }
    if rail {
        return 0;
    }
    let pl = env.r32(req + 0x18);
    env.callv(0x0217_B410, &[fm, gs, pl])
}

/// C: `FUN_0218A074` (unlabeled routine at `0x0218A074`): the grid-map entry point: `FUN_021AA8F8(e)` on the
/// encounter object, [`grid_check`]; on an event: `FUN_021AA8EC(e)` when it was a talk, `FUN_021BC5BC(..)`,
/// `FUN_021A3550(player, flag)`, `FUN_021A356C(player)`, `FUN_021F3B10(..)` when `FUN_02189A70(f)` is set, and
/// [`after_event`]. Returns the GMEVENT.
pub fn grid_entry<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let tf = env.mem().push_local(8);
    let r = (|| {
        let f = env.callv(0x0201_210C, &[gs]);
        let e = env.callv(0x0218_8CE8, &[f]);
        // C: `FUN_021AA8F8(e)` (thunk to `FUN_021AAF54`)
        env.callv(0x021A_A8F8, &[e]);
        let (flag, talked) = (tf, tf + 4);
        let ev = grid_check(env, gs, fm, talked, flag);
        if ev == 0 {
            return 0;
        }
        if env.r32(talked) != 0 {
            env.callv(0x021A_A8EC, &[e]);
        }
        let a = env.callv(0x0218_8D20, &[f]);
        env.callv(0x021B_C5BC, &[a]);
        let pl = env.callv(0x0218_8CB4, &[f]);
        let fl = env.r32(flag);
        env.callv(0x021A_3550, &[pl, fl]);
        let pl = env.callv(0x0218_8CB4, &[f]);
        env.callv(0x021A_356C, &[pl]);
        let b = env.callv(0x0218_9A70, &[f]);
        if b != 0 {
            env.callv(0x021F_3B10, &[b]);
        }
        after_event(env, gs, f);
        ev
    })();
    env.mem().pop_local(tf);
    r
}

/// C: `FUN_0218A0E4` (unlabeled routine at `0x0218A0E4`): the event check of the comm zones (`c = FUN_02026474(x)`):
/// without `c`, `FUN_021C06CC` (when `FUN_0200FB24()` and `FUN_020221A4()`) or `FUN_021850FC`; with `c`, nothing
/// while `FUN_0217391C(c) == 1` (`FUN_02026400(x)` once `FUN_02173928(c)` is 0), else talk (direction 9), the
/// overlay-18 checks `FUN_0217B80C` / `FUN_0217B83C`, the MMDL in front, `FUN_021C06CC`, `FUN_0215FCE0`.
pub fn comm_check<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let req = env.mem().push_local(REQ_SIZE);
    let r = comm_check_at(env, gs, fm, req);
    env.mem().pop_local(req);
    r
}

fn comm_check_at<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, req: u32) -> u32 {
    let x = env.callv(0x0201_2138, &[gs]);
    let c = env.callv(0x0202_6474, &[x]);
    request_init(env, req, gs, fm);
    if c == 0 {
        if env.callv(0x0200_FB24, &[]) != 0 {
            if env.callv(0x0202_21A4, &[]) != 0 {
                return env.call(0x021C_06CC, &mut [CArg::V(gs), CArg::X(fm)]);
            }
            return 0;
        }
        // the spec's stub lists the first argument
        return env.call(0x0218_50FC, &mut [CArg::V(gs), CArg::X(fm)]);
    }
    if env.callv(0x0217_391C, &[c]) == 1 {
        if env.callv(0x0217_3928, &[c]) == 0 {
            env.call(0x0202_6400, &mut [CArg::X(x)]);
        }
        return 0;
    }
    if flags(env, req) & 4 != 0 {
        let e = talk_dir(env, req, 9);
        if e != 0 {
            return e;
        }
    }
    let e = env.callv(0x0217_B80C, &[gs, fm]);
    if e != 0 {
        return e;
    }
    if flags(env, req) & 1 != 0 {
        let e = env.callv(0x0217_B83C, &[gs, fm]);
        if e != 0 {
            return e;
        }
    }
    if flags(env, req) & 1 != 0 && env.callv(0x0217_394C, &[gs]) == 1 {
        let m = talk_target(env, req, fm);
        if m != 0 {
            let s = env.callv(0x0216_D5B8, &[m]);
            if env.callv(0x0215_8D90, &[s & 0xFFFF]) == 1 {
                let pl = env.r32(req + 0x18);
                let mm = env.callv(0x021A_3074, &[pl]);
                let a = env.callv(0x0216_D588, &[m]);
                if env.callv(0x0217_AF38, &[a]) == 0 {
                    let pl = env.r32(req + 0x18);
                    env.callv(0x021A_2F6C, &[pl]);
                    let heap = env.r16(req) as u32;
                    return env.call(
                        0x0219_F01C,
                        &mut [
                            CArg::V(gs),
                            CArg::P(fm),
                            CArg::V(s),
                            CArg::P(mm),
                            CArg::V(m),
                            CArg::X(heap),
                        ],
                    );
                }
            }
        }
    }
    if env.callv(0x0201_212C, &[gs]) == 1 && env.callv(0x0202_21A4, &[]) != 0 {
        return env.call(0x021C_06CC, &mut [CArg::V(gs), CArg::X(fm)]);
    }
    if env.callv(0x0217_394C, &[gs]) == 0 {
        return 0;
    }
    if flags(env, req) & 2 != 0 && env.callv(0x0202_21A4, &[]) != 0 {
        let heap = env.r16(req) as u32;
        return env.callv(0x0215_FCE0, &[gs, fm, heap]);
    }
    0
}

/// C: `FUN_0218A240` (unlabeled routine at `0x0218A240`): the rail-map entry point: [`rail_check`]; on an event
/// `FUN_021BC5BC(..)`, `FUN_021F3B10(..)` when `FUN_02189A70(f)` is set, [`after_event`].
pub fn rail_entry<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let ev = rail_check(env, gs, fm);
    if ev == 0 {
        return 0;
    }
    let f = env.callv(0x0201_210C, &[gs]);
    let a = env.callv(0x0218_8D20, &[f]);
    env.callv(0x021B_C5BC, &[a]);
    let b = env.callv(0x0218_9A70, &[f]);
    if b != 0 {
        env.callv(0x021F_3B10, &[b]);
    }
    after_event(env, gs, f);
    ev
}

/// C: `FUN_0218A278` (unlabeled routine at `0x0218A278`): the rail-map event check: the pending script, on a step
/// the rail position script `FUN_02162958`, the rail warp, the step checks, the step encounter; with A pressed the
/// rail MMDL in front and `FUN_02162A5C`; on a facing trigger the rail bump warps; the push-down event; then the
/// comm / menu checks.
pub fn rail_check<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let req = env.mem().push_local(REQ_SIZE);
    let rp = env.mem().push_local(8);
    let rp2 = env.mem().push_local(8);
    let r = rail_check_at(env, gs, fm, req, rp, rp2);
    env.mem().pop_local(rp2);
    env.mem().pop_local(rp);
    env.mem().pop_local(req);
    r
}

fn rail_check_at<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, req: u32, rp: u32, rp2: u32) -> u32 {
    request_init(env, req, gs, fm);
    let pl = env.r32(req + 0x18);
    let mm = env.callv(0x021A_3074, &[pl]);
    // C: `FUN_0219E634(mmdl, &rp2)`: the rail position in front
    env.call(0x0219_E634, &mut [CArg::V(mm), CArg::Out(rp2, 8)]);
    let e = pending_script(env, req);
    if e != 0 {
        return e;
    }
    if flags(env, req) & 4 != 0 {
        let gd = env.r32(req + 8);
        clear_02008df0(env, gd);
    }
    if flags(env, req) & 4 != 0 {
        let pl = env.r32(req + 0x18);
        let mm2 = env.callv(0x021A_3074, &[pl]);
        let gd = env.r32(req + 8);
        let ev = env.callv(0x0201_296C, &[gd]);
        env.call(0x0219_E2F4, &mut [CArg::V(mm2), CArg::Out(rp, 8)]);
        let zev = env.r32(req + 0xC);
        let s = env.call(
            0x0216_2958,
            &mut [CArg::V(zev), CArg::V(ev), CArg::In(rp, 8)],
        );
        if s != 0xFFFF {
            return script_event(env, req, s);
        }
        let e = rail_warp_check(env, req, gs, fm);
        if e != 0 {
            return e;
        }
        let e = step_checks(env, req, fm);
        if e != 0 {
            return e;
        }
    }
    let e = step_encounter(env, req, gs, fm);
    if e != 0 {
        return e;
    }
    if flags(env, req) & 1 != 0 {
        let m = rail_talk_target(env, req, fm);
        if m != 0 {
            let s = env.callv(0x0216_D5B8, &[m]);
            if env.callv(0x0215_8D90, &[s & 0xFFFF]) == 1 {
                let pl = env.r32(req + 0x18);
                let mm3 = env.callv(0x021A_3074, &[pl]);
                let pl = env.r32(req + 0x18);
                env.callv(0x021A_2F6C, &[pl]);
                let heap = env.r16(req) as u32;
                return env.call(
                    0x0219_F01C,
                    &mut [
                        CArg::V(gs),
                        CArg::P(fm),
                        CArg::V(s),
                        CArg::P(mm3),
                        CArg::V(m),
                        CArg::X(heap),
                    ],
                );
            }
        }
        let gd = env.r32(req + 8);
        let ev = env.callv(0x0201_296C, &[gd]);
        let f = env.callv(0x0216_D5F8, &[mm]);
        let mut x = [0u8; 8];
        for (k, b) in x.iter_mut().enumerate() {
            *b = env.r8(rp2 + k as u32);
        }
        let zev = env.r32(req + 0xC);
        let s = env.call(
            0x0216_2A5C,
            &mut [CArg::V(zev), CArg::V(ev), CArg::InBuf(&x), CArg::V(f)],
        );
        if s != 0xFFFF {
            return script_event(env, req, s);
        }
    }
    if flags(env, req) & 0x20 != 0 {
        let pl = env.callv(0x0218_8CB4, &[fm]);
        if !facing_free(env, pl, req) {
            let e = rail_bump_warp(env, req, gs, fm, rp2);
            if e != 0 {
                return e;
            }
        }
    }
    let e = push_down_event(env, req, gs, fm);
    if e != 0 {
        return e;
    }
    comm_checks(env, req, gs, fm, 0, true)
}

/// C: `FUN_0218A4BC` (unlabeled routine at `0x0218A4BC`): the hybrid-map entry point: [`grid_entry`] or, when
/// `FUN_02188CFC(fm)` says the player is on a rail, [`rail_entry`].
pub fn hybrid_entry<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    if env.callv(0x0218_8CFC, &[fm]) == 0 {
        grid_entry(env, gs, fm)
    } else {
        rail_entry(env, gs, fm)
    }
}

/// C: `FUN_0218A4E0` (unlabeled routine at `0x0218A4E0`): clear and fill the EVENT REQUEST `req` from the
/// FIELDMAP `fm`, the key state and the player. FLAGS: when the move state (`+0x1C`) is 0 or 3: bit 10, bit 0 =
/// held & 1, bit 1 = held & 0x400, bit 6 = held & 0x800, bit 7 = none of these three; always: bit 2 = (`+0x20` == 1
/// and state 3), bit 9 = `FUN_021A3584(player)`, bit 3 = state 3, bit 4 = (state != 3 and `+0x20` == 2), bit 5 = a
/// trigger key in the facing direction.
pub fn request_init<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32) {
    env.mem().fill(req, REQ_SIZE, 0);
    // C: `FUN_02188C84(fm)`: heap id
    let heap = env.callv(0x0218_8C84, &[fm]);
    env.w16(req, heap as u16);
    env.w32(req + 4, gs);
    // C: `FUN_020120F4(gs)`: GAMEDATA; `FUN_02012844(gd)`: zone event data
    let gd = env.callv(0x0201_20F4, &[gs]);
    env.w32(req + 8, gd);
    let zev = env.callv(0x0201_2844, &[gd]);
    env.w32(req + 0xC, zev);
    // C: `FUN_02188C9C(fm)`: current zone id
    let zone = env.callv(0x0218_8C9C, &[fm]);
    env.w16(req + 0x10, zone as u16);
    // C: `FUN_020362DC()` / `FUN_0203630C()`: keys held / triggered
    let held = env.callv(0x0203_62DC, &[]);
    env.w32(req + 0x28, held);
    let trig = env.callv(0x0203_630C, &[]);
    env.w16(req + 0x30, 9);
    env.w32(req + 0x2C, trig);
    let d = trigger_dir(trig);
    if d != 9 {
        env.w16(req + 0x30, d as u16);
    }
    env.w32(req + 0x14, fm);
    // C: `FUN_02188CB4(fm)`: the player
    let pl = env.callv(0x0218_8CB4, &[fm]);
    env.w32(req + 0x18, pl);
    // C: `FUN_021A2FE4(player)`, `FUN_021A3080(player)`, `FUN_021A308C(player)` (move state)
    env.callv(0x021A_2FE4, &[pl]);
    let pl = env.r32(req + 0x18);
    let v = env.callv(0x021A_3080, &[pl]);
    env.w32(req + 0x20, v);
    let pl = env.r32(req + 0x18);
    let st = env.callv(0x021A_308C, &[pl]);
    env.w32(req + 0x1C, st);
    // C: `FUN_021A3074(player)`: its MMDL; `FUN_0216D5F8(mmdl)`: facing; `FUN_0216D888(mmdl)`: position
    let pl = env.r32(req + 0x18);
    let mm = env.callv(0x021A_3074, &[pl]);
    let f = env.callv(0x0216_D5F8, &[mm]);
    env.w16(req + REQ_FACING, f as u16);
    let pos = env.callv(0x0216_D888, &[mm]);
    env.w32(req + REQ_POS, pos);
    // C: `FUN_02014478(FUN_02012B38(gd))`: pending script id
    let gd = env.r32(req + 8);
    let x = env.callv(0x0201_2B38, &[gd]);
    let s = env.callv(0x0201_4478, &[x]);
    env.w16(req + 0x26, s as u16);
    // C: `FUN_021A30C8(player)`
    let pl = env.r32(req + 0x18);
    let v = env.callv(0x021A_30C8, &[pl]);
    env.w32(req + 0x38, v);
    let fl = req + REQ_FLAGS;
    let st = env.r32(req + 0x1C);
    if st == 3 || st == 0 {
        let v = env.r32(fl) | 0x400;
        env.w32(fl, v);
        let b0 = env.r32(req + 0x28) & 1;
        let v = (env.r32(fl) & !1) | b0;
        env.w32(fl, v);
        let b1 = (env.r32(req + 0x28) & 0x400 != 0) as u32;
        let v = (env.r32(fl) & !2) | (b1 << 1);
        env.w32(fl, v);
        let b6 = (env.r32(req + 0x28) & 0x800 != 0) as u32;
        let v = (env.r32(fl) & !0x40) | (b6 << 6);
        let n = (v & 1) + ((v >> 1) & 1) + ((v >> 6) & 1);
        env.w32(fl, v);
        if n & 0xFF == 0 {
            env.w32(fl, v | 0x80);
        }
    }
    let v = env.r32(fl) & !4 & 0xFFFF_FDFF;
    env.w32(fl, v);
    if env.r32(req + 0x20) == 1 {
        let b = (env.r32(req + 0x1C) == 3) as u32;
        let v = (env.r32(fl) & !4) | (b << 2);
        env.w32(fl, v);
    }
    // C: `FUN_021A3584(player)`
    let pl = env.r32(req + 0x18);
    if env.callv(0x021A_3584, &[pl]) != 0 {
        let v = env.r32(fl) | 0x200;
        env.w32(fl, v);
    }
    let b = (env.r32(req + 0x1C) == 3) as u32;
    let mut v = ((env.r32(fl) & !8) | (b << 3)) & !0x10;
    env.w32(fl, v);
    if env.r32(req + 0x1C) != 3 && env.r32(req + 0x20) == 2 {
        v |= 0x10;
        env.w32(fl, v);
    }
    let f = env.r16(req + REQ_FACING);
    let t = env.r32(req + 0x2C);
    let hit = (f == 0 && t & 0x40 != 0)
        || (f == 1 && t & 0x80 != 0)
        || (f == 2 && t & 0x20 != 0)
        || (f == 3 && t & 0x10 != 0);
    let v = env.r32(fl);
    env.w32(fl, if hit { v | 0x20 } else { v & !0x20 });
}

/// C: `FUN_0218A6E8` (unlabeled routine at `0x0218A6E8`): after an event: `x = FUN_021A1358(FUN_02188CCC(f))`;
/// no `x`, or `FUN_021F2AB8(x) == 0`: `FUN_02028A70(FUN_02012A30(GAMEDATA))`, and in the second case also
/// `FUN_021F2AD4(x)`.
pub fn after_event<E: FieldEnv>(env: &mut E, gs: u32, f: u32) {
    let a = env.callv(0x0218_8CCC, &[f]);
    let x = env.callv(0x021A_1358, &[a]);
    let reset = |env: &mut E| {
        let gd = env.callv(0x0201_20F4, &[gs]);
        let y = env.callv(0x0201_2A30, &[gd]);
        env.callv(0x0202_8A70, &[y]);
    };
    if x == 0 {
        reset(env);
    } else if env.callv(0x021F_2AB8, &[x]) == 0 {
        reset(env);
        env.callv(0x021F_2AD4, &[x]);
    }
}

/// C: `FUN_0218A728` (unlabeled routine at `0x0218A728`): `FUN_02008DF0(FUN_02012F2C(gd), 0)` (the thunk calls
/// `FUN_02008DAC(x, 0, 1)`).
pub fn clear_02008df0<E: FieldEnv>(env: &mut E, gd: u32) {
    let x = env.callv(0x0201_2F2C, &[gd]);
    env.callv(0x0200_8DF0, &[x, 0]);
}

/// C: `FUN_0218A738` (unlabeled routine at `0x0218A738`): the pending script (`+0x26`): clear it
/// (`FUN_02014474(FUN_02012B38(gd), 0)`) and return its script event; none: `FUN_02158B00(gamesys, heap)`.
pub fn pending_script<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    if env.r16(req + 0x26) != 0 {
        let gd = env.r32(req + 8);
        let x = env.callv(0x0201_2B38, &[gd]);
        env.callv(0x0201_4474, &[x, 0]);
        let (s, heap, gs) = (
            env.r16(req + 0x26) as u32,
            env.r16(req) as u32,
            env.r32(req + 4),
        );
        return env.callv(SCRIPT_EVENT, &[gs, s, 0, heap]);
    }
    let (heap, gs) = (env.r16(req) as u32, env.r32(req + 4));
    env.callv(0x0215_8B00, &[gs, heap])
}

/// C: `FUN_0218A76C` (unlabeled routine at `0x0218A76C`): [`talk_bg`] or else [`talk_dir`] with direction 9.
pub fn talk<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    match talk_bg(env, req) {
        0 => talk_dir(env, req, 9),
        e => e,
    }
}

/// The player position into a local (`FUN_021A3038(player, &pos)`), and whether the attribute there is 1
/// (`FUN_021AB188(FUN_021AB0F0(FUN_021AB0D4(FUN_02188C98(f), &pos)))`).
fn player_tile<E: FieldEnv>(env: &mut E, pl: u32, f: u32) -> ([u8; 12], bool) {
    let mut pos = [0u8; 12];
    env.call(0x021A_3038, &mut [CArg::V(pl), CArg::OutBuf(&mut pos)]);
    let a = env.callv(0x0218_8C98, &[f]);
    let b = env.call(0x021A_B0D4, &mut [CArg::V(a), CArg::InBuf(&pos)]);
    let c = env.callv(0x021A_B0F0, &[b]);
    let one = env.callv(0x021A_B188, &[c]) == 1;
    (pos, one)
}

/// C: `FUN_0218A784` (unlabeled routine at `0x0218A784`): talk to what is in direction `dir`: nothing when the
/// player's tile attribute is 1; else the script `FUN_02162928(evdata, ev, &pos, dir)` (0xFFFF = none) as an event.
pub fn talk_dir<E: FieldEnv>(env: &mut E, req: u32, dir: u32) -> u32 {
    let gs = env.r32(req + 4);
    let f = env.callv(0x0201_210C, &[gs]);
    let gd = env.r32(req + 8);
    let ev = env.callv(0x0201_296C, &[gd]);
    let pl = env.r32(req + 0x18);
    let (pos, one) = player_tile(env, pl, f);
    if one {
        return 0;
    }
    let zev = env.r32(req + 0xC);
    let s = env.call(
        0x0216_2928,
        &mut [CArg::V(zev), CArg::V(ev), CArg::InBuf(&pos), CArg::V(dir)],
    );
    if s == 0xFFFF {
        return 0;
    }
    let (heap, gs) = (env.r16(req) as u32, env.r32(req + 4));
    env.callv(SCRIPT_EVENT, &[gs, s, 0, heap])
}

/// C: `FUN_0218A7F0` (unlabeled routine at `0x0218A7F0`): talk in the facing direction, else in the direction of
/// `FUN_021A2FA4(player, trigger)` when that is neither 9 nor the facing.
pub fn talk_facing<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    let f = env.r16(req + REQ_FACING) as u32;
    let e = talk_dir(env, req, f);
    if e != 0 {
        return e;
    }
    let (pl, t) = (env.r32(req + 0x18), env.r32(req + 0x2C));
    let d = env.callv(0x021A_2FA4, &[pl, t]);
    if d == 9 || d == env.r16(req + REQ_FACING) as u32 {
        return 0;
    }
    talk_dir(env, req, d)
}

/// C: `FUN_0218A824` (unlabeled routine at `0x0218A824`): talk in the trigger direction when it is a direction
/// other than the facing and the move state is neither 1 nor 2.
pub fn talk_trigger<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    let d = trigger_dir(env.r32(req + 0x2C));
    let st = env.r32(req + 0x1C);
    if d == env.r16(req + REQ_FACING) as u32 || d == 9 || st == 1 || st == 2 {
        return 0;
    }
    talk_dir(env, req, d)
}

/// C: `FUN_0218A878` (unlabeled routine at `0x0218A878`): in a zone where `FUN_02013D5C(zone)` holds, the push
/// check `FUN_0217B4D8(gs, fm, player, pos, facing, d)` for `d = FUN_021A2FA4(player, trigger)` (none for 9).
pub fn push_check<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32) -> u32 {
    let zone = env.r16(req + 0x10) as u32;
    if env.callv(0x0201_3D5C, &[zone]) == 0 {
        return 0;
    }
    let (pl, t) = (env.r32(req + 0x18), env.r32(req + 0x2C));
    let d = env.callv(0x021A_2FA4, &[pl, t]);
    if d == 9 {
        return 0;
    }
    let f = env.r16(req + REQ_FACING) as u32;
    let (pl, pos) = (env.r32(req + 0x18), env.r32(req + REQ_POS));
    // the spec's stub lists the first five arguments
    env.call(
        0x0217_B4D8,
        &mut [
            CArg::V(gs),
            CArg::V(fm),
            CArg::V(pl),
            CArg::V(pos),
            CArg::V(f),
            CArg::X(d),
        ],
    )
}

/// C: `FUN_0218A8BC` (unlabeled routine at `0x0218A8BC`): talk to the BG event under the player: on a tile with
/// attribute 1, the BG event `FUN_0216282C(evdata, ev, &pos, 9)` of kind 0 (`u16 +8`) runs its script (`u16 +0`)
/// when the player stands at its centre (`+0xA + +0xE / 2`, `+0xC + +0x10 / 2`), or when `FUN_021A3098(player)`
/// is set or the MMDL code `FUN_0216D7C4` is 0x58..0x5B or 0xA3..0xA6.
pub fn talk_bg<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    let gs = env.r32(req + 4);
    let f = env.callv(0x0201_210C, &[gs]);
    let pl = env.callv(0x0218_8CB4, &[f]);
    let (pos, one) = player_tile(env, pl, f);
    if !one {
        return 0;
    }
    let gd = env.r32(req + 8);
    let ev = env.callv(0x0201_296C, &[gd]);
    let zev = env.r32(req + 0xC);
    let bg = env.call(
        0x0216_282C,
        &mut [CArg::V(zev), CArg::V(ev), CArg::InBuf(&pos), CArg::V(9)],
    );
    if bg == 0 || env.r16(bg + 8) != 0 {
        return 0;
    }
    let cz = env.r16(bg + 0xC) as u32 + (env.r16(bg + 0x10) as u32 >> 1);
    let cx = env.r16(bg + 0xA) as u32 + (env.r16(bg + 0xE) as u32 >> 1);
    let px = u32::from_le_bytes(pos[0..4].try_into().unwrap());
    let pz = u32::from_le_bytes(pos[8..12].try_into().unwrap());
    let run = |env: &mut E| {
        let (s, heap, gs) = (env.r16(bg) as u32, env.r16(req) as u32, env.r32(req + 4));
        env.callv(SCRIPT_EVENT, &[gs, s, 0, heap])
    };
    if cx as i32 == tile_of(px) && cz as i32 == tile_of(pz) {
        return run(env);
    }
    let t = env.callv(0x021A_3098, &[pl]);
    let mm = env.callv(0x021A_3074, &[pl]);
    let k = env.callv(0x0216_D7C4, &[mm]);
    if t != 0 || (0x58..=0x5B).contains(&k) || (k.wrapping_add(0xFF5D) & 0xFFFF) <= 3 {
        return run(env);
    }
    0
}

/// C: `FUN_0218A9B0` (unlabeled routine at `0x0218A9B0`): the step checks: with FLAGS bit 2 or 9 the encounter
/// check [`encounter_check`]; then only with bit 2 (a step): [`step_counters`], [`step_zone_event`],
/// [`egg_step`], [`step_0200d564`], the first event wins.
pub fn step_checks<E: FieldEnv>(env: &mut E, req: u32, fm: u32) -> u32 {
    env.callv(0x0218_8C84, &[fm]);
    // C: `FUN_02188C80(fm)`: the game system
    let gs = env.callv(0x0218_8C80, &[fm]);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let fl = env.r32(req + REQ_FLAGS);
    if fl & 4 != 0 || fl & 0x200 != 0 {
        let e = encounter_check(env, fm, gs, gd);
        if e != 0 {
            return e;
        }
    }
    if env.r32(req + REQ_FLAGS) & 4 == 0 {
        return 0;
    }
    let gd2 = env.r32(req + 8);
    step_counters(env, gd2, fm);
    let zone = env.r16(req + 0x10) as u32;
    let e = step_zone_event(env, gs, fm, zone);
    if e != 0 {
        return e;
    }
    let e = egg_step(env, req, fm, gs, gd);
    if e != 0 {
        return e;
    }
    step_0200d564(env, fm, gs, gd)
}

/// C: `FUN_0201AA18(party)`: party size; `FUN_0201AB28(party, i)`: member `i`; `FUN_02017E1C(mon, field, 0)`:
/// a field of a Pokemon (0xA held item, 0x4C is-egg, 9 friendship / hatch counter).
const PARTY_COUNT: u32 = 0x0201_AA18;
const PARTY_MON: u32 = 0x0201_AB28;
const MON_GET: u32 = 0x0201_7E1C;
const MON_SET: u32 = 0x0201_7E40;

/// C: `FUN_0218AA2C` (unlabeled routine at `0x0218AA2C`): egg-hatch speed: 2 when a party member that is not an
/// egg holds item 0x31 or 0x28 (meaning of the items UNVERIFIED), else 1.
pub fn hatch_speed<E: FieldEnv>(env: &mut E, party: u32) -> u32 {
    let n = env.callv(PARTY_COUNT, &[party]) as i32;
    for i in 0..n.max(0) as u32 {
        let m = env.callv(PARTY_MON, &[party, i]);
        let item = env.callv(MON_GET, &[m, 0xA, 0]);
        if env.callv(MON_GET, &[m, 0x4C, 0]) == 0 && (item == 0x31 || item == 0x28) {
            return 2;
        }
    }
    1
}

/// C: `FUN_0218AA74` (unlabeled routine at `0x0218AA74`): every egg's hatch counter (field 9) goes down by
/// [`hatch_speed`] (not below 0).
pub fn eggs_step<E: FieldEnv>(env: &mut E, party: u32) {
    let n = env.callv(PARTY_COUNT, &[party]) as i32;
    let k = hatch_speed(env, party);
    for i in 0..n.max(0) as u32 {
        let m = env.callv(PARTY_MON, &[party, i]);
        let egg = env.callv(MON_GET, &[m, 0x4C, 0]);
        let c = env.callv(MON_GET, &[m, 9, 0]);
        if egg == 1 {
            let v = c.saturating_sub(k);
            // the fourth word is r3, which the 3-argument callee does not read: `FUN_0218AA2C`'s epilogue leaves
            // the party pointer there (the spec's stub logs it)
            env.callv(MON_SET, &[m, 9, v, party]);
        }
    }
}

/// C: `FUN_0218AAD0` (unlabeled routine at `0x0218AAD0`): the egg step counter (`FUN_02008970` / `FUN_0200896C` on
/// the save block `FUN_02008854(FUN_02012EBC(gd))`) goes up by `FUN_02010FF4(0x100)`; past 0x10000 the eggs step
/// ([`eggs_step`]) and it restarts at 0.
///
/// Verified (in [`PORTED`] since the Z19 entry state: the r2 / r3 its spec's stub `02010ff4 args=v,p,v,v` logs are this
/// function's entry r2 (the `r2` argument) and r3 (0 on every input)). Callers reach it through their (forced) stub
/// (`FieldEnv::call(0x0218AAD0, ..)`).
pub fn egg_counter<E: FieldEnv>(env: &mut E, gd: u32, party: u32, r2: u32) {
    let save = env.callv(0x0201_2EBC, &[gd]);
    let s = env.callv(0x0200_8854, &[save]);
    let mut c = [0u8; 4];
    env.call(0x0200_8970, &mut [CArg::V(s), CArg::OutBuf(&mut c)]);
    let mut c = u32::from_le_bytes(c);
    // r1 = &c, r2 = the entry r2, r3 = the thunk's target (left by `FUN_02008854`): not read by the callee
    let d = env.call(
        0x0201_0FF4,
        &mut [
            CArg::V(0x100),
            CArg::P(0),
            CArg::V(r2),
            CArg::V(0x0200_71CD),
        ],
    );
    c = c.wrapping_add(d);
    if c > 0x1_0000 {
        eggs_step(env, party);
        c = 0;
    }
    env.callv(0x0200_896C, &[s, c]);
}

/// C: `FUN_0218AB0C` (unlabeled routine at `0x0218AB0C`): a party member is an egg with hatch counter 0.
pub fn egg_ready<E: FieldEnv>(env: &mut E, party: u32) -> bool {
    let n = env.callv(PARTY_COUNT, &[party]) as i32;
    for i in 0..n.max(0) as u32 {
        let m = env.callv(PARTY_MON, &[party, i]);
        let egg = env.callv(MON_GET, &[m, 0x4C, 0]);
        let c = env.callv(MON_GET, &[m, 9, 0]);
        if egg == 1 && c == 0 {
            return true;
        }
    }
    false
}

/// C: `FUN_0218AB50` (unlabeled routine at `0x0218AB50`): the egg step: `FUN_021C3378(FUN_02188D50(fm))`, the egg
/// counter (`FUN_0218AAD0`, through its stub), then the hatch script 0x8CA when an egg is ready.
pub fn egg_step<E: FieldEnv>(env: &mut E, _req: u32, fm: u32, gs: u32, gd: u32) -> u32 {
    let h = env.callv(0x0218_8C84, &[fm]);
    let party = env.callv(0x0201_2934, &[gd]);
    let x = env.callv(0x0218_8D50, &[fm]);
    env.callv(0x021C_3378, &[x]);
    env.callv(0x0218_AAD0, &[gd, party]);
    if egg_ready(env, party) {
        return env.callv(SCRIPT_EVENT, &[gs, 0x8CA, 0, h]);
    }
    0
}

/// C: `FUN_0218AB9C` (unlabeled routine at `0x0218AB9C`): the 128-step counter (`u16` at save block `+0x7C`,
/// `FUN_02008978` / `FUN_02008980`); when it wraps, every party member gets the walk step `FUN_0201B7EC(mon,
/// zone, heap)` (friendship).
pub fn step_counters<E: FieldEnv>(env: &mut E, gd: u32, fm: u32) {
    let save = env.callv(0x0201_2EBC, &[gd]);
    let s = env.callv(0x0200_8854, &[save]);
    let mut n = env.callv(0x0200_8978, &[s]).wrapping_add(1) & 0xFFFF;
    let wrap = n >= 0x80;
    if wrap {
        n = 0;
    }
    env.callv(0x0200_8980, &[s, n]);
    if !wrap {
        return;
    }
    let party = env.callv(0x0201_2934, &[gd]);
    let zone = env.callv(0x0218_8C9C, &[fm]);
    let heap = env.callv(0x0218_8C84, &[fm]);
    let cnt = env.callv(PARTY_COUNT, &[party]) as i32;
    for i in 0..cnt.max(0) as u32 {
        let m = env.callv(PARTY_MON, &[party, i]);
        // the fourth word is r3 (not read by the callee): the thunk `FUN_02008854` left its target there
        env.callv(0x0201_B7EC, &[m, zone, heap, 0x0200_71CD]);
    }
}

/// C: `FUN_0218AC08` (unlabeled routine at `0x0218AC08`): in a zone where `FUN_02013FFC(zone)` holds, the event
/// `FUN_021D220C(gs, fm)`, and the save block counter `FUN_02008990` / `FUN_02008988` goes up by one.
pub fn step_zone_event<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, zone: u32) -> u32 {
    let gd = env.callv(0x0201_20F4, &[gs]);
    let save = env.callv(0x0201_2EBC, &[gd]);
    let s = env.callv(0x0200_8854, &[save]);
    if env.callv(0x0201_3FFC, &[zone]) == 0 {
        return 0;
    }
    let e = env.callv(0x021D_220C, &[gs, fm]);
    let n = env.callv(0x0200_8990, &[s]).wrapping_add(1) & 0xFFFF;
    env.callv(0x0200_8988, &[s, n]);
    e
}

/// C: `FUN_0218AC4C` (unlabeled routine at `0x0218AC4C`): script 0x2792 when the byte
/// `FUN_0200D564(FUN_0200D418(save))` is nonzero.
pub fn step_0200d564<E: FieldEnv>(env: &mut E, fm: u32, gs: u32, gd: u32) -> u32 {
    let save = env.callv(0x0201_2EBC, &[gd]);
    let b = env.callv(0x0200_D418, &[save]);
    if env.callv(0x0200_D564, &[b]) & 0xFF == 0 {
        return 0;
    }
    let h = env.callv(0x0218_8C84, &[fm]);
    env.callv(SCRIPT_EVENT, &[gs, 0x2792, 0, h])
}

/// C: `FUN_0218AC84` (unlabeled routine at `0x0218AC84`): the wild-encounter check `FUN_021AA944(enc)` of the
/// encounter object `FUN_02188CE8(fm)`; no event: `FUN_021AA80C(enc)`.
pub fn encounter_check<E: FieldEnv>(env: &mut E, fm: u32, _gs: u32, gd: u32) -> u32 {
    env.callv(0x0201_2EBC, &[gd]);
    let e = env.callv(0x0218_8CE8, &[fm]);
    let r = env.callv(0x021A_A944, &[e]);
    if r == 0 {
        env.callv(0x021A_A80C, &[e]);
    }
    r
}

/// C: `FUN_0218ACA8` (unlabeled routine at `0x0218ACA8`): unless `FUN_02010F18() == 0x30`, the GMEVENT
/// `FUN_020124AC(gs, 0x53, 0x021F532D, 0)`.
pub fn event_0218aca8<E: FieldEnv>(env: &mut E, gs: u32) -> u32 {
    if env.callv(0x0201_0F18, &[]) == 0x30 {
        return 0;
    }
    env.callv(0x0201_24AC, &[gs, 0x53, 0x021F_532D, 0])
}

/// C: `FUN_0218ACD0` (unlabeled routine at `0x0218ACD0`): a warp at `pos` ([`warp_at`]) whose object is enabled
/// (`FUN_021626EC(o) != 1`) and of a warp kind ([`is_warp_kind`]): store the return location ([`set_return`]) and
/// start the warp ([`warp_event`]).
pub fn warp_check<E: FieldEnv>(env: &mut E, req: u32, fm: u32, pos: u32) -> u32 {
    let id = warp_at(env, req, pos);
    if id == 0xFFFF {
        return 0;
    }
    let zev = env.r32(req + 0xC);
    // C: `FUN_021625CC(evdata, id)`: the warp object `id` of the zone events
    let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
    if env.callv(0x0216_26EC, &[o]) == 1 || !is_warp_kind(env, o) {
        return 0;
    }
    set_return(env, req, id, pos);
    warp_event(env, req, fm, id, pos)
}

/// C: `FUN_0218AD28` (unlabeled routine at `0x0218AD28`): the attribute events of the player's tile:
/// `FUN_021C8A74(a)` -> `FUN_021C8A30(0, gamesys, fm)`, else `FUN_021C8CF0(a)` -> `FUN_021C8C88(0, gamesys, fm)`.
pub fn tile_event<E: FieldEnv>(env: &mut E, req: u32, fm: u32) -> u32 {
    let mut pos = [0u8; 12];
    let pl = env.r32(req + 0x18);
    env.call(0x021A_3038, &mut [CArg::V(pl), CArg::OutBuf(&mut pos)]);
    let a = env.callv(0x0218_8C98, &[fm]);
    let b = env.call(0x021A_B0D4, &mut [CArg::V(a), CArg::InBuf(&pos)]);
    let t = env.callv(0x021A_B0F0, &[b]);
    if env.callv(0x021C_8A74, &[t]) != 0 {
        let gs = env.r32(req + 4);
        return env.callv(0x021C_8A30, &[0, gs, fm]);
    }
    if env.callv(0x021C_8CF0, &[t]) != 0 {
        let gs = env.r32(req + 4);
        return env.callv(0x021C_8C88, &[0, gs, fm]);
    }
    0
}

/// C: `FUN_0218AD80` (unlabeled routine at `0x0218AD80`): the warp event of warp `id` (`FUN_021626A8(o, pos)` gives
/// the target when `pos` is set): kind 6 `FUN_02185410`, else `FUN_02185360` (overlay 20, event_mapchange).
pub fn warp_event<E: FieldEnv>(env: &mut E, req: u32, fm: u32, id: u32, pos: u32) -> u32 {
    let zev = env.r32(req + 0xC);
    let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
    let w = if pos != 0 {
        env.call(0x0216_26A8, &mut [CArg::V(o), CArg::In(pos, 12)])
    } else {
        0
    };
    let k = env.callv(0x0216_26E8, &[o]);
    let gs = env.r32(req + 4);
    let f = if k == 6 { 0x0218_5410 } else { 0x0218_5360 };
    env.call(f, &mut [CArg::V(gs), CArg::P(fm), CArg::V(o), CArg::V(w)])
}

/// C: `FUN_0218ADC8` (unlabeled routine at `0x0218ADC8`): store the return location of warp `id` at `pos`:
/// `FUN_02014110(&loc, zone, (s16) id, 0, 0, pos[0], pos[1], pos[2])`, `FUN_02012878(gd, &loc)`.
pub fn set_return<E: FieldEnv>(env: &mut E, req: u32, id: u32, pos: u32) {
    let fm = env.r32(req + 0x14);
    let zone = env.callv(0x0218_8C9C, &[fm]);
    let (p0, p1, p2) = (env.r32(pos), env.r32(pos + 4), env.r32(pos + 8));
    let mut loc = [0u8; 0x1C];
    env.call(
        0x0201_4110,
        &mut [
            CArg::OutBuf(&mut loc),
            CArg::V(zone),
            CArg::V(id as u16 as i16 as i32 as u32),
            CArg::V(0),
            CArg::V(0),
            CArg::V(p0),
            CArg::V(p1),
            CArg::V(p2),
        ],
    );
    let gd = env.r32(req + 8);
    env.call(0x0201_2878, &mut [CArg::V(gd), CArg::InBuf(&loc)]);
}

/// C: `FUN_0218AE04` (unlabeled routine at `0x0218AE04`): the warp `FUN_02162564(evdata, pos)` at `pos` if it is of a
/// warp kind ([`is_warp_kind`]) or can be entered facing this way (`FUN_021626B8(o, facing)`, through the veneer
/// `FUN_0218AECC`); else 0xFFFF.
pub fn warp_at<E: FieldEnv>(env: &mut E, req: u32, pos: u32) -> u32 {
    let zev = env.r32(req + 0xC);
    let id = env.call(0x0216_2564, &mut [CArg::V(zev), CArg::In(pos, 12)]);
    if id == 0xFFFF {
        return 0xFFFF;
    }
    let zev = env.r32(req + 0xC);
    let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
    if is_warp_kind(env, o) {
        return id;
    }
    let f = env.r16(req + REQ_FACING) as u32;
    if env.callv(0x0218_AECC, &[o, f]) != 0 {
        id
    } else {
        0xFFFF
    }
}

/// C: `FUN_0218AEAC` (unlabeled routine at `0x0218AEAC`): the warp kind `FUN_021626E8(o)` is 0, 5 or 6.
pub fn is_warp_kind<E: FieldEnv>(env: &mut E, o: u32) -> bool {
    let k = env.callv(0x0216_26E8, &[o]);
    k <= 6 && (0x61 >> k) & 1 != 0
}

/// C: `FUN_0218AED4` (unlabeled routine at `0x0218AED4`): the bump warps: on a front tile with attribute bit 0,
/// the kind-1 warp under the player, else the warp in front.
pub fn bump_warp<E: FieldEnv>(env: &mut E, req: u32, _gs: u32, fm: u32) -> u32 {
    let front = env.mem().push_local(12);
    let r = bump_warp_at(env, req, fm, front);
    env.mem().pop_local(front);
    r
}

fn bump_warp_at<E: FieldEnv>(env: &mut E, req: u32, fm: u32, front: u32) -> u32 {
    front_position(env.mem(), req, front);
    let a = env.callv(0x0218_8C98, &[fm]);
    let b = env.call(0x021A_B0D4, &mut [CArg::V(a), CArg::In(front, 12)]);
    if env.callv(0x021A_B0F8, &[b]) & 1 == 0 {
        return 0;
    }
    let pos = env.r32(req + REQ_POS);
    let id = warp_at(env, req, pos);
    if id != 0xFFFF {
        let zev = env.r32(req + 0xC);
        let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
        if env.callv(0x0216_26EC, &[o]) == 0 && env.callv(0x0216_26E8, &[o]) == 1 {
            let pos = env.r32(req + REQ_POS);
            set_return(env, req, id, pos);
            let pos = env.r32(req + REQ_POS);
            return warp_event(env, req, fm, id, pos);
        }
    }
    let id = warp_at(env, req, front);
    if id == 0xFFFF {
        return 0;
    }
    let zev = env.r32(req + 0xC);
    let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
    if env.callv(0x0216_26EC, &[o]) != 0 {
        return 0;
    }
    let pos = env.r32(req + REQ_POS);
    set_return(env, req, id, pos);
    warp_event(env, req, fm, id, front)
}

/// C: `FUN_0218AF90` (unlabeled routine at `0x0218AF90`): the bump checks of the front tile: gimmick 6
/// (`FUN_021BE288(fm, 6)`) -> `FUN_021F3A88(gs, x >> 16, z >> 16)`; gimmicks 0x19..0x1B -> `FUN_021F38A4(fm,
/// facing)`; on a front tile with attribute bit 0: the waterfall script 0x271F (surfing, a party member knows field
/// move 0x7F ([`party_move_slot`]), attribute `FUN_021AB310` and the tile behind it is lower); with no warp in front,
/// gimmick 2 -> `FUN_021F404C(gs)`, gimmick 5 -> `FUN_021F37E0(gs)`.
pub fn bump_checks<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32) -> u32 {
    let front = env.mem().push_local(12);
    let p = env.mem().push_local(12);
    let r = bump_checks_at(env, req, gs, fm, front, p);
    env.mem().pop_local(p);
    env.mem().pop_local(front);
    r
}

fn bump_checks_at<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32, front: u32, p: u32) -> u32 {
    front_position(env.mem(), req, front);
    if env.callv(0x021B_E288, &[fm, 6]) != 0 {
        let (x, z) = (env.r32(front) as i32, env.r32(front + 8) as i32);
        return env.callv(
            0x021F_3A88,
            &[gs, (x / 0x10000) as u32, (z / 0x10000) as u32],
        );
    }
    if env.callv(0x021B_E288, &[fm, 0x19]) != 0
        || env.callv(0x021B_E288, &[fm, 0x1A]) != 0
        || env.callv(0x021B_E288, &[fm, 0x1B]) != 0
    {
        let f = env.r16(req + REQ_FACING) as u32;
        return env.callv(0x021F_38A4, &[fm, f]);
    }
    let a = env.callv(0x0218_8C98, &[fm]);
    let a = env.call(0x021A_B0D4, &mut [CArg::V(a), CArg::In(front, 12)]);
    if env.callv(0x021A_B0F8, &[a]) & 1 == 0 {
        return 0;
    }
    let pl = env.r32(req + 0x18);
    if env.callv(0x021A_3098, &[pl]) == 2 {
        let gd = env.r32(req + 8);
        if party_move_slot(env, gd, 0x7F) != 6 {
            let t = env.callv(0x021A_B0F0, &[a]);
            if env.callv(0x021A_B310, &[t]) == 1 {
                env.mem().copy(p, front, 12);
                let pl = env.r32(req + 0x18);
                let mm = env.callv(0x021A_3074, &[pl]);
                let f = env.r16(req + REQ_FACING) as u32;
                // C: `FUN_02164208(dir, &pos, 0x10000)`: move `pos` one tile in direction `dir`
                env.call(
                    0x0216_4208,
                    &mut [CArg::V(f), CArg::In(p, 12), CArg::V(0x10000)],
                );
                let mut h = [0u8; 4];
                let ok = env.call(
                    0x0216_3E58,
                    &mut [CArg::V(mm), CArg::In(p, 12), CArg::OutBuf(&mut h)],
                );
                if ok == 1 && (env.r32(front + 4) as i32) > i32::from_le_bytes(h) {
                    let (heap, gs) = (env.r16(req) as u32, env.r32(req + 4));
                    return env.callv(SCRIPT_EVENT, &[gs, 0x271F, 0, heap]);
                }
            }
        }
    }
    if warp_at(env, req, front) != 0xFFFF {
        return 0;
    }
    if env.callv(0x021B_E288, &[fm, 2]) != 0 {
        return env.callv(0x021F_404C, &[gs]);
    }
    if env.callv(0x021B_E288, &[fm, 5]) != 0 {
        return env.callv(0x021F_37E0, &[gs]);
    }
    0
}

/// The player's rail position (`FUN_0219E2F4(mmdl, &rp)`, 8 bytes) into the local `rp`.
fn rail_position<E: FieldEnv>(env: &mut E, fm: u32, rp: u32) {
    let pl = env.callv(0x0218_8CB4, &[fm]);
    let mm = env.callv(0x021A_3074, &[pl]);
    env.call(0x0219_E2F4, &mut [CArg::V(mm), CArg::Out(rp, 8)]);
}

/// C: `FUN_0218B0CC` (unlabeled routine at `0x0218B0CC`): the rail-map warp under the player (`FUN_02162598(evdata,
/// &rp)`), enabled and of a warp kind: [`rail_set_return`] and [`rail_warp_event`].
pub fn rail_warp_check<E: FieldEnv>(env: &mut E, req: u32, _gs: u32, fm: u32) -> u32 {
    let rp = env.mem().push_local(8);
    let r = (|| {
        rail_position(env, fm, rp);
        let zev = env.r32(req + 0xC);
        let id = env.call(0x0216_2598, &mut [CArg::V(zev), CArg::In(rp, 8)]);
        if id == 0xFFFF {
            return 0;
        }
        let zev = env.r32(req + 0xC);
        let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
        if env.callv(0x0216_26EC, &[o]) == 1 || !is_warp_kind(env, o) {
            return 0;
        }
        rail_set_return(env, req, id, rp);
        rail_warp_event(env, req, fm, id, rp)
    })();
    env.mem().pop_local(rp);
    r
}

/// C: `FUN_0218B13C` (unlabeled routine at `0x0218B13C`): the rail-map bump warps: the kind-1 warp under the player
/// that can be entered facing this way, else the warp at `rp2`; both use the player's rail position.
pub fn rail_bump_warp<E: FieldEnv>(env: &mut E, req: u32, _gs: u32, fm: u32, rp2: u32) -> u32 {
    let rp = env.mem().push_local(8);
    let r = (|| {
        rail_position(env, fm, rp);
        let zev = env.r32(req + 0xC);
        let id = env.call(0x0216_2598, &mut [CArg::V(zev), CArg::In(rp, 8)]);
        if id != 0xFFFF {
            let zev = env.r32(req + 0xC);
            let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
            if env.callv(0x0216_26EC, &[o]) == 0 {
                let f = env.r16(req + REQ_FACING) as u32;
                if env.callv(0x0218_AECC, &[o, f]) != 0 && env.callv(0x0216_26E8, &[o]) == 1 {
                    rail_set_return(env, req, id, rp);
                    return rail_warp_event(env, req, fm, id, rp);
                }
            }
        }
        let zev = env.r32(req + 0xC);
        let id = env.call(0x0216_2598, &mut [CArg::V(zev), CArg::In(rp2, 8)]);
        if id == 0xFFFF {
            return 0;
        }
        let zev = env.r32(req + 0xC);
        let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
        if env.callv(0x0216_26EC, &[o]) != 0 {
            return 0;
        }
        let f = env.r16(req + REQ_FACING) as u32;
        if env.callv(0x0218_AECC, &[o, f]) == 0 {
            return 0;
        }
        rail_set_return(env, req, id, rp);
        rail_warp_event(env, req, fm, id, rp)
    })();
    env.mem().pop_local(rp);
    r
}

/// C: `FUN_0218B208` (unlabeled routine at `0x0218B208`): on a step (FLAGS bit 3) the wild-encounter check of the
/// encounter object `FUN_02188CE8(fm)`: `FUN_021AA248(e, FUN_0218DC70(FUN_02188C98(fm)))` in zones where
/// `FUN_02013DD0(zone)` holds, else `FUN_021A9F40(e, 0)`.
pub fn step_encounter<E: FieldEnv>(env: &mut E, req: u32, _gs: u32, fm: u32) -> u32 {
    let e = env.callv(0x0218_8CE8, &[fm]);
    if env.r32(req + REQ_FLAGS) & 8 == 0 {
        return 0;
    }
    let zone = env.r16(req + 0x10) as u32;
    if env.callv(0x0201_3DD0, &[zone]) != 0 {
        let a = env.callv(0x0218_8C98, &[fm]);
        let b = env.callv(0x0218_DC70, &[a]);
        return env.callv(0x021A_A248, &[e, b]);
    }
    env.callv(0x021A_9F40, &[e, 0])
}

/// C: `FUN_0218B24C` (unlabeled routine at `0x0218B24C`): pushing down (direction 1) while standing or walking, on a
/// tile where `FUN_021BB188(FUN_021AB0F0(+0x38))` holds and `FUN_0219E418(mmdl, 1) & 3`: `FUN_021C46F8(gs, fm, 1)`.
pub fn push_down_event<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32) -> u32 {
    let pl = env.callv(0x0218_8CB4, &[fm]);
    let mm = env.callv(0x021A_3074, &[pl]);
    let t = env.r32(req + 0x2C);
    let d = env.callv(0x021A_2FA4, &[pl, t]);
    if d != 1 {
        return 0;
    }
    let st = env.r32(req + 0x1C);
    if st != 0 && st != 3 {
        return 0;
    }
    let x = env.r32(req + 0x38);
    let a = env.callv(0x021A_B0F0, &[x]);
    if env.callv(0x021B_B188, &[a]) == 0 {
        return 0;
    }
    if env.callv(0x0219_E418, &[mm, d]) & 3 == 0 {
        return 0;
    }
    env.callv(0x021C_46F8, &[gs, fm, d])
}

/// C: `FUN_0218B2A8` (unlabeled routine at `0x0218B2A8`): store the rail-map return location of warp `id` at the rail
/// position `rp`: `FUN_02014134(&loc, zone, (s16) id, 0, 0, u16 rp+0, u16 rp+6, s16 rp+4)`, `FUN_02012878(gd, &loc)`.
pub fn rail_set_return<E: FieldEnv>(env: &mut E, req: u32, id: u32, rp: u32) {
    let fm = env.r32(req + 0x14);
    let zone = env.callv(0x0218_8C9C, &[fm]);
    let (a, b, c) = (
        env.r16(rp) as u32,
        env.r16(rp + 6) as u32,
        env.r16(rp + 4) as i16 as i32 as u32,
    );
    let mut loc = [0u8; 0x1C];
    env.call(
        0x0201_4134,
        &mut [
            CArg::OutBuf(&mut loc),
            CArg::V(zone),
            CArg::V(id as u16 as i16 as i32 as u32),
            CArg::V(0),
            CArg::V(0),
            CArg::V(a),
            CArg::V(b),
            CArg::V(c),
        ],
    );
    let gd = env.r32(req + 8);
    env.call(0x0201_2878, &mut [CArg::V(gd), CArg::InBuf(&loc)]);
}

/// C: `FUN_0218B2E8` (unlabeled routine at `0x0218B2E8`): the rail-map warp event of warp `id`:
/// `FUN_02185360(gs, fm, o, FUN_021626B0(o, rp))`.
pub fn rail_warp_event<E: FieldEnv>(env: &mut E, req: u32, fm: u32, id: u32, rp: u32) -> u32 {
    let zev = env.r32(req + 0xC);
    let o = env.callv(0x0216_25CC, &[zev, id & 0xFFFF]);
    let w = env.call(0x0216_26B0, &mut [CArg::V(o), CArg::In(rp, 8)]);
    let gs = env.r32(req + 4);
    env.call(
        0x0218_5360,
        &mut [CArg::V(gs), CArg::P(fm), CArg::V(o), CArg::V(w)],
    )
}

/// C: `FUN_0218B310` (unlabeled routine at `0x0218B310`): `FUN_0219E418(mmdl of player, facing) & 3 == 0`.
pub fn facing_free<E: FieldEnv>(env: &mut E, pl: u32, req: u32) -> bool {
    let mm = env.callv(0x021A_3074, &[pl]);
    let f = env.r16(req + REQ_FACING) as u32;
    env.callv(0x0219_E418, &[mm, f]) & 3 == 0
}

/// C: `FUN_0218B32C` (unlabeled routine at `0x0218B32C`): the GMEVENT callback of the ledge jump started by
/// [`jump_event`] (`w`: `+4 u16` direction, `+6 u16` long jump, `+8` gamesys, `+0xC` FIELDMAP). Step 0: stop the
/// MMDL's animation (`FUN_0216D9A0(FUN_0216D67C(mmdl))`) and face `dir`; 1: `FUN_021ACBB8(FUN_021A2F98(p), 0)`,
/// `FUN_021A2F48(p, 1)`, `FUN_021A2F60(p)`, start the jump animation `FUN_02197E7C(dir, long ? 0x38 : 0x34)`;
/// 2: wait for it (`FUN_0216D454(mmdl) == 1`, then `FUN_021A2FD0(p)`); 3: `FUN_0216D9E4(..)`, `FUN_021A35A0(p, 1)`,
/// done (1). Steps advance `*seq`.
pub fn jump_step<E: FieldEnv>(env: &mut E, _ev: u32, seq: u32, w: u32) -> u32 {
    let fm = env.r32(w + 0xC);
    let p = env.callv(0x0218_8CB4, &[fm]);
    env.callv(0x021A_3098, &[p]);
    let mm = env.callv(0x021A_3074, &[p]);
    let next = |env: &mut E| {
        let v = env.r32(seq).wrapping_add(1);
        env.w32(seq, v);
    };
    match env.r32(seq) {
        0 => {
            let a = env.callv(0x0216_D67C, &[mm]);
            env.callv(0x0216_D9A0, &[a]);
            let d = env.r16(w + 4) as u32;
            env.callv(0x0216_D5E0, &[mm, d]);
            next(env);
        }
        1 => {
            let a = env.callv(0x021A_2F98, &[p]);
            env.callv(0x021A_CBB8, &[a, 0]);
            // the spec's stub lists the first argument
            env.call(0x021A_2F48, &mut [CArg::V(p), CArg::X(1)]);
            env.callv(0x021A_2F60, &[p]);
            let d = env.r16(w + 4) as u32;
            let mut a = env.callv(0x0219_7E7C, &[d, 0x34]);
            if env.r16(w + 6) == 1 {
                let d = env.r16(w + 4) as u32;
                a = env.callv(0x0219_7E7C, &[d, 0x38]);
            }
            env.callv(0x0216_D410, &[mm, a]);
            next(env);
        }
        2 => {
            if env.callv(0x0216_D454, &[mm]) == 1 {
                env.callv(0x021A_2FD0, &[p]);
                next(env);
            }
        }
        3 => {
            let a = env.callv(0x0216_D67C, &[mm]);
            env.callv(0x0216_D9E4, &[a]);
            env.callv(0x021A_35A0, &[p, 1]);
            return 1;
        }
        _ => {}
    }
    0
}

/// C: `FUN_0218B3DC` (unlabeled routine at `0x0218B3DC`): create the ledge-jump GMEVENT (`FUN_020122C0(gs, 0,
/// FUN_0218B32C, 0x10)`) and fill its work ([`jump_step`]); returns it.
pub fn jump_event<E: FieldEnv>(
    env: &mut E,
    _req: u32,
    gs: u32,
    fm: u32,
    dir: u32,
    long: u32,
) -> u32 {
    let ev = env.callv(0x0201_22C0, &[gs, 0, 0x0218_B32D, 0x10]);
    // C: `FUN_020124E8(ev)`: the event's work
    let w = env.callv(0x0201_24E8, &[ev]);
    env.w32(w + 8, gs);
    env.w32(w + 0xC, fm);
    env.w16(w + 4, dir as u16);
    env.w16(w + 6, long as u16);
    ev
}

/// C: `FUN_0218B408` (unlabeled routine at `0x0218B408`): the ledge jump: when the player state is 2 and a direction
/// `d` is pushed, the tile `t` in that direction (`FUN_021A3364`); a ledge attribute (`FUN_021AB2E8 == 1`) makes it a
/// long jump over one more tile (`FUN_02164208(d, &t, 0x10000)`); with both landing tiles free
/// ([`jump_tiles_free`]) and the landing attribute bits 0 and 1 clear, [`jump_event`].
pub fn ledge_check<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32) -> u32 {
    let t = env.mem().push_local(12);
    let r = ledge_check_at(env, req, gs, fm, t);
    env.mem().pop_local(t);
    r
}

fn ledge_check_at<E: FieldEnv>(env: &mut E, req: u32, gs: u32, fm: u32, t: u32) -> u32 {
    let p = env.callv(0x0218_8CB4, &[fm]);
    if env.callv(0x021A_3098, &[p]) != 2 {
        return 0;
    }
    let a0 = env.callv(0x0218_8C98, &[fm]);
    let (pl, tr) = (env.r32(req + 0x18), env.r32(req + 0x2C));
    let d = env.callv(0x021A_2FA4, &[pl, tr]);
    if d == 9 {
        return 0;
    }
    env.call(0x021A_3364, &mut [CArg::V(p), CArg::V(d), CArg::Out(t, 12)]);
    let b = env.call(0x021A_B0D4, &mut [CArg::V(a0), CArg::In(t, 12)]);
    let mut bits = env.callv(0x021A_B0F8, &[b]);
    let c = env.callv(0x021A_B0F0, &[b]);
    let mut long = 0;
    if env.callv(0x021A_B2E8, &[c]) == 1 {
        env.call(
            0x0216_4208,
            &mut [CArg::V(d), CArg::In(t, 12), CArg::V(0x10000)],
        );
        let b = env.call(0x021A_B0D4, &mut [CArg::V(a0), CArg::In(t, 12)]);
        bits = env.callv(0x021A_B0F8, &[b]);
        long = 1;
    }
    if jump_tiles_free(env, req, fm) && bits & 1 == 0 && bits & 2 == 0 {
        return jump_event(env, req, gs, fm, d, long);
    }
    0
}

/// C: `FUN_0218B4B0` (unlabeled routine at `0x0218B4B0`): the MMDL to talk to: the tile in front of the player
/// (`FUN_021A3460` tile x/y/z; on a counter attribute `FUN_021AB194 == 1` one tile further, `FUN_02164240`), then
/// `FUN_0216DDDC(mmdl system, x, z, height, 0x10000, 0)`.
pub fn talk_target<E: FieldEnv>(env: &mut E, req: u32, fm: u32) -> u32 {
    let xyz = env.mem().push_local(8);
    let pos = env.mem().push_local(12);
    let r = talk_target_at(env, req, fm, xyz, pos);
    env.mem().pop_local(pos);
    env.mem().pop_local(xyz);
    r
}

fn talk_target_at<E: FieldEnv>(env: &mut E, req: u32, fm: u32, xyz: u32, pos: u32) -> u32 {
    // the original's stack layout: z at +0, y at +2, x at +4 (all s16)
    let (z, y, x) = (xyz, xyz + 2, xyz + 4);
    let pl = env.r32(req + 0x18);
    let mm = env.callv(0x021A_3074, &[pl]);
    let pl = env.r32(req + 0x18);
    env.call(
        0x021A_3460,
        &mut [
            CArg::V(pl),
            CArg::Out(x, 2),
            CArg::Out(y, 2),
            CArg::Out(z, 2),
        ],
    );
    let (vx, vy, vz) = (
        env.r16(x) as i16 as i32 as u32,
        env.r16(y) as i16 as i32 as u32,
        env.r16(z) as i16 as i32 as u32,
    );
    env.call(
        0x0216_4294,
        &mut [CArg::V(vx), CArg::V(vy), CArg::V(vz), CArg::Out(pos, 12)],
    );
    let a = env.callv(0x0218_8C98, &[fm]);
    let b = env.call(0x021A_B0D4, &mut [CArg::V(a), CArg::In(pos, 12)]);
    let c = env.callv(0x021A_B0F0, &[b]);
    if env.callv(0x021A_B194, &[c]) == 1 {
        let f = env.callv(0x0216_D5F8, &[mm]);
        // C: `FUN_02164240(dir, &x, &z, 1)`: moves the tile x / z one step in direction `dir`
        env.call(
            0x0216_4240,
            &mut [CArg::V(f), CArg::Out(x, 2), CArg::Out(z, 2), CArg::V(1)],
        );
    }
    let sys = env.callv(0x0218_8C7C, &[fm]);
    let h = env.callv(0x0216_D8AC, &[mm]);
    let (vx, vz) = (
        env.r16(x) as i16 as i32 as u32,
        env.r16(z) as i16 as i32 as u32,
    );
    env.callv(0x0216_DDDC, &[sys, vx, vz, h, 0x10000, 0])
}

/// C: `FUN_0218B53C` (unlabeled routine at `0x0218B53C`): the rail-map MMDL to talk to:
/// `FUN_0219E634(mmdl, &rp)` ? `FUN_0219E570(mmdl system, &rp, 0)` : 0.
pub fn rail_talk_target<E: FieldEnv>(env: &mut E, req: u32, fm: u32) -> u32 {
    let rp = env.mem().push_local(8);
    let pl = env.r32(req + 0x18);
    let mm = env.callv(0x021A_3074, &[pl]);
    let r = if env.call(0x0219_E634, &mut [CArg::V(mm), CArg::Out(rp, 8)]) != 0 {
        let sys = env.callv(0x0218_8C7C, &[fm]);
        env.call(
            0x0219_E570,
            &mut [CArg::V(sys), CArg::In(rp, 8), CArg::V(0)],
        )
    } else {
        0
    };
    env.mem().pop_local(rp);
    r
}

/// ROM `0x021D3CE8`: the 13 attribute scripts of [`attribute_script`]: `(predicate, direction (9 = any), script)`.
pub const ATTRIBUTE_SCRIPTS: [(u32, u16, u16); 13] = [
    (0x021A_B1A1, 0, 0x276A),
    (0x021A_B1AD, 9, 0xFFFF),
    (0x021A_B1B9, 0, 0x2918),
    (0x021A_B1D1, 9, 0x09C4),
    (0x021A_B1DD, 9, 0x09C5),
    (0x021A_B1E9, 9, 0x09C6),
    (0x021A_B1F5, 9, 0x09C7),
    (0x021A_B201, 9, 0x09C8),
    (0x021A_B20D, 9, 0x09C8),
    (0x021A_B219, 9, 0x09C9),
    (0x021A_B225, 9, 0x09CA),
    (0x021A_B231, 9, 0x09CB),
    (0x021A_B1C5, 0, 0x28A0),
];

/// C: `FUN_0218B56C` (unlabeled routine at `0x0218B56C`): the script of the attribute in front of the player (0xFFFF =
/// none; nothing while `FUN_02012B70(gd) == 1`): the first [`ATTRIBUTE_SCRIPTS`] entry for this facing whose
/// predicate holds; then, in zones where `FUN_02013E04(zone)` holds, 0x2712 (`FUN_021A3504`, with a party member that
/// knows field move 0x39) or 0x2716 (`FUN_021A3524`).
pub fn attribute_script<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    let gd = env.r32(req + 8);
    if env.callv(0x0201_2B70, &[gd]) == 1 {
        return 0xFFFF;
    }
    let pl = env.r32(req + 0x18);
    let a = env.callv(0x021A_30C8, &[pl]);
    let (pl, f) = (env.r32(req + 0x18), env.r16(req + REQ_FACING) as u32);
    let b = env.callv(0x021A_3128, &[pl, f]);
    let t = env.callv(0x021A_B0F0, &[b]);
    for &(pred, dir, script) in ATTRIBUTE_SCRIPTS.iter() {
        let f = env.r16(req + REQ_FACING);
        if (f == dir || dir == 9) && env.callv(pred & !1, &[t]) != 0 {
            return script as u32;
        }
    }
    let zone = env.r16(req + 0x10) as u32;
    if env.callv(0x0201_3E04, &[zone]) == 0 {
        return 0xFFFF;
    }
    let gd = env.r32(req + 8);
    if env.callv(0x0201_2B70, &[gd]) == 1 {
        return 0xFFFF;
    }
    let pl = env.r32(req + 0x18);
    if env.callv(0x021A_3504, &[pl, a, b]) != 0 {
        let gd = env.r32(req + 8);
        if party_move_slot(env, gd, 0x39) != 6 {
            return 0x2712;
        }
    }
    let pl = env.r32(req + 0x18);
    if env.callv(0x021A_3524, &[pl, a, b]) != 0 {
        return 0x2716;
    }
    0xFFFF
}

/// C: `FUN_0218B630` (unlabeled routine at `0x0218B630`): the first party slot holding a non-egg that knows `mv` (one
/// of its move params 0x36..0x39), else 6.
pub fn party_move_slot<E: FieldEnv>(env: &mut E, gd: u32, mv: u32) -> u32 {
    let party = env.callv(0x0201_2934, &[gd]);
    let n = env.callv(PARTY_COUNT, &[party]) as i32;
    for i in 0..n.max(0) as u32 {
        let m = env.callv(PARTY_MON, &[party, i]);
        if env.callv(MON_GET, &[m, 0x4C, 0]) != 0 {
            continue;
        }
        for k in 0x36..=0x39 {
            if mv == env.callv(MON_GET, &[m, k, 0]) {
                return i;
            }
        }
    }
    6
}

/// C: `FUN_0218B6AC` (unlabeled routine at `0x0218B6AC`): the number of non-egg party members whose param 0xA0 is
/// nonzero.
pub fn party_count_a0<E: FieldEnv>(env: &mut E, gs: u32) -> u32 {
    let gd = env.callv(0x0201_20F4, &[gs]);
    let party = env.callv(0x0201_2934, &[gd]);
    let n = env.callv(PARTY_COUNT, &[party]) as i32;
    let mut c = 0;
    for i in 0..n.max(0) as u32 {
        let m = env.callv(PARTY_MON, &[party, i]);
        if env.callv(MON_GET, &[m, 0x4C, 0]) == 0 && env.callv(MON_GET, &[m, 0xA0, 0]) != 0 {
            c += 1;
        }
    }
    c
}

/// C: `FUN_0218B6F8` (unlabeled routine at `0x0218B6F8`): the two tiles past the player in the trigger direction are
/// free of MMDLs (`FUN_0216DDC0(mmdl system, x, z, 1) == 0`; the tile steps `FUN_021641E8` / `FUN_021641F8` are
/// added as `s16`).
pub fn jump_tiles_free<E: FieldEnv>(env: &mut E, req: u32, fm: u32) -> bool {
    let pl = env.r32(req + 0x18);
    let mm = env.callv(0x021A_3074, &[pl]);
    let d = env.r16(req + 0x30) as u32;
    let mut x = env.callv(0x0216_D83C, &[mm]);
    let mut z = env.callv(0x0216_D86C, &[mm]);
    for _ in 0..2 {
        x = x.wrapping_add(env.callv(0x0216_41E8, &[d])) as u16 as i16 as i32 as u32;
        z = z.wrapping_add(env.callv(0x0216_41F8, &[d])) as u16 as i16 as i32 as u32;
        let sys = env.callv(0x0218_8C7C, &[fm]);
        if env.callv(0x0216_DDC0, &[sys, x, z, 1]) != 0 {
            return false;
        }
    }
    true
}

/// C: `FUN_0218AE48` (unlabeled routine at `0x0218AE48`): the tile in front of the player: `out` = the
/// three words of `*(req + 0x34)`, then by the facing direction `z -= 1.0` (0), `z += 1.0` (1),
/// `x -= 1.0` (2), `x += 1.0` (3) in fx32 (`0x10000`); other directions leave the position.
pub fn front_position(mem: &mut Mem, req: u32, out: u32) {
    // the original's order (`ldm` 2 words, `stm` them, then the third), so an overlap behaves the same
    let p = mem.r32(req + REQ_POS);
    let (x, y) = (mem.r32(p), mem.r32(p + 4));
    mem.w32(out, x);
    mem.w32(out + 4, y);
    let z = mem.r32(p + 8);
    mem.w32(out + 8, z);
    let step = |mem: &mut Mem, at: u32, up: bool| {
        let v = mem.r32(at);
        mem.w32(
            at,
            if up {
                v.wrapping_add(0x10000)
            } else {
                v.wrapping_sub(0x10000)
            },
        );
    };
    match mem.r16(req + REQ_FACING) {
        0 => step(mem, out + 8, false),
        1 => step(mem, out + 8, true),
        2 => step(mem, out, false),
        3 => step(mem, out, true),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
