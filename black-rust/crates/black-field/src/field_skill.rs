//! The field-skill (HM / field move) table code of overlay 10 (`0x0215E840..0x0215F168`, the second part of F1 row
//! 10.19 "move_pokemon.c"; F5b; the file has no `__FILE__` string, the name is by role, UNVERIFIED): which field moves
//! the player may use where they stand, and the GMEVENTs that run them.
//!
//! Spec: `tools/rom-diff/specs/black-field/field_skill.spec` (its header has the SKILL CTX layout and the tables). Every
//! callee outside this area is reached through [`FieldEnv::call`] by address; callees inside it (also the table entries
//! that [`can_use`] / [`use_skill`] call) are Rust calls.
//!
//! SKILL CTX (0x14, [`ctx_init`]): `+0 u16` zone, `+2 u16` usable flags (bit `k` = skill `k`), `+4` player form, `+8`
//! GAMESYS, `+0xC` the OBJ in front of the player (0 = none), `+0x10` FIELDMAP. Skill `k` (0..10) has a check fn (0 =
//! usable) and an event fn (`(param, ctx)` -> GMEVENT; `param` = `{u16 a, u16 b, u32 c}`, [`set_param`]) in
//! [`SKILL_TABLE`], or [`SKILL_TABLE_B`] in zones where `FUN_02013FFC(zone) != 0`.

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_E840,
    0x0215_EA14,
    0x0215_EA30,
    0x0215_EA38,
    0x0215_EA58,
    0x0215_EA6C,
    0x0215_EA94,
    0x0215_EAC8,
    0x0215_EAF0,
    0x0215_EB18,
    0x0215_EB4C,
    0x0215_EB60,
    0x0215_EB88,
    0x0215_EBBC,
    0x0215_EBD0,
    0x0215_EBF8,
    0x0215_EC2C,
    0x0215_EC40,
    0x0215_EC54,
    0x0215_EC6C,
    0x0215_EC80,
    0x0215_EC94,
    0x0215_ECBC,
    0x0215_ED8C,
    0x0215_EDA0,
    0x0215_EDD0,
    0x0215_EE24,
    0x0215_EE38,
    0x0215_EE68,
    0x0215_EEBC,
    0x0215_EED0,
    0x0215_EEF8,
    0x0215_EF2C,
    0x0215_EF40,
    0x0215_EF60,
    0x0215_EFCC,
    0x0215_EFF4,
    0x0215_F060,
    0x0215_F088,
    0x0215_F0B8,
    0x0215_F0E8,
    0x0215_F104,
    0x0215_F118,
];

/// Number of field skills (entries of [`SKILL_TABLE`]).
pub const SKILL_COUNT: u32 = 11;

/// ROM table `0x021711FC`: `(event fn, check fn)` of each skill (Thumb addresses), normal zones.
pub const SKILL_TABLE: [(u32, u32); 11] = [
    (0x0215_EA6D, 0x0215_EA59),
    (0x0215_EAF1, 0x0215_EAC9),
    (0x0215_EB61, 0x0215_EB4D),
    (0x0215_EBD1, 0x0215_EBBD),
    (0x0215_EC55, 0x0215_EC6D),
    (0x0215_EC95, 0x0215_EC81),
    (0x0215_EE39, 0x0215_EE25),
    (0x0215_EDA1, 0x0215_ED8D),
    (0x0215_EC41, 0x0215_EC2D),
    (0x0215_EF41, 0x0215_EF2D),
    (0x0215_EED1, 0x0215_EEBD),
];

/// ROM table `0x02171254`: [`SKILL_TABLE`] for zones with `FUN_02013FFC(zone) != 0` (skills 3 and 5 differ).
pub const SKILL_TABLE_B: [(u32, u32); 11] = [
    (0x0215_EA6D, 0x0215_EA59),
    (0x0215_EAF1, 0x0215_EAC9),
    (0x0215_EB61, 0x0215_EB4D),
    (0x0215_EFCD, 0x0215_EBBD),
    (0x0215_EC55, 0x0215_EC6D),
    (0x0215_F061, 0x0215_EC81),
    (0x0215_EE39, 0x0215_EE25),
    (0x0215_EDA1, 0x0215_ED8D),
    (0x0215_EC41, 0x0215_EC2D),
    (0x0215_EF41, 0x0215_EF2D),
    (0x0215_EED1, 0x0215_EEBD),
];

