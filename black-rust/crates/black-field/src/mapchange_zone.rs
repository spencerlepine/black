//! `event_mapchange.c` part 3 (overlay 20, `0x02185694..0x02186278`, F1 row 20.00; F6b): the special-location event, the
//! zone entry (LOCATION resolve, the PLAYER record, the zone hooks, NPCs and WFBC people, weather), the zone data load
//! (event data, map matrix, the flash flags), the SB list of a zone (ARC 101), the comm map-change events, the
//! special-location updates, the default record (message bank 190), the two `event_mapchange.c` allocations and the WFBC
//! list helpers that the game-start events use.
//!
//! Parts 1 and 2 are [`crate::mapchange_proc`] and [`crate::mapchange_new`]. Spec:
//! `tools/rom-diff/specs/black-field/mapchange_zone.spec` (the header of `mapchange_proc.spec` documents the MAPCHANGE
//! work and the arena layout). Every callee outside this area (the other two mapchange areas included) is reached
//! through [`FieldEnv::call`] by address; callees inside it are Rust calls. Stack locals of the original are
//! [`Mem::push_local`] windows. Screen fades, sound and VRAM work are callees (hooks), never done here (D3).
//!
//! LOCATION (0x1C): `+0` kind (1 = copy as is, 3 = keep the special location), `+4` u16 zone, `+6` u16 warp, `+8` s16
//! direction code, `+0xC` rail flag (`020141DC`), `+0x10` the position (grid: 3 words, rail: 8 bytes).
//!
//! [`Mem::push_local`]: crate::hooks::Mem::push_local

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_5694,
    0x0218_571C,
    0x0218_5758,
    0x0218_5788,
    0x0218_57C0,
    0x0218_58E4,
    0x0218_5928,
    0x0218_59F0,
    0x0218_59FC,
    0x0218_5A34,
    0x0218_5A68,
    0x0218_5B70,
    0x0218_5B88,
    0x0218_5B9C,
    0x0218_5BFC,
    0x0218_5C4C,
    0x0218_5C8C,
    0x0218_5D38,
    0x0218_5EB0,
    0x0218_5F18,
    0x0218_5F48,
    0x0218_5F6C,
    0x0218_601C,
    0x0218_604C,
    0x0218_606C,
    0x0218_608C,
    0x0218_6090,
    0x0218_609C,
    0x0218_60B8,
    0x0218_60D4,
    0x0218_61BC,
    0x0218_61F4,
    0x0218_625C,
];

/// The special-location GMEVENT proc [`special_proc`] (Thumb address).
pub const SPECIAL_PROC: u32 = 0x0218_5695;
/// The comm map-change GMEVENT proc [`comm_proc`] (Thumb address).
pub const COMM_PROC: u32 = 0x0218_5D39;
/// The comm map-change GMEVENT proc [`comm_change_proc`] (Thumb address).
pub const COMM_CHANGE_PROC: u32 = 0x0218_5C8D;
/// Address of the string `"event_mapchange.c"` (the file argument of [`alloc_a`] / [`alloc_b`]).
pub const FILE_EVENT_MAPCHANGE: u32 = 0x0218_7EA0;

/// `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// `FUN_0201210C(gsys)`: the FIELDMAP of the game system.
const FIELDMAP: u32 = 0x0201_210C;
/// `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// `FUN_02012374(event, child)`: call the child GMEVENT.
const EVENT_CALL: u32 = 0x0201_2374;
/// `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK: u32 = 0x0201_24E8;
/// `FUN_020127A4(gd)`: the PLAYER record.
const PLAYER: u32 = 0x0201_27A4;
/// `FUN_020127C4(gd)`: WFBC list 1 of the GAMEDATA.
const WFBC_LIST: u32 = 0x0201_27C4;
/// `FUN_02012844(gd)`: the EVENTDATA_SYSTEM.
const EVENTDATA: u32 = 0x0201_2844;
/// `FUN_02012964(gd)`: the MMDLSYS.
const MMDLSYS: u32 = 0x0201_2964;
/// `FUN_0201296C(gd)`: the EVENT WORK.
const EVENT_WORK_OBJ: u32 = 0x0201_296C;
/// `FUN_02012984(gd)`: the season.
const SEASON: u32 = 0x0201_2984;
/// `FUN_02012B38(gd)`: the EVENT WORK +0x1B8 object.
const FLAG_OBJ: u32 = 0x0201_2B38;
/// `FUN_02012138(gsys)`: the intrude / comm system.
const COMM_SYS: u32 = 0x0201_2138;
/// `FUN_02158344(comm)`: the comm record (may be null).
const COMM_REC: u32 = 0x0215_8344;
/// `FUN_02013DD0(zone)`: zonedata predicate (WFBC zone).
const ZONE_WFBC: u32 = 0x0201_3DD0;
/// `FUN_0201444C(obj)`: a flag of the EVENT WORK +0x1B8 object.
const OBJ_FLAG: u32 = 0x0201_444C;
/// `FUN_0216CEAC(mmdlsys, list, zone, n, ew)`: add OBJs.
const MMDL_ADD: u32 = 0x0216_CEAC;
/// `FUN_02014204(loc, zone)`: LOCATION of a zone.
const LOC_ZONE: u32 = 0x0201_4204;
/// `FUN_020128C0(gd, loc)`: set the special LOCATION.
const SET_SPECIAL: u32 = 0x0201_28C0;
/// `FUN_02012B2C(pl, form)`: the PLAYER record's move form.
const SET_FORM: u32 = 0x0201_2B2C;
/// `Heap_Free(block)`.
const HEAP_FREE: u32 = 0x0203_07B0;
/// `Heap_AllocDebug(heap, size, flag, file, line)`.
const HEAP_ALLOC: u32 = 0x0203_0734;
/// `FUN_02158780(event, script, 0, 0, x)`: a script event.
const SCRIPT_EVENT: u32 = 0x0215_8780;
/// `FUN_02188C84(fm)`: a FIELDMAP getter.
const FM_GET: u32 = 0x0218_8C84;
/// `FUN_02184CEC(m, gsys)` (area `mapchange_new`): MAPCHANGE work init.
const WORK_INIT: u32 = 0x0218_4CEC;
/// `FUN_0218562C(gsys)` (area `mapchange_new`): the zone entry from the special location.
const SPECIAL_ENTER: u32 = 0x0218_562C;
/// The map-change GMEVENT proc `0x02184505` (area `mapchange_proc`).
const WARP_PROC: u32 = 0x0218_4505;
/// The size of a MAPCHANGE work.
const WORK_SIZE: u32 = 0x9C;
/// The size of a LOCATION.
const LOC: u32 = 0x1C;

