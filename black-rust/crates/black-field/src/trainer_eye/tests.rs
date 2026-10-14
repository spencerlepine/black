use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/trainer_eye.spec");

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AE0CC, "47bc89a18ba5c0ef"),
    (0x021AE198, "0c20bccb920c0f94"),
    (0x021AE1FC, "c93f75304885a8be"),
    (0x021AE288, "29c5a68dd7ba8a0d"),
    (0x021AE348, "3cc557c88f633dca"),
    (0x021AE360, "a37f5802b499ec4f"),
    (0x021AE394, "46c6d1fbc0f967f0"),
    (0x021AE3C8, "a5e9f4aa51c90424"),
    (0x021AE3FC, "a14f402dedbbaa9d"),
    (0x021AE430, "3185a441921ffc63"),
    (0x021AE454, "9001cffe3312a85d"),
    (0x021AE498, "92dbfaed48d9b959"),
    (0x021AE4B4, "4fba05629afde101"),
    (0x021AE4E0, "b9670188293d9377"),
    (0x021AE524, "872e6767de150fb9"),
    (0x021AE550, "e8c0dcef661233f7"),
    (0x021AE56C, "bdab57defc5f49bf"),
    (0x021AE58C, "b9babd05a28be11d"),
    (0x021AE5A8, "cf6bce2cddccfa6a"),
    (0x021AE5BC, "1a7c0cc7dd73aca5"),
    (0x021AE5D4, "d1dc52683dc56dab"),
    (0x021AE5EC, "cd8c9bbb23cac204"),
    (0x021AE604, "76af0193e45aede3"),
    (0x021AE630, "1ca608a9689e73ff"),
    (0x021AE650, "b04dbbc33a6f3165"),
    (0x021AE66C, "ee5fa039933de499"),
    (0x021AE6E0, "88175cb14b25aa47"),
    (0x021AE708, "b6134c28492ac676"),
    (0x021AE724, "a981c81fa0135dd5"),
    (0x021AE72C, "8efdcd0416f9f5bb"),
    (0x021AE754, "3ec2a51c052a68ac"),
    (0x021AE79C, "ed1f0dc4eeeb46bc"),
    (0x021AE7A8, "937aed02f2fe3fc5"),
    (0x021AE884, "1e54150b7c2b26bd"),
    (0x021AE890, "016f1bcc7186fab2"),
    (0x021AE8B4, "2e28e9bf224997bf"),
    (0x021AE8D4, "a1661df7546a8576"),
    (0x021AE930, "47eb113f8c280758"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        (match addr {
            0x021A_E0CC => check(e, v[0]),
            0x021A_E198 => sees_dir(e, v[0], v[1], v[2], v[3]),
            0x021A_E1FC => search(e, v[0], v[1], v[2]),
            0x021A_E288 => sees(e, v[0], v[1], v[2], v[3]),
            0x021A_E348 => line_test(&e.mem, v[0], v[1] as i32, v[2], v[3]) as u32,
            0x021A_E360 => line_up(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E394 => line_down(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E3C8 => line_left(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E3FC => line_right(&e.mem, v[0] as i32, v[1], v[2]) as u32,
            0x021A_E430 => approach_event_new(e, v[0], v[1], v[2]),
            0x021A_E454 => task_new(e, v[0], v[1], v[2], v[3]),
            0x021A_E498 => state_unpause(e, v[0]),
            0x021A_E4B4 => state_face(e, v[0]),
            0x021A_E4E0 => state_wait_player(e, v[0]),
            0x021A_E524 => state_face_acmd(e, v[0]),
            0x021A_E550 => state_face_done(e, v[0]),
            0x021A_E56C => state_balloon(e, v[0]),
            0x021A_E58C => state_balloon_done(e, v[0]),
            0x021A_E5A8 => state_acmd_65(e, v[0]),
            0x021A_E5BC => state_acmd_65_done(e, v[0]),
            0x021A_E5D4 => state_wait_30(&mut e.mem, v[0]),
            0x021A_E5EC => state_walk_check(&mut e.mem, v[0]),
            0x021A_E604 => state_walk(e, v[0]),
            0x021A_E630 => state_walk_done(e, v[0]),
            0x021A_E650 => state_wait_8(&mut e.mem, v[0]),
            0x021A_E66C => state_player_turn(e, v[0]),
            0x021A_E6E0 => state_player_turn_done(e, v[0]),
            0x021A_E708 => state_end(e, v[0]),
            0x021A_E724 => state_done(&mut e.mem, v[0]),
            0x021A_E72C => task_step(e, v[0]),
            0x021A_E754 => approach_event(e, v[0], v[1], v[2]),
            0x021A_E79C => trainer_kind(e, v[0]),
            0x021A_E7A8 => path_blocked(e, v[0], v[1], v[2], v[3]),
            0x021A_E884 => trainer_id(e, v[0]),
            0x021A_E890 => defeated(e, v[0], v[1]),
            0x021A_E8B4 => {
                record_fill(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x021A_E8D4 => partner(e, v[0], v[1]),
            0x021A_E930 => script_event(e, v[0], v[1]),
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
