use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C8810, "586b8293f7d8c5e5"),
    (0x021C8828, "072104025f7c962d"),
    (0x021C8838, "21042ebd3ebb28d8"),
    (0x021C8860, "7808761ee6d38910"),
    (0x021C888C, "174edc567e4ff1ff"),
    (0x021C88A4, "c0288ac5b86e5289"),
    (0x021C88B4, "3e06f6d0261736df"),
    (0x021C88C8, "246451131471b7b5"),
    (0x021C88E4, "bb17ecadc10227d1"),
    (0x021C8910, "00380a9ba83399f1"),
];
// svflow-recorded-end

/// Address of the work image in the emulator.
const WORK: u32 = 0x02E0_0000;

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x021C_8810 => {
            super::str_clear(&mut c.bufs[0]);
            0
        }
        0x021C_8828 => super::str_is_set(&c.bufs[0]) as u64,
        0x021C_8838 => {
            super::str_init(&mut c.bufs[0], v[1], v[2]);
            0
        }
        0x021C_8860 => {
            super::str_push(&mut c.bufs[0], v[1]);
            0
        }
        0x021C_888C => {
            super::str_set_extra(&mut c.bufs[0], v[1]);
            0
        }
        0x021C_88A4 => super::handex_push(&mut c.work, v[1], v[2]) as u64,
        0x021C_88B4 => {
            super::handex_push_exec(&mut c.env, &mut c.work, WORK, v[1], v[2]);
            0
        }
        0x021C_88C8 => {
            super::handex_pop(&mut c.env, &mut c.work, WORK, v[1] - WORK);
            0
        }
        0x021C_88E4 => super::handex_state(&c.work) as u64,
        0x021C_8910 => super::handex_state_thunk(&c.work) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py hem`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "hem",
        include_str!("../../../../../tools/rom-diff/svflow_specs/hem.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
