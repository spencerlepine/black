//! `event_mapchange.c` part 2 (overlay 20, `0x02184CEC..0x02185694`, F1 row 20.00; F6b): the map-change event
//! constructors. Each one creates a GMEVENT with a MAPCHANGE work (0x9C bytes, [`work_init`]) and fills its destination
//! LOCATION at `+0x10` from its arguments (a position, a rail position, a warp record, the special location, a zone);
//! the procs that run them are in [`crate::mapchange_proc`]. Also the player's LOCATION ([`player_loc`]) and the zone
//! entry from the special location ([`special_enter`]).
//!
//! Spec: `tools/rom-diff/specs/black-field/mapchange_new.spec` (the header of `mapchange_proc.spec` documents the work).
//! Every callee outside this area (the other two mapchange areas included) is reached through [`FieldEnv::call`] by
//! address; callees inside it are Rust calls. Stack locals of the original are `Mem::push_local` windows.
//!
//! A direction argument goes through `FUN_020142A4`; the original stores the answer back into the caller's stack
//! argument as a `u16` and reads it as `s16` (the ports pass `answer as u16 as i16`).

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_4CEC,
    0x0218_4D38,
    0x0218_4D90,
    0x0218_4DE8,
    0x0218_4E40,
    0x0218_4E94,
    0x0218_4EEC,
    0x0218_4F40,
    0x0218_4F88,
    0x0218_4FD4,
    0x0218_5050,
    0x0218_50A8,
    0x0218_50FC,
    0x0218_5140,
    0x0218_5214,
    0x0218_52F0,
    0x0218_5360,
    0x0218_5410,
    0x0218_5458,
    0x0218_54B0,
    0x0218_54CC,
    0x0218_5534,
    0x0218_5588,
    0x0218_562C,
];

/// The MAPCHANGE GMEVENT procs of [`crate::mapchange_proc`] (Thumb addresses) that these constructors start.
pub mod procs {
    /// `FUN_02184388` (door).
    pub const DOOR: u32 = 0x0218_4389;
    /// `FUN_02184410` (door, comm).
    pub const DOOR_COMM: u32 = 0x0218_4411;
    /// `FUN_02184504` (warp).
    pub const WARP: u32 = 0x0218_4505;
    /// `FUN_02184584`.
    pub const FADE: u32 = 0x0218_4585;
    /// `FUN_0218466C`.
    pub const WAIT: u32 = 0x0218_466D;
    /// `FUN_02184730`.
    pub const POS: u32 = 0x0218_4731;
    /// `FUN_021847CC`.
    pub const SPECIAL_A: u32 = 0x0218_47CD;
    /// `FUN_02184894`.
    pub const SPECIAL_B: u32 = 0x0218_4895;
    /// `FUN_0218495C`.
    pub const ZONE_C: u32 = 0x0218_495D;
    /// `FUN_02184A14`.
    pub const SPECIAL_D: u32 = 0x0218_4A15;
    /// `FUN_02184AB4`.
    pub const ZONE_E: u32 = 0x0218_4AB5;
    /// `FUN_02184B54`.
    pub const GRID_F: u32 = 0x0218_4B55;
    /// `FUN_02184BF8` (to the union room).
    pub const UNION_IN: u32 = 0x0218_4BF9;
    /// `FUN_02184C78` (from the union room).
    pub const UNION_OUT: u32 = 0x0218_4C79;
}

