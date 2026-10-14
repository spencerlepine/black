//! `weather_task.c` (overlay 21, `0x021A1DB4..0x021A2D88`, the second part of F1 row 21.18; F11) minus its particle
//! system: the weather task. A weather (overlay 0x18..0x1C) is a data block of callbacks; the task runs them as a state
//! machine ([`frame`]: resources, start, fade in, run, fade out, end) and offers the weather overlays an API: the light
//! (field_light: table change, fade requests), the fog and zone fog, the two 3D backgrounds, a value stepper with a
//! callback, and the weather sound effect.
//!
//! Spec: `tools/rom-diff/specs/black-field/weather_task.spec` (its header has the TASK, STEPPER and SOUND layouts).
//! Verified by `tools/check-fn.sh black-field weather_task --per-function`. Every callee outside this file (field_light,
//! field_fog, field_zonefog, field_3dbg, the camera, the sound system, the heap) and the weather overlays' callbacks are
//! reached through [`FieldEnv::call`] by address. Not ported ([`RENDER_ONLY`], D3: U-phase / G8): the particle system
//! (pool, sprites, positions, the camera scroll of the particles) and the sprite / 3D-background resources; the ported
//! functions reach them through [`FieldEnv::call`] too.

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_1DB4,
    0x021A_1DF8,
    0x021A_1E08,
    0x021A_1FEC,
    0x021A_2050,
    0x021A_20C8,
    0x021A_2108,
    0x021A_2114,
    0x021A_2120,
    0x021A_2130,
    0x021A_213C,
    0x021A_2148,
    0x021A_2154,
    0x021A_2160,
    0x021A_2170,
    0x021A_2180,
    0x021A_2190,
    0x021A_21A0,
    0x021A_21B0,
    0x021A_21B8,
    0x021A_21F8,
    0x021A_2228,
    0x021A_223C,
    0x021A_2248,
    0x021A_22E0,
    0x021A_231C,
    0x021A_232C,
    0x021A_2390,
    0x021A_23B0,
    0x021A_23D4,
    0x021A_23E8,
    0x021A_23F4,
    0x021A_2400,
    0x021A_240C,
    0x021A_2418,
    0x021A_25BC,
    0x021A_25CC,
    0x021A_25DC,
    0x021A_2604,
    0x021A_26D8,
    0x021A_2724,
    0x021A_273C,
    0x021A_282C,
    0x021A_284C,
    0x021A_28CC,
    0x021A_2924,
    0x021A_2968,
    0x021A_2970,
    0x021A_2A84,
    0x021A_2AAC,
    0x021A_2AB8,
    0x021A_2B8C,
    0x021A_2C2C,
    0x021A_2C30,
    0x021A_2C6C,
    0x021A_2C98,
    0x021A_2CC8,
    0x021A_2CEC,
    0x021A_2CF8,
    0x021A_2D04,
];

/// The functions of the file that are not ported (D3): the particle system and the sprite / 3D-background resources.
pub const RENDER_ONLY: &[u32] = &[
    0x021A_21BC,
    0x021A_21C8,
    0x021A_21E0,
    0x021A_2254,
    0x021A_2424,
    0x021A_244C,
    0x021A_2564,
    0x021A_2630,
    0x021A_2638,
    0x021A_2640,
    0x021A_2654,
    0x021A_26C4,
    0x021A_274C,
    0x021A_277C,
    0x021A_27B8,
    0x021A_27F8,
    0x021A_2860,
    0x021A_28A0,
    0x021A_2978,
    0x021A_2994,
    0x021A_29A8,
    0x021A_29F4,
    0x021A_2A18,
    0x021A_2A68,
    0x021A_2BB8,
    0x021A_2BC4,
    0x021A_2BD4,
    0x021A_2C10,
    0x021A_2C24,
    0x021A_2C28,
    0x021A_2D30,
];

/// Address of the string `"weather_task.c"` (the file argument of the allocations in [`new`] and [`work_new`]).
pub const FILE_WEATHER_TASK: u32 = 0x021D_DE80;
/// Size of the TASK object.
pub const TASK_SIZE: u32 = 0x784;
/// Number of particles of a task.
pub const PARTICLES: u32 = 32;
/// ROM table `0x021D88E8`: the status of each state ([`status`]).
pub const STATUS: [u8; 14] = [0, 1, 1, 1, 1, 2, 2, 1, 3, 4, 4, 5, 5, 0];

/// TASK fields.
pub const T_FOG: u32 = 0x4;
pub const T_LIGHT: u32 = 0x8;
pub const T_SOUND: u32 = 0xC;
pub const T_ZONEFOG: u32 = 0x10;
pub const T_CAMERA: u32 = 0x14;
pub const T_WEATHER: u32 = 0x18;
pub const T_HEAP: u32 = 0x1C;
pub const T_DATA: u32 = 0x20;
pub const T_FLAGS: u32 = 0x24;
pub const T_MODE: u32 = 0x25;
pub const T_STATE: u32 = 0x26;
pub const T_READY: u32 = 0x27;
pub const T_RES: u32 = 0x28;
pub const T_WORK: u32 = 0x3C;
pub const T_SND: u32 = 0x40;
pub const T_SND_MODE: u32 = 0x48;
pub const T_STEPPER: u32 = 0x4C;
pub const T_PARTICLES: u32 = 0x74;
pub const T_FOG_FADE: u32 = 0x77C;
/// Weather data fields: particles on, work size, the state callbacks.
pub const D_PARTICLES: u32 = 0x2;
pub const D_WORK_SIZE: u32 = 0x28;
pub const D_CB_START: u32 = 0x2C;
pub const D_CB_FADE_IN: u32 = 0x30;
pub const D_CB_FADE_IN_PLAIN: u32 = 0x34;
pub const D_CB_RUN: u32 = 0x38;
pub const D_CB_FADE_OUT1: u32 = 0x3C;
pub const D_CB_FADE_OUT2: u32 = 0x40;
pub const D_CB_END: u32 = 0x44;

