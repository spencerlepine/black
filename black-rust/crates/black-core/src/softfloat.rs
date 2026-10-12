//! NitroSDK / compiler-runtime soft-float helpers (Z17), main ARM9 `0x0209B750`-`0x0209CB8C`.
//!
//! Pure integer ports (no host floating point). Despite the old note "int to double", the
//! routines the battle code uses are **single precision** (IEEE-754 binary32): the arguments and
//! the result are bit patterns passed in `r0` (and `r1`), as the ARM9 has no FPU. Only `i2d` /
//! `u2d` produce a double: they return it in `r0` (low word) : `r1` (high word), which this
//! module models as one `u64` (`r1 << 32 | r0`).
//!
//! The original is hand-written ARM code, not a library `__fadd`: rounding is to nearest even
//! on the 32-bit working mantissa, but NaN, infinity and denormal handling have quirks (a NaN
//! result is always `0x7FFF_FFFF`; the sticky bit is lost for exponent gaps over 32; a few
//! special-value paths return odd bit patterns). The ports follow the machine code label by
//! label, so these are reproduced as they are. Verified per function against the ROM with
//! `tools/check-fn.sh black-core softfloat` (spec `tools/rom-diff/specs/black-core/softfloat.spec`).

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0209_B750,
    0x0209_B790,
    0x0209_BDBC,
    0x0209_BE04,
    0x0209_C494,
    0x0209_C95C,
    0x0209_CB8C,
];

const S: u32 = 0x8000_0000;
const INF: u32 = 0x7F80_0000;
const NAN: u32 = 0x7FFF_FFFF;

/// ARM `lsl rX, rY, rZ`: the amount is the low byte of the register; 32 or more gives 0.
fn lsl(x: u32, n: u32) -> u32 {
    let n = n & 0xFF;
    if n >= 32 {
        0
    } else {
        x << n
    }
}

/// ARM `lsr rX, rY, rZ` (register amount, see [`lsl`]).
fn lsr(x: u32, n: u32) -> u32 {
    let n = n & 0xFF;
    if n >= 32 {
        0
    } else {
        x >> n
    }
}

/// The shared tail of every rounding path: `m` is the normalised mantissa with the leading 1 in
/// bit 31, `e` the sign and biased exponent (placed at bit 23, so a mantissa carry bumps the
/// exponent), `sign` extra bits OR-ed in. Rounds to nearest even on the 8 dropped bits.
fn pack_round(m: u32, e: u32, sign: u32) -> u32 {
    let low = m & 0xFF;
    let mut r = sign | (m.wrapping_add(m) >> 9) | (e << 23);
    if low & 0x80 != 0 && (low & 0x7F != 0 || r & 1 != 0) {
        r = r.wrapping_add(1);
    }
    r
}

/// C: `FUN_0209BDBC`: `int x` (r0 = x), converts a signed 32-bit integer to a single, rounding to
/// nearest even. Returns the float's bits.
pub fn i2f(x: i32) -> u32 {
    let x = x as u32;
    let sign = x & S;
    let mag = if sign != 0 { x.wrapping_neg() } else { x };
    if mag == 0 {
        return 0;
    }
    let z = mag.leading_zeros();
    pack_round(mag << z, 0x9E - z, sign)
}

/// C: `FUN_0209BE04`: `uint x` (r0 = x), converts an unsigned 32-bit integer to a single, rounding
/// to nearest even. Returns the float's bits.
pub fn u2f(x: u32) -> u32 {
    if x == 0 {
        return 0;
    }
    let (m, e) = if x & S != 0 {
        (x, 0x9E)
    } else {
        let z = x.leading_zeros();
        (x << z, 0x9E - z)
    };
    pack_round(m, e, 0)
}