/// `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// `FUN_0201210C(gsys)`: the FIELDMAP of the game system.
const FIELDMAP: u32 = 0x0201_210C;
/// `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK: u32 = 0x0201_24E8;
/// `FUN_020127A4(gd)`: the PLAYER record.
const PLAYER: u32 = 0x0201_27A4;
/// `FUN_02012894(gd)`: the GAMEDATA LOCATION.
const GD_LOCATION: u32 = 0x0201_2894;
/// `FUN_02012914(gd)`: a GAMEDATA getter.
const GD_2914: u32 = 0x0201_2914;
/// `FUN_02012984(gd)`: the season.
const SEASON: u32 = 0x0201_2984;
/// `FUN_02012138(gsys)`: the intrude / comm system.
const COMM_SYS: u32 = 0x0201_2138;
/// `FUN_02158344(comm)`: the comm record (may be null).
const COMM_REC: u32 = 0x0215_8344;
/// `FUN_02014184(loc, zone, dir, x, y, z)`: LOCATION of a grid position.
const LOC_GRID: u32 = 0x0201_4184;
/// `FUN_020141A8(loc, zone, dir, line, front, side)`: LOCATION of a rail position.
const LOC_RAIL: u32 = 0x0201_41A8;
/// `FUN_02014204(loc, zone)`: LOCATION of a zone.
const LOC_ZONE: u32 = 0x0201_4204;
/// `FUN_020142A4(d)`: a direction code.
const DIR_CODE: u32 = 0x0201_42A4;
/// `FUN_02162694(warp, loc, x)`: the destination LOCATION of a warp record.
const WARP_DEST: u32 = 0x0216_2694;
/// `FUN_021626E8(warp)`: the door kind of a warp record.
const WARP_DOOR: u32 = 0x0216_26E8;
/// `FUN_02185EB0(gd, loc)` (area `mapchange_zone`): the special-location update.
const SPECIAL_UPDATE: u32 = 0x0218_5EB0;
/// `FUN_02185F18(gd, loc)` (area `mapchange_zone`): the special-location copy.
const SPECIAL_COPY: u32 = 0x0218_5F18;
/// The size of a MAPCHANGE work.
pub const WORK_SIZE: u32 = 0x9C;
/// The size of a LOCATION.
const LOC: u32 = 0x1C;

/// `FUN_020142A4(d)` read back as the original does (stored as `u16`, loaded as `s16`).
fn dir_code<E: FieldEnv>(env: &mut E, d: u32) -> u32 {
    env.callv(DIR_CODE, &[d & 0xFFFF]) as u16 as i16 as u32
}

/// A GMEVENT with proc `proc_` and a MAPCHANGE work ([`work_init`]); returns `(event, work)`.
fn new_event<E: FieldEnv>(env: &mut E, gsys: u32, proc_: u32) -> (u32, u32) {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, proc_, WORK_SIZE]);
    let m = env.callv(EVENT_WORK, &[ev]);
    work_init(env, m, gsys);
    (ev, m)
}

/// `FUN_02014184(m + 0x10, zone, dir, *pos, *(pos + 4), *(pos + 8))`.
fn loc_grid_at<E: FieldEnv>(env: &mut E, loc: u32, zone: u32, dir: u32, pos: u32) {
    let x = env.r32(pos);
    let y = env.r32(pos + 4);
    let z = env.r32(pos + 8);
    loc_grid(env, loc, zone, dir, x, y, z);
}

/// `FUN_02014184(loc, zone, dir, x, y, z)`.
fn loc_grid<E: FieldEnv>(env: &mut E, loc: u32, zone: u32, dir: u32, x: u32, y: u32, z: u32) {
    env.call(
        LOC_GRID,
        &mut [
            CArg::Out(loc, LOC),
            CArg::V(zone),
            CArg::V(dir),
            CArg::V(x),
            CArg::V(y),
            CArg::V(z),
        ],
    );
}

/// `FUN_020141A8(loc, zone, dir, u16 r+0, u16 r+6, s16 r+4)` for the rail position `r`.
fn loc_rail_at<E: FieldEnv>(env: &mut E, loc: u32, zone: u32, dir: u32, r: u32) {
    let line = env.r16(r) as u32;
    let front = env.r16(r + 6) as u32;
    let side = env.r16(r + 4) as i16 as u32;
    env.call(
        LOC_RAIL,
        &mut [
            CArg::Out(loc, LOC),
            CArg::V(zone),
            CArg::V(dir),
            CArg::V(line),
            CArg::V(front),
            CArg::V(side),
        ],
    );
}

