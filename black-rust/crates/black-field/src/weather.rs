//! `weather.c` (overlay 21, `0x021A19E8..0x021A1DB4`, the first part of F1 row 21.18; F11): the weather manager. A WEATHER
//! owns two weather tasks ([`crate::weather_task`]), the overlay of the current weather and two 3D backgrounds, and changes
//! the weather: at once ([`set`]), or on request ([`request`]) through a change proc run every frame ([`update`]): a plain
//! change (stop the task, swap the overlay, start it again with the new weather's data) or a cross-fade between the
//! rain-like weathers 1 / 4 / 5 (the second task starts while the first stops, then they swap).
//!
//! Spec: `tools/rom-diff/specs/black-field/weather.spec` (its header has the WEATHER layout). Verified by
//! `tools/check-fn.sh black-field weather --per-function`. The weather tasks, the 3D backgrounds (field_3dbg.c, G8), the
//! sprite system, the TCB and the overlay loader are reached through [`FieldEnv::call`] by address (D3: the weather
//! effects themselves live in the weather overlays 0x18..0x1C).

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_19E8,
    0x021A_1AAC,
    0x021A_1AF4,
    0x021A_1B38,
    0x021A_1B50,
    0x021A_1B74,
    0x021A_1BDC,
    0x021A_1BE4,
    0x021A_1BF4,
    0x021A_1BF8,
    0x021A_1C04,
    0x021A_1C1C,
    0x021A_1C34,
    0x021A_1C68,
    0x021A_1D04,
    0x021A_1D78,
];

/// Address of the string `"weather.c"` (the file argument of the allocation in [`new`]).
pub const FILE_WEATHER: u32 = 0x021D_DE74;
/// Size of the WEATHER object.
pub const WEATHER_SIZE: u32 = 0x30;
/// No weather / no request (u16).
pub const NONE: u16 = 0xFFFF;
/// No overlay loaded.
pub const NO_OVERLAY: u32 = 0xFFFF_FFFF;
/// ROM table `0x021D8868`: per weather 0..14 `{task data (in the weather's overlay), overlay id}`.
pub const WEATHER_TABLE: u32 = 0x021D_8868;
/// The weather table: `(task data, overlay)` of weathers 0..14 (ROM `0x021D8868`).
pub const WEATHERS: [(u32, u32); 15] = [
    (0x021E_E8C0, 0x18),
    (0x021E_F9E4, 0x19),
    (0x021E_FDD8, 0x1A),
    (0x021E_EBA0, 0x1B),
    (0x021E_F900, 0x19),
    (0x021E_F94C, 0x19),
    (0x021E_FE24, 0x1A),
    (0x021E_FE70, 0x1A),
    (0x021E_F998, 0x19),
    (0x021E_EBD8, 0x1C),
    (0x021E_EB8C, 0x1C),
    (0x021E_EC24, 0x1C),
    (0x021E_EBEC, 0x1B),
    (0x021E_EC70, 0x1C),
    (0x021E_EB40, 0x1C),
];
/// ROM `0x021D8850`: the parameters of the two 3D backgrounds (12 bytes each), `0x021D8840` the sprite resources.
pub const BG3D_PARAMS: u32 = 0x021D_8850;
pub const SPRITE_PARAMS: u32 = 0x021D_8840;
/// The V-blank TCB callback [`vblank`] (Thumb address).
pub const VBLANK_TCB: u32 = 0x021A_1C1D;

/// WEATHER fields.
pub const W_PROC: u32 = 0x8;
pub const W_STEP: u32 = 0xA;
pub const W_CURRENT: u32 = 0xC;
pub const W_NEXT: u32 = 0xE;
pub const W_FLAG: u32 = 0x10;
pub const W_PENDING: u32 = 0x14;
pub const W_SPRITES: u32 = 0x18;
pub const W_SPRITE_RES: u32 = 0x1C;
pub const W_TASKS: u32 = 0x20;
pub const W_OVERLAY: u32 = 0x28;
pub const W_TCB: u32 = 0x2C;

