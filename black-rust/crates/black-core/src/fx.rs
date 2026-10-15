//! NitroSDK fixed-point math (FX / VEC / MTX) of the ARM9 main image, `0x0207AF50..0x0207D184` (Z24).
//!
//! Types: `fx32` = `i32` (20.12), `fx16` = `i16` (4.12); a `VecFx32` is `[i32; 3]`, a `VecFx16` `[i16; 3]`, a
//! `MtxFx33` `[i32; 9]` and a `MtxFx43` `[i32; 12]` (rows of the C structs in memory order).
//!
//! **Hardware unit.** `FX_Div`, `FX_Inv`, `FX_Sqrt`, `FX_InvSqrt`, `VEC_Mag`, `VEC_Normalize` and the rest of the
//! division / square-root users drive the DS divider and square-root coprocessor through the memory-mapped
//! `REG_DIVCNT` (0x04000280), `REG_DIV_NUMER` (0x04000290), `REG_DIV_DENOM` (0x04000298), `REG_DIV_RESULT`
//! (0x040002A0), `REG_DIVREM_RESULT` (0x040002A8), `REG_SQRTCNT` (0x040002B0), `REG_SQRT_RESULT` (0x040002B4) and
//! `REG_SQRT_PARAM` (0x040002B8). The ports are pure: [`div32`], [`div64_32`], [`div64_64`] return the result
//! registers as GBATEK defines them (truncating division, divide by zero, `MIN / -1`), [`sqrt32`] / [`sqrt64`] the
//! integer square root. No register state is kept in the API.
//!
//! **Tables.** `FX_SinCosTable_` (main `0x020A1B38`, 4096 `{s16 sin, s16 cos}` pairs) and the arctangent table
//! (`0x020A5B38`, 129 `s16`) are computed ([`sin`], [`cos`], [`atan_entry`]); the tests compare all entries with the
//! ROM image. Nothing from the ROM is embedded.
//!
//! Verification: the functions in [`PORTED`] (no hardware unit) run under `tools/check-fn.sh black-core fx
//! --per-function`; the hardware users in [`HW_PORTED`] cannot, because the generic harness leaves the I/O page
//! as plain memory. They are verified with `tools/rom-diff/fx_hw_diff.py` (the same emulator with the divider
//! modelled) against the recorded hash in `fx/tests.rs`.

/// Functions verified by `check-fn` (no divider / square-root unit).
pub const PORTED: &[u32] = &[
    0x0207_AF50, // MTX_RotAxis33
    0x0207_B65C, // MTX_RotAxis43
    0x0207_BC08, // MTX_MultVec43
    0x0207_C9D0, // VEC_Add
    0x0207_CA00, // VEC_Subtract
    0x0207_CA30, // VEC_Fx16Add
    0x0207_CA68, // VEC_DotProduct
    0x0207_CAA4, // VEC_Fx16DotProduct
    0x0207_CAE8, // VEC_CrossProduct
    0x0207_CB68, // VEC_Fx16CrossProduct
    0x0207_CEE4, // VEC_MultAdd
    0x0207_D184, // FX_AcosIdx
];

/// Functions that use the divider / square-root unit (verified by `fx_hw_diff.py`).
pub const HW_PORTED: &[u32] = &[
    0x0207_C768, // FX_Div
    0x0207_C778, // FX_Inv
    0x0207_C7B4, // FX_Sqrt
    0x0207_C7E8, // FX_InvSqrt
    0x0207_C958, // CP_Divide32 (quotient)
    0x0207_C994, // CP_Divide32 (remainder)
    0x0207_CBD0, // VEC_Mag
    0x0207_CC30, // VEC_Fx16Mag
    0x0207_CCA4, // VEC_Normalize
    0x0207_CDB8, // VEC_Fx16Normalize
    0x0207_CF3C, // VEC_Distance
    0x0207_CFB8, // FX_Atan2Idx
];

pub type Vec3 = [i32; 3];
pub type Vec16 = [i16; 3];
pub type Mtx33 = [i32; 9];
pub type Mtx43 = [i32; 12];

// ------------------------------------------------------------------ hardware unit

