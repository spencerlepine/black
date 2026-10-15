//! `field_rail.c` part 3 (overlay 21, `0x021B9B7C..0x021BB088`; F3b): the procs of the rail data's three ROM tables: the
//! position procs (`fn(w, &pos)`, table ROM 0x021DDDA0: straight line, curve around the line record's centre, curve with a
//! free axis), the line-hit procs (`fn(i, r, &hit, &loc, &pos)`, ROM 0x021DDDAC: straight line, stepped line) and the camera
//! procs (`fn(r)`, ROM 0x021DDDB8: fixed / interpolated angles and distance, orbit, look-at, behind the player, ...).
//!
//! Camera records (0x24, the RAIL DATA's camera table): `+4` / `+8` u16 angles, `+0xC` distance or a vector, `+0x10..`
//! further values the procs interpret differently. The field camera itself (the system's `+0x10`, fieldmap camera
//! `0218E124` .. `0218F05C`), the RAIL getters of [`crate::rail`] / [`crate::rail_move`] and the NitroSDK vector / fx
//! functions are reached through [`FieldEnv::call`] (D3: the camera is driven, not rendered here). Stack vectors of the
//! original are [`Mem::push_local`](crate::hooks::Mem::push_local) windows; FX_Mul products are computed as the original
//! does; `FX_SinCosTable_` (ARM9 0x020A1B38) is read through the memory window. Verified by
//! `tools/check-fn.sh black-field rail_cam --per-function` (spec `tools/rom-diff/specs/black-field/rail_cam.spec`).

use crate::hooks::{CArg, FieldEnv};
use crate::rail::rd32;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_9B7C,
    0x021B_9C28,
    0x021B_9D60,
    0x021B_9E84,
    0x021B_9FC8,
    0x021B_A25C,
    0x021B_A2B4,
    0x021B_A310,
    0x021B_A348,
    0x021B_A494,
    0x021B_A4C4,
    0x021B_A4FC,
    0x021B_A550,
    0x021B_A6E8,
    0x021B_A7B0,
    0x021B_A7E0,
    0x021B_A8E0,
    0x021B_A9B4,
    0x021B_AA30,
    0x021B_AA9C,
    0x021B_AB30,
    0x021B_ABB4,
    0x021B_ABDC,
    0x021B_AC04,
    0x021B_AC74,
    0x021B_ACA8,
    0x021B_AE7C,
    0x021B_B014,
];

// crate::rail getters
const W_REC: u32 = 0x021B_82F4;
const W_ALONG: u32 = 0x021B_82F8;
const W_LEN: u32 = 0x021B_82FC;
const W_LAT: u32 = 0x021B_8300;
const W_WIDTH: u32 = 0x021B_8304;
const W_UNIT: u32 = 0x021B_8308;
const W_KIND: u32 = 0x021B_82F0;
const W_CAM: u32 = 0x021B_830C;
const W_START: u32 = 0x021B_833C;
const W_END: u32 = 0x021B_834C;
const W_LINE_REC: u32 = 0x021B_835C;
const W_POINT_CAM: u32 = 0x021B_836C;
const W_POS_NO_LAT: u32 = 0x021B_8314;
const W_POS_AT: u32 = 0x021B_8328;
const W_DIR_VEC: u32 = 0x021B_8050;
const W_SET_LOC: u32 = 0x021B_7EE4;
const W_POS: u32 = 0x021B_8028;
const R_CAMERA: u32 = 0x021B_821C;
const R_CURRENT: u32 = 0x021B_8220;
const R_POINT: u32 = 0x021B_8224;
const R_LINE: u32 = 0x021B_8230;
const R_LINE_REC: u32 = 0x021B_823C;
const R_UNIT: u32 = 0x021B_8248;
const R_MAIN: u32 = 0x021B_824C;
const R_CUR_POS: u32 = 0x021B_79D4;
const R_CLAMP: u32 = 0x021B_8250;
const R_LOC_POS: u32 = 0x021B_7D38;
const ROUND16: u32 = 0x021B_8378;
const INTERP: u32 = 0x021B_9B24;
// fieldmap camera (ov21)
const CAM_MODE: u32 = 0x0218_E124;
const CAM_E_C54: u32 = 0x0218_EC54;
const CAM_E_C60: u32 = 0x0218_EC60;
const CAM_SET_POS: u32 = 0x0218_EC90;
const CAM_SET_TARGET: u32 = 0x0218_ECF4;
const CAM_SET_VEC: u32 = 0x0218_ECD4;
const CAM_SET_A: u32 = 0x0218_ED34;
const CAM_SET_B: u32 = 0x0218_ED48;
const CAM_SET_DIST: u32 = 0x0218_ED58;
const CAM_F05C: u32 = 0x0218_F05C;
// NitroSDK
const VEC_ADD: u32 = 0x0207_C9D0;
const VEC_SUB: u32 = 0x0207_CA00;
const VEC_DOT: u32 = 0x0207_CA68;
const VEC_CROSS: u32 = 0x0207_CAE8;
const VEC_MAG: u32 = 0x0207_CBD0;
const VEC_NORMALIZE: u32 = 0x0207_CCA4;
const VEC_MULT_ADD: u32 = 0x0207_CEE4;
const VEC_DISTANCE: u32 = 0x0207_CF3C;
const VEC_FX16_MAG: u32 = 0x0207_CC30;
const FX_DIV: u32 = 0x0207_C768;
const FX_ATAN2_IDX: u32 = 0x0207_CFB8;
/// `FX_SinCosTable_` (ARM9 data): `{s16 sin, s16 cos}` per angle index.
pub const SIN_COS_TABLE: u32 = 0x020A_1B38;
/// The up vector (ROM 0x021D9A54 / 0x021D9A60).
const UP: [u32; 3] = [0, 0x1000, 0];

/// `FX_Mul(a, b)`: `((a * b) + 0x800) >> 12` on the 64-bit product, low 32 bits.
fn fx_mul(a: i32, b: i32) -> i32 {
    ((i64::from(a) * i64::from(b) + 0x800) >> 12) as i32
}