const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
/// field_light: change table, restore table, fade request (with a way back / once), fade set.
const LIGHT_CHANGE: u32 = 0x021A_0590;
const LIGHT_RESTORE: u32 = 0x021A_05D8;
const LIGHT_FADE: u32 = 0x021A_0618;
const LIGHT_FADE_ONCE: u32 = 0x021A_063C;
const LIGHT_FADE_SET: u32 = 0x021A_065C;
/// The camera getter `FUN_0218E188` (+0x10).
const CAMERA_GET: u32 = 0x0218_E188;
/// field_fog (G8): on, setters (+1, +3, +2, +4), byte +2, apply, fades, busy.
const FOG_ON: u32 = 0x021A_01A4;
const FOG_SET1: u32 = 0x021A_01AC;
const FOG_SET3: u32 = 0x021A_01CC;
const FOG_SET2: u32 = 0x021A_01B4;
const FOG_SET4: u32 = 0x021A_01BC;
const FOG_BYTE2: u32 = 0x021A_01D4;
const FOG_APPLY: u32 = 0x021A_01D8;
const FOG_FADE: u32 = 0x021A_01FC;
const FOG_FADE2: u32 = 0x021A_0224;
const FOG_BUSY: u32 = 0x021A_0244;
/// field_zonefog (G8): the state byte.
const ZONEFOG_STATE: u32 = 0x021A_0354;
/// The sound system: BGM busy, SE on a handle (play / play2 / stop), SE (play / play2), the player of an SE, stop it.
const BGM_BUSY: u32 = 0x0200_5C64;
const SE_PLAY_H: u32 = 0x0202_8A7C;
const SE_PLAY2_H: u32 = 0x0202_8ACC;
const SE_STOP_H: u32 = 0x0202_8B50;
const SE_PLAY: u32 = 0x0200_61E4;
const SE_PLAY2: u32 = 0x0200_61F0;
const SE_PLAYER: u32 = 0x0200_6148;
const SE_PLAYER_STOP: u32 = 0x0200_61F8;
/// Not ported (D3): 3D-background resources (new / free), particles frame, sprite resources (open, load, free, close),
/// particle sprite (new, delete), particle clear.
const BG_RES_NEW: u32 = 0x021A_2860;
const BG_RES_FREE: u32 = 0x021A_28A0;
const PARTICLES_FRAME: u32 = 0x021A_27F8;
const SPR_OPEN: u32 = 0x021A_2978;
const SPR_LOAD: u32 = 0x021A_29A8;
const SPR_FREE: u32 = 0x021A_29F4;
const SPR_CLOSE: u32 = 0x021A_2994;
const PARTICLE_NEW: u32 = 0x021A_2BD4;
const PARTICLE_DELETE: u32 = 0x021A_2C10;
const PARTICLE_CLEAR: u32 = 0x021A_2BB8;

/// The weather data of the task.
fn data<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r32(t + T_DATA)
}

/// The callback at `off` of the weather data.
fn cb_of<E: FieldEnv>(env: &mut E, t: u32, off: u32) -> u32 {
    let d = data(env, t);
    env.r32(d + off)
}

/// C: `FUN_021A1DB4(spr, camera, light, fog, zonefog, weather, sound, heap16, heap)`: a new TASK (0x784 bytes on the u16
/// `heap`).
#[allow(clippy::too_many_arguments)]
pub fn new<E: FieldEnv>(
    env: &mut E,
    spr: u32,
    camera: u32,
    light: u32,
    fog: u32,
    zonefog: u32,
    weather: u32,
    sound: u32,
    heap16: u32,
    heap: u32,
) -> u32 {
    let t = env.callv(
        HEAP_ALLOC,
        &[heap & 0xFFFF, TASK_SIZE, 1, FILE_WEATHER_TASK, 0x17E],
    );
    env.w32(t, spr);
    env.w32(t + T_CAMERA, camera);
    env.w32(t + T_FOG, fog);
    env.w32(t + T_LIGHT, light);
    env.w32(t + T_ZONEFOG, zonefog);
    env.w32(t + T_WEATHER, weather);
    env.w32(t + T_SOUND, sound);
    env.w16(t + T_HEAP, heap16 as u16);
    t
}

/// C: `FUN_021A1DF8(t)`: end the weather now and free the TASK.
pub fn delete<E: FieldEnv>(env: &mut E, t: u32) {
    end(env, t);
    env.callv(HEAP_FREE, &[t]);
}

