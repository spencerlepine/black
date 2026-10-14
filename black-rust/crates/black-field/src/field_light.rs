//! `field_light.c` (overlay 21, `0x021A03D0..0x021A0CF4`, F1 row 21.15; F11): the time-of-day light of the field. A
//! LIGHT holds a keyframe table (ARC 0x3D, up to 16 keyframes of 4 directional lights with colours, the material
//! colours, the fog and clear colours, each at a time of day of the season), follows the clock (a 60-frame cross-fade to
//! the next keyframe when the time passes it), runs fade-to-colour requests of the weather (out, optionally back), and
//! uploads the current light set.
//!
//! Spec: `tools/rom-diff/specs/black-field/field_light.spec` (its header has the LIGHT, KEYFRAME and INTERPOLATOR
//! layouts). Verified by `tools/check-fn.sh black-field field_light --per-function`. D3: the upload ([`upload`]) and the
//! FX vector / matrix routines are reached through [`FieldEnv::call`] by address (the G3 light / material / clear-colour
//! calls, the field fog's colour, the ARM9 FX library); the keyframe logic, colour ramps and the rotation angle are ported.
//! `FX_SinCosTable_` is computed ([`fx_sin`], [`crate::fieldmap::fx_cos`]).

use crate::enc_pokeset::u32_divmod;
use crate::fieldmap::fx_cos;
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_03D0,
    0x021A_0448,
    0x021A_0458,
    0x021A_0568,
    0x021A_0590,
    0x021A_05D8,
    0x021A_0618,
    0x021A_063C,
    0x021A_065C,
    0x021A_066C,
    0x021A_0734,
    0x021A_0748,
    0x021A_07B0,
    0x021A_07B4,
    0x021A_07F8,
    0x021A_0818,
    0x021A_0860,
    0x021A_08CC,
    0x021A_0918,
    0x021A_0974,
    0x021A_0980,
    0x021A_0A50,
    0x021A_0B14,
    0x021A_0B34,
    0x021A_0B4C,
    0x021A_0C50,
    0x021A_0C60,
    0x021A_0C74,
    0x021A_0CE4,
];

/// Address of the string `"field_light.c"` (the file argument of the allocation in [`new`]).
pub const FILE_FIELD_LIGHT: u32 = 0x021D_DE38;
/// Size of the LIGHT object.
pub const LIGHT_SIZE: u32 = 0x490;
/// The archive of the keyframe tables.
pub const ARC_LIGHT: u32 = 0x3D;
/// Size of a keyframe (and of the current light).
pub const KEYFRAME_SIZE: u32 = 0x34;
/// Seconds of a day (the wrap of the keyframe times).
pub const DAY_SECONDS: u32 = 86400;
/// Frames of the clock-driven cross-fade.
pub const CROSSFADE_FRAMES: u32 = 60;
/// "No way back" frames of a fade request.
pub const NO_BACK: u32 = 0xFFFF;

/// LIGHT fields.
pub const L_SEASON: u32 = 0x2;
pub const L_G3D: u32 = 0x4;
pub const L_FOG: u32 = 0x8;
pub const L_COUNT: u32 = 0xC;
pub const L_KEYFRAMES: u32 = 0x10;
pub const L_INDEX: u32 = 0x350;
pub const L_MEMBER: u32 = 0x354;
pub const L_CURRENT: u32 = 0x356;
pub const L_ENABLED: u32 = 0x38A;
pub const L_DIRTY: u32 = 0x38C;
pub const L_INTERP: u32 = 0x390;
pub const L_REQUEST: u32 = 0x484;
pub const L_NOW: u32 = 0x48C;
/// INTERPOLATOR fields: frame / frames.
pub const I_FRAME: u32 = 0xF0;
pub const I_FRAMES: u32 = 0xF2;

