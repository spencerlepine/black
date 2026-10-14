use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02185694, "bcf030481cd5e586"),
    (0x0218571C, "8818778ac7b790f6"),
    (0x02185758, "8364b0da1410ae4b"),
    (0x02185788, "a48846b208691807"),
    (0x021857C0, "380c043e22e413f5"),
    (0x021858E4, "f053e03bec590d41"),
    (0x02185928, "f49d22c5002e956e"),
    (0x021859F0, "74797f93af3037b1"),
    (0x021859FC, "96b75e2598643a57"),
    (0x02185A34, "df93846188dbb564"),
    (0x02185A68, "d2969e372a620903"),
    (0x02185B70, "0f363f248ff80a8f"),
    (0x02185B88, "b30b6eea248bbf7e"),
    (0x02185B9C, "688e4cfc1585a620"),
    (0x02185BFC, "7a14810a063ea4bc"),
    (0x02185C4C, "ab8ef24a9110dbaf"),
    (0x02185C8C, "bfe549d0acbe2110"),
    (0x02185D38, "1a816c06853298b9"),
    (0x02185EB0, "e2cfbe382b26c368"),
    (0x02185F18, "4aed2fb8b0328236"),
    (0x02185F48, "e88d4dc85e72fb5e"),
    (0x02185F6C, "d36cc8ba565c22bc"),
    (0x0218601C, "2075fe0a5e5c06fc"),
    (0x0218604C, "8371002620f05e62"),
    (0x0218606C, "4de03e08c75e7374"),
    (0x0218608C, "c99542a0deb47d2d"),
    (0x02186090, "b3484dcf03db5f8a"),
    (0x0218609C, "44a400ac419ea3ed"),
    (0x021860B8, "49afec2a37efc57f"),
    (0x021860D4, "f1ce89e3a1020d03"),
    (0x021861BC, "3302aab3fd2b5156"),
    (0x021861F4, "630a81799692158c"),
    (0x0218625C, "a5ab9994da548f71"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mapchange_zone.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0218_5694 => special_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_571C => special_new(e, v[0]) as u64,
        0x0218_5758 => dir_angle(v[0]) as u64,
        0x0218_5788 => {
            resolve(e, v[0], v[1], v[2]);
            0
        }
        0x0218_57C0 => {
            zone_enter(e, v[0], v[1]);
            0
        }
        0x0218_58E4 => {
            zone_npcs(e, v[0], v[1]);
            0
        }
        0x0218_5928 => {
            wfbc_people(e, v[0], v[1]);
            0
        }
        0x0218_59F0 => {
            mmdl_reset(e, v[0]);
            0
        }
        0x0218_59FC => {
            wfbc_hook(e, v[0], v[1]);
            0
        }
        0x0218_5A34 => {
            flash_flags(e, v[0], v[1]);
            0
        }
        0x0218_5A68 => {
            zone_load(e, v[0], v[1]);
            0
        }
        0x0218_5B70 => {
            zone_leave(e, v[0]);
            0
        }
        0x0218_5B88 => {
            player_mmdl_set(e, v[0], v[1]);
            0
        }
        0x0218_5B9C => {
            sb_list(e, v[0], v[1]);
            0
        }
        0x0218_5BFC => comm_new(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x0218_5C4C => comm_change_new(e, v[0], v[1], v[2]) as u64,
        0x0218_5C8C => comm_change_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_5D38 => comm_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_5EB0 => {
            special_update(e, v[0], v[1]);
            0
        }
        0x0218_5F18 => {
            special_copy(e, v[0], v[1]);
            0
        }
        0x0218_5F48 => {
            form_reset(e, v[0]);
            0
        }
        0x0218_5F6C => {
            default_record(e, v[0]);
            0
        }
        0x0218_601C => {
            season_records(e, v[0]);
            0
        }
        0x0218_604C => alloc_a(e) as u64,
        0x0218_606C => alloc_b(e) as u64,
        0x0218_608C => {
            nop();
            0
        }
        0x0218_6090 => obj_index(e, v[0]) as u64,
        0x0218_609C => {
            list_a_clear(e, v[0]);
            0
        }
        0x0218_60B8 => {
            list_b_clear(e, v[0]);
            0
        }
        0x0218_60D4 => {
            people_generate(e, v[0], v[1], v[2]);
            0
        }
        0x0218_61BC => list_ready(e, v[0]) as u64,
        0x0218_61F4 => {
            list_update(e, v[0], v[1]);
            0
        }
        0x0218_625C => list_check(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mapchange_zone`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mapchange_zone", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn dir_angle_codes() {
    assert_eq!(
        (0..6).map(dir_angle).collect::<Vec<_>>(),
        [0, 0, 0x8000, 0x4000, 0xC000, 0]
    );
}