/// C: `FUN_021A1E08(t, heap)`: one frame of the state machine (states in the spec header); a pending second fog fade is
/// applied first once the fog is idle.
pub fn frame<E: FieldEnv>(env: &mut E, t: u32, heap: u32) {
    if fog_fade_pending(env, t) != 0 {
        let f = env.r32(t + T_FOG);
        if env.callv(FOG_BUSY, &[f]) == 0 {
            fog_fade_apply(env, t);
            fog_fade_clear(env, t);
        }
    }
    let next = match env.r8(t + T_STATE) {
        1 => {
            res_load(env, t, heap);
            env.callv(BG_RES_NEW, &[t, heap]);
            work_new(env, t, heap);
            set_ready(env, t);
            5
        }
        2 => {
            res_load(env, t, heap);
            3
        }
        3 => {
            env.callv(BG_RES_NEW, &[t, heap]);
            4
        }
        4 => {
            work_new(env, t, heap);
            5
        }
        5 => {
            let cb = cb_of(env, t, D_CB_START);
            if callback(env, t, cb, heap) != 0 {
                let s = if env.r8(t + T_FLAGS) >> 4 != 0 { 6 } else { 7 };
                env.w8(t + T_STATE, s);
            }
            sound_resume_t(env, t);
            set_ready(env, t);
            return;
        }
        6 | 7 | 9 | 10 => {
            let (off, to) = match env.r8(t + T_STATE) {
                6 => (D_CB_FADE_IN, 8),
                7 => (D_CB_FADE_IN_PLAIN, 8),
                9 => (D_CB_FADE_OUT1, 10),
                _ => (D_CB_FADE_OUT2, 11),
            };
            let s = env.r8(t + T_STATE);
            let cb = cb_of(env, t, off);
            if callback(env, t, cb, heap) != 0 {
                env.w8(t + T_STATE, to);
            }
            if s != 7 {
                env.callv(PARTICLES_FRAME, &[t]);
            }
            sound_resume_t(env, t);
            return;
        }
        8 => {
            let cb = cb_of(env, t, D_CB_RUN);
            callback(env, t, cb, heap);
            env.callv(PARTICLES_FRAME, &[t]);
            sound_resume_t(env, t);
            return;
        }
        11 => {
            if ready(env, t) != 0 {
                let cb = cb_of(env, t, D_CB_END);
                callback(env, t, cb, heap);
            }
            12
        }
        12 => {
            work_free(env, t);
            env.callv(BG_RES_FREE, &[t]);
            res_free(env, t);
            reset(env, t);
            0
        }
        _ => return,
    };
    env.w8(t + T_STATE, next);
}

/// [`sound_resume`] of the task's SOUND with its handle.
fn sound_resume_t<E: FieldEnv>(env: &mut E, t: u32) {
    let h = env.r32(t + T_SOUND);
    sound_resume(env, t + T_SND, h);
}

/// C: `FUN_021A1FEC(t, data, mode, fadeout, kind, snd mode, -)`: start a weather: its data, the flags (`fadeout` in the
/// high nibble, `kind` in the low one), the start mode (0: state 1, all at once; 1: state 2, in steps; else the state
/// stays), the sound mode. The original's last argument is unused.
pub fn start<E: FieldEnv>(
    env: &mut E,
    t: u32,
    data: u32,
    mode: u32,
    fadeout: u32,
    kind: u32,
    snd_mode: u32,
) {
    env.w32(t + T_DATA, data);
    let b = (env.r8(t + T_FLAGS) & !0xF) | (kind as u8 & 0xF);
    env.w8(t + T_FLAGS, b);
    let b = (env.r8(t + T_FLAGS) & !0xF0) | ((fadeout as u8 & 0xF) << 4);
    env.w8(t + T_FLAGS, b);
    env.w8(t + T_MODE, mode as u8);
    match env.r8(t + T_MODE) {
        0 => env.w8(t + T_STATE, 1),
        1 => env.w8(t + T_STATE, 2),
        _ => {}
    }
    sound_set_mode(env, t + T_SND, snd_mode);
}

/// C: `FUN_021A2050(t, fadeout, kind)`: stop the weather (unless idle or already stopping, status 4 / 5): new flags;
/// before the run (state <= 5) straight to the end (11), else to the fade out (9) when `fadeout` is set, else the end.
pub fn stop<E: FieldEnv>(env: &mut E, t: u32, fadeout: u32, kind: u32) {
    let s = status(env, t);
    let st = env.r8(t + T_STATE);
    if st == 0 || s.wrapping_sub(4) <= 1 {
        return;
    }
    let b = (env.r8(t + T_FLAGS) & 0x0F) | ((fadeout as u8 & 0xF) << 4);
    env.w8(t + T_FLAGS, b);
    let b = (env.r8(t + T_FLAGS) & 0xF0) | (kind as u8 & 0xF);
    env.w8(t + T_FLAGS, b);
    if st <= 5 {
        env.w8(t + T_STATE, 11);
        return;
    }
    let s = if env.r8(t + T_FLAGS) >> 4 != 0 { 9 } else { 11 };
    env.w8(t + T_STATE, s);
}