/// C: `Heap_AllocDebug` / `Heap_Free`.
const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
/// C: `FUN_020490A8(arc, member)` the size of an archive member, `FUN_02048F4C(dst, arc, member)` load it.
const ARC_SIZE: u32 = 0x0204_90A8;
const ARC_LOAD: u32 = 0x0204_8F4C;
/// C: `FUN_02011400(season, zone)`: the hour of a time zone in a season (ARM9 main, a ROM table).
const ZONE_HOUR: u32 = 0x0201_1400;
/// ARM9 FX library: `VEC_Fx16Normalize(src, dst)`, `VEC_Fx16CrossProduct(a, b, out)`, `VEC_Fx16DotProduct(a, b)`,
/// `FX_AcosIdx(x)`, `MTX_RotAxis33(out, axis, sin, cos)`, `MTX_MultVec33(v, m, out)`, `VEC_Normalize(src, dst)`.
const VEC16_NORMALIZE: u32 = 0x0207_CDB8;
const VEC16_CROSS: u32 = 0x0207_CB68;
const VEC16_DOT: u32 = 0x0207_CAA4;
const ACOS_IDX: u32 = 0x0207_D184;
const MTX_ROT_AXIS33: u32 = 0x0207_B65C;
const MTX_MULT_VEC33: u32 = 0x0207_BC08;
const VEC_NORMALIZE: u32 = 0x0207_CCA4;
/// The upload: G3D global state light vector / colour, `G3_MaterialColorDiffAmb`, `G3_MaterialColorSpecEmi`, the field
/// fog colour (`FUN_021A01C4`, field_fog.c), `G3X_SetClearColor`.
const G3D_LIGHT_VECTOR: u32 = 0x0204_8CA4;
const G3D_LIGHT_COLOR: u32 = 0x0204_8CDC;
const MATERIAL_DIFF_AMB: u32 = 0x0206_AA8C;
const MATERIAL_SPEC_EMI: u32 = 0x0206_AAB0;
const FOG_COLOR: u32 = 0x021A_01C4;
const CLEAR_COLOR: u32 = 0x0207_DD24;

/// `FX_SinCosTable_` sine entry `i` (ROM main `0x020A1B38`, 4096 `(sin, cos)` pairs of fx16): `round(4096 * sin(2 pi i /
/// 4096))`, computed like [`fx_cos`].
pub fn fx_sin(i: u32) -> i16 {
    (((i as f64) * std::f64::consts::TAU / 4096.0).sin() * 4096.0).round() as i16
}

/// C: `_s32_div_f` (`FUN_0209C0A4`, ARM9 library): signed quotient truncating toward zero; a zero divisor returns the
/// dividend.
fn s32_div(n: i32, d: i32) -> i32 {
    if d == 0 {
        n
    } else {
        n.wrapping_div(d)
    }
}

/// Address of keyframe `k` of LIGHT `l`.
fn keyframe(l: u32, k: u32) -> u32 {
    l.wrapping_add(L_KEYFRAMES)
        .wrapping_add(KEYFRAME_SIZE.wrapping_mul(k))
}

/// C: `FUN_021A03D0(member, now, season, fog, g3d, heap)`: a new LIGHT (0x490 bytes on `heap`) for `season` with the
/// default table `member` of ARC 0x3D; the keyframe of `now` becomes the current light; upload enabled and pending.
pub fn new<E: FieldEnv>(
    env: &mut E,
    member: u32,
    now: u32,
    season: u32,
    fog: u32,
    g3d: u32,
    heap: u32,
) -> u32 {
    let l = env.callv(
        HEAP_ALLOC,
        &[heap & 0xFFFF, LIGHT_SIZE, 1, FILE_FIELD_LIGHT, 0x167],
    );
    env.w16(l + L_SEASON, season as u16);
    load_default(env, l, member);
    env.w16(l + L_ENABLED, 1);
    let k = keyframe_at(env, l, now);
    env.w32(l + L_INDEX, k);
    let k = env.r32(l + L_INDEX);
    env.mem().copy(l + L_CURRENT, keyframe(l, k), KEYFRAME_SIZE);
    env.w32(l + L_DIRTY, 1);
    env.w32(l + L_FOG, fog);
    env.w32(l + L_G3D, g3d);
    l
}

/// C: `FUN_021A0448(l)`: free the LIGHT.
pub fn delete<E: FieldEnv>(env: &mut E, l: u32) {
    nop();
    env.callv(HEAP_FREE, &[l]);
}

