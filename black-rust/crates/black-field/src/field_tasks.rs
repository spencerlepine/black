//! The concrete field tasks after `field_task_manager.c` (F1 row 21.75, second part; overlay 21,
//! `0x021C6F00..0x021C7DAA`; no `__FILE__` string, named by role; F14), plus the compiler-runtime float compare
//! `FUN_0209BBB0` ([`fcmpgt`]) that they branch on.
//!
//! Each task is a constructor `X(fm, ...)` that creates a [`crate::field_task`] FIELD TASK (`021C6C70(heap, size,
//! proc)`, heap = `02188C84(fm)`, work = `021C6CCC(task)`) and fills its work block, and the proc `int proc(work)` the
//! task manager calls every frame (1 = done): camera distance / angle / target / position interpolation, OBJ spin
//! (three direction curves), player and OBJ moves and offsets, an OBJ shake, a wait, a deferred call and a periodic
//! call. The values only feed the camera / OBJ setters (all hooks); what they look like is D3 (rendered by the field
//! view), the timing and the values passed are ported here.
//!
//! Every callee outside this area is reached through [`FieldEnv::call`] by address (the field_task constructors, the
//! fieldmap / camera / OBJ / player getters and setters, sound, and the soft-float helpers that are not ported:
//! `fmul` 0209C77C, `fdiv` 0209CE10, `f2u` 0209C4C8, `f2d` 0209BD38, the double helper 0209C5E0 and `d2f` 0209B64C;
//! the two double results go through [`FieldEnv64`]). The ported soft-float helpers (`i2f`, `u2f`, `f2i`, `fadd`,
//! `fsub`) are called as [`black_core::softfloat`]. Stack structs of the original are [`Mem::push_local`] windows.
//! Verified by `tools/check-fn.sh black-field field_tasks --per-function` (spec
//! `tools/rom-diff/specs/black-field/field_tasks.spec`).
//!
//! [`Mem::push_local`]: crate::hooks::Mem::push_local

use crate::hooks::{CArg, FieldEnv};
use black_core::softfloat::{f2i, fadd, fsub, i2f, u2f};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0209_BBB0,
    0x021C_6F00,
    0x021C_6F30,
    0x021C_701C,
    0x021C_7058,
    0x021C_7094,
    0x021C_70DC,
    0x021C_71A4,
    0x021C_71D4,
    0x021C_7220,
    0x021C_7260,
    0x021C_72AC,
    0x021C_72F8,
    0x021C_734C,
    0x021C_73B4,
    0x021C_7418,
    0x021C_748C,
    0x021C_7500,
    0x021C_754C,
    0x021C_7584,
    0x021C_7684,
    0x021C_76C4,
    0x021C_7730,
    0x021C_77D8,
    0x021C_77FC,
    0x021C_7814,
    0x021C_7844,
    0x021C_7858,
    0x021C_7878,
    0x021C_78DC,
    0x021C_7940,
    0x021C_7A20,
    0x021C_7A7C,
    0x021C_7AE4,
    0x021C_7B1C,
    0x021C_7C10,
    0x021C_7C74,
    0x021C_7D28,
    0x021C_7D78,
];

/// A [`FieldEnv`] that can also call a routine returning a 64-bit value (`r1:r0`, a double).
pub trait FieldEnv64: FieldEnv {
    /// Calls the routine at `addr` with scalar arguments; returns `r1 << 32 | r0`.
    fn call64(&mut self, addr: u32, args: &[u32]) -> u64;
}

/// C: `FUN_021C6C70(heap, size, proc)` (field_task): a new task.
const TASK_NEW: u32 = 0x021C_6C70;
/// C: `FUN_021C6CCC(task)` (field_task): its work block.
const TASK_WORK: u32 = 0x021C_6CCC;
/// C: `FUN_02188C84(fm)`: the field heap id.
const HEAP_ID: u32 = 0x0218_8C84;
/// C: `FUN_02188C58(fm)`: the field camera.
const CAMERA: u32 = 0x0218_8C58;
/// C: `FUN_02188CB4(fm)`: the player.
const PLAYER: u32 = 0x0218_8CB4;
/// C: `FUN_021A3074(player)`: the player's OBJ.
const PLAYER_OBJ: u32 = 0x021A_3074;
/// C: `FUN_0216D5F8(obj)`: the OBJ's direction.
const OBJ_DIR: u32 = 0x0216_D5F8;
/// C: `FUN_0216D410(obj, dir)`: set the OBJ's displayed direction.
const OBJ_SET_DRAW_DIR: u32 = 0x0216_D410;
/// C: `FUN_0216D5E0(obj, dir)`: set the OBJ's direction.
const OBJ_SET_DIR: u32 = 0x0216_D5E0;
/// C: `FUN_02188CD0(fm)`.
const FM_2CD0: u32 = 0x0218_8CD0;
/// C: `FUN_020061E4(se)`: play a sound effect.
const PLAY_SE: u32 = 0x0200_61E4;
/// C: camera word get / set (`FUN_0218E138(cam)` / `FUN_0218E13C(cam, v)`), saved and cleared around camera changes.
const CAM_WORD_GET: u32 = 0x0218_E138;
const CAM_WORD_SET: u32 = 0x0218_E13C;
/// C: compiler-runtime float multiply / divide and float -> u32 (not ported; pure).
const FMUL: u32 = 0x0209_C77C;
const FDIV: u32 = 0x0209_CE10;
const F2U: u32 = 0x0209_C4C8;
/// C: `FUN_0209BD38(f)` (float -> double), `FUN_0209C5E0(d)` (double -> double), `FUN_0209B64C(d)` (double -> float).
const F2D: u32 = 0x0209_BD38;
const D_FN: u32 = 0x0209_C5E0;
const D2F: u32 = 0x0209_B64C;

