use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02197F5C, "7c35198a5c82f60a"),
    (0x02197F6C, "9bb657d6daad12f0"),
    (0x02197F90, "6b6a22ff650d4cbd"),
    (0x02197F9C, "235018cea121d98b"),
    (0x02197FA8, "15fcd5c5a8f747d3"),
    (0x02197FB4, "beca78d49e5659b1"),
    (0x02197FC0, "d032acac0c5cc728"),
    (0x0219800C, "b242a680dcfb4d3b"),
    (0x02198060, "7c8c66ed381c4cce"),
    (0x02198074, "7586598b79aaa769"),
    (0x02198088, "a98890d108bdadd0"),
    (0x0219809C, "db4fe9f6be011fae"),
    (0x021980B0, "baf0a296a8ff321c"),
    (0x021980C4, "5339524c391054ba"),
    (0x021980DC, "5582a46242c87cde"),
    (0x021980F0, "d67f79d35f34b33b"),
    (0x02198104, "0f658fe825c0ccf8"),
    (0x0219811C, "c1db7db03105967e"),
    (0x02198134, "2b2be7df5cfc9824"),
    (0x02198148, "a1d8835b3aed21b2"),
    (0x0219815C, "86d0e2dcb4cd63da"),
    (0x02198170, "2c89cbdf26925684"),
    (0x02198184, "df798cb8ad8e7d26"),
    (0x02198198, "68940af05ed8c016"),
    (0x021981AC, "03b0803d459997ee"),
    (0x021981C4, "81ec598e6eea62ef"),
    (0x021981DC, "a6f5e11595fab691"),
    (0x021981F0, "8aa83d4ca26ed3b6"),
    (0x02198208, "cb99905d98520570"),
    (0x0219821C, "3617318b8fd3ab79"),
    (0x02198234, "9b8728fed0f20573"),
    (0x02198248, "1a284dcf3c3920dd"),
    (0x02198260, "2f548a5727057d05"),
    (0x02198278, "6f4716c136f4e102"),
    (0x02198290, "6620531bd45bafa7"),
    (0x021982A4, "952fd415bfc17714"),
    (0x021982BC, "8224075ffc4d6740"),
    (0x021982D4, "c69475cd5a2ff8ed"),
    (0x021982EC, "1348b98144308dbf"),
    (0x02198300, "f5cd635f0538c6dc"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2a.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_7F5C => end_step(e, v[0]) as u64,
        0x0219_7F6C => {
            face(e, v[0], v[1]);
            0
        }
        0x0219_7F90 => face_up(e, v[0]) as u64,
        0x0219_7F9C => face_down(e, v[0]) as u64,
        0x0219_7FA8 => face_left(e, v[0]) as u64,
        0x0219_7FB4 => face_right(e, v[0]) as u64,
        0x0219_7FC0 => {
            walk_start(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x0219_800C => walk_update(e, v[0]) as u64,
        0x0219_8060 => walk_04(e, v[0]) as u64,
        0x0219_8074 => walk_05(e, v[0]) as u64,
        0x0219_8088 => walk_06(e, v[0]) as u64,
        0x0219_809C => walk_07(e, v[0]) as u64,
        0x0219_80B0 => walk_08(e, v[0]) as u64,
        0x0219_80C4 => walk_09(e, v[0]) as u64,
        0x0219_80DC => walk_0a(e, v[0]) as u64,
        0x0219_80F0 => walk_0b(e, v[0]) as u64,
        0x0219_8104 => walk_0c(e, v[0]) as u64,
        0x0219_811C => walk_0d(e, v[0]) as u64,
        0x0219_8134 => walk_0e(e, v[0]) as u64,
        0x0219_8148 => walk_0f(e, v[0]) as u64,
        0x0219_815C => walk_10(e, v[0]) as u64,
        0x0219_8170 => walk_11(e, v[0]) as u64,
        0x0219_8184 => walk_12(e, v[0]) as u64,
        0x0219_8198 => walk_13(e, v[0]) as u64,
        0x0219_81AC => walk_14(e, v[0]) as u64,
        0x0219_81C4 => walk_15(e, v[0]) as u64,
        0x0219_81DC => walk_16(e, v[0]) as u64,
        0x0219_81F0 => walk_17(e, v[0]) as u64,
        0x0219_8208 => walk_54(e, v[0]) as u64,
        0x0219_821C => walk_55(e, v[0]) as u64,
        0x0219_8234 => walk_56(e, v[0]) as u64,
        0x0219_8248 => walk_57(e, v[0]) as u64,
        0x0219_8260 => walk_58(e, v[0]) as u64,
        0x0219_8278 => walk_59(e, v[0]) as u64,
        0x0219_8290 => walk_5a(e, v[0]) as u64,
        0x0219_82A4 => walk_5b(e, v[0]) as u64,
        0x0219_82BC => walk_9b(e, v[0]) as u64,
        0x0219_82D4 => walk_9c(e, v[0]) as u64,
        0x0219_82EC => walk_9d(e, v[0]) as u64,
        0x0219_8300 => walk_9e(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2a`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2a", SPEC, PORTED, RECORDED), &call);
}
