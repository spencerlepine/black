use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AB5F4, "a3c2e51c0862d8f7"),
    (0x021AB624, "50f97bce948f6f66"),
    (0x021AB634, "a296f39cabc3cf23"),
    (0x021AB650, "274a4ae0c4ad3517"),
    (0x021AB660, "9ab48194f1ecbbec"),
    (0x021AB75C, "6f07ee2c8dd566e3"),
    (0x021AB760, "18557770a05e921b"),
    (0x021AB764, "9ea5ae83da7096f3"),
    (0x021AB768, "4499b630343cf812"),
    (0x021AB780, "a85e6d7a0b7f0dcd"),
    (0x021AB798, "e3bb558e899ff3bd"),
    (0x021AB7C0, "11464e1c43d9e934"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/scenearea.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        match addr {
            0x021A_B5F4 => return new(e, v[0], v[1], v[2]) as u64,
            0x021A_B624 => delete(e, v[0]),
            0x021A_B634 => set_list(e, v[0], v[1], v[2], v[3]),
            0x021A_B650 => clear_list(e, v[0]),
            0x021A_B660 => return update(e, v[0], v[1]) as u64,
            0x021A_B75C => set_enabled(e, v[0], v[1]),
            0x021A_B760 => return fieldsys(e, v[0]) as u64,
            0x021A_B764 => return fieldmap(e, v[0]) as u64,
            0x021A_B768 => return has_check(e, v[0], v[1]) as u64,
            0x021A_B780 => return has_update(e, v[0], v[1]) as u64,
            0x021A_B798 => return run_check(e, v[0], v[1], v[2], v[3], v[4]) as u64,
            0x021A_B7C0 => return run_update(e, v[0], v[1], v[2], v[3], v[4]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
        0
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field scenearea`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("scenearea", SPEC, PORTED, RECORDED), &call);
}
