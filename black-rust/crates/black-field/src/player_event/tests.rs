use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C1094, "c18ea68039a0a196"),
    (0x021C10A0, "fd8ac52e091b8203"),
    (0x021C12C4, "e99d310f2b862f16"),
    (0x021C12D0, "567e9a5406aa55b1"),
    (0x021C12E4, "58f3e36eaae0db5c"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_event.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021C_1094 => surf_start_done(e, v[0]) as u64,
        0x021C_10A0 => {
            surf_start_delete(e, v[0]);
            0
        }
        0x021C_12C4 => surf_end_done(e, v[0]) as u64,
        0x021C_12D0 => {
            surf_end_delete(e, v[0]);
            0
        }
        0x021C_12E4 => {
            set_ride_offset(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_event`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_event", SPEC, PORTED, RECORDED), &call);
}
