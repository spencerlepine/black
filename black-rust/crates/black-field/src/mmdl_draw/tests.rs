use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02197C4C, "c606af7e1bb1d85a"),
    (0x02197C78, "a5c9e59d614ed474"),
    (0x02197C9C, "f07f62817440e8e9"),
    (0x02197CCC, "e5e4d12e291b6fa4"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mmdl_draw.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_7C4C => {
            delete_draw(e, v[0]);
            0
        }
        0x0219_7C78 => {
            draw(e, v[0]);
            0
        }
        0x0219_7C9C => anim_held(e, v[0]) as u64,
        0x0219_7CCC => {
            draw_pos(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mmdl_draw`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mmdl_draw", SPEC, PORTED, RECORDED), &call);
}
