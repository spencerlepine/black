//! `event_field_proclink.c` (F1 row 10.22, overlay 10, `0x02160A94..0x02161D96`; F14): the logic around starting the
//! sub-applications (party list, bag, Pokedex, trainer card, ...) from the field and returning to it.
//!
//! The field PROCLINK table (ROM `0x021712CC`, 12 entries of 0x18 bytes `{overlay, proc data, init, return, special,
//! free}`, read by the proclink event proc at `0x0216075C..0x02160A92`, which F1 put in row 10.21) names, per app, an
//! `init(pl, mode, kind, prev)` that builds the parameter block handed to the app (from the field state and from the
//! parameter block `prev` of the app that ran before it: `kind` says which one) and a `return(pl, param)` that reads
//! the app's results back into the PROCLINK object and picks the next action (`pl+0xC`, return value 1 = back to the
//! field, 2 = the event's result in `S+0x28`, 3 / 4 = switch apps). The apps themselves are U-phase; here only these
//! callbacks and the field GMEVENTs used around the launches (pause the OBJs, an acmd list on the player, the
//! jump-down move, the fade + sub-proc event) are ported. Every app / system callee is a hook reached through
//! [`FieldEnv::call`] by address.
//!
//! PROCLINK object `pl` (as read here): `+0x08` return kind, `+0x0C` next action, `+0x18` -> the proclink event work
//! `S` (`S+0` GAMESYS, `S+4` FIELDMAP, `S+0xC` state, `S+0x28` result, `S+0x2C` slot, `S+0x30..0x3C` params), `+0x34`
//! the bag's item word, `+0x4C..0x5F` copied into the party-list parameter, `+0x60 u8` party slot, `+0x62 u16` item,
//! `+0x64`, `+0x68` sub-state, `+0x6C` mode flag.
//!
//! Verified by `tools/check-fn.sh black-field proclink --per-function` (spec
//! `tools/rom-diff/specs/black-field/proclink.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_0A94,
    0x0216_0C88,
    0x0216_0D6C,
    0x0216_0DDC,
    0x0216_0E08,
    0x0216_0F88,
    0x0216_0FB4,
    0x0216_0FE0,
    0x0216_1070,
    0x0216_117C,
    0x0216_11E0,
    0x0216_1214,
    0x0216_1280,
    0x0216_1308,
    0x0216_1340,
    0x0216_1350,
    0x0216_1378,
    0x0216_13A4,
    0x0216_13CC,
    0x0216_1484,
    0x0216_1534,
    0x0216_1554,
    0x0216_1610,
    0x0216_163C,
    0x0216_1668,
    0x0216_16A4,
    0x0216_16F0,
    0x0216_1724,
    0x0216_1824,
    0x0216_1850,
    0x0216_18A0,
    0x0216_18E0,
    0x0216_1938,
    0x0216_1950,
    0x0216_1964,
    0x0216_1984,
    0x0216_19B8,
    0x0216_1AC0,
    0x0216_1B1C,
    0x0216_1B74,
    0x0216_1B90,
    0x0216_1BF8,
    0x0216_1C84,
    0x0216_1CDC,
    0x0216_1D18,
    0x0216_1D64,
];

/// Address of the string `"event_field_proclink.c"` (the file argument of the parameter allocations).
pub const FILE: u32 = 0x0217_2978;
/// The heap the parameter blocks are allocated on.
const HEAP: u32 = 4;
/// The overlay the item screens (entry 9) live in.
const OVERLAY_ITEM: u32 = 0x88;

/// C: `FUN_020120F4(gs)`: the GAMEDATA.
const GAMEDATA: u32 = 0x0201_20F4;
/// C: `FUN_02012934(gd)`: the party.
const PARTY: u32 = 0x0201_2934;
/// C: `FUN_0201AA18(party)`: party count.
const PARTY_COUNT: u32 = 0x0201_AA18;
/// C: `FUN_0201AB28(party, i)`: the i-th party Pokemon.
const PARTY_MON: u32 = 0x0201_AB28;
/// C: `FUN_02017E1C(mon, field, 0)`: a Pokemon field.
const MON_GET: u32 = 0x0201_7E1C;
/// C: `FUN_02034AC4(id)` / `FUN_02034A5C(id)`: load / unload an overlay.
const OVERLAY_LOAD: u32 = 0x0203_4AC4;
const OVERLAY_UNLOAD: u32 = 0x0203_4A5C;
/// C: `FUN_020122C0(gsys, parent, proc, work size)` / `FUN_020124E8(event)`: GMEVENT create / work.
const EVENT_NEW: u32 = 0x0201_22C0;
const EVENT_WORK: u32 = 0x0201_24E8;
/// C: `FUN_02188C7C(fm)`: the MMDL system.
const MMDL_SYS: u32 = 0x0218_8C7C;
/// C: `FUN_02188CB4(fm)` / `FUN_021A3074(player)`: the player / its OBJ.
const PLAYER: u32 = 0x0218_8CB4;
const PLAYER_OBJ: u32 = 0x021A_3074;
/// C: `FUN_0216D5F8(obj)`: the OBJ's direction.
const OBJ_DIR: u32 = 0x0216_D5F8;
/// C: `FUN_02197D3C(obj, list)` / `FUN_02197DA0(task)` / `FUN_02197DAC(task)`: start an acmd list / ended / free.
const ACMD_START: u32 = 0x0219_7D3C;
const ACMD_ENDED: u32 = 0x0219_7DA0;
const ACMD_FREE: u32 = 0x0219_7DAC;
/// C: `FUN_020061E4(se)`: play a sound effect.
const PLAY_SE: u32 = 0x0200_61E4;

/// The ROM acmd lists started on the player's OBJ by direction 0..3 (`0x02171404 / 0x021713FC / 0x021713F4 /
/// 0x021713EC`).
pub const DIR_LISTS: [u32; 4] = [0x0217_1404, 0x0217_13FC, 0x0217_13F4, 0x0217_13EC];

/// The jump-down height arc (ROM `0x0217140C`, fx32, read at the even indices 0..14; index 16 is the next ROM word).
pub const ARC: [u32; 17] = [
    0x2000, 0x4000, 0x6000, 0x8000, 0x9000, 0xA000, 0xA000, 0xA000, 0x9000, 0x8000, 0x6000, 0x5000,
    0x3000, 0x2000, 0, 0, 1,
];

/// `S`, the proclink event work `pl+0x18` points to.
fn work_s<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    env.r32(pl + 0x18)
}

/// The GAMESYS (`S+0`).
fn gs_of<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    let s = work_s(env, pl);
    env.r32(s)
}

/// `FUN_020120F4(gs)`, the GAMEDATA.
fn gd_of<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    let gs = gs_of(env, pl);
    env.callv(GAMEDATA, &[gs])
}

/// `Heap_AllocDebug(4, size, flag, "event_field_proclink.c", line)`.
fn alloc<E: FieldEnv>(env: &mut E, size: u32, flag: u32, line: u32) -> u32 {
    env.callv(HEAP_ALLOC, &[HEAP, size, flag, FILE, line])
}

