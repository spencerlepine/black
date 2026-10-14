use super::*;
use crate::poke_tool::tests::Rom;
use black_fndiff::{run_suite, Case};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x0201854C, "1a566a4812a0b332")];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_854C => {
            let mut mon = std::mem::take(&mut c.regions[0]);
            let r = {
                let mut h = Rom::new(&mut c.env, c.vals[0]);
                exp_at_level(&mut mon, &mut h)
            };
            c.regions[0] = mon;
            r as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-pokemon exp`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &black_fndiff::Suite::new(
            "exp",
            include_str!("../../../../tools/rom-diff/specs/black-pokemon/exp.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}
