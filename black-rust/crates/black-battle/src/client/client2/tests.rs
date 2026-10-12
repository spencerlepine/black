use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x021CDEE8, "d33fcc0ff04b4924")];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021C_DEE8 => super::hp_drop_percent(&mut c.env, &c.bufs[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client2`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client2",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client2.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