/// C: `FUN_0209C494`: `float f` (r0 = bits), converts a single to a signed 32-bit integer,
/// truncating towards zero; out-of-range values, infinities and NaNs saturate to
/// `i32::MAX` (positive sign) or `i32::MIN` (negative sign).
pub fn f2i(a: u32) -> i32 {
    let mag = a & 0x7FFF_FFFF;
    let sh = 0x9Eu32.wrapping_sub(mag >> 23) as i32;
    if sh <= 0 {
        return (!(((a as i32) >> 31) as u32)).wrapping_add(S) as i32;
    }
    let v = lsr((mag << 8) | S, sh as u32);
    (if a & S != 0 { v.wrapping_neg() } else { v }) as i32
}

/// C: `FUN_0209B750`: `int x` (r0 = x), converts a signed 32-bit integer to a double (exact);
/// returns `r1 << 32 | r0`.
pub fn i2d(x: i32) -> u64 {
    let x = x as u32;
    let sign = x & S;
    let mag = if sign != 0 { x.wrapping_neg() } else { x };
    if mag == 0 {
        return 0;
    }
    let z = mag.leading_zeros();
    pack_d(mag << z, 0x41E - z, sign)
}

/// C: `FUN_0209B790`: `uint x` (r0 = x), converts an unsigned 32-bit integer to a double (exact);
/// returns `r1 << 32 | r0`.
pub fn u2d(x: u32) -> u64 {
    if x == 0 {
        return 0;
    }
    if x & S != 0 {
        pack_d(x, 0x41E, 0)
    } else {
        let z = x.leading_zeros();
        pack_d(x << z, 0x41E - z, 0)
    }
}

/// `m` has its leading 1 in bit 31: low word `m << 21`, high word `sign | m<<1>>12 | e << 20`.
fn pack_d(m: u32, e: u32, sign: u32) -> u64 {
    let hi = sign | (m.wrapping_add(m) >> 12) | (e << 20);
    ((hi as u64) << 32) | (m << 21) as u64
}

/// C: `FUN_0209C95C`: `float a, float b` (r0 = a, r1 = b), `a + b` on the bit patterns.
pub fn fadd(a: u32, b: u32) -> u32 {
    if ((a ^ b) as i32) < 0 {
        sub_path(a, b ^ S)
    } else {
        add_path(a, b)
    }
}

/// C: `FUN_0209CB8C`: `float a, float b` (r0 = a, r1 = b), `a - b`. Same code as [`fadd`] with the
/// sign test inverted, so it equals `fadd(a, b ^ 0x8000_0000)`.
pub fn fsub(a: u32, b: u32) -> u32 {
    if ((a ^ b) as i32) < 0 {
        add_path(a, b ^ S)
    } else {
        sub_path(a, b)
    }
}

fn sign_of(r3: u32) -> u32 {
    if (r3 as i32) >= 0x100 {
        S
    } else {
        0
    }
}

/// `0x0209C968`: both operands have the same sign: add the magnitudes.
fn add_path(mut r0: u32, mut r1: u32) -> u32 {
    let mut ip = r0.wrapping_sub(r1);
    if r0 < r1 {
        r0 = r0.wrapping_sub(ip);
        r1 = r1.wrapping_add(ip);
    }
    let r3 = r0 >> 23;
    r0 = S | (r0 << 8);
    ip = r3 & 0xFF;
    if ip == 0 || ip == 0xFF {
        return add_a_special(r0, r1, r3);
    }
    ip = r1 >> 23;
    r1 = S | (r1 << 8);
    if ip & 0xFF == 0 {
        // 0x0209CA3C: the second operand is zero or denormal
        return add_denorm_b(r0, r1, sign_of(r3), r3 & 0xFF);
    }
    add_align(r0, r1, r3, ip)
}

/// `0x0209C99C`: shift the smaller mantissa right (with a sticky bit), add, normalise, round.
fn add_align(mut r0: u32, mut r1: u32, mut r3: u32, ip: u32) -> u32 {
    let ip = r3.wrapping_sub(ip);
    if ip != 0 {
        let lost = lsl(r1, 0x20u32.wrapping_sub(ip));
        r1 = lsr(r1, ip);
        if lost != 0 {
            r1 |= 1;
        }
    }
    let (sum, carry) = r0.overflowing_add(r1);
    r0 = sum;
    if carry {
        r1 = r0 & 1;
        r0 = r1 | (r0 >> 1) | S;
        r3 = r3.wrapping_add(1);
        if r3 & 0xFF == 0xFF {
            // 0x0209CB44: overflow to infinity
            return sign_of(r3) | INF;
        }
    }
    pack_round(r0, r3, 0)
}

