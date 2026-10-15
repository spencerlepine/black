use super::*;
use black_fndiff::{run_suite, Case, Rng, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0207AF50, "1c58a2eb1681341e"),
    (0x0207B65C, "3fa5c8814ccb6a19"),
    (0x0207BC08, "c72856297aeb6109"),
    (0x0207C9D0, "b3464002c617e9f4"),
    (0x0207CA00, "1cc0ef4165de7674"),
    (0x0207CA30, "ed407ee85f71b5e2"),
    (0x0207CA68, "9b6ffd8cbbff9392"),
    (0x0207CAA4, "5d563a3cd386ac37"),
    (0x0207CAE8, "a34429ee5f08bdda"),
    (0x0207CB68, "f4286e1346b69900"),
    (0x0207CEE4, "6fb4d24c63675122"),
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
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function without the hardware unit against the original (`tools/check-fn.sh black-core fx`).
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

// ---------------------------------------------------------------- hardware users

const BOUND: [u32; 23] = [
    0,
    1,
    0xFFFF_FFFF,
    0x7FFF_FFFF,
    0x8000_0000,
    0x8000_0001,
    0x1000,
    0xFFFF_F000,
    0x800,
    0xFFFF_F800,
    0x10000,
    0xFFFF_0000,
    0x4000_0000,
    0xC000_0000,
    2,
    3,
    0xFFFF_FFFE,
    0x1001,
    0xFFFF_EFFF,
    0x7FFF,
    0x8000,
    0xFFFF_8000,
    0xFFFF,
];

/// Same draw as `word` in `tools/rom-diff/fx_hw_diff.py`.
fn word(r: &mut Rng) -> u32 {
    match r.next() % 8 {
        0 | 1 => BOUND[(r.next() % 23) as usize],
        2 | 3 => r.next(),
        4 => r.next() % 0x20000,
        5 => (r.next() % 0x20000).wrapping_neg(),
        6 => r.next() & 0xFF_FFFF,
        _ => BOUND[(r.next() % 23) as usize]
            .wrapping_add(r.next() % 9)
            .wrapping_sub(4),
    }
}

struct Fnv(u64);

impl Fnv {
    fn w(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 = (self.0 ^ b as u64).wrapping_mul(1099511628211);
        }
    }
}

/// Port results for the 3000 inputs of `fx_hw_diff.py`, hashed like it.
fn hw_hash(addr: u32) -> String {
    let mut r = Rng::new(addr);
    let mut h = Fnv(14695981039346656037);
    for _ in 0..3000 {
        match addr {
            0x0207_C768 | 0x0207_C958 | 0x0207_C994 => {
                let (a, b) = (word(&mut r) as i32, word(&mut r) as i32);
                h.w(match addr {
                    0x0207_C768 => fx_div(a, b),
                    0x0207_C958 => cp_div32(a, b),
                    _ => cp_mod32(a, b),
                } as u32);
            }
            0x0207_C778 | 0x0207_C7B4 | 0x0207_C7E8 => {
                let x = word(&mut r) as i32;
                h.w(match addr {
                    0x0207_C778 => fx_inv(x),
                    0x0207_C7B4 => fx_sqrt(x),
                    _ => fx_inv_sqrt(x),
                } as u32);
            }
            0x0207_CBD0 | 0x0207_CC30 => {
                let w = [word(&mut r), word(&mut r), word(&mut r)];
                h.w(if addr == 0x0207_CBD0 {
                    vec_mag(&w.map(|x| x as i32))
                } else {
                    vec_fx16_mag(&w.map(|x| x as u16 as i16))
                } as u32);
            }
            0x0207_CCA4 => {
                let w = [word(&mut r), word(&mut r), word(&mut r)];
                for x in vec_normalize(&w.map(|x| x as i32)) {
                    h.w(x as u32);
                }
            }
            0x0207_CDB8 => {
                let w = [word(&mut r), word(&mut r), word(&mut r)];
                for x in vec_fx16_normalize(&w.map(|x| x as u16 as i16)) {
                    h.w(x as u16 as u32);
                }
            }
            0x0207_CF3C => {
                let w: Vec<i32> = (0..6).map(|_| word(&mut r) as i32).collect();
                h.w(vec_distance(&[w[0], w[1], w[2]], &[w[3], w[4], w[5]]) as u32);
            }
            0x0207_CFB8 => {
                let fix = |x: u32| if x == 0x8000_0000 { 0x8000_0001 } else { x } as i32;
                let (y, x) = (fix(word(&mut r)), fix(word(&mut r)));
                h.w(fx_atan2_idx(y, x) as u16 as u32);
            }
            _ => unreachable!(),
        }
    }
    format!("{:016x}", h.0)
}

/// Hashes printed by `tools/rom-diff/fx_hw_diff.py` (the original code under Unicorn with the divider / sqrt unit modelled).
const HW_RECORDED: &[(u32, &str)] = &[
    (0x0207C768, "d0d495166a4ad55e"),
    (0x0207C778, "9ad34aefe1aede94"),
    (0x0207C7B4, "d1e40d5845d690e3"),
    (0x0207C7E8, "3af0bf6685ef1f28"),
    (0x0207C958, "40888aa497bef33b"),
    (0x0207C994, "01c13a135254e586"),
    (0x0207CBD0, "0abe14f05561eb19"),
    (0x0207CC30, "4d3f18229bb92e8c"),
    (0x0207CCA4, "be16f5e30525440b"),
    (0x0207CDB8, "e8c719c23276f1f8"),
    (0x0207CF3C, "67c4b4a8c745f6ab"),
    (0x0207CFB8, "92387acd14cda12e"),
];

/// The hardware users equal the ROM run of `fx_hw_diff.py` (recorded hashes; the script needs the ROM, this test does not).
#[test]
fn hw_match_rom_model() {
    assert_eq!(
        HW_RECORDED.len(),
        super::HW_PORTED.len(),
        "one recorded hash per HW_PORTED function"
    );
    for &(a, want) in HW_RECORDED {
        assert!(super::HW_PORTED.contains(&a));
        assert_eq!(hw_hash(a), want, "{a:#010x}");
    }
}

#[test]
fn print_hw_hashes() {
    if std::env::var("FX_PRINT_HW").is_ok() {
        for &a in super::HW_PORTED {
            println!("HW {a:08x} {}", hw_hash(a));
        }
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
