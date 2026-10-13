use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219B41C, "7acf7715805cabc8"),
    (0x0219B494, "6a1c25e7763334ed"),
    (0x0219B4C8, "b05e8b0879482c0b"),
    (0x0219B508, "2569610849461695"),
    (0x0219B9B0, "5ee982f09b1bd801"),
    (0x0219BA2C, "0fa04383e09d4c8e"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3c.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_B41C => ev_busy(e, v[0]) as u64,
        0x0219_B494 => moved(e, v[0]) as u64,
        0x0219_B4C8 => standing(e, v[0]) as u64,
        0x0219_B508 => {
            ev_steps_init(e, v[0]);
            0
        }
        0x0219_B9B0 => {
            follow_start(e, v[0], v[1]);
            0
        }
        0x0219_BA2C => follower_base(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3c`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3c", SPEC, PORTED, RECORDED), &call);
}
