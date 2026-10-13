use crate::server::rd32;
use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B8564, "a6ecf34d65d89d16"),
    (0x021B8670, "ccdb76e7d39a0391"),
];
// svflow-recorded-end

/// The BATTLE_SETUP that `M[0]` points at, inside the arena of argument 0 (M itself at `vals[0]`).
fn setup(c: &Case) -> &[u8] {
    let ofs = rd32(&c.bufs[0], 0).wrapping_sub(c.vals[0]) as usize;
    &c.bufs[0][ofs..]
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021B_8564 => super::all_clients_done(&c.bufs[0]) as u64,
        0x021B_8670 => super::rule_positions_per_side(setup(c)) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py server2`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "server2",
        include_str!("../../../../../tools/rom-diff/svflow_specs/server2.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