const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
/// field_3dbg.c (G8): new (params, heap), delete, draw, V-blank.
const BG3D_NEW: u32 = 0x021A_15A4;
const BG3D_DELETE: u32 = 0x021A_15D4;
const BG3D_DRAW: u32 = 0x021A_15F4;
const BG3D_VBLANK: u32 = 0x021A_1670;
/// The sprite system: new (64, 8, heap), resources new (params, 1, heap), set resources, delete both.
const SPRITES_NEW: u32 = 0x0204_B100;
const SPRITE_RES_NEW: u32 = 0x0204_B034;
const SPRITES_SET_RES: u32 = 0x0204_B258;
const SPRITE_RES_DELETE: u32 = 0x0204_B084;
const SPRITES_DELETE: u32 = 0x0204_B1CC;
/// TCB add (callback, work, priority) / delete; overlay load / unload.
const TCB_ADD: u32 = 0x0200_56A0;
const TCB_DELETE: u32 = 0x0203_0EAC;
const OVERLAY_LOAD: u32 = 0x0203_4AC4;
const OVERLAY_UNLOAD: u32 = 0x0203_4A5C;
/// weather_task.c: new, delete, one frame, start, stop, force end, status.
const TASK_NEW: u32 = 0x021A_1DB4;
const TASK_DELETE: u32 = 0x021A_1DF8;
const TASK_FRAME: u32 = 0x021A_1E08;
const TASK_START: u32 = 0x021A_1FEC;
const TASK_STOP: u32 = 0x021A_2050;
const TASK_END: u32 = 0x021A_20C8;
const TASK_STATUS: u32 = 0x021A_21B0;

/// The task data / overlay of weather `w` from the ROM table (`w` must be 0..14; the original reads past the table
/// otherwise, UNVERIFIED: this port panics).
fn data_of(w: u32) -> u32 {
    WEATHERS[w as usize].0
}
fn overlay_of(w: u32) -> u32 {
    WEATHERS[w as usize].1
}

/// C: `FUN_021A19E8(a, b, c, d, e, f, heap)`: a new WEATHER (0x30 bytes on `heap`): the two 3D backgrounds, no weather,
/// the sprite system, two weather tasks (`FUN_021A1DB4(sprites, a, b, c, d, W, e, f, heap)`), the V-blank TCB.
#[allow(clippy::too_many_arguments)]
pub fn new<E: FieldEnv>(
    env: &mut E,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    heap: u32,
) -> u32 {
    let w = env.callv(HEAP_ALLOC, &[heap, WEATHER_SIZE, 1, FILE_WEATHER, 0x158]);
    for i in 0..2u32 {
        let bg = env.callv(BG3D_NEW, &[BG3D_PARAMS + 12 * i, heap]);
        env.w32(w + 4 * i, bg);
    }
    env.w16(w + W_PROC, 0);
    env.w16(w + W_STEP, 0);
    env.w16(w + W_CURRENT, NONE);
    env.w16(w + W_NEXT, NONE);
    env.w16(w + W_PENDING, NONE);
    env.w32(w + W_OVERLAY, NO_OVERLAY);
    let s = env.callv(SPRITES_NEW, &[64, 8, heap]);
    env.w32(w + W_SPRITES, s);
    let r = env.callv(SPRITE_RES_NEW, &[SPRITE_PARAMS, 1, heap]);
    env.w32(w + W_SPRITE_RES, r);
    let s = env.r32(w + W_SPRITES);
    env.callv(SPRITES_SET_RES, &[s, r]);
    for i in 0..2u32 {
        let s = env.r32(w + W_SPRITES);
        let t = env.callv(TASK_NEW, &[s, a, b, c, d, w, e, f, heap]);
        env.w32(w + W_TASKS + 4 * i, t);
    }
    let t = env.callv(TCB_ADD, &[VBLANK_TCB, w, 0x40]);
    env.w32(w + W_TCB, t);
    w
}

/// C: `FUN_021A1AAC(w)`: delete the WEATHER: the TCB, both tasks, the weather overlay, the sprite system, the 3D
/// backgrounds.
pub fn delete<E: FieldEnv>(env: &mut E, w: u32) {
    let t = env.r32(w + W_TCB);
    env.callv(TCB_DELETE, &[t]);
    for i in 0..2u32 {
        let t = env.r32(w + W_TASKS + 4 * i);
        env.callv(TASK_DELETE, &[t]);
    }
    unload_overlay(env, w);
    let r = env.r32(w + W_SPRITE_RES);
    env.callv(SPRITE_RES_DELETE, &[r]);
    let s = env.r32(w + W_SPRITES);
    env.callv(SPRITES_DELETE, &[s]);
    for i in 0..2u32 {
        let bg = env.r32(w + 4 * i);
        env.callv(BG3D_DELETE, &[bg]);
    }
    env.callv(HEAP_FREE, &[w]);
}