/// C: `FUN_02184CEC`: initialise the MAPCHANGE work `m`: `+0` gsys, `+4` gd, `+8` fm,
/// `+0xC` the current zone (`FUN_02188C9C(fm)`), `+0x40 = +0x44 = 0`, `+0x48 = +0x4A` the season;
/// `FUN_021AAAF0(FUN_02188CE8(fm), 1)`.
pub fn work_init<E: FieldEnv>(env: &mut E, m: u32, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let fm = env.callv(FIELDMAP, &[gsys]);
    let s = env.callv(SEASON, &[gd]);
    env.w32(m, gsys);
    env.w32(m + 4, gd);
    env.w32(m + 8, fm);
    let zone = env.callv(0x0218_8C9C, &[fm]);
    env.w16(m + 0xC, zone as u16);
    env.w32(m + 0x40, 0);
    env.w32(m + 0x44, 0);
    env.w16(m + 0x48, s as u16);
    env.w16(m + 0x4A, s as u16);
    let fm = env.r32(m + 8);
    let e = env.callv(0x0218_8CE8, &[fm]);
    env.callv(0x021A_AAF0, &[e, 1]);
}

/// C: `FUN_02184D38`: a door event ([`procs::DOOR`]) to grid position `pos` of
/// `zone` (direction `dir` through `FUN_020142A4`), `+0x2C = 0`, `+0x40 = flag`.
pub fn door_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    zone: u32,
    pos: u32,
    dir: u32,
    flag: u32,
) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::DOOR);
    let d = dir_code(env, dir);
    loc_grid_at(env, m + 0x10, zone, d, pos);
    env.w32(m + 0x2C, 0);
    env.w32(m + 0x40, flag);
    ev
}

/// C: `FUN_02184D90`: a door event ([`procs::DOOR`]) to rail position `r` of
/// `zone`, `+0x2C = 0`, `+0x40 = flag`.
pub fn door_rail_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    zone: u32,
    r: u32,
    dir: u32,
    flag: u32,
) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::DOOR);
    let d = dir_code(env, dir);
    loc_rail_at(env, m + 0x10, zone, d, r);
    env.w32(m + 0x2C, 0);
    env.w32(m + 0x40, flag);
    ev
}

/// C: `FUN_02184DE8`: a warp event ([`procs::WARP`]) to rail position `r` of
/// `zone`, `+0x2C = 0`, `+0x40 = flag`.
pub fn warp_rail_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    zone: u32,
    r: u32,
    dir: u32,
    flag: u32,
) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::WARP);
    let d = dir_code(env, dir);
    loc_rail_at(env, m + 0x10, zone, d, r);
    env.w32(m + 0x2C, 0);
    env.w32(m + 0x40, flag);
    ev
}

/// C: `FUN_02184E40`: a [`procs::POS`] event to grid position `pos` of `zone`
/// (direction 1) with the 12 bytes at `p` in `+0x34`, `+0x2C = 0`.
pub fn pos_new<E: FieldEnv>(env: &mut E, gsys: u32, p: u32, zone: u32, pos: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::POS);
    loc_grid_at(env, m + 0x10, zone, 1, pos);
    for i in 0..3 {
        let v = env.r32(p + 4 * i);
        env.w32(m + 0x34 + 4 * i, v);
    }
    env.w32(m + 0x2C, 0);
    ev
}

/// The constructors of `FUN_02184E94` / `FUN_02184EEC`: the LOCATION `FUN_020128B8(gd)`, the special-location copy,
/// `+0x10 = 1`, `+0x2C = 0`, `+0x40 = 1`.
fn special_event<E: FieldEnv>(env: &mut E, gsys: u32, proc_: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, proc_);
    let gd = env.r32(m + 4);
    let src = env.callv(0x0201_28B8, &[gd]);
    env.mem().copy(m + 0x10, src, LOC);
    let gd = env.r32(m + 4);
    env.call(SPECIAL_COPY, &mut [CArg::V(gd), CArg::Out(m + 0x10, LOC)]);
    env.w32(m + 0x10, 1);
    env.w32(m + 0x2C, 0);
    env.w32(m + 0x40, 1);
    ev
}

