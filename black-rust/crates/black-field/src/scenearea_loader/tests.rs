use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AB7E4, "8a4b703374453bae"),
    (0x021AB81C, "95cadb6caf8162b2"),
    (0x021AB840, "d720e536c0ab756a"),
    (0x021AB868, "2d726e229e716753"),
    (0x021AB880, "211946f7a6f8f672"),
    (0x021AB884, "a990d75715547b2c"),
    (0x021AB888, "2b28767877c37fdd"),
    (0x021AB890, "f67b26e8a8a81d2d"),
    (0x021AB908, "95c25698c3dcc8ae"),
    (0x021AB9FC, "b2654f2c6f4b4fcb"),
    (0x021ABA3C, "fe5d105996c5a021"),
    (0x021ABABC, "643472dc68f43d8b"),
    (0x021ABB54, "65a123754f607516"),
    (0x021ABB78, "b1a674af9d2756f3"),
    (0x021ABBA0, "d0d2260a546cc64e"),
    (0x021ABBF0, "5ad1eca183e37282"),
    (0x021ABC14, "20963c9ae4874acd"),
    (0x021ABC38, "ca0219e5eda3e9f4"),
    (0x021ABC5C, "bb0bf54f08257e18"),
    (0x021ABD08, "b76eee94dc523b96"),
];
// fndiff-recorded-end

const SPEC: &str =
    include_str!("../../../../tools/rom-diff/specs/black-field/scenearea_loader.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        match addr {
            0x021A_B7E4 => return new(e, v[0]) as u64,
            0x021A_B81C => delete(e, v[0]),
            0x021A_B840 => load(e, v[0], v[1], v[2], v[3]),
            0x021A_B868 => free_list(e, v[0]),
            0x021A_B880 => return list(e, v[0]) as u64,
            0x021A_B884 => return count(e, v[0]) as u64,
            0x021A_B888 => return callback_table() as u64,
            0x021A_B890 => return check_ring(e, v[0], v[1], v[2]) as u64,
            0x021A_B908 => cam_circle(e, v[0], v[1], v[2]),
            0x021A_B9FC => cam_fixed(e, v[0], v[1], v[2]),
            0x021A_BA3C => return check_rect(e, v[0], v[1], v[2]) as u64,
            0x021A_BABC => cam_lerp(e, v[0], v[1], v[2]),
            0x021A_BB54 => cam_lerp_pre(e, v[0], v[1], v[2]),
            0x021A_BB78 => cam_lerp_post(e, v[0], v[1], v[2]),
            0x021A_BBA0 => rect_pos(e, v[0], v[1], v[2], v[3]),
            0x021A_BBF0 => cam_lerp_all(e, v[0], v[1], v[2]),
            0x021A_BC14 => cam_lerp_all_pre(e, v[0], v[1], v[2]),
            0x021A_BC38 => cam_lerp_all_post(e, v[0], v[1], v[2]),
            0x021A_BC5C => cam_offset(e, v[0], v[1], v[2]),
            0x021A_BD08 => cam_fov(e, v[0], v[1], v[2]),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
        0
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field scenearea_loader`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new("scenearea_loader", SPEC, PORTED, RECORDED),
        &call,
    );
}

/// [`ARCS`] and [`CALLBACK_TABLE`]'s descriptor are the ROM's (overlay 21 `0x021D8EEC` / `0x021D8EF4`).
#[test]
fn tables_match_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    assert_eq!([w(0x021D_8EEC), w(0x021D_8EF0)], ARCS);
    assert_eq!(w(CALLBACK_TABLE), 0x021D_DFD8);
    assert_eq!(w(CALLBACK_TABLE + 4), 0x021D_DFE0);
    assert_eq!(w(CALLBACK_TABLE + 8), 0x0008_0002);
    let checks = [0x021A_B891, 0x021A_BA3D];
    let updates = [
        0x021A_B909,
        0x021A_B9FD,
        0x021A_BABD,
        0x021A_BB55,
        0x021A_BB79,
        0x021A_BBF1,
        0x021A_BC15,
        0x021A_BC39,
    ];
    for (i, &f) in checks.iter().enumerate() {
        assert_eq!(w(0x021D_DFD8 + 4 * i as u32), f);
    }
    for (i, &f) in updates.iter().enumerate() {
        assert_eq!(w(0x021D_DFE0 + 4 * i as u32), f);
    }
}