const F_ONE: u32 = 0x3F80_0000;
const F_HALF: u32 = 0x3F00_0000;
const F_THREE: u32 = 0x4040_0000;
const F_FOUR: u32 = 0x4080_0000;
const F_MINUS_TWO: u32 = 0xC000_0000;
const F_4096: u32 = 0x4580_0000;

/// The three direction curves' tables (ROM `0x021DAAC0` / `0x021DAAB8` / `0x021DAAC8`, 4 x u16 each; all equal).
pub const DIR_TABLE_A: [u16; 4] = [1, 3, 0, 2];
pub const DIR_TABLE_B: [u16; 4] = [1, 3, 0, 2];
pub const DIR_TABLE_C: [u16; 4] = [1, 3, 0, 2];

/// C: `FUN_0209BBB0(a, b)` (ARM runtime helper): 1 when the float `a` is greater than the float `b`, 0 otherwise or
/// when either is a NaN (the original also sets the C flag to the answer; its callers branch on it).
pub fn fcmpgt(a: u32, b: u32) -> u32 {
    if a << 1 > 0xFF00_0000 || b << 1 > 0xFF00_0000 {
        return 0;
    }
    let s = |x: u32| -> i32 {
        if (x as i32) < 0 {
            ((x & 0x7FFF_FFFF) as i32).wrapping_neg()
        } else {
            x as i32
        }
    };
    (s(a) > s(b)) as u32
}

fn fmul<E: FieldEnv>(env: &mut E, a: u32, b: u32) -> u32 {
    env.callv(FMUL, &[a, b])
}

fn fdiv<E: FieldEnv>(env: &mut E, a: u32, b: u32) -> u32 {
    env.callv(FDIV, &[a, b])
}

/// `round(4096 * q)` as the tasks write it: `f2i(q > 0 ? 0.5 + 4096 * q : 4096 * q - 0.5)`.
fn round_fx<E: FieldEnv>(env: &mut E, q: u32) -> i32 {
    let gt = fcmpgt(q, 0) != 0;
    let p = fmul(env, F_4096, q);
    f2i(if gt { fadd(F_HALF, p) } else { fsub(p, F_HALF) })
}

/// `FX_Mul(a, b)`: `(a * b + 0x800) >> 12` on 64 bits (the `0209C054` sequence of the original).
fn fx_mul(a: u32, b: i32) -> u32 {
    ((i64::from(a as i32) * i64::from(b)).wrapping_add(0x800) >> 12) as u32
}

/// Index `f2i(x) % 4` (C remainder) into a 4-entry direction table copied to the stack.
fn dir_of(t: &[u16; 4], x: u32) -> u16 {
    let k = f2i(x).wrapping_rem(4);
    // UNVERIFIED: a negative index reads below the stack copy (garbage) in the original; the float is never negative
    // in the game (products of u2f values) and the spec keeps it non-negative; the port answers 0.
    if k < 0 {
        0
    } else {
        t[k as usize]
    }
}

/// Copy the 12-byte vector at `src` to `dst`.
fn copy_vec<E: FieldEnv>(env: &mut E, dst: u32, src: u32) {
    for k in 0..3 {
        let v = env.r32(src + 4 * k);
        env.w32(dst + 4 * k, v);
    }
}

/// C: `FUN_021C6F00(fm, frames, target)`: camera distance task (proc [`zoom_proc`]); work 0x14 `{fm, u8 seq, u16
/// count, u16 frames, start, target}`.
pub fn zoom_new<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, target: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x14, 0x021C_6F31]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w8(w + 4, 0);
    env.w32(w, fm);
    env.w16(w + 6, 0);
    env.w16(w + 8, frames as u16);
    env.w32(w + 0x10, target);
    t
}