/// Copy `n` words from `src` to `dst`.
fn copy_words<E: FieldEnv>(env: &mut E, dst: u32, src: u32, n: u32) {
    for k in 0..n {
        let v = env.r32(src + 4 * k);
        env.w32(dst + 4 * k, v);
    }
}

/// `*seq += 1` (a GMEVENT's sequence word).
fn next_seq<E: FieldEnv>(env: &mut E, seq: u32) {
    let s = env.r32(seq).wrapping_add(1);
    env.w32(seq, s);
}

/// C: `FUN_02160A94(pl, mode, kind, prev)`: init of entry 0 (overlay 0x5B, the party list): a 0x98-byte parameter
/// block (party, field data, the copy of `pl+0x4C..0x5F`, the slot `S+0x2C`); mode 5 is the special mode (`pl+0x6C`
/// = 1); `kind` 2 / 6 / 9 / 10 set the list's mode from the previous app's result (`prev`).
pub fn party_init<E: FieldEnv>(env: &mut E, pl: u32, mode: u32, kind: u32, prev: u32) -> u32 {
    let gd = gd_of(env, pl);
    let p = alloc(env, 0x98, 1, 0x330);
    let party = env.callv(PARTY, &[gd]);
    env.w32(p, party);
    copy_words(env, p + 0x1C, pl + 0x4C, 5);
    let v = env.callv(0x0201_292C, &[gd]);
    env.w32(p + 4, v);
    let v = env.callv(0x0200_8FEC, &[gd]);
    env.w32(p + 8, v);
    let v = env.callv(0x0200_CA1C, &[gd]);
    env.w32(p + 0x18, v);
    env.w32(p + 0x34, 0);
    let s = work_s(env, pl);
    let slot = env.r32(s + 0x2C);
    env.w32(p + 0x3C, slot);
    env.w16(p + 0x46, 0);
    env.w8(p + 0x5E, 0xFF);
    let z = env.callv(0x0201_27A4, &[gd]);
    let zone = env.r16(z);
    env.w16(p + 0x30, zone);
    let v = env.callv(0x0201_2984, &[gd]);
    env.w8(p + 0x5F, v as u8);
    if mode == 5 {
        env.w32(p + 0x3C, 0);
        env.w32(p + 0x34, 5);
        env.w16(p + 0x44, 0x1D2);
        env.w32(pl + 0x6C, 1);
        return p;
    }
    match kind {
        0 => env.w32(p + 0x3C, 0),
        2 => {
            let item = env.r32(prev + 0x38);
            env.w16(p + 0x44, item as u16);
            if !matches!(env.r32(prev + 0x34), 1 | 3 | 4 | 18) {
                env.w32(p + 0x3C, 0);
            }
            let m = match env.r32(prev + 0x34) {
                1 => (env.r32(prev + 0x2C) == 2).then_some(0),
                3 | 4 | 18 => Some(10),
                5 => Some(5),
                6 => Some(6),
                12 => Some(16),
                _ => Some(9),
            };
            if let Some(m) = m {
                env.w32(p + 0x34, m);
            }
        }
        6 => {
            let b = env.r8(prev + 0x12);
            if b > 1 {
                return p;
            }
            if env.r8(prev + 0xD) == 2 {
                if env.r32(pl + 0x68) == 6 {
                    env.w32(p + 0x34, 8);
                    let v = env.r32(pl + 0x64);
                    env.w32(p + 0x50, v);
                } else {
                    env.w32(p + 0x34, 7);
                }
                let v = env.r16(pl + 0x62);
                env.w16(p + 0x44, v);
                let v = env.r16(prev + 0x14);
                env.w16(p + 0x46, v);
                let v = if env.r8(prev + 0x12) == 0 {
                    env.r8(prev + 0x11)
                } else {
                    0xFF
                };
                env.w8(p + 0x48, v);
            } else {
                env.w32(p + 0x34, 0);
                let v = env.r8(prev + 0xF);
                env.w32(p + 0x3C, u32::from(v));
            }
        }
        9 => {
            env.callv(OVERLAY_LOAD, &[OVERLAY_ITEM]);
            let r = env.callv(0x021C_3E00, &[prev]);
            env.callv(OVERLAY_UNLOAD, &[OVERLAY_ITEM]);
            match env.r32(pl + 0x68) {
                0 => {
                    env.w32(p + 0x34, 11);
                    let v = env.r8(pl + 0x60);
                    env.w32(p + 0x3C, u32::from(v));
                    let v = env.r16(pl + 0x62);
                    env.w16(p + 0x44, v);
                }
                2 if r == 1 => {
                    env.w32(p + 0x34, 12);
                    let v = env.r8(pl + 0x60);
                    env.w32(p + 0x3C, u32::from(v));
                    let v = env.r16(pl + 0x62);
                    env.w16(p + 0x44, v);
                }
                2 => {
                    env.w32(p + 0x34, 0);
                    let v = env.r8(pl + 0x60);
                    env.w32(p + 0x3C, u32::from(v));
                }
                _ => {}
            }
        }
        10 => {
            env.w32(p + 0x34, 5);
            let v = env.r16(pl + 0x62);
            env.w16(p + 0x44, v);
        }
        _ => {}
    }
    p
}

/// C: `FUN_02160C88(pl, param)`: return of entry 0: the chosen slot goes to `S+0x2C` and `pl+0x60`; in the special
/// mode the answer is "slot 8 chosen"; else the next action by the list's end mode (`param+0x40`).
pub fn party_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let s = work_s(env, pl);
    let slot = env.r32(p + 0x3C);
    env.w32(s + 0x2C, slot);
    let slot = env.r32(p + 0x3C);
    env.w8(pl + 0x60, slot as u8);
    if env.r32(pl + 0x6C) == 1 {
        return (env.r32(p + 0x3C) == 8) as u32;
    }
    let end = env.r32(p + 0x40);
    let set = |env: &mut E, a: u32| env.w32(pl + 0xC, a);
    match end {
        0 => {
            let s = work_s(env, pl);
            let st = env.r32(s + 0xC);
            if env.r32(p + 0x34) == 6 || st == 2 {
                set(env, 2);
                4
            } else {
                (env.r32(p + 0x3C) == 8) as u32
            }
        }
        1 => {
            set(env, 6);
            4
        }
        3 | 10 => {
            set(env, 2);
            4
        }
        4 | 5 => {
            set(env, 6);
            let v = env.r16(p + 0x44);
            env.w16(pl + 0x62, v);
            let v = env.r32(p + 0x50);
            env.w32(pl + 0x64, v);
            4
        }
        6 | 7 => {
            set(env, 9);
            4
        }
        8 | 9 => {
            set(env, 10);
            let v = env.r16(p + 0x44);
            env.w16(pl + 0x62, v);
            4
        }
        15 => {
            set(env, 7);
            4
        }
        11..=14 | 16..=21 => {
            let s = work_s(env, pl);
            env.w32(s + 0x28, end - 11);
            2
        }
        _ => 1,
    }
}