/// C: `FUN_02184E94`: a [`procs::SPECIAL_A`] event to the special location
/// (`_a0` unused).
pub fn special_a_new<E: FieldEnv>(env: &mut E, _a0: u32, gsys: u32) -> u32 {
    special_event(env, gsys, procs::SPECIAL_A)
}

/// C: `FUN_02184EEC`: a [`procs::SPECIAL_B`] event to the special location.
pub fn special_b_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    special_event(env, gsys, procs::SPECIAL_B)
}

/// C: `FUN_02184F40`: a [`procs::ZONE_C`] event to the LOCATION of zone
/// `FUN_02159F94(FUN_02012914(gd))`, `+0x10 = 1`, `+0x2C = 0`.
pub fn zone_c_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let x = env.callv(GD_2914, &[gd]);
    let (ev, m) = new_event(env, gsys, procs::ZONE_C);
    let zone = env.callv(0x0215_9F94, &[x]);
    env.call(LOC_ZONE, &mut [CArg::Out(m + 0x10, LOC), CArg::V(zone)]);
    env.w32(m + 0x10, 1);
    env.w32(m + 0x2C, 0);
    ev
}

/// C: `FUN_02184F88`: a [`procs::SPECIAL_D`] event to the GAMEDATA LOCATION
/// (`FUN_02012894`), `+0x10 = 1`, direction `+0x18 = 2`, `+0x2C = 0`.
pub fn special_d_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::SPECIAL_D);
    let gd = env.r32(m + 4);
    let src = env.callv(GD_LOCATION, &[gd]);
    env.mem().copy(m + 0x10, src, LOC);
    env.w32(m + 0x10, 1);
    env.w16(m + 0x18, 2);
    env.w32(m + 0x2C, 0);
    ev
}

/// C: `FUN_02184FD4`: a [`procs::ZONE_E`] event to the LOCATION of `zone`,
/// `+0x2C = 0`, direction `+0x18 = 1`; the player's current LOCATION (PLAYER record zone `FUN_02012AC8`, direction
/// `FUN_02012AE4`, position `FUN_02012AA0`) becomes the GAMEDATA LOCATION (`FUN_0201289C`).
pub fn zone_e_new<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::ZONE_E);
    env.call(LOC_ZONE, &mut [CArg::Out(m + 0x10, LOC), CArg::V(zone)]);
    env.w32(m + 0x2C, 0);
    env.w16(m + 0x18, 1);
    let gd = env.r32(m + 4);
    let pl = env.callv(PLAYER, &[gd]);
    let pos = env.callv(0x0201_2AA0, &[pl]);
    let z = env.callv(0x0201_2AC8, &[pl]);
    let d = env.callv(0x0201_2AE4, &[pl]) as u16 as i16 as u32;
    let l = env.mem().push_local(LOC);
    loc_grid_at(env, l, z, d, pos);
    let gd = env.r32(m + 4);
    env.call(0x0201_289C, &mut [CArg::V(gd), CArg::In(l, LOC)]);
    env.mem().pop_local(l);
    ev
}

/// C: `FUN_02185050`: a [`procs::GRID_F`] event to grid position `pos` of `zone`,
/// `+0x2C = 5`, `+0x40 = 0`.
pub fn grid_f_new<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32, pos: u32, dir: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::GRID_F);
    let d = dir_code(env, dir);
    loc_grid_at(env, m + 0x10, zone, d, pos);
    env.w32(m + 0x2C, 5);
    env.w32(m + 0x40, 0);
    ev
}

