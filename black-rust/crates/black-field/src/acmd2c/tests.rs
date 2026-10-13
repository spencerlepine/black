use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02198960, "e8a48d22613cdf5e"),
    (0x02198978, "abb75e924af358bf"),
    (0x02198998, "240ec7dbe62819a4"),
    (0x021989A4, "b04c026b267e2486"),
    (0x021989B0, "62d3cbb67787c647"),
    (0x021989BC, "fdb14095036c0c0e"),
    (0x021989C8, "ede33609bc5a412c"),
    (0x021989D4, "5df0bb36ac3c3b2b"),
    (0x021989E0, "3cc29bcb829a2c63"),
    (0x021989EC, "746831addfcf415f"),
    (0x02198A10, "e18d944b4f43a5a5"),
    (0x02198A58, "5c1f76c079978f65"),
    (0x02198A8C, "68e00bfadd6b5ca9"),
    (0x02198AD4, "074636bc44647db5"),
    (0x02198AE8, "921cf33ed421f70d"),
    (0x02198AFC, "44bec72de2e5cc10"),
    (0x02198B10, "2cb169af9d375071"),
    (0x02198B24, "a4da648fe512c204"),
    (0x02198B38, "7ff159b3a37da08d"),
    (0x02198B4C, "7bec43efce943fe7"),
    (0x02198B80, "9dc5638fd40d03b1"),
    (0x02198BA8, "0fc8eed911fa4db6"),
    (0x02198BB8, "b6067566cc156122"),
    (0x02198BC8, "b8a7c72f9c88e616"),
    (0x02198BD8, "96930be064ffc381"),
    (0x02198BE8, "b956677089b85e92"),
    (0x02198BF8, "ed6695420ad96338"),
    (0x02198C08, "6cf46cfc893553ee"),
    (0x02198C4C, "03f6a843e2bb51db"),
    (0x02198CA4, "b6597c990d1b0162"),
    (0x02198CB4, "646ad1f7ff4e76f4"),
    (0x02198CC4, "92651ef2d2a8e104"),
    (0x02198CD4, "3e55ecf3113b8d50"),
    (0x02198CE4, "b40e5aec16292a3c"),
    (0x02198CF4, "b5ba39577d712079"),
    (0x02198D04, "f8d30755241dc6d7"),
    (0x02198D14, "a540b2a23ac29e3a"),
    (0x02198D24, "122f56eee0e552b1"),
    (0x02198D3C, "5f37e49c49a7b736"),
    (0x02198D4C, "a567daedfb147367"),
    (0x02198D5C, "d8c3d7f2469fd373"),
    (0x02198D6C, "97967fcbc30c44a3"),
    (0x02198D7C, "7c8ab46d67c9fe1a"),
    (0x02198D94, "b2153cb3f9cfd23a"),
    (0x02198DA4, "6b9f268161601d2c"),
    (0x02198DB4, "de8714e5f4a0c90f"),
    (0x02198DC4, "8a6ca74ae762e6df"),
    (0x02198DD4, "7ddb51bd48c6714f"),
    (0x02198DEC, "ad7e5d4693de01a0"),
    (0x02198DFC, "dde4285f9105dd36"),
    (0x02198E0C, "6ecce6ac1b991f73"),
    (0x02198E1C, "0aa28aa41f29080f"),
    (0x02198E2C, "e6c974b8c06cd8b3"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2c.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_8960 => {
            wait_start(e, v[0], v[1]);
            0
        }
        0x0219_8978 => wait_update(e, v[0]) as u64,
        0x0219_89BC => wait_8(e, v[0]) as u64,
        0x0219_8A8C => fall_update(e, v[0]) as u64,
        0x0219_8C4C => table_walk_update(e, v[0], v[1]) as u64,
        0x0219_8D7C => table_walk_update_3(e, v[0]) as u64,
        0x0219_89EC => rise_start(e, v[0]) as u64,
        0x0219_8A10 => rise_update(e, v[0]) as u64,
        0x0219_8A58 => fall_start(e, v[0]) as u64,
        0x0219_8AD4 => status4_on(e, v[0]) as u64,
        0x0219_8AE8 => status4_off(e, v[0]) as u64,
        0x0219_8AFC => dir_lock_on(e, v[0]) as u64,
        0x0219_8B10 => dir_lock_off(e, v[0]) as u64,
        0x0219_8B24 => anim_hold_on(e, v[0]) as u64,
        0x0219_8B38 => anim_hold_off(e, v[0]) as u64,
        0x0219_8B4C => {
            balloon_start(e, v[0], v[1]);
            0
        }
        0x0219_8B80 => balloon_update(e, v[0]) as u64,
        0x0219_8C08 => {
            table_walk_start(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0219_8D24 => table_walk_update_6(e, v[0]) as u64,
        0x0219_8DD4 => table_walk_update_7(e, v[0]) as u64,
        0x0219_8E2C => table_walk_update_12(e, v[0]) as u64,
        0x0219_8998 => wait_3c(e, v[0]) as u64,
        0x0219_89A4 => wait_3d(e, v[0]) as u64,
        0x0219_89B0 => wait_3e(e, v[0]) as u64,
        0x0219_89C8 => wait_40(e, v[0]) as u64,
        0x0219_89D4 => wait_41(e, v[0]) as u64,
        0x0219_89E0 => wait_42(e, v[0]) as u64,
        0x0219_8BA8 => balloon_4b(e, v[0]) as u64,
        0x0219_8BB8 => balloon_67(e, v[0]) as u64,
        0x0219_8BC8 => balloon_99(e, v[0]) as u64,
        0x0219_8BD8 => balloon_9f(e, v[0]) as u64,
        0x0219_8BE8 => balloon_a0(e, v[0]) as u64,
        0x0219_8BF8 => balloon_a1(e, v[0]) as u64,
        0x0219_8CA4 => table_walk_4c(e, v[0]) as u64,
        0x0219_8CB4 => table_walk_4d(e, v[0]) as u64,
        0x0219_8CC4 => table_walk_4e(e, v[0]) as u64,
        0x0219_8CD4 => table_walk_4f(e, v[0]) as u64,
        0x0219_8CE4 => table_walk_a3(e, v[0]) as u64,
        0x0219_8CF4 => table_walk_a4(e, v[0]) as u64,
        0x0219_8D04 => table_walk_a5(e, v[0]) as u64,
        0x0219_8D14 => table_walk_a6(e, v[0]) as u64,
        0x0219_8D3C => table_walk_50(e, v[0]) as u64,
        0x0219_8D4C => table_walk_51(e, v[0]) as u64,
        0x0219_8D5C => table_walk_52(e, v[0]) as u64,
        0x0219_8D6C => table_walk_53(e, v[0]) as u64,
        0x0219_8D94 => table_walk_60(e, v[0]) as u64,
        0x0219_8DA4 => table_walk_61(e, v[0]) as u64,
        0x0219_8DB4 => table_walk_62(e, v[0]) as u64,
        0x0219_8DC4 => table_walk_63(e, v[0]) as u64,
        0x0219_8DEC => table_walk_a7(e, v[0]) as u64,
        0x0219_8DFC => table_walk_a8(e, v[0]) as u64,
        0x0219_8E0C => table_walk_a9(e, v[0]) as u64,
        0x0219_8E1C => table_walk_aa(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2c`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2c", SPEC, PORTED, RECORDED), &call);
}

/// [`SPEED_TABLES`] are the build's, and the four updates pass those addresses (their literal pools).
#[test]
fn speed_tables_match_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    for (a, t) in SPEED_TABLES.iter() {
        for (k, &s) in t.iter().enumerate() {
            assert_eq!(w(a + 4 * k as u32), s, "{a:#x} entry {k}");
        }
    }
    assert_eq!(w(0x0219_8D38), 0x021D_4A68);
    assert_eq!(w(0x0219_8D90), 0x021D_4A44);
    assert_eq!(w(0x0219_8DE8), 0x021D_4A80);
    assert_eq!(w(0x0219_8E40), 0x021D_4A9C);
}