/// C: `FUN_021A0458(l, now)`: one frame. Mode 0 follows the clock: unless `time(k - 1) <= now < time(k)` (for `k = 0` the
/// previous keyframe is the last one when its time is not after `now`, and then the current one counts from the next
/// day and nothing changes) the next keyframe becomes current and a 60-frame cross-fade to it starts. Mode 1 runs the
/// fade request and returns to mode 0 when it is done. A running interpolator then steps into the current light (upload
/// pending); the clock value is kept.
pub fn update<E: FieldEnv>(env: &mut E, l: u32, now: u32) {
    if env.r32(l + L_COUNT) == 0 {
        return;
    }
    let mode = env.r16(l);
    if mode == 0 {
        let k = env.r32(l + L_INDEX);
        let season = env.r16(l + L_SEASON) as u32;
        let mut t = keyframe_time(env, season, keyframe(l, k));
        let k = env.r32(l + L_INDEX);
        let mut wrap = false;
        let prev;
        if (k.wrapping_sub(1) as i32) < 0 {
            let c = env.r32(l + L_COUNT);
            let season = env.r16(l + L_SEASON) as u32;
            let tp = keyframe_time(env, season, keyframe(l, c.wrapping_sub(1)));
            if (tp as i32) > (now as i32) {
                prev = 0;
            } else {
                prev = tp;
                wrap = true;
                t = t.wrapping_add(DAY_SECONDS);
            }
        } else {
            let season = env.r16(l + L_SEASON) as u32;
            prev = keyframe_time(env, season, keyframe(l, k.wrapping_sub(1)));
        }
        let inside = (prev as i32) <= (now as i32) && (t as i32) > (now as i32);
        if !inside && !wrap {
            let k = env.r32(l + L_INDEX);
            let c = env.r32(l + L_COUNT);
            let k = u32_divmod(k.wrapping_add(1), c).1;
            env.w32(l + L_INDEX, k);
            crossfade60(env, l + L_INTERP, l + L_CURRENT, keyframe(l, k));
        }
    } else if mode == 1 {
        let k = env.r32(l + L_INDEX);
        if request_step(
            env,
            l + L_REQUEST,
            l + L_INTERP,
            l + L_CURRENT,
            keyframe(l, k),
        ) != 0
        {
            env.w16(l, 0);
        }
    }
    if running(env, l + L_INTERP) != 0 {
        step(env, l + L_INTERP);
        light_at(env, l + L_INTERP, l + L_CURRENT);
        env.w32(l + L_DIRTY, 1);
    }
    env.w32(l + L_NOW, now);
}

/// C: `FUN_021A0568(l, force)`: upload the light when it changed (or `force`), clear the pending flag.
pub fn upload_if_dirty<E: FieldEnv>(env: &mut E, l: u32, force: u32) {
    if env.r32(l + L_DIRTY) != 0 || force != 0 {
        let fog = env.r32(l + L_FOG);
        let g3d = env.r32(l + L_G3D);
        upload(env, l, fog, g3d);
        env.w32(l + L_DIRTY, 0);
    }
}

/// C: `FUN_021A0590(l, arc, member, frames)`: load another keyframe table and cross-fade to its keyframe of the last clock
/// value over `frames` frames.
pub fn change_table<E: FieldEnv>(env: &mut E, l: u32, arc: u32, member: u32, frames: u32) {
    nop();
    load(env, l, arc, member);
    let now = env.r32(l + L_NOW);
    let k = keyframe_at(env, l, now);
    env.w32(l + L_INDEX, k);
    crossfade(env, l + L_INTERP, l + L_CURRENT, keyframe(l, k), frames);
}

/// C: `FUN_021A05D8(l)`: reload the default table and cross-fade (60 frames) to its keyframe of the last clock value.
pub fn restore_table<E: FieldEnv>(env: &mut E, l: u32) {
    nop();
    let member = env.r16(l + L_MEMBER) as u32;
    load_default(env, l, member);
    let now = env.r32(l + L_NOW);
    let k = keyframe_at(env, l, now);
    env.w32(l + L_INDEX, k);
    crossfade60(env, l + L_INTERP, l + L_CURRENT, keyframe(l, k));
}

/// C: `FUN_021A0618(l, colour, out, back)`: fade to `colour` over `out` frames, then back to the clock's keyframe over
/// `back` frames ([`request_set`]); mode 1.
pub fn fade_request<E: FieldEnv>(env: &mut E, l: u32, colour: u32, out: u32, back: u32) {
    request_set(env, l + L_REQUEST, out & 0xFFFF, back & 0xFFFF, colour);
    env.w16(l, 1);
}