fn c1<E: FieldEnv>(env: &mut E, f: u32, a: u32) -> u32 {
    env.callv(f, &[a])
}

/// A block of stack locals of the original (`n` bytes, zeroed).
fn local<E: FieldEnv>(env: &mut E, n: u32) -> u32 {
    env.mem().push_local(n)
}

fn free<E: FieldEnv>(env: &mut E, a: u32) {
    env.mem().pop_local(a);
}

fn wv<E: FieldEnv>(env: &mut E, a: u32, v: [u32; 3]) {
    for (k, x) in v.iter().enumerate() {
        env.w32(a + 4 * k as u32, *x);
    }
}

fn rv<E: FieldEnv>(env: &mut E, a: u32) -> [i32; 3] {
    [
        env.r32(a) as i32,
        env.r32(a + 4) as i32,
        env.r32(a + 8) as i32,
    ]
}

fn vsub<E: FieldEnv>(env: &mut E, a: u32, b: u32, out: u32) {
    env.call(
        VEC_SUB,
        &mut [CArg::In(a, 12), CArg::In(b, 12), CArg::Out(out, 12)],
    );
}

fn vadd<E: FieldEnv>(env: &mut E, a: u32, b: u32, out: u32) {
    env.call(
        VEC_ADD,
        &mut [CArg::In(a, 12), CArg::In(b, 12), CArg::Out(out, 12)],
    );
}

fn vcross<E: FieldEnv>(env: &mut E, a: u32, b: u32, out: u32) {
    env.call(
        VEC_CROSS,
        &mut [CArg::In(a, 12), CArg::In(b, 12), CArg::Out(out, 12)],
    );
}

fn vnorm<E: FieldEnv>(env: &mut E, a: u32, out: u32) {
    env.call(VEC_NORMALIZE, &mut [CArg::In(a, 12), CArg::Out(out, 12)]);
}

fn vmag<E: FieldEnv>(env: &mut E, a: u32) -> i32 {
    env.call(VEC_MAG, &mut [CArg::In(a, 12)]) as i32
}

fn vmadd<E: FieldEnv>(env: &mut E, k: i32, a: u32, b: u32, out: u32) {
    env.call(
        VEC_MULT_ADD,
        &mut [
            CArg::V(k as u32),
            CArg::In(a, 12),
            CArg::In(b, 12),
            CArg::Out(out, 12),
        ],
    );
}

fn fx_div<E: FieldEnv>(env: &mut E, a: i32, b: i32) -> i32 {
    env.callv(FX_DIV, &[a as u32, b as u32]) as i32
}

/// `t` = along / length (fx32) of a RAIL WORK.
fn frac<E: FieldEnv>(env: &mut E, w: u32) -> i32 {
    let p = c1(env, W_ALONG, w);
    let l = c1(env, W_LEN, w);
    fx_div(env, p.wrapping_shl(12) as i32, l.wrapping_shl(12) as i32)
}

/// [`frac`] with the length read first (the order of the getter calls of 021BA348 / 021BACA8).
fn frac_len_first<E: FieldEnv>(env: &mut E, w: u32) -> i32 {
    let l = c1(env, W_LEN, w);
    let p = c1(env, W_ALONG, w);
    fx_div(env, p.wrapping_shl(12) as i32, l.wrapping_shl(12) as i32)
}

/// The lateral offset in fx32 (`FX_Mul(lat << 12, unit)`).
fn lat_fx<E: FieldEnv>(env: &mut E, w: u32) -> i32 {
    let l = c1(env, W_LAT, w);
    let u = c1(env, W_UNIT, w);
    fx_mul(l.wrapping_shl(12) as i32, u as i32)
}

/// C: `FUN_021B9B7C(w, out)`: position proc "straight": the point at `t` on the line from its start to its end point,
/// moved sideways by the lateral offset (the normalised line direction x up).
pub fn pos_line<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    c1(env, W_REC, w);
    let t = frac(env, w);
    let a = c1(env, W_START, w) + 0x30;
    let b = c1(env, W_END, w) + 0x30;
    let l = local(env, 36);
    let d = l + 24;
    let up = l + 12;
    let n = l;
    vsub(env, b, a, d);
    vmadd(env, t, d, a, out);
    wv(env, up, UP);
    let lat = c1(env, W_LAT, w);
    let unit = c1(env, W_UNIT, w);
    vcross(env, d, up, n);
    vnorm(env, n, n);
    let k = fx_mul(lat.wrapping_shl(12) as i32, unit as i32);
    vmadd(env, k, n, out, out);
    free(env, l);
}

/// C: `FUN_021B9C28(w, out)`: position proc "curve": around the line record's centre (its x / z; the height interpolated
/// between the end points), interpolating the directions to the end points (`021B9B24`), moved sideways.
pub fn pos_curve<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    c1(env, W_REC, w);
    let a = c1(env, W_START, w) + 0x30;
    let b = c1(env, W_END, w) + 0x30;
    let pos = c1(env, W_ALONG, w);
    let len = c1(env, W_LEN, w);
    let t = fx_div(
        env,
        pos.wrapping_shl(12) as i32,
        len.wrapping_shl(12) as i32,
    );
    let dy = (env.r32(b + 4) as i32).wrapping_sub(env.r32(a + 4) as i32);
    env.w32(out + 4, dy as u32);
    let m = fx_mul(dy, pos.wrapping_shl(12) as i32);
    let y = fx_div(env, m, len.wrapping_shl(12) as i32);
    env.w32(out + 4, y as u32);
    let y = y.wrapping_add(env.r32(a + 4) as i32);
    env.w32(out + 4, y as u32);
    let rec = c1(env, W_LINE_REC, w);
    let l = local(env, 96);
    let (v, e1, e2, f, g, h, c) = (l + 84, l + 72, l + 60, l + 36, l + 48, l + 24, l + 12);
    let rx = rd32(env, rec + 4);
    let ry = env.r32(out + 4);
    let rz = rd32(env, rec + 0xC);
    wv(env, v, [rx, ry, rz]);
    vsub(env, a, v, e1);
    vsub(env, b, v, e2);
    env.w32(e1 + 4, 0);
    env.w32(e2 + 4, 0);
    vsub(env, e2, e1, f);
    env.w32(f + 4, 0);
    vnorm(env, f, f);
    env.call(
        INTERP,
        &mut [
            CArg::Out(g, 12),
            CArg::In(e1, 12),
            CArg::In(e2, 12),
            CArg::V(t as u32),
        ],
    );
    vadd(env, v, g, out);
    let k = lat_fx(env, w);
    vnorm(env, g, h);
    vcross(env, h, f, c);
    let k = if (env.r32(c + 4) as i32) < 0 {
        k.wrapping_neg()
    } else {
        k
    };
    vmadd(env, k, h, out, out);
    free(env, l);
}