/// The two rects [`ctx_init`] tests the player against (ROM `0x021711EC` / `0x021711F4`: `u16 x, z, w, h`) and their
/// event flags.
pub const RECTS: [(u32, [u16; 4], u32); 2] = [
    (0x0217_11EC, [14, 26, 1, 2], 0x113),
    (0x0217_11F4, [40, 26, 1, 2], 0x112),
];

/// The magic word [`work_init`] puts at `+0` of a skill event work.
pub const WORK_MAGIC: u32 = 0x1974_0205;

/// `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// `FUN_0201210C(gsys)`: the FIELDMAP of the game system.
const FIELDMAP: u32 = 0x0201_210C;
/// `FUN_0201296C(gd)`: the EVENT WORK of the GAMEDATA.
const EVENT_WORK: u32 = 0x0201_296C;
/// `FUN_02012B38(gd)`: the object at GAMEDATA `+0x1B8`.
const GD_B38: u32 = 0x0201_2B38;
/// `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK_OF: u32 = 0x0201_24E8;
/// `FUN_0201235C(event, next)`: change the GMEVENT to `next`.
const EVENT_CHANGE: u32 = 0x0201_235C;
/// `FUN_02013FFC(zone)`: the zone uses [`SKILL_TABLE_B`].
const ZONE_TABLE_B: u32 = 0x0201_3FFC;
/// `FUN_02165534(n)`: game-beacon notification (X1).
const BEACON: u32 = 0x0216_5534;
/// `FUN_021587B0(ev, script, 0, 0)`: start a script event.
const SCRIPT_START: u32 = 0x0215_87B0;
/// `FUN_02158780(ev, script, 0, 0, x)`: start a script event with a parameter.
const SCRIPT_START_X: u32 = 0x0215_8780;
/// `FUN_02159058(s, id, 0, 0, 0)`: set a script parameter.
const SCRIPT_PARAM: u32 = 0x0215_9058;
/// `FUN_02188C84(fm)`: a FIELDMAP field (u16).
const FM_C84: u32 = 0x0218_8C84;