/// C: `FUN_021A063C(l, colour, out)`: fade to `colour` over `out` frames and stay ([`request_set_once`]); mode 1.
pub fn fade_request_once<E: FieldEnv>(env: &mut E, l: u32, colour: u32, out: u32) {
    request_set_once(env, l + L_REQUEST, out & 0xFFFF, colour);
    env.w16(l, 1);
}

/// C: `FUN_021A065C(l)`: a fade request is set ([`request_active`] of `l + 0x484`).
pub fn fade_active<E: FieldEnv>(env: &mut E, l: u32) -> u32 {
    request_active(env, l + L_REQUEST)
}

/// C: `FUN_021A066C(l, fog, g3d)`: upload the current light (when enabled): per light its direction and colour, or a zero
/// vector and colour when it is off; the material colours (diffuse + ambient with the vertex-colour flag, specular +
/// emission), the fog colour, the clear colour (alpha 31, depth 0x7FFF).
pub fn upload<E: FieldEnv>(env: &mut E, l: u32, fog: u32, g3d: u32) {
    if env.r16(l + L_ENABLED) == 0 {
        return;
    }
    let zero = [0u8; 6];
    for i in 0..4u32 {
        if env.r8(l + L_CURRENT + 4 + i) != 0 {
            env.call(
                G3D_LIGHT_VECTOR,
                &mut [
                    CArg::V(g3d),
                    CArg::V(i),
                    CArg::In(l + L_CURRENT + 0x10 + 6 * i, 6),
                ],
            );
            env.call(
                G3D_LIGHT_COLOR,
                &mut [
                    CArg::V(g3d),
                    CArg::V(i),
                    CArg::In(l + L_CURRENT + 8 + 2 * i, 2),
                ],
            );
        } else {
            env.call(
                G3D_LIGHT_VECTOR,
                &mut [CArg::V(g3d), CArg::V(i), CArg::InBuf(&zero)],
            );
            env.call(
                G3D_LIGHT_COLOR,
                &mut [CArg::V(g3d), CArg::V(i), CArg::InBuf(&zero[..2])],
            );
        }
    }
    let c = l + L_CURRENT;
    let (d, a) = (env.r16(c + 0x28) as u32, env.r16(c + 0x2A) as u32);
    env.callv(MATERIAL_DIFF_AMB, &[d, a, 1]);
    let (s, e) = (env.r16(c + 0x2C) as u32, env.r16(c + 0x2E) as u32);
    env.callv(MATERIAL_SPEC_EMI, &[s, e, 0]);
    let f = env.r16(c + 0x30) as u32;
    env.callv(FOG_COLOR, &[fog, f]);
    let k = env.r16(c + 0x32) as u32;
    env.callv(CLEAR_COLOR, &[k, 31, 0x7FFF, 0, 0]);
}

/// C: `FUN_021A0734(l, member)`: remember `member` as the default table and load it from ARC 0x3D.
pub fn load_default<E: FieldEnv>(env: &mut E, l: u32, member: u32) {
    env.w16(l + L_MEMBER, member as u16);
    load(env, l, ARC_LIGHT, member);
}

/// C: `FUN_021A0748(l, arc, member)`: load a keyframe table: the count is the member size / 0x34, the light directions
/// are normalised in place.
pub fn load<E: FieldEnv>(env: &mut E, l: u32, arc: u32, member: u32) {
    let size = env.callv(ARC_SIZE, &[arc, member]);
    env.w32(l + L_COUNT, u32_divmod(size, KEYFRAME_SIZE).0);
    env.call(
        ARC_LOAD,
        &mut [
            CArg::Out(l + L_KEYFRAMES, 16 * KEYFRAME_SIZE),
            CArg::V(arc),
            CArg::V(member),
        ],
    );
    let mut i = 0u32;
    while i < env.r32(l + L_COUNT) {
        for j in 0..4u32 {
            let v = keyframe(l, i) + 0x10 + 6 * j;
            env.call(VEC16_NORMALIZE, &mut [CArg::In(v, 6), CArg::Out(v, 6)]);
        }
        i += 1;
    }
}

/// C: `FUN_021A07B0()`: empty.
pub fn nop() {}

