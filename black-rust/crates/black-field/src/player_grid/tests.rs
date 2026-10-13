use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A5304, "5178a9ea306cc7e6"),
    (0x021A530C, "53f85f95155c0c9a"),
    (0x021A5318, "1c2a9a6f8ff3d57f"),
    (0x021A5768, "68f72573ed797dcc"),
    (0x021A5778, "fb7ae193d4545242"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_grid.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_5304 => {
            set_flags(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_530C => {
            clear_flags(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_5318 => has_flags(&e.mem, v[0], v[1]) as u64,
        0x021A_5768 => {
            set_1c(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_5778 => state_is_3(&e.mem, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_grid`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_grid", SPEC, PORTED, RECORDED), &call);
}
