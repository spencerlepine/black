use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A94B0, "faa9a0fa40414d9b"),
    (0x021A996C, "07e1c7f19f7738d0"),
    (0x021A9DC8, "ec3cc29e686f3cd0"),
    (0x021A9DE0, "2af26d6b476b6055"),
    (0x021A9E64, "74773d933136a7bb"),
    (0x021A9E98, "cce9ca220a45f360"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/enc_pokeset.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_94B0 => rate_modifier(e, v[0], v[2]) as u64,
        0x021A_996C => rejected(&e.mem, v[0], v[1]) as u64,
        0x021A_9DC8 => roll100(e) as u64,
        0x021A_9DE0 => land_slot(e) as u64,
        0x021A_9E64 => water_slot(e) as u64,
        0x021A_9E98 => fishing_slot(e) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field enc_pokeset`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("enc_pokeset", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn slot_thresholds() {
    let land = [20, 40, 50, 60, 70, 80, 85, 90, 94, 98];
    assert_eq!(slot_by(0, &land), 0);
    assert_eq!(slot_by(19, &land), 0);
    assert_eq!(slot_by(20, &land), 1);
    assert_eq!(slot_by(97, &land), 9);
    assert_eq!(slot_by(99, &[60, 90, 95, 99]), 4);
    assert_eq!(slot_by(94, &[40, 80, 95, 99]), 2);
}

#[test]
fn level_filter() {
    let base = crate::testenv::WORK;
    let mut mem = Mem::new(base, vec![0u8; 0x40]);
    mem.w8(base + 0x1E, 20);
    mem.w32(base + 0x28, F_REPEL);
    assert!(rejected(&mem, base, 19));
    assert!(!rejected(&mem, base, 20));
    mem.w32(base + 0x28, F_REPEL | F_FISHING);
    assert!(!rejected(&mem, base, 19));
    mem.w32(base + 0x28, F_LOW_LEVEL_SKIP);
    assert!(rejected(&mem, base, 15));
    assert!(!rejected(&mem, base, 16));
    mem.w32(base + 0x28, F_LOW_LEVEL_SKIP | F_NO_FILTER);
    assert!(!rejected(&mem, base, 1));
}