/// C: `FUN_0215E840`: fill the SKILL CTX `ctx` for the player on FIELDMAP `fm` (the flags are in the spec header):
/// cleared to 0x14 zero bytes, then zone, GAMESYS, FIELDMAP, form and front OBJ; the flag bits are set one by one in
/// the original order (read-modify-write of `ctx+2`), all cleared when the zone allows no field moves
/// (`FUN_02013E04(zone) == 0`) or `FUN_02012B70(gd) == 1`.
pub fn ctx_init<E: FieldEnv>(env: &mut E, fm: u32, ctx: u32) {
    let or = |env: &mut E, v: u16| {
        let f = env.r16(ctx + 2);
        env.w16(ctx + 2, f | v);
    };
    env.mem().fill(ctx, 0x14, 0);
    let zone = env.callv(0x0218_8C9C, &[fm]);
    env.w16(ctx, zone as u16);
    let gsys = env.callv(0x0218_8C80, &[fm]);
    env.w32(ctx + 8, gsys);
    env.w32(ctx + 0x10, fm);
    let player = env.callv(0x0218_8CB4, &[fm]);
    let form = env.callv(0x021A_3098, &[player]);
    env.w32(ctx + 4, form);
    let m = env.callv(0x021A_3250, &[player]);
    env.w32(ctx + 0xC, m);
    if m != 0 {
        let code = env.callv(0x0216_D598, &[m]);
        let sys = env.callv(0x0218_8C7C, &[fm]);
        let pos = env.mem().push_local(12);
        env.call(0x0216_D88C, &mut [CArg::V(m), CArg::Out(pos, 12)]);
        if env.callv(0x0216_E5FC, &[code]) != 0
            && env.call(0x0216_E5B0, &mut [CArg::V(sys), CArg::In(pos, 12)]) == 0
        {
            or(env, 8);
        }
        env.mem().pop_local(pos);
        if code == 0x6C {
            or(env, 1);
        }
    }
    let dir = env.callv(0x021A_3050, &[player]);
    let a = env.callv(0x021A_30C8, &[player]);
    let b = env.callv(0x021A_3128, &[player, dir]);
    if env.callv(0x021A_3504, &[player, a, b]) == 1 {
        or(env, 2);
    }
    if env.callv(0x021A_3524, &[player, a, b]) == 1 {
        or(env, 4);
    }
    let zone = env.r16(ctx) as u32;
    if env.callv(0x0201_3C40, &[zone]) == 1 {
        or(env, 0x10);
        or(env, 0x40);
    }
    let zone = env.r16(ctx) as u32;
    if env.callv(0x0201_3C50, &[zone]) == 1 {
        or(env, 0x80);
    }
    let gsys = env.r32(ctx + 8);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let e = env.callv(GD_B38, &[gd]);
    if env.callv(0x0201_4470, &[e]) & 1 != 0 {
        or(env, 0x20);
    }
    let tmp = env.mem().push_local(2);
    if env.call(0x021C_D440, &mut [CArg::V(fm), CArg::Out(tmp, 2)]) == 1 {
        or(env, 0x400);
    }
    env.mem().pop_local(tmp);
    let zone = env.r16(ctx) as u32;
    if env.callv(ZONE_TABLE_B, &[zone]) != 0 {
        let fm2 = env.r32(ctx + 0x10);
        let p = env.callv(0x0218_8CB4, &[fm2]);
        let gx = env.mem().push_local(2);
        let gy = env.mem().push_local(2);
        let gz = env.mem().push_local(2);
        env.call(
            0x021A_35FC,
            &mut [
                CArg::V(p),
                CArg::Out(gx, 2),
                CArg::Out(gy, 2),
                CArg::Out(gz, 2),
            ],
        );
        let x = env.r16(gx) as u32;
        let z = env.r16(gz) as u32;
        env.mem().pop_local(gz);
        env.mem().pop_local(gy);
        env.mem().pop_local(gx);
        let zone = env.r16(ctx) as u32;
        if env.callv(0x0201_400C, &[zone]) != 0 {
            let (r, _, f) = RECTS[0];
            if in_rect(env, r, x, z, f, fm) != 0 {
                or(env, 0x20);
            }
        }
        let zone = env.r16(ctx) as u32;
        if env.callv(0x0201_4018, &[zone]) != 0 {
            let (r, _, f) = RECTS[1];
            if in_rect(env, r, x, z, f, fm) != 0 {
                or(env, 8);
            }
        }
    }
    let zone = env.r16(ctx) as u32;
    if env.callv(0x0201_3E04, &[zone]) == 0 {
        env.w16(ctx + 2, 0);
    } else {
        or(env, 0x100);
        or(env, 0x200);
    }
    let gsys = env.r32(ctx + 8);
    let gd = env.callv(GAMEDATA, &[gsys]);
    if env.callv(0x0201_2B70, &[gd]) == 1 {
        env.w16(ctx + 2, 0);
    }
}

/// Runs the check fn at Thumb address `f` (an entry of [`SKILL_TABLE`] / [`SKILL_TABLE_B`]) on `ctx`.
fn run_check<E: FieldEnv>(env: &mut E, f: u32, ctx: u32) -> u32 {
    match f & !1 {
        0x0215_EA58 => check_bit(env, ctx, 0),
        0x0215_EAC8 => check_skill1(env, ctx),
        0x0215_EB4C => check_bit(env, ctx, 2),
        0x0215_EBBC => check_bit(env, ctx, 3),
        0x0215_EC6C => check_bit(env, ctx, 4),
        0x0215_EC80 => check_bit(env, ctx, 5),
        0x0215_EE24 => check_bit(env, ctx, 6),
        0x0215_ED8C => check_bit(env, ctx, 7),
        0x0215_EC2C => check_bit(env, ctx, 8),
        0x0215_EF2C => check_bit(env, ctx, 9),
        0x0215_EEBC => check_bit(env, ctx, 10),
        a => env.callv(a, &[ctx]),
    }
}