/// C: `FUN_021B9D60(w, out)`: position proc "curve, free axis": around the line record's centre point (x / y / z); a
/// direction with a zero x or z is replaced by (end - start) x up.
pub fn pos_curve3<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    c1(env, W_REC, w);
    let a = c1(env, W_START, w) + 0x30;
    let b = c1(env, W_END, w) + 0x30;
    let t = frac(env, w);
    let rec = c1(env, W_LINE_REC, w);
    let l = local(env, 100);
    let (v, e1, e2, f, g, h, c, up) = (
        l + 88,
        l + 76,
        l + 64,
        l + 40,
        l + 52,
        l + 28,
        l + 16,
        l + 4,
    );
    let rv3 = [rd32(env, rec + 4), rd32(env, rec + 8), rd32(env, rec + 0xC)];
    wv(env, v, rv3);
    vsub(env, a, v, e1);
    vsub(env, b, v, e2);
    vsub(env, e2, e1, f);
    vnorm(env, f, f);
    env.call(
        INTERP,
        &mut [
            CArg::Out(g, 12),
            CArg::In(e1, 12),
            CArg::In(e2, 12),
            CArg::V(t as u32),
        ],
    );
    vadd(env, v, g, out);
    let k = lat_fx(env, w);
    if env.r32(g) != 0 && env.r32(g + 8) != 0 {
        env.w32(g + 4, 0);
    } else {
        wv(env, up, UP);
        vsub(env, b, a, g);
        vcross(env, g, up, g);
    }
    vnorm(env, g, h);
    vcross(env, h, f, c);
    let k = if (env.r32(c + 4) as i32) < 0 {
        k.wrapping_neg()
    } else {
        k
    };
    vmadd(env, k, h, out, out);
    free(env, l);
}

/// The LOCATION of a hit: line `i`, kind 1, byte 3 = 3, `x` = ±round(d), `z` = round(z); clamped to the line, its
/// position to `pos`. Returns 1 when the clamp did not change it.
fn hit_loc<E: FieldEnv>(env: &mut E, r: u32, loc: u32, x: u32, neg: bool, z: Option<u32>) -> u32 {
    let rx = env.callv(ROUND16, &[x]);
    env.w16(loc + 4, if neg { rx.wrapping_neg() } else { rx } as u16);
    if let Some(z) = z {
        let rz = env.callv(ROUND16, &[z]);
        env.w16(loc + 6, rz as u16);
    }
    let t = local(env, 8);
    env.call(
        R_CLAMP,
        &mut [CArg::V(r), CArg::In(loc, 8), CArg::Out(t, 8)],
    );
    t
}

fn hit_finish<E: FieldEnv>(env: &mut E, r: u32, loc: u32, cl: CArg<'_>, t: u32, pos: u32) -> u32 {
    env.call(R_LOC_POS, &mut [CArg::V(r), cl, CArg::Out(pos, 12)]);
    let mut same = true;
    for i in 0..8 {
        if env.r8(loc + i) != env.r8(t + i) {
            same = false;
        }
    }
    free(env, t);
    same as u32
}

/// C: `FUN_021B9E84(i, r, hit, loc, pos)`: line-hit proc "straight": the hit point projected on line `i` (021BAE7C) gives a
/// LOCATION (x = the ground distance in units, z = along); accepted when within the width interpolated from the end
/// points' values; clamped to the line, its position to `pos`. Returns 1 when the clamped LOCATION is the computed one.
pub fn hit_line<E: FieldEnv>(env: &mut E, i: u32, r: u32, hit: u32, loc: u32, pos: u32) -> u32 {
    let ln = env.callv(R_LINE, &[r, i]);
    let i0 = env.r32(ln);
    let pa = env.callv(R_POINT, &[r, i0]);
    let i1 = env.r32(ln + 4);
    let pb = env.callv(R_POINT, &[r, i1]);
    let unit = c1(env, R_UNIT, r) as i32;
    let l = local(env, 64);
    let (p, q, side, u, v) = (l, l + 12, l + 16, l + 28, l + 40);
    if tri_hit(env, pa, pb, hit, p, q, side) == 0 {
        free(env, l);
        return 0;
    }
    let h = rv(env, hit);
    wv(env, u, [h[0] as u32, 0, h[2] as u32]);
    let pp = rv(env, p);
    wv(env, v, [pp[0] as u32, 0, pp[2] as u32]);
    let d = env.call(VEC_DISTANCE, &mut [CArg::In(u, 12), CArg::In(v, 12)]) as i32;
    let d = fx_div(env, d, unit) >> 12;
    let i3 = env.r32(ln + 0xC);
    env.callv(R_LINE_REC, &[r, i3]);
    let len = env.r32(ln + 0x14).wrapping_shl(4) as i32;
    let qv = env.r32(q) as i32;
    let z = len.wrapping_sub(fx_div(env, qv, unit) >> 12);
    let mut a = 0u32;
    let mut b = 0u32;
    for k in 0..4u32 {
        if i == env.r32(pa + 4 * k) {
            a = env.r32(pa + 4 * k + 0x20);
        }
        if i == env.r32(pb + 4 * k) {
            b = env.r32(pb + 4 * k + 0x20);
        }
    }
    // _u32_div_f answers the dividend for a zero divisor
    let num = b.wrapping_sub(a).wrapping_mul(z as u32);
    let m = num.checked_div(len as u32).unwrap_or(num);
    let neg = (env.r32(side + 4) as i32) < 0;
    free(env, l);
    if (d as u32) > a.wrapping_add(m).wrapping_add(16) {
        return 0;
    }
    env.w8(loc + 2, 1);
    env.w16(loc, i as u16);
    env.w8(loc + 3, 3);
    let t = hit_loc(env, r, loc, d as u32, neg, Some(z as u32));
    hit_finish(env, r, loc, CArg::In(t, 8), t, pos)
}

