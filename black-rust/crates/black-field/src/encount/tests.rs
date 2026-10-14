use super::*;
use crate::testenv::{ofs_arg, run};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A9ED8, "3f78c77aead01b66"),
    (0x021A9F28, "75c69f1048ac41b0"),
    (0x021A9F40, "59f01756b65bc0ea"),
    (0x021AA0A0, "03e76ad61646e256"),
    (0x021AA180, "62561175bcd7d0ea"),
    (0x021AA248, "24432c45bd3e0877"),
    (0x021AA304, "deb9d2aaeab47ab1"),
    (0x021AA35C, "fea6cfe9d2453a12"),
    (0x021AA3E0, "e58485b1e4fc408c"),
    (0x021AA3FC, "4a0ea365af4f3bc3"),
    (0x021AA42C, "083287addf0dde4c"),
    (0x021AA490, "ce058ebd026f8e01"),
    (0x021AA548, "137a4e54265f626b"),
    (0x021AA5CC, "bec790c19761ef47"),
    (0x021AA69C, "be901561c94fb25f"),
    (0x021AA6E0, "6cf5cf1e9c1b7cf8"),
    (0x021AA6F8, "f62887a14439d41f"),
    (0x021AA728, "5c1b2333bcb4cf7a"),
    (0x021AA778, "9e0936cf35143d00"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/encount.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_9ED8 => ofs_arg(create(e, v[0]), 1),
        0x021A_9F28 => {
            delete(e, v[0]);
            0
        }
        0x021A_9F40 => step_check(e, v[0], v[1]) as u64,
        0x021A_A0A0 => fixed_battle(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_A180 => ofs_arg(fishing(e, v[0], v[1]), 0),
        0x021A_A248 => wfbc_step(e, v[0], v[1]) as u64,
        0x021A_A304 => {
            field_situation(e, v[0], v[1], v[2]);
            0
        }
        0x021A_A35C => step_type(e, v[0], v[1], v[2]) as u64,
        0x021A_A42C => wfbc_check(e, v[0], v[1]) as u64,
        0x021A_A490 => {
            setup_wild(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021A_A548 => {
            setup_roamer(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021A_A5CC => {
            setup_trainer(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x021A_A6F8 => {
            reset_steps(e, v[0], v[1]);
            0
        }
        0x021A_A728 => {
            count_step(e, v[0], v[1]);
            0
        }
        0x021A_A3E0 => rate_of(&e.mem, v[0], v[1], v[2]) as u64,
        0x021A_A3FC => rate_check(e, v[1], v[2]) as u64,
        0x021A_A69C => weather_class(e, v[0]) as u64,
        0x021A_A6E0 => rate_roll(e) as u64,
        0x021A_A778 => steps(&e.mem, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field encount`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("encount", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn rate_of_table() {
    let base = crate::testenv::WORK;
    let mut mem = Mem::new(base, vec![0u8; 0x200]);
    mem.w32(base + 0xC, base + 0x20);
    mem.w8(base + 0x20, 10);
    mem.w8(base + 0x26, 5);
    assert_eq!(rate_of(&mem, base, 0, 10), 20);
    assert_eq!(rate_of(&mem, base, 1, 10), 0); // zero rates get no bonus
    assert_eq!(rate_of(&mem, base, 6, (-8i32) as u32), 0);
    assert_eq!(rate_of(&mem, base, 7, 0), 0);
}
