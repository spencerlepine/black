use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B9C24, "a01f2ec8dc6186bd"),
    (0x021BA710, "92a928e0955bdc88"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021B_9C24 => {
            super::party_move_to_end(&mut c.bufs[0], c.vals[1]);
            0
        }
        0x021B_A710 => super::client_relation(c.vals[0], c.vals[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server3`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server3",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server3.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