/// C: `FUN_021850A8`: the event that leaves the union room ([`procs::UNION_OUT`]):
/// zone 0x1A6, position `(0xB8000, 0, 0xF8000)`, direction `FUN_020142A4(0)`, `+0x2C = 0`.
pub fn union_out_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::UNION_OUT);
    let d = dir_code(env, 0);
    loc_grid(env, m + 0x10, 0x1A6, d, 0xB_8000, 0, 0xF_8000);
    env.w32(m + 0x2C, 0);
    ev
}

/// C: `FUN_021850FC`: the event that enters the union room ([`procs::UNION_IN`]) with
/// the GAMEDATA LOCATION (`FUN_02012894`).
pub fn union_in_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let src = env.callv(GD_LOCATION, &[gd]);
    let (ev, m) = new_event(env, gsys, procs::UNION_IN);
    env.mem().copy(m + 0x10, src, LOC);
    ev
}

/// C: `FUN_02185140`: the comm map-change event to grid position `pos` of
/// `zone`: the player's LOCATION ([`player_loc`]) becomes the GAMEDATA LOCATION (`FUN_020128E4`), `FUN_0200BFA4(
/// FUN_0200BE48(sv), FUN_02012FCC(gd), 0)`; with a comm record `r` (and `FUN_02158590(gd)`) `k = FUN_02176230(r, r + 0x1AC)`,
/// `d = k - FUN_02012B58(gd)` (plus the byte `r+0x902` when negative) and `x += 0x10000 + (d << 22)`; returns
/// `FUN_02185BFC(gsys, fm, &loc, a3, k)` with `loc` = grid (`zone`, `FUN_020142A4(0)`, x, y, z).
pub fn comm_pos_new<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32, pos: u32, a3: u32) -> u32 {
    let fm = env.callv(FIELDMAP, &[gsys]);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let mut x = env.r32(pos);
    let y = env.r32(pos + 4);
    let z = env.r32(pos + 8);
    let cur = env.mem().push_local(LOC);
    player_loc(env, cur, fm);
    env.call(0x0201_28E4, &mut [CArg::V(gd), CArg::In(cur, LOC)]);
    let sv = env.callv(0x0201_2EBC, &[gd]);
    let a = env.callv(0x0200_BE48, &[sv]);
    let b = env.callv(0x0201_2FCC, &[gd]);
    env.callv(0x0200_BFA4, &[a, b, 0]);
    let c = env.callv(COMM_SYS, &[gsys]);
    let r = env.callv(COMM_REC, &[c]);
    let mut k = 0;
    if r != 0 && env.callv(0x0215_8590, &[gd]) != 0 {
        k = env.callv(0x0217_6230, &[r, r.wrapping_add(0x1AC)]);
        let mut d = k.wrapping_sub(env.callv(0x0201_2B58, &[gd]));
        if (d as i32) < 0 {
            d = d.wrapping_add(env.r8(r.wrapping_add(0x902)) as u32);
        }
        x = x.wrapping_add(0x1_0000u32.wrapping_add(d << 22));
    }
    let dd = dir_code(env, 0);
    let l = env.mem().push_local(LOC);
    loc_grid(env, l, zone, dd, x, y, z);
    let ev = env.call(
        0x0218_5BFC,
        &mut [
            CArg::V(gsys),
            CArg::V(fm),
            CArg::In(l, LOC),
            CArg::V(a3),
            CArg::V(k),
        ],
    );
    env.mem().pop_local(l);
    env.mem().pop_local(cur);
    ev
}

