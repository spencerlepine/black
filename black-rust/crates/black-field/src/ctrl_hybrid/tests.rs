use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A6DC4, "4a29ae4f75adac5d"),
    (0x021A6E30, "046ffe39cacb1b4e"),
    (0x021A6F18, "2e9607745cff80e1"),
    (0x021A6F24, "df27ce482e6aca70"),
    (0x021A6F4C, "a6673c886be445f3"),
    (0x021A6F5C, "c64e71d45f5b3328"),
    (0x021A7010, "d9c017fe3974b00d"),
    (0x021A70D0, "5fd01973e3f30bc2"),
    (0x021A712C, "852550ecc62c447f"),
    (0x021A7198, "05c4f91dd7801571"),
    (0x021A71E4, "7a735548689603ab"),
    (0x021A7258, "46678abfa3212c9a"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/ctrl_hybrid.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_6DC4 => {
            reenter(e, v[0], v[1]);
            0
        }
        0x021A_6E30 => {
            create(e, v[0], v[1], v[2]);
            0
        }
        0x021A_6F18 => {
            delete(e, v[0]);
            0
        }
        0x021A_6F24 => {
            frame(e, v[0], v[1]);
            0
        }
        0x021A_6F4C => u64::from(pos(e, v[0])),
        0x021A_6F5C => {
            frame_grid(e, v[0], v[1]);
            0
        }
        0x021A_7010 => {
            frame_rail(e, v[0], v[1]);
            0
        }
        0x021A_70D0 => {
            to_rail(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x021A_712C => {
            to_grid(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x021A_7198 => u64::from(grid_entry(e, v[0], v[1], v[2], v[3])),
        0x021A_71E4 => u64::from(rail_entry(e, v[0], v[1], v[2], v[3])),
        0x021A_7258 => {
            set_mode(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field ctrl_hybrid`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("ctrl_hybrid", SPEC, PORTED, RECORDED), &call);
}