/// Division unit, `DIVCNT` mode 0 (32 / 32): returns `(DIV_RESULT, DIVREM_RESULT)` as the 64-bit registers.
/// A zero divisor gives the quotient `-1` (numerator >= 0) or `+1` with the upper word inverted, the remainder is
/// the numerator; `i32::MIN / -1` gives `0x0000_0000_8000_0000`, remainder 0.
pub fn div32(n: i32, d: i32) -> (u64, u64) {
    if d == 0 {
        let q: u64 = if n < 0 {
            0xFFFF_FFFF_0000_0001
        } else {
            0x0000_0000_FFFF_FFFF
        };
        return (q, n as i64 as u64);
    }
    if n == i32::MIN && d == -1 {
        return (0x8000_0000, 0);
    }
    ((n / d) as i64 as u64, (n % d) as i64 as u64)
}

/// Mode 1 (64 / 32, the denominator is the low word, signed): `(DIV_RESULT, DIVREM_RESULT)`. A zero divisor gives
/// `-1` (numerator >= 0) or `+1`, remainder = numerator; `i64::MIN / -1` wraps to `i64::MIN`, remainder 0.
pub fn div64_32(n: i64, d: i32) -> (i64, i64) {
    div64_64(n, d as i64)
}

/// Mode 2 (64 / 64): see [`div64_32`].
pub fn div64_64(n: i64, d: i64) -> (i64, i64) {
    if d == 0 {
        return (if n < 0 { 1 } else { -1 }, n);
    }
    (n.wrapping_div(d), n.wrapping_rem(d))
}

/// Integer square root of a 32-bit parameter (`SQRTCNT` bit 0 = 0): `floor(sqrt(x))`.
pub fn sqrt32(x: u32) -> u32 {
    sqrt64(x as u64)
}

/// Integer square root of a 64-bit parameter (`SQRTCNT` bit 0 = 1): `floor(sqrt(x))` (fits 32 bits).
pub fn sqrt64(x: u64) -> u32 {
    let mut r = (x as f64).sqrt() as u64;
    while r.saturating_mul(r) > x || r > u32::MAX as u64 {
        r -= 1;
    }
    while (r + 1) <= u32::MAX as u64 && (r + 1) * (r + 1) <= x {
        r += 1;
    }
    r as u32
}

// ------------------------------------------------------------------ tables

/// `FX_SinCosTable_` sine entry `i & 0xFFF`: `round(4096 sin(2 pi i / 4096))`.
pub fn sin(i: u32) -> i16 {
    (((i & 0xFFF) as f64 * std::f64::consts::TAU / 4096.0).sin() * 4096.0).round() as i16
}

/// `FX_SinCosTable_` cosine entry `i & 0xFFF`.
pub fn cos(i: u32) -> i16 {
    (((i & 0xFFF) as f64 * std::f64::consts::TAU / 4096.0).cos() * 4096.0).round() as i16
}

/// Arctangent table (main `0x020A5B38`): `round(atan(i / 128) * 65536 / (2 pi))`, `i` in `0..=128`. Panics outside
/// (the original reads whatever follows the table).
pub fn atan_entry(i: i32) -> i16 {
    assert!(
        (0..=128).contains(&i),
        "atan table index {i} outside the table"
    );
    ((i as f64 / 128.0).atan() / std::f64::consts::TAU * 65536.0).round() as i16
}

// ------------------------------------------------------------------ FX

/// `FX_Mul` as the callers inline it: `(a * b + 0x800) >> 12` on the 64-bit product, low 32 bits.
pub fn fx_mul(a: i32, b: i32) -> i32 {
    ((a as i64 * b as i64 + 0x800) >> 12) as i32
}

/// Rounded `(q + 0x80000) >> 20` of a 64-bit quotient register, low 32 bits (`0x0207C86C`).
fn result_fx(q: i64) -> i32 {
    (((q as u64).wrapping_add(0x8_0000)) >> 20) as u32 as i32
}

/// C: `FX_Div` (`0x0207C768`): `a / b` in fx32 (numerator `a << 32`, 64 / 32 division, rounded).
pub fn fx_div(a: i32, b: i32) -> i32 {
    result_fx(div64_32((a as i64) << 32, b).0)
}

/// C: `FX_Inv` (`0x0207C778`): `1 / x` in fx32 (`FX_Div(0x1000, x)`).
pub fn fx_inv(x: i32) -> i32 {
    fx_div(0x1000, x)
}

/// C: `FX_Sqrt` (`0x0207C7B4`): `sqrt(x)` in fx32; 0 for `x <= 0`.
pub fn fx_sqrt(x: i32) -> i32 {
    if x <= 0 {
        return 0;
    }
    let r = sqrt64((x as u64) << 32);
    (r.wrapping_add(0x200) >> 10) as i32
}