/// C: `FUN_021B9FC8(i, r, hit, loc, pos)`: line-hit proc "stepped": walks line `i` unit by unit (main RAIL WORK placed at
/// z = 1, 2, ..), projecting the hit on each segment; the first one within the width gives the LOCATION (x = the ground
/// distance), clamped, its position to `pos`. Returns 1 when the clamped LOCATION is the computed one.
pub fn hit_steps<E: FieldEnv>(env: &mut E, i: u32, r: u32, hit: u32, loc: u32, pos: u32) -> u32 {
    let ln = env.callv(R_LINE, &[r, i]);
    let i0 = env.r32(ln);
    let pa = env.callv(R_POINT, &[r, i0]);
    let i1 = env.r32(ln + 4);
    let pb = env.callv(R_POINT, &[r, i1]);
    let l = local(env, 0xC0);
    let (o1, o2, o3) = (l + 116, l + 52, l + 104);
    if tri_hit(env, pa, pb, hit, o1, o2, o3) == 0 {
        free(env, l);
        return 0;
    }
    let mut j = 1u32;
    env.w8(loc + 2, 1);
    env.w8(loc + 3, 3);
    env.w16(loc, i as u16);
    env.w16(loc + 4, 0);
    env.w16(loc + 6, 0);
    let w = c1(env, R_MAIN, r);
    env.call(W_SET_LOC, &mut [CArg::V(w), CArg::In(loc, 8)]);
    let n = c1(env, W_LEN, w) >> 4;
    let (cur, prev) = (l + 176, l + 164);
    env.call(W_POS, &mut [CArg::V(w), CArg::Out(cur, 12)]);
    let c = rv(env, cur);
    wv(env, prev, [c[0] as u32, c[1] as u32, c[2] as u32]);
    let unit = c1(env, R_UNIT, r) as i32;
    if n <= 1 {
        free(env, l);
        return 0;
    }
    let (d, e, nd, dn, en, cr, p, u, v) = (
        l + 152,
        l + 140,
        l + 128,
        l + 68,
        l + 56,
        l + 104,
        l + 116,
        l + 92,
        l + 80,
    );
    loop {
        env.w16(loc + 6, j as u16);
        env.call(W_SET_LOC, &mut [CArg::V(w), CArg::In(loc, 8)]);
        env.call(W_POS, &mut [CArg::V(w), CArg::Out(cur, 12)]);
        let wid = c1(env, W_WIDTH, w);
        vsub(env, prev, cur, d);
        vsub(env, hit, cur, e);
        vnorm(env, d, nd);
        let len_d = vmag(env, d);
        vnorm(env, d, dn);
        vnorm(env, e, en);
        vcross(env, dn, en, cr);
        let ndv = rv(env, nd);
        wv(env, d, [ndv[0] as u32, ndv[1] as u32, ndv[2] as u32]);
        let len_e = vmag(env, e);
        vnorm(env, e, e);
        let k = fx_div(env, len_e, len_d);
        let ev = rv(env, e);
        wv(
            env,
            e,
            [
                fx_mul(ev[0], k) as u32,
                fx_mul(ev[1], k) as u32,
                fx_mul(ev[2], k) as u32,
            ],
        );
        let dot = env.call(VEC_DOT, &mut [CArg::In(d, 12), CArg::In(e, 12)]) as i32;
        if (0..=0x1000).contains(&dot) {
            let t = fx_mul(dot, len_d);
            wv(
                env,
                p,
                [
                    fx_mul(ndv[0], t) as u32,
                    fx_mul(ndv[1], t) as u32,
                    fx_mul(ndv[2], t) as u32,
                ],
            );
            vadd(env, cur, p, p);
            let h = rv(env, hit);
            wv(env, u, [h[0] as u32, 0, h[2] as u32]);
            let pv = rv(env, p);
            wv(env, v, [pv[0] as u32, 0, pv[2] as u32]);
            let dist = env.call(VEC_DISTANCE, &mut [CArg::In(u, 12), CArg::In(v, 12)]) as i32;
            let dist = fx_div(env, dist, unit) >> 12;
            let wid = wid.wrapping_add(16);
            if wid >= dist as u32 {
                let neg = (env.r32(cr + 4) as i32) < 0;
                let t = hit_loc(env, r, loc, dist as u32, neg, None);
                let res = hit_finish(env, r, loc, CArg::In(loc, 8), t, pos);
                free(env, l);
                return res;
            }
        }
        let c = rv(env, cur);
        wv(env, prev, [c[0] as u32, c[1] as u32, c[2] as u32]);
        j += 1;
        if j >= n {
            break;
        }
    }
    free(env, l);
    0
}

/// C: `FUN_021BA25C(r)`: camera proc: target = the current position + the camera record's vector (`+4..+0xC`).
pub fn cam_follow<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let p = local(env, 12);
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(p, 12)]);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 2]);
    let v = rv(env, p);
    let x = (v[0] as u32).wrapping_add(rd32(env, c + 4));
    let y = (v[1] as u32).wrapping_add(rd32(env, c + 8));
    let z = (v[2] as u32).wrapping_add(rd32(env, c + 0xC));
    wv(env, p, [x, y, z]);
    let cam = c1(env, R_CAMERA, r);
    env.call(CAM_SET_TARGET, &mut [CArg::V(cam), CArg::In(p, 12)]);
    let cam = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam);
    free(env, p);
}