/// C: `FUN_021A07B4(l, now)`: the first keyframe `k` with `time(k - 1) <= now < time(k)` (signed; the time before the
/// first keyframe is 0), else 0.
pub fn keyframe_at<E: FieldEnv>(env: &mut E, l: u32, now: u32) -> u32 {
    let mut prev: u32 = 0;
    let mut i = 0u32;
    while i < env.r32(l + L_COUNT) {
        let season = env.r16(l + L_SEASON) as u32;
        let t = keyframe_time(env, season, keyframe(l, i));
        if (prev as i32) <= (now as i32) && (t as i32) > (now as i32) {
            return i;
        }
        prev = t;
        i += 1;
    }
    0
}

/// C: `FUN_021A07F8(season, kf)`: the time of a keyframe in seconds of the day, `hour(season, zone) * 3600 + minutes *
/// 60` (wrapping).
pub fn keyframe_time<E: FieldEnv>(env: &mut E, season: u32, kf: u32) -> u32 {
    let zone = env.r16(kf) as u32;
    let h = env.callv(ZONE_HOUR, &[season, zone]);
    let m = env.r16(kf + 2) as i16 as i32;
    h.wrapping_mul(3600).wrapping_add(m.wrapping_mul(60) as u32)
}

/// C: `FUN_021A0818(ramp, c0, c1)`: the RGB555 ramp `{r0, r1 - r0, g0, g1 - g0, b0, b1 - b0}` (u16 stores).
pub fn ramp_init<E: FieldEnv>(env: &mut E, ramp: u32, c0: u32, c1: u32) {
    let r0 = c0 & 0x1F;
    env.w16(ramp, r0 as u16);
    let r0 = env.r16(ramp) as u32;
    env.w16(ramp + 2, (c1 & 0x1F).wrapping_sub(r0) as u16);
    env.w16(ramp + 4, ((c0 & 0x3E0) >> 5) as u16);
    let g0 = env.r16(ramp + 4) as u32;
    env.w16(ramp + 6, ((c1 & 0x3E0) >> 5).wrapping_sub(g0) as u16);
    env.w16(ramp + 8, ((c0 & 0x7C00) >> 10) as u16);
    let b0 = env.r16(ramp + 8) as u32;
    env.w16(ramp + 10, ((c1 & 0x7C00) >> 10).wrapping_sub(b0) as u16);
}

/// C: `FUN_021A0860(ramp, f, n)`: the RGB555 colour at frame `f` of `n`: per channel `c0 + (c1 - c0) * f / n` (signed
/// 32-bit product and division, the sums truncated to s16 before the channels are or-ed together).
pub fn ramp_at<E: FieldEnv>(env: &mut E, ramp: u32, f: u32, n: u32) -> u32 {
    let ch = |env: &mut E, o: u32| -> i32 {
        let d = env.r16(ramp + o + 2) as i16 as i32;
        let q = s32_div(d.wrapping_mul(f as i32), n as i32) as i16 as i32;
        let c0 = env.r16(ramp + o) as i16 as i32;
        q.wrapping_add(c0) as i16 as i32
    };
    let r = ch(env, 0);
    let g = ch(env, 4);
    let b = ch(env, 8);
    ((g << 5) | r | (b << 10)) as u32 & 0xFFFF
}

/// C: `FUN_021A08CC(rot, a, b)`: the rotation from direction `a` to `b`: `rot+0` = `a` as a VecFx32, `rot+0xC` = the cross
/// product, `rot+0x18` = the angle (`FX_AcosIdx` of the dot product).
pub fn rot_init<E: FieldEnv>(env: &mut E, rot: u32, a: u32, b: u32) {
    let ay = env.r16(a + 2) as i16 as i32 as u32;
    let az = env.r16(a + 4) as i16 as i32 as u32;
    let ax = env.r16(a) as i16 as i32 as u32;
    env.w32(rot + 4, ay);
    env.w32(rot + 8, az);
    env.w32(rot, ax);
    let mut x = [0u8; 6];
    env.call(
        VEC16_CROSS,
        &mut [CArg::In(a, 6), CArg::In(b, 6), CArg::OutBuf(&mut x)],
    );
    let s = |k: usize| i16::from_le_bytes([x[k], x[k + 1]]) as i32 as u32;
    env.w32(rot + 0xC, s(0));
    env.w32(rot + 0x10, s(2));
    env.w32(rot + 0x14, s(4));
    let d = env.call(VEC16_DOT, &mut [CArg::In(a, 6), CArg::In(b, 6)]);
    let ang = env.callv(ACOS_IDX, &[d]);
    env.w16(rot + 0x18, ang as u16);
}