/// C: `FUN_021A20C8(t)`: end the weather now: the end callback (before state 12), the frees (up to state 12), reset.
pub fn end<E: FieldEnv>(env: &mut E, t: u32) {
    let st = env.r8(t + T_STATE);
    if st == 0 {
        return;
    }
    if st <= 11 {
        let cb = cb_of(env, t, D_CB_END);
        callback(env, t, cb, 0);
    }
    if env.r8(t + T_STATE) <= 12 {
        work_free(env, t);
        env.callv(BG_RES_FREE, &[t]);
        res_free(env, t);
    }
    reset(env, t);
}

/// C: `FUN_021A2108(t, arc, member)`: field_light change table over 60 frames.
pub fn light_change<E: FieldEnv>(env: &mut E, t: u32, arc: u32, member: u32) {
    let l = env.r32(t + T_LIGHT);
    env.callv(LIGHT_CHANGE, &[l, arc, member, 60]);
}

/// C: `FUN_021A2114(t, arc, member)`: field_light change table at once (1 frame).
pub fn light_change_now<E: FieldEnv>(env: &mut E, t: u32, arc: u32, member: u32) {
    let l = env.r32(t + T_LIGHT);
    env.callv(LIGHT_CHANGE, &[l, arc, member, 1]);
}

/// C: `FUN_021A2120(t, flag)`: `flag == 1`: field_light back to its default table.
pub fn light_restore<E: FieldEnv>(env: &mut E, t: u32, flag: u32) {
    if flag == 1 {
        let l = env.r32(t + T_LIGHT);
        env.callv(LIGHT_RESTORE, &[l]);
    }
}

/// C: `FUN_021A2130(t, colour, out, back)`: field_light fade request with a way back.
pub fn light_fade<E: FieldEnv>(env: &mut E, t: u32, colour: u32, out: u32, back: u32) {
    let l = env.r32(t + T_LIGHT);
    env.callv(LIGHT_FADE, &[l, colour, out, back]);
}

/// C: `FUN_021A213C(t)`: field_light fade request set.
pub fn light_fade_set<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    let l = env.r32(t + T_LIGHT);
    env.callv(LIGHT_FADE_SET, &[l])
}

/// C: `FUN_021A2148(t)`: the camera's `+0x10` (`FUN_0218E188`).
pub fn camera_target<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    let c = env.r32(t + T_CAMERA);
    env.callv(CAMERA_GET, &[c])
}

/// C: `FUN_021A2154(t, colour, out)`: field_light fade request once.
pub fn light_fade_once<E: FieldEnv>(env: &mut E, t: u32, colour: u32, out: u32) {
    let l = env.r32(t + T_LIGHT);
    env.callv(LIGHT_FADE_ONCE, &[l, colour, out]);
}

/// The body of the five 3D-background setters ([`bg_set_8`] ..): set a field of 3D background `i` of the WEATHER through
/// the field_3dbg setter `setter`.
fn bg_set<E: FieldEnv>(env: &mut E, setter: u32, t: u32, v: u32, i: u32) {
    let w = env.r32(t + T_WEATHER);
    let bg = env.r32(w.wrapping_add(i.wrapping_mul(4)));
    env.callv(setter, &[bg, v]);
}

/// C: `FUN_021A21B0(t)`: a thunk to [`status`].
pub fn status_thunk<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    status(env, t)
}

/// C: `FUN_021A21B8(t)`: the weather's work.
pub fn work<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r32(t + T_WORK)
}

/// C: `FUN_021A21F8(t, value, period, value end, period end, period step, count period, value step, callback)`: init the
/// task's STEPPER on the task ([`stepper_init`]).
#[allow(clippy::too_many_arguments)]
pub fn stepper_start<E: FieldEnv>(
    env: &mut E,
    t: u32,
    value: u32,
    period: u32,
    vend: u32,
    pend: u32,
    pstep: u32,
    cperiod: u32,
    vstep: u32,
    cb: u32,
) {
    stepper_init(
        env,
        t + T_STEPPER,
        t,
        period,
        value,
        vend,
        pend,
        pstep,
        cperiod,
        vstep,
        cb,
    );
}

/// C: `FUN_021A2228(t, value end, period end, period step, value step)`: [`stepper_targets`] of the task's STEPPER.
pub fn stepper_retarget<E: FieldEnv>(
    env: &mut E,
    t: u32,
    vend: u32,
    pend: u32,
    pstep: u32,
    vstep: u32,
) {
    stepper_targets(env, t + T_STEPPER, vend, pend, pstep, vstep);
}

/// C: `FUN_021A223C(t, x)`: [`stepper_step`] of the task's STEPPER.
pub fn stepper_frame<E: FieldEnv>(env: &mut E, t: u32, x: u32) -> u32 {
    stepper_step(env, t + T_STEPPER, x)
}

/// C: `FUN_021A2248(t, x)`: [`stepper_tick`] of the task's STEPPER.
pub fn stepper_frame_plain<E: FieldEnv>(env: &mut E, t: u32, x: u32) -> u32 {
    stepper_tick(env, t + T_STEPPER, x)
}

