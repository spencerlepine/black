use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02198E44, "5ef0dcb370eb7cb1"),
    (0x02198E98, "da3ed0b93de2f16a"),
    (0x02198EC8, "b15f888878391784"),
    (0x021990D4, "584bdae6b4c22a0c"),
    (0x0219912C, "e2a8d05fa2545b69"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2d.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_8E44 => anim10_start(e, v[0]) as u64,
        0x0219_8E98 => anim10_start_se(e, v[0]) as u64,
        0x0219_8EC8 => anim_wait(e, v[0]) as u64,
        0x0219_90D4 => bounce_update(e, v[0]) as u64,
        0x0219_912C => {
            vec_move_start(e, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2d`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2d", SPEC, PORTED, RECORDED), &call);
}