/// C: `FUN_021A0918(rot, f, n, out)`: the direction at frame `f` of `n`: the start vector rotated about the axis by
/// `angle * f / n` (signed division, `FX_SinCosTable_` index `(u16) angle >> 4`), normalised, stored as a VecFx16.
pub fn rot_at<E: FieldEnv>(env: &mut E, rot: u32, f: u32, n: u32, out: u32) {
    let ang = env.r16(rot + 0x18) as u32;
    let a = (s32_div(ang.wrapping_mul(f) as i32, n as i32) as u32 & 0xFFFF) >> 4;
    let (sin, cos) = (fx_sin(a) as i32 as u32, fx_cos(a) as i32 as u32);
    let mut m = [0u8; 0x24];
    env.call(
        MTX_ROT_AXIS33,
        &mut [
            CArg::OutBuf(&mut m),
            CArg::In(rot + 0xC, 12),
            CArg::V(sin),
            CArg::V(cos),
        ],
    );
    let mut v = [0u8; 12];
    env.call(
        MTX_MULT_VEC33,
        &mut [CArg::In(rot, 12), CArg::InBuf(&m), CArg::OutBuf(&mut v)],
    );
    let vin = v;
    env.call(
        VEC_NORMALIZE,
        &mut [CArg::InBuf(&vin), CArg::OutBuf(&mut v)],
    );
    for k in 0..3u32 {
        let o = 4 * k as usize;
        env.w16(out + 2 * k, u16::from_le_bytes([v[o], v[o + 1]]));
    }
}

/// C: `FUN_021A0974(i, cur, target)`: [`crossfade`] over 60 frames.
pub fn crossfade60<E: FieldEnv>(env: &mut E, i: u32, cur: u32, target: u32) {
    crossfade(env, i, cur, target, CROSSFADE_FRAMES);
}

/// C: `FUN_021A0980(i, cur, target, n)`: start a cross-fade of `n` frames from the light `cur` to the keyframe `target`:
/// the target's header and on flags, colour ramps cur -> target, the rotations of the light directions.
pub fn crossfade<E: FieldEnv>(env: &mut E, i: u32, cur: u32, target: u32, n: u32) {
    env.w16(i + I_FRAME, 0);
    env.w16(i + I_FRAMES, n as u16);
    let h = env.r16(target);
    env.w16(i, h);
    let h = env.r16(target + 2);
    env.w16(i + 2, h);
    for k in 0..4u32 {
        let (c0, c1) = (
            env.r16(cur + 8 + 2 * k) as u32,
            env.r16(target + 8 + 2 * k) as u32,
        );
        ramp_init(env, i + 8 + 12 * k, c0, c1);
        rot_init(
            env,
            i + 0x38 + 0x1C * k,
            cur + 0x10 + 6 * k,
            target + 0x10 + 6 * k,
        );
        let on = env.r8(target + 4 + k);
        env.w8(i + 4 + k, on);
    }
    for k in 0..6u32 {
        let (c0, c1) = (
            env.r16(cur + 0x28 + 2 * k) as u32,
            env.r16(target + 0x28 + 2 * k) as u32,
        );
        ramp_init(env, i + 0xA8 + 12 * k, c0, c1);
    }
}

/// C: `FUN_021A0A50(i, cur, colour, n)`: start a fade of `n` frames from the light `cur` to one colour on every ramp; the
/// directions stay (a rotation of each to itself), header and on flags from `cur`.
pub fn fade_to_colour<E: FieldEnv>(env: &mut E, i: u32, cur: u32, colour: u32, n: u32) {
    env.w16(i + I_FRAME, 0);
    env.w16(i + I_FRAMES, n as u16);
    let h = env.r16(cur);
    env.w16(i, h);
    let h = env.r16(cur + 2);
    env.w16(i + 2, h);
    for k in 0..4u32 {
        let c0 = env.r16(cur + 8 + 2 * k) as u32;
        ramp_init(env, i + 8 + 12 * k, c0, colour);
        rot_init(
            env,
            i + 0x38 + 0x1C * k,
            cur + 0x10 + 6 * k,
            cur + 0x10 + 6 * k,
        );
        let on = env.r8(cur + 4 + k);
        env.w8(i + 4 + k, on);
    }
    for k in 0..6u32 {
        let c0 = env.r16(cur + 0x28 + 2 * k) as u32;
        ramp_init(env, i + 0xA8 + 12 * k, c0, colour);
    }
}

