//! `event_mapchange.c` part 1 (overlay 20, `0x02183B60..0x02184CEC`, F1 row 20.00; F6b): the game-start events (new
//! game, continue, the event chosen by kind), the MAPCHANGE work helpers (the season-change check, the door /
//! transition sub-record), the zone-change child event (overlay 0xE6 loaded around the zone switch, the integrity
//! calls, the zone hooks) and the map-change GMEVENT procs (door, warp, stairs / fade, special location, union room ...).
//!
//! Parts 2 and 3 are [`crate::mapchange_new`] (the constructors that start these procs) and [`crate::mapchange_zone`]
//! (the zone entry and data load). Spec: `tools/rom-diff/specs/black-field/mapchange_proc.spec` (its header documents
//! the MAPCHANGE work). Every callee outside this area (the other two mapchange areas included) is reached through
//! [`FieldEnv::call`] by address; callees inside it are Rust calls. The screen fades, the VRAM clears (`0208068C`,
//! `02082AC0`, `02080B98`), sound and the other field systems are callees (hooks; D3).
//!
//! MAPCHANGE work M (0x9C, `crate::mapchange_new::work_init`): `+0` GAMESYS, `+4` GAMEDATA, `+8` FIELDMAP, `+0xC` u16
//! the zone being left, `+0x10` the destination LOCATION (0x1C), `+0x2C` the door kind, `+0x30` u8 the zone-change kind
//! (0..4), `+0x34` a 12-byte position, `+0x40` / `+0x44` the season-change flags, `+0x48` / `+0x4A` u16 the new / old
//! season, `+0x4C` the door / transition sub-record ([`door_sub`]).
//!
//! A GMEVENT proc `(event, seq, work)` returns 1 when done, else 0 (`seq` is the address of the step word; the door
//! procs answer 33 after their first door step, as the original does).

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_3B60,
    0x0218_3B6C,
    0x0218_3B90,
    0x0218_3C34,
    0x0218_3C58,
    0x0218_3C9C,
    0x0218_3D68,
    0x0218_3DD4,
    0x0218_3DE4,
    0x0218_3F5C,
    0x0218_403C,
    0x0218_409C,
    0x0218_4120,
    0x0218_41C0,
    0x0218_4360,
    0x0218_4388,
    0x0218_4410,
    0x0218_4504,
    0x0218_4584,
    0x0218_466C,
    0x0218_4730,
    0x0218_47CC,
    0x0218_4894,
    0x0218_495C,
    0x0218_4A14,
    0x0218_4AB4,
    0x0218_4B54,
    0x0218_4BF8,
    0x0218_4C78,
];

/// The callees that answer a 64-bit value (`r0:r1`), which [`FieldEnv::call`] cannot return.
pub trait MapChangeEnv: FieldEnv {
    /// Calls the routine at `addr` (no arguments) and returns `r1 << 32 | r0`.
    fn call64(&mut self, addr: u32) -> u64;
}

/// GMEVENT procs of this area (Thumb addresses).
pub mod procs {
    /// [`super::start_proc`].
    pub const START: u32 = 0x0218_3B91;
    /// [`super::new_game_proc`].
    pub const NEW_GAME: u32 = 0x0218_3C9D;
    /// [`super::continue_proc`].
    pub const CONTINUE: u32 = 0x0218_3DE5;
    /// [`super::zone_change_proc`].
    pub const ZONE_CHANGE: u32 = 0x0218_41C1;
    /// [`super::warp_proc`] (used for the copies of [`super::zone_e_proc`]).
    pub const WARP: u32 = 0x0218_4505;
}

/// The integrity-check callback invoker [`call_cb`] (Thumb address) and the callbacks handed to it.
const CHECK_CB: u32 = 0x0218_3B61;

/// `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// `FUN_0201210C(gsys)`: the FIELDMAP of the game system.
const FIELDMAP: u32 = 0x0201_210C;
/// `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// `FUN_0201235C(event, next)`: change the GMEVENT to `next`.
const EVENT_CHANGE: u32 = 0x0201_235C;
/// `FUN_02012374(event, child)`: call the child GMEVENT.
const EVENT_CALL: u32 = 0x0201_2374;
/// `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK: u32 = 0x0201_24E8;
/// `FUN_020127C4(gd)`: WFBC list 1 of the GAMEDATA.
const WFBC_LIST: u32 = 0x0201_27C4;
/// `FUN_02012984(gd)`: the season.
const SEASON: u32 = 0x0201_2984;
/// `FUN_02012A30(gd)`: the BGM player object.
const BGM_OBJ: u32 = 0x0201_2A30;
/// `FUN_02012EBC(gd)`: the save data.
const SAVE: u32 = 0x0201_2EBC;
/// `FUN_02161824(gsys, fm)`: the leave-the-map event.
const LEAVE_EVENT: u32 = 0x0216_1824;
/// `FUN_02188D20(fm)`: the place-name system.
const PLACE_NAME: u32 = 0x0218_8D20;
/// `FUN_021BE8D4(gsys, fm, kind, 0)`: fade-out event.
const FADE_OUT: u32 = 0x021B_E8D4;
/// `FUN_021BE92C(gsys, fm, kind, 0, a, b, c)`: fade-in event.
const FADE_IN: u32 = 0x021B_E92C;
/// `FUN_02188C7C(fm)`: the MMDLSYS of the FIELDMAP.
const FM_MMDLSYS: u32 = 0x0218_8C7C;
/// The size of a MAPCHANGE work.
const WORK_SIZE: u32 = 0x9C;
/// The size of a LOCATION.
const LOC: u32 = 0x1C;

/// `seq += 1`.
fn next<E: FieldEnv>(env: &mut E, seq: u32) {
    let s = env.r32(seq).wrapping_add(1);
    env.w32(seq, s);
}

/// `FUN_02012374(event, child)`.
fn call_child<E: FieldEnv>(env: &mut E, event: u32, child: u32) {
    env.callv(EVENT_CALL, &[event, child]);
}

/// The VRAM clear the game-start events end with (D3 hooks): `FUN_0208068C(0x1FF)`, `FUN_02082AC0(0, 0x06800000,
/// 0xA4000)`, `FUN_02080B98()`.
fn vram_clear<E: FieldEnv>(env: &mut E) {
    env.callv(0x0208_068C, &[0x1FF]);
    env.callv(0x0208_2AC0, &[0, 0x0680_0000, 0xA_4000]);
    env.callv(0x0208_0B98, &[]);
}

