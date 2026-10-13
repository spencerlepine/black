use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CFBBC, "53e5920dd3b052c2"),
    (0x021D0B10, "4f1a7c9aeaa1766f"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021C_FBBC => super::slot_holds(&c.bufs[0], c.vals[1] as i32) as u64,
        0x021D_0B10 => {
            super::setup_party_block(
                &mut c.env, &c.bufs[0], c.vals[1], c.vals[2], c.vals[3], c.vals[4],
            );
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client3`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client3",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client3.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
