use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219A098, "bd984c8e3c605e68"),
    (0x0219A0CC, "58807c955155a546"),
    (0x0219A1E8, "95cf090568566ca4"),
    (0x0219A528, "837dc0cb022442d4"),
    (0x0219A54C, "cc1e36dc00095413"),
    (0x0219A7C4, "ee4fa05f545e9254"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3a.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_A098 => {
            look_init(e, v[0], v[1]);
            0
        }
        0x0219_A0CC => {
            look_init_all(e, v[0]);
            0
        }
        0x0219_A1E8 => next_dir(v[0], v[1]) as u64,
        0x0219_A528 => {
            face_init(e, v[0], v[1]);
            0
        }
        0x0219_A54C => {
            face_move(e, v[0]);
            0
        }
        0x0219_A7C4 => spin_wait(e, v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3a`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3a", SPEC, PORTED, RECORDED), &call);
}
