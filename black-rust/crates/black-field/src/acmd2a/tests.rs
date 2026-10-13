use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02197F5C, "7c35198a5c82f60a"),
    (0x02197F6C, "9bb657d6daad12f0"),
    (0x02197F90, "6b6a22ff650d4cbd"),
    (0x02197FC0, "d032acac0c5cc728"),
    (0x0219800C, "b242a680dcfb4d3b"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2a.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_7F5C => end_step(e, v[0]) as u64,
        0x0219_7F6C => {
            face(e, v[0], v[1]);
            0
        }
        0x0219_7F90 => face_up(e, v[0]) as u64,
        0x0219_7FC0 => {
            walk_start(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x0219_800C => walk_update(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2a`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2a", SPEC, PORTED, RECORDED), &call);
}
