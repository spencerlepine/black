use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021643AC, "c6d9913f1bafd121"),
    (0x021643E8, "f045ebfecc6ed54b"),
    (0x0216447C, "d8b5a86c2cc786f5"),
    (0x02164480, "b73cc770c9e09133"),
    (0x021644C4, "2439bb9206cafc79"),
    (0x021644F4, "49b67a3996f1a59f"),
    (0x02164508, "64e9efd0f4a55207"),
    (0x02164564, "fbd08ece2aef604f"),
    (0x0216458C, "f18f5bdb46949b4c"),
    (0x0216459C, "e1f44105fdf96e9a"),
    (0x021645C0, "0e08ddf5aa467256"),
    (0x021645E0, "f43a6af7be4e610d"),
    (0x02164654, "91e758a7c54b2083"),
    (0x02164678, "ac2ad0f1dfcc26d1"),
    (0x021646A0, "dae6e7c8e1937cfa"),
    (0x021646C4, "862647a15df71e28"),
    (0x021646E8, "507fe2bc5c2ed984"),
    (0x02164728, "7d09409347611027"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/eventdata_tail.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        (match addr {
            0x0216_43AC => {
                zone_enter(e, v[0], v[1]);
                0
            }
            0x0216_43E8 => {
                zone_change(e, v[0], v[1]);
                0
            }
            0x0216_447C => {
                nothing();
                0
            }
            0x0216_4480 => {
                menu_checks(e, v[0], v[1], v[2]);
                0
            }
            0x0216_44C4 => menu_check(e, v[0], v[1], v[2]),
            0x0216_44F4 => menu_event(e, v[0], v[1], v[2]),
            0x0216_4508 => menu_check_0(e, v[0], v[1], v[2]),
            0x0216_4564 => menu_check_3(e, v[0], v[1], v[2]),
            0x0216_458C => menu_check_4(e, v[0], v[1], v[2]),
            0x0216_459C => menu_check_5(e, v[0], v[1], v[2]),
            0x0216_45C0 => menu_check_6(e, v[0], v[1], v[2]),
            0x0216_45E0 => event_a_proc(e, v[0], v[1], v[2]),
            0x0216_4654 => event_a_create(e, v[0], v[1]),
            0x0216_4678 => mapchange_event(e, v[0], v[1]),
            0x0216_46A0 => event_b_proc(e, v[0], v[1], v[2]),
            0x0216_46C4 => event_b_create(e, v[0], v[1]),
            0x0216_46E8 => hidden_item_index(v[0]) as u32,
            0x0216_4708 => hidden_item_flag(v[0]) as u32,
            0x0216_4728 => {
                hidden_items_respawn(e, v[0]);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }) as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field eventdata_tail`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("eventdata_tail", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn hidden_item_table() {
    // record i holds flag offset i; the flag of script 8000 + i is 900 + i
    for (i, r) in HIDDEN_ITEMS.iter().enumerate() {
        assert_eq!(r.flag as usize, i);
    }
    assert_eq!(hidden_item_flag(8000), 900);
    assert_eq!(hidden_item_flag(8176), 900 + 0xB0);
    assert_eq!(hidden_item_flag(7999), 900);
    assert_eq!(HIDDEN_ITEMS.iter().filter(|r| r.respawn != 0).count(), 45);
}
