use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02199888, "f5ae3163c89ad91c"),
    (0x02199970, "b4481a123a24c74d"),
    (0x02199AEC, "bc80c2c4f3a66879"),
    (0x02199CE4, "bf95d98817c9511c"),
    (0x02199D14, "306df02baf5e3ece"),
    (0x02199F94, "d65cfc5a6b68ee2c"),
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
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2e`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2e", SPEC, PORTED, RECORDED), &call);
}