/// C: `FUN_021BA2B4(r)`: camera proc: the camera record's two angles and distance.
pub fn cam_angles<E: FieldEnv>(env: &mut E, r: u32) {
    let cam = c1(env, R_CAMERA, r);
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let p = local(env, 12);
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(p, 12)]);
    free(env, p);
    let a = rd32(env, c + 4) & 0xFFFF;
    let b = rd32(env, c + 8) & 0xFFFF;
    let d = rd32(env, c + 0xC);
    env.callv(CAM_SET_B, &[cam, b]);
    env.callv(CAM_SET_A, &[cam, a]);
    env.callv(CAM_SET_DIST, &[cam, d]);
}

/// C: `FUN_021BA310(r)`: [`cam_angles`], then the camera position = the RAIL WORK's position without the lateral offset.
pub fn cam_angles_pos<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let cam = c1(env, R_CAMERA, r);
    cam_angles(env, r);
    c1(env, CAM_E_C54, cam);
    let p = local(env, 12);
    env.call(W_POS_NO_LAT, &mut [CArg::V(w), CArg::Out(p, 12)]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(p, 12)]);
    free(env, p);
}

/// The shortest angle difference `b - a` (u16 angles; wraps at ±0x8000).
fn wrap(d: i32) -> i32 {
    if d >= 0x8000 {
        d - 0x10000
    } else if d <= -0x8000 {
        d + 0x10000
    } else {
        d
    }
}

/// C: `FUN_021BA348(r)`: camera proc: angles and distance interpolated at `t` between the camera records of the line's end
/// points (angles the short way round).
pub fn cam_interp<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let pa = c1(env, W_START, w);
    let pb = c1(env, W_END, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let ca = env.callv(W_POINT_CAM, &[w, pa]);
    let cb = env.callv(W_POINT_CAM, &[w, pb]);
    let t = frac_len_first(env, w);
    interp_angles(env, cam, ca, cb, t);
}

/// The angle / distance part of [`cam_interp`] and [`cam_interp_full`].
fn interp_angles<E: FieldEnv>(env: &mut E, cam: u32, ca: u32, cb: u32, t: i32) {
    let a1 = (rd32(env, ca + 4) & 0xFFFF) as i32;
    let b1 = (rd32(env, ca + 8) & 0xFFFF) as i32;
    let d1 = rd32(env, ca + 0xC) as i32;
    let b2 = (rd32(env, cb + 8) & 0xFFFF) as i32;
    let d2 = rd32(env, cb + 0xC) as i32;
    let a2 = (rd32(env, cb + 4) & 0xFFFF) as i32;
    let da = wrap(a2 - a1);
    let a = (a1.wrapping_add(fx_mul(da << 12, t) >> 12) as u32) & 0xFFFF;
    let db = wrap(b2 - b1);
    let b = (b1.wrapping_add(fx_mul(db << 12, t) >> 12) as u32) & 0xFFFF;
    env.callv(CAM_SET_B, &[cam, b]);
    env.callv(CAM_SET_A, &[cam, a]);
    let d = d1.wrapping_add(fx_mul(d2.wrapping_sub(d1), t));
    env.callv(CAM_SET_DIST, &[cam, d as u32]);
}

/// C: `FUN_021BA494(r)`: [`cam_interp`], then the camera position = the current position.
pub fn cam_interp_pos<E: FieldEnv>(env: &mut E, r: u32) {
    let cam = c1(env, R_CAMERA, r);
    cam_interp(env, r);
    c1(env, CAM_E_C54, cam);
    let p = local(env, 12);
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(p, 12)]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(p, 12)]);
    free(env, p);
}

/// C: `FUN_021BA4C4(r)`: [`cam_interp`], then the camera position = the position without the lateral offset.
pub fn cam_interp_pos2<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let cam = c1(env, R_CAMERA, r);
    cam_interp(env, r);
    c1(env, CAM_E_C54, cam);
    let p = local(env, 12);
    env.call(W_POS_NO_LAT, &mut [CArg::V(w), CArg::Out(p, 12)]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(p, 12)]);
    free(env, p);
}

/// C: `FUN_021BA4FC(r)`: camera proc: fixed target (`+4..+0xC`) and position (`+0x10..+0x18`) from the camera record.
pub fn cam_fixed<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 2]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C54, cam2);
    let l = local(env, 24);
    let t = [rd32(env, c + 4), rd32(env, c + 8), rd32(env, c + 0xC)];
    let p = [
        rd32(env, c + 0x10),
        rd32(env, c + 0x14),
        rd32(env, c + 0x18),
    ];
    wv(env, l + 12, t);
    wv(env, l, p);
    env.call(CAM_SET_TARGET, &mut [CArg::V(cam), CArg::In(l + 12, 12)]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(l, 12)]);
    free(env, l);
}

/// C: `FUN_021BA550(r)`: camera proc: target and position interpolated at `t` between the end points' camera records.
pub fn cam_fixed_interp<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let cam = c1(env, R_CAMERA, r);
    let pa = c1(env, W_START, w);
    let pb = c1(env, W_END, w);
    let ca = env.callv(W_POINT_CAM, &[w, pa]);
    let cb = env.callv(W_POINT_CAM, &[w, pb]);
    env.callv(CAM_MODE, &[cam, 2]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C54, cam2);
    let len = c1(env, W_LEN, w);
    let pos = c1(env, W_ALONG, w);
    let p12 = pos.wrapping_shl(12) as i32;
    let l12 = len.wrapping_shl(12) as i32;
    let mut v = [0u32; 6];
    for (k, x) in v.iter_mut().enumerate() {
        let o = 4 + 4 * k as u32;
        let d = (rd32(env, cb + o) as i32).wrapping_sub(rd32(env, ca + o) as i32);
        *x = fx_div(env, fx_mul(d, p12), l12) as u32;
    }
    for (k, x) in v.iter_mut().enumerate() {
        *x = x.wrapping_add(rd32(env, ca + 4 + 4 * k as u32));
    }
    let l = local(env, 24);
    wv(env, l + 12, [v[0], v[1], v[2]]);
    wv(env, l, [v[3], v[4], v[5]]);
    env.call(CAM_SET_TARGET, &mut [CArg::V(cam), CArg::In(l + 12, 12)]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(l, 12)]);
    free(env, l);
}