/// C: `FUN_021C6F30(work)`: seq 0 reads the start distance; seq 1 sets `start * (1 - t) + target * t` (`t` = count /
/// frames, fx32 / 4096 floats) with the camera word cleared around it; done when count >= frames.
pub fn zoom_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let fm = env.r32(w);
    let cam = env.callv(CAMERA, &[fm]);
    match env.r8(w + 4) {
        0 => {
            let v = env.callv(0x0218_ED54, &[cam]);
            env.w32(w + 0xC, v);
            let s = env.r8(w + 4).wrapping_add(1);
            env.w8(w + 4, s);
            0
        }
        1 => {
            let c = env.r16(w + 6).wrapping_add(1);
            env.w16(w + 6, c);
            let saved = env.callv(CAM_WORD_GET, &[cam]);
            env.callv(CAM_WORD_SET, &[cam, 0]);
            let cnt = u2f(u32::from(env.r16(w + 6)));
            let frames = u2f(u32::from(env.r16(w + 8)));
            let t = fdiv(env, cnt, frames);
            let start = i2f(env.r32(w + 0xC) as i32);
            let a = fdiv(env, start, F_4096);
            let target = i2f(env.r32(w + 0x10) as i32);
            let b = fdiv(env, target, F_4096);
            let x = fmul(env, fsub(F_ONE, t), a);
            let y = fmul(env, t, b);
            let z = fadd(x, y);
            let v = round_fx(env, z);
            env.callv(0x0218_ED58, &[cam, v as u32]);
            env.callv(CAM_WORD_SET, &[cam, saved]);
            (env.r16(w + 8) <= env.r16(w + 6)) as u32
        }
        _ => 0,
    }
}

/// C: `FUN_021C701C(fm, frames, target)` (axis 0) / `FUN_021C7058` (axis 1): camera angle task (proc [`rot_proc`]);
/// work 0x1C `{fm, cam, axis, u8 seq, u16 count, u16 frames, start, target}`.
pub fn rot_new<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, target: u32, axis: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x1C, 0x021C_71A5]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w32(w, fm);
    let cam = env.callv(CAMERA, &[fm]);
    env.w32(w + 4, cam);
    env.w8(w + 0xC, 0);
    env.w32(w + 8, axis);
    env.w16(w + 0xE, 0);
    env.w16(w + 0x10, frames as u16);
    env.w32(w + 0x18, target);
    t
}

/// C: `FUN_021C7094(work)`: read the current angle of the axis (1: `0218ED2C`, else `0218ED40`) and unwrap start /
/// target so that the turn takes the short way (a gap over 0x8000 adds 0x10000 to the smaller one).
pub fn rot_init<E: FieldEnv>(env: &mut E, w: u32) {
    let axis = env.r32(w + 8);
    let cam = env.r32(w + 4);
    let mut target = env.r32(w + 0x18);
    let mut cur = if axis == 1 {
        env.callv(0x0218_ED2C, &[cam])
    } else {
        env.callv(0x0218_ED40, &[cam])
    };
    if cur < target {
        if target - cur > 0x8000 {
            cur = cur.wrapping_add(0x10000);
        }
    } else if target < cur && cur - target > 0x8000 {
        target = target.wrapping_add(0x10000);
    }
    env.w32(w + 0x14, cur);
    env.w32(w + 0x18, target);
}

/// C: `FUN_021C70DC(work)`: one step: count + 1; up to `frames`, set the axis to `start * (1 - s) + target * s` with
/// `s = t^2 (3 - 2t)` (`t` = count / frames), converted with `f2u` and truncated to u16.
pub fn rot_step<E: FieldEnv>(env: &mut E, w: u32) {
    let frames = env.r16(w + 0x10);
    let cam = env.r32(w + 4);
    let c = env.r16(w + 0xE).wrapping_add(1);
    env.w16(w + 0xE, c);
    let cnt = env.r16(w + 0xE);
    if cnt > frames {
        return;
    }
    let t = fdiv(env, u2f(u32::from(cnt)), u2f(u32::from(frames)));
    let a = fmul(env, F_MINUS_TWO, t);
    let b = fadd(F_THREE, a);
    let t2 = fmul(env, t, t);
    let s = fmul(env, b, t2);
    let e = fsub(F_ONE, s);
    let start = u2f(env.r32(w + 0x14));
    let f = fmul(env, e, start);
    let target = u2f(env.r32(w + 0x18));
    let g = fmul(env, s, target);
    let h = fadd(f, g);
    let v = env.callv(F2U, &[h]) & 0xFFFF;
    let saved = env.callv(CAM_WORD_GET, &[cam]);
    env.callv(CAM_WORD_SET, &[cam, 0]);
    match env.r32(w + 8) {
        0 => {
            env.callv(0x0218_ED48, &[cam, v]);
        }
        1 => {
            env.callv(0x0218_ED34, &[cam, v]);
        }
        _ => {}
    }
    env.callv(CAM_WORD_SET, &[cam, saved]);
}

