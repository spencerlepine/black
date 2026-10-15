use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A6CB8, "4fe8eba3b7919577"),
    (0x021A6D30, "bbf6705d98cd8d75"),
    (0x021A6D3C, "de8f0409e27179be"),
    (0x021A6D48, "156edfdb367d5d83"),
    (0x021A6D58, "5a10ce9b786534f3"),
    (0x021A6D9C, "d3af91660e75ccd6"),
    (0x021A6DA4, "f5bf7fe74f8edbae"),
    (0x021A6DC0, "6fb01c306d326088"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/ctrl_nogrid.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_6CB8 => {
            create(e, v[0], v[1], v[2]);
            0
        }
        0x021A_6D30 => {
            delete(e, v[0]);
            0
        }
        0x021A_6D3C => {
            frame(e, v[0]);
            0
        }
        0x021A_6D48 => u64::from(pos(e, v[0])),
        0x021A_6D58 => u64::from(work_new(e, v[0], v[1])),
        0x021A_6D9C => {
            work_free(e, v[0]);
            0
        }
        0x021A_6DA4 => {
            work_frame(e, v[0]);
            0
        }
        0x021A_6DC0 => u64::from(work_player(e, v[0])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field ctrl_nogrid`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("ctrl_nogrid", SPEC, PORTED, RECORDED), &call);
}