/// C: `FUN_02160D6C(pl, page)`: init of entry 1 (overlay 0xD4): a 0x10-byte block `{gd, 0200CA1C(gd), 02012944(gd),
/// u16 page (-1 -> 0)}`, after `02029DBC(02012A30(gd), 02012134(gs))`.
pub fn d4_init<E: FieldEnv>(env: &mut E, pl: u32, page: u32) -> u32 {
    let p = alloc(env, 0x10, 0, 0x49B);
    env.mem().fill(p, 0x10, 0);
    let gd = gd_of(env, pl);
    env.w32(p, gd);
    let v = env.callv(0x0200_CA1C, &[gd]);
    env.w32(p + 4, v);
    let gd = env.r32(p);
    let v = env.callv(0x0201_2944, &[gd]);
    env.w32(p + 8, v);
    let gd = env.r32(p);
    let x = env.callv(0x0201_2A30, &[gd]);
    let gs = gs_of(env, pl);
    let y = env.callv(0x0201_2134, &[gs]);
    env.callv(0x0202_9DBC, &[x, y]);
    env.w16(p + 0xC, if page == 0xFFFF_FFFF { 0 } else { page as u16 });
    p
}

/// C: `FUN_02160DDC(pl, param)`: return of entry 1: `02029DD8(02012A30(param.gd), 02012134(gs))`; 1 when
/// `param+0xE` is set.
pub fn d4_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let gd = env.r32(p);
    let x = env.callv(0x0201_2A30, &[gd]);
    let gs = gs_of(env, pl);
    let y = env.callv(0x0201_2134, &[gs]);
    env.callv(0x0202_9DD8, &[x, y]);
    (env.r16(p + 0xE) != 0) as u32
}

/// The first party Pokemon `i` with field 0xA9 set, (with `fainted_ok` false) `0201897C(mon)` set, and field 0x4C
/// clear; `None` when there is none.
fn first_mon<E: FieldEnv>(env: &mut E, party: u32, check_897c: bool) -> Option<u32> {
    if (env.callv(PARTY_COUNT, &[party]) as i32) <= 0 {
        return None;
    }
    let mut i = 0u32;
    loop {
        let mon = env.callv(PARTY_MON, &[party, i]);
        if env.callv(MON_GET, &[mon, 0xA9, 0]) != 0
            && (!check_897c || env.callv(0x0201_897C, &[mon]) != 0)
            && env.callv(MON_GET, &[mon, 0x4C, 0]) == 0
        {
            return Some(i);
        }
        i += 1;
        if (i as i32) >= (env.callv(PARTY_COUNT, &[party]) as i32) {
            return None;
        }
    }
}

/// C: `FUN_02160E08(pl, mode, kind, prev)`: init of entry 6 (overlay 0x83): a 0x28-byte block; after the party list
/// (`kind` 0) it takes the list's party / slot and end mode; `kind` 6 starts on the party with `mode` (1 / 2: the
/// first Pokemon that qualifies).
pub fn app83_init<E: FieldEnv>(env: &mut E, pl: u32, mode: u32, kind: u32, prev: u32) -> u32 {
    let p = alloc(env, 0x28, 1, 0x4DE);
    let gd = gd_of(env, pl);
    let h = env.callv(0x0201_2EBC, &[gd]);
    let x = env.callv(0x0200_CA1C, &[gd]);
    env.w8(p + 0xC, 1);
    env.w32(p + 8, gd);
    env.w32(p + 0x24, 0);
    let v = env.callv(0x0200_CA38, &[x]);
    env.w32(p + 0x18, v);
    if kind == 0 {
        let party = env.r32(prev);
        env.w32(p, party);
        let v = env.r32(prev + 0xC);
        env.w32(p + 4, v);
        let n = env.callv(PARTY_COUNT, &[party]);
        env.w8(p + 0xE, n as u8);
        let v = env.r32(prev + 0x3C);
        env.w8(p + 0xF, v as u8);
        let end = env.r32(prev + 0x40);
        if end == 4 || end == 5 {
            let v = env.r16(prev + 0x46);
            env.w16(p + 0x14, v);
            env.w8(p + 0xD, 2);
            env.w8(p + 0x10, 1);
            env.w32(p + 0x20, 0);
            env.w32(pl + 0x68, if end == 4 { 5 } else { 6 });
        } else {
            env.w8(p + 0x10, 0);
            env.w8(p + 0xD, 0);
            env.w32(p + 0x20, 1);
        }
    } else if kind == 6 {
        let v = env.callv(0x0200_873C, &[h]);
        env.w32(p + 4, v);
        let party = env.callv(PARTY, &[gd]);
        env.w32(p, party);
        let party = env.callv(PARTY, &[gd]);
        let n = env.callv(PARTY_COUNT, &[party]);
        env.w8(p + 0xE, n as u8);
        env.w8(p + 0xF, 0);
        env.w8(p + 0xD, 0);
        env.w32(p + 0x20, 1);
        if mode == 0xFFFF_FFFF {
            env.w8(p + 0x10, 0);
        } else {
            env.w8(p + 0x10, mode as u8);
            let m = env.r8(p + 0x10);
            if m == 2 || m == 1 {
                let party = env.callv(PARTY, &[gd]);
                if let Some(i) = first_mon(env, party, m == 2) {
                    env.w8(p + 0xF, i as u8);
                }
            }
        }
    }
    p
}

/// C: `FUN_02160F88(pl, param)`: return of entry 6.
pub fn app83_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    if env.r8(p + 0x12) > 1 {
        return 1;
    }
    let s = work_s(env, pl);
    if env.r32(s + 0xC) == 6 {
        return 0;
    }
    env.w32(pl + 0xC, 0);
    4
}

/// C: `FUN_02160FB4(gs)`: the bag's start mode by zone (`02012AC8(020120F8(gs))`): 1 / 3 when `02013CC8` /
/// `02013CE0` answer 1, else 0.
pub fn zone_mode<E: FieldEnv>(env: &mut E, gs: u32) -> u32 {
    let x = env.callv(0x0201_20F8, &[gs]);
    let z = env.callv(0x0201_2AC8, &[x]);
    if env.callv(0x0201_3CC8, &[z]) == 1 {
        1
    } else if env.callv(0x0201_3CE0, &[z]) == 1 {
        3
    } else {
        0
    }
}

/// C: `FUN_02160FE0(pl, item, kind, prev)`: init of entry 2 (overlay 0x46, the bag): `0202E474(gd, &pl+0x34, mode,
/// 4)`, mode [`zone_mode`] (2 after an app whose end mode was 3, `kind` 0); `kind` 2 preselects `item` (unless -1).
pub fn bag_init<E: FieldEnv>(env: &mut E, pl: u32, item: u32, kind: u32, prev: u32) -> u32 {
    let gd = gd_of(env, pl);
    if kind == 0 && env.r32(prev + 0x40) == 3 {
        return env.callv(0x0202_E474, &[gd, pl + 0x34, 2, 4]);
    }
    let gs = gs_of(env, pl);
    let m = zone_mode(env, gs);
    let r = env.callv(0x0202_E474, &[gd, pl + 0x34, m, 4]);
    if kind == 2 && item != 0xFFFF_FFFF {
        let x = env.r32(r + 0xC);
        env.callv(0x0200_8298, &[x, item & 0xFFFF]);
    }
    r
}