/// C: `FUN_021C71A4(work)`: the camera angle proc: seq 0 [`rot_init`], seq 1 [`rot_step`] until count >= frames.
pub fn rot_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    match env.r8(w + 0xC) {
        0 => {
            rot_init(env, w);
            let s = env.r8(w + 0xC).wrapping_add(1);
            env.w8(w + 0xC, s);
            0
        }
        1 => {
            rot_step(env, w);
            (env.r16(w + 0x10) <= env.r16(w + 0xE)) as u32
        }
        _ => 0,
    }
}

/// C: `FUN_021C71D4(fm, frames, turns)` (proc `021C7418`) / `FUN_021C7260` (proc `021C748C`): spin the player's OBJ;
/// work 0x10 `{fm, obj, u16 count, u16 frames, u8 turns, u16 start dir}`.
pub fn spin_new_player<E: FieldEnv>(
    env: &mut E,
    fm: u32,
    frames: u32,
    turns: u32,
    proc_: u32,
) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let player = env.callv(PLAYER, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x10, proc_]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w32(w, fm);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    env.w32(w + 4, obj);
    env.w16(w + 8, 0);
    env.w16(w + 0xA, frames as u16);
    env.w8(w + 0xC, turns as u8);
    let d = env.callv(OBJ_DIR, &[obj]);
    env.w16(w + 0xE, d as u16);
    t
}

/// C: `FUN_021C7220(fm, frames, turns, obj)`: the [`spin_new_player`] task (proc `021C7418`) for a given OBJ.
pub fn spin_new_obj<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, turns: u32, obj: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x10, 0x021C_7419]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w32(w, fm);
    env.w32(w + 4, obj);
    env.w16(w + 8, 0);
    env.w16(w + 0xA, frames as u16);
    env.w8(w + 0xC, turns as u8);
    let d = env.callv(OBJ_DIR, &[obj]);
    env.w16(w + 0xE, d as u16);
    t
}

/// C: `FUN_021C72AC(fm, frames, turns, end dir)`: spin the player's OBJ with the curve [`curve_c`] (proc
/// `021C7500`); the work's `+0xE` is the direction set at the end.
pub fn spin_new_end<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, turns: u32, end: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let player = env.callv(PLAYER, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x10, 0x021C_7501]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w32(w, fm);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    env.w32(w + 4, obj);
    env.w16(w + 8, 0);
    env.w16(w + 0xA, frames as u16);
    env.w8(w + 0xC, turns as u8);
    env.w16(w + 0xE, end as u16);
    t
}

/// C: `FUN_021C72F8(count, frames, turns)` (floats): direction `[1, 3, 0, 2][f2i(4 turns / frames * count) % 4]`.
pub fn curve_a<E: FieldEnv>(env: &mut E, a: u32, b: u32, c: u32) -> u16 {
    let x = fmul(env, F_FOUR, c);
    let y = fdiv(env, x, b);
    let z = fmul(env, y, a);
    dir_of(&DIR_TABLE_A, z)
}

/// C: `FUN_021C734C(count, frames, turns)`: direction `[..][f2i(a * (4 turns / frames^2 * a)) % 4]` (accelerating).
pub fn curve_b<E: FieldEnv>(env: &mut E, a: u32, b: u32, c: u32) -> u16 {
    let x = fmul(env, F_FOUR, c);
    let y = fmul(env, b, b);
    let z = fdiv(env, x, y);
    let u = fmul(env, z, a);
    let v = fmul(env, a, u);
    dir_of(&DIR_TABLE_B, v)
}

/// C: `FUN_021C73B4(count, frames, turns)`: direction `[..][f2i(d2f(0209C5E0(f2d(16 turns^2 / frames * count)))) % 4]`
/// (the double helper is a square root by its use; not checked here).
pub fn curve_c<E: FieldEnv64>(env: &mut E, a: u32, b: u32, c: u32) -> u16 {
    let x = fmul(env, F_FOUR, c);
    let y = fmul(env, x, x);
    let z = fdiv(env, y, b);
    let u = fmul(env, z, a);
    let d = env.call64(F2D, &[u]);
    let s = env.call64(D_FN, &[d as u32, (d >> 32) as u32]);
    let f = env.callv(D2F, &[s as u32, (s >> 32) as u32]);
    dir_of(&DIR_TABLE_C, f)
}

