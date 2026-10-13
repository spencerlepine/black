use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215E2B8, "eee20e1ee87c231f"),
    (0x0215E31C, "4f1eb2de3267adad"),
    (0x0215E33C, "81de3d57f53f968a"),
    (0x0215E34C, "73cf5fe269c388ec"),
    (0x0215E358, "bd830468a80bd244"),
    (0x0215E364, "5b85f4c83a1f964b"),
    (0x0215E3A0, "ae3be475c1a96bc4"),
    (0x0215E3E4, "6da815577882823b"),
    (0x0215E404, "b4bb3a5e30a49624"),
    (0x0215E428, "42fa3505f8b7c075"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/encount_st.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0215_E2B8 => load_table(e, v[0], v[1], v[2], v[3]) as u64,
        0x0215_E31C => work_new(e, v[0]) as u64,
        0x0215_E33C => {
            effect_setup(e, v[0]);
            0
        }
        0x0215_E34C => {
            reset_effect_steps(e, v[0]);
            0
        }
        0x0215_E364 => {
            move_roamers(e, v[0]);
            0
        }
        0x0215_E3A0 => {
            move_roamers_b(e, v[0]);
            0
        }
        0x0215_E358 => move_roamer(e, v[0], v[1], v[2]) as u64,
        0x0215_E3E4 => swarm_zone(e, v[0], v[1]) as u64,
        0x0215_E404 => any_roamer_active(e, v[0]) as u64,
        0x0215_E428 => {
            push_location(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field encount_st`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("encount_st", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn swarm_zone_table() {
    assert_eq!(SWARM_ZONES[0], 319);
    assert_eq!(SWARM_ZONES[15], 387);
    assert_eq!(SWARM_ZONES[16], 0xFFFF);
}
