use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02198318, "c8a9f52d9932846b"),
    (0x0219834C, "95e7b40431641801"),
    (0x02198380, "007eb8597db33785"),
    (0x021984C0, "35b2d5fc63789f24"),
    (0x0219852C, "6a96f6d1dd051f71"),
    (0x02198550, "03e9fc650c6433f2"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2b.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_8318 => {
            stay_start(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0219_834C => stay_update(e, v[0]) as u64,
        0x0219_8380 => stay_up_32(e, v[0]) as u64,
        0x0219_84C0 => {
            jump_start(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
            0
        }
        0x0219_852C => {
            jump_start_se(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x0219_8550 => jump_update(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2b`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2b", SPEC, PORTED, RECORDED), &call);
}
