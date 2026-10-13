use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02197DE8, "444601f6d35221ae"),
    (0x02197E44, "0affd666e073898a"),
    (0x02197E7C, "8374968a8a72d04f"),
    (0x02197EDC, "653e1c5505c96161"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd1.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_7DE8 => list_start(&mut e.mem, v[0]) as u64,
        0x0219_7E44 => list_next(&mut e.mem, v[0]) as u64,
        0x0219_7E7C => code_for_dir(v[0], v[1]) as u64,
        0x0219_7EDC => action(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd1`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd1", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn code_for_dir_maps_within_a_kind() {
    assert_eq!(code_for_dir(2, 0x0D), 0x0E);
    assert_eq!(code_for_dir(3, 0x5C), 0x5D);
    assert_eq!(code_for_dir(1, 0x99), 0x99);
    assert_eq!(code_for_dir(0, 0x1_0000), 0x1_0000);
}