/// C: `FUN_021A1AF4(w, x)`: one frame: the change proc (ROM table `0x021DDE6C`: [`change_plain`] / [`change_crossfade`]);
/// with no change running a weather asked for meanwhile is requested; both tasks run a frame.
pub fn update<E: FieldEnv>(env: &mut E, w: u32, x: u32) {
    match env.r16(w + W_PROC) {
        0 => change_plain(env, w, x),
        1 => change_crossfade(env, w, x),
        // UNVERIFIED: the original calls whatever word follows the 2-entry table
        k => panic!("weather change proc {k} outside the table"),
    }
    if env.r16(w + W_STEP) == 0 {
        let p = env.r16(w + W_PENDING);
        if p != NONE {
            request_thunk(env, w, p as u32);
            env.w16(w + W_PENDING, NONE);
        }
    }
    for i in 0..2u32 {
        let t = env.r32(w + W_TASKS + 4 * i);
        env.callv(TASK_FRAME, &[t, x]);
    }
}

/// C: `FUN_021A1B38(w)`: draw both 3D backgrounds (`w` may be null).
pub fn draw<E: FieldEnv>(env: &mut E, w: u32) {
    if w == 0 {
        return;
    }
    for i in 0..2u32 {
        let bg = env.r32(w + 4 * i);
        env.callv(BG3D_DRAW, &[bg]);
    }
}

/// C: `FUN_021A1B50(w)`: 1 when a task's status is 1 (loading its resources), else 0 (`w` may be null).
pub fn starting<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    if w == 0 {
        return 0;
    }
    for i in 0..2u32 {
        let t = env.r32(w + W_TASKS + 4 * i);
        if env.callv(TASK_STATUS, &[t]) == 1 {
            return 1;
        }
    }
    0
}

/// C: `FUN_021A1B74(w, weather, x)`: set the weather at once (unless it is already the current one, weather 0 always
/// restarts): end both tasks, swap the overlay, start task 0 with the weather's data.
pub fn set<E: FieldEnv>(env: &mut E, w: u32, weather: u32, x: u32) {
    if weather == env.r16(w + W_CURRENT) as u32 && weather != 0 {
        return;
    }
    for i in 0..2u32 {
        let t = env.r32(w + W_TASKS + 4 * i);
        env.callv(TASK_END, &[t]);
    }
    unload_overlay(env, w);
    env.w16(w + W_STEP, 0);
    load_overlay(env, w, overlay_of(weather));
    let t = env.r32(w + W_TASKS);
    env.callv(TASK_START, &[t, data_of(weather), 0, 0, 1, 1, x]);
    env.w16(w + W_CURRENT, weather as u16);
    env.w16(w + W_NEXT, NONE);
}

/// C: `FUN_021A1BDC(w, weather)`: a thunk to [`request`].
pub fn request_thunk<E: FieldEnv>(env: &mut E, w: u32, weather: u32) -> u32 {
    request(env, w, weather)
}

/// C: `FUN_021A1BE4(w, weather)`: [`request`] with the start flag cleared afterwards.
pub fn request_quiet<E: FieldEnv>(env: &mut E, w: u32, weather: u32) {
    request(env, w, weather);
    env.w32(w + W_FLAG, 0);
}

/// C: `FUN_021A1BF4(w)`: the current weather.
pub fn current<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r16(w + W_CURRENT) as u32
}

/// C: `FUN_021A1BF8(w, ovl)`: load the weather overlay `ovl` (remembered for [`unload_overlay`]).
pub fn load_overlay<E: FieldEnv>(env: &mut E, w: u32, ovl: u32) {
    env.w32(w + W_OVERLAY, ovl);
    env.callv(OVERLAY_LOAD, &[ovl]);
}

/// C: `FUN_021A1C04(w)`: unload the weather overlay, if any.
pub fn unload_overlay<E: FieldEnv>(env: &mut E, w: u32) {
    let o = env.r32(w + W_OVERLAY);
    if o != NO_OVERLAY {
        env.callv(OVERLAY_UNLOAD, &[o]);
        env.w32(w + W_OVERLAY, NO_OVERLAY);
    }
}

/// C: `FUN_021A1C1C(tcb, w)`: the V-blank TCB: both 3D backgrounds' V-blank work.
pub fn vblank<E: FieldEnv>(env: &mut E, w: u32) {
    for i in 0..2u32 {
        let bg = env.r32(w + 4 * i);
        env.callv(BG3D_VBLANK, &[bg]);
    }
}

