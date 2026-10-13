use super::*;
use crate::testenv::{run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x02162254, "28b863af3fd4d04e")];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/wfbc_data.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0216_2254 => slot_a_get(&mut e.mem, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field wfbc_data`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("wfbc_data", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn slot_copy_needs_the_flag() {
    let mut mem = Mem::new(WORK, vec![0u8; 0x100]);
    mem.w32(WORK + 0x18, 0x1122_3344);
    assert!(!slot_a_get(&mut mem, WORK, 2, WORK + 0x80));
    mem.w16(WORK + 2, 4);
    assert!(slot_a_get(&mut mem, WORK, 2, WORK + 0x80));
    assert_eq!(mem.r32(WORK + 0x80), 0x1122_3344);
    assert!(!slot_a_get(&mut mem, WORK, 34, WORK + 0x80));
}