/// C: `FX_InvSqrt` (`0x0207C7E8`): `1 / sqrt(x)` in fx32; 0 for `x <= 0`.
pub fn fx_inv_sqrt(x: i32) -> i32 {
    if x <= 0 {
        return 0;
    }
    let r = div64_32(0x1000i64 << 32, x).0 as u64;
    let s = sqrt64((x as u64) << 32) as u64;
    let hi = (r.wrapping_mul(s) >> 32) as u32;
    (hi.wrapping_add(0x200) as i32) >> 10
}

/// C: `0x0207C958`: hardware 32-bit quotient (`REG_DIV_RESULT` low word).
pub fn cp_div32(n: i32, d: i32) -> i32 {
    div32(n, d).0 as u32 as i32
}

/// C: `0x0207C994`: hardware 32-bit remainder (`REG_DIVREM_RESULT` low word).
pub fn cp_mod32(n: i32, d: i32) -> i32 {
    div32(n, d).1 as u32 as i32
}

/// C: `FX_AcosIdx` (`0x0207D184`): angle index (0x10000 = full circle, units of 8) of a cosine, by binary search of
/// the cosine column of `FX_SinCosTable_` (indices 0..=0x400 for a non-negative argument, 0x400..=0x800 else; the
/// comparison uses the low 16 bits).
pub fn fx_acos_idx(c: i32) -> u32 {
    let v = c as i16;
    let (mut lo, mut hi): (i32, i32) = if c < 0 { (0x400, 0x800) } else { (0, 0x400) };
    let mut mid = 0;
    while lo <= hi {
        mid = (lo + hi) / 2;
        let t = cos(mid as u32);
        if v == t {
            break;
        }
        if t < v {
            hi = mid - 1;
        } else {
            lo = mid + 1;
        }
    }
    let x = (mid * 2 + 1).wrapping_mul(0x10000);
    ((x as u32).wrapping_add(((x >> 12) as u32) >> 19) & 0x1FFF_FFFF) >> 13
}

/// C: `FX_Atan2Idx(y, x)` (`0x0207CFB8`): angle of `(x, y)` as a signed 16-bit index (0x10000 = full circle).
/// Panics where the original would index outside its arctangent table (`i32::MIN` operands).
pub fn fx_atan2_idx(y: i32, x: i32) -> i16 {
    // `num` / `den` and the sign of the table term, as in the original's branch ladder.
    let (base, num, den, plus): (i16, i32, i32, bool);
    if y < 1 {
        if y > -1 {
            return if x < 0 { -0x8000 } else { 0 };
        }
        let n = y.wrapping_neg();
        if x < 0 {
            let m = x.wrapping_neg();
            if y == x || m < n {
                if n <= m {
                    return -0x6000;
                }
                base = -0x4000;
                den = n;
                num = m;
                plus = false;
            } else {
                base = -0x8000;
                den = m;
                num = n;
                plus = true;
            }
        } else {
            if x < 1 {
                return -0x4000;
            }
            if x < n {
                base = -0x4000;
                den = n;
                num = x;
                plus = true;
            } else {
                if x <= n {
                    return -0x2000;
                }
                base = 0;
                den = x;
                num = n;
                plus = false;
            }
        }
    } else if x < 1 {
        if x > -1 {
            return 0x4000;
        }
        let n = x.wrapping_neg();
        if n < y {
            base = 0x4000;
            den = y;
            num = n;
            plus = true;
        } else {
            if y.wrapping_neg() == x || n < y {
                return 0x6000;
            }
            base = -0x8000;
            den = n;
            num = y;
            plus = false;
        }
    } else if y < x {
        base = 0;
        den = x;
        num = y;
        plus = true;
    } else {
        if y <= x {
            return 0x2000;
        }
        base = 0x4000;
        den = y;
        num = x;
        plus = false;
    }
    if den == 0 {
        return 0;
    }
    let t = atan_entry(fx_div(num, den) >> 5);
    if plus {
        base.wrapping_add(t)
    } else {
        base.wrapping_sub(t)
    }
}

// ------------------------------------------------------------------ VEC

/// C: `VEC_Add` (`0x0207C9D0`).
pub fn vec_add(a: &Vec3, b: &Vec3) -> Vec3 {
    [
        a[0].wrapping_add(b[0]),
        a[1].wrapping_add(b[1]),
        a[2].wrapping_add(b[2]),
    ]
}