/// C: `FUN_02161070(pl, param)`: return of entry 2: next action by the bag's end mode (`param+0x34`): back to the
/// field, another app (`pl+0xC`), or an event result in `S+0x28` (3).
pub fn bag_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let result = |env: &mut E, v: u32| {
        let s = work_s(env, pl);
        env.w32(s + 0x28, v);
        3
    };
    let next = |env: &mut E, a: u32| {
        env.w32(pl + 0xC, a);
        4
    };
    match env.r32(p + 0x34) {
        0 => 1,
        1 => {
            if env.r32(p + 0x2C) == 2 {
                next(env, 0)
            } else {
                0
            }
        }
        2 => {
            let item = env.r32(p + 0x38) & 0xFFFF;
            if env.callv(0x0202_136C, &[item]) == 1 {
                env.w32(pl + 0x68, 0);
            }
            next(env, 0)
        }
        7 => next(env, 7),
        8 => next(env, 8),
        10 | 11 => result(env, 0),
        12 => next(env, 0),
        13 => result(env, 1),
        14 => result(env, 2),
        15 => result(env, 3),
        16 => result(env, 4),
        17 => next(env, 11),
        18 => {
            let gd = gd_of(env, pl);
            let party = env.callv(PARTY, &[gd]);
            let s = work_s(env, pl);
            let slot = env.r32(s + 0x2C);
            let mon = env.callv(PARTY_MON, &[party, slot]);
            let a = if env.callv(MON_GET, &[mon, 6, 0]) != 0 {
                0
            } else {
                9
            };
            env.w32(pl + 0xC, a);
            env.w32(pl + 0x68, 2);
            4
        }
        19 => next(env, 9),
        20 => result(env, 5),
        _ => next(env, 0),
    }
}

/// C: `FUN_0216117C(pl, x)`: init of entry 3 (overlay 0x6F): `0216F7BC(gd, 4, flag)` (flag 0 in the zones of
/// `02013CC8` / `02013CE0`), its `+0x14` = `x` unless -1; `02029DBC(02012A30(gd), 02012134(gs))` before.
pub fn app6f_init<E: FieldEnv>(env: &mut E, pl: u32, x: u32) -> u32 {
    let gd = gd_of(env, pl);
    let z = env.callv(0x0201_27A4, &[gd]);
    let zone = u32::from(env.r16(z));
    let a = env.callv(0x0201_2A30, &[gd]);
    let gs = gs_of(env, pl);
    let b = env.callv(0x0201_2134, &[gs]);
    env.callv(0x0202_9DBC, &[a, b]);
    let flag = if env.callv(0x0201_3CC8, &[zone]) != 0 || env.callv(0x0201_3CE0, &[zone]) != 0 {
        0
    } else {
        1
    };
    let r = env.callv(0x0216_F7BC, &[gd, 4, flag]);
    if x != 0xFFFF_FFFF {
        env.w32(r + 0x14, x);
    }
    r
}

/// C: `FUN_021611E0(pl, param)`: return of entry 3: `02029DD8(02012A30(gd), 02012134(gs))`; 1 when `param+0x10` is 1.
pub fn app6f_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let gd = gd_of(env, pl);
    let a = env.callv(0x0201_2A30, &[gd]);
    let gs = gs_of(env, pl);
    let b = env.callv(0x0201_2134, &[gs]);
    env.callv(0x0202_9DD8, &[a, b]);
    (env.r32(p + 0x10) == 1) as u32
}

/// C: `FUN_02161214(pl)`: init of entry 7 (overlay 0x48): a 0x18-byte block `{S+0xC != 0, -, u16 02012AC8(zone), s16
/// 020128B8(gd)+4, .., gs}`.
pub fn app48_init<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    let p = alloc(env, 0x18, 1, 0x68A);
    env.w32(p, 1);
    let gs = gs_of(env, pl);
    env.w32(p + 0x14, gs);
    let gd = gd_of(env, pl);
    let z = env.callv(0x0201_27A4, &[gd]);
    let gd2 = gd_of(env, pl);
    let v = env.callv(0x0201_2AC8, &[z]);
    env.w16(p + 8, v as u16);
    let q = env.callv(0x0201_28B8, &[gd2]);
    let v = env.r16(q + 4);
    env.w16(p + 0xA, v);
    let s = work_s(env, pl);
    let st = env.r32(s + 0xC);
    env.w32(p, (st != 0) as u32);
    p
}

/// C: `FUN_02161280(pl, param)`: return of entry 7, by the proclink state `S+0xC` (7 / 2 / 0) and the result
/// `param+4` (state 0, result 2: the event result 4 with the params `param+8 / +0xC / +0x10`).
pub fn app48_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let s = work_s(env, pl);
    let r = env.r32(p + 4);
    match env.r32(s + 0xC) {
        7 => (r != 0) as u32,
        2 => {
            if r == 0 {
                env.w32(pl + 0xC, 2);
                4
            } else {
                1
            }
        }
        0 => match r {
            0 => {
                env.w32(pl + 0xC, 0);
                4
            }
            2 => {
                env.w32(s + 0x28, 4);
                let v = env.r16(p + 8);
                env.w32(s + 0x30, u32::from(v));
                let v = env.r32(p + 0xC);
                env.w32(s + 0x34, v);
                let v = env.r32(p + 0x10);
                env.w32(s + 0x38, v);
                env.w32(s + 0x3C, 0);
                2
            }
            _ => 1,
        },
        _ => 1,
    }
}

/// C: `FUN_02161308(pl)`: init of entry 5 (overlay 0x44): `0200873C(02012EBC(gd))`; an 8-byte block `{-, gs}`.
pub fn app44_init<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    let gd = gd_of(env, pl);
    let h = env.callv(0x0201_2EBC, &[gd]);
    env.callv(0x0200_873C, &[h]);
    let p = alloc(env, 8, 1, 0x705);
    let gs = gs_of(env, pl);
    env.w32(p + 4, gs);
    p
}

/// C: `FUN_02161340(pl, param)`: return of entry 5: `param+0 != 0`.
pub fn app44_return<E: FieldEnv>(env: &mut E, p: u32) -> u32 {
    (env.r32(p) != 0) as u32
}

