use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0207AF50, "1c58a2eb1681341e"),
    (0x0207B65C, "3fa5c8814ccb6a19"),
    (0x0207BC08, "c72856297aeb6109"),
    (0x0207C768, "c88b679dc7288239"),
    (0x0207C778, "f984f51593b346d5"),
    (0x0207C7B4, "05e06a0085e9798e"),
    (0x0207C7E8, "2cb324d89cf9659c"),
    (0x0207C958, "65ad7133f1c83410"),
    (0x0207C994, "02c483431347bccf"),
    (0x0207C9D0, "b3464002c617e9f4"),
    (0x0207CA00, "1cc0ef4165de7674"),
    (0x0207CA30, "ed407ee85f71b5e2"),
    (0x0207CA68, "9b6ffd8cbbff9392"),
    (0x0207CAA4, "5d563a3cd386ac37"),
    (0x0207CAE8, "a34429ee5f08bdda"),
    (0x0207CB68, "f4286e1346b69900"),
    (0x0207CBD0, "5ec1c7d7a6b8b552"),
    (0x0207CC30, "ce2c158e2a1b1673"),
    (0x0207CCA4, "d785c13595db2f79"),
    (0x0207CDB8, "19cce8f0a1c32c67"),
    (0x0207CEE4, "6fb4d24c63675122"),
    (0x0207CF3C, "98b48469e1f4c759"),
    (0x0207CFB8, "7dc3a638b67d945d"),
    (0x0207D184, "ad3015007275a5a6"),
];
// fndiff-recorded-end

fn v3(b: &[u8]) -> Vec3 {
    [0, 1, 2].map(|i| i32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap()))
}

fn v16(b: &[u8]) -> Vec16 {
    [0, 1, 2].map(|i| i16::from_le_bytes(b[2 * i..2 * i + 2].try_into().unwrap()))
}

fn put3(b: &mut [u8], v: &[i32]) {
    for (i, x) in v.iter().enumerate() {
        b[4 * i..4 * i + 4].copy_from_slice(&x.to_le_bytes());
    }
}