/// Runs the event fn at Thumb address `f` (an entry of [`SKILL_TABLE`] / [`SKILL_TABLE_B`]) on `(param, ctx)`.
fn run_event<E: FieldEnv>(env: &mut E, f: u32, param: u32, ctx: u32) -> u32 {
    match f & !1 {
        0x0215_EA6C => script_event(env, param, ctx, 0x0215_EA95),
        0x0215_EAF0 => script_event(env, param, ctx, 0x0215_EB19),
        0x0215_EB60 => script_event(env, param, ctx, 0x0215_EB89),
        0x0215_EBD0 => script_event(env, param, ctx, 0x0215_EBF9),
        0x0215_EC94 => script_event(env, param, ctx, 0x0215_ECBD),
        0x0215_EED0 => script_event(env, param, ctx, 0x0215_EEF9),
        0x0215_EFCC => script_event(env, param, ctx, 0x0215_EF61),
        0x0215_F060 => script_event(env, param, ctx, 0x0215_EFF5),
        0x0215_EDA0 => beacon_script_event(env, param, ctx, 0x5B, 0x0215_EDD1),
        0x0215_EE38 => beacon_script_event(env, param, ctx, 100, 0x0215_EE69),
        0x0215_EC40 => event_c6078(env, param, ctx),
        0x0215_EC54 => event_c8268(env, param, ctx),
        0x0215_EF40 => event_d1c24(env, param, ctx),
        a => env.callv(a, &[param, ctx]),
    }
}

/// C: `FUN_0215EA14`: whether skill `k` may be used: the check fn of [`check_fn`] on `ctx` (0 = usable), 1 when the
/// skill has none (`k >= 11`).
pub fn can_use<E: FieldEnv>(env: &mut E, k: u32, ctx: u32) -> u32 {
    match check_fn(env, ctx, k) {
        0 => 1,
        f => run_check(env, f, ctx),
    }
}

/// C: `FUN_0215EA30`: fill a skill parameter: `p+0 = (u16) a`, `p+2 = (u16) b`, `p+4 = c`.
pub fn set_param<E: FieldEnv>(env: &mut E, p: u32, a: u32, b: u32, c: u32) {
    env.w16(p, a as u16);
    env.w16(p + 2, b as u16);
    env.w32(p + 4, c);
}

/// C: `FUN_0215EA38`: start skill `k`: the event fn of [`event_fn`] on `(param, ctx)` (its GMEVENT), 0 when the
/// skill has none.
pub fn use_skill<E: FieldEnv>(env: &mut E, k: u32, param: u32, ctx: u32) -> u32 {
    match event_fn(env, ctx, k) {
        0 => 0,
        f => run_event(env, f, param, ctx),
    }
}

/// C: `FUN_0215EA58`, `FUN_0215EB4C`, `FUN_0215EBBC`, `FUN_0215EC2C`, `FUN_0215EC6C`, `FUN_0215EC80`, `FUN_0215ED8C`,
/// `FUN_0215EE24`, `FUN_0215EEBC`, `FUN_0215EF2C`: the check fns of skills 0, 2, 3, 8, 4, 5, 7, 6, 10, 9: 0 when flag
/// `bit` (= the skill) of the ctx is set, else 1.
pub fn check_bit<E: FieldEnv>(env: &mut E, ctx: u32, bit: u32) -> u32 {
    (flag(env, ctx, bit) == 0) as u32
}

/// C: `FUN_0215EAC8`: the check fn of skill 1: player form (`ctx+4`) 2 -> 4, 3 -> 1, else as [`check_bit`] 1.
pub fn check_skill1<E: FieldEnv>(env: &mut E, ctx: u32) -> u32 {
    match env.r32(ctx + 4) {
        2 => 4,
        3 => 1,
        _ => check_bit(env, ctx, 1),
    }
}