/// C: `FUN_021A22E0(t, a, b, flag)`: `flag == 1`: switch the fog on with the settings `a` / `b` (+1 = 0, +3 = 31).
pub fn fog_on<E: FieldEnv>(env: &mut E, t: u32, a: u32, b: u32, flag: u32) {
    if flag != 1 {
        return;
    }
    let f = env.r32(t + T_FOG);
    env.callv(FOG_SET1, &[f, 0]);
    let f = env.r32(t + T_FOG);
    env.callv(FOG_SET3, &[f, 31]);
    let f = env.r32(t + T_FOG);
    env.callv(FOG_SET2, &[f, a]);
    let f = env.r32(t + T_FOG);
    env.callv(FOG_SET4, &[f, b]);
    let f = env.r32(t + T_FOG);
    env.callv(FOG_APPLY, &[f]);
    let f = env.r32(t + T_FOG);
    env.callv(FOG_ON, &[f, 1]);
}

/// C: `FUN_021A231C(t, flag)`: `flag == 1`: switch the fog off.
pub fn fog_off<E: FieldEnv>(env: &mut E, t: u32, flag: u32) {
    if flag == 1 {
        let f = env.r32(t + T_FOG);
        env.callv(FOG_ON, &[f, 0]);
    }
}

/// C: `FUN_021A232C(t, a, b, n, mode)`: a fog fade: mode 1 straight to `(b, a)` over `n`; mode 2 the same when the fog's
/// byte +2 is already `a`, else first to 0x7FDF over `n / 2` (signed) with the second half kept for later
/// ([`fog_fade_request`]); other modes nothing.
pub fn fog_fade<E: FieldEnv>(env: &mut E, t: u32, a: u32, b: u32, n: u32, mode: u32) {
    if mode == 1 {
        let f = env.r32(t + T_FOG);
        env.callv(FOG_FADE, &[f, b, a, n]);
        return;
    }
    if mode != 2 {
        return;
    }
    let f = env.r32(t + T_FOG);
    if a == env.callv(FOG_BYTE2, &[f]) {
        let f = env.r32(t + T_FOG);
        env.callv(FOG_FADE, &[f, b, a, n]);
        return;
    }
    let h = ((n as i32) / 2) as u32;
    let f = env.r32(t + T_FOG);
    let cur = env.callv(FOG_BYTE2, &[f]);
    let f = env.r32(t + T_FOG);
    env.callv(FOG_FADE, &[f, 0x7FDF, cur, h]);
    fog_fade_request(env, t, a, b, h);
}

/// C: `FUN_021A2390(t, a, b, flag)`: `flag == 1`: a fog fade to `a` from the fog's byte +2 over `b`.
pub fn fog_set<E: FieldEnv>(env: &mut E, t: u32, a: u32, b: u32, flag: u32) {
    if flag == 1 {
        let f = env.r32(t + T_FOG);
        let cur = env.callv(FOG_BYTE2, &[f]);
        let f = env.r32(t + T_FOG);
        env.callv(FOG_FADE, &[f, a, cur, b]);
    }
}

/// C: `FUN_021A23B0(t)`: the fog is idle with no pending second fade.
pub fn fog_idle<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    let f = env.r32(t + T_FOG);
    if env.callv(FOG_BUSY, &[f]) != 0 {
        return 0;
    }
    (fog_fade_pending(env, t) == 0) as u32
}

/// C: `FUN_021A23D4(t)`: the zone fog's state is 1.
pub fn zonefog_on<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    let z = env.r32(t + T_ZONEFOG);
    (env.callv(ZONEFOG_STATE, &[z]) == 1) as u32
}

/// The body of the five zone-fog thunks ([`zonefog_get_0380`] ..): the field_zonefog getter `getter` of the task's zone
/// fog.
fn zonefog_get<E: FieldEnv>(env: &mut E, getter: u32, t: u32) -> u32 {
    let z = env.r32(t + T_ZONEFOG);
    env.callv(getter, &[z])
}

/// C: `FUN_021A25BC(t, se)`: start the weather SE ([`sound_start`]).
pub fn se_start<E: FieldEnv>(env: &mut E, t: u32, se: u32) {
    let h = env.r32(t + T_SOUND);
    sound_start(env, t + T_SND, h, se);
}

/// C: `FUN_021A25CC(t)`: stop the weather SE ([`sound_stop`]).
pub fn se_stop<E: FieldEnv>(env: &mut E, t: u32) {
    let h = env.r32(t + T_SOUND);
    sound_stop(env, t + T_SND, h);
}

/// C: `FUN_021A25DC(t, se)`: play an SE when the BGM system is idle (on the handle in handle mode).
pub fn se_play<E: FieldEnv>(env: &mut E, t: u32, se: u32) {
    if env.callv(BGM_BUSY, &[]) != 0 {
        return;
    }
    if env.r32(t + T_SND_MODE) != 0 {
        let h = env.r32(t + T_SOUND);
        env.callv(SE_PLAY_H, &[h, se]);
    } else {
        env.callv(SE_PLAY, &[se]);
    }
}

/// C: `FUN_021A2604(t, se, v)`: [`se_play`] with a second parameter.
pub fn se_play2<E: FieldEnv>(env: &mut E, t: u32, se: u32, v: u32) {
    if env.callv(BGM_BUSY, &[]) != 0 {
        return;
    }
    if env.r32(t + T_SND_MODE) != 0 {
        let h = env.r32(t + T_SOUND);
        env.callv(SE_PLAY2_H, &[h, se, v]);
    } else {
        env.callv(SE_PLAY2, &[se, v]);
    }
}