/// C: `FUN_021A0B14(i)`: the next frame (up to the last).
pub fn step<E: FieldEnv>(env: &mut E, i: u32) {
    let f = env.r16(i + I_FRAME);
    if f < env.r16(i + I_FRAMES) {
        env.w16(i + I_FRAME, f + 1);
    }
}

/// C: `FUN_021A0B34(i)`: the interpolator has frames left.
pub fn running<E: FieldEnv>(env: &mut E, i: u32) -> u32 {
    (env.r16(i + I_FRAME) < env.r16(i + I_FRAMES)) as u32
}

/// C: `FUN_021A0B4C(i, out)`: the light at the interpolator's frame: header, light colours ([`ramp_at`]) and directions
/// ([`rot_at`]), on flags, the six other colours.
pub fn light_at<E: FieldEnv>(env: &mut E, i: u32, out: u32) {
    let h = env.r16(i);
    env.w16(out, h);
    let h = env.r16(i + 2);
    env.w16(out + 2, h);
    for k in 0..4u32 {
        let (f, n) = (env.r16(i + I_FRAME) as u32, env.r16(i + I_FRAMES) as u32);
        let c = ramp_at(env, i + 8 + 12 * k, f, n);
        env.w16(out + 8 + 2 * k, c as u16);
        let (f, n) = (env.r16(i + I_FRAME) as u32, env.r16(i + I_FRAMES) as u32);
        rot_at(env, i + 0x38 + 0x1C * k, f, n, out + 0x10 + 6 * k);
        let on = env.r8(i + 4 + k);
        env.w8(out + 4 + k, on);
    }
    for k in 0..6u32 {
        let (f, n) = (env.r16(i + I_FRAME) as u32, env.r16(i + I_FRAMES) as u32);
        let c = ramp_at(env, i + 0xA8 + 12 * k, f, n);
        env.w16(out + 0x28 + 2 * k, c as u16);
    }
}

/// C: `FUN_021A0C50(req, out, back, colour)`: request `{1, out, back, colour}`.
pub fn request_set<E: FieldEnv>(env: &mut E, req: u32, out: u32, back: u32, colour: u32) {
    env.w16(req, 1);
    env.w16(req + 2, out as u16);
    env.w16(req + 4, back as u16);
    env.w16(req + 6, colour as u16);
}

/// C: `FUN_021A0C60(req, out, colour)`: request `{1, out, 0xFFFF (no way back), colour}`.
pub fn request_set_once<E: FieldEnv>(env: &mut E, req: u32, out: u32, colour: u32) {
    env.w16(req + 2, out as u16);
    env.w16(req, 1);
    env.w16(req + 6, colour as u16);
    env.w16(req + 4, NO_BACK as u16);
}

/// C: `FUN_021A0C74(req, i, cur, target)`: one step of a fade request: state 1 starts the fade to the colour (state 2);
/// state 2 waits for it, then (with a way back) starts the cross-fade back to `target` (state 3); state 3 waits for that
/// (state 0). 1 only in state 0 (done).
pub fn request_step<E: FieldEnv>(env: &mut E, req: u32, i: u32, cur: u32, target: u32) -> u32 {
    let next = match env.r16(req) {
        0 => return 1,
        1 => {
            let (colour, out) = (env.r16(req + 6) as u32, env.r16(req + 2) as u32);
            fade_to_colour(env, i, cur, colour, out);
            2
        }
        2 => {
            if running(env, i) != 0 {
                return 0;
            }
            let back = env.r16(req + 4) as u32;
            if back != NO_BACK {
                crossfade(env, i, cur, target, back);
            }
            3
        }
        3 => {
            if running(env, i) != 0 {
                return 0;
            }
            0
        }
        _ => return 0,
    };
    env.w16(req, next);
    0
}

/// C: `FUN_021A0CE4(req)`: the request is set (state != 0).
pub fn request_active<E: FieldEnv>(env: &mut E, req: u32) -> u32 {
    (env.r16(req) != 0) as u32
}

#[cfg(test)]
mod tests;
