use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B6804, "956abe7a31829290"),
    (0x021B6818, "9a0b47e048505380"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021B_6804 => {
            super::set_exit_subproc(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021B_6818 => super::setup_type_is_3(&c.bufs[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server1`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server1",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server1.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