/// The spin procs' common step: set the curve's direction, count + 1; 1 when the count passed `frames`.
fn spin_step<E: FieldEnv64>(env: &mut E, w: u32, curve: u32) -> bool {
    let obj = env.r32(w + 4);
    let a = u2f(u32::from(env.r16(w + 8)));
    let b = u2f(u32::from(env.r16(w + 0xA)));
    let c = u2f(u32::from(env.r8(w + 0xC)));
    let d = match curve {
        0 => curve_a(env, a, b, c),
        1 => curve_b(env, a, b, c),
        _ => curve_c(env, a, b, c),
    };
    env.callv(OBJ_SET_DRAW_DIR, &[obj, u32::from(d)]);
    let n = env.r16(w + 8).wrapping_add(1);
    env.w16(w + 8, n);
    env.r16(w + 0xA) < env.r16(w + 8)
}

/// C: `FUN_021C7418(work)` (curve [`curve_a`]) / `FUN_021C748C` ([`curve_b`]): spin proc; after `frames` steps the
/// OBJ gets its start direction back (0..3; another value gives 0).
pub fn spin_proc<E: FieldEnv64>(env: &mut E, w: u32, curve: u32) -> u32 {
    if !spin_step(env, w, curve) {
        return 0;
    }
    let obj = env.r32(w + 4);
    let d = match env.r16(w + 0xE) {
        d @ 0..=3 => u32::from(d),
        _ => 0,
    };
    env.callv(OBJ_SET_DRAW_DIR, &[obj, d]);
    1
}

/// C: `FUN_021C7500(work)`: spin proc with [`curve_c`]; at the end the OBJ's direction becomes the work's end dir.
pub fn spin_proc_end<E: FieldEnv64>(env: &mut E, w: u32) -> u32 {
    if !spin_step(env, w, 2) {
        return 0;
    }
    let obj = env.r32(w + 4);
    let d = u32::from(env.r16(w + 0xE));
    env.callv(OBJ_SET_DIR, &[obj, d]);
    1
}

/// C: `FUN_021C754C(fm, frames, &delta)` (proc [`move_proc`]) / `FUN_021C7AE4` (proc [`campos_proc`]): a move task;
/// work 0x28 `{u8 seq, fm, count, frames, start (0x10), delta (0x1C)}`.
pub fn move_new<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, delta: u32, proc_: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x28, proc_]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w8(w, 0);
    env.w32(w + 4, fm);
    env.w32(w + 8, 0);
    env.w32(w + 0xC, frames);
    copy_vec(env, w + 0x1C, delta);
    t
}

/// The move procs' seq 1: count + 1, `i = round(4096 * count / frames)`, `out = start * (4096 - i) + delta * i`
/// (fx32 scale `020520A4`, add `0207C9D0`), handed to `set(out)`; 1 when count >= frames.
fn move_blend<E: FieldEnv>(env: &mut E, w: u32, set: &mut dyn FnMut(&mut E, u32)) -> u32 {
    let cnt = env.r32(w + 8).wrapping_add(1);
    env.w32(w + 8, cnt);
    let n = env.r32(w + 0xC);
    let q = fdiv(env, i2f(cnt as i32), i2f(n as i32));
    let gt = fcmpgt(q, 0) != 0;
    let q2 = fdiv(env, i2f(cnt as i32), i2f(n as i32));
    let p = fmul(env, F_4096, q2);
    let i = f2i(if gt { fadd(F_HALF, p) } else { fsub(p, F_HALF) }) as u32;
    let a = env.mem().push_local(12);
    let b = env.mem().push_local(12);
    let o = env.mem().push_local(12);
    env.call(
        0x0205_20A4,
        &mut [
            CArg::In(w + 0x10, 12),
            CArg::V(0x1000u32.wrapping_sub(i)),
            CArg::Out(a, 12),
        ],
    );
    env.call(
        0x0205_20A4,
        &mut [CArg::In(w + 0x1C, 12), CArg::V(i), CArg::Out(b, 12)],
    );
    env.call(
        0x0207_C9D0,
        &mut [CArg::In(a, 12), CArg::In(b, 12), CArg::Out(o, 12)],
    );
    set(env, o);
    env.mem().pop_local(o);
    env.mem().pop_local(b);
    env.mem().pop_local(a);
    ((env.r32(w + 0xC) as i32) <= (env.r32(w + 8) as i32)) as u32
}

/// C: `FUN_021C7584(work)`: player move proc: seq 0 reads the player position into the work; seq 1 moves the
/// player (`021A3044`) and updates its OBJ (`02164010`).
pub fn move_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let fm = env.r32(w + 4);
    let player = env.callv(PLAYER, &[fm]);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    match env.r8(w) {
        0 => {
            env.call(0x021A_3038, &mut [CArg::V(player), CArg::Out(w + 0x10, 12)]);
            let s = env.r8(w).wrapping_add(1);
            env.w8(w, s);
            0
        }
        1 => move_blend(env, w, &mut |env: &mut E, o| {
            env.call(0x021A_3044, &mut [CArg::V(player), CArg::In(o, 12)]);
            env.callv(0x0216_4010, &[obj]);
        }),
        _ => 0,
    }
}