/// The zone BGM step: `FUN_02029D44(p, gd, u16 M+0x14, u8 M+0x4A)`, `FUN_02029D7C(p)`.
fn zone_bgm<E: FieldEnv>(env: &mut E, m: u32, p: u32, gd: u32) {
    let zone = env.r16(m + 0x14) as u32;
    let s = env.r16(m + 0x4A) as u32 & 0xFF;
    env.callv(0x0202_9D44, &[p, gd, zone, s]);
    env.callv(0x0202_9D7C, &[p]);
}

/// The place-name step: `FUN_021BC550(FUN_02188D20(fm), s16 M+0x14)`.
fn place_name<E: FieldEnv>(env: &mut E, m: u32, fm: u32) {
    let pn = env.callv(PLACE_NAME, &[fm]);
    let zone = env.r16(m + 0x14) as i16 as u32;
    env.callv(0x021B_C550, &[pn, zone]);
}

/// C: `FUN_02183B60`: call `f()` when `f` is not null; returns `!f` (the callback
/// invoker that the integrity calls of [`zone_change_proc`] receive).
pub fn call_cb<E: FieldEnv>(env: &mut E, f: u32) -> u32 {
    if f != 0 {
        env.callv(f & !1, &[]);
    }
    !f
}

/// C: `FUN_02183B6C`: the game-start event of kind `*k`: 0 [`start_new`], 1
/// [`continue_new`], 2 [`new_game_new`] (each given `(a, k)`); other kinds answer `a`.
pub fn start_event<E: MapChangeEnv>(env: &mut E, a: u32, k: u32) -> u32 {
    match env.r32(k) {
        0 => start_new(env, a, k),
        1 => continue_new(env, a, k),
        2 => new_game_new(env, a, k),
        _ => a,
    }
}

/// C: `FUN_02183B90`: step `*seq` of the start event (work `w`: `+0` gsys, `+4` x):
/// 0: the VRAM clear; 1: `FUN_0202E8CC(gd, (u16) FUN_0202E888())`, call `FUN_02187808(gsys, s, s)` (s = season); 2:
/// call `FUN_02161B90(gsys, event, 5, 0, 1)`; 3: change to [`new_game_new`]`(gsys, w+4)`. Answers 0.
pub fn start_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    let gd = env.callv(GAMEDATA, &[gsys]);
    match env.r32(seq) {
        0 => {
            vram_clear(env);
            next(env, seq);
        }
        1 => {
            let t = env.callv(0x0202_E888, &[]);
            env.callv(0x0202_E8CC, &[gd, t & 0xFFFF]);
            let s = env.callv(SEASON, &[gd]);
            let child = env.callv(0x0218_7808, &[gsys, s, s]);
            call_child(env, event, child);
            next(env, seq);
        }
        2 => {
            let child = env.callv(0x0216_1B90, &[gsys, event, 5, 0, 1]);
            call_child(env, event, child);
            next(env, seq);
        }
        3 => {
            let x = env.r32(w + 4);
            let nx = new_game_new(env, gsys, x);
            env.callv(EVENT_CHANGE, &[event, nx]);
        }
        _ => {}
    }
    0
}

/// C: `FUN_02183C34`: the start event: [`procs::START`], work 8 `{gsys, x}`.
pub fn start_new<E: FieldEnv>(env: &mut E, gsys: u32, x: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, procs::START, 8]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, gsys);
    env.w32(w + 4, x);
    ev
}

/// C: `FUN_02183C58`: reset the WFBC lists of a new game:
/// `FUN_021860D4(list, FUN_02012944(gd), 1)` ([`crate::mapchange_zone::people_generate`]), `FUN_02161D64(list)`,
/// `FUN_02161F14(FUN_020127F0(gd), list, 1)` (`list = FUN_020127C4(gd)`).
pub fn wfbc_reset<E: FieldEnv>(env: &mut E, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let list = env.callv(WFBC_LIST, &[gd]);
    let ms = env.callv(0x0201_2944, &[gd]);
    env.callv(0x0218_60D4, &[list, ms, 1]);
    let list = env.callv(WFBC_LIST, &[gd]);
    env.callv(0x0216_1D64, &[list]);
    let x = env.callv(0x0201_27F0, &[gd]);
    let list = env.callv(WFBC_LIST, &[gd]);
    env.callv(0x0216_1F14, &[x, list, 1]);
}

/// C: `FUN_02183C9C`: step `*seq` of the new-game event (work `w` 0x24: `+0`
/// gsys, `+4` gd, `+8` LOCATION): 0: `FUN_02158A10(gsys)`, zone load `FUN_02185A68(gsys, u16 w+0xC)`, zone entry
/// `FUN_021857C0(gsys, w+8)`; 1: call `FUN_020298A0(gsys, FUN_02029DF4(gd, s16 w+0xC, season), 0, 60)`; 2: call
/// `FUN_020145FC(gsys)`; 3: call the fade in `FUN_021BE92C(gsys, fm, 1, 0, 1, 0, 0)`; 4: the place name
/// (`FUN_021BC578`) when there is a place-name system, done (1). Steps 0..3 move on.
pub fn new_game_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.callv(0x0201_296C, &[gd]);
    match env.r32(seq) {
        0 => {
            env.callv(0x0215_8A10, &[gsys]);
            let zone = env.r16(w + 0xC) as u32;
            env.callv(0x0218_5A68, &[gsys, zone]);
            env.call(0x0218_57C0, &mut [CArg::V(gsys), CArg::In(w + 8, LOC)]);
        }
        1 => {
            let s = env.callv(SEASON, &[gd]);
            let zone = env.r16(w + 0xC) as i16 as u32;
            let bgm = env.callv(0x0202_9DF4, &[gd, zone, s]);
            let child = env.callv(0x0202_98A0, &[gsys, bgm, 0, 60]);
            call_child(env, event, child);
        }
        2 => {
            let child = env.callv(0x0201_45FC, &[gsys]);
            call_child(env, event, child);
        }
        3 => {
            let fm = env.callv(FIELDMAP, &[gsys]);
            let child = env.callv(FADE_IN, &[gsys, fm, 1, 0, 1, 0, 0]);
            call_child(env, event, child);
        }
        4 => {
            let fm = env.callv(FIELDMAP, &[gsys]);
            if env.callv(PLACE_NAME, &[fm]) != 0 {
                let pn = env.callv(PLACE_NAME, &[fm]);
                let zone = env.r16(w + 0xC) as i16 as u32;
                env.callv(0x021B_C578, &[pn, zone]);
            }
            return 1;
        }
        _ => return 0,
    }
    next(env, seq);
    0
}

