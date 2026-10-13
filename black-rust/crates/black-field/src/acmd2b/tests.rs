use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02198318, "c8a9f52d9932846b"),
    (0x0219834C, "95e7b40431641801"),
    (0x02198380, "007eb8597db33785"),
    (0x02198390, "9d11b5aa0aff7e32"),
    (0x021983A0, "5ea3a1a9f1533676"),
    (0x021983B0, "61794923a35ff42f"),
    (0x021983C0, "f5540599ba5140d7"),
    (0x021983D0, "39d177fc4ecaafc0"),
    (0x021983E0, "680b28cdb0cc2292"),
    (0x021983F0, "3b0b51893ebf7ffc"),
    (0x02198400, "67bf3ae6485e7b7e"),
    (0x02198410, "6c7125aff5c974f8"),
    (0x02198420, "792fe81d5a21924c"),
    (0x02198430, "a465e5274df968d8"),
    (0x02198440, "a1ee088f0baeef5b"),
    (0x02198450, "aac20b8351f315a5"),
    (0x02198460, "dace117354e53da6"),
    (0x02198470, "d22b36456c9b64eb"),
    (0x02198480, "ca23b80b5ccce642"),
    (0x02198490, "fbd56eb90e374bb3"),
    (0x021984A0, "0cefcf83d6e8467b"),
    (0x021984B0, "8941237738612152"),
    (0x021984C0, "35b2d5fc63789f24"),
    (0x0219852C, "6a96f6d1dd051f71"),
    (0x02198550, "03e9fc650c6433f2"),
    (0x0219862C, "2a30aa3475798911"),
    (0x02198650, "0d876499c180326d"),
    (0x02198674, "8d046d45956c8694"),
    (0x02198694, "94e97645171df17d"),
    (0x021986B8, "4563030123e33b6e"),
    (0x021986D8, "e6fc814a050b8318"),
    (0x021986F8, "d11a7807f59f4a82"),
    (0x02198718, "f44a3955f55e6a4d"),
    (0x02198738, "1d8cb7c8c5f8738c"),
    (0x02198758, "8a0dcb399d212112"),
    (0x0219877C, "efeba4bfc41f86b5"),
    (0x021987A0, "b3a15341dd9ae4a9"),
    (0x021987C0, "6a3ed06bf925098d"),
    (0x021987E0, "4c47e3aa5497509a"),
    (0x02198804, "76a5b90d28f3c7a7"),
    (0x02198824, "e29994fa3070ae07"),
    (0x02198848, "d21cb2c59e33f2ac"),
    (0x02198868, "f8157eafa78a1b4f"),
    (0x02198888, "b05ff4e02a496ba1"),
    (0x021988AC, "b8c52a3a492c6be8"),
    (0x021988D0, "84524c558b384ac4"),
    (0x021988F4, "c43e711a6a54a5df"),
    (0x02198918, "89d9d6dce7cdf9e0"),
    (0x0219893C, "103c38317be4ed71"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2b.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_8318 => {
            stay_start(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0219_834C => stay_update(e, v[0]) as u64,
        0x0219_8380 => stay_up_32(e, v[0]) as u64,
        0x0219_84C0 => {
            jump_start(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
            0
        }
        0x0219_852C => {
            jump_start_se(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x0219_8550 => jump_update(e, v[0]) as u64,
        0x0219_8390 => stay_19(e, v[0]) as u64,
        0x0219_83A0 => stay_1a(e, v[0]) as u64,
        0x0219_83B0 => stay_1b(e, v[0]) as u64,
        0x0219_83C0 => stay_1c(e, v[0]) as u64,
        0x0219_83D0 => stay_1d(e, v[0]) as u64,
        0x0219_83E0 => stay_1e(e, v[0]) as u64,
        0x0219_83F0 => stay_1f(e, v[0]) as u64,
        0x0219_8400 => stay_20(e, v[0]) as u64,
        0x0219_8410 => stay_21(e, v[0]) as u64,
        0x0219_8420 => stay_22(e, v[0]) as u64,
        0x0219_8430 => stay_23(e, v[0]) as u64,
        0x0219_8440 => stay_24(e, v[0]) as u64,
        0x0219_8450 => stay_25(e, v[0]) as u64,
        0x0219_8460 => stay_26(e, v[0]) as u64,
        0x0219_8470 => stay_27(e, v[0]) as u64,
        0x0219_8480 => stay_28(e, v[0]) as u64,
        0x0219_8490 => stay_29(e, v[0]) as u64,
        0x0219_84A0 => stay_2a(e, v[0]) as u64,
        0x0219_84B0 => stay_2b(e, v[0]) as u64,
        0x0219_862C => jump_2c(e, v[0]) as u64,
        0x0219_8650 => jump_2d(e, v[0]) as u64,
        0x0219_8674 => jump_2e(e, v[0]) as u64,
        0x0219_8694 => jump_2f(e, v[0]) as u64,
        0x0219_86B8 => jump_30(e, v[0]) as u64,
        0x0219_86D8 => jump_31(e, v[0]) as u64,
        0x0219_86F8 => jump_32(e, v[0]) as u64,
        0x0219_8718 => jump_33(e, v[0]) as u64,
        0x0219_8738 => jump_34(e, v[0]) as u64,
        0x0219_8758 => jump_35(e, v[0]) as u64,
        0x0219_877C => jump_36(e, v[0]) as u64,
        0x0219_87A0 => jump_37(e, v[0]) as u64,
        0x0219_87C0 => jump_38(e, v[0]) as u64,
        0x0219_87E0 => jump_39(e, v[0]) as u64,
        0x0219_8804 => jump_3a(e, v[0]) as u64,
        0x0219_8824 => jump_3b(e, v[0]) as u64,
        0x0219_8848 => jump_5c(e, v[0]) as u64,
        0x0219_8868 => jump_5d(e, v[0]) as u64,
        0x0219_8888 => jump_5e(e, v[0]) as u64,
        0x0219_88AC => jump_5f(e, v[0]) as u64,
        0x0219_88D0 => jump_75(e, v[0]) as u64,
        0x0219_88F4 => jump_76(e, v[0]) as u64,
        0x0219_8918 => jump_77(e, v[0]) as u64,
        0x0219_893C => jump_78(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2b`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2b", SPEC, PORTED, RECORDED), &call);
}

/// [`JUMP_HEIGHTS`] is the build's: fldmmdl_acmd.c's .data `0x021DDBD4` points at the three 16-word tables.
#[test]
fn jump_heights_match_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    for (i, t) in JUMP_HEIGHTS.iter().enumerate() {
        let p = w(0x021D_DBD4 + 4 * i as u32);
        assert_eq!(p, [0x021D_4CC4, 0x021D_4C44, 0x021D_4C84][i]);
        for (k, &y) in t.iter().enumerate() {
            assert_eq!(w(p + 4 * k as u32), y, "table {i} entry {k}");
        }
    }
}
