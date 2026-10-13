use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02199888, "e80c29d068bc1c7d"),
    (0x02199970, "b4481a123a24c74d"),
    (0x02199AEC, "bc80c2c4f3a66879"),
    (0x02199B10, "d8d5c0200939cce2"),
    (0x02199B38, "432d294b9320ff3d"),
    (0x02199B60, "c5dbab588474e2c1"),
    (0x02199B88, "97f5269b0ff84ad9"),
    (0x02199BAC, "9dd38f7dc63015e7"),
    (0x02199BD8, "937b18313e6b8080"),
    (0x02199C00, "d5fb32e34a100077"),
    (0x02199C28, "1b6d58ea549e5124"),
    (0x02199C50, "9fdcf489f3e8363c"),
    (0x02199C7C, "f577979e9b4b3ad1"),
    (0x02199CA4, "3504c1ad15b0f5c3"),
    (0x02199CC8, "795c05ccc0aa0ab4"),
    (0x02199CE4, "bf95d98817c9511c"),
    (0x02199D00, "bf9b5ed3f8fee119"),
    (0x02199D14, "306df02baf5e3ece"),
    (0x02199D4C, "8b2eac3337428d76"),
    (0x02199D8C, "b94584131248e7c2"),
    (0x02199DB4, "372a44867c2ccfc9"),
    (0x02199EAC, "fee8368c838292de"),
    (0x02199EC0, "28e41f4df8c46bc8"),
    (0x02199ED4, "0a664a42b2ef65b0"),
    (0x02199F30, "a891868c8140d872"),
    (0x02199F50, "411be475634a033d"),
    (0x02199F94, "d65cfc5a6b68ee2c"),
    (0x0219A014, "c2c63366f7f5b3d0"),
    (0x0219A028, "9592539dc11e8b5f"),
    (0x0219A03C, "26a243efb3f1a217"),
    (0x0219A058, "b302480a54dd97bd"),
    (0x0219A07C, "0f2643d459e8ea33"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2e.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_9888 => {
            arc_start(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8]);
            0
        }
        0x0219_9970 => arc_update(e, v[0]) as u64,
        0x0219_9AEC => arc_85(e, v[0]) as u64,
        0x0219_9CE4 => count_22(e, v[0]) as u64,
        0x0219_9D14 => slow_left_start(e, v[0]) as u64,
        0x0219_9F94 => model_wait(e, v[0]) as u64,
        0x0219_9CC8 => count_22_start(e, v[0]) as u64,
        0x0219_9D00 => anim10_ac(e, v[0]) as u64,
        0x0219_9D4C => slow_left_update(e, v[0]) as u64,
        0x0219_9D8C => model_rise_start(e, v[0]) as u64,
        0x0219_9DB4 => model_rise_update(e, v[0]) as u64,
        0x0219_9EAC => anim10_af(e, v[0]) as u64,
        0x0219_9EC0 => anim11_b0(e, v[0]) as u64,
        0x0219_9ED4 => anim_face_down(e, v[0]) as u64,
        0x0219_9F30 => offset_reset(e, v[0]) as u64,
        0x0219_9F50 => model_wait_start(e, v[0]) as u64,
        0x0219_A014 => anim10_b1(e, v[0]) as u64,
        0x0219_A028 => anim11_b2(e, v[0]) as u64,
        0x0219_A03C => anim12_b3(e, v[0]) as u64,
        0x0219_A058 => model_done_b1(e, v[0]) as u64,
        0x0219_A07C => model_done(e, v[0]) as u64,
        0x0219_9B10 => arc_86(e, v[0]) as u64,
        0x0219_9B38 => arc_87(e, v[0]) as u64,
        0x0219_9B60 => arc_88(e, v[0]) as u64,
        0x0219_9B88 => arc_89(e, v[0]) as u64,
        0x0219_9BAC => arc_8a(e, v[0]) as u64,
        0x0219_9BD8 => arc_8b(e, v[0]) as u64,
        0x0219_9C00 => arc_8c(e, v[0]) as u64,
        0x0219_9C28 => arc_8d(e, v[0]) as u64,
        0x0219_9C50 => arc_8e(e, v[0]) as u64,
        0x0219_9C7C => arc_8f(e, v[0]) as u64,
        0x0219_9CA4 => arc_90(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2e`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2e", SPEC, PORTED, RECORDED), &call);
}

/// [`MODEL_OFFSET`] is the build's (`0x021D4ACC`, the literal of `02199F50`).
#[test]
fn model_offset_matches_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    assert_eq!(w(0x0219_9F90), 0x021D_4ACC);
    for (k, &c) in MODEL_OFFSET.iter().enumerate() {
        assert_eq!(w(0x021D_4ACC + 4 * k as u32), c);
    }
}

/// `_s32_div_f` with a zero divisor returns the dividend (V-0026), else truncates toward zero.
#[test]
fn s32_div_zero_returns_dividend() {
    assert_eq!(s32_div(0x1000, 0), 0x1000);
    assert_eq!(s32_div(0x1000, 8), 0x200);
    assert_eq!(s32_div(0x1000, 0xFF), 0x10);
    assert_eq!(s32_div(-7, 2), -3);
}
