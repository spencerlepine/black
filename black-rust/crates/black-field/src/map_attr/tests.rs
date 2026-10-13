use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AB0F0, "04e931146b9319e1"),
    (0x021AB100, "6e923909b2e34ca5"),
    (0x021AB250, "09bdf3432f2f4591"),
    (0x021AB270, "f0601593713d4759"),
    (0x021AB29C, "6f65057c43ebd492"),
    (0x021AB504, "7aa0038a043177e6"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/map_attr.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = &c.vals;
    match addr {
        0x021A_B0F0 => value(v[0]) as u64,
        0x021A_B100 => valid(v[0]) as u64,
        0x021A_B250 => is_long_grass_a(v[0]) as u64,
        0x021A_B270 => is_long_grass_b(v[0]) as u64,
        0x021A_B29C => is_long_grass(v[0]) as u64,
        0x021A_B504 => effect_kind(v[0]) as u64,
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