/// C: `FUN_02161350(pl)`: the special callback of entry 4: start the GMEVENT [`event_new`]`(gs, fm, u16 pl+0,
/// 02188C54(fm), &pl+0x10)` as a sub-event of `pl+0x14` (`02012374`).
pub fn report_special<E: FieldEnv>(env: &mut E, pl: u32) {
    let s = work_s(env, pl);
    let fm = env.r32(s + 4);
    let x = env.callv(0x0218_8C54, &[fm]);
    let id = u32::from(env.r16(pl));
    let gs = env.r32(s);
    let fm = env.r32(s + 4);
    let ev = event_new(env, gs, fm, id, x, pl + 0x10);
    let parent = env.r32(pl + 0x14);
    env.callv(0x0201_2374, &[parent, ev]);
}

/// C: `FUN_02161378(pl)`: init of entry 8 (overlay 0x80): an 8-byte block `{gd, -}`.
pub fn app80_init<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    let p = alloc(env, 8, 1, 0x74C);
    let gd = gd_of(env, pl);
    env.w32(p, gd);
    p
}

/// C: `FUN_021613A4(pl, param)`: return of entry 8.
pub fn app80_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let s = work_s(env, pl);
    if env.r32(s + 0xC) == 8 {
        return (env.r32(p + 4) != 0) as u32;
    }
    if env.r32(p + 4) == 0 {
        env.w32(pl + 0xC, 2);
        4
    } else {
        1
    }
}

/// C: `FUN_021613CC(pl, mode, kind, prev)`: init of entry 9 (loads overlay 0x88, unloaded by [`item_return`] /
/// [`item_free`]): the item screens `021C3CC0(gd, 2, slot, 0202138C(item), 4)` / `021C3D44(gd, mon, 4)` /
/// `021C3DA4(gd, 0202138C(item), 4)` after the party list (`kind` 0) or the bag.
pub fn item_init<E: FieldEnv>(env: &mut E, pl: u32, kind: u32, prev: u32) -> u32 {
    let gd = gd_of(env, pl);
    env.callv(OVERLAY_LOAD, &[OVERLAY_ITEM]);
    if kind == 0 {
        let party = env.callv(PARTY, &[gd]);
        let slot = env.r32(prev + 0x3C);
        env.w8(pl + 0x60, slot as u8);
        let item = env.r16(prev + 0x44);
        env.w16(pl + 0x62, item);
        if env.r32(prev + 0x40) == 6 {
            let z = env.callv(0x0202_138C, &[u32::from(item)]);
            let slot = env.r32(prev + 0x3C) & 0xFF;
            return env.callv(0x021C_3CC0, &[gd, 2, slot, z, 4]);
        }
        env.w32(pl + 0x68, 1);
        let mon = env.callv(PARTY_MON, &[party, slot]);
        return env.callv(0x021C_3D44, &[gd, mon, 4]);
    }
    let item = env.r32(prev + 0x38) & 0xFFFF;
    if env.r32(prev + 0x2C) == 2 {
        let z = env.callv(0x0202_138C, &[item]);
        let b = env.r32(prev + 0x38) & 0xFF;
        let r = env.callv(0x021C_3CC0, &[gd, 2, b, z, 4]);
        let v = env.r32(prev + 0x38);
        env.w16(pl + 0x62, v as u16);
        return r;
    }
    env.w32(pl + 0x68, 1);
    let z = env.callv(0x0202_138C, &[item]);
    env.callv(0x021C_3DA4, &[gd, z, 4])
}

/// Hand the item screen's result to the party slot `pl+0x60` (`021C3E04`) when `021C3E00(param)` answers 1.
fn item_apply<E: FieldEnv>(env: &mut E, pl: u32, p: u32, gd: u32) -> bool {
    if env.callv(0x021C_3E00, &[p]) != 1 {
        return false;
    }
    let party = env.callv(PARTY, &[gd]);
    let slot = u32::from(env.r8(pl + 0x60));
    let mon = env.callv(PARTY_MON, &[party, slot]);
    env.callv(0x021C_3E04, &[p, mon]);
    true
}

/// C: `FUN_02161484(pl, param)`: return of entry 9: apply the result, pick the next action (back to the party list
/// 0 or the bag 2), unload overlay 0x88.
pub fn item_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let gd = gd_of(env, pl);
    let next = match env.r32(pl + 8) {
        0 => match env.r32(pl + 0x68) {
            0 | 2 => {
                if item_apply(env, pl, p, gd) || env.r32(pl + 0x68) != 0 {
                    0
                } else {
                    2
                }
            }
            _ => 0,
        },
        2 if env.r32(pl + 0x68) == 2 => {
            item_apply(env, pl, p, gd);
            0
        }
        _ => 2,
    };
    env.w32(pl + 0xC, next);
    env.callv(OVERLAY_UNLOAD, &[OVERLAY_ITEM]);
    4
}

/// C: `FUN_02161534(param)`: free callback of entry 9 (`021C3E24` inside overlay 0x88).
pub fn item_free<E: FieldEnv>(env: &mut E, p: u32) {
    env.callv(OVERLAY_LOAD, &[OVERLAY_ITEM]);
    env.callv(0x021C_3E24, &[p]);
    env.callv(OVERLAY_UNLOAD, &[OVERLAY_ITEM]);
}

/// C: `FUN_02161554(pl, mode, kind, prev)`: init of entry 10 (overlay 0xC3): only after the party list (`kind` 0,
/// else 0): `0201B2B0(party, mon, k, x, 02012984(gd), &out, 4)` (end mode 8: `k` 3 with the item `prev+0x44`, else 0
/// with the zone) and a 0x14-byte block `{gd, party, u16 result, u8 slot, u8 out, 1, 1}`.
pub fn appc3_init<E: FieldEnv>(env: &mut E, pl: u32, kind: u32, prev: u32) -> u32 {
    let gd = gd_of(env, pl);
    if kind != 0 {
        return 0;
    }
    let party = env.callv(PARTY, &[gd]);
    let out = env.mem().push_local(4);
    let r = if env.r32(prev + 0x40) == 8 {
        env.w32(pl + 0x68, 3);
        let slot = env.r32(prev + 0x3C);
        let mon = env.callv(PARTY_MON, &[party, slot]);
        let v = env.callv(0x0201_2984, &[gd]);
        let x = u32::from(env.r16(prev + 0x44));
        env.call(
            0x0201_B2B0,
            &mut [
                CArg::V(party),
                CArg::V(mon),
                CArg::V(3),
                CArg::V(x),
                CArg::V(v),
                CArg::Out(out, 4),
                CArg::V(4),
            ],
        )
    } else {
        env.w32(pl + 0x68, 4);
        let slot = env.r32(prev + 0x3C);
        let mon = env.callv(PARTY_MON, &[party, slot]);
        let z = env.callv(0x0201_27A4, &[gd]);
        let v = env.callv(0x0201_2984, &[gd]);
        let x = u32::from(env.r16(z));
        env.call(
            0x0201_B2B0,
            &mut [
                CArg::V(party),
                CArg::V(mon),
                CArg::V(0),
                CArg::V(x),
                CArg::V(v),
                CArg::Out(out, 4),
                CArg::V(4),
            ],
        )
    };
    let o = env.r8(out);
    env.mem().pop_local(out);
    let p = alloc(env, 0x14, 0, 0x868);
    env.w32(p, gd);
    env.w32(p + 4, party);
    env.w16(p + 8, r as u16);
    let slot = env.r32(prev + 0x3C);
    env.w8(p + 0xA, slot as u8);
    env.w8(p + 0xB, o);
    env.w32(p + 0xC, 1);
    env.w32(p + 0x10, 1);
    p
}

