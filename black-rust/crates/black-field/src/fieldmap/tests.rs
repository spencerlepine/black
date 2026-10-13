use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x02189B04, "8ff8981a934eb2dd")];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0218_9B04 => is_hold_zone(c.vals[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field fieldmap`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "fieldmap",
            include_str!("../../../../tools/rom-diff/specs/black-field/fieldmap.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn hold_zones() {
    for z in [0x3F, 0x8C, 0x8D, 0x8E] {
        assert!(is_hold_zone(z));
    }
    for z in [0, 0x3E, 0x40, 0x8B, 0x8F, 0x1_003F] {
        assert!(!is_hold_zone(z));
    }
}
