use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219EF14, "e1b2a7371bd8eee1"),
    (0x0219EF44, "d1a70a58295acf8b"),
    (0x0219EF6C, "52bcae4de6b5d360"),
    (0x0219EF7C, "bcb432290a9119f5"),
    (0x0219EFFC, "f06e688c8c6d5335"),
    (0x0219F018, "f4ee179558442aef"),
    (0x0219F01C, "29d887e90fd7b753"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/rail_loader.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_EF14 => {
            clear(e, v[0]);
            0
        }
        0x0219_EF44 => u64::from(create(e, v[0])),
        0x0219_EF6C => {
            delete(e, v[0]);
            0
        }
        0x0219_EF7C => {
            load(e, v[0], v[1], v[2]);
            0
        }
        0x0219_EFFC => {
            unload(e, v[0]);
            0
        }
        0x0219_F018 => u64::from(rail_data(v[0])),
        0x0219_F01C => u64::from(call_02158768(e, v[0], v[2], v[4], v[5])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field rail_loader`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("rail_loader", SPEC, PORTED, RECORDED), &call);
}