/// C: `FUN_021A26D8(t)`: reset the TASK: zero everything but the links `+0..+0x1B` and the heap `+0x1C`.
pub fn reset<E: FieldEnv>(env: &mut E, t: u32) {
    let keep: Vec<u32> = (0..7).map(|k| env.r32(t + 4 * k)).collect();
    let heap = env.r16(t + T_HEAP);
    env.mem().fill(t, TASK_SIZE, 0);
    for (k, v) in keep.into_iter().enumerate() {
        env.w32(t + 4 * k as u32, v);
    }
    env.w16(t + T_HEAP, heap);
}

/// C: `FUN_021A2724(t, cb, x)`: run a weather callback `cb(t, kind, x)` (1 when there is none).
pub fn callback<E: FieldEnv>(env: &mut E, t: u32, cb: u32, x: u32) -> u32 {
    if cb == 0 {
        return 1;
    }
    let kind = (env.r8(t + T_FLAGS) & 0xF) as u32;
    env.callv(cb & !1, &[t, kind, x])
}

/// C: `FUN_021A273C(t)`: the status of the task's state ([`STATUS`]: 0 idle, 1 loading (states 1..4, 7), 2 starting (5, 6), 3 running (8), 4 / 5
/// stopping). The state must be 0..13 (the original reads past the table otherwise, UNVERIFIED: this port panics).
pub fn status<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    STATUS[env.r8(t + T_STATE) as usize] as u32
}

/// C: `FUN_021A282C(t, heap)`: load the sprite resources of the weather (not ported, D3).
pub fn res_load<E: FieldEnv>(env: &mut E, t: u32, heap: u32) {
    let d = data(env, t);
    env.callv(SPR_OPEN, &[t + T_RES, d, heap]);
    let d = data(env, t);
    env.callv(SPR_LOAD, &[t + T_RES, d, heap]);
}

/// C: `FUN_021A284C(t)`: free the sprite resources (not ported, D3).
pub fn res_free<E: FieldEnv>(env: &mut E, t: u32) {
    env.callv(SPR_FREE, &[t + T_RES]);
    env.callv(SPR_CLOSE, &[t + T_RES]);
}

/// C: `FUN_021A28CC(t, heap)`: allocate the weather's work (its size from the data) and, when the weather has particles,
/// create the 32 particle sprites (not ported, D3).
pub fn work_new<E: FieldEnv>(env: &mut E, t: u32, heap: u32) {
    let d = data(env, t);
    let size = env.r32(d + D_WORK_SIZE);
    let w = env.callv(HEAP_ALLOC, &[heap, size, 1, FILE_WEATHER_TASK, 0x874]);
    env.w32(t + T_WORK, w);
    let d = data(env, t);
    if env.r16(d + D_PARTICLES) != 0 {
        for i in 0..PARTICLES {
            let spr = env.r32(t);
            env.callv(
                PARTICLE_NEW,
                &[t + T_PARTICLES + 0x38 * i, t, t + T_RES, spr, heap],
            );
        }
    }
}

/// C: `FUN_021A2924(t)`: free the work and the particles, stop the SE.
pub fn work_free<E: FieldEnv>(env: &mut E, t: u32) {
    let w = env.r32(t + T_WORK);
    if w != 0 {
        env.callv(HEAP_FREE, &[w]);
        env.w32(t + T_WORK, 0);
    }
    let d = data(env, t);
    if env.r16(d + D_PARTICLES) != 0 {
        for i in 0..PARTICLES {
            let p = t + T_PARTICLES + 0x38 * i;
            env.callv(PARTICLE_DELETE, &[p]);
            env.callv(PARTICLE_CLEAR, &[p]);
        }
    }
    se_stop(env, t);
}

/// C: `FUN_021A2968(t)`: the resources are ready.
pub fn set_ready<E: FieldEnv>(env: &mut E, t: u32) {
    env.w8(t + T_READY, 1);
}

/// C: `FUN_021A2970(t)`: the ready flag.
pub fn ready<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r8(t + T_READY) as u32
}

/// C: `FUN_021A2A84(s, obj, period, value, value end, period end, period step, count period, value step, callback)`: a
/// STEPPER (countdown and count 0).
#[allow(clippy::too_many_arguments)]
pub fn stepper_init<E: FieldEnv>(
    env: &mut E,
    s: u32,
    obj: u32,
    period: u32,
    value: u32,
    vend: u32,
    pend: u32,
    pstep: u32,
    cperiod: u32,
    vstep: u32,
    cb: u32,
) {
    env.w32(s, obj);
    env.w16(s + 6, 0);
    env.w16(s + 0xA, 0);
    env.w16(s + 4, value as u16);
    env.w16(s + 0xC, vend as u16);
    env.w16(s + 8, period as u16);
    env.w16(s + 0xE, pend as u16);
    env.w16(s + 0x10, pstep as u16);
    env.w16(s + 0x12, cperiod as u16);
    env.w32(s + 0x14, vstep);
    env.w32(s + 0x18, cb);
}