/// C: `FUN_0215EA6C`, `FUN_0215EAF0`, `FUN_0215EB60`, `FUN_0215EBD0`, `FUN_0215EC94`, `FUN_0215EED0`, `FUN_0215EFCC`,
/// `FUN_0215F060`: the script events of skills 0, 1, 2, 3, 5, 10, 3 (table B), 5 (table B): a GMEVENT of `proc`
/// (`0x0215EA95`, `0x0215EB19`, `0x0215EB89`, `0x0215EBF9`, `0x0215ECBD`, `0x0215EEF9`, `0x0215EF61`, `0x0215EFF5`) with a
/// 0x14-byte work ([`work_init`]).
pub fn script_event<E: FieldEnv>(env: &mut E, param: u32, ctx: u32, proc_: u32) -> u32 {
    let gsys = env.r32(ctx + 8);
    let ev = env.callv(EVENT_NEW, &[gsys, 0, proc_, 0x14]);
    let w = env.callv(EVENT_WORK_OF, &[ev]);
    work_init(env, w, param, ctx);
    ev
}

/// C: `FUN_0215EDA0`, `FUN_0215EE38`: the script events of skills 7 and 6: game-beacon notification `n` (0x5B / 100),
/// then [`script_event`] with proc `0x0215EDD1` / `0x0215EE69`.
pub fn beacon_script_event<E: FieldEnv>(
    env: &mut E,
    param: u32,
    ctx: u32,
    n: u32,
    proc_: u32,
) -> u32 {
    env.callv(BEACON, &[n]);
    script_event(env, param, ctx, proc_)
}

/// C: `FUN_0215EC40`: the event of skill 8: `FUN_021C6078(gsys, fm, (u8) param.a)` (a tail call).
pub fn event_c6078<E: FieldEnv>(env: &mut E, param: u32, ctx: u32) -> u32 {
    let a = env.r16(param) as u32 & 0xFF;
    let gsys = env.r32(ctx + 8);
    let fm = env.r32(ctx + 0x10);
    env.callv(0x021C_6078, &[gsys, fm, a])
}

/// C: `FUN_0215EC54`: the event of skill 4: game-beacon notification 19, then `FUN_021C8268(gsys, fm, param.c)`.
pub fn event_c8268<E: FieldEnv>(env: &mut E, param: u32, ctx: u32) -> u32 {
    env.callv(BEACON, &[19]);
    let gsys = env.r32(ctx + 8);
    let fm = env.r32(ctx + 0x10);
    let c = env.r32(param + 4);
    env.callv(0x021C_8268, &[gsys, fm, c])
}

/// C: `FUN_0215EF40`: the event of skill 9: game-beacon notification 0x1C0, then `FUN_021D1C24(gsys, fm, (u8)
/// param.a)`.
pub fn event_d1c24<E: FieldEnv>(env: &mut E, param: u32, ctx: u32) -> u32 {
    env.callv(BEACON, &[0x1C0]);
    let a = env.r16(param) as u32 & 0xFF;
    let gsys = env.r32(ctx + 8);
    let fm = env.r32(ctx + 0x10);
    env.callv(0x021D_1C24, &[gsys, fm, a])
}

/// The script start shared by the procs: `s = start(ev, script, 0, 0[, x])`, `FUN_02159058(s, (u16) w+0xC, 0, 0, 0)`.
fn start_script<E: FieldEnv>(env: &mut E, ev: u32, w: u32, script: u32, x: Option<u32>) {
    let id = env.r16(w + 0xC) as u32;
    let s = match x {
        None => env.callv(SCRIPT_START, &[ev, script, 0, 0]),
        Some(x) => env.callv(SCRIPT_START_X, &[ev, script, 0, 0, x]),
    };
    env.callv(SCRIPT_PARAM, &[s, id, 0, 0, 0]);
}

/// C: `FUN_0215EA94`, `FUN_0215EB18`, `FUN_0215EB88`, `FUN_0215EBF8`, `FUN_0215EEF8`: the GMEVENT procs of skills 0,
/// 1, 2, 3, 10: start `script` (0x2715, 0x2713, 0x2717, 0x2711, 0x271C) with the work's `+0xC` id; returns 0 (the
/// event becomes the script). `FUN_0201296C(gd)` is called first; its result is unused.
pub fn script_proc<E: FieldEnv>(env: &mut E, ev: u32, _seq: u32, w: u32, script: u32) -> u32 {
    let gsys = env.r32(w + 8);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.callv(EVENT_WORK, &[gd]);
    start_script(env, ev, w, script, None);
    0
}

