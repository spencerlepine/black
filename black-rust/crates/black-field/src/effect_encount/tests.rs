use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AA7DC, "101957db2c123694"),
    (0x021AAB48, "c1a269747cc5d424"),
    (0x021AABC4, "2de542984a587430"),
    (0x021AABEC, "9ae9caf275a47fbc"),
    (0x021AB070, "43816e9c3e1eac63"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/effect_encount.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_A7DC => {
            set_params(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_AB48 => item(&e.mem, v[0]) as u64,
        0x021A_ABC4 => step_due(&mut e.mem, v[0], v[1]) as u64,
        0x021A_ABEC => {
            reset_counter(&mut e.mem, v[0]);
            0
        }
        0x021A_B070 => shadow_item(e) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field effect_encount`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("effect_encount", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn counter_saturates() {
    let base = crate::testenv::WORK;
    let mut mem = Mem::new(base, vec![0u8; 0x600]);
    let (ewk, eff) = (base, base + 0x20);
    mem.w16(eff + EFF_INTERVAL, 2);
    assert!(!step_due(&mut mem, ewk, eff));
    assert!(step_due(&mut mem, ewk, eff));
    mem.w32(ewk + 0xC, u32::MAX);
    assert!(step_due(&mut mem, ewk, eff));
    assert_eq!(mem.r32(ewk + 0xC), u32::MAX);
    reset_counter(&mut mem, ewk);
    assert_eq!(mem.r32(ewk + 0xC), 0);
}
