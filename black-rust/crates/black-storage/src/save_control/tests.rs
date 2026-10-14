use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020071E4, "0049ddb77138fea2"),
    (0x020071EC, "848b2b33886656ce"),
    (0x020071F0, "26473eae5ec3ee72"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let s = SaveStatus {
        exists: c.bufs[0][0],
        complete: c.bufs[0][1],
    };
    (match addr {
        0x0200_71E4 => s.exists(),
        0x0200_71EC => s.complete(),
        0x0200_71F0 => s.usable() as u32,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }) as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage save_control`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "save_control",
            include_str!("../../../../tools/rom-diff/specs/black-storage/save_control.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}