/// C: `VEC_Subtract` (`0x0207CA00`): `a - b`.
pub fn vec_subtract(a: &Vec3, b: &Vec3) -> Vec3 {
    [
        a[0].wrapping_sub(b[0]),
        a[1].wrapping_sub(b[1]),
        a[2].wrapping_sub(b[2]),
    ]
}

/// C: `VEC_Fx16Add` (`0x0207CA30`): component sums truncated to 16 bits.
pub fn vec_fx16_add(a: &Vec16, b: &Vec16) -> Vec16 {
    [
        (a[0] as i32 + b[0] as i32) as i16,
        (a[1] as i32 + b[1] as i32) as i16,
        (a[2] as i32 + b[2] as i32) as i16,
    ]
}

/// Bits 12..43 of a 64-bit accumulator (`lsr 12 | hi << 20`).
fn bits12(p: i64) -> i32 {
    ((p as u64) >> 12) as u32 as i32
}

fn sumsq(v: &Vec3) -> i64 {
    let sq = |x: i32| (x as i64).wrapping_mul(x as i64);
    sq(v[2]).wrapping_add(sq(v[0])).wrapping_add(sq(v[1]))
}

/// C: `VEC_DotProduct` (`0x0207CA68`): `(a . b + 0x800) >> 12` on the 64-bit sum.
pub fn vec_dot_product(a: &Vec3, b: &Vec3) -> i32 {
    let m = |i: usize| (a[i] as i64).wrapping_mul(b[i] as i64);
    bits12(
        m(1).wrapping_add(m(0))
            .wrapping_add(m(2))
            .wrapping_add(0x800),
    )
}

/// C: `VEC_Fx16DotProduct` (`0x0207CAA4`): the first two products are summed in 32 bits (it wraps), then the
/// rounded 64-bit sum.
pub fn vec_fx16_dot_product(a: &Vec16, b: &Vec16) -> i32 {
    let p = |i: usize| (a[i] as i32).wrapping_mul(b[i] as i32);
    let r0 = p(2).wrapping_add(0x800);
    let r1 = (a[0] as i32).wrapping_mul(b[0] as i32).wrapping_add(p(1));
    bits12((r1 as i64).wrapping_add(r0 as i64))
}

/// C: `VEC_CrossProduct` (`0x0207CAE8`): `a x b`, each component `(.. + 0x800) >> 12` on the 64-bit difference.
pub fn vec_cross_product(a: &Vec3, b: &Vec3) -> Vec3 {
    let m = |x: i32, y: i32| (x as i64).wrapping_mul(y as i64);
    let c = |p: i64, q: i64| bits12(p.wrapping_sub(q).wrapping_add(0x800));
    [
        c(m(a[1], b[2]), m(a[2], b[1])),
        c(m(a[2], b[0]), m(a[0], b[2])),
        c(m(a[0], b[1]), m(a[1], b[0])),
    ]
}

/// C: `VEC_Fx16CrossProduct` (`0x0207CB68`): 32-bit arithmetic, `(.. + 0x800) >> 12` truncated to 16 bits.
pub fn vec_fx16_cross_product(a: &Vec16, b: &Vec16) -> Vec16 {
    let m = |x: i16, y: i16| (x as i32).wrapping_mul(y as i32);
    let c = |p: i32, q: i32| (p.wrapping_sub(q).wrapping_add(0x800) >> 12) as i16;
    [
        c(m(a[1], b[2]), m(a[2], b[1])),
        c(m(a[2], b[0]), m(a[0], b[2])),
        c(m(a[0], b[1]), m(a[1], b[0])),
    ]
}

/// `(sqrt(4 s) + 1) >> 1` as the magnitude functions return it.
fn mag_from_sumsq(s: i64) -> i32 {
    (sqrt64((s as u64).wrapping_mul(4)) as i32).wrapping_add(1) >> 1
}

/// C: `VEC_Mag` (`0x0207CBD0`): `|v|` in fx32.
pub fn vec_mag(v: &Vec3) -> i32 {
    mag_from_sumsq(sumsq(v))
}

/// C: `VEC_Fx16Mag` (`0x0207CC30`).
pub fn vec_fx16_mag(v: &Vec16) -> i32 {
    mag_from_sumsq(sumsq(&[v[0] as i32, v[1] as i32, v[2] as i32]))
}

