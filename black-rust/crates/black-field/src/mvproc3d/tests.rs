use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219BE60, "d897500eb05dd0d1"),
    (0x0219BF3C, "9cde8f866ae12dcc"),
    (0x0219BF48, "458f7e2be3817fed"),
    (0x0219BF54, "fc5c248f9db15331"),
    (0x0219BF60, "f2189c17855573e3"),
    (0x0219C0D4, "fafca15e1a9642ba"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3d.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_BE60 => {
            effect_delete(e, v[0]);
            0
        }
        0x0219_BF3C => {
            set_effect_handle(e, v[0], v[1]);
            0
        }
        0x0219_BF48 => effect_handle(e, v[0]) as u64,
        0x0219_BF54 => {
            effect_done(e, v[0]);
            0
        }
        0x0219_BF60 => {
            mirror_init(e, v[0], v[1], v[2]);
            0
        }
        0x0219_C0D4 => {
            wall_init(e, v[0], v[1], v[2]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3d`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3d", SPEC, PORTED, RECORDED), &call);
}