/// C: `FUN_02161610(pl, param)`: return of entry 10, by `pl+0x68` (3: action 2, 4: action 0, else 0).
pub fn appc3_return<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    gd_of(env, pl);
    match env.r32(pl + 0x68) {
        3 => {
            env.w32(pl + 0xC, 2);
            4
        }
        4 => {
            env.w32(pl + 0xC, 0);
            4
        }
        _ => 0,
    }
}

/// C: `FUN_0216163C(pl)`: init of entry 11 (overlay 0xBB): a 0x10-byte block `{-, gd, ..}`.
pub fn appbb_init<E: FieldEnv>(env: &mut E, pl: u32) -> u32 {
    let p = alloc(env, 0x10, 1, 0x8AC);
    let gd = gd_of(env, pl);
    env.w32(p + 4, gd);
    p
}

/// C: `FUN_02161668(pl, param)`: return of entry 11, by the proclink state (11 or not) and `param+0xC`.
pub fn appbb_return<E: FieldEnv>(env: &mut E, pl: u32, p: u32) -> u32 {
    let s = work_s(env, pl);
    let v = env.r32(p + 0xC);
    if env.r32(s + 0xC) == 11 {
        return (v == 1) as u32;
    }
    match v {
        0 => {
            env.w32(pl + 0xC, 2);
            4
        }
        1 => 1,
        _ => 0,
    }
}

/// C: `FUN_021616A4(gs, fm, id, x, &result)`: a GMEVENT (proc [`event_proc`]), work 0x1C `{02012EBC(gd), gs, fm, x,
/// u16 id, &result}`.
pub fn event_new<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, id: u32, x: u32, result: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, 0, 0x0216_16F1, 0x1C]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.mem().fill(w, 0x1C, 0);
    env.w32(w + 0xC, x);
    env.w16(w + 0x10, id as u16);
    env.w32(w + 4, gs);
    env.w32(w + 8, fm);
    let gd = env.callv(GAMEDATA, &[gs]);
    let v = env.callv(0x0201_2EBC, &[gd]);
    env.w32(w, v);
    env.w32(w + 0x14, result);
    ev
}

/// C: `FUN_021616F0(event, seq, work)`: the proc: `02169AB8(work)` 0 -> `*result = 1`, done; 1 -> `*result = 0`,
/// done; else wait.
pub fn event_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let v = match env.callv(0x0216_9AB8, &[w]) {
        0 => 1,
        1 => 0,
        _ => return 0,
    };
    let res = env.r32(w + 0x14);
    env.w32(res, v);
    1
}

/// C: `FUN_02161724(event, seq, work)`: pause the field OBJs: seq 0 pauses (`0216DE54(sys, 0x30)`, `0216D9A0`;
/// the player's OBJ `0216DEE0` gets `0216DB04` unless `0216D454` says it is ready, flag bit 0 of `work+0xC`); seq 1
/// releases each flagged OBJ once it is ready (bits 0 / 2 / 1) and moves on when no flag is left; seq 2 ends.
/// (In seq 0 the original's test for bit 2 sits after a `movs` that sets Z, so it never runs.)
pub fn pause_proc<E: FieldEnv>(env: &mut E, seq: u32, w: u32) -> u32 {
    let flags = w + 0xC;
    match env.r32(seq) {
        0 => {
            let fm = env.r32(w + 4);
            let sys = env.callv(MMDL_SYS, &[fm]);
            let obj = env.callv(0x0216_DEE0, &[sys]);
            env.callv(0x0216_DE54, &[sys, 0x30]);
            env.w8(flags, 0);
            env.callv(0x0216_D9A0, &[sys]);
            if env.callv(0x0216_D454, &[obj]) == 0 {
                let f = env.r8(flags) | 1;
                env.w8(flags, f);
                env.callv(0x0216_DB04, &[obj]);
            }
            next_seq(env, seq);
            0
        }
        1 => {
            let fm = env.r32(w + 4);
            let sys = env.callv(MMDL_SYS, &[fm]);
            let obj = env.callv(0x0216_DEE0, &[sys]);
            env.callv(0x0216_DE54, &[sys, 0x30]);
            if env.r8(flags) & 1 != 0 && env.callv(0x0216_D454, &[obj]) == 1 {
                env.callv(0x0216_DAF8, &[obj]);
                let f = env.r8(flags) & 0xFE;
                env.w8(flags, f);
            }
            if env.r8(flags) & 4 != 0 && env.callv(0x0216_DA5C, &[0]) == 0 {
                env.callv(0x0216_DAF8, &[0]);
                let f = env.r8(flags) & 0xFB;
                env.w8(flags, f);
            }
            if env.r8(flags) & 2 != 0 {
                let o = env.callv(0x0216_DE54, &[sys, 0x30]);
                if env.callv(0x0216_DA5C, &[o]) == 0 {
                    env.callv(0x0216_DAF8, &[o]);
                    let f = env.r8(flags) & 0xFD;
                    env.w8(flags, f);
                }
            }
            if env.r8(flags) == 0 {
                next_seq(env, seq);
            }
            0
        }
        2 => 1,
        _ => 0,
    }
}

/// C: `FUN_02161824(gs, fm)`: the pause event (proc [`pause_proc`], work 0x10 `{gs, fm, gd, flags}`).
pub fn pause_new<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, 0, 0x0216_1725, 0x10]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, gs);
    env.w32(w + 4, fm);
    let gd = env.callv(GAMEDATA, &[gs]);
    env.w32(w + 8, gd);
    ev
}

/// C: `FUN_02161850(event, seq, work)`: acmd-list event proc: seq 0 starts the list `work+0x10` on the OBJ
/// [`obj_of`]`(sys, id)` (no OBJ: done), seq 1 waits until every started list ended ([`slots_update`]).
pub fn acmd_proc<E: FieldEnv>(env: &mut E, seq: u32, w: u32) -> u32 {
    let fm = env.r32(w + 4);
    let sys = env.callv(MMDL_SYS, &[fm]);
    match env.r32(seq) {
        0 => {
            let id = u32::from(env.r16(w + 0xC));
            let obj = obj_of(env, sys, id);
            if obj == 0 {
                return 1;
            }
            let list = env.r32(w + 0x10);
            let t = env.callv(ACMD_START, &[obj, list]);
            slot_put(env, w, t);
            next_seq(env, seq);
            0
        }
        1 => (slots_update(env, w) == 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_021618A0(gs, fm, id, list)`: the acmd-list event (proc [`acmd_proc`], work 0x34 `{gs, fm, gd, u16 id,
/// list, slots[8]}`).
pub fn acmd_new<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, id: u32, list: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, 0, 0x0216_1851, 0x34]);
    let w = env.callv(EVENT_WORK, &[ev]);
    slots_clear(env, w);
    env.w32(w, gs);
    env.w32(w + 4, fm);
    let gd = env.callv(GAMEDATA, &[gs]);
    env.w32(w + 8, gd);
    env.w16(w + 0xC, id as u16);
    env.w32(w + 0x10, list);
    ev
}

/// The ROM acmd list for the OBJ direction `d` (0..3).
fn dir_list(d: u32) -> u32 {
    // UNVERIFIED: another direction leaves the list register as the caller had it (a callee-saved register);
    // the port uses 0.
    DIR_LISTS.get(d as usize).copied().unwrap_or(0)
}

/// C: `FUN_021618E0(gs, fm)`: [`acmd_new`] for the player's OBJ (id 0xFF) with the ROM list of its direction.
pub fn acmd_new_player<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let player = env.callv(PLAYER, &[fm]);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    let d = env.callv(OBJ_DIR, &[obj]);
    acmd_new(env, gs, fm, 0xFF, dir_list(d))
}