/// `0x0209C9FC`: the larger operand has exponent 0 or 255 (`r0` = its shifted mantissa, `r1` =
/// the raw second operand, `r3` = sign and exponent).
fn add_a_special(mut r0: u32, mut r1: u32, mut r3: u32) -> u32 {
    let r2 = sign_of(r3);
    r3 &= 0xFF;
    if r3 == 0 {
        return add_a_zero(r0, r1, r2);
    }
    // infinity or NaN
    r0 <<= 1;
    if r0 != 0 {
        return NAN;
    }
    let ip = (r1 >> 23) & 0xFF;
    r1 <<= 9;
    if ip == 0 || ip < 0xFF || r1 == 0 {
        return r2 | INF;
    }
    NAN
}

/// `0x0209CA60`: the larger operand has exponent 0 (zero or denormal).
fn add_a_zero(mut r0: u32, r1: u32, r2: u32) -> u32 {
    r0 <<= 1;
    if r0 == 0 {
        // 0x0209CA98: a is zero
        let r3 = (r1 >> 23) & 0xFF;
        let m = r1 << 9;
        if r3 == 0 || r3 < 0xFF {
            return r1;
        }
        return if m == 0 { r2 | INF } else { NAN };
    }
    let mut r3 = 1;
    r0 >>= 1;
    let mut ip = (r1 >> 23) & 0xFF;
    let b = r1 << 8;
    if ip == 0 {
        return add_denorm_b(r0, b, r2, r3);
    }
    if ip == 0xFF {
        // 0x0209CA54
        return if (b << 1) != 0 { NAN } else { r2 | INF };
    }
    r3 |= r2 >> 23;
    ip |= r2 >> 23;
    add_align(r0, b | S, r3, ip)
}

/// `0x0209CABC`: second operand has exponent 0 (`r1` = its shifted mantissa).
fn add_denorm_b(r0: u32, mut r1: u32, r2: u32, mut r3: u32) -> u32 {
    r1 <<= 1;
    if r1 == 0 {
        // 0x0209CB2C: b is zero
        if (r0 as i32) >= 0 {
            r3 = r3.wrapping_sub(1);
        }
        return r2 | (r0.wrapping_add(r0) >> 9) | (r3 << 23);
    }
    r1 >>= 1;
    let mut ip = 1 | (r2 >> 23);
    r3 |= r2 >> 23;
    if (r0 as i32) < 0 {
        return add_align(r0, r1, r3, ip);
    }
    let (mut sum, carry) = r0.overflowing_add(r1);
    if carry {
        sum = (sum & 1) | (sum >> 1) | S;
        ip = ip.wrapping_add(1);
    }
    if (sum as i32) >= 0 {
        ip = ip.wrapping_sub(1);
    }
    pack_round(sum, ip, 0)
}

/// `0x0209CB98`: the operands have the same sign (after the caller's flip): subtract magnitudes.
fn sub_path(mut r0: u32, mut r1: u32) -> u32 {
    let mut ip = r0.wrapping_sub(r1);
    if r0 < r1 {
        ip ^= S;
        r0 = r0.wrapping_sub(ip);
        r1 = r1.wrapping_add(ip);
    }
    let r3 = r0 >> 23;
    r0 = S | (r0 << 8);
    ip = r3 & 0xFF;
    if ip == 0 || ip == 0xFF {
        return sub_a_special(r0, r1, r3);
    }
    ip = r1 >> 23;
    r1 = S | (r1 << 8);
    if ip & 0xFF == 0 {
        // 0x0209CCF4
        return sub_denorm_b(r0, r1, sign_of(ip), r3 & 0xFF);
    }
    sub_align(r0, r1, r3, ip)
}

