use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02198960, "e8a48d22613cdf5e"),
    (0x02198978, "abb75e924af358bf"),
    (0x021989BC, "fdb14095036c0c0e"),
    (0x02198A8C, "68e00bfadd6b5ca9"),
    (0x02198C4C, "03f6a843e2bb51db"),
    (0x02198D7C, "7c8ab46d67c9fe1a"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2c.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_8960 => {
            wait_start(e, v[0], v[1]);
            0
        }
        0x0219_8978 => wait_update(e, v[0]) as u64,
        0x0219_89BC => wait_8(e, v[0]) as u64,
        0x0219_8A8C => fall_update(e, v[0]) as u64,
        0x0219_8C4C => table_walk_update(e, v[0], v[1]) as u64,
        0x0219_8D7C => table_walk_update_3(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2c`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2c", SPEC, PORTED, RECORDED), &call);
}