/// C: `FUN_02185694`: the special-location event at step `*seq` of
/// work `w` (`+0` gsys, `+4` gd, `+8` LOCATION): 0: [`crate::mapchange_new`] zone entry `FUN_0218562C(gsys)`; 1: player
/// form 0, `FUN_0202C110(FUN_0202B578(FUN_02012134(gsys)))`, call the BGM event of zone `s16 w+0xC`; 2: call
/// `FUN_020145FC(gsys)`; 3: done (1). Steps 0..2 move on; other steps answer 0.
pub fn special_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    match env.r32(seq) {
        0 => {
            env.callv(SPECIAL_ENTER, &[gsys]);
        }
        1 => {
            let gd = env.callv(GAMEDATA, &[gsys]);
            let pl = env.callv(PLAYER, &[gd]);
            env.callv(SET_FORM, &[pl, 0]);
            let x = env.callv(0x0201_2134, &[gsys]);
            let y = env.callv(0x0202_B578, &[x]);
            env.callv(0x0202_C110, &[y]);
            let gd = env.r32(w + 4);
            let s = env.callv(SEASON, &[gd]);
            let zone = env.r16(w + 0xC) as i16 as u32;
            let bgm = env.callv(0x0202_9DF4, &[gd, zone, s]);
            let child = env.callv(0x0202_98A0, &[gsys, bgm, 0, 60]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        2 => {
            let child = env.callv(0x0201_45FC, &[gsys]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        3 => return 1,
        _ => return 0,
    }
    let s = env.r32(seq).wrapping_add(1);
    env.w32(seq, s);
    0
}

/// C: `FUN_0218571C`: the special-location event: [`SPECIAL_PROC`], work 0x24
/// `{gsys, gd, LOCATION}` with the LOCATION from `FUN_02159FA8(FUN_02012914(gd), w+8)`.
pub fn special_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    env.callv(GAMEDATA, &[gsys]);
    let ev = env.callv(EVENT_NEW, &[gsys, 0, SPECIAL_PROC, 0x24]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 4, gd);
    let x = env.callv(0x0201_2914, &[gd]);
    env.call(0x0215_9FA8, &mut [CArg::V(x), CArg::Out(w + 8, LOC)]);
    ev
}

/// C: `FUN_02185758`: the angle of direction code `d` (2: 0x8000, 3: 0x4000,
/// 4: 0xC000, others 0).
pub fn dir_angle(d: u32) -> u32 {
    match d {
        2 => 0x8000,
        3 => 0x4000,
        4 => 0xC000,
        _ => 0,
    }
}

/// C: `FUN_02185788`: resolve LOCATION `src` into `dst`: kind 1 is copied;
/// otherwise the exit location `FUN_021625FC(ed, dst, u16 src+6, u16 src+0xA)`, and when that fails the LOCATION of
/// zone `u16 src+4` (`FUN_02014204`).
pub fn resolve<E: FieldEnv>(env: &mut E, ed: u32, src: u32, dst: u32) {
    if env.r32(src) == 1 {
        env.mem().copy(dst, src, LOC);
        return;
    }
    let warp = env.r16(src + 6) as u32;
    let b = env.r16(src + 0xA) as u32;
    let r = env.call(
        0x0216_25FC,
        &mut [CArg::V(ed), CArg::Out(dst, LOC), CArg::V(warp), CArg::V(b)],
    );
    if r == 0 {
        let zone = env.r16(src + 4) as u32;
        env.call(LOC_ZONE, &mut [CArg::Out(dst, LOC), CArg::V(zone)]);
    }
}

/// C: `FUN_021857C0`: enter the zone of LOCATION `loc`: the resolved LOCATION
/// `l` ([`resolve`]; kind 3 keeps the special location `FUN_0201289C(gd, FUN_02012870(gd))`), the PLAYER record's zone,
/// direction ([`dir_angle`]) and position (grid `FUN_02012A90` / rail `FUN_02012AA4` by `FUN_020141DC`), the GAMEDATA
/// LOCATION (`FUN_02012854`), the zone id `FUN_02159FE0` (`FUN_02012900` when nonzero), [`sb_list`], the zone hooks
/// (`FUN_02168EEC`, `FUN_021643E8`, `FUN_0215A014`, `FUN_02158AE0(gsys, 4)`), [`zone_npcs`] of `loc`, [`flash_flags`],
/// [`wfbc_hook`], `FUN_0202C83C(gsys, zone)` and the weather (`FUN_0201405C(zone)` below 16: `FUN_0216E29C(mmdlsys, w)`,
/// else `FUN_0216CCCC(mmdlsys)`).
pub fn zone_enter<E: FieldEnv>(env: &mut E, gsys: u32, loc: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let pl = env.callv(PLAYER, &[gd]);
    let ed = env.callv(EVENTDATA, &[gd]);
    let o = env.callv(FLAG_OBJ, &[gd]);
    env.callv(0x0201_4454, &[o, 1]);
    let l = env.mem().push_local(LOC);
    resolve(env, ed, loc, l);
    if env.r32(l) == 3 {
        let p = env.callv(0x0201_2870, &[gd]);
        env.call(0x0201_289C, &mut [CArg::V(gd), CArg::In(p, LOC)]);
    }
    let zone = env.r16(l + 4) as u32;
    env.callv(0x0201_2AC4, &[pl, zone]);
    let d = env.r16(l + 8) as i16 as u32;
    let a = dir_angle(d);
    env.callv(0x0201_2ACC, &[pl, a]);
    if env.call(0x0201_41DC, &mut [CArg::In(l, LOC)]) == 0 {
        env.call(0x0201_2A90, &mut [CArg::V(pl), CArg::In(l + 0x10, 12)]);
        env.callv(0x0201_2ABC, &[pl, 0]);
    } else {
        env.call(0x0201_2AA4, &mut [CArg::V(pl), CArg::In(l + 0x10, 8)]);
        env.callv(0x0201_2ABC, &[pl, 1]);
    }
    let x = env.callv(0x0201_2134, &[gsys]);
    let zone = env.r16(l + 4) as u32;
    env.callv(0x0202_B544, &[x, zone]);
    env.call(0x0201_2854, &mut [CArg::V(gd), CArg::In(l, LOC)]);
    let sz = env.r16(l + 4) as i16 as u32;
    let id = env.callv(0x0215_9FE0, &[sz]) & 0xFFFF;
    if id != 0 {
        env.callv(0x0201_2900, &[gd, id]);
    }
    let sz = env.r16(l + 4) as i16 as u32;
    sb_list(env, gd, sz);
    env.callv(0x0216_8EEC, &[gd]);
    let zone = env.r16(l + 4) as u32;
    env.callv(0x0216_43E8, &[gd, zone]);
    let sz = env.r16(l + 4) as i16 as u32;
    env.callv(0x0215_A014, &[gd, sz]);
    env.callv(0x0215_8AE0, &[gsys, 4]);
    zone_npcs(env, gd, loc);
    let zone = env.r16(l + 4) as u32;
    flash_flags(env, gd, zone);
    wfbc_hook(env, gd, l);
    let sz = env.r16(l + 4) as i16 as u32;
    env.callv(0x0202_C83C, &[gsys, sz]);
    let zone = env.r16(l + 4) as u32;
    let k = env.callv(0x0201_405C, &[zone]);
    let mm = env.callv(MMDLSYS, &[gd]);
    if (k as i32) < 16 {
        env.callv(0x0216_E29C, &[mm, k]);
    } else {
        env.callv(0x0216_CCCC, &[mm]);
    }
    env.mem().pop_local(l);
}

/// C: `FUN_021858E4`: the NPCs of the zone of `loc`: when the event data has an
/// NPC list (`m = FUN_02162744(ed)`), `FUN_0216CEAC(mmdlsys, FUN_02162740(ed), s16 loc+4, m, event work)`.
pub fn zone_npcs<E: FieldEnv>(env: &mut E, gd: u32, loc: u32) {
    let ed = env.callv(EVENTDATA, &[gd]);
    let m = env.callv(0x0216_2744, &[ed]);
    if m == 0 {
        return;
    }
    let ew = env.callv(EVENT_WORK_OBJ, &[gd]);
    let mm = env.callv(MMDLSYS, &[gd]);
    let list = env.callv(0x0216_2740, &[ed]);
    let zone = env.r16(loc + 4) as i16 as u32;
    env.callv(MMDL_ADD, &[mm, list, zone, m, ew]);
}

/// C: `FUN_02185928`: the WFBC people of the zone of `loc` (only in zones
/// `FUN_02013DD0` accepts): `i = FUN_0201444C(FUN_02012B38(gd))`, the list `i ? FUN_020127D0(gd, 0) :
/// FUN_020127C4(gd)`; the people block `FUN_02186538(list, i, 1)` with `FUN_02186278(list, i)` and the block
/// `FUN_021868B8(x, i, u16 list+2, 1)` of `x = FUN_020127E0(gd)` with `FUN_02186854(x)` are added with
/// `FUN_0216CEAC` when both are nonzero, and freed.
pub fn wfbc_people<E: FieldEnv>(env: &mut E, gd: u32, loc: u32) {
    let zone = env.r16(loc + 4) as u32;
    if env.callv(ZONE_WFBC, &[zone]) == 0 {
        return;
    }
    let o = env.callv(FLAG_OBJ, &[gd]);
    let i = env.callv(OBJ_FLAG, &[o]);
    let mm = env.callv(MMDLSYS, &[gd]);
    let ew = env.callv(EVENT_WORK_OBJ, &[gd]);
    let list = if i == 0 {
        env.callv(WFBC_LIST, &[gd])
    } else {
        env.callv(0x0201_27D0, &[gd, 0])
    };
    let a = env.callv(0x0218_6538, &[list, i, 1]);
    let n = env.callv(0x0218_6278, &[list, i]);
    if a != 0 && n != 0 {
        let zone = env.r16(loc + 4) as i16 as u32;
        env.callv(MMDL_ADD, &[mm, a, zone, n, ew]);
    }
    if a != 0 {
        env.callv(HEAP_FREE, &[a]);
    }
    let x = env.callv(0x0201_27E0, &[gd]);
    let n = env.callv(0x0218_6854, &[x]);
    let k = env.r16(list + 2) as u32;
    let b = env.callv(0x0218_68B8, &[x, i, k, 1]);
    if b != 0 && n != 0 {
        let zone = env.r16(loc + 4) as i16 as u32;
        env.callv(MMDL_ADD, &[mm, b, zone, n, ew]);
    }
    if b != 0 {
        env.callv(HEAP_FREE, &[b]);
    }
}

/// C: `FUN_021859F0`: `FUN_0216CF6C(FUN_02012964(gd))`.
pub fn mmdl_reset<E: FieldEnv>(env: &mut E, gd: u32) {
    let mm = env.callv(MMDLSYS, &[gd]);
    env.callv(0x0216_CF6C, &[mm]);
}

/// C: `FUN_021859FC`: `FUN_02186470(gd)`; in WFBC zones ([`wfbc_people`]
/// first) `FUN_021864DC(FUN_020127C4(gd))` when the `FUN_0201444C` flag is clear.
pub fn wfbc_hook<E: FieldEnv>(env: &mut E, gd: u32, loc: u32) {
    env.callv(0x0218_6470, &[gd]);
    let zone = env.r16(loc + 4) as u32;
    if env.callv(ZONE_WFBC, &[zone]) == 0 {
        return;
    }
    wfbc_people(env, gd, loc);
    let o = env.callv(FLAG_OBJ, &[gd]);
    if env.callv(OBJ_FLAG, &[o]) == 0 {
        let list = env.callv(WFBC_LIST, &[gd]);
        env.callv(0x0218_64DC, &[list]);
    }
}

/// C: `FUN_02185A34`: the flash flags of `zone`: `f = FUN_02013F6C(zone)`; bit 0
/// with the flash already on (`FUN_02014468`) becomes bit 1; `FUN_0201446C(obj, f)`.
pub fn flash_flags<E: FieldEnv>(env: &mut E, gd: u32, zone: u32) {
    let o = env.callv(FLAG_OBJ, &[gd]);
    let mut f = env.callv(0x0201_3F6C, &[zone]);
    if f & 1 != 0 && env.callv(0x0201_4468, &[o]) != 0 {
        f = (f & !1) | 2;
    }
    env.callv(0x0201_446C, &[o, f]);
}

/// C: `FUN_02185A68`: load the data of `zone`: the `FUN_02014450` flag (`FUN_02012B70(gd)
/// == 1` and not `FUN_02013D5C(zone)`), the GAMEDATA kind `FUN_020129B8` (2 / 5 for the zonedata predicates
/// `FUN_02013CC8` / `FUN_02013CE0`, `FUN_02013DF0`, `FUN_02013D20`; otherwise 3 or 0 by `FUN_02012B70` and `FUN_020129AC`, then
/// 0 unless the kind is 3, 4 or 6), the event data (`FUN_02162408(ed, zone, season)`), the map matrix
/// (`FUN_02159B94(mm, FUN_02013B44(zone), zone, 0x8001)`, `FUN_02159CC4(mm, gsys, 0x8001)`) and [`flash_flags`].
pub fn zone_load<E: FieldEnv>(env: &mut E, gsys: u32, zone: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let ed = env.callv(EVENTDATA, &[gd]);
    env.callv(FIELDMAP, &[gsys]);
    let on = env.callv(0x0201_2B70, &[gd]) == 1 && env.callv(0x0201_3D5C, &[zone]) == 0;
    let o = env.callv(FLAG_OBJ, &[gd]);
    env.callv(0x0201_4450, &[o, on as u32]);
    if env.callv(0x0201_3CC8, &[zone]) != 0 {
        env.callv(0x0201_29B8, &[gd, 2]);
    } else if env.callv(0x0201_3CE0, &[zone]) != 0
        || env.callv(0x0201_3DF0, &[zone]) != 0
        || env.callv(0x0201_3D20, &[zone]) != 0
    {
        env.callv(0x0201_29B8, &[gd, 5]);
    } else {
        if env.callv(0x0201_2B70, &[gd]) == 1 && env.callv(0x0201_29AC, &[gd]) != 3 {
            env.callv(0x0201_29B8, &[gd, 3]);
        } else if env.callv(0x0201_2B70, &[gd]) == 0 && env.callv(0x0201_29AC, &[gd]) == 3 {
            env.callv(0x0201_29B8, &[gd, 0]);
        }
        let k = env.callv(0x0201_29AC, &[gd]);
        if k != 3 && k != 4 && k != 6 {
            env.callv(0x0201_29B8, &[gd, 0]);
        }
    }
    let s = env.callv(SEASON, &[gd]);
    env.callv(0x0216_2408, &[ed, zone, s]);
    let mm = env.callv(0x0201_2974, &[gd]);
    let id = env.callv(0x0201_3B44, &[zone]);
    env.callv(0x0215_9B94, &[mm, id, zone, 0x8001]);
    env.callv(0x0215_9CC4, &[mm, gsys, 0x8001]);
    flash_flags(env, gd, zone);
}

/// C: `FUN_02185B70`: leave the zone: `FUN_0201210C(gsys)` (unused), [`mmdl_reset`].
pub fn zone_leave<E: FieldEnv>(env: &mut E, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.callv(FIELDMAP, &[gsys]);
    mmdl_reset(env, gd);
}

/// C: `FUN_02185B88`: `FUN_0216DAC8(FUN_021A3074(FUN_02188CB4(fm)), x)` (the player's MMDL).
pub fn player_mmdl_set<E: FieldEnv>(env: &mut E, fm: u32, x: u32) {
    let pl = env.callv(0x0218_8CB4, &[fm]);
    let m = env.callv(0x021A_3074, &[pl]);
    env.callv(0x0216_DAC8, &[m, x]);
}

/// C: `FUN_02185B9C`: the SB list of `zone`: clear it (`FUN_0200B37C(sb)`, `sb =
/// FUN_02012EC4(gd)`), load member 0 of ARC 101 (heap 0x8001; records of 8 bytes: `u32` zone, `u16` id) and add the id
/// of the first record of `zone` (`FUN_0200B394`); free the table, close the archive.
pub fn sb_list<E: FieldEnv>(env: &mut E, gd: u32, zone: u32) {
    let sb = env.callv(0x0201_2EC4, &[gd]);
    env.callv(0x0200_B37C, &[sb]);
    let h = env.callv(0x0204_90F4, &[101, 0x8001]);
    let t = env.callv(0x0204_9250, &[h, 0, 0x8001]);
    let n = env.callv(0x0204_93D4, &[h, 0]) >> 3;
    for i in 0..n {
        let r = t.wrapping_add(i * 8);
        if env.r32(r) == zone {
            let id = env.r16(r + 4) as u32;
            env.callv(0x0200_B394, &[sb, id]);
            break;
        }
    }
    env.callv(HEAP_FREE, &[t]);
    env.callv(0x0204_9238, &[h]);
}

/// C: `FUN_02185BFC`: the comm map-change event: [`COMM_PROC`], work 0x30
/// `{LOCATION *loc, gsys, gd, fm, a, u8 b}`.
pub fn comm_new<E: FieldEnv>(env: &mut E, gsys: u32, fm: u32, loc: u32, a: u32, b: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, COMM_PROC, 0x30]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.mem().copy(w, loc, LOC);
    env.w32(w + 0x1C, gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 0x20, gd);
    env.w32(w + 0x24, fm);
    env.w32(w + 0x28, a);
    env.w8(w + 0x2C, b as u8);
    ev
}

/// C: `FUN_02185C4C`: the comm map-change child event: [`COMM_CHANGE_PROC`],
/// work 0x30 `{LOCATION *loc, gsys, gd, fm}`.
pub fn comm_change_new<E: FieldEnv>(env: &mut E, gsys: u32, fm: u32, loc: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, COMM_CHANGE_PROC, 0x30]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.mem().copy(w, loc, LOC);
    env.w32(w + 0x1C, gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 0x20, gd);
    env.w32(w + 0x24, fm);
    ev
}