/// `0x0209CBD0`: align, subtract, normalise.
fn sub_align(r0: u32, mut r1: u32, r3: u32, ip: u32) -> u32 {
    let ip = r3.wrapping_sub(ip);
    if ip == 0 {
        // 0x0209CC18: equal exponents
        let d = r0.wrapping_sub(r1);
        if d == 0 {
            return 0;
        }
        return sub_norm(d, r3, false);
    }
    let lost = lsl(r1, 0x20u32.wrapping_sub(ip));
    r1 = lsr(r1, ip);
    if lost != 0 {
        r1 |= 1;
    }
    let d = r0.wrapping_sub(r1);
    if (d as i32) < 0 {
        // 0x0209CBF0: the shifted operand was larger (only with a denormal first operand)
        return pack_round(d, r3, 0);
    }
    sub_norm(d, r3, true)
}

/// `0x0209CC20` (`round` = false) / `0x0209CC5C` (`round` = true): renormalise `d`, build the
/// result (sign from bit 8 of `r3`); a denormal result is shifted without rounding.
fn sub_norm(d: u32, r3: u32, round: bool) -> u32 {
    let r2 = (r3 << 23) & S;
    let mut r3 = r3 & !0x100;
    let z = d.leading_zeros();
    let r0 = d << z;
    r3 = r3.wrapping_sub(z);
    if (r3 as i32) > 0 {
        if round {
            return pack_round(r0, r3, r2);
        }
        return r2 | (r0.wrapping_add(r0) >> 9) | (r3 << 23);
    }
    r2 | lsr(r0, 9u32.wrapping_sub(r3))
}

/// `0x0209CCB4`: the larger operand has exponent 0 or 255.
fn sub_a_special(mut r0: u32, mut r1: u32, mut r3: u32) -> u32 {
    let mut r2 = sign_of(r3);
    r3 &= 0xFF;
    if r3 == 0 {
        // 0x0209CD1C: a is zero or denormal
        r0 <<= 1;
        if r0 == 0 {
            // 0x0209CD54: a is zero
            let t = r1 >> 23;
            let m = r1 << 9;
            let e = t & 0xFF;
            if e == 0 {
                return if m == 0 { 0 } else { r1 };
            }
            if e < 0xFF {
                return r1;
            }
            // the original returns `e | 0x7F80_0000` here (r2 is clobbered); unreachable for
            // ordered operands
            return if m != 0 { NAN } else { e | INF };
        }
        r0 >>= 1;
        r3 = 1;
        let mut ip = (r1 >> 23) & 0xFF;
        r1 <<= 8;
        if ip == 0 {
            return sub_denorm_b(r0, r1, r2, r3);
        }
        if ip == 0xFF {
            r2 ^= S;
            return if (r1 << 1) != 0 { NAN } else { r2 | INF };
        }
        r1 |= S;
        r3 |= r2 >> 23;
        ip |= r2 >> 23;
        return sub_align(r0, r1, r3, ip);
    }
    // infinity or NaN
    r0 <<= 1;
    if r0 != 0 {
        return NAN;
    }
    let ip = (r1 >> 23) & 0xFF;
    r1 <<= 9;
    if ip == 0 || ip < 0xFF {
        return r2 | INF;
    }
    // inf - inf (or NaN second operand)
    let _ = r1;
    NAN
}

/// `0x0209CD84`: second operand has exponent 0 (`r1` = its shifted mantissa).
fn sub_denorm_b(r0: u32, mut r1: u32, r2: u32, mut r3: u32) -> u32 {
    r1 <<= 1;
    if r1 == 0 {
        // 0x0209CDA8: b is zero
        if (r0 as i32) >= 0 {
            r3 = r3.wrapping_sub(1);
        }
        return r2 | (r0.wrapping_add(r0) >> 9) | (r3 << 23);
    }
    r1 >>= 1;
    let ip = 1 | (r2 >> 23);
    r3 |= r2 >> 23;
    sub_align(r0, r1, r3, ip)
}

#[cfg(test)]
mod tests;
