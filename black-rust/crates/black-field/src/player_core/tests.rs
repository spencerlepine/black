use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A3C24, "2b8a55792ed82023"),
    (0x021A3C3C, "b1666604ca238329"),
    (0x021A3C54, "8483be8f63ca7b5a"),
    (0x021A3C94, "32fc474f22c818fb"),
    (0x021A3CB0, "58ef84c83e497712"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_core.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_3C24 => {
            remember_keys(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_3C3C => key_dir_x(v[0]) as u64,
        0x021A_3C54 => key_dir_z(v[0]) as u64,
        0x021A_3C94 => first_request(v[0]) as u64,
        0x021A_3CB0 => {
            set_requests(&mut e.mem, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_core`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_core", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn pad_directions() {
    assert_eq!(key_dir_x(0x30), 2);
    assert_eq!(key_dir_x(0x10), 3);
    assert_eq!(key_dir_z(0xC0), 0);
    assert_eq!(key_dir_z(0x80), 1);
    assert_eq!(key_dir_z(0x30), DIR_NONE);
    assert_eq!(first_request(0), 10);
    assert_eq!(first_request(0x400), 10);
    assert_eq!(first_request(0x104), 2);
}