/// C: `FUN_02185214`: the comm map-change child event to the LOCATION
/// `FUN_020128DC(gd)` (re-made from its zone when `FUN_02013DD0(zone) == 1`) through `FUN_02185C4C`; then
/// `FUN_0200BFA4(.., 1)`, the comm cleanup (`FUN_0202648C`, `FUN_02026484` unless `FUN_02026454`), `FUN_0217BCDC(0, gsys, 0,
/// 0, 0)` when `FUN_021A3238(player)` answers 0, `FUN_02012B78(gd, 0)`, and without a comm record `FUN_02012B64(gd, 0)`,
/// `FUN_02012B8C(gd, 0)`, `FUN_02012B4C(gd, 1)`.
pub fn comm_return_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let fm = env.callv(FIELDMAP, &[gsys]);
    let src = env.callv(0x0201_28DC, &[gd]);
    let l = env.mem().push_local(LOC);
    env.mem().copy(l, src, LOC);
    let zone = env.r16(l + 4) as u32;
    if env.callv(0x0201_3DD0, &[zone]) == 1 {
        let zone = env.r16(l + 4) as u32;
        env.call(LOC_ZONE, &mut [CArg::Out(l, LOC), CArg::V(zone)]);
    }
    let ev = env.call(
        0x0218_5C4C,
        &mut [CArg::V(gsys), CArg::V(fm), CArg::In(l, LOC)],
    );
    let sv = env.callv(0x0201_2EBC, &[gd]);
    let a = env.callv(0x0200_BE48, &[sv]);
    let b = env.callv(0x0201_2FCC, &[gd]);
    env.callv(0x0200_BFA4, &[a, b, 1]);
    let c = env.callv(COMM_SYS, &[gsys]);
    env.callv(0x0202_648C, &[c]);
    if env.callv(0x0202_6454, &[c]) == 0 {
        env.callv(0x0202_6484, &[c]);
    }
    let pl = env.callv(0x0218_8CB4, &[fm]);
    if env.callv(0x021A_3238, &[pl]) == 0 {
        env.callv(0x0217_BCDC, &[0, gsys, 0, 0, 0]);
    }
    env.callv(0x0201_2B78, &[gd, 0]);
    let c = env.callv(COMM_SYS, &[gsys]);
    if env.callv(COMM_REC, &[c]) == 0 {
        env.callv(0x0201_2B64, &[gd, 0]);
        env.callv(0x0201_2B8C, &[gd, 0]);
        env.callv(0x0201_2B4C, &[gd, 1]);
    }
    env.mem().pop_local(l);
    ev
}

/// C: `FUN_021852F0`: the player's LOCATION into `loc`: on a grid map the MMDL
/// position (`FUN_0216D888(FUN_021A3074(player))`, direction 1), on a rail map the rail position of
/// `FUN_021A36BC(player, &r)` (direction 2); the zone is `FUN_02188C9C(fm)`.
pub fn player_loc<E: FieldEnv>(env: &mut E, loc: u32, fm: u32) {
    let pl = env.callv(0x0218_8CB4, &[fm]);
    if env.callv(0x0218_8CFC, &[fm]) == 0 {
        let mmdl = env.callv(0x021A_3074, &[pl]);
        let pos = env.callv(0x0216_D888, &[mmdl]);
        let zone = env.callv(0x0218_8C9C, &[fm]);
        loc_grid_at(env, loc, zone, 1, pos);
    } else {
        let r = env.mem().push_local(8);
        env.call(0x021A_36BC, &mut [CArg::V(pl), CArg::Out(r, 8)]);
        let zone = env.callv(0x0218_8C9C, &[fm]);
        loc_rail_at(env, loc, zone, 2, r);
        env.mem().pop_local(r);
    }
}

