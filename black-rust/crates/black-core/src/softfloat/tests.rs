use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0209B750, "5ac9cafb5828957c"),
    (0x0209B790, "5dc8adc77f077745"),
    (0x0209BDBC, "9fff2d59445e617b"),
    (0x0209BE04, "a7da350a0303f18a"),
    (0x0209C494, "e6161d21aa61937d"),
    (0x0209C95C, "e5530df467189e7d"),
    (0x0209CB8C, "c92941c7033b7674"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let (a, b) = (c.vals[0], c.vals.get(1).copied().unwrap_or(0));
    match addr {
        0x0209_B750 => i2d(a as i32),
        0x0209_B790 => u2d(a),
        0x0209_BDBC => i2f(a as i32) as u64,
        0x0209_BE04 => u2f(a) as u64,
        0x0209_C494 => f2i(a) as u32 as u64,
        0x0209_C95C => fadd(a, b) as u64,
        0x0209_CB8C => fsub(a, b) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-core softfloat`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "softfloat",
            include_str!("../../../../tools/rom-diff/specs/black-core/softfloat.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn int_to_float_known_values() {
    assert_eq!(i2f(0), 0);
    assert_eq!(i2f(1), 0x3F80_0000);
    assert_eq!(i2f(-1), 0xBF80_0000);
    assert_eq!(i2f(100), 0x42C8_0000);
    assert_eq!(i2f(i32::MIN), 0xCF00_0000);
    assert_eq!(i2f(i32::MAX), 0x4F00_0000);
    // 2^24 + 1 is a tie: rounds to even (2^24); 2^24 + 3 rounds up to 2^24 + 4
    assert_eq!(i2f(0x0100_0001), 0x4B80_0000);
    assert_eq!(i2f(0x0100_0003), 0x4B80_0002);
    assert_eq!(u2f(0), 0);
    assert_eq!(u2f(0x8000_0000), 0x4F00_0000);
    assert_eq!(u2f(u32::MAX), 0x4F80_0000);
    assert_eq!(u2f(3), 0x4040_0000);
}

#[test]
fn float_to_int_known_values() {
    assert_eq!(f2i(0x3F80_0000), 1);
    assert_eq!(f2i(0x3FBF_FFFF), 1);
    assert_eq!(f2i(0xBFC0_0000), -1);
    assert_eq!(f2i(0x3F7F_FFFF), 0);
    assert_eq!(f2i(0x4F00_0000), i32::MAX);
    assert_eq!(f2i(0xCF00_0000), i32::MIN);
    assert_eq!(f2i(0x7F80_0000), i32::MAX);
    assert_eq!(f2i(0xFF80_0000), i32::MIN);
    assert_eq!(f2i(0x4EFF_FFFF), 0x7FFF_FF80);
}

#[test]
fn add_sub_known_values() {
    let (one, two, three) = (0x3F80_0000, 0x4000_0000, 0x4040_0000);
    assert_eq!(fadd(one, two), three);
    assert_eq!(fsub(three, two), one);
    assert_eq!(fsub(one, one), 0);
    assert_eq!(fadd(one, one ^ S), 0);
    assert_eq!(fadd(0x3F00_0000, 0x3F00_0000), one);
    // 1 + 2^-24 is a tie: rounds to even (1.0)
    assert_eq!(fadd(one, 0x3380_0000), one);
    assert_eq!(fadd(0x7F80_0000, 0xFF80_0000), 0x7FFF_FFFF);
    assert_eq!(fadd(0x7F7F_FFFF, 0x7F7F_FFFF), 0x7F80_0000);
    assert_eq!(fsub(0x4000_0000, 0x3F80_0000), one);
}

#[test]
fn int_to_double_known_values() {
    assert_eq!(i2d(0), 0);
    assert_eq!(i2d(1), 0x3FF0_0000_0000_0000);
    assert_eq!(i2d(-2), 0xC000_0000_0000_0000);
    assert_eq!(i2d(i32::MIN), 0xC1E0_0000_0000_0000);
    assert_eq!(u2d(u32::MAX), 0x41EF_FFFF_FFE0_0000);
    assert_eq!(u2d(0x8000_0000), 0x41E0_0000_0000_0000);
}
