use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x021646E8, "507fe2bc5c2ed984")];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = &c.vals;
    match addr {
        0x0216_46E8 => hidden_item_index(v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field eventdata_tail`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "eventdata_tail",
            include_str!("../../../../tools/rom-diff/specs/black-field/eventdata_tail.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn hidden_item_range() {
    assert_eq!(hidden_item_index(7999), 0);
    assert_eq!(hidden_item_index(8000), 0);
    assert_eq!(hidden_item_index(8176), 176);
    assert_eq!(hidden_item_index(0x225F), 799);
    assert_eq!(hidden_item_index(0x2260), 0);
}