/// C: `FUN_021BA6E8(r)`: camera proc: orbit ([`orbit`]) with the angle / distance interpolated between the end points'
/// camera records (on a point: its own record), around the first record's vector `+0xC..+0x14`.
pub fn cam_orbit_interp<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    c1(env, R_CAMERA, r);
    let (ca, cb, len, pos) = if c1(env, W_KIND, w) == 1 {
        let pa = c1(env, W_START, w);
        let pb = c1(env, W_END, w);
        let ca = env.callv(W_POINT_CAM, &[w, pa]);
        let cb = env.callv(W_POINT_CAM, &[w, pb]);
        let len = c1(env, W_LEN, w) as i32;
        let pos = c1(env, W_ALONG, w) as i32;
        (ca, cb, len, pos)
    } else {
        let ca = c1(env, W_CAM, w);
        let cb = c1(env, W_CAM, w);
        (ca, cb, 1, 0)
    };
    let a1 = rd32(env, ca + 4) as i32;
    let a2 = rd32(env, cb + 4) as i32;
    let q = a2.wrapping_sub(a1).wrapping_mul(pos).wrapping_div(len);
    let ang = (a1.wrapping_add(q) as u32) & 0xFFFF;
    let d = (rd32(env, cb + 8) as i32).wrapping_sub(rd32(env, ca + 8) as i32);
    let m = fx_mul(d, pos.wrapping_shl(12));
    let k = fx_div(env, m, len.wrapping_shl(12));
    let dist = (rd32(env, ca + 8) as i32).wrapping_add(k);
    let v = local(env, 12);
    let vv = [
        rd32(env, ca + 0xC),
        rd32(env, ca + 0x10),
        rd32(env, ca + 0x14),
    ];
    wv(env, v, vv);
    orbit(env, r, ang, dist as u32, v);
    free(env, v);
}

/// C: `FUN_021BA7B0(r)`: camera proc: orbit with the camera record's angle (`+4`), distance (`+8`) and centre
/// (`+0xC..+0x14`).
pub fn cam_orbit<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let a = rd32(env, c + 4) & 0xFFFF;
    let d = rd32(env, c + 8);
    let v = local(env, 12);
    let vv = [rd32(env, c + 0xC), rd32(env, c + 0x10), rd32(env, c + 0x14)];
    wv(env, v, vv);
    orbit(env, r, a, d, v);
    free(env, v);
}

fn sin_cos<E: FieldEnv>(env: &mut E, ang: u32) -> (i32, i32) {
    let i = ((ang as i32) >> 4).wrapping_shl(1) as u32;
    let s = env.r16(SIN_COS_TABLE.wrapping_add(i.wrapping_shl(1))) as i16;
    let c = env.r16(SIN_COS_TABLE.wrapping_add(i.wrapping_add(1).wrapping_shl(1))) as i16;
    (i32::from(s), i32::from(c))
}

/// C: `FUN_021BA7E0(r, ang, dist, centre)`: place the camera on a circle around `centre`: the target `centre` (in its
/// original height `y`), the position at `dist` along the ground direction from the centre to the current position, raised
/// by `sin(ang) * dist` (the ground part `cos(ang) * dist`).
pub fn orbit<E: FieldEnv>(env: &mut E, r: u32, ang: u32, dist: u32, centre: u32) {
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 2]);
    c1(env, CAM_E_C54, cam);
    let l = local(env, 64);
    let (v, p, n, t) = (l + 40, l + 52, l + 28, l + 16);
    let cv = rv(env, centre);
    wv(env, v, [cv[0] as u32, 0, cv[2] as u32]);
    let y0 = cv[1];
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(p, 12)]);
    env.w32(p + 4, 0);
    vsub(env, p, v, p);
    vnorm(env, p, n);
    let (s, c) = sin_cos(env, ang);
    let up = fx_mul(s, dist as i32);
    let k = fx_mul(c, dist as i32);
    let nv = rv(env, n);
    let x = fx_mul(nv[0], k).wrapping_add(cv[0]);
    let y = up.wrapping_add(y0);
    let z = fx_mul(nv[2], k).wrapping_add(cv[2]);
    wv(env, t, [x as u32, y as u32, z as u32]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(centre, 12)]);
    env.call(CAM_SET_TARGET, &mut [CArg::V(cam), CArg::In(t, 12)]);
    free(env, l);
}

/// C: `FUN_021BA8E0(r)`: camera proc: orbit with the record's angle / distance around a centre moving at `t` from
/// `+0xC..+0x14` to `+0x18..+0x20`.
pub fn cam_orbit_moving<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    c1(env, W_KIND, w);
    let len = c1(env, W_LEN, w);
    let pos = c1(env, W_ALONG, w);
    let c = c1(env, W_CAM, w);
    let p12 = pos.wrapping_shl(12) as i32;
    let l12 = len.wrapping_shl(12) as i32;
    let a = rd32(env, c + 4) & 0xFFFF;
    let d = rd32(env, c + 8);
    let mut v = [0u32; 3];
    for (k, x) in v.iter_mut().enumerate() {
        let o = 0xC + 4 * k as u32;
        let e = (rd32(env, c + o + 0xC) as i32).wrapping_sub(rd32(env, c + o) as i32);
        let m = fx_div(env, fx_mul(e, p12), l12);
        *x = rd32(env, c + o).wrapping_add(m as u32);
    }
    let l = local(env, 12);
    wv(env, l, v);
    orbit(env, r, a, d, l);
    free(env, l);
}

