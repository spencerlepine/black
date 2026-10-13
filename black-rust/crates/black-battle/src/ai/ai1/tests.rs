use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02188F08, "f236c712ccfa7746"),
    (0x02188F50, "50082511dab9c5f2"),
    (0x021893B4, "46856b1bf0b935c6"),
    (0x021894B4, "5476ae9d9449c11c"),
    (0x021894C4, "0e495444e2605892"),
];
// svflow-recorded-end

// Each ai1 function takes (vm, ai); the ports read/write the AI object buffer (arg 1).
fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0218_8F08 => super::cmd_result_from_score(&mut c.bufs[1]) as u64,
        0x0218_8F50 => super::cmd_status(&c.bufs[1]) as u64,
        0x0218_93B4 => super::cmd_set_abort(&mut c.bufs[1]) as u64,
        0x0218_94B4 => super::cmd_result_from_98(&mut c.bufs[1]) as u64,
        0x0218_94C4 => super::cmd_result_from_9c(&mut c.bufs[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py ai1`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "ai1",
        include_str!("../../../../../tools/rom-diff/svflow_specs/ai1.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