/// C: `FUN_0215ECBC`: the GMEVENT proc of skill 5 (normal zones): step 0 starts script 0x2718 (parameter
/// `FUN_02188C84(fm)`); step 1 turns the light on once: when `FUN_02014468(E)` (E = the GAMEDATA `+0x1B8` object) is
/// still 0, `FUN_02014464(E, 1)`, beacon 0x94, `FUN_021C7E50(x, 1)` (x = `FUN_021C7FBC(FUN_02189A10(fm))`) and flag bit 0
/// -> bit 1 (`FUN_0201446C`), otherwise done; step 2 waits for `FUN_021C7E54(x) == 3`. Returns 1 when done.
pub fn flash_proc<E: FieldEnv>(env: &mut E, ev: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w + 8);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.callv(EVENT_WORK, &[gd]);
    let e = env.callv(GD_B38, &[gd]);
    let gsys = env.r32(w + 8);
    let fm = env.callv(FIELDMAP, &[gsys]);
    let y = env.callv(0x0218_9A10, &[fm]);
    let x = env.callv(0x021C_7FBC, &[y]);
    let gsys = env.r32(w + 8);
    let fm = env.callv(FIELDMAP, &[gsys]);
    let p = env.callv(FM_C84, &[fm]);
    match env.r32(seq) {
        0 => {
            start_script(env, ev, w, 0x2718, Some(p));
        }
        1 => {
            if env.callv(0x0201_4468, &[e]) != 0 {
                return 1;
            }
            let e2 = env.callv(GD_B38, &[gd]);
            env.callv(0x0201_4464, &[e, 1]);
            env.callv(BEACON, &[0x94]);
            env.callv(0x021C_7E50, &[x, 1]);
            let f = env.callv(0x0201_4470, &[e2]);
            env.callv(0x0201_446C, &[e2, (f & !1) | 2]);
        }
        2 => {
            if env.callv(0x021C_7E54, &[x]) == 3 {
                return 1;
            }
            return 0;
        }
        _ => return 0,
    }
    let s = env.r32(seq).wrapping_add(1);
    env.w32(seq, s);
    0
}

/// C: `FUN_0215EDD0`, `FUN_0215EE68`: the GMEVENT procs of skills 7 and 6: step 0 starts `script` (0x271A / 0x2719,
/// parameter 21); step 1 changes the event to `next(gsys)` (`FUN_02184EEC` / `FUN_02184F40`). Returns 0.
pub fn change_proc<E: FieldEnv>(
    env: &mut E,
    ev: u32,
    seq: u32,
    w: u32,
    script: u32,
    next: u32,
) -> u32 {
    match env.r32(seq) {
        0 => {
            start_script(env, ev, w, script, Some(21));
            let s = env.r32(seq).wrapping_add(1);
            env.w32(seq, s);
        }
        1 => {
            let gsys = env.r32(w + 8);
            let n = env.callv(next, &[gsys]);
            env.callv(EVENT_CHANGE, &[ev, n]);
        }
        _ => {}
    }
    0
}

/// C: `FUN_0215EF60`, `FUN_0215EFF4`: the GMEVENT procs of skills 3 and 5 in table-B zones: step 0 starts `s0`
/// (0x271E / 0x271D, parameter `FUN_02188C84(fm)`); step 1 starts `s1` (0x28E3 / 0x28E2). Returns 0.
pub fn two_script_proc<E: FieldEnv>(
    env: &mut E,
    ev: u32,
    seq: u32,
    w: u32,
    s0: u32,
    s1: u32,
) -> u32 {
    let gsys = env.r32(w + 8);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.callv(EVENT_WORK, &[gd]);
    let gsys = env.r32(w + 8);
    let fm = env.callv(FIELDMAP, &[gsys]);
    let p = env.callv(FM_C84, &[fm]);
    match env.r32(seq) {
        0 => {
            start_script(env, ev, w, s0, Some(p));
            let s = env.r32(seq).wrapping_add(1);
            env.w32(seq, s);
        }
        1 => {
            env.callv(SCRIPT_START, &[ev, s1, 0, 0]);
        }
        _ => {}
    }
    0
}