/// C: `FUN_021BA9B4(r)`: camera proc: angles / distance from the record, the second angle pointing from the record's
/// ground point (`+0xC`, `+0x10`) to the current position.
pub fn cam_face<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let l = local(env, 24);
    let (p, v) = (l, l + 12);
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(p, 12)]);
    env.w32(p + 4, 0);
    let vx = rd32(env, c + 0xC);
    let vz = rd32(env, c + 0x10);
    wv(env, v, [vx, 0, vz]);
    vsub(env, p, v, p);
    vnorm(env, p, p);
    let pv = rv(env, p);
    let a = env.callv(FX_ATAN2_IDX, &[pv[0] as u32, pv[2] as u32]);
    let c4 = rd32(env, c + 4) & 0xFFFF;
    env.callv(CAM_SET_A, &[cam, c4]);
    let c8 = rd32(env, c + 8);
    env.callv(CAM_SET_DIST, &[cam, c8]);
    env.callv(CAM_SET_B, &[cam, a]);
    free(env, l);
}

/// Direction of travel of the RAIL WORK as three s16 (`021B8050`).
fn dir16<E: FieldEnv>(env: &mut E, w: u32, out: u32) -> [i32; 3] {
    env.call(W_DIR_VEC, &mut [CArg::V(w), CArg::Out(out, 6)]);
    [
        i32::from(env.r16(out) as i16),
        i32::from(env.r16(out + 2) as i16),
        i32::from(env.r16(out + 4) as i16),
    ]
}

/// C: `FUN_021BAA30(r)`: camera proc: behind the RAIL WORK (angle of the reversed direction of travel + record `+4`),
/// record angle `+8` and distance `+0xC`.
pub fn cam_behind<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let l = local(env, 8);
    let d = dir16(env, w, l);
    free(env, l);
    let a = env.callv(
        FX_ATAN2_IDX,
        &[d[0].wrapping_neg() as u32, d[2].wrapping_neg() as u32],
    );
    let c8 = rd32(env, c + 8) & 0xFFFF;
    env.callv(CAM_SET_A, &[cam, c8]);
    let cc = rd32(env, c + 0xC);
    env.callv(CAM_SET_DIST, &[cam, cc]);
    let c4 = rd32(env, c + 4);
    env.callv(CAM_SET_B, &[cam, a.wrapping_add(c4) & 0xFFFF]);
}

/// C: `FUN_021BAA9C(r)`: [`cam_behind`] with the pitch of the direction of travel added to the record angle `+8`.
pub fn cam_behind_pitch<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let l = local(env, 12);
    let d = dir16(env, w, l + 6);
    let a1 = env.callv(
        FX_ATAN2_IDX,
        &[d[0].wrapping_neg() as u32, d[2].wrapping_neg() as u32],
    );
    env.w16(l, d[0] as u16);
    env.w16(l + 4, d[2] as u16);
    env.w16(l + 2, 0);
    let m = env.call(VEC_FX16_MAG, &mut [CArg::In(l, 6)]);
    free(env, l);
    let a2 = env.callv(FX_ATAN2_IDX, &[d[1].wrapping_neg() as u32, m]);
    let c8 = rd32(env, c + 8);
    env.callv(CAM_SET_A, &[cam, a2.wrapping_add(c8) & 0xFFFF]);
    let cc = rd32(env, c + 0xC);
    env.callv(CAM_SET_DIST, &[cam, cc]);
    let c4 = rd32(env, c + 4);
    env.callv(CAM_SET_B, &[cam, a1.wrapping_add(c4) & 0xFFFF]);
}

/// C: `FUN_021BAB30(r)`: [`cam_behind`] (mode set differently) with the camera position = the position without the
/// lateral offset.
pub fn cam_behind_pos<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C54, cam2);
    let l = local(env, 24);
    let d = dir16(env, w, l + 4);
    let a = env.callv(
        FX_ATAN2_IDX,
        &[d[0].wrapping_neg() as u32, d[2].wrapping_neg() as u32],
    );
    let c8 = rd32(env, c + 8) & 0xFFFF;
    env.callv(CAM_SET_A, &[cam, c8]);
    let cc = rd32(env, c + 0xC);
    env.callv(CAM_SET_DIST, &[cam, cc]);
    let c4 = rd32(env, c + 4);
    env.callv(CAM_SET_B, &[cam, a.wrapping_add(c4) & 0xFFFF]);
    env.call(W_POS_NO_LAT, &mut [CArg::V(w), CArg::Out(l + 12, 12)]);
    let cam3 = c1(env, R_CAMERA, r);
    env.call(CAM_SET_POS, &mut [CArg::V(cam3), CArg::In(l + 12, 12)]);
    free(env, l);
}

/// C: `FUN_021BABB4(r)`: on a line: the camera of its start point ([`cam_at_point`]).
pub fn cam_start_point<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let k = c1(env, W_KIND, w);
    let p = c1(env, W_START, w);
    if k == 1 {
        cam_at_point(env, r, w, p);
    }
}

/// C: `FUN_021BABDC(r)`: on a line: the camera of its end point ([`cam_at_point`]).
pub fn cam_end_point<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let k = c1(env, W_KIND, w);
    let p = c1(env, W_END, w);
    if k == 1 {
        cam_at_point(env, r, w, p);
    }
}

/// C: `FUN_021BAC04(r)`: camera proc: the record's angles, distance and vector `+0x10..+0x18`.
pub fn cam_angles_vec<E: FieldEnv>(env: &mut E, r: u32) {
    let cam = c1(env, R_CAMERA, r);
    let w = c1(env, R_CURRENT, r);
    let c = c1(env, W_CAM, w);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let l = local(env, 24);
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(l + 12, 12)]);
    let a = rd32(env, c + 4) & 0xFFFF;
    let b = rd32(env, c + 8) & 0xFFFF;
    let d = rd32(env, c + 0xC);
    env.callv(CAM_SET_B, &[cam, b]);
    env.callv(CAM_SET_A, &[cam, a]);
    env.callv(CAM_SET_DIST, &[cam, d]);
    let v = [
        rd32(env, c + 0x10),
        rd32(env, c + 0x14),
        rd32(env, c + 0x18),
    ];
    wv(env, l, v);
    env.call(CAM_SET_VEC, &mut [CArg::V(cam), CArg::In(l, 12)]);
    free(env, l);
}

