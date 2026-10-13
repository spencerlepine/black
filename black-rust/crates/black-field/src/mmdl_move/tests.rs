use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x021642B4, "96a464d84c2e8a0a")];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = &c.vals;
    match addr {
        0x0216_42B4 => dir_between(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field mmdl_move`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "mmdl_move",
            include_str!("../../../../tools/rom-diff/specs/black-field/mmdl_move.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn x_decides_before_z() {
    assert_eq!(dir_between(5, 0, 4, 9), 2);
    assert_eq!(dir_between(-1, 0, 0, -9), 3);
    assert_eq!(dir_between(3, 2, 3, 1), 0);
    assert_eq!(dir_between(3, 2, 3, 2), 1);
}
