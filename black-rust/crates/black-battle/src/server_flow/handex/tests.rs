use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C8D0C, "c33aadb4ce8322f4"),
    (0x021C9F0C, "fc50fbab0b2c5822"),
    (0x021CA738, "cf0c7e8a9b80ccd4"),
    (0x021CA7C4, "83d168221ad0d042"),
    (0x021CA7D4, "ccd83cb3350f1b51"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021C_8D0C => super::proc_03(&mut c.env, &c.work, &c.bufs[1]) as u64,
        0x021C_9F0C => super::proc_25(
            &mut c.env,
            &c.work,
            &c.bufs[1],
            &mut c.regions[0],
            &mut c.globals[0],
        ) as u64,
        0x021C_A738 => super::proc_38(&mut c.env, &mut c.regions[0], &mut c.globals[0]) as u64,
        0x021C_A7C4 => super::proc_3a(&c.bufs[1], &mut c.regions[0]) as u64,
        0x021C_A7D4 => super::proc_3b(&mut c.regions[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py handex`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "handex",
        include_str!("../../../../../tools/rom-diff/svflow_specs/handex.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
