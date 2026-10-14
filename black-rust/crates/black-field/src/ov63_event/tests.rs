use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C46F8, "47996af069df8063"),
    (0x021C474C, "cf09f84132775237"),
    (0x021C4788, "743b1275aff392ab"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/ov63_event.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021C_46F8 => u64::from(player_event_new(e, v[0], v[1], v[2])),
        0x021C_474C => u64::from(obj_event_new(e, v[0], v[1], v[2])),
        0x021C_4788 => u64::from(proc_(e, v[0])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field ov63_event`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("ov63_event", SPEC, PORTED, RECORDED), &call);
}
