use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CD9F4, "db9988908c6de10a"),
    (0x021CDD34, "331f18650f7f4e7a"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021C_D9F4 => {
            super::counter_add(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021C_DD34 => {
            super::result_clear(&mut c.bufs[0], c.vals[0]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client1`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client1",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client1.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
