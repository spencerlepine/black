//! `fld_scenearea_loader.c` (overlay 21, `0x021AB7E4..0x021ABD50`, the second part of F1 row 21.31; F2b): the loader of a map's
//! scene areas (ARC 0x50 / 0x9E members are lists of 0x48-byte AREA records) and the scene-area callbacks of the callback table
//! it hands out ([`callback_table`], ROM `0x021D8EF4`): the checks [`check_ring`] / [`check_rect`] and the camera updates
//! [`cam_circle`], [`cam_fixed`], [`cam_lerp`] (+ [`cam_lerp_pre`] / [`cam_lerp_post`]), [`cam_lerp_all`] / [`cam_lerp_all_pre`] /
//! [`cam_lerp_all_post`] (with [`cam_offset`] and [`cam_fov`]).
//!
//! Spec: `tools/rom-diff/specs/black-field/scenearea_loader.spec` (its header has the LOADER / AREA layouts). Verified by
//! `tools/check-fn.sh black-field scenearea_loader --per-function`. The camera (field_camera.c, G8) and the FX vector library
//! are reached through [`FieldEnv::call`] by address (D3); the 64-bit fx32 products of the original (`0209C054`) and
//! `FX_SinCosTable_` are computed here.

use crate::field_light::fx_sin;
use crate::fieldmap::fx_cos;
use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_B7E4,
    0x021A_B81C,
    0x021A_B840,
    0x021A_B868,
    0x021A_B880,
    0x021A_B884,
    0x021A_B888,
    0x021A_B890,
    0x021A_B908,
    0x021A_B9FC,
    0x021A_BA3C,
    0x021A_BABC,
    0x021A_BB54,
    0x021A_BB78,
    0x021A_BBA0,
    0x021A_BBF0,
    0x021A_BC14,
    0x021A_BC38,
    0x021A_BC5C,
    0x021A_BD08,
];

/// Address of the string `"fld_scenearea_loader.c"` (the file argument of the allocation in [`new`]).
pub const FILE_LOADER: u32 = 0x021D_E000;
/// The two archives (ROM `0x021D8EEC`).
pub const ARCS: [u32; 2] = [0x50, 0x9E];
/// The callback table descriptor (ROM `0x021D8EF4`): {check table `0x021DDFD8` = [`check_ring`], [`check_rect`]; update
/// table `0x021DDFE0` = [`cam_circle`], [`cam_fixed`], [`cam_lerp`], [`cam_lerp_pre`], [`cam_lerp_post`], [`cam_lerp_all`],
/// [`cam_lerp_all_pre`], [`cam_lerp_all_post`]; u16 2; u16 8}.
pub const CALLBACK_TABLE: u32 = 0x021D_8EF4;
/// Size of an AREA record.
pub const AREA_SIZE: u32 = 0x48;

/// SA getters (area `scenearea`, reached by address).
const SA_FIELDSYS: u32 = 0x021A_B760;
const SA_FIELDMAP: u32 = 0x021A_B764;
/// fieldmap: map kind (1 = rail).
const FIELDMAP_KIND: u32 = 0x0218_8CFC;
/// Archive open / close / load a member (handle, member, 0, heap, &size).
const ARC_OPEN: u32 = 0x0204_90F4;
const ARC_CLOSE: u32 = 0x0204_9238;
const ARC_LOAD: u32 = 0x0204_A39C;
/// FX library (main): VEC_Subtract, VEC_Mag, VEC_Normalize, FX_Atan2Idx, FX_Div.
const VEC_SUB: u32 = 0x0207_CA00;
const VEC_MAG: u32 = 0x0207_CBD0;
const VEC_NORMALIZE: u32 = 0x0207_CCA4;
const FX_ATAN2_IDX: u32 = 0x0207_CFB8;
const FX_DIV: u32 = 0x0207_C768;
/// field_camera.c (G8): the camera (mode), target, position, target offset, angles, distance, field of view, two others.
const CAM_MODE: u32 = 0x0218_E124;
const CAM_TARGET: u32 = 0x0218_EC90;
const CAM_POSITION: u32 = 0x0218_ECF4;
const CAM_OFFSET: u32 = 0x0218_ECD4;
const CAM_ANGLE1: u32 = 0x0218_ED34;
const CAM_ANGLE2: u32 = 0x0218_ED48;
const CAM_DISTANCE: u32 = 0x0218_ED58;
const CAM_FOV: u32 = 0x0218_ED64;
const CAM_0218F0D4: u32 = 0x0218_F0D4;
const CAM_0218F05C: u32 = 0x0218_F05C;