/// C: `FUN_021C7684(fm, obj, frames, amplitude)`: OBJ shake task (proc [`shake_proc`]); work 0x14 `{u8 seq, fm,
/// u16 count, u16 frames, i2f(amplitude), obj}`.
pub fn shake_new<E: FieldEnv>(env: &mut E, fm: u32, obj: u32, frames: u32, amp: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x14, 0x021C_7731]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w8(w, 0);
    env.w32(w + 4, fm);
    env.w16(w + 8, 0);
    env.w16(w + 0xA, frames as u16);
    env.w32(w + 0xC, i2f(amp as i32));
    env.w32(w + 0x10, obj);
    t
}

/// C: `FUN_021C76C4(&vec, count, frames, amplitude)` (floats): `vec = {0, round(4096 * (amp - count * (amp /
/// frames^2) * count)), 0}` (a parabola: amp at 0, 0 at count = frames).
pub fn shake_calc<E: FieldEnv>(env: &mut E, out: u32, a: u32, b: u32, c: u32) {
    let x = fmul(env, b, b);
    let y = fdiv(env, c, x);
    let z = fmul(env, y, a);
    let u = fmul(env, a, z);
    let v = fsub(c, u);
    env.w32(out, 0);
    let r = round_fx(env, v);
    env.w32(out + 4, r as u32);
    env.w32(out + 8, 0);
}

/// C: `FUN_021C7730(work)`: shake proc: offsets the OBJ (`0216D8C0`) by [`shake_calc`] each frame; sound 0x681 at
/// the start, 0x682 and `021AC528(obj, 02188CD0(fm))` at the end.
pub fn shake_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let fm = env.r32(w + 4);
    env.callv(PLAYER, &[fm]);
    let obj = env.r32(w + 0x10);
    let seq = env.r8(w);
    if seq > 1 {
        return 0;
    }
    if seq == 1 {
        let c = env.r16(w + 8).wrapping_add(1);
        env.w16(w + 8, c);
    }
    let a = u2f(u32::from(env.r16(w + 8)));
    let b = u2f(u32::from(env.r16(w + 0xA)));
    let amp = env.r32(w + 0xC);
    let l = env.mem().push_local(12);
    shake_calc(env, l, a, b, amp);
    env.call(0x0216_D8C0, &mut [CArg::V(obj), CArg::In(l, 12)]);
    env.mem().pop_local(l);
    if seq == 0 {
        env.callv(PLAY_SE, &[0x681]);
        env.w8(w, 1);
        return 0;
    }
    if env.r16(w + 0xA) > env.r16(w + 8) {
        return 0;
    }
    let fm = env.r32(w + 4);
    let x = env.callv(FM_2CD0, &[fm]);
    env.callv(0x021A_C528, &[obj, x]);
    env.callv(PLAY_SE, &[0x682]);
    1
}

/// C: `FUN_021C77D8(fm, frames)`: wait task (proc [`wait_proc`]); work 4 `{u16 count, u16 frames}`.
pub fn wait_new<E: FieldEnv>(env: &mut E, fm: u32, frames: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 4, 0x021C_77FD]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w16(w, 0);
    env.w16(w + 2, frames as u16);
    t
}

/// C: `FUN_021C77FC(work)`: count + 1; done when count >= frames.
pub fn wait_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let c = env.r16(w).wrapping_add(1);
    env.w16(w, c);
    (env.r16(w + 2) <= env.r16(w)) as u32
}

/// C: `FUN_021C7814(fm, a, b, c, d)`: deferred call task (proc [`call_proc`]); work 0x14 `{fm, a, b, c, d}`.
pub fn call_new<E: FieldEnv>(env: &mut E, fm: u32, a: u32, b: u32, c: u32, d: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x14, 0x021C_7845]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w32(w, fm);
    env.w32(w + 4, a);
    env.w32(w + 8, b);
    env.w32(w + 0xC, c);
    env.w32(w + 0x10, d);
    t
}

/// C: `FUN_021C7844(work)`: `0204E720(a, b, c, d)`; done at once.
pub fn call_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let a = env.r32(w + 4);
    let b = env.r32(w + 8);
    let c = env.r32(w + 0xC);
    let d = env.r32(w + 0x10);
    env.callv(0x0204_E720, &[a, b, c, d]);
    1
}