/// C: `FUN_021A2AAC(s, value end, period end, period step, value step)`: new targets of a STEPPER.
pub fn stepper_targets<E: FieldEnv>(
    env: &mut E,
    s: u32,
    vend: u32,
    pend: u32,
    pstep: u32,
    vstep: u32,
) {
    env.w16(s + 0xC, vend as u16);
    env.w16(s + 0xE, pend as u16);
    env.w16(s + 0x10, pstep as u16);
    env.w32(s + 0x14, vstep);
}

/// Signed halfword of a STEPPER.
fn sh<E: FieldEnv>(env: &mut E, a: u32) -> i32 {
    env.r16(a) as i16 as i32
}

/// The end test of a STEPPER ramp: the current value `c` reached the end `e` in the direction of the step (`neg`: step
/// <= 0).
fn reached(c: i32, e: i32, neg: bool) -> bool {
    (c >= e && !neg) || (c <= e && neg)
}

/// C: `FUN_021A2AB8(s, x)`: one STEPPER frame. The period (step +0x10) and the value (step +0x14) each stop at their end
/// (clamped; a zero step counts as at the end); when the countdown runs out the callback gets `(obj, value, x)`, the
/// countdown restarts from the period, the period moves by its step and every `count period` frames the value by its
/// step, while not at their ends. Answers 1 when the period is at its end.
pub fn stepper_step<E: FieldEnv>(env: &mut E, s: u32, x: u32) -> u32 {
    let mut flags = 0u32;
    let w = sh(env, s + 0x10);
    if w == 0 {
        flags |= 1;
    } else {
        let e = sh(env, s + 0xE);
        let c = sh(env, s + 8);
        if reached(c, e, w <= 0) {
            flags |= 1;
            env.w16(s + 8, e as u16);
        }
    }
    let v = env.r32(s + 0x14) as i32;
    if v == 0 {
        flags |= 2;
    } else {
        let e = sh(env, s + 0xC);
        let c = sh(env, s + 4);
        if reached(c, e, v <= 0) {
            flags |= 2;
            env.w16(s + 4, e as u16);
        }
    }
    let n = sh(env, s + 6) - 1;
    env.w16(s + 6, n as u16);
    if sh(env, s + 6) <= 0 {
        let (obj, val, cb) = (env.r32(s), sh(env, s + 4) as u32, env.r32(s + 0x18));
        env.callv(cb & !1, &[obj, val, x]);
        let p = env.r16(s + 8);
        env.w16(s + 6, p);
        if flags & 1 == 0 {
            let p = sh(env, s + 8) + sh(env, s + 0x10);
            env.w16(s + 8, p as u16);
        }
        if flags & 2 == 0 {
            let k = sh(env, s + 0xA) + 1;
            env.w16(s + 0xA, k as u16);
            if sh(env, s + 0xA) >= sh(env, s + 0x12) {
                env.w16(s + 0xA, 0);
                let step = env.r32(s + 0x14) as i16 as i32;
                let v = sh(env, s + 4) + step;
                env.w16(s + 4, v as u16);
            }
        }
    }
    flags & 1
}

/// C: `FUN_021A2B8C(s, x)`: a plain STEPPER frame: when the countdown runs out the callback gets `(obj, value, x)` and
/// the countdown restarts from the period (1), else 0.
pub fn stepper_tick<E: FieldEnv>(env: &mut E, s: u32, x: u32) -> u32 {
    let n = sh(env, s + 6) - 1;
    env.w16(s + 6, n as u16);
    if sh(env, s + 6) > 0 {
        return 0;
    }
    let (obj, val, cb) = (env.r32(s), sh(env, s + 4) as u32, env.r32(s + 0x18));
    env.callv(cb & !1, &[obj, val, x]);
    let p = env.r16(s + 8);
    env.w16(s + 6, p);
    1
}

/// C: `FUN_021A2C2C(snd, mode)`: the SOUND's handle mode (play on the handle when set).
pub fn sound_set_mode<E: FieldEnv>(env: &mut E, snd: u32, mode: u32) {
    env.w32(snd + 8, mode);
}

/// Play the SOUND's SE (on the handle `h` in handle mode).
fn sound_play<E: FieldEnv>(env: &mut E, snd: u32, h: u32) {
    let se = env.r16(snd + 2) as u32;
    if env.r32(snd + 8) != 0 {
        env.callv(SE_PLAY_H, &[h, se]);
    } else {
        env.callv(SE_PLAY, &[se]);
    }
}

/// C: `FUN_021A2C30(snd, h, se)`: start the weather SE (the previous one stopped): at once when the BGM system is idle
/// (state 1), else later (state 2, [`sound_resume`]).
pub fn sound_start<E: FieldEnv>(env: &mut E, snd: u32, h: u32, se: u32) {
    sound_stop(env, snd, h);
    if env.callv(BGM_BUSY, &[]) != 0 {
        env.w16(snd, 2);
        env.w16(snd + 2, se as u16);
        return;
    }
    env.w16(snd, 1);
    env.w16(snd + 2, se as u16);
    sound_play(env, snd, h);
}

