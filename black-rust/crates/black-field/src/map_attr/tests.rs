use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AB0D4, "4680a4153d8763ea"),
    (0x021AB0F0, "04e931146b9319e1"),
    (0x021AB0F8, "9b11d4b80402c8cc"),
    (0x021AB100, "6e923909b2e34ca5"),
    (0x021AB11C, "47c162fc7e5f2119"),
    (0x021AB140, "70fcb1189e598aa9"),
    (0x021AB158, "dca298381e3f66ce"),
    (0x021AB164, "daccc0c664541596"),
    (0x021AB170, "88f2006a9629dbe5"),
    (0x021AB17C, "5627a90750077a7e"),
    (0x021AB188, "2d74861c10efec1e"),
    (0x021AB194, "07f916191c4e54c1"),
    (0x021AB1A0, "da8ecd97d1cc69ae"),
    (0x021AB1AC, "d282bc352554d99d"),
    (0x021AB1B8, "e254f5a63b4c211e"),
    (0x021AB1C4, "9ca8b0350000f575"),
    (0x021AB1D0, "752be5e72ceb65da"),
    (0x021AB1DC, "3186c2b664dc092e"),
    (0x021AB1E8, "883133272437ebc1"),
    (0x021AB1F4, "3f96246236ed7859"),
    (0x021AB200, "612b651e869c87f9"),
    (0x021AB20C, "70cd7f8ec5712f05"),
    (0x021AB218, "653e365eb6ce1ae1"),
    (0x021AB224, "8b28bbf07565727a"),
    (0x021AB230, "5f311fa3f00dca3a"),
    (0x021AB23C, "13d642d2d108cd15"),
    (0x021AB250, "09bdf3432f2f4591"),
    (0x021AB264, "b6eb4511aab3cb86"),
    (0x021AB270, "f0601593713d4759"),
    (0x021AB27C, "77cbd519e62c5919"),
    (0x021AB29C, "6f65057c43ebd492"),
    (0x021AB2BC, "2de08018b5127166"),
    (0x021AB2DC, "3f04b9916f97624d"),
    (0x021AB2E8, "cd7a3dc53509ffd5"),
    (0x021AB2F8, "aacd4364e2e64f66"),
    (0x021AB304, "96e0d0805ffb9679"),
    (0x021AB310, "9bfa3923a6eba8fa"),
    (0x021AB31C, "dc852f0f34158ac2"),
    (0x021AB328, "87e322f505182a62"),
    (0x021AB334, "67275269dc157159"),
    (0x021AB348, "c502fcd62e4ecf2a"),
    (0x021AB358, "5e3eea58345dbf72"),
    (0x021AB364, "d31de73ab75281f2"),
    (0x021AB370, "5894d53efbea58ba"),
    (0x021AB37C, "f34f6998667205dd"),
    (0x021AB390, "17bea1b310c6456a"),
    (0x021AB3B0, "432ea917690f5985"),
    (0x021AB3D4, "826fe84321aba911"),
    (0x021AB3E0, "971cf5a88df67c89"),
    (0x021AB3EC, "af13839bc4bb3efa"),
    (0x021AB3F8, "ce778680666ec5f6"),
    (0x021AB404, "04f34def87bf2545"),
    (0x021AB410, "1411e246c31fc9d9"),
    (0x021AB41C, "6571afc98dafdfb9"),
    (0x021AB428, "fd67083deef46392"),
    (0x021AB434, "c18a5ebd8c33db9e"),
    (0x021AB440, "7a7421920f0519fd"),
    (0x021AB44C, "e6ab076fdbd5fe5e"),
    (0x021AB458, "5f16531a7090dc29"),
    (0x021AB474, "9e1ac14d6cf3dd25"),
    (0x021AB490, "7fc86014cfcf7ebe"),
    (0x021AB4A4, "ee5feb0bb338ad22"),
    (0x021AB4B8, "193116cbe345cb1e"),
    (0x021AB4EC, "9b6c504292f1de22"),
    (0x021AB4F8, "bb43e0107488f37e"),
    (0x021AB504, "7aa0038a043177e6"),
    (0x021AB580, "dd2b84de004c5af7"),
    (0x021AB5AC, "db5f8fcd140c69fe"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/map_attr.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    if addr == 0x021A_B0D4 {
        return run(c, SPEC, addr, |e, v| at(e, v[0], v[1]) as u64);
    }
    let v = &c.vals;
    match addr {
        0x021A_B0F0 => value(v[0]) as u64,
        0x021A_B0F8 => flags(v[0]) as u64,
        0x021A_B100 => valid(v[0]) as u64,
        0x021A_B11C => blocked(v[0]) as u64,
        0x021A_B140 => is_8_or_9(v[0]) as u64,
        0x021A_B158 => is_74(v[0]) as u64,
        0x021A_B164 => is_75(v[0]) as u64,
        0x021A_B170 => is_73(v[0]) as u64,
        0x021A_B17C => is_72(v[0]) as u64,
        0x021A_B188 => is_7c(v[0]) as u64,
        0x021A_B194 => is_d4(v[0]) as u64,
        0x021A_B1A0 => is_d6(v[0]) as u64,
        0x021A_B1AC => is_d7(v[0]) as u64,
        0x021A_B1B8 => is_d8(v[0]) as u64,
        0x021A_B1C4 => is_e2(v[0]) as u64,
        0x021A_B1D0 => is_d9(v[0]) as u64,
        0x021A_B1DC => is_da(v[0]) as u64,
        0x021A_B1E8 => is_db(v[0]) as u64,
        0x021A_B1F4 => is_dc(v[0]) as u64,
        0x021A_B200 => is_dd(v[0]) as u64,
        0x021A_B20C => is_de(v[0]) as u64,
        0x021A_B218 => is_df(v[0]) as u64,
        0x021A_B224 => is_e0(v[0]) as u64,
        0x021A_B230 => is_e1(v[0]) as u64,
        0x021A_B23C => is_grass_a(v[0]) as u64,
        0x021A_B250 => is_long_grass_a(v[0]) as u64,
        0x021A_B264 => is_grass_b(v[0]) as u64,
        0x021A_B270 => is_long_grass_b(v[0]) as u64,
        0x021A_B27C => is_short_grass(v[0]) as u64,
        0x021A_B29C => is_long_grass(v[0]) as u64,
        0x021A_B2BC => is_grass(v[0]) as u64,
        0x021A_B2DC => is_1d(v[0]) as u64,
        0x021A_B2E8 => is_41_or_44(v[0]) as u64,
        0x021A_B2F8 => is_14(v[0]) as u64,
        0x021A_B304 => is_17(v[0]) as u64,
        0x021A_B310 => is_40(v[0]) as u64,
        0x021A_B31C => is_0e(v[0]) as u64,
        0x021A_B328 => is_0f(v[0]) as u64,
        0x021A_B334 => is_0e_b(v[0]) as u64,
        0x021A_B348 => is_18_or_10(v[0]) as u64,
        0x021A_B358 => is_10(v[0]) as u64,
        0x021A_B364 => is_1c(v[0]) as u64,
        0x021A_B370 => is_19(v[0]) as u64,
        0x021A_B37C => is_0c_0b_23(v[0]) as u64,
        0x021A_B390 => is_19_0c_0b_23(v[0]) as u64,
        0x021A_B3B0 => is_3d_to_43(v[0]) as u64,
        0x021A_B3D4 => is_02(v[0]) as u64,
        0x021A_B3E0 => is_1e(v[0]) as u64,
        0x021A_B3EC => is_30(v[0]) as u64,
        0x021A_B3F8 => is_32(v[0]) as u64,
        0x021A_B404 => is_be(v[0]) as u64,
        0x021A_B410 => is_bf(v[0]) as u64,
        0x021A_B41C => is_0c(v[0]) as u64,
        0x021A_B428 => is_05(v[0]) as u64,
        0x021A_B434 => is_07(v[0]) as u64,
        0x021A_B440 => is_1b(v[0]) as u64,
        0x021A_B44C => is_33(v[0]) as u64,
        0x021A_B458 => is_53_55_56(v[0]) as u64,
        0x021A_B474 => is_54_57_58(v[0]) as u64,
        0x021A_B490 => is_52_56_58(v[0]) as u64,
        0x021A_B4A4 => is_51_55_57(v[0]) as u64,
        0x021A_B4B8 => is_8_9_be_bf_0f(v[0]) as u64,
        0x021A_B4EC => is_21(v[0]) as u64,
        0x021A_B4F8 => is_22(v[0]) as u64,
        0x021A_B504 => effect_kind(v[0]) as u64,
        0x021A_B580 => ground_kind(v[0]) as u64,
        0x021A_B5AC => enters(v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field map_attr`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("map_attr", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn attribute_words() {
    assert!(valid(0x0004_0004));
    assert!(!valid(0x0004_00FF));
    assert!(!valid(u32::MAX));
    assert_eq!(effect_kind(0x0004_0004), 0);
    assert_eq!(effect_kind(0x0000_0004), 8);
    assert_eq!(effect_kind(0x0000_0020), 7);
    assert_eq!(effect_kind(0x0004_000A), 4);
    assert!(is_long_grass(0x22) && is_long_grass(9) && !is_long_grass(8));
}
