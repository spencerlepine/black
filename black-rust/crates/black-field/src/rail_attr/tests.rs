use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BB088, "c9a0f22ff4b313ac"),
    (0x021BB0A4, "59628bfcba58420a"),
    (0x021BB0B4, "1838a8e6dbe14bb6"),
    (0x021BB11C, "5f789bf54c368ad7"),
    (0x021BB13C, "b738ce0e7f7e0902"),
    (0x021BB188, "c921423d6e0139b6"),
    (0x021BB1A0, "5936755344bc02f6"),
    (0x021BB1AC, "c2a8d96d49f70235"),
    (0x021BB1B8, "807be5553e586d96"),
    (0x021BB1C4, "e2ab332f3eb274fa"),
    (0x021BB1D0, "f108cfefe8882a19"),
    (0x021BB1DC, "bc1ff157f8900e7a"),
    (0x021BB1E8, "d2e8ee4a6c48d336"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/rail_attr.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021B_B088 => u64::from(create(e, v[0])),
        0x021B_B0A4 => {
            delete(e, v[0]);
            0
        }
        0x021B_B0B4 => {
            load(e, v[0], v[1], v[2]);
            0
        }
        0x021B_B11C => {
            unload(e, v[0]);
            0
        }
        0x021B_B13C => u64::from(attr_at(e, v[0], v[1])),
        0x021B_B188 => u64::from(is_a0_a1(v[0])),
        0x021B_B1A0 => u64::from(is_a3(v[0])),
        0x021B_B1AC => u64::from(is_a5(v[0])),
        0x021B_B1B8 => u64::from(is_a4(v[0])),
        0x021B_B1C4 => u64::from(is_a7(v[0])),
        0x021B_B1D0 => u64::from(is_a6(v[0])),
        0x021B_B1DC => u64::from(is_a8(v[0])),
        0x021B_B1E8 => u64::from(is_b0(v[0])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field rail_attr`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("rail_attr", SPEC, PORTED, RECORDED), &call);
}
