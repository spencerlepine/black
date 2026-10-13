use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C0598, "cd3ede0886dfd55c"),
    (0x021C0688, "0d725dd5e1bf09d1"),
    (0x021C068C, "b84e81969871669e"),
    (0x021C06A8, "78dc3b05ac38d1ab"),
    (0x021C06B4, "be1a0871baa7f7f0"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/height_ex.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021C_0598 => rect_contains(&e.mem, v[0], v[1], v[2]) as u64,
        0x021C_0688 => count(&e.mem, v[0]) as u64,
        0x021C_068C => record_contains(&e.mem, v[0], v[1], v[2], v[3]) as u64,
        0x021C_06A8 => height(&e.mem, v[0], v[1]) as u64,
        0x021C_06B4 => extra(&e.mem, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field height_ex`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("height_ex", SPEC, PORTED, RECORDED), &call);
}