/// C: `FUN_02185C8C`: the comm map change at step `*seq` of work `w`: 0: sound
/// effect 0x773, `FUN_021CF498(FUN_02189A08(fm), event, 36)`; 1: call a map-change event (proc `0x02184505`, the MAPCHANGE
/// work initialised by `FUN_02184CEC` with the LOCATION of `w`, `+0x2C = +0x40 = 0`); 2: call `FUN_021BE9F0(gsys, fm, 0, 0,
/// 2)`; 3: done (1). Steps 0..2 move on; other steps answer 0.
pub fn comm_change_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w + 0x1C);
    let fm = env.r32(w + 0x24);
    match env.r32(seq) {
        0 => {
            env.callv(0x0200_61E4, &[0x773]);
            let x = env.callv(0x0218_9A08, &[fm]);
            env.callv(0x021C_F498, &[x, event, 36]);
        }
        1 => {
            let child = env.callv(EVENT_NEW, &[gsys, 0, WARP_PROC, WORK_SIZE]);
            let m = env.callv(EVENT_WORK, &[child]);
            env.callv(WORK_INIT, &[m, gsys]);
            env.mem().copy(m + 0x10, w, LOC);
            env.w32(m + 0x2C, 0);
            env.w32(m + 0x40, 0);
            env.callv(EVENT_CALL, &[event, child]);
        }
        2 => {
            let child = env.callv(0x021B_E9F0, &[gsys, fm, 0, 0, 2]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        3 => return 1,
        _ => return 0,
    }
    let s = env.r32(seq).wrapping_add(1);
    env.w32(seq, s);
    0
}

/// C: `FUN_02185D38`: the comm map change at step `*seq` of work `w` (comm system
/// `c = FUN_02012138(gsys)`, record `FUN_02158344(c)`): 0: scripts 0x279A / 0x279B (`FUN_021D2750(gsys)` and no
/// `FUN_0217C070` record answer; 0x279B when `FUN_02158630(c) == 3`; `FUN_02026400(c)` unless `FUN_02158590(gd)`), next
/// step 1, else scripts 0x279C / 0x279D, next step 3; 1: wait for the comm (`FUN_02026454(c)`, unless `FUN_02158590(gd)`);
/// `w+0x28 == 1` with `FUN_02158630(c) != 3`: script 0x279F, step 3; else the record updates (`FUN_0217574C(r, 1)`,
/// `FUN_02175710(r, u8 w+0x2C)`, `FUN_02175B4C(r, r + 0x1AC)` for `w+0x28 == 1`), `FUN_02012B78(gd, 1)`,
/// `FUN_02012B64(gd, FUN_02039338())`, call [`comm_change_new`] of `w`, step 2; 2: `FUN_0217574C(r, 0)`, step 3; 3: done.
pub fn comm_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w + 0x1C);
    let c = env.callv(COMM_SYS, &[gsys]);
    match env.r32(seq) {
        0 => {
            let r = env.callv(COMM_REC, &[c]);
            let mut alt = 0;
            let mut busy = 0;
            if r != 0 {
                busy = env.callv(0x0217_C070, &[r]);
                if env.callv(0x0215_8630, &[c]) == 3 {
                    alt = 1;
                }
            }
            let next = if busy == 0 && env.callv(0x021D_2750, &[gsys]) != 0 {
                let script = if alt != 0 { 0x279B } else { 0x279A };
                let fm = env.r32(w + 0x24);
                let x = env.callv(FM_GET, &[fm]);
                env.callv(SCRIPT_EVENT, &[event, script, 0, 0, x]);
                let gd = env.r32(w + 0x20);
                if env.callv(0x0215_8590, &[gd]) == 0 {
                    env.callv(0x0202_6400, &[c]);
                }
                1
            } else {
                let script = if busy != 0 { 0x279D } else { 0x279C };
                let fm = env.r32(w + 0x24);
                let x = env.callv(FM_GET, &[fm]);
                env.callv(SCRIPT_EVENT, &[event, script, 0, 0, x]);
                3
            };
            env.w32(seq, next);
            0
        }
        1 => {
            let gd = env.r32(w + 0x20);
            if env.callv(0x0215_8590, &[gd]) == 0 {
                if env.callv(0x0202_6454, &[c]) != 0 {
                    return 0;
                }
                env.callv(0x0202_6484, &[c]);
            }
            if env.r32(w + 0x28) == 1 && env.callv(0x0215_8630, &[c]) != 3 {
                let fm = env.r32(w + 0x24);
                let x = env.callv(FM_GET, &[fm]);
                env.callv(SCRIPT_EVENT, &[event, 0x279F, 0, 0, x]);
                env.w32(seq, 3);
                return 0;
            }
            let r = env.callv(COMM_REC, &[c]);
            if r != 0 {
                env.callv(0x0217_574C, &[r, 1]);
                let b = env.r8(w + 0x2C) as u32;
                env.callv(0x0217_5710, &[r, b]);
                if env.r32(w + 0x28) == 1 {
                    env.callv(0x0217_5B4C, &[r, r.wrapping_add(0x1AC)]);
                }
            }
            let gd = env.r32(w + 0x20);
            env.callv(0x0201_2B78, &[gd, 1]);
            let id = env.callv(0x0203_9338, &[]);
            let gd = env.r32(w + 0x20);
            env.callv(0x0201_2B64, &[gd, id]);
            let gsys = env.r32(w + 0x1C);
            let fm = env.r32(w + 0x24);
            let child = comm_change_new(env, gsys, fm, w);
            env.callv(EVENT_CALL, &[event, child]);
            env.w32(seq, 2);
            0
        }
        2 => {
            let r = env.callv(COMM_REC, &[c]);
            if r != 0 {
                env.callv(0x0217_574C, &[r, 0]);
            }
            env.w32(seq, 3);
            0
        }
        3 => 1,
        _ => 0,
    }
}