/// C: `FUN_02185360`: a door event ([`procs::DOOR`]) through warp record `warp`:
/// `+0x40 = 1`; the GAMEDATA LOCATION for a warp to the special location (`FUN_021625E8`), else the warp's destination
/// (`FUN_02162694(warp, loc, a3)`); `+0x2C` the door kind (`FUN_021626E8`); a special location `l = FUN_02012870(gd)` whose
/// zone `FUN_02013E7C` accepts is kept (`FUN_020128C0(gd, l)`) when the destination zone is refused; the special-location
/// update `FUN_02185EB0`; `FUN_0201447C(FUN_02012B38(gd), 2)`.
pub fn door_warp_new<E: FieldEnv>(env: &mut E, gsys: u32, warp: u32, a3: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let (ev, m) = new_event(env, gsys, procs::DOOR);
    env.w32(m + 0x40, 1);
    if env.callv(0x0216_25E8, &[warp]) != 0 {
        let src = env.callv(GD_LOCATION, &[gd]);
        env.mem().copy(m + 0x10, src, LOC);
    } else {
        env.call(
            WARP_DEST,
            &mut [CArg::V(warp), CArg::Out(m + 0x10, LOC), CArg::V(a3)],
        );
    }
    let door = env.callv(WARP_DOOR, &[warp]);
    env.w32(m + 0x2C, door);
    let l = env.callv(0x0201_2870, &[gd]);
    let lz = env.r16(l + 4) as u32;
    if env.callv(0x0201_3E7C, &[lz]) == 1 {
        let zone = env.r16(m + 0x14) as u32;
        if env.callv(0x0201_3E7C, &[zone]) == 0 {
            env.call(0x0201_28C0, &mut [CArg::V(gd), CArg::In(l, LOC)]);
        }
    }
    env.call(SPECIAL_UPDATE, &mut [CArg::V(gd), CArg::In(m + 0x10, LOC)]);
    let o = env.callv(0x0201_2B38, &[gd]);
    env.callv(0x0201_447C, &[o, 2]);
    ev
}

/// C: `FUN_02185410`: a comm door event ([`procs::DOOR_COMM`]) through warp record
/// `warp` (`FUN_02162694(warp, loc, a3)`), `+0x40 = 1`, `+0x2C` the door kind.
pub fn door_comm_new<E: FieldEnv>(env: &mut E, gsys: u32, warp: u32, a3: u32) -> u32 {
    env.callv(GAMEDATA, &[gsys]);
    let (ev, m) = new_event(env, gsys, procs::DOOR_COMM);
    env.w32(m + 0x40, 1);
    env.call(
        WARP_DEST,
        &mut [CArg::V(warp), CArg::Out(m + 0x10, LOC), CArg::V(a3)],
    );
    let door = env.callv(WARP_DOOR, &[warp]);
    env.w32(m + 0x2C, door);
    ev
}

/// C: `FUN_02185458`: a warp event ([`procs::WARP`]) to grid position `pos` of
/// `zone`, `+0x2C = 0`, the zone-change kind `+0x30 = kind`.
pub fn warp_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    kind: u32,
    zone: u32,
    pos: u32,
    dir: u32,
) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::WARP);
    let d = dir_code(env, dir);
    loc_grid_at(env, m + 0x10, zone, d, pos);
    env.w32(m + 0x2C, 0);
    env.w8(m + 0x30, kind as u8);
    ev
}

/// C: `FUN_021854B0`: [`warp_new`] with kind 0.
pub fn warp0_new<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32, pos: u32, dir: u32) -> u32 {
    warp_new(env, gsys, 0, zone, pos, dir & 0xFFFF)
}

/// C: `FUN_021854CC`: a warp event ([`procs::WARP`]) to LOCATION `loc` with
/// direction `+0x18 = dir`, `+0x2C = 0`, kind `+0x30 = 1`; the player walks (`FUN_02012B2C(player, 0)`); the
/// special-location update `FUN_02185EB0`.
pub fn warp_loc_new<E: FieldEnv>(env: &mut E, gsys: u32, loc: u32, dir: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let pl = env.callv(PLAYER, &[gd]);
    env.callv(0x0201_2B2C, &[pl, 0]);
    let (ev, m) = new_event(env, gsys, procs::WARP);
    env.mem().copy(m + 0x10, loc, LOC);
    env.w16(m + 0x18, dir as u16);
    env.w32(m + 0x2C, 0);
    env.w8(m + 0x30, 1);
    env.call(SPECIAL_UPDATE, &mut [CArg::V(gd), CArg::In(m + 0x10, LOC)]);
    ev
}