/// `(a * b + 0x800) >> 12` on the 64-bit product (the original's `0209C054` sequence), low 32 bits.
fn fx_mul64(a: u32, b: u32) -> u32 {
    ((a as i32 as i64 * b as i32 as i64).wrapping_add(0x800) >> 12) as u32
}

/// `(a * b) / 4096` on the 32-bit product, rounding toward zero.
fn mul_div4096(a: u32, b: u32) -> u32 {
    (a.wrapping_mul(b) as i32 / 4096) as u32
}

fn vec_bytes(v: [u32; 3]) -> [u8; 12] {
    let mut b = [0u8; 12];
    for (i, x) in v.iter().enumerate() {
        b[4 * i..4 * i + 4].copy_from_slice(&x.to_le_bytes());
    }
    b
}

fn vec_words(b: &[u8; 12]) -> [u32; 3] {
    [0, 1, 2].map(|i| u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap()))
}

fn read_vec<E: FieldEnv>(env: &mut E, p: u32) -> [u32; 3] {
    [env.r32(p), env.r32(p + 4), env.r32(p + 8)]
}

/// C: `FUN_021AB7E4(heap)`: a new LOADER (0x10 bytes on `heap`) with both archives open.
pub fn new<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    let l = env.callv(HEAP_ALLOC, &[heap, 0x10, 1, FILE_LOADER, 0x81]);
    for (i, &arc) in ARCS.iter().enumerate() {
        let h = env.callv(ARC_OPEN, &[arc, heap]);
        env.w32(l + 8 + 4 * i as u32, h);
    }
    l
}

/// C: `FUN_021AB81C(l)`: free the area list, close both archives, free the LOADER.
pub fn delete<E: FieldEnv>(env: &mut E, l: u32) {
    free_list(env, l);
    for i in 0..2u32 {
        let h = env.r32(l + 8 + 4 * i);
        env.callv(ARC_CLOSE, &[h]);
    }
    env.callv(HEAP_FREE, &[l]);
}

/// C: `FUN_021AB840(l, i, member, heap)`: load member `member` of archive `i` on `heap` as the area list; the count is its size
/// / 0x48 (`heap` is the original's r3, passed on to the loader untouched).
pub fn load<E: FieldEnv>(env: &mut E, l: u32, i: u32, member: u32, heap: u32) {
    let h = env.r32(l.wrapping_add(i << 2).wrapping_add(8));
    let mut size = [0u8; 4];
    let p = env.call(
        ARC_LOAD,
        &mut [
            CArg::V(h),
            CArg::V(member),
            CArg::V(0),
            CArg::V(heap),
            CArg::OutBuf(&mut size),
        ],
    );
    env.w32(l, p);
    env.w32(l + 4, u32::from_le_bytes(size) / AREA_SIZE);
}

/// C: `FUN_021AB868(l)`: free the area list, if any.
pub fn free_list<E: FieldEnv>(env: &mut E, l: u32) {
    let p = env.r32(l);
    if p != 0 {
        env.callv(HEAP_FREE, &[p]);
        env.w32(l, 0);
        env.w32(l + 4, 0);
    }
}

/// C: `FUN_021AB880(l)`: the area list.
pub fn list<E: FieldEnv>(env: &mut E, l: u32) -> u32 {
    env.r32(l)
}