/// C: `FUN_02185EB0`: update the special location when entering the zone of
/// `loc`: `l1 = FUN_02012870(gd)`, `l2 = FUN_0201284C(gd)`; for a zone that `FUN_02014068` accepts, `l2`'s zone 136 sets
/// `l1` (`FUN_020128C0`), else a `l2` zone that `FUN_02014068` refuses sets the LOCATION of zone 214; `l2`'s zone 158 with
/// `FUN_02013C50(zone)` sets `l1`.
pub fn special_update<E: FieldEnv>(env: &mut E, gd: u32, loc: u32) {
    let l1 = env.callv(0x0201_2870, &[gd]);
    let l2 = env.callv(0x0201_284C, &[gd]);
    let zone = env.r16(loc + 4) as u32;
    if env.callv(0x0201_4068, &[zone]) != 0 {
        let z = env.r16(l2 + 4) as i16;
        if z == 136 {
            env.call(SET_SPECIAL, &mut [CArg::V(gd), CArg::In(l1, LOC)]);
        } else if env.callv(0x0201_4068, &[z as u16 as u32]) == 0 {
            let t = env.mem().push_local(LOC);
            env.call(LOC_ZONE, &mut [CArg::Out(t, LOC), CArg::V(214)]);
            env.call(SET_SPECIAL, &mut [CArg::V(gd), CArg::In(t, LOC)]);
            env.mem().pop_local(t);
        }
    }
    if env.r16(l2 + 4) as i16 == 158 {
        let zone = env.r16(loc + 4) as u32;
        if env.callv(0x0201_3C50, &[zone]) != 0 {
            env.call(SET_SPECIAL, &mut [CArg::V(gd), CArg::In(l1, LOC)]);
        }
    }
}