/// C: `FUN_021A1C34(w, weather)`: request a change to `weather`: during a change it is kept for later (0); the current
/// weather again (but 0) is ignored (0); else the change starts (plain or cross-fade by [`crossfade_pair`]) and 1.
pub fn request<E: FieldEnv>(env: &mut E, w: u32, weather: u32) -> u32 {
    if env.r16(w + W_STEP) != 0 {
        env.w16(w + W_PENDING, weather as u16);
        return 0;
    }
    if env.r16(w + W_CURRENT) as u32 == weather && weather != 0 {
        return 0;
    }
    env.w16(w + W_NEXT, weather as u16);
    let cur = env.r16(w + W_CURRENT) as u32;
    let next = env.r16(w + W_NEXT) as u32;
    env.w32(w + W_FLAG, 1);
    let k = crossfade_pair(cur, next);
    env.w16(w + W_PROC, k as u16);
    env.w16(w + W_STEP, 1);
    1
}

/// C: `FUN_021A1C68(w, x)`: the plain change: stop task 0, wait for it, unload the old overlay, load the new one, start
/// task 0 with the new weather's data (the start flag of the request); then it is the current weather.
pub fn change_plain<E: FieldEnv>(env: &mut E, w: u32, x: u32) {
    match env.r16(w + W_STEP) {
        1 => {
            let t = env.r32(w + W_TASKS);
            env.callv(TASK_STOP, &[t, 1, 1]);
            env.w16(w + W_STEP, 2);
        }
        2 => {
            let t = env.r32(w + W_TASKS);
            if env.callv(TASK_STATUS, &[t]) == 0 {
                unload_overlay(env, w);
                env.w16(w + W_STEP, 3);
            }
        }
        3 => {
            let n = env.r16(w + W_NEXT) as u32;
            load_overlay(env, w, overlay_of(n));
            env.w16(w + W_STEP, 4);
        }
        4 => {
            let t = env.r32(w + W_TASKS);
            let n = env.r16(w + W_NEXT) as u32;
            let f = env.r32(w + W_FLAG);
            env.callv(TASK_START, &[t, data_of(n), 1, 1, 1, f, x]);
            env.w32(w + W_FLAG, 1);
            let n = env.r16(w + W_NEXT);
            env.w16(w + W_CURRENT, n);
            env.w16(w + W_NEXT, NONE);
            env.w16(w + W_STEP, 0);
        }
        _ => {}
    }
}

/// C: `FUN_021A1D04(w, x)`: the cross-fade: start task 1 with the new weather (kind 2) and stop task 0 (kind 2); when
/// task 0 has stopped the tasks swap and the new weather is the current one.
pub fn change_crossfade<E: FieldEnv>(env: &mut E, w: u32, x: u32) {
    match env.r16(w + W_STEP) {
        1 => {
            let t1 = env.r32(w + W_TASKS + 4);
            let n = env.r16(w + W_NEXT) as u32;
            let f = env.r32(w + W_FLAG);
            env.callv(TASK_START, &[t1, data_of(n), 1, 1, 2, f, x]);
            let t0 = env.r32(w + W_TASKS);
            env.callv(TASK_STOP, &[t0, 1, 2]);
            env.w16(w + W_STEP, 2);
        }
        2 => {
            let t0 = env.r32(w + W_TASKS);
            if env.callv(TASK_STATUS, &[t0]) == 0 {
                let t0 = env.r32(w + W_TASKS);
                let t1 = env.r32(w + W_TASKS + 4);
                env.w32(w + W_TASKS, t1);
                let n = env.r16(w + W_NEXT);
                env.w32(w + W_TASKS + 4, t0);
                env.w16(w + W_CURRENT, n);
                env.w16(w + W_NEXT, NONE);
                env.w32(w + W_FLAG, 1);
                env.w16(w + W_STEP, 0);
            }
        }
        _ => {}
    }
}

/// C: `FUN_021A1D78(cur, next)`: 1 when the change from `cur` to `next` is a cross-fade (1 -> 4 / 5 with `next` taken as
/// u16, 4 -> 1 / 5, 5 -> 4 / 1), else 0.
pub fn crossfade_pair(cur: u32, next: u32) -> u32 {
    let r = match cur {
        1 => (next.wrapping_add(0xFFFC) & 0xFFFF) <= 1,
        4 => next == 1 || next == 5,
        5 => next == 4 || next == 1,
        _ => false,
    };
    r as u32
}

#[cfg(test)]
mod tests;
