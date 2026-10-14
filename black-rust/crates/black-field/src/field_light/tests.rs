use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A03D0, "ba51a7c5375e275d"),
    (0x021A0448, "0585bf1c8b29e4d6"),
    (0x021A0458, "3fd6a7ad8dd333f3"),
    (0x021A0568, "6d3f9687c0d11db5"),
    (0x021A0590, "3b03d3e88ccb2992"),
    (0x021A05D8, "4ef4b59a1233c3be"),
    (0x021A0618, "7d900173fe3b7cde"),
    (0x021A063C, "1d6b36c800e01216"),
    (0x021A065C, "e43a3b2cad08e0b6"),
    (0x021A066C, "c02bc34835019c18"),
    (0x021A0734, "3e5e22c8b5eea2cb"),
    (0x021A0748, "a92e51a3c8806c04"),
    (0x021A07B0, "c99542a0deb47d2d"),
    (0x021A07B4, "8a9124572a6c74db"),
    (0x021A07F8, "68383bb2a5787bd2"),
    (0x021A0818, "06e1919dc085657b"),
    (0x021A0860, "2062524d462e8e3b"),
    (0x021A08CC, "27b8d3ba10f93687"),
    (0x021A0918, "6764b0e2dafcb3ba"),
    (0x021A0974, "b53ad6c9b3f7aef6"),
    (0x021A0980, "1838499bc99b8708"),
    (0x021A0A50, "acebe05ae9873928"),
    (0x021A0B14, "9998ddbe40a617a4"),
    (0x021A0B34, "71996c697de56ddd"),
    (0x021A0B4C, "f15c98f4b8c13a51"),
    (0x021A0C50, "bd6ee531833202c1"),
    (0x021A0C60, "383a3670955c000d"),
    (0x021A0C74, "36a165731a62e19a"),
    (0x021A0CE4, "4ad3e2afa210498e"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/field_light.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        match addr {
            0x021A_03D0 => return new(e, v[0], v[1], v[2], v[3], v[4], v[5]) as u64,
            0x021A_0448 => delete(e, v[0]),
            0x021A_0458 => update(e, v[0], v[1]),
            0x021A_0568 => upload_if_dirty(e, v[0], v[1]),
            0x021A_0590 => change_table(e, v[0], v[1], v[2], v[3]),
            0x021A_05D8 => restore_table(e, v[0]),
            0x021A_0618 => fade_request(e, v[0], v[1], v[2], v[3]),
            0x021A_063C => fade_request_once(e, v[0], v[1], v[2]),
            0x021A_065C => return fade_active(e, v[0]) as u64,
            0x021A_066C => upload(e, v[0], v[1], v[2]),
            0x021A_0734 => load_default(e, v[0], v[1]),
            0x021A_0748 => load(e, v[0], v[1], v[2]),
            0x021A_07B0 => nop(),
            0x021A_07B4 => return keyframe_at(e, v[0], v[1]) as u64,
            0x021A_07F8 => return keyframe_time(e, v[0], v[1]) as u64,
            0x021A_0818 => ramp_init(e, v[0], v[1], v[2]),
            0x021A_0860 => return ramp_at(e, v[0], v[1], v[2]) as u64,
            0x021A_08CC => rot_init(e, v[0], v[1], v[2]),
            0x021A_0918 => rot_at(e, v[0], v[1], v[2], v[3]),
            0x021A_0974 => crossfade60(e, v[0], v[1], v[2]),
            0x021A_0980 => crossfade(e, v[0], v[1], v[2], v[3]),
            0x021A_0A50 => fade_to_colour(e, v[0], v[1], v[2], v[3]),
            0x021A_0B14 => step(e, v[0]),
            0x021A_0B34 => return running(e, v[0]) as u64,
            0x021A_0B4C => light_at(e, v[0], v[1]),
            0x021A_0C50 => request_set(e, v[0], v[1], v[2], v[3]),
            0x021A_0C60 => request_set_once(e, v[0], v[1], v[2]),
            0x021A_0C74 => return request_step(e, v[0], v[1], v[2], v[3]) as u64,
            0x021A_0CE4 => return request_active(e, v[0]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
        0
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field field_light`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("field_light", SPEC, PORTED, RECORDED), &call);
}

/// [`fx_sin`] is `FX_SinCosTable_` (main 0x020A1B38, `$BLACK_ROM_BUILD/main.sbin` loaded at 0x02004000).
#[test]
fn sincos_matches_rom() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(m) = dir.and_then(|d| std::fs::read(format!("{d}/main.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "main.sbin missing");
        return;
    };
    let o = (0x020A_1B38 - 0x0200_4000) as usize;
    for i in 0..4096usize {
        let s = i16::from_le_bytes([m[o + 4 * i], m[o + 4 * i + 1]]);
        let c = i16::from_le_bytes([m[o + 4 * i + 2], m[o + 4 * i + 3]]);
        assert_eq!((fx_sin(i as u32), fx_cos(i as u32)), (s, c), "entry {i}");
    }
}