/// C: `FUN_021BAC74(r)`: [`cam_angles_vec`], the camera position = the current position, then `0218F05C(cam)`.
pub fn cam_angles_vec_pos<E: FieldEnv>(env: &mut E, r: u32) {
    let cam = c1(env, R_CAMERA, r);
    cam_angles_vec(env, r);
    c1(env, CAM_E_C54, cam);
    let p = local(env, 12);
    env.call(R_CUR_POS, &mut [CArg::V(r), CArg::Out(p, 12)]);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(p, 12)]);
    free(env, p);
    c1(env, CAM_F05C, cam);
}

/// C: `FUN_021BACA8(r)`: [`cam_interp`] plus the vector `+0x10..+0x18` interpolated at `t`.
pub fn cam_interp_full<E: FieldEnv>(env: &mut E, r: u32) {
    let w = c1(env, R_CURRENT, r);
    let pa = c1(env, W_START, w);
    let pb = c1(env, W_END, w);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    let cam2 = c1(env, R_CAMERA, r);
    c1(env, CAM_E_C60, cam2);
    let ca = env.callv(W_POINT_CAM, &[w, pa]);
    let cb = env.callv(W_POINT_CAM, &[w, pb]);
    let t = frac_len_first(env, w);
    interp_angles(env, cam, ca, cb, t);
    let mut v = [0u32; 3];
    for (k, x) in v.iter_mut().enumerate() {
        let o = 0x10 + 4 * k as u32;
        let a = rd32(env, ca + o) as i32;
        let b = rd32(env, cb + o) as i32;
        *x = a.wrapping_add(fx_mul(b.wrapping_sub(a), t)) as u32;
    }
    let l = local(env, 12);
    wv(env, l, v);
    env.call(CAM_SET_VEC, &mut [CArg::V(cam), CArg::In(l, 12)]);
    free(env, l);
}

/// C: `FUN_021BAE7C(pa, pb, hit, p, q, side)`: project `hit` on the segment `pb`..`pa` (point positions): `*q` = the
/// distance from `pb` along it, `p` = the projected point, `side` = (pa - pb) x (hit - pb) normalised; 1 when the
/// projection falls on the segment, `hit` is within 256.0 of it and within 8.0 in height.
pub fn tri_hit<E: FieldEnv>(
    env: &mut E,
    pa: u32,
    pb: u32,
    hit: u32,
    p: u32,
    q: u32,
    side: u32,
) -> u32 {
    let l = local(env, 64);
    let (d, e, nd, dn, en) = (l + 48, l + 36, l + 24, l + 12, l);
    vsub(env, pa + 0x30, pb + 0x30, d);
    vsub(env, hit, pb + 0x30, e);
    vnorm(env, d, nd);
    let len_d = vmag(env, d);
    let ndv = rv(env, nd);
    wv(env, d, [ndv[0] as u32, ndv[1] as u32, ndv[2] as u32]);
    let len_e = vmag(env, e);
    vnorm(env, e, e);
    let k = fx_div(env, len_e, len_d);
    let ev = rv(env, e);
    wv(
        env,
        e,
        [
            fx_mul(ev[0], k) as u32,
            fx_mul(ev[1], k) as u32,
            fx_mul(ev[2], k) as u32,
        ],
    );
    let dot = env.call(VEC_DOT, &mut [CArg::In(d, 12), CArg::In(e, 12)]) as i32;
    env.w32(q, dot as u32);
    if !(0..=0x1000).contains(&dot) {
        free(env, l);
        return 0;
    }
    env.w32(q, fx_mul(dot, len_d) as u32);
    vnorm(env, d, dn);
    vnorm(env, e, en);
    env.call(
        VEC_CROSS,
        &mut [CArg::In(dn, 12), CArg::In(en, 12), CArg::Out(side, 12)],
    );
    let t = env.r32(q) as i32;
    let z = fx_mul(ndv[2], t);
    let y = fx_mul(ndv[1], t);
    let x = fx_mul(ndv[0], t);
    wv(env, p, [x as u32, y as u32, z as u32]);
    vadd(env, pb + 0x30, p, p);
    free(env, l);
    let dist = env.call(VEC_DISTANCE, &mut [CArg::In(hit, 12), CArg::In(p, 12)]) as i32;
    if dist >= 0x10_0000 {
        return 0;
    }
    let mut dy = (env.r32(hit + 4) as i32).wrapping_sub(env.r32(p + 4) as i32);
    if dy < 0 {
        dy = dy.wrapping_neg();
    }
    (dy <= 0x8000) as u32
}

/// C: `FUN_021BB014(r, w, pt)`: the camera of the point `pt`: position = the RAIL WORK's position at that end of its line
/// (start: 0, end: its length), angles / distance from the point's camera record.
pub fn cam_at_point<E: FieldEnv>(env: &mut E, r: u32, w: u32, pt: u32) {
    let along = if c1(env, W_START, w) == pt {
        0
    } else {
        c1(env, W_LEN, w)
    };
    let v = local(env, 12);
    env.call(
        W_POS_AT,
        &mut [CArg::V(w), CArg::V(along), CArg::Out(v, 12)],
    );
    let c = env.callv(W_POINT_CAM, &[w, pt]);
    let cam = c1(env, R_CAMERA, r);
    env.callv(CAM_MODE, &[cam, 0]);
    c1(env, CAM_E_C54, cam);
    env.call(CAM_SET_POS, &mut [CArg::V(cam), CArg::In(v, 12)]);
    free(env, v);
    let a = rd32(env, c + 4) & 0xFFFF;
    env.callv(CAM_SET_A, &[cam, a]);
    let b = rd32(env, c + 8) & 0xFFFF;
    env.callv(CAM_SET_B, &[cam, b]);
    let d = rd32(env, c + 0xC);
    env.callv(CAM_SET_DIST, &[cam, d]);
}

#[cfg(test)]
mod tests;
