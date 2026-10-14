use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021664CC, "f8a17b7ff508364c"),
    (0x021665D0, "e60881648170f538"),
    (0x021665F0, "5c5330c86bdd4182"),
    (0x02166618, "dc55bae196854875"),
    (0x0216662C, "785c1f0bc966f342"),
    (0x02166640, "a534d6c98c3a0a8e"),
    (0x021666A4, "d57f515ca0eda0dd"),
    (0x02166B58, "82664cc8dbf97331"),
    (0x02166C4C, "37eb4e15e281f326"),
    (0x02166C7C, "eca43438a247f4bd"),
    (0x02166CD0, "164675c24adea72d"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/symbol_map.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0216_64CC => symbol_record(e, v[0], v[1], v[2]) as u64,
        0x0216_65D0 => beacon_value(e, v[0], v[1]) as u64,
        0x0216_65F0 => beacon_index(e, v[0], v[1], v[2]) as u64,
        0x0216_6618 => kind_is_0(v[0]) as u64,
        0x0216_662C => kind_is_4(v[0]) as u64,
        0x0216_6640 => kind_is_1_to_3(v[0]) as u64,
        0x0216_66A4 => {
            free_ptr(e, v[0]);
            0
        }
        0x0216_6B58 => hatch_proc(e, v[0], v[1], v[2]) as u64,
        0x0216_6C4C => hatch_event_new(e, v[0], v[1], v[2]) as u64,
        0x0216_6C7C => zone_group(e, v[0], v[1]) as u64,
        0x0216_6CD0 => check_flag(e, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field symbol_map`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("symbol_map", SPEC, PORTED, RECORDED), &call);
}

/// The embedded remap table is the ROM's (`0x02171F2C`, overlay 10 at `0x02155160`).
#[test]
fn rom_tables_match() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(ov) = dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
        return;
    };
    let o = (0x0217_1F2C - 0x0215_5160) as usize;
    let w = |k: usize| u32::from_le_bytes(ov[o + 4 * k..o + 4 * k + 4].try_into().unwrap());
    for (i, &(r, key)) in ZONE_REMAP.iter().enumerate() {
        assert_eq!((w(2 * i), w(2 * i + 1)), (r, key));
    }
}

#[test]
fn symbol_kinds() {
    assert_eq!(kind_is_0(0), 1);
    assert_eq!(kind_is_0(0x100), 1);
    assert_eq!(kind_is_4(4), 1);
    assert_eq!(kind_is_4(5), 0);
    assert_eq!(
        (0..6).map(kind_is_1_to_3).collect::<Vec<_>>(),
        [0, 1, 1, 1, 0, 0]
    );
    assert_eq!(kind_is_1_to_3(0xFF), 0);
}
