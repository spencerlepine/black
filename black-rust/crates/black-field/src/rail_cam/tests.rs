use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B9B7C, "d742b26c87e8bd64"),
    (0x021B9C28, "7eaac1cab3c3a7ff"),
    (0x021B9D60, "aaed7aab389f5c4c"),
    (0x021B9E84, "2cd01961cf042d0c"),
    (0x021B9FC8, "7a9b2e430dcab5e6"),
    (0x021BA25C, "e7224a9390086d8f"),
    (0x021BA2B4, "f1edc471cfc0b50c"),
    (0x021BA310, "f2dcff92afacc40c"),
    (0x021BA348, "8eea71b594aa5ce8"),
    (0x021BA494, "09a90c14400a925a"),
    (0x021BA4C4, "7241d1e11418ba71"),
    (0x021BA4FC, "0b6119502c43d63a"),
    (0x021BA550, "c9974bb155397c4b"),
    (0x021BA6E8, "915683b393d73dad"),
    (0x021BA7B0, "5542eb9943d91801"),
    (0x021BA7E0, "1d82898f2bdc4b9c"),
    (0x021BA8E0, "2e1e899d24950771"),
    (0x021BA9B4, "41c8a164c4bf84ae"),
    (0x021BAA30, "02bb264a7e17835a"),
    (0x021BAA9C, "aafd5599161457cf"),
    (0x021BAB30, "071fe4731b7fec97"),
    (0x021BABB4, "3cf6036404a695e4"),
    (0x021BABDC, "caaa70c94e627551"),
    (0x021BAC04, "49c07d43f29e5a31"),
    (0x021BAC74, "c64e8264e6d7e345"),
    (0x021BACA8, "15e88e840e133119"),
    (0x021BAE7C, "219266626bdb4898"),
    (0x021BB014, "25dd11cb2f549e3d"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/rail_cam.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021B_A25C => {
            cam_follow(e, v[0]);
            0
        }
        0x021B_A2B4 => {
            cam_angles(e, v[0]);
            0
        }
        0x021B_A310 => {
            cam_angles_pos(e, v[0]);
            0
        }
        0x021B_A348 => {
            cam_interp(e, v[0]);
            0
        }
        0x021B_A494 => {
            cam_interp_pos(e, v[0]);
            0
        }
        0x021B_A4C4 => {
            cam_interp_pos2(e, v[0]);
            0
        }
        0x021B_A4FC => {
            cam_fixed(e, v[0]);
            0
        }
        0x021B_A550 => {
            cam_fixed_interp(e, v[0]);
            0
        }
        0x021B_A6E8 => {
            cam_orbit_interp(e, v[0]);
            0
        }
        0x021B_A7B0 => {
            cam_orbit(e, v[0]);
            0
        }
        0x021B_A8E0 => {
            cam_orbit_moving(e, v[0]);
            0
        }
        0x021B_A9B4 => {
            cam_face(e, v[0]);
            0
        }
        0x021B_AA30 => {
            cam_behind(e, v[0]);
            0
        }
        0x021B_AA9C => {
            cam_behind_pitch(e, v[0]);
            0
        }
        0x021B_AB30 => {
            cam_behind_pos(e, v[0]);
            0
        }
        0x021B_ABB4 => {
            cam_start_point(e, v[0]);
            0
        }
        0x021B_ABDC => {
            cam_end_point(e, v[0]);
            0
        }
        0x021B_AC04 => {
            cam_angles_vec(e, v[0]);
            0
        }
        0x021B_AC74 => {
            cam_angles_vec_pos(e, v[0]);
            0
        }
        0x021B_ACA8 => {
            cam_interp_full(e, v[0]);
            0
        }
        0x021B_9B7C => {
            pos_line(e, v[0], v[1]);
            0
        }
        0x021B_9C28 => {
            pos_curve(e, v[0], v[1]);
            0
        }
        0x021B_9D60 => {
            pos_curve3(e, v[0], v[1]);
            0
        }
        0x021B_9E84 => u64::from(hit_line(e, v[0], v[1], v[2], v[3], v[4])),
        0x021B_9FC8 => u64::from(hit_steps(e, v[0], v[1], v[2], v[3], v[4])),
        0x021B_A7E0 => {
            orbit(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_AE7C => u64::from(tri_hit(e, v[0], v[1], v[2], v[3], v[4], v[5])),
        0x021B_B014 => {
            cam_at_point(e, v[0], v[1], v[2]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field rail_cam`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("rail_cam", SPEC, PORTED, RECORDED), &call);
}
