use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C7CBC, "b88ee8e5c09d85ab"),
    (0x021C7CF4, "b37143d193e00aec"),
    (0x021C7D54, "9a5c8f996b574f82"),
    (0x021C804C, "c9ecbf9f96485d44"),
    (0x021C831C, "93b0f4dbaf0773c3"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x021C_7CBC => super::word_1f7c(&c.work) as u64,
        0x021C_7CF4 => super::mon_by_id(&mut c.env, &c.work, v[1]) as u64,
        0x021C_7D54 => super::list_foes(&mut c.env, v[1], &mut c.bufs[2]) as u64,
        0x021C_804C => super::find_action(&mut c.env, &c.work, v[1], &mut c.bufs[2]) as u64,
        0x021C_831C => {
            super::list6_push(&mut c.work, v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py api`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "api",
        include_str!("../../../../../tools/rom-diff/svflow_specs/api.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