fn put16(b: &mut [u8], v: &[i16]) {
    for (i, x) in v.iter().enumerate() {
        b[2 * i..2 * i + 2].copy_from_slice(&x.to_le_bytes());
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0207_AF50 => {
            let m = mtx_rot_axis33(&v3(c.region("axis")), c.vals[2] as i32, c.vals[3] as i32);
            put3(c.region_mut("dst"), &m);
            0
        }
        0x0207_B65C => {
            let m = mtx_rot_axis43(&v3(c.region("axis")), c.vals[2] as i32, c.vals[3] as i32);
            put3(c.region_mut("dst"), &m);
            0
        }
        0x0207_BC08 => {
            let m: Mtx43 = {
                let b = c.region("m");
                std::array::from_fn(|i| i32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap()))
            };
            let r = mtx_mult_vec43(&v3(c.region("v")), &m);
            put3(c.region_mut("dst"), &r);
            0
        }
        0x0207_C9D0 => {
            let r = vec_add(&v3(c.region("a")), &v3(c.region("b")));
            put3(c.region_mut("dst"), &r);
            0
        }
        0x0207_CA00 => {
            let r = vec_subtract(&v3(c.region("a")), &v3(c.region("b")));
            put3(c.region_mut("dst"), &r);
            0
        }
        0x0207_CA30 => {
            let r = vec_fx16_add(&v16(c.region("a")), &v16(c.region("b")));
            put16(c.region_mut("dst"), &r);
            0
        }
        0x0207_CA68 => vec_dot_product(&v3(c.region("a")), &v3(c.region("b"))) as u32 as u64,
        0x0207_CAA4 => vec_fx16_dot_product(&v16(c.region("a")), &v16(c.region("b"))) as u32 as u64,
        0x0207_CAE8 => {
            let r = vec_cross_product(&v3(c.region("a")), &v3(c.region("b")));
            put3(c.region_mut("dst"), &r);
            0
        }
        0x0207_CB68 => {
            let r = vec_fx16_cross_product(&v16(c.region("a")), &v16(c.region("b")));
            put16(c.region_mut("dst"), &r);
            0
        }
        0x0207_CEE4 => {
            let r = vec_mult_add(c.vals[0] as i32, &v3(c.region("a")), &v3(c.region("b")));
            put3(c.region_mut("dst"), &r);
            0
        }
        0x0207_D184 => fx_acos_idx(c.vals[0] as i32) as u64,
        0x0207_C768 => fx_div(c.vals[0] as i32, c.vals[1] as i32) as u32 as u64,
        0x0207_C778 => fx_inv(c.vals[0] as i32) as u32 as u64,
        0x0207_C7B4 => fx_sqrt(c.vals[0] as i32) as u32 as u64,
        0x0207_C7E8 => fx_inv_sqrt(c.vals[0] as i32) as u32 as u64,
        0x0207_C958 => cp_div32(c.vals[0] as i32, c.vals[1] as i32) as u32 as u64,
        0x0207_C994 => cp_mod32(c.vals[0] as i32, c.vals[1] as i32) as u32 as u64,
        0x0207_CBD0 => vec_mag(&v3(c.region("v"))) as u32 as u64,
        0x0207_CC30 => vec_fx16_mag(&v16(c.region("v"))) as u32 as u64,
        0x0207_CCA4 => {
            let r = vec_normalize(&v3(c.region("v")));
            put3(c.region_mut("dst"), &r);
            0
        }
        0x0207_CDB8 => {
            let r = vec_fx16_normalize(&v16(c.region("v")));
            put16(c.region_mut("dst"), &r);
            0
        }
        0x0207_CF3C => vec_distance(&v3(c.region("a")), &v3(c.region("b"))) as u32 as u64,
        0x0207_CFB8 => fx_atan2_idx(c.vals[0] as i32, c.vals[1] as i32) as u16 as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function (the divider / sqrt users too: the emulator models the unit) against the original (`tools/check-fn.sh black-core fx`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "fx",
            include_str!("../../../../tools/rom-diff/specs/black-core/fx.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

fn rom_main() -> Option<Vec<u8>> {
    let got = std::env::var("BLACK_ROM_BUILD")
        .ok()
        .and_then(|b| std::fs::read(format!("{b}/main.sbin")).ok());
    if got.is_none() && std::env::var("REQUIRE_ROM").is_ok_and(|v| !v.is_empty() && v != "0") {
        panic!("REQUIRE_ROM is set but $BLACK_ROM_BUILD/main.sbin is missing");
    }
    got
}

/// [`sin`] / [`cos`] are all 4096 entries of `FX_SinCosTable_` (0x020A1B38), [`atan_entry`] the 129 of 0x020A5B38.
#[test]
fn tables_match_rom() {
    assert_eq!(
        (sin(0), cos(0), sin(1024), cos(1024), cos(2048)),
        (0, 4096, 4096, 0, -4096)
    );
    let Some(main) = rom_main() else { return };
    let at = |a: u32| (a - 0x0200_4000) as usize;
    for i in 0..4096u32 {
        let e = at(0x020A_1B38) + 4 * i as usize;
        let s = i16::from_le_bytes([main[e], main[e + 1]]);
        let c = i16::from_le_bytes([main[e + 2], main[e + 3]]);
        assert_eq!((sin(i), cos(i)), (s, c), "sin/cos entry {i}");
    }
    for i in 0..=128i32 {
        let e = at(0x020A_5B38) + 2 * i as usize;
        assert_eq!(
            atan_entry(i),
            i16::from_le_bytes([main[e], main[e + 1]]),
            "atan entry {i}"
        );
    }
}

// ---------------------------------------------------------------- known values and hardware rules

#[test]
fn division_unit_rules() {
    assert_eq!(div32(-7, 2), (-3i64 as u64, -1i64 as u64));
    assert_eq!(div32(5, 0), (0x0000_0000_FFFF_FFFF, 5));
    assert_eq!(div32(-5, 0), (0xFFFF_FFFF_0000_0001, -5i64 as u64));
    assert_eq!(div32(i32::MIN, -1), (0x8000_0000, 0));
    assert_eq!(div64_32(i64::MIN, -1), (i64::MIN, 0));
    assert_eq!(div64_32(7, 0), (-1, 7));
    assert_eq!(div64_64(-7, 0), (1, -7));
    assert_eq!(div64_64(-9, 4), (-2, -1));
    assert_eq!(
        (
            sqrt64(0),
            sqrt64(15),
            sqrt64(16),
            sqrt64(u64::MAX),
            sqrt32(u32::MAX)
        ),
        (0, 3, 4, u32::MAX, 65535)
    );
}

#[test]
fn fx_known_values() {
    assert_eq!(fx_div(3 << 12, 2 << 12), 0x1800);
    assert_eq!(fx_div(1 << 12, 0), 0);
    assert_eq!(fx_inv(2 << 12), 0x800);
    assert_eq!(fx_sqrt(4 << 12), 2 << 12);
    assert_eq!(fx_sqrt(-1), 0);
    assert_eq!(fx_inv_sqrt(4 << 12), 0x800);
    assert_eq!(vec_mag(&[3 << 12, 0, 4 << 12]), 5 << 12);
    assert_eq!(vec_normalize(&[0, 0, 0]), [0, 0, 0]);
    assert_eq!(vec_normalize(&[0, 0x2000, 0]), [0, 0x1000, 0]);
    assert_eq!(vec_distance(&[0x1000, 0, 0], &[0, 0, 0]), 0x1000);
    assert_eq!(fx_acos_idx(0), 0x4008); // table entry 1024 (the quarter turn) is the only zero cosine
    assert_eq!(fx_atan2_idx(0, 5), 0);
    assert_eq!(fx_atan2_idx(5, 5), 0x2000);
    assert_eq!(fx_atan2_idx(5, 0), 0x4000);
    assert_eq!(fx_atan2_idx(0, -5), -0x8000);
    assert_eq!(fx_atan2_idx(-5, -5), -0x6000);
    assert_eq!(fx_mul(0x1800, 0x2000), 0x3000);
    // rotation by 90 degrees about z
    assert_eq!(
        mtx_rot_axis33(&[0, 0, 0x1000], 0x1000, 0),
        [0, 0x1000, 0, -0x1000, 0, 0, 0, 0, 0x1000]
    );
}