/// The table of the ctx's zone: [`SKILL_TABLE_B`] when `FUN_02013FFC(zone) != 0`, else [`SKILL_TABLE`]; `None` for
/// `k >= 11` (signed; UNVERIFIED: a negative `k` reads before the table in the original, it panics here).
fn table_entry<E: FieldEnv>(env: &mut E, ctx: u32, k: u32) -> Option<(u32, u32)> {
    if k as i32 >= SKILL_COUNT as i32 {
        return None;
    }
    let zone = env.r16(ctx) as u32;
    let t = if env.callv(ZONE_TABLE_B, &[zone]) != 0 {
        &SKILL_TABLE_B
    } else {
        &SKILL_TABLE
    };
    Some(t[k as usize])
}

/// C: `FUN_0215F088`: the check fn of skill `k` for the ctx's zone (Thumb address), 0 for `k >= 11`.
pub fn check_fn<E: FieldEnv>(env: &mut E, ctx: u32, k: u32) -> u32 {
    table_entry(env, ctx, k).map_or(0, |e| e.1)
}

/// C: `FUN_0215F0B8`: the event fn of skill `k` for the ctx's zone (Thumb address), 0 for `k >= 11`.
pub fn event_fn<E: FieldEnv>(env: &mut E, ctx: u32, k: u32) -> u32 {
    table_entry(env, ctx, k).map_or(0, |e| e.0)
}

/// C: `FUN_0215F0E8`: init a skill event work: `+0` [`WORK_MAGIC`], `+4` the ctx's front OBJ, `+0xC` / `+0x10` the
/// two words of `param`, `+8` GAMESYS.
pub fn work_init<E: FieldEnv>(env: &mut E, w: u32, param: u32, ctx: u32) {
    env.w32(w, WORK_MAGIC);
    let m = env.r32(ctx + 0xC);
    env.w32(w + 4, m);
    let p0 = env.r32(param);
    let p1 = env.r32(param + 4);
    env.w32(w + 0xC, p0);
    env.w32(w + 0x10, p1);
    let gsys = env.r32(ctx + 8);
    env.w32(w + 8, gsys);
}

/// C: `FUN_0215F104`: bit `k` of the ctx's flags (`ctx+2`); 0 for `k >= 32` (a Thumb register shift; only the low byte
/// of `k` counts).
pub fn flag<E: FieldEnv>(env: &mut E, ctx: u32, k: u32) -> u32 {
    let f = env.r16(ctx + 2) as u32;
    let s = k & 0xFF;
    let m = if s >= 32 { 0 } else { 1u32 << s };
    (f & m != 0) as u32
}

/// The rect at `rect`: one of [`RECTS`] (ROM), else read from memory.
fn rect_of<E: FieldEnv>(env: &mut E, rect: u32) -> [u16; 4] {
    if let Some(&(_, r, _)) = RECTS.iter().find(|r| r.0 == rect) {
        return r;
    }
    [
        env.r16(rect),
        env.r16(rect + 2),
        env.r16(rect + 4),
        env.r16(rect + 6),
    ]
}

/// C: `FUN_0215F118`: 0 when event flag `flag` is set (`FUN_020142E8` on the EVENT WORK of the GAMEDATA of
/// `FUN_02188C80(fm)`); else 1 when `|rect.x - x| < rect.w` and `|rect.z - z| < rect.h` (32-bit signed differences),
/// 0 otherwise.
pub fn in_rect<E: FieldEnv>(env: &mut E, rect: u32, x: u32, z: u32, flag: u32, fm: u32) -> u32 {
    let gsys = env.callv(0x0218_8C80, &[fm]);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let ew = env.callv(EVENT_WORK, &[gd]);
    if env.callv(0x0201_42E8, &[ew, flag]) != 0 {
        return 0;
    }
    let r = rect_of(env, rect);
    let dx = (r[0] as u32).wrapping_sub(x) as i32;
    let dz = (r[1] as u32).wrapping_sub(z) as i32;
    let dx = dx.wrapping_abs();
    if dx >= r[2] as i32 {
        return 0;
    }
    let dz = dz.wrapping_abs();
    if dz >= r[3] as i32 {
        return 0;
    }
    1
}

#[cfg(test)]
mod tests;