/// C: `FUN_021C7858(fm, frames, &delta)`: [`offset_new`] (proc `021C7A20`) for the player's OBJ.
pub fn offset_new_player<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, delta: u32) -> u32 {
    let player = env.callv(PLAYER, &[fm]);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    offset_new(env, fm, frames, delta, obj, 0x021C_7A21)
}

/// C: `FUN_021C7878(fm, frames, &delta, obj)` (proc [`offset_proc`] `021C7A20`) / `FUN_021C78DC` (proc `021C7A7C`):
/// OBJ offset task; work 0x2C `{u8 seq, fm, reverse, u16 count, u16 frames, delta (0x10), base offset (0x1C), obj}`;
/// a negative `frames` runs backwards over `-frames`.
pub fn offset_new<E: FieldEnv>(
    env: &mut E,
    fm: u32,
    frames: u32,
    delta: u32,
    obj: u32,
    proc_: u32,
) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x2C, proc_]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w8(w, 0);
    env.w32(w + 4, fm);
    env.w32(w + 0x28, obj);
    env.call(0x0216_D88C, &mut [CArg::V(obj), CArg::Out(w + 0x1C, 12)]);
    if (frames as i32) >= 0 {
        env.w32(w + 8, 0);
        env.w16(w + 0xC, 0);
        env.w16(w + 0xE, frames as u16);
    } else {
        env.w32(w + 8, 1);
        env.w16(w + 0xC, 0);
        env.w16(w + 0xE, frames.wrapping_neg() as u16);
    }
    copy_vec(env, w + 0x10, delta);
    t
}

/// C: `FUN_021C7940(&out, count, &delta, frames)`: `out = FX_Mul(delta, round(4096 * count / frames))` per axis.
pub fn lerp<E: FieldEnv>(env: &mut E, out: u32, cnt: u32, delta: u32, frames: u32) {
    let q = fdiv(env, u2f(cnt), u2f(frames));
    let gt = fcmpgt(q, 0) != 0;
    let q2 = fdiv(env, u2f(cnt), u2f(frames));
    let p = fmul(env, F_4096, q2);
    let s = f2i(if gt { fadd(F_HALF, p) } else { fsub(p, F_HALF) });
    for k in 0..3 {
        let v = env.r32(delta + 4 * k);
        env.w32(out + 4 * k, fx_mul(v, s));
    }
}

/// C: `FUN_021C7A20(work)` (`add_base` false) / `FUN_021C7A7C` (true): offset proc (seq 0 only): count + 1; the
/// offset is `delta * count / frames` (reverse: `(frames - count) / frames`; another mode leaves the zeroed local),
/// set with `0216D8C0`, or added to the base offset (`0207C9D0`) and set with `0216D89C`; done when count >= frames.
pub fn offset_proc<E: FieldEnv>(env: &mut E, w: u32, add_base: bool) -> u32 {
    let seq = env.r8(w);
    let obj = env.r32(w + 0x28);
    if seq != 0 {
        return 0;
    }
    let c = env.r16(w + 0xC).wrapping_add(1);
    env.w16(w + 0xC, c);
    let l = env.mem().push_local(12);
    match env.r32(w + 8) {
        0 => {
            let cnt = u32::from(env.r16(w + 0xC));
            let frames = u32::from(env.r16(w + 0xE));
            lerp(env, l, cnt, w + 0x10, frames);
        }
        1 => {
            let frames = u32::from(env.r16(w + 0xE));
            let cnt = frames.wrapping_sub(u32::from(env.r16(w + 0xC))) & 0xFFFF;
            lerp(env, l, cnt, w + 0x10, frames);
        }
        _ => {}
    }
    if add_base {
        env.call(
            0x0207_C9D0,
            &mut [CArg::In(l, 12), CArg::In(w + 0x1C, 12), CArg::Out(l, 12)],
        );
        env.call(0x0216_D89C, &mut [CArg::V(obj), CArg::In(l, 12)]);
    } else {
        env.call(0x0216_D8C0, &mut [CArg::V(obj), CArg::In(l, 12)]);
    }
    env.mem().pop_local(l);
    (env.r16(w + 0xE) <= env.r16(w + 0xC)) as u32
}

/// C: `FUN_021C7B1C(work)`: camera target move proc: seq 0 reads the target (`0218ECC4`), seq 1 sets it
/// (`0218ECD4`) as [`move_proc`] does for the player.
pub fn campos_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let fm = env.r32(w + 4);
    let cam = env.callv(CAMERA, &[fm]);
    match env.r8(w) {
        0 => {
            env.call(0x0218_ECC4, &mut [CArg::V(cam), CArg::Out(w + 0x10, 12)]);
            let s = env.r8(w).wrapping_add(1);
            env.w8(w, s);
            0
        }
        1 => move_blend(env, w, &mut |env: &mut E, o| {
            env.call(0x0218_ECD4, &mut [CArg::V(cam), CArg::In(o, 12)]);
        }),
        _ => 0,
    }
}