/// C: `FUN_02185F18`: copy the GAMEDATA LOCATION (`FUN_02012894`) into `dst`
/// when `FUN_02013FEC` accepts the zone of `FUN_0201284C(gd)`.
pub fn special_copy<E: FieldEnv>(env: &mut E, gd: u32, dst: u32) {
    let l2 = env.callv(0x0201_284C, &[gd]);
    let zone = env.r16(l2 + 4) as u32;
    if env.callv(0x0201_3FEC, &[zone]) != 0 {
        let src = env.callv(0x0201_2894, &[gd]);
        env.mem().copy(dst, src, LOC);
    }
}

/// C: `FUN_02185F48`: the PLAYER record's move form: 3 becomes 2, everything
/// else 0.
pub fn form_reset<E: FieldEnv>(env: &mut E, gd: u32) {
    let pl = env.callv(PLAYER, &[gd]);
    let f = if env.callv(0x0201_2B28, &[pl]) == 3 {
        2
    } else {
        0
    };
    env.callv(SET_FORM, &[pl, f]);
}

/// C: `FUN_02185F6C`: build a default record (`FUN_02008F08(1)`): the name
/// from string 59 of message bank 190 (`FUN_02045B38(0, 2, 190, 1)`, `FUN_02045EC0`, 16 characters through
/// `FUN_02045A38`), fields 1 / 3 / 2 / 21 (`FUN_02008FA0` / `FA8` / `FB4` / `FBC`), the ROM strings `0x02187DA4` /
/// `0x02187DAC` / `0x02187DB4` (`FUN_02008FD8(rec, s, i)`), value 0x58F (`FUN_02008FC4`); store it with
/// `FUN_0200903C(FUN_02008FEC(gd), 0, 0, rec)` and free everything.
pub fn default_record<E: FieldEnv>(env: &mut E, gd: u32) {
    let m = env.callv(0x0204_5B38, &[0, 2, 190, 1]);
    let s = env.callv(0x0204_5EC0, &[m, 59]);
    let buf = env.mem().push_local(32);
    env.call(
        0x0204_5A38,
        &mut [CArg::V(s), CArg::Out(buf, 32), CArg::V(16)],
    );
    let rec = env.callv(0x0200_8F08, &[1]);
    env.call(0x0200_8F88, &mut [CArg::V(rec), CArg::In(buf, 32)]);
    env.callv(0x0200_8FA0, &[rec, 1]);
    env.callv(0x0200_8FA8, &[rec, 3]);
    env.callv(0x0200_8FB4, &[rec, 2]);
    env.callv(0x0200_8FBC, &[rec, 21]);
    env.callv(0x0200_8FD8, &[rec, 0x0218_7DA4, 0]);
    env.callv(0x0200_8FD8, &[rec, 0x0218_7DAC, 1]);
    env.callv(0x0200_8FD8, &[rec, 0x0218_7DB4, 2]);
    env.callv(0x0200_8FC4, &[rec, 0x58F]);
    let store = env.callv(0x0200_8FEC, &[gd]);
    env.callv(0x0200_903C, &[store, 0, 0, rec]);
    env.callv(HEAP_FREE, &[rec]);
    env.callv(0x0204_5808, &[s]);
    env.callv(0x0204_5C04, &[m]);
    env.mem().pop_local(buf);
}

