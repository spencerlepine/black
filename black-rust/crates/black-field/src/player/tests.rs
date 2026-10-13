use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A3190, "5a417f2cf8abdbd6"),
    (0x021A31A4, "c7e81effe976cd55"),
    (0x021A31D0, "dd5b97749a5b3fb8"),
    (0x021A36E0, "f93cdd2b02d44263"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_3190 => entry_code(v[0], v[1]) as u64,
        0x021A_31A4 => form_code(v[0], v[1]) as u64,
        0x021A_31D0 => code_entry(v[0]) as u64,
        0x021A_36E0 => grid_of(&e.mem, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn form_table_lookups() {
    assert_eq!(form_code(0, 2), 3);
    assert_eq!(form_code(1, 1), 5);
    assert_eq!(form_code(0, 7), 1);
    assert_eq!(code_entry(0x8A), 5);
    assert_eq!(code_entry(6), 2);
    assert_eq!(code_entry(0x77), 10);
    assert_eq!(entry_code(1, 9), 0x9D);
}
