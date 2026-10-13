use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021D2990, "3efc5f0cb6f6816f"),
    (0x021D299C, "5bd1973e1e6d105a"),
    (0x021D2C7C, "b271ff9d79f401b9"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x021D_2990 => super::play_sound(&mut c.env, &c.bufs[2]) as u64,
        0x021D_299C => {
            let (obj, state) = c.bufs.split_at_mut(1);
            super::view_effect_step(&mut c.env, &obj[0], &mut state[0]) as u64
        }
        0x021D_2C7C => super::window_sound_event(&mut c.env, c.vals[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client4`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client4",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client4.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