/// C: `FUN_0218601C`: `FUN_0216A358(gd, FUN_0201AD0C(sv), FUN_02008B14(sv) + 20,
/// season)` with `sv = FUN_02012EBC(gd)`.
pub fn season_records<E: FieldEnv>(env: &mut E, gd: u32) {
    let sv = env.callv(0x0201_2EBC, &[gd]);
    let a = env.callv(0x0201_AD0C, &[sv]);
    let s = env.callv(SEASON, &[gd]);
    let b = env.callv(0x0200_8B14, &[sv]);
    env.callv(0x0216_A358, &[gd, a, b.wrapping_add(20), s]);
}

/// C: `FUN_0218604C`: `Heap_AllocDebug(0x8004, 0x1000, 0, "event_mapchange.c", 0xE35)`.
pub fn alloc_a<E: FieldEnv>(env: &mut E) -> u32 {
    env.callv(
        HEAP_ALLOC,
        &[0x8004, 0x1000, 0, FILE_EVENT_MAPCHANGE, 0xE35],
    )
}

/// C: `FUN_0218606C`: `Heap_AllocDebug(0x8004, 0x1000, 0, "event_mapchange.c", 0xE3A)`.
pub fn alloc_b<E: FieldEnv>(env: &mut E) -> u32 {
    env.callv(
        HEAP_ALLOC,
        &[0x8004, 0x1000, 0, FILE_EVENT_MAPCHANGE, 0xE3A],
    )
}