/// C: `FUN_02185534`: a [`procs::FADE`] event to grid position `pos` of `zone`,
/// `+0x2C = 0`.
pub fn fade_new<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32, pos: u32, dir: u32) -> u32 {
    let (ev, m) = new_event(env, gsys, procs::FADE);
    let d = dir_code(env, dir);
    loc_grid_at(env, m + 0x10, zone, d, pos);
    env.w32(m + 0x2C, 0);
    ev
}

/// C: `FUN_02185588`: a [`procs::WAIT`] event to grid position `pos` of `zone`;
/// in a zone that `FUN_02013D5C` accepts with a comm record `r`, `x += d << 22` with `d = (u16 r+0x86 & 7) -
/// FUN_02012B58(gd)` (plus the byte `r+0x902` when negative); direction `FUN_020142A4(0)`, `+0x2C = +0x40 = 0`.
pub fn wait_new<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32, pos: u32) -> u32 {
    let mut x = env.r32(pos);
    let y = env.r32(pos + 4);
    let z = env.r32(pos + 8);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let ev = env.callv(EVENT_NEW, &[gsys, 0, procs::WAIT, WORK_SIZE]);
    let m = env.callv(EVENT_WORK, &[ev]);
    if env.callv(0x0201_3D5C, &[zone]) == 1 {
        let c = env.callv(COMM_SYS, &[gsys]);
        let r = env.callv(COMM_REC, &[c]);
        let mut d = 0u32;
        if r != 0 {
            d = (env.r16(r.wrapping_add(0x86)) & 7) as u32;
            d = d.wrapping_sub(env.callv(0x0201_2B58, &[gd]));
            if (d as i32) < 0 {
                d = d.wrapping_add(env.r8(r.wrapping_add(0x902)) as u32);
            }
        }
        x = x.wrapping_add(d << 22);
    }
    work_init(env, m, gsys);
    let dd = dir_code(env, 0);
    loc_grid(env, m + 0x10, zone, dd, x, y, z);
    env.w32(m + 0x2C, 0);
    env.w32(m + 0x40, 0);
    ev
}

/// C: `FUN_0218562C`: enter the zone of the special location: `a` from
/// `FUN_02159FA8(FUN_02012914(gd), &a)`, the GAMEDATA special LOCATION from the zone `FUN_02159F94(FUN_02012914(gd))`
/// (`FUN_020128C0`), then the zone switch of [`crate::mapchange_zone`] (`FUN_02185B70`, `FUN_02185A68(gsys, zone)`,
/// `FUN_021857C0(gsys, &a)`) and the hooks `FUN_0202735C(zone, gd)`, `FUN_02168EEC(gd)`, `FUN_0216446C(gd, zone)`.
pub fn special_enter<E: FieldEnv>(env: &mut E, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let x = env.callv(GD_2914, &[gd]);
    let a = env.mem().push_local(LOC);
    env.call(0x0215_9FA8, &mut [CArg::V(x), CArg::Out(a, LOC)]);
    let x = env.callv(GD_2914, &[gd]);
    let zone = env.callv(0x0215_9F94, &[x]);
    let b = env.mem().push_local(LOC);
    env.call(LOC_ZONE, &mut [CArg::Out(b, LOC), CArg::V(zone)]);
    env.call(0x0201_28C0, &mut [CArg::V(gd), CArg::In(b, LOC)]);
    env.callv(0x0218_5B70, &[gsys]);
    let zone = env.r16(a + 4) as u32;
    env.callv(0x0218_5A68, &[gsys, zone]);
    env.call(0x0218_57C0, &mut [CArg::V(gsys), CArg::In(a, LOC)]);
    let zone = env.r16(a + 4) as u32;
    env.callv(0x0202_735C, &[zone, gd]);
    env.callv(0x0216_8EEC, &[gd]);
    let zone = env.r16(a + 4) as u32;
    env.callv(0x0216_446C, &[gd, zone]);
    env.mem().pop_local(b);
    env.mem().pop_local(a);
}

#[cfg(test)]
mod tests;