/// C: `FUN_021AB884(l)`: the area count.
pub fn count<E: FieldEnv>(env: &mut E, l: u32) -> u32 {
    env.r32(l + 4)
}

/// C: `FUN_021AB888()`: the callback table descriptor ([`CALLBACK_TABLE`]).
pub fn callback_table() -> u32 {
    CALLBACK_TABLE
}

/// C: `FUN_021AB890(sa, area, pos)` (check): 1 when `pos` is inside the ring sector of the area: distance from the centre
/// (`+0x10`, y ignored) in `[+8, +0xC)` (signed) and direction `FX_Atan2Idx` in `[+0, +4)` (wrapping past 0x10000 when
/// `+4 < +0`).
pub fn check_ring<E: FieldEnv>(env: &mut E, _sa: u32, area: u32, pos: u32) -> u32 {
    let c = vec_bytes(read_vec(env, area + 0x10));
    let mut d = [0u8; 12];
    env.call(
        VEC_SUB,
        &mut [CArg::In(pos, 12), CArg::InBuf(&c), CArg::OutBuf(&mut d)],
    );
    d[4..8].copy_from_slice(&[0; 4]);
    let len = env.call(VEC_MAG, &mut [CArg::InBuf(&d)]);
    let din = d;
    env.call(
        VEC_NORMALIZE,
        &mut [CArg::InBuf(&din), CArg::OutBuf(&mut d)],
    );
    let w = vec_words(&d);
    let ang = env.callv(FX_ATAN2_IDX, &[w[0], w[2]]);
    if env.r32(area + 8) as i32 > len as i32 {
        return 0;
    }
    if env.r32(area + 0xC) as i32 <= len as i32 {
        return 0;
    }
    let a0 = env.r32(area);
    let a1 = env.r32(area + 4);
    let inside = if a1 >= a0 {
        a0 <= ang && a1 > ang
    } else {
        a1 >= ang || (ang < 0x1_0000 && ang >= a0)
    };
    inside as u32
}

/// C: `FUN_021AB908(sa, area, pos)` (update): the camera looks at the centre (`+0x10`) from distance `+0x20` at elevation
/// angle `+0x1C`, on the side of `pos` (camera mode 2).
pub fn cam_circle<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    env.callv(CAM_MODE, &[cam, 2]);
    let cy = env.r32(area + 0x14);
    let mut l1 = [env.r32(area + 0x10), 0, env.r32(area + 0x18)];
    let mut v = read_vec(env, pos);
    v[1] = 0;
    let vb = vec_bytes(v);
    let mut out = [0u8; 12];
    env.call(
        VEC_SUB,
        &mut [
            CArg::InBuf(&vb),
            CArg::InBuf(&vec_bytes(l1)),
            CArg::OutBuf(&mut out),
        ],
    );
    let mut n = [0u8; 12];
    env.call(
        VEC_NORMALIZE,
        &mut [CArg::InBuf(&out), CArg::OutBuf(&mut n)],
    );
    let n = vec_words(&n);
    l1[1] = cy;
    // UNVERIFIED: an angle above 0xFFFF (or negative) reads outside FX_SinCosTable_ in the original; fx_sin / fx_cos
    // continue the sine periodically here
    let i = (env.r32(area + 0x1C) as i32 >> 4) as u32;
    let dist = env.r32(area + 0x20);
    let h = fx_mul64(fx_sin(i) as i32 as u32, dist);
    let r = fx_mul64(fx_cos(i) as i32 as u32, dist);
    let px = fx_mul64(n[0], r).wrapping_add(l1[0]);
    let py = h.wrapping_add(cy);
    let pz = fx_mul64(n[2], r).wrapping_add(l1[2]);
    env.call(CAM_TARGET, &mut [CArg::V(cam), CArg::InBuf(&vec_bytes(l1))]);
    env.call(
        CAM_POSITION,
        &mut [CArg::V(cam), CArg::InBuf(&vec_bytes([px, py, pz]))],
    );
}

