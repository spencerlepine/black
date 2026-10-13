use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021D3CC8, "2839ec220663ac18"),
    (0x021D3EC8, "21b1c52f074ee6ec"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021D_3CC8 => {
            let (obj, args) = c.bufs.split_at_mut(2);
            super::relay_mon_block(&mut c.env, &obj[0], &args[0]) as u64
        }
        0x021D_3EC8 => super::item_cures_status(&mut c.env, c.vals[0], c.vals[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client5`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client5",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client5.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