/// C: `FUN_021A2C6C(snd, h)`: stop the weather SE if it plays (state 1 -> 0).
pub fn sound_stop<E: FieldEnv>(env: &mut E, snd: u32, h: u32) {
    if env.r16(snd) != 1 {
        return;
    }
    env.w16(snd, 0);
    let se = env.r16(snd + 2) as u32;
    if env.r32(snd + 8) != 0 {
        env.callv(SE_STOP_H, &[h, se]);
    } else {
        let p = env.callv(SE_PLAYER, &[se]);
        env.callv(SE_PLAYER_STOP, &[p]);
    }
}

/// C: `FUN_021A2C98(snd, h)`: a waiting weather SE (state 2) starts once the BGM system is idle (state 1).
pub fn sound_resume<E: FieldEnv>(env: &mut E, snd: u32, h: u32) {
    if env.r16(snd) != 2 || env.callv(BGM_BUSY, &[]) != 0 {
        return;
    }
    env.w16(snd, 1);
    sound_play(env, snd, h);
}

/// C: `FUN_021A2CC8(t, a, b, n)`: keep the second half of a fog fade: `{1, (u8) a, (u8)(n / 2)}` (signed), `b`.
pub fn fog_fade_request<E: FieldEnv>(env: &mut E, t: u32, a: u32, b: u32, n: u32) {
    env.w8(t + T_FOG_FADE, 1);
    env.w8(t + T_FOG_FADE + 1, a as u8);
    env.w32(t + T_FOG_FADE + 4, b);
    env.w8(t + T_FOG_FADE + 2, ((n as i32) / 2) as u8);
}

/// C: `FUN_021A2CEC(t)`: drop the pending fog fade.
pub fn fog_fade_clear<E: FieldEnv>(env: &mut E, t: u32) {
    env.w8(t + T_FOG_FADE, 0);
}

/// C: `FUN_021A2CF8(t)`: a fog fade is pending.
pub fn fog_fade_pending<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r8(t + T_FOG_FADE) as u32
}

/// C: `FUN_021A2D04(t)`: run the pending fog fade (`FUN_021A0224(fog, 0x7FDF, b, a, a, n)`).
pub fn fog_fade_apply<E: FieldEnv>(env: &mut E, t: u32) {
    let a = env.r8(t + T_FOG_FADE + 1) as u32;
    let n = env.r8(t + T_FOG_FADE + 2) as u32;
    let f = env.r32(t + T_FOG);
    let b = env.r32(t + T_FOG_FADE + 4);
    env.callv(FOG_FADE2, &[f, 0x7FDF, b, a, a, n]);
}

/// C: `FUN_021A2160(t, v, i)`: field_3dbg `FUN_021A16D4` (+8 = v) of 3D background `i`.
pub fn bg_set_8<E: FieldEnv>(env: &mut E, t: u32, v: u32, i: u32) {
    bg_set(env, 0x021A_16D4, t, v, i);
}

/// C: `FUN_021A2170(t, v, i)`: field_3dbg `FUN_021A16E4` (+0x14 = v) of 3D background `i`.
pub fn bg_set_14<E: FieldEnv>(env: &mut E, t: u32, v: u32, i: u32) {
    bg_set(env, 0x021A_16E4, t, v, i);
}

/// C: `FUN_021A2180(t, v, i)`: field_3dbg `FUN_021A16E8` (+0x18 = v) of 3D background `i`.
pub fn bg_set_18<E: FieldEnv>(env: &mut E, t: u32, v: u32, i: u32) {
    bg_set(env, 0x021A_16E8, t, v, i);
}

/// C: `FUN_021A2190(t, v, i)`: field_3dbg `FUN_021A16DC` (+0xC = v) of 3D background `i`.
pub fn bg_set_c<E: FieldEnv>(env: &mut E, t: u32, v: u32, i: u32) {
    bg_set(env, 0x021A_16DC, t, v, i);
}

/// C: `FUN_021A21A0(t, v, i)`: field_3dbg `FUN_021A16E0` (+0x10 = v) of 3D background `i`.
pub fn bg_set_10<E: FieldEnv>(env: &mut E, t: u32, v: u32, i: u32) {
    bg_set(env, 0x021A_16E0, t, v, i);
}

/// C: `FUN_021A23E8(t)`: thunk to field_zonefog `FUN_021A0380` of the task's zone fog.
pub fn zonefog_get_0380<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    zonefog_get(env, 0x021A_0380, t)
}

/// C: `FUN_021A23F4(t)`: thunk to field_zonefog `FUN_021A0390` of the task's zone fog.
pub fn zonefog_get_0390<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    zonefog_get(env, 0x021A_0390, t)
}

/// C: `FUN_021A2400(t)`: thunk to field_zonefog `FUN_021A03A4` of the task's zone fog.
pub fn zonefog_get_03a4<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    zonefog_get(env, 0x021A_03A4, t)
}

/// C: `FUN_021A240C(t)`: thunk to field_zonefog `FUN_021A03B8` of the task's zone fog.
pub fn zonefog_get_03b8<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    zonefog_get(env, 0x021A_03B8, t)
}

/// C: `FUN_021A2418(t)`: thunk to field_zonefog `FUN_021A03CC` of the task's zone fog.
pub fn zonefog_get_03cc<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    zonefog_get(env, 0x021A_03CC, t)
}

#[cfg(test)]
mod tests;