/// C: `FUN_02161938(sys, id)`: the OBJ of `id`: 0xF2 -> `0216DE54(sys, 0x30)`, 0xF1 -> `sys` itself (as the original
/// returns its first argument), else `0216DE94(sys, id)`.
pub fn obj_of<E: FieldEnv>(env: &mut E, sys: u32, id: u32) -> u32 {
    match id {
        0xF2 => env.callv(0x0216_DE54, &[sys, 0x30]),
        0xF1 => sys,
        _ => env.callv(0x0216_DE94, &[sys, id]),
    }
}

/// C: `FUN_02161950(work)`: clear the eight list slots (`work+0x14`).
pub fn slots_clear<E: FieldEnv>(env: &mut E, w: u32) {
    for i in 0..8 {
        env.w32(w + 0x14 + 4 * i, 0);
    }
}

/// C: `FUN_02161964(work, list)`: put `list` in the first free slot (none free: dropped).
pub fn slot_put<E: FieldEnv>(env: &mut E, w: u32, list: u32) {
    for i in 0..8 {
        if env.r32(w + 0x14 + 4 * i) == 0 {
            env.w32(w + 0x14 + 4 * i, list);
            return;
        }
    }
}

/// C: `FUN_02161984(work)`: free the slots whose list ended; 1 while one still runs.
pub fn slots_update<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let mut busy = 0;
    for i in 0..8 {
        let a = w + 0x14 + 4 * i;
        let t = env.r32(a);
        if t == 0 {
            continue;
        }
        if env.callv(ACMD_ENDED, &[t]) == 1 {
            let t = env.r32(a);
            env.callv(ACMD_FREE, &[t]);
            env.w32(a, 0);
        } else {
            busy = 1;
        }
    }
    busy
}

/// `FX_Mul(a, b)`: `(a * b + 0x800) >> 12` on 64 bits.
fn fx_mul(a: u32, b: u32) -> u32 {
    ((i64::from(a as i32) * i64::from(b as i32)).wrapping_add(0x800) >> 12) as u32
}

/// C: `FUN_021619B8(event, seq, work)`: jump-down event proc. Seq 0: position = `from` + (the arc height
/// `ARC[i]` on y, `FX_Div(FX_Mul(delta, i' << 12), 15 << 12)` on x / z) with `i` = `work+8` (+= 2 per frame,
/// `i'` the new value); next seq once `i' > 15`. Seq 1: drop by 8 px per frame down to the target y, then put the OBJ
/// on the target (`0216E014` with its direction), sound 0x67B, `021AC528(obj, work+0)`, done.
pub fn jump_proc<E: FieldEnv>(env: &mut E, seq: u32, w: u32) -> u32 {
    let obj = env.r32(w + 4);
    match env.r32(seq) {
        0 => {
            let l = env.mem().push_local(12);
            copy_words(env, l, w + 0xC, 3);
            let i = i32::from(env.r16(w + 8) as i16);
            // UNVERIFIED: an index outside 0..=16 reads other ROM words (the port reads 0).
            let h = usize::try_from(i)
                .ok()
                .and_then(|k| ARC.get(k))
                .copied()
                .unwrap_or(0);
            let y = env.r32(l + 4).wrapping_add(h);
            env.w32(l + 4, y);
            env.w16(w + 8, (i + 2) as u16);
            let i2 = i32::from(env.r16(w + 8) as i16);
            let x = fx_mul(env.r32(w + 0x18), (i2 << 12) as u32);
            let dx = env.callv(0x0207_C768, &[x, 0xF000]);
            let v = env.r32(l).wrapping_add(dx);
            env.w32(l, v);
            let z = fx_mul(env.r32(w + 0x20), (i2 << 12) as u32);
            let dz = env.callv(0x0207_C768, &[z, 0xF000]);
            let v = env.r32(l + 8).wrapping_add(dz);
            env.w32(l + 8, v);
            env.call(0x0216_D89C, &mut [CArg::V(obj), CArg::In(l, 12)]);
            env.mem().pop_local(l);
            if i32::from(env.r16(w + 8) as i16) as u32 > 15 {
                next_seq(env, seq);
            }
            0
        }
        1 => {
            let l = env.mem().push_local(12);
            env.call(0x0216_D88C, &mut [CArg::V(obj), CArg::Out(l, 12)]);
            let target = env.r32(w + 0x10).wrapping_add(env.r32(w + 0x1C));
            let y = env.r32(l + 4).wrapping_sub(0x8000);
            if (y as i32) > (target as i32) {
                env.w32(l + 4, y);
                env.call(0x0216_D89C, &mut [CArg::V(obj), CArg::In(l, 12)]);
                env.mem().pop_local(l);
                return 0;
            }
            env.w32(l + 4, target);
            let v = env.r32(w + 0xC).wrapping_add(env.r32(w + 0x18));
            env.w32(l, v);
            let v = env.r32(w + 0x14).wrapping_add(env.r32(w + 0x20));
            env.w32(l + 8, v);
            let d = env.callv(OBJ_DIR, &[obj]);
            env.call(
                0x0216_E014,
                &mut [CArg::V(obj), CArg::In(l, 12), CArg::V(d)],
            );
            env.callv(PLAY_SE, &[0x67B]);
            let x = env.r32(w);
            env.callv(0x021A_C528, &[obj, x]);
            env.mem().pop_local(l);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_02161AC0(gs, obj, &from, &to)`: the jump-down event (proc [`jump_proc`], work 0x24 `{02188CD0(0201210C(gs)),
/// obj, index, from, to - from}`); sound 0x55E.
pub fn jump_new<E: FieldEnv>(env: &mut E, gs: u32, obj: u32, from: u32, to: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, 0, 0x0216_19B9, 0x24]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w + 4, obj);
    let fm = env.callv(0x0201_210C, &[gs]);
    let v = env.callv(0x0218_8CD0, &[fm]);
    env.w32(w, v);
    copy_words(env, w + 0xC, from, 3);
    env.call(
        0x0207_CA00,
        &mut [
            CArg::In(to, 12),
            CArg::In(from, 12),
            CArg::Out(w + 0x18, 12),
        ],
    );
    env.callv(PLAY_SE, &[0x55E]);
    ev
}

/// C: `FUN_02161B1C(fm)`: start the ROM acmd list of the player OBJ's direction (after `0216D4B4(obj)`); returns
/// the list task.
pub fn acmd_start_player<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let player = env.callv(PLAYER, &[fm]);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    let d = env.callv(OBJ_DIR, &[obj]);
    let list = dir_list(d);
    env.callv(0x0216_D4B4, &[obj]);
    env.callv(ACMD_START, &[obj, list])
}

/// C: `FUN_02161B74(task)`: 1 (and free it) once the acmd list ended.
pub fn acmd_end<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    if env.callv(ACMD_ENDED, &[t]) == 1 {
        env.callv(ACMD_FREE, &[t]);
        1
    } else {
        0
    }
}