/// C: `FUN_02183D68`: the new-game event: [`procs::NEW_GAME`], work 0x24 `{gsys, gd,
/// LOCATION cleared by FUN_02014264}`; [`wfbc_reset`]; the RTC record of save block 0x25 (`FUN_02008B20(FUN_02008B08(sv))`);
/// `FUN_02021444()`; the default record `FUN_02185F6C(gd)`; the VRAM clear. `_x` is unused.
pub fn new_game_new<E: FieldEnv>(env: &mut E, gsys: u32, _x: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, procs::NEW_GAME, 0x24]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 4, gd);
    env.call(0x0201_4264, &mut [CArg::Out(w + 8, LOC)]);
    wfbc_reset(env, gsys);
    let gd = env.r32(w + 4);
    let sv = env.callv(SAVE, &[gd]);
    let rec = env.callv(0x0200_8B08, &[sv]);
    env.callv(0x0200_8B20, &[rec]);
    env.callv(0x0202_1444, &[]);
    let gd = env.r32(w + 4);
    env.callv(0x0218_5F6C, &[gd]);
    vram_clear(env);
    ev
}

/// C: `FUN_02183DD4`: `FUN_02161D64(FUN_020127C4(gd))` (post-process WFBC list 1).
pub fn wfbc_post<E: FieldEnv>(env: &mut E, gsys: u32) {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let list = env.callv(WFBC_LIST, &[gd]);
    env.callv(0x0216_1D64, &[list]);
}

/// C: `FUN_02183DE4`: step `*seq` of the continue event (work `w` 0x10: `+0` gsys,
/// `+4` gd, `+8` u16 zone, `+0xC` flag 0x966 was set): 0: `FUN_0201445C(FUN_02012B38(gd), 1)`; with the flag the zone
/// of the GAMEDATA LOCATION `l = FUN_02012894(gd)` becomes `w+8` and the zone is entered (`FUN_02185B70`, `FUN_02185A68`,
/// `FUN_0218601C`, `FUN_021857C0(gsys, l)`, `FUN_0202735C`), else the zone is reloaded (`FUN_02185A68`, `FUN_0218601C`,
/// `FUN_02168EEC`, `FUN_02164474`, `FUN_0202C860`, the weather); 1: call the zone BGM event; 2: call `FUN_020145FC(gsys)`;
/// 3: with the flag and var 0x4041 set the script 0x83B, else call the fade in `FUN_021BE92C(gsys, fm, 3, 0, 0, s, s)`;
/// 4: the place name, done (1). Steps 0..3 move on.
pub fn continue_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let ew = env.callv(0x0201_296C, &[gd]);
    match env.r32(seq) {
        0 => {
            let o = env.callv(0x0201_2B38, &[gd]);
            env.callv(0x0201_445C, &[o, 1]);
            if env.r32(w + 0xC) != 0 {
                let l = env.callv(0x0201_2894, &[gd]);
                let zone = env.r16(l + 4);
                env.w16(w + 8, zone);
                env.callv(0x0218_5B70, &[gsys]);
                let zone = env.r16(w + 8) as u32;
                env.callv(0x0218_5A68, &[gsys, zone]);
                env.callv(0x0218_601C, &[gd]);
                env.call(0x0218_57C0, &mut [CArg::V(gsys), CArg::In(l, LOC)]);
                let zone = env.r16(l + 4) as u32;
                env.callv(0x0202_735C, &[zone, gd]);
            } else {
                let zone = env.r16(w + 8) as u32;
                env.callv(0x0218_5A68, &[gsys, zone]);
                env.callv(0x0218_601C, &[gd]);
                env.callv(0x0216_8EEC, &[gd]);
                let zone = env.r16(w + 8) as u32;
                env.callv(0x0216_4474, &[gd, zone]);
                let zone = env.r16(w + 8) as u32;
                env.callv(0x0202_C860, &[gd, zone]);
                let zone = env.r16(w + 8) as u32;
                let k = env.callv(0x0201_405C, &[zone]);
                let mm = env.callv(0x0201_2964, &[gd]);
                if (k as i32) < 16 {
                    env.callv(0x0216_E29C, &[mm, k]);
                } else {
                    env.callv(0x0216_CCCC, &[mm]);
                }
            }
        }
        1 => {
            let s = env.callv(SEASON, &[gd]);
            let zone = env.r16(w + 8) as u32;
            let bgm = env.callv(0x0202_9DF4, &[gd, zone, s]);
            let child = env.callv(0x0202_98A0, &[gsys, bgm, 0, 60]);
            call_child(env, event, child);
        }
        2 => {
            let child = env.callv(0x0201_45FC, &[gsys]);
            call_child(env, event, child);
        }
        3 => {
            let script = env.r32(w + 0xC) != 0 && {
                let v = env.callv(0x0201_43CC, &[ew, 0x4041]);
                env.r16(v) != 0
            };
            if script {
                env.callv(0x0215_8780, &[event, 0x83B, 0, 0, 21]);
            } else {
                let s = env.callv(SEASON, &[gd]);
                let fm = env.callv(FIELDMAP, &[gsys]);
                let child = env.callv(FADE_IN, &[gsys, fm, 3, 0, 0, s, s]);
                call_child(env, event, child);
            }
        }
        4 => {
            let fm = env.callv(FIELDMAP, &[gsys]);
            if env.callv(PLACE_NAME, &[fm]) != 0 {
                let pn = env.callv(PLACE_NAME, &[fm]);
                let zone = env.r16(w + 8) as u32;
                env.callv(0x021B_C578, &[pn, zone]);
            }
            return 1;
        }
        _ => return 0,
    }
    next(env, seq);
    0
}

