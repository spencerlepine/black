use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AA3E0, "e58485b1e4fc408c"),
    (0x021AA3FC, "4a0ea365af4f3bc3"),
    (0x021AA69C, "be901561c94fb25f"),
    (0x021AA6E0, "6cf5cf1e9c1b7cf8"),
    (0x021AA778, "9e0936cf35143d00"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/encount.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_A3E0 => rate_of(&e.mem, v[0], v[1], v[2]) as u64,
        0x021A_A3FC => rate_check(e, v[1], v[2]) as u64,
        0x021A_A69C => weather_class(e, v[0]) as u64,
        0x021A_A6E0 => rate_roll(e) as u64,
        0x021A_A778 => steps(&e.mem, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field encount`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("encount", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn rate_of_table() {
    let base = crate::testenv::WORK;
    let mut mem = Mem::new(base, vec![0u8; 0x200]);
    mem.w32(base + 0xC, base + 0x20);
    mem.w8(base + 0x20, 10);
    mem.w8(base + 0x26, 5);
    assert_eq!(rate_of(&mem, base, 0, 10), 20);
    assert_eq!(rate_of(&mem, base, 1, 10), 0); // zero rates get no bonus
    assert_eq!(rate_of(&mem, base, 6, (-8i32) as u32), 0);
    assert_eq!(rate_of(&mem, base, 7, 0), 0);
}