/// C: `FUN_02161B90(gs, parent, a, b, flag)`: the fade + sub-proc event (proc [`fade_proc`], work 0x24 `{gs,
/// 02012138(gs), sub-proc parameter (0x18)}`; the parameter from [`param_a`] when `flag`, else [`param_b`]); `a` 3
/// also calls `0216570C()`.
pub fn fade_new<E: FieldEnv>(env: &mut E, gs: u32, parent: u32, a: u32, b: u32, flag: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, parent, 0x0216_1BF9, 0x24]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.mem().fill(w, 0x24, 0);
    env.w32(w, gs);
    let v = env.callv(0x0201_2138, &[gs]);
    env.w32(w + 4, v);
    if flag != 0 {
        param_a(env, w + 8, gs, a & 0xFF, b & 0xFF);
    } else {
        param_b(env, w + 8, gs, a & 0xFF, b & 0xFF);
    }
    if a == 3 {
        env.callv(0x0216_570C, &[]);
    }
    ev
}

/// C: `FUN_02161BF8(event, seq, work)`: seq 0 saves `02012144(gs)` and sets `0201213C(gs, 1)`, starts the fade task
/// `work+4` (`02026454` / `02026400`; nothing to start: skip seq 1); seq 1 waits for it; seq 2 runs the sub-proc
/// `020120B4(gs, 0xCB, 0x021BFB3C, &param)`; seq 3 waits for it (`020120F0`) and restores the saved value.
pub fn fade_proc<E: FieldEnv>(env: &mut E, seq: u32, w: u32) -> u32 {
    let gs = env.r32(w);
    match env.r32(seq) {
        0 => {
            let v = env.callv(0x0201_2144, &[gs]);
            env.w32(w + 0x20, v);
            let gs = env.r32(w);
            env.callv(0x0201_213C, &[gs, 1]);
            let t = env.r32(w + 4);
            if env.callv(0x0202_6454, &[t]) == 0 {
                let s = env.r32(seq).wrapping_add(2);
                env.w32(seq, s);
            } else {
                let t = env.r32(w + 4);
                env.callv(0x0202_6400, &[t]);
                next_seq(env, seq);
            }
            0
        }
        1 => {
            let t = env.r32(w + 4);
            if env.callv(0x0202_6454, &[t]) == 0 {
                next_seq(env, seq);
            }
            0
        }
        2 => {
            env.callv(0x0201_20B4, &[gs, 0xCB, 0x021B_FB3C, w + 8]);
            next_seq(env, seq);
            0
        }
        3 => {
            if env.callv(0x0201_20F0, &[gs]) != 0 {
                return 0;
            }
            let gs = env.r32(w);
            let v = env.r32(w + 0x20);
            env.callv(0x0201_213C, &[gs, v]);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_02161C84(param, gs, x, b, w, c, d, e)`: fill the 0x18-byte sub-proc parameter `{gs, x, w, u8 b, u8 c,
/// u8 d, e & 0x7F | bit 7 = 02008550(02012944(gd)) & 1}`; returns the last byte (the original leaves it in r0).
#[allow(clippy::too_many_arguments)]
pub fn param_fill<E: FieldEnv>(
    env: &mut E,
    p: u32,
    gs: u32,
    x: u32,
    b: u32,
    w: u32,
    c: u32,
    d: u32,
    e: u32,
) -> u32 {
    env.mem().fill(p, 0x18, 0);
    env.w32(p, gs);
    env.w32(p + 4, x);
    env.w8(p + 0xC, b as u8);
    env.w32(p + 8, w);
    env.w8(p + 0xD, c as u8);
    env.w8(p + 0xE, d as u8);
    let f = (env.r8(p + 0xF) & 0x80) | (e as u8 & 0x7F);
    env.w8(p + 0xF, f);
    let gd = env.callv(GAMEDATA, &[gs]);
    let y = env.callv(0x0201_2944, &[gd]);
    let z = env.callv(0x0200_8550, &[y]);
    let f = ((z as u8 & 1) << 7) | (env.r8(p + 0xF) & 0x7F);
    env.w8(p + 0xF, f);
    u32::from(f)
}

/// C: `FUN_02161CDC(param, gs, x, b)`: [`param_fill`] with `w` 0, two bytes of `0203F4CC(&out)` and
/// `0202E888(..)`.
pub fn param_a<E: FieldEnv>(env: &mut E, p: u32, gs: u32, x: u32, b: u32) -> u32 {
    let l = env.mem().push_local(8);
    let r = env.call(0x0203_F4CC, &mut [CArg::Out(l, 8)]);
    let e = env.callv(0x0202_E888, &[r]);
    let c = u32::from(env.r8(l));
    let d = u32::from(env.r8(l + 4));
    env.mem().pop_local(l);
    param_fill(env, p, gs, x, b, 0, c, d, e)
}

/// C: `FUN_02161D18(param, gs, x, b)`: [`param_fill`] with `w` 0 and `02169100(gd)`, `02169110(gd)`,
/// `02012984(gd)`.
pub fn param_b<E: FieldEnv>(env: &mut E, p: u32, gs: u32, x: u32, b: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gs]);
    let c = env.callv(0x0216_9100, &[gd]);
    let d = env.callv(0x0216_9110, &[gd]);
    let e = env.callv(0x0201_2984, &[gd]);
    param_fill(env, p, gs, x, b, 0, c & 0xFF, d & 0xFF, e)
}

/// C: `FUN_02161D64(rec)`: `rec+2` 1 -> `02013A80(0, 1)`, `02013A80(1, 1)`; 0 -> the same with 0.
pub fn display_set<E: FieldEnv>(env: &mut E, rec: u32) {
    if env.r16(rec + 2) == 1 {
        env.callv(0x0201_3A80, &[0, 1]);
        env.callv(0x0201_3A80, &[1, 1]);
    }
    if env.r16(rec + 2) == 0 {
        env.callv(0x0201_3A80, &[0, 0]);
        env.callv(0x0201_3A80, &[1, 0]);
    }
}

#[cfg(test)]
mod tests;