/// C: `FUN_02183F5C`: the continue event: [`procs::CONTINUE`], work 0x10 `{gsys, gd,
/// u16 p+0x10, flag 0x966}` (flag 0x966 is cleared); when the RTC record of save block 0x25 fails `FUN_02008B4C` or
/// `FUN_02008B78` it is re-made (`FUN_02008C00(FUN_02008B14(sv))`, `FUN_0216A390(gd, a)`, `FUN_0200C384(sb, ticks)` with the
/// 64-bit `FUN_0203F524()`); `FUN_02008B20(rec)`, `FUN_0216AAE4(gd, a, season)`, [`wfbc_post`], `FUN_02021444()`, the
/// VRAM clear (`a = FUN_0201AD0C(sv)`, `sb = FUN_02012F08(gd)`).
pub fn continue_new<E: MapChangeEnv>(env: &mut E, gsys: u32, p: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, procs::CONTINUE, 0x10]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 4, gd);
    let z = env.r16(p + 0x10);
    env.w16(w + 8, z);
    let ew = env.callv(0x0201_296C, &[gd]);
    let f = env.callv(0x0201_42E8, &[ew, 0x966]);
    env.w32(w + 0xC, f);
    env.callv(0x0201_433C, &[ew, 0x966]);
    let gd = env.r32(w + 4);
    let sv = env.callv(SAVE, &[gd]);
    let rec = env.callv(0x0200_8B08, &[sv]);
    let a = env.callv(0x0201_AD0C, &[sv]);
    let gd = env.r32(w + 4);
    let sb = env.callv(0x0201_2F08, &[gd]);
    if env.callv(0x0200_8B4C, &[rec]) == 0 || env.callv(0x0200_8B78, &[rec]) == 0 {
        let t = env.callv(0x0200_8B14, &[sv]);
        env.callv(0x0200_8C00, &[t]);
        let gd = env.r32(w + 4);
        env.callv(0x0216_A390, &[gd, a]);
        let tick = env.call64(0x0203_F524);
        env.callv(0x0200_C384, &[sb, tick as u32, (tick >> 32) as u32]);
    }
    env.callv(0x0200_8B20, &[rec]);
    let gd = env.r32(w + 4);
    let s = env.callv(SEASON, &[gd]);
    let gd = env.r32(w + 4);
    env.callv(0x0216_AAE4, &[gd, a, s]);
    wfbc_post(env, gsys);
    env.callv(0x0202_1444, &[]);
    vram_clear(env);
    ev
}

/// C: `FUN_0218403C`: the season-change check of work `m`: the two seasons `a` /
/// `b` of `FUN_0202E848(gd, &a, &b)`; when they differ and the zone's record (`FUN_0201407C(FUN_02188C84(fm),
/// FUN_02013B38(u16 m+0x14), 0)`, freed with `FUN_020140B4`) answers `FUN_020140D8`: `+0x44 = 1`, `+0x48 = a`,
/// `+0x4A = b`.
pub fn season_check<E: FieldEnv>(env: &mut E, m: u32) {
    let t = env.mem().push_local(4);
    let gd = env.r32(m + 4);
    env.call(
        0x0202_E848,
        &mut [CArg::V(gd), CArg::Out(t + 2, 2), CArg::Out(t, 2)],
    );
    let fm = env.r32(m + 8);
    let x = env.callv(0x0218_8C84, &[fm]);
    let zone = env.r16(m + 0x14) as u32;
    let z = env.callv(0x0201_3B38, &[zone]);
    let rec = env.callv(0x0201_407C, &[x, z, 0]);
    let f = env.callv(0x0201_40D8, &[rec]) != 0;
    env.callv(0x0201_40B4, &[rec]);
    let a = env.r16(t + 2);
    let b = env.r16(t);
    env.mem().pop_local(t);
    if a != b && f {
        env.w32(m + 0x44, 1);
        env.w16(m + 0x48, a);
        env.w16(m + 0x4A, b);
    }
}

/// The part of [`door_sub`] / [`door_sub_exit`] they share; returns the sub-record address `m + 0x4C`.
fn door_sub_common<E: FieldEnv>(env: &mut E, m: u32, event: u32) -> u32 {
    let s = m + 0x4C;
    let v = env.r32(m);
    env.w32(s + 4, v);
    let v = env.r32(m + 4);
    env.w32(s + 8, v);
    let v = env.r32(m + 8);
    env.w32(s + 0xC, v);
    let v = env.r32(m + 0x2C);
    env.w32(m + 0x4C, event);
    env.w32(s + 0x10, v);
    let v = env.r16(m + 0xC);
    env.w16(s + 0x18, v);
    env.mem().copy(s + 0x1C, m + 0x10, LOC);
    let a = env.r16(m + 0xC) as u32;
    let b = env.r16(m + 0x14) as u32;
    let k = env.callv(0x021B_E89C, &[a, b]);
    env.w32(s + 0x38, k);
    let a = env.r16(m + 0xC) as u32;
    let b = env.r16(m + 0x14) as u32;
    let k = env.callv(0x021B_E8B8, &[a, b]);
    env.w32(s + 0x3C, k);
    let season = (env.r32(m + 0x40) != 0 && env.r32(m + 0x44) != 0) as u32;
    env.w32(s + 0x40, season);
    let a = env.r16(m + 0x48) as u32 & 0xFF;
    let c = env.callv(0x0202_E8A4, &[a]);
    env.w8(s + 0x44, c as u8);
    let b = env.r16(m + 0x4A);
    env.w8(s + 0x45, b as u8);
    s
}

/// C: `FUN_0218409C`: fill the door / transition sub-record at `m + 0x4C`: `+0` the
/// event, `+4..+0xC` M's gsys / gd / fm, `+0x10` M's door kind, `+0x18` u16 M's old zone, `+0x1C` a copy of M's LOCATION,
/// `+0x38` / `+0x3C` the fade kinds `FUN_021BE89C` / `FUN_021BE8B8` of (old zone, new zone), `+0x40` the season flag,
/// `+0x44` `FUN_0202E8A4(u8 m+0x48)`, `+0x45` u8 `m+0x4A`, `+0x48 = +0x4C = 0`.
pub fn door_sub<E: FieldEnv>(env: &mut E, m: u32, event: u32) {
    let s = door_sub_common(env, m, event);
    env.w32(s + 0x48, 0);
    env.w32(s + 0x4C, 0);
}