/// C: `FUN_0218608C`: an empty function.
pub fn nop() {}

/// C: `FUN_02186090`: `FUN_0216D588(x) - 0x80` (the OBJ id of an MMDL minus 0x80).
pub fn obj_index<E: FieldEnv>(env: &mut E, x: u32) -> u32 {
    env.callv(0x0216_D588, &[x]).wrapping_sub(0x80)
}

/// C: `FUN_0218609C`: clear the 10 records of WFBC list A (`FUN_02161E78(l + 8 + 24 i)`).
pub fn list_a_clear<E: FieldEnv>(env: &mut E, l: u32) {
    for i in 0..10 {
        env.callv(0x0216_1E78, &[l + 8 + 24 * i]);
    }
}

/// C: `FUN_021860B8`: clear the 10 records of WFBC list B (`FUN_02161E78(l + 0xF8 + 24 i)`).
pub fn list_b_clear<E: FieldEnv>(env: &mut E, l: u32) {
    for i in 0..10 {
        env.callv(0x0216_1E78, &[l + 0xF8 + 24 * i]);
    }
}

/// C: `FUN_021860D4`: generate the WFBC people list `l`: `+0 = 1`, `+2 = 0`,
/// `+4` the trainer id of `ms` (`FUN_02008530`); with the people data `h = FUN_021867C4(0, heap & 0x7FFF | 0x8000)`, 10
/// records of list A (`+8`) and 3 of list B (`+0xF8`): a pick `r = FUN_021869E0(l, rand(budget), h, &k)` written into
/// the free record `FUN_02186B60(list)` by `FUN_02186B84(rec, (u8) k, ms, x, flag)` (list A: `x = u16 r+6`, flag 1 while
/// `u16 l+2 == 0`, else `u16 r+4`, flag 0; list B: `x = 0` with the last flag), the budget 0x1CE minus each pick's
/// `u16 r+44`; `FUN_02186804(h)`.
pub fn people_generate<E: FieldEnv>(env: &mut E, l: u32, ms: u32, heap: u32) {
    env.w16(l, 1);
    env.w16(l + 2, 0);
    let tid = env.callv(0x0200_8530, &[ms]);
    env.w32(l + 4, tid);
    let h = env.callv(0x0218_67C4, &[0, ((heap & 0x7FFF) | 0x8000) & 0xFFFF]);
    env.callv(0x0218_682C, &[h]);
    let mut budget: u32 = 0x1CE;
    let k = env.mem().push_local(4);
    let mut flag = 0;
    for _ in 0..10 {
        let r = env.callv(0x0200_56EC, &[budget]);
        let rec = env.callv(0x0218_6B60, &[l + 8]);
        let pick = env.call(
            0x0218_69E0,
            &mut [CArg::Out(l, 4), CArg::V(r), CArg::V(h), CArg::Out(k, 4)],
        );
        let x = if env.r16(l + 2) == 0 {
            flag = 1;
            env.r16(pick + 6)
        } else {
            flag = 0;
            env.r16(pick + 4)
        } as u32;
        let kk = env.r32(k) & 0xFF;
        env.callv(0x0218_6B84, &[rec, kk, ms, x, flag]);
        budget = budget.wrapping_sub(env.r16(pick + 44) as u32);
    }
    for _ in 0..3 {
        let r = env.callv(0x0200_56EC, &[budget]);
        let rec = env.callv(0x0218_6B60, &[l + 0xF8]);
        let pick = env.call(
            0x0218_69E0,
            &mut [CArg::Out(l, 4), CArg::V(r), CArg::V(h), CArg::Out(k, 4)],
        );
        let kk = env.r32(k) & 0xFF;
        env.callv(0x0218_6B84, &[rec, kk, ms, 0, flag]);
        budget = budget.wrapping_sub(env.r16(pick + 44) as u32);
    }
    env.mem().pop_local(k);
    env.callv(0x0218_6804, &[h]);
}