/// C: `FUN_021C7C10(fm, frames, &from, &to)`: camera position task (proc [`cam_proc`]); work 0x34 `{u8 seq, fm, u16
/// count, u16 frames, from (0xC), to (0x18), to - from (0x24, `0207CA00`), saved camera word (0x30)}`.
pub fn cam_new<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, from: u32, to: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    env.callv(CAMERA, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x34, 0x021C_7C75]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w8(w, 0);
    env.w32(w + 4, fm);
    env.w16(w + 8, 0);
    env.w16(w + 0xA, frames as u16);
    copy_vec(env, w + 0xC, from);
    copy_vec(env, w + 0x18, to);
    env.call(
        0x0207_CA00,
        &mut [
            CArg::In(w + 0x18, 12),
            CArg::In(w + 0xC, 12),
            CArg::Out(w + 0x24, 12),
        ],
    );
    t
}

/// C: `FUN_021C7C74(work)`: seq 0 saves and clears the camera word; seq 1 sets the position `from + (to - from) *
/// round(4096 * count / frames)` (`0207CEE4`, `0218EC90`); at the end the camera word comes back.
pub fn cam_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let fm = env.r32(w + 4);
    let cam = env.callv(CAMERA, &[fm]);
    match env.r8(w) {
        0 => {
            let v = env.callv(CAM_WORD_GET, &[cam]);
            env.w32(w + 0x30, v);
            env.callv(CAM_WORD_SET, &[cam, 0]);
            let s = env.r8(w).wrapping_add(1);
            env.w8(w, s);
            0
        }
        1 => {
            let c = env.r16(w + 8).wrapping_add(1);
            env.w16(w + 8, c);
            let cnt = u2f(u32::from(env.r16(w + 8)));
            let frames = u2f(u32::from(env.r16(w + 0xA)));
            let q = fdiv(env, cnt, frames);
            let i = round_fx(env, q) as u32;
            let l = env.mem().push_local(12);
            env.call(
                0x0207_CEE4,
                &mut [
                    CArg::V(i),
                    CArg::In(w + 0x24, 12),
                    CArg::In(w + 0xC, 12),
                    CArg::Out(l, 12),
                ],
            );
            env.call(0x0218_EC90, &mut [CArg::V(cam), CArg::In(l, 12)]);
            env.mem().pop_local(l);
            if env.r16(w + 0xA) > env.r16(w + 8) {
                return 0;
            }
            let saved = env.r32(w + 0x30);
            env.callv(CAM_WORD_SET, &[cam, saved]);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_021C7D28(fm, x, frames, period)` (`x` unused): periodic task (proc [`periodic_proc`]); work 0x14 `{fm,
/// 02188CD0(fm), player OBJ, u8 stop, u16 count, u16 frames, u16 period}`.
pub fn periodic_new<E: FieldEnv>(env: &mut E, fm: u32, frames: u32, period: u32) -> u32 {
    let heap = env.callv(HEAP_ID, &[fm]);
    let player = env.callv(PLAYER, &[fm]);
    let t = env.callv(TASK_NEW, &[heap, 0x14, 0x021C_7D79]);
    let w = env.callv(TASK_WORK, &[t]);
    env.w32(w, fm);
    let x = env.callv(FM_2CD0, &[fm]);
    env.w32(w + 4, x);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    env.w32(w + 8, obj);
    env.w8(w + 0xC, 0);
    env.w16(w + 0xE, 0);
    env.w16(w + 0x10, frames as u16);
    env.w16(w + 0x12, period as u16);
    t
}

/// C: `FUN_021C7D78(work)`: unless stopped, every `period` frames (count % period == 0) `021ADD7C(obj, x)`; count + 1;
/// done when the old count reached `frames`.
pub fn periodic_proc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    if env.r8(w + 0xC) != 0 {
        return 0;
    }
    let c = u32::from(env.r16(w + 0xE));
    let p = u32::from(env.r16(w + 0x12));
    // A period of 0: the runtime divide (0209C0A4) returns remainder 0 for a zero divisor, so the
    // callback runs every frame (V-0034, ROM-confirmed by the validator).
    let rem = if p == 0 { 0 } else { c % p };
    if rem == 0 {
        let obj = env.r32(w + 8);
        let x = env.r32(w + 4);
        env.callv(0x021A_DD7C, &[obj, x]);
    }
    let old = env.r16(w + 0xE);
    env.w16(w + 0xE, old.wrapping_add(1));
    (env.r16(w + 0x10) <= old) as u32
}

#[cfg(test)]
mod tests;