/// C: `FUN_02184120`: as [`door_sub`] without `+0x48` / `+0x4C`, plus `+0x14` the
/// door kind of the exit warp (`FUN_021626E8(FUN_021625CC(FUN_02012844(gd), u16 m+0x16))`, 0 for a LOCATION of kind 1).
pub fn door_sub_exit<E: FieldEnv>(env: &mut E, m: u32, event: u32) {
    let s = door_sub_common(env, m, event);
    if env.r32(m + 0x10) == 1 {
        env.w32(s + 0x14, 0);
        return;
    }
    let gd = env.r32(m + 4);
    let ed = env.callv(0x0201_2844, &[gd]);
    let warp = env.r16(m + 0x16) as u32;
    let rec = env.callv(0x0216_25CC, &[ed, warp]);
    let k = env.callv(0x0216_26E8, &[rec]);
    env.w32(s + 0x14, k);
}

/// C: `FUN_021841C0`: step `*seq` of the zone-change child event (work `wp`: `+0`
/// the MAPCHANGE work M; comm system `c = FUN_02012138(gsys)`): 0: `FUN_0201447C(FUN_02012B38(gd), 2)`, call
/// `FUN_02014554(gsys, fm)`; 1: load overlay 0xE6; next step 2 after `FUN_02026400(c)` when `FUN_02013D34(zone) == 1`
/// and `FUN_02026454(c)` is 1 or 2 (low byte), else 3; 2: wait until `FUN_02026454(c)` is 0, then step 3; 3: the zone
/// switch: integrity calls `FUN_021882A0` / `FUN_02188318` / `FUN_021883CC` with the invoker 0x02183B61 and the
/// callbacks 0x0218604D / 0x0218606D / 0x0218608D, leave the zone (`FUN_02185B70`), the season change (`FUN_0202E8CC(gd,
/// u16 M+0x4A)`, `FUN_0216AAE4(gd, party, s)`, `FUN_0216A358(gd, party, FUN_02008B14(sv) + 20, s)`), the zone load
/// `FUN_02185A68` and entry `FUN_021857C0(gsys, M+0x10)`, `FUN_0202735C(zone, gd)` unless kind 4, the zone hook of kind
/// `M+0x30` (1: `FUN_02164454`, 2: `FUN_02164464`, 3: `FUN_0216445C`), unload overlay 0xE6, `FUN_0215874C(c)`; 4: call
/// `FUN_020145FC(gsys)`; 5: call `FUN_02029C64(gsys)`; 6: done (1). Steps 0, 3, 4, 5 move on.
pub fn zone_change_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, wp: u32) -> u32 {
    let m = env.r32(wp);
    let fm = env.r32(m + 8);
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let c = env.callv(0x0201_2138, &[gsys]);
    match env.r32(seq) {
        0 => {
            let o = env.callv(0x0201_2B38, &[gd]);
            env.callv(0x0201_447C, &[o, 2]);
            let child = env.callv(0x0201_4554, &[gsys, fm]);
            call_child(env, event, child);
            next(env, seq);
        }
        1 => {
            env.callv(0x0203_4AC4, &[0xE6]);
            let zone = env.r16(m + 0x14) as u32;
            let s = if env.callv(0x0201_3D34, &[zone]) == 1
                && env.callv(0x0202_6454, &[c]).wrapping_sub(1) & 0xFF <= 1
            {
                env.callv(0x0202_6400, &[c]);
                2
            } else {
                3
            };
            env.w32(seq, s);
        }
        2 => {
            if env.callv(0x0202_6454, &[c]) == 0 {
                env.w32(seq, 3);
            }
        }
        3 => {
            env.callv(0x0218_82A0, &[CHECK_CB, 0x0218_604D, 0]);
            env.callv(0x0218_5B70, &[gsys]);
            if env.r32(m + 0x40) != 0 && env.r32(m + 0x44) != 0 {
                let b = env.r16(m + 0x4A) as u32;
                env.callv(0x0202_E8CC, &[gd, b]);
                let sv = env.callv(SAVE, &[gd]);
                let t = env.callv(0x0200_8B14, &[sv]);
                let s = env.callv(SEASON, &[gd]);
                let party = env.callv(0x0201_2934, &[gd]);
                env.callv(0x0216_AAE4, &[gd, party, s]);
                env.callv(0x0216_A358, &[gd, party, t.wrapping_add(20), s]);
            }
            env.callv(0x0218_8318, &[CHECK_CB, 0x0218_606D, 0]);
            let zone = env.r16(m + 0x14) as u32;
            env.callv(0x0218_5A68, &[gsys, zone]);
            env.call(0x0218_57C0, &mut [CArg::V(gsys), CArg::In(m + 0x10, LOC)]);
            if env.r8(m + 0x30) != 4 {
                let zone = env.r16(m + 0x14) as u32;
                env.callv(0x0202_735C, &[zone, gd]);
            }
            env.callv(0x0218_83CC, &[CHECK_CB, 0x0218_608D, 0]);
            let hook = match env.r8(m + 0x30) {
                1 => Some(0x0216_4454),
                2 => Some(0x0216_4464),
                3 => Some(0x0216_445C),
                _ => None,
            };
            if let Some(h) = hook {
                let zone = env.r16(m + 0x14) as u32;
                env.callv(h, &[gd, zone]);
            }
            env.callv(0x0203_4A5C, &[0xE6]);
            env.callv(0x0215_874C, &[c]);
            next(env, seq);
        }
        4 => {
            let child = env.callv(0x0201_45FC, &[gsys]);
            call_child(env, event, child);
            next(env, seq);
        }
        5 => {
            let child = env.callv(0x0202_9C64, &[gsys]);
            call_child(env, event, child);
            next(env, seq);
        }
        6 => return 1,
        _ => {}
    }
    0
}

/// C: `FUN_02184360`: the zone-change child event of work `m`:
/// [`procs::ZONE_CHANGE`], work 4 `{m}`; `m+0x30 = kind`.
pub fn zone_change_new<E: FieldEnv>(env: &mut E, m: u32, kind: u32) -> u32 {
    let gsys = env.r32(m);
    let ev = env.callv(EVENT_NEW, &[gsys, 0, procs::ZONE_CHANGE, 4]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, m);
    env.w8(m + 0x30, kind as u8);
    ev
}