/// C: `FUN_021AB9FC(sa, area, pos)` (update): a fixed camera: target `+0x24`, position `+0x30` (camera mode 2).
pub fn cam_fixed<E: FieldEnv>(env: &mut E, sa: u32, area: u32, _pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    env.callv(CAM_MODE, &[cam, 2]);
    let t = vec_bytes(read_vec(env, area + 0x24));
    let p = vec_bytes(read_vec(env, area + 0x30));
    env.call(CAM_TARGET, &mut [CArg::V(cam), CArg::InBuf(&t)]);
    env.call(CAM_POSITION, &mut [CArg::V(cam), CArg::InBuf(&p)]);
}

/// Grid coordinate of an fx32 position component: `((v - 0x8000) >> 12) / 16` (rounding toward zero), as u16.
fn grid(v: u32) -> u32 {
    let r = (v.wrapping_sub(0x8000) as i32) >> 12;
    (r / 16) as u32 & 0xFFFF
}

/// C: `FUN_021ABA3C(sa, area, pos)` (check): on a grid map (the fieldmap's kind is not 1), 1 when the grid cell of `pos` is
/// inside the rectangle `x = +0`, `z = +2`, extents `(+8, +6)` when `+4 == 1`, else `(+4, +8)`.
pub fn check_rect<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) -> u32 {
    let m = env.call(SA_FIELDMAP, &mut [CArg::P(sa)]);
    if env.callv(FIELDMAP_KIND, &[m]) == 1 {
        return 0;
    }
    let gx = grid(env.r32(pos));
    let gz = grid(env.r32(pos + 8));
    let k = env.r16(area + 4) as u32;
    let (w, h) = if k == 1 {
        (env.r16(area + 8) as u32, env.r16(area + 6) as u32)
    } else {
        (k, env.r16(area + 8) as u32)
    };
    let x = env.r16(area) as u32;
    if gx < x || gx >= x + w {
        return 0;
    }
    let z = env.r16(area + 2) as u32;
    if gz < z || gz >= z + h {
        return 0;
    }
    1
}

/// The two values [`rect_pos`] writes: `(+8 << 16, offset of pos along the rectangle's axis)`.
fn rect_vals<E: FieldEnv>(env: &mut E, area: u32, pos: u32) -> (u32, u32) {
    let x = env.r32(pos).wrapping_sub(0x8000);
    let z = env.r32(pos + 8).wrapping_sub(0x8000);
    let len = (env.r16(area + 8) as u32) << 16;
    let t = if env.r16(area + 4) == 1 {
        x.wrapping_sub((env.r16(area) as u32) << 16)
    } else {
        z.wrapping_sub((env.r16(area + 2) as u32) << 16)
    };
    (len, t)
}

/// C: `FUN_021ABBA0(area, pos, &len, &t)`: the rectangle's length (`+8 << 16`) and the offset of `pos` (x / z minus 0x8000)
/// from the rectangle's start along its axis (`+4 == 1`: x from `+0 << 16`, else z from `+2 << 16`).
pub fn rect_pos<E: FieldEnv>(env: &mut E, area: u32, pos: u32, len: u32, t: u32) {
    let (a, b) = rect_vals(env, area, pos);
    env.w32(len, a);
    env.w32(t, b);
}

/// C: `FUN_021ABABC(sa, area, pos)` (update): camera mode 0; the two camera angles (`+0xC` -> `+0x14`, `+0xE` -> `+0x16`) and
/// the distance (`+0x10` -> `+0x18`) interpolated by `pos`'s offset along the rectangle (`FX_Div(t, len)`).
pub fn cam_lerp<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    env.callv(CAM_MODE, &[cam, 0]);
    let (len, t) = rect_vals(env, area, pos);
    let f = env.callv(FX_DIV, &[t, len]);
    let a0 = env.r16(area + 0xC) as u32;
    let a1 = env.r16(area + 0x14) as u32;
    let v1 = mul_div4096(a1.wrapping_sub(a0), f).wrapping_add(a0);
    let b0 = env.r16(area + 0xE) as u32;
    let b1 = env.r16(area + 0x16) as u32;
    let v2 = mul_div4096(b1.wrapping_sub(b0), f).wrapping_add(b0);
    let d0 = env.r32(area + 0x10);
    let d = fx_mul64(env.r32(area + 0x18).wrapping_sub(d0), f).wrapping_add(d0);
    env.callv(CAM_ANGLE1, &[cam, v1 & 0xFFFF]);
    env.callv(CAM_ANGLE2, &[cam, v2 & 0xFFFF]);
    env.callv(CAM_DISTANCE, &[cam, d]);
}

