use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219AA38, "792ba237f200f9f5"),
    (0x0219AA68, "85ff9595735eaa1b"),
    (0x0219B334, "250c416edf81a8b6"),
    (0x0219B358, "6a6d15862cfe4603"),
    (0x0219B374, "293133a3389b5f3e"),
    (0x0219B404, "02aa2a3ce47774e1"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3b.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_AA38 => {
            route_init(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0219_AA68 => {
            route_init_15(e, v[0]);
            0
        }
        0x0219_B334 => walks_fast_near(e, v[0]) as u64,
        0x0219_B358 => spins_while_walking(e, v[0]) as u64,
        0x0219_B374 => {
            spin_order(e, v[0], v[1]);
            0
        }
        0x0219_B404 => {
            spin_unlock(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3b`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3b", SPEC, PORTED, RECORDED), &call);
}