/// C: `FUN_02184388`: step `*seq` of the door event (work `m`): 0: [`season_check`],
/// pause the MMDLs (`FUN_0216D9A0(FUN_02188C7C(fm))`), [`door_sub`], call `FUN_021BD648(m + 0x4C)`, answer 33; 1: call
/// [`zone_change_new`]`(m, 0)`; 2: [`door_sub_exit`], call `FUN_021BD9BC(m + 0x4C)`; 3: done (1). Steps 0..2 move on.
pub fn door_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let fm = env.r32(m + 8);
    match env.r32(seq) {
        0 => {
            season_check(env, m);
            let mm = env.callv(FM_MMDLSYS, &[fm]);
            env.callv(0x0216_D9A0, &[mm]);
            door_sub(env, m, event);
            let child = env.callv(0x021B_D648, &[m + 0x4C]);
            call_child(env, event, child);
            next(env, seq);
            33
        }
        1 => {
            let child = zone_change_new(env, m, 0);
            call_child(env, event, child);
            next(env, seq);
            0
        }
        2 => {
            door_sub_exit(env, m, event);
            let child = env.callv(0x021B_D9BC, &[m + 0x4C]);
            call_child(env, event, child);
            next(env, seq);
            0
        }
        3 => 1,
        _ => 0,
    }
}

/// C: `FUN_02184410`: step `*seq` of the comm door event (work `m`, comm record
/// `r = FUN_02158344(FUN_02012138(gsys))`): 0: pause the MMDLs, next step 5 when `FUN_02013DD0(zone) == 1`, else 1; 1:
/// [`door_sub`], call `FUN_021BD648(m + 0x4C)`, answer 33; 2: call [`zone_change_new`]`(m, 0)`; 3: [`door_sub_exit`], call
/// `FUN_021BD9BC(m + 0x4C)`; 4: done (1); 5: without `r` step 7, else step 6 once `FUN_021746AC(r, FUN_0215841C(gd)) ==
/// 1`; 6: without `r` step 7, else step 1 once `FUN_02158428(r) == 1`; 7: resume the MMDLs, done (1).
pub fn door_comm_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let fm = env.r32(m + 8);
    let gd = env.r32(m + 4);
    let gsys = env.r32(m);
    let c = env.callv(0x0201_2138, &[gsys]);
    let r = env.callv(0x0215_8344, &[c]);
    match env.r32(seq) {
        0 => {
            let mm = env.callv(FM_MMDLSYS, &[fm]);
            env.callv(0x0216_D9A0, &[mm]);
            let zone = env.r16(m + 0x14) as u32;
            let s = if env.callv(0x0201_3DD0, &[zone]) == 1 {
                5
            } else {
                1
            };
            env.w32(seq, s);
            0
        }
        1 => {
            door_sub(env, m, event);
            let child = env.callv(0x021B_D648, &[m + 0x4C]);
            call_child(env, event, child);
            next(env, seq);
            33
        }
        2 => {
            let child = zone_change_new(env, m, 0);
            call_child(env, event, child);
            next(env, seq);
            0
        }
        3 => {
            door_sub_exit(env, m, event);
            let child = env.callv(0x021B_D9BC, &[m + 0x4C]);
            call_child(env, event, child);
            next(env, seq);
            0
        }
        4 => 1,
        5 => {
            if r == 0 {
                env.w32(seq, 7);
            } else {
                let x = env.callv(0x0215_841C, &[gd]);
                if env.callv(0x0217_46AC, &[r, x]) == 1 {
                    next(env, seq);
                }
            }
            0
        }
        6 => {
            if r == 0 {
                env.w32(seq, 7);
            } else if env.callv(0x0215_8428, &[r]) == 1 {
                env.w32(seq, 1);
            }
            0
        }
        7 => {
            let mm = env.callv(FM_MMDLSYS, &[fm]);
            env.callv(0x0216_D9E4, &[mm]);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_02184504`: step `*seq` of the warp event (work `m`, BGM object `p =
/// FUN_02012A30(gd)`): 0: call `FUN_02161824(gsys, fm)`; 1: the zone BGM; 2: call [`zone_change_new`]`(m, u8 m+0x30)`;
/// 3: done (1). Steps 0..2 move on.
pub fn warp_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let fm = env.r32(m + 8);
    let p = env.callv(BGM_OBJ, &[gd]);
    match env.r32(seq) {
        0 => {
            let child = env.callv(LEAVE_EVENT, &[gsys, fm]);
            call_child(env, event, child);
        }
        1 => zone_bgm(env, m, p, gd),
        2 => {
            let kind = env.r8(m + 0x30) as u32;
            let child = zone_change_new(env, m, kind);
            call_child(env, event, child);
        }
        3 => return 1,
        _ => return 0,
    }
    next(env, seq);
    0
}

/// C: `FUN_02184584`: step `*seq` of the fade event (work `m`): 0: [`season_check`],
/// call `FUN_02161824(gsys, fm)`; 1: call the fade out `FUN_021BE8D4(gsys, fm, k, 0)` (`k` 0 on a season change, else
/// `FUN_021BE89C(u16 m+0xC, u16 m+0x14)`); 2: call [`zone_change_new`]`(m, u8 m+0x30)`; 3: call the fade in
/// `FUN_021BE92C` (season change: `(3, 0, 0, u8 m+0x48, u8 m+0x4A)`, else `(FUN_021BE8B8(..), 0, 1, 0, 0)`); 4: done.
pub fn fade_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let fm = env.r32(m + 8);
    let season = |env: &mut E| env.r32(m + 0x40) != 0 && env.r32(m + 0x44) != 0;
    match env.r32(seq) {
        0 => {
            season_check(env, m);
            let child = env.callv(LEAVE_EVENT, &[gsys, fm]);
            call_child(env, event, child);
        }
        1 => {
            let k = if season(env) {
                0
            } else {
                let a = env.r16(m + 0xC) as u32;
                let b = env.r16(m + 0x14) as u32;
                env.callv(0x021B_E89C, &[a, b])
            };
            let child = env.callv(FADE_OUT, &[gsys, fm, k, 0]);
            call_child(env, event, child);
        }
        2 => {
            let kind = env.r8(m + 0x30) as u32;
            let child = zone_change_new(env, m, kind);
            call_child(env, event, child);
        }
        3 => {
            let child = if season(env) {
                let a = env.r16(m + 0x48) as u32 & 0xFF;
                let b = env.r16(m + 0x4A) as u32 & 0xFF;
                env.callv(FADE_IN, &[gsys, fm, 3, 0, 0, a, b])
            } else {
                let a = env.r16(m + 0xC) as u32;
                let b = env.r16(m + 0x14) as u32;
                let k = env.callv(0x021B_E8B8, &[a, b]);
                env.callv(FADE_IN, &[gsys, fm, k, 0, 1, 0, 0])
            };
            call_child(env, event, child);
        }
        4 => return 1,
        _ => return 0,
    }
    next(env, seq);
    0
}