/// C: `FUN_021ABB54(sa, area, pos)` (update): `0218F0D4(camera)` when `+0x1C` is set, then [`cam_lerp`].
pub fn cam_lerp_pre<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    if env.r32(area + 0x1C) != 0 {
        env.callv(CAM_0218F0D4, &[cam]);
    }
    cam_lerp(env, sa, area, pos);
}

/// C: `FUN_021ABB78(sa, area, pos)` (update): [`cam_lerp`], then `0218F05C(camera)` when `+0x1C` is set.
pub fn cam_lerp_post<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    cam_lerp(env, sa, area, pos);
    if env.r32(area + 0x1C) != 0 {
        env.callv(CAM_0218F05C, &[cam]);
    }
}

/// C: `FUN_021ABBF0(sa, area, pos)` (update): [`cam_lerp`], [`cam_offset`], [`cam_fov`].
pub fn cam_lerp_all<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    cam_lerp(env, sa, area, pos);
    cam_offset(env, sa, area, pos);
    cam_fov(env, sa, area, pos);
}

/// C: `FUN_021ABC14(sa, area, pos)` (update): [`cam_lerp_pre`], [`cam_offset`], [`cam_fov`].
pub fn cam_lerp_all_pre<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    cam_lerp_pre(env, sa, area, pos);
    cam_offset(env, sa, area, pos);
    cam_fov(env, sa, area, pos);
}

/// C: `FUN_021ABC38(sa, area, pos)` (update): [`cam_lerp_post`], [`cam_offset`], [`cam_fov`].
pub fn cam_lerp_all_post<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    cam_lerp_post(env, sa, area, pos);
    cam_offset(env, sa, area, pos);
    cam_fov(env, sa, area, pos);
}

/// C: `FUN_021ABC5C(sa, area, pos)` (update): the camera target offset interpolated from `+0x20` to `+0x2C` (each component
/// `start + FX_Div(((end - start) * t + 0x800) >> 12, len)`).
pub fn cam_offset<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    let (len, t) = rect_vals(env, area, pos);
    let mut o = [0u32; 3];
    for (k, x) in o.iter_mut().enumerate() {
        let k = 4 * k as u32;
        let s = env.r32(area + 0x20 + k);
        let e = env.r32(area + 0x2C + k);
        let p = fx_mul64(e.wrapping_sub(s), t);
        let q = env.callv(FX_DIV, &[p, len]);
        *x = env.r32(area + 0x20 + k).wrapping_add(q);
    }
    env.call(CAM_OFFSET, &mut [CArg::V(cam), CArg::InBuf(&vec_bytes(o))]);
}

/// C: `FUN_021ABD08(sa, area, pos)` (update): the field of view interpolated from `+0x38` to `+0x3A`.
pub fn cam_fov<E: FieldEnv>(env: &mut E, sa: u32, area: u32, pos: u32) {
    let cam = env.call(SA_FIELDSYS, &mut [CArg::P(sa)]);
    let (len, t) = rect_vals(env, area, pos);
    let f = env.callv(FX_DIV, &[t, len]);
    let a = env.r16(area + 0x38) as u32;
    let b = env.r16(area + 0x3A) as u32;
    let v = mul_div4096(f, b.wrapping_sub(a)).wrapping_add(a);
    env.callv(CAM_FOV, &[cam, v & 0xFFFF]);
}

#[cfg(test)]
mod tests;
