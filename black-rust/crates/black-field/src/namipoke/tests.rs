use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021ACBB8, "a69a84dd05f40bfe"),
    (0x021ACBE4, "d931040f589761aa"),
    (0x021ACC04, "adf6c2eab41c6a30"),
    (0x021ACCAC, "15e96184e19bea96"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/namipoke.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_CBB8 => {
            set_mode(e, v[0], v[1]);
            0
        }
        0x021A_CBE4 => follow_speed(v[0] as i32) as u32 as u64,
        0x021A_CC04 => ahead_along(&e.mem, v[0], v[1], v[2]) as u64,
        0x021A_CCAC => {
            draw_none();
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field namipoke`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("namipoke", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn follow_speed_clamps() {
    assert_eq!(follow_speed(0x5_0000), 0x4_0000 / 0xA3D * 0x28);
    assert_eq!(follow_speed(0xA3C), 0);
    assert_eq!(follow_speed(-0xA3D), -0x28);
}