/// C: `FUN_0218466C`: step `*seq` of the wait event (work `m`): 0: call
/// `FUN_02161824(gsys, fm)`; 1..7: wait; 8: call the fade out `FUN_021BE8D4(gsys, fm, 2, 0)`; 9: the zone BGM; 10: call
/// [`zone_change_new`]`(m, u8 m+0x30)`; 11: call the fade in `FUN_021BE92C(gsys, fm, 2, 0, 0, 0, 0)`; 12: the place name
/// (`FUN_021BC550`), done (1). Every other step moves on.
pub fn wait_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let fm = env.r32(m + 8);
    let p = env.callv(BGM_OBJ, &[gd]);
    match env.r32(seq) {
        0 => {
            let child = env.callv(LEAVE_EVENT, &[gsys, fm]);
            call_child(env, event, child);
        }
        8 => {
            let child = env.callv(FADE_OUT, &[gsys, fm, 2, 0]);
            call_child(env, event, child);
        }
        9 => zone_bgm(env, m, p, gd),
        10 => {
            let kind = env.r8(m + 0x30) as u32;
            let child = zone_change_new(env, m, kind);
            call_child(env, event, child);
        }
        11 => {
            let child = env.callv(FADE_IN, &[gsys, fm, 2, 0, 0, 0, 0]);
            call_child(env, event, child);
        }
        12 => {
            place_name(env, m, fm);
            return 1;
        }
        _ => {}
    }
    next(env, seq);
    0
}

/// C: `FUN_02184730`: step `*seq` of the position event (work `m`): 0: call
/// `FUN_02161824(gsys, fm)`; 1: call `FUN_021BFD34(event, gsys, fm, m + 0x34)`; 2: the zone BGM; 3: call
/// [`zone_change_new`]`(m, 0)`; 4: call `FUN_021BF58C(event, gsys, fm)`; 5: done (1). Steps 0..4 move on.
pub fn pos_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let fm = env.r32(m + 8);
    let p = env.callv(BGM_OBJ, &[gd]);
    let child = match env.r32(seq) {
        0 => env.callv(LEAVE_EVENT, &[gsys, fm]),
        1 => env.callv(0x021B_FD34, &[event, gsys, fm, m + 0x34]),
        2 => {
            zone_bgm(env, m, p, gd);
            next(env, seq);
            return 0;
        }
        3 => zone_change_new(env, m, 0),
        4 => env.callv(0x021B_F58C, &[event, gsys, fm]),
        5 => return 1,
        _ => return 0,
    };
    call_child(env, event, child);
    next(env, seq);
    0
}

/// The procs of `FUN_021847CC` / `FUN_02184894` (only the two transition events differ).
fn special_proc<E: FieldEnv>(
    env: &mut E,
    event: u32,
    seq: u32,
    m: u32,
    enter: u32,
    leave: u32,
) -> u32 {
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let fm = env.r32(m + 8);
    let p = env.callv(BGM_OBJ, &[gd]);
    let child = match env.r32(seq) {
        0 => {
            season_check(env, m);
            env.callv(LEAVE_EVENT, &[gsys, fm])
        }
        1 => {
            let x = env.r32(m + 0x44);
            env.callv(enter, &[event, gsys, fm, x])
        }
        2 => {
            env.callv(0x0218_5F48, &[gd]);
            zone_bgm(env, m, p, gd);
            next(env, seq);
            return 0;
        }
        3 => zone_change_new(env, m, 2),
        4 => {
            let x = env.r32(m + 0x44);
            let a = env.r16(m + 0x48) as u32;
            let b = env.r16(m + 0x4A) as u32;
            env.callv(leave, &[event, gsys, fm, x, a, b])
        }
        5 => {
            place_name(env, m, fm);
            return 1;
        }
        _ => return 0,
    };
    call_child(env, event, child);
    next(env, seq);
    0
}

/// C: `FUN_021847CC`: step `*seq` of a special-location event (work `m`): 0:
/// [`season_check`], call `FUN_02161824(gsys, fm)`; 1: call `FUN_021BFD70(event, gsys, fm, m+0x44)`; 2: the player form
/// (`FUN_02185F48(gd)`), the zone BGM; 3: call [`zone_change_new`]`(m, 2)`; 4: call `FUN_021BF5B4(event, gsys, fm,
/// m+0x44, u16 m+0x48, u16 m+0x4A)`; 5: the place name, done (1). Steps 0..4 move on.
pub fn special_a_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    special_proc(env, event, seq, m, 0x021B_FD70, 0x021B_F5B4)
}

/// C: `FUN_02184894`: as [`special_a_proc`] with `FUN_021BFDA0` / `FUN_021BF5E8`.
pub fn special_b_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    special_proc(env, event, seq, m, 0x021B_FDA0, 0x021B_F5E8)
}

/// C: `FUN_0218495C`: step `*seq` of a zone event (work `m`): 0: call
/// `FUN_02161824(gsys, fm)`; 1: call `FUN_021BFDD0(event, gsys, fm, 1)`; 2: the player walks (`FUN_02012B2C(player, 0)`),
/// the zone BGM; 3: call [`zone_change_new`]`(m, 3)`; 4: call `FUN_021BF61C(event, gsys, fm)`; 5: the place name, done.
pub fn zone_c_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let fm = env.r32(m + 8);
    let p = env.callv(BGM_OBJ, &[gd]);
    let child = match env.r32(seq) {
        0 => env.callv(LEAVE_EVENT, &[gsys, fm]),
        1 => env.callv(0x021B_FDD0, &[event, gsys, fm, 1]),
        2 => {
            let pl = env.callv(0x0201_27A4, &[gd]);
            env.callv(0x0201_2B2C, &[pl, 0]);
            zone_bgm(env, m, p, gd);
            next(env, seq);
            return 0;
        }
        3 => zone_change_new(env, m, 3),
        4 => env.callv(0x021B_F61C, &[event, gsys, fm]),
        5 => {
            place_name(env, m, fm);
            return 1;
        }
        _ => return 0,
    };
    call_child(env, event, child);
    next(env, seq);
    0
}

