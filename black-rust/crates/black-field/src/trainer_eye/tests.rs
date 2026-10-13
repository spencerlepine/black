use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/trainer_eye.spec");

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AE348, "3cc557c88f633dca"),
    (0x021AE360, "a37f5802b499ec4f"),
    (0x021AE394, "46c6d1fbc0f967f0"),
    (0x021AE3C8, "a5e9f4aa51c90424"),
    (0x021AE3FC, "a14f402dedbbaa9d"),
    (0x021AE5D4, "d1dc52683dc56dab"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        (match addr {
            0x021A_E348 => line_test(&e.mem, v[0], v[1] as i32, v[2], v[3]) as u32,
            0x021A_E360 => line_up(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E394 => line_down(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E3C8 => line_left(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E3FC => line_right(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E5D4 => state_wait_30(&mut e.mem, v[0]),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }) as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field trainer_eye`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("trainer_eye", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn line_tests_by_direction() {
    // trainer at (2, 0, 2); positions are s16 x, y, z
    let mut m = Mem::new(0x100, vec![0; 0x20]);
    let pos = |m: &mut Mem, at: u32, x: i16, y: i16, z: i16| {
        m.w16(at, x as u16);
        m.w16(at + 2, y as u16);
        m.w16(at + 4, z as u16);
    };
    pos(&mut m, 0x100, 2, 0, 2);
    pos(&mut m, 0x108, 2, 0, 0); // two tiles up
    assert_eq!(line_test(&m, 0, 2, 0x100, 0x108), 2);
    assert_eq!(line_test(&m, 0, 1, 0x100, 0x108), -1); // out of range
    assert_eq!(line_test(&m, 1, 5, 0x100, 0x108), -1); // behind
    pos(&mut m, 0x108, 5, 0, 2); // three tiles right
    assert_eq!(line_test(&m, 3, 3, 0x100, 0x108), 3);
    assert_eq!(line_test(&m, 2, 3, 0x100, 0x108), -1);
    pos(&mut m, 0x108, 5, 1, 2); // other height
    assert_eq!(line_test(&m, 3, 3, 0x100, 0x108), -1);
    pos(&mut m, 0x108, 2, 0, 2); // same tile
    assert!((0..4).all(|d| line_test(&m, d, 9, 0x100, 0x108) == -1));
}