/// C: `VEC_MultAdd` (`0x0207CEE4`): `k * a + b` per component, the product `>> 12` on 64 bits.
pub fn vec_mult_add(k: i32, a: &Vec3, b: &Vec3) -> Vec3 {
    let f = |i: usize| b[i].wrapping_add(bits12((k as i64).wrapping_mul(a[i] as i64)));
    [f(0), f(1), f(2)]
}

/// C: `VEC_Distance` (`0x0207CF3C`): `|a - b|` in fx32 (differences wrap in 32 bits).
pub fn vec_distance(a: &Vec3, b: &Vec3) -> i32 {
    mag_from_sumsq(sumsq(&vec_subtract(a, b)))
}

/// Shared tail of the two normalisers: `s = sqrt(4 |v|^2)`, `R = 2^56 / |v|^2` (64 / 64 division), `K = s * R`;
/// component `v` becomes `(hi32(v * K) + 0x1000) >> 13`.
fn normalize(v: [i32; 3], s2: i64) -> [i32; 3] {
    let r = div64_64(1i64 << 56, s2).0;
    let s = sqrt64((s2 as u64).wrapping_mul(4)) as i32;
    let k = (s as i64).wrapping_mul(r);
    let f = |x: i32| ((((x as i64).wrapping_mul(k) >> 32) as i32).wrapping_add(0x1000)) >> 13;
    [f(v[0]), f(v[1]), f(v[2])]
}

/// C: `VEC_Normalize` (`0x0207CCA4`): the unit vector (fx32); the zero vector gives zero.
pub fn vec_normalize(v: &Vec3) -> Vec3 {
    normalize(*v, sumsq(v))
}

/// C: `VEC_Fx16Normalize` (`0x0207CDB8`).
pub fn vec_fx16_normalize(v: &Vec16) -> Vec16 {
    let w = [v[0] as i32, v[1] as i32, v[2] as i32];
    let o = normalize(w, sumsq(&w));
    [o[0] as i16, o[1] as i16, o[2] as i16]
}

// ------------------------------------------------------------------ MTX

/// C: `MTX_RotAxis33` (`0x0207AF50`): rotation about the (unit) `axis` with the given sine and cosine (fx32).
pub fn mtx_rot_axis33(axis: &Vec3, sin: i32, cos: i32) -> Mtx33 {
    let (x, y, z) = (axis[0] as i64, axis[1] as i64, axis[2] as i64);
    let oc = (0x1000i32.wrapping_sub(cos)) as i64;
    let b24 = |p: i64| ((p as u64) >> 24) as u32 as i32;
    let a = oc.wrapping_mul(x);
    let xy = b24(a.wrapping_mul(y));
    let xz = b24(a.wrapping_mul(z));
    let yz = b24(oc.wrapping_mul(y).wrapping_mul(z));
    let sq = |t: i64| b24(oc.wrapping_mul(t.wrapping_mul(t))).wrapping_add(cos);
    let s = |t: i64| ((sin as i64).wrapping_mul(t) as u64 >> 12) as u32 as i32;
    let (sx, sy, sz) = (s(x), s(y), s(z));
    [
        sq(x),
        xy.wrapping_add(sz),
        xz.wrapping_sub(sy),
        xy.wrapping_sub(sz),
        sq(y),
        yz.wrapping_add(sx),
        xz.wrapping_add(sy),
        yz.wrapping_sub(sx),
        sq(z),
    ]
}

/// C: `MTX_RotAxis43` (`0x0207B65C`): [`mtx_rot_axis33`] with a zero translation.
pub fn mtx_rot_axis43(axis: &Vec3, sin: i32, cos: i32) -> Mtx43 {
    let m = mtx_rot_axis33(axis, sin, cos);
    let mut o = [0; 12];
    o[..9].copy_from_slice(&m);
    o
}

/// C: `MTX_MultVec43` (`0x0207BC08`): `v * M + t` with the 3x3 part in rows (`out_j = sum_i v_i M[3i + j]`).
pub fn mtx_mult_vec43(v: &Vec3, m: &Mtx43) -> Vec3 {
    let p = |a: i32, b: i32| (a as i64).wrapping_mul(b as i64);
    let f = |j: usize| {
        let acc = p(m[6 + j], v[2])
            .wrapping_add(p(m[j], v[0]))
            .wrapping_add(p(v[1], m[3 + j]));
        bits12(acc).wrapping_add(m[9 + j])
    };
    [f(0), f(1), f(2)]
}

#[cfg(test)]
mod tests;