/// C: `FUN_02184A14`: step `*seq` of a fade event (work `m`, `x =
/// FUN_02188D38(fm)`): 0: call the fade out `FUN_021BE8D4(gsys, fm, 1, 0)`; 1: wait for `FUN_021C6E48(x)`, then call
/// `FUN_02161824(gsys, fm)`; 2: call [`zone_change_new`]`(m, 0)`; 3: call the fade in `FUN_021BE92C(gsys, fm, 1, 0, 1, 0,
/// 0)`; 4: the place name, done (1).
pub fn special_d_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let fm = env.r32(m + 8);
    let gsys = env.r32(m);
    let x = env.callv(0x0218_8D38, &[fm]);
    let child = match env.r32(seq) {
        0 => env.callv(FADE_OUT, &[gsys, fm, 1, 0]),
        1 => {
            if env.callv(0x021C_6E48, &[x]) == 0 {
                return 0;
            }
            env.callv(LEAVE_EVENT, &[gsys, fm])
        }
        2 => zone_change_new(env, m, 0),
        3 => env.callv(FADE_IN, &[gsys, fm, 1, 0, 1, 0, 0]),
        4 => {
            place_name(env, m, fm);
            return 1;
        }
        _ => return 0,
    };
    call_child(env, event, child);
    next(env, seq);
    0
}

/// C: `FUN_02184AB4`: step `*seq` of a zone event (work `m`): 0: call
/// `FUN_02161824(gsys, fm)`; 1: call the fade out `FUN_021BE8D4(gsys, fm, 0, 0)`; 2: call a warp event ([`procs::WARP`])
/// whose work is a copy of `m`; 3: call the fade in `FUN_021BE92C(gsys, fm, 0, 0, 1, 0, 0)`; 4: the place name, done.
pub fn zone_e_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let fm = env.r32(m + 8);
    let child = match env.r32(seq) {
        0 => env.callv(LEAVE_EVENT, &[gsys, fm]),
        1 => env.callv(FADE_OUT, &[gsys, fm, 0, 0]),
        2 => {
            let ev = env.callv(EVENT_NEW, &[gsys, 0, procs::WARP, WORK_SIZE]);
            let w = env.callv(EVENT_WORK, &[ev]);
            env.mem().copy(w, m, WORK_SIZE);
            ev
        }
        3 => env.callv(FADE_IN, &[gsys, fm, 0, 0, 1, 0, 0]),
        4 => {
            place_name(env, m, fm);
            return 1;
        }
        _ => return 0,
    };
    call_child(env, event, child);
    next(env, seq);
    0
}

/// C: `FUN_02184B54`: step `*seq` of a grid event (work `m`): 0: call
/// `FUN_02161824(gsys, fm)`; 1: call `FUN_021BFE00(event, gsys, fm)`; 2: the zone BGM, call [`zone_change_new`]`(m, 0)`;
/// 3: call `FUN_021BF644(event, gsys, fm)`; 4: the place name, done (1).
pub fn grid_f_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let gd = env.r32(m + 4);
    let fm = env.r32(m + 8);
    let p = env.callv(BGM_OBJ, &[gd]);
    let child = match env.r32(seq) {
        0 => env.callv(LEAVE_EVENT, &[gsys, fm]),
        1 => env.callv(0x021B_FE00, &[event, gsys, fm]),
        2 => {
            zone_bgm(env, m, p, gd);
            zone_change_new(env, m, 0)
        }
        3 => env.callv(0x021B_F644, &[event, gsys, fm]),
        4 => {
            place_name(env, m, fm);
            return 1;
        }
        _ => return 0,
    };
    call_child(env, event, child);
    next(env, seq);
    0
}

/// C: `FUN_02184BF8`: step `*seq` of the event that enters the union room (work
/// `m`): 0: call `FUN_021BFDD0(event, gsys, fm, 0)`; 1: call [`zone_change_new`]`(m, 0)`; 2: unload overlay 0xC, load
/// 0xB, the script 0x83A (`FUN_02158780(event, 0x83A, 0, 0, 21)`); 3: done (1). Steps 0..2 move on.
pub fn union_in_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let fm = env.r32(m + 8);
    match env.r32(seq) {
        0 => {
            let child = env.callv(0x021B_FDD0, &[event, gsys, fm, 0]);
            call_child(env, event, child);
        }
        1 => {
            let child = zone_change_new(env, m, 0);
            call_child(env, event, child);
        }
        2 => {
            env.callv(0x0203_4A5C, &[0xC]);
            env.callv(0x0203_4AC4, &[0xB]);
            env.callv(0x0215_8780, &[event, 0x83A, 0, 0, 21]);
        }
        3 => return 1,
        _ => return 0,
    }
    next(env, seq);
    0
}

/// C: `FUN_02184C78`: step `*seq` of the event that leaves the union room (work
/// `m`): 0: call `FUN_02161824(gsys, fm)`; 1: unload overlay 0xB, load 0xC, call [`zone_change_new`]`(m, 4)`; 2:
/// `FUN_02173744(gsys)` (overlay 0xC code); 3: call `FUN_021BF66C(event, gsys, fm)`; 4: done (1). Steps 0..3 move on.
pub fn union_out_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, m: u32) -> u32 {
    let gsys = env.r32(m);
    let fm = env.r32(m + 8);
    match env.r32(seq) {
        0 => {
            let child = env.callv(LEAVE_EVENT, &[gsys, fm]);
            call_child(env, event, child);
        }
        1 => {
            env.callv(0x0203_4A5C, &[0xB]);
            env.callv(0x0203_4AC4, &[0xC]);
            let child = zone_change_new(env, m, 4);
            call_child(env, event, child);
        }
        2 => {
            env.callv(0x0217_3744, &[gsys]);
        }
        3 => {
            let child = env.callv(0x021B_F66C, &[event, gsys, fm]);
            call_child(env, event, child);
        }
        4 => return 1,
        _ => return 0,
    }
    next(env, seq);
    0
}

#[cfg(test)]
mod tests;