/// C: `FUN_021861BC`: 0 for a list in state 1 with `u16 l+2 >= 2`; otherwise 1
/// when every list-A record answers `FUN_02161E84`, else 0.
pub fn list_ready<E: FieldEnv>(env: &mut E, l: u32) -> u32 {
    if env.r16(l) == 1 && env.r16(l + 2) >= 2 {
        return 0;
    }
    for i in 0..10 {
        if env.callv(0x0216_1E84, &[l + 8 + 24 * i]) == 0 {
            return 0;
        }
    }
    1
}

/// C: `FUN_021861F4`: normalise the list state (`u16 l+0` above 1 becomes 1;
/// state 0 sets `u16 l+2 = 2`; state 1 with `l+2 >= 2` sets `l+2` to 0 when `FUN_021584F4((u8) FUN_0200855C(ms),
/// FUN_02008530(ms)) == 21`, else 1) and `FUN_021866F8` each list-A record.
pub fn list_update<E: FieldEnv>(env: &mut E, l: u32, ms: u32) {
    if env.r16(l) > 1 {
        env.w16(l, 1);
    }
    if env.r16(l) == 0 && env.r16(l + 2) != 2 {
        env.w16(l + 2, 2);
    }
    if env.r16(l) == 1 && env.r16(l + 2) >= 2 {
        let a = env.callv(0x0200_855C, &[ms]);
        let b = env.callv(0x0200_8530, &[ms]);
        let v = if env.callv(0x0215_84F4, &[a & 0xFF, b]) == 21 {
            0
        } else {
            1
        };
        env.w16(l + 2, v);
    }
    for i in 0..10 {
        env.callv(0x0218_66F8, &[l + 8 + 24 * i]);
    }
}

/// C: `FUN_0218625C`: 0 for a list in state 0, else [`list_ready`] as 0 / 1.
pub fn list_check<E: FieldEnv>(env: &mut E, l: u32) -> u32 {
    if env.r16(l) == 0 {
        return 0;
    }
    (list_ready(env, l) != 0) as u32
}

#[cfg(test)]
mod tests;
