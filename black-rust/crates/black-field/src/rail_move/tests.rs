use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021B88DC, "242a5653a2a485b2"),
    (0x021B9164, "e56f8700a040cfef"),
    (0x021B919C, "a471af3283e5ea83"),
    (0x021B91A8, "46ba3457dca148b0"),
    (0x021B91AC, "d354c64fd6bdee68"),
    (0x021B91C8, "6a9f83cd0894971f"),
    (0x021B91F8, "aa9b8851dccb9f1d"),
    (0x021B9238, "a54900c39a2ac70f"),
    (0x021B9278, "407c08dabaae7024"),
    (0x021B9280, "bf1a341b1638554e"),
    (0x021B9338, "c4502c741a27631c"),
    (0x021B9368, "eb559e266ed6171d"),
    (0x021B93D4, "1dddfe7c6e675b6f"),
    (0x021B9404, "0a17d813ae893060"),
    (0x021B9434, "4f911b353a880ef1"),
    (0x021B9464, "12a948d4243d33f4"),
    (0x021B9478, "d6c4c67b1b46589a"),
    (0x021B949C, "2c7fa0cd286b8bd9"),
    (0x021B94C0, "0860a533ca14e397"),
    (0x021B94D8, "87d1fa4f97e04c76"),
    (0x021B9510, "e4ed6fcf6ff71a30"),
    (0x021B9530, "bd3946f1329dfc9d"),
    (0x021B95C0, "f552d0a8cc39e66d"),
    (0x021B960C, "d5aa06d4486684aa"),
    (0x021B963C, "e05e96c18c817896"),
    (0x021B96AC, "f33eefd9eff67be6"),
    (0x021B9768, "5f0019f709706958"),
    (0x021B976C, "1fbb6ca20c482e7d"),
    (0x021B977C, "17d9abf0523cd68f"),
    (0x021B97D0, "11f5f02cf6182395"),
    (0x021B97FC, "9cacaf9c01d132d3"),
    (0x021B9828, "940f3e26c37a1bb5"),
    (0x021B9A50, "69394d54ad68bb12"),
    (0x021B9B24, "f88db7501aaecf10"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/rail_move.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021B_88DC => u64::from(line_move(e, v[0], v[1], v[2])),
        0x021B_9164 => {
            obj_init(e, v[0], v[1], v[2]);
            0
        }
        0x021B_919C => {
            obj_reset(e, v[0], v[1]);
            0
        }
        0x021B_91A8 => u64::from(obj_in_use(e, v[0])),
        0x021B_91AC => u64::from(active(e, v[0])),
        0x021B_91C8 => u64::from(cam_of(e, v[0])),
        0x021B_91F8 => {
            pos_of(e, v[0], v[1]);
            0
        }
        0x021B_9238 => {
            loc_of(e, v[0], v[1]);
            0
        }
        0x021B_9278 => u64::from(line_len(e, v[0]) as u32),
        0x021B_9280 => u64::from(line_width(e, v[0], v[1], v[2] as i32, v[3]) as u32),
        0x021B_9338 => u64::from(clamp_lat(e, v[0], v[1], v[2], v[3])),
        0x021B_9368 => u64::from(walk(e, v[0], v[1], v[2], v[3], v[4])),
        0x021B_93D4 => u64::from(dir_back(v[0])),
        0x021B_9404 => u64::from(dir_right(v[0])),
        0x021B_9434 => u64::from(dir_left(v[0])),
        0x021B_9464 => {
            blk_clear(e, v[0]);
            0
        }
        0x021B_9478 => u64::from(blk_add(e, v[0], v[1])),
        0x021B_949C => u64::from(blk_remove(e, v[0], v[1])),
        0x021B_94C0 => u64::from(blk_has(e, v[0], v[1])),
        0x021B_94D8 => u64::from(blk_has_line(e, v[0], v[1], v[2])),
        0x021B_9510 => u64::from(line_if_open(e, v[0], v[1], v[2])),
        0x021B_9530 => {
            side_step(e, v[0], v[1], v[2] as i32, v[3] as i32, v[4]);
            0
        }
        0x021B_95C0 => u64::from(can_enter(
            e,
            v[0],
            v[1],
            v[2] as i32,
            v[3] as i32,
            v[4],
            v[5],
        )),
        0x021B_960C => u64::from(dir_of_line(e, v[0], v[1], v[2])),
        0x021B_963C => u64::from(corner_ok_line(e, v[0], v[1])),
        0x021B_96AC => u64::from(corner_ok(e, v[0], v[1])),
        0x021B_9768 => {
            debug_hook();
            0
        }
        0x021B_976C => {
            debug_point(v[1]);
            0
        }
        0x021B_977C => {
            debug_work(e, v[0]);
            0
        }
        0x021B_97D0 => u64::from(point_index(e, v[0], v[1])),
        0x021B_97FC => u64::from(line_index(e, v[0], v[1])),
        0x021B_9828 => u64::from(seg_hit(e, v[0], v[1], v[2], v[3], v[4], v[5])),
        0x021B_9A50 => {
            slerp(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021B_9B24 => {
            interp(e, v[0], v[1], v[2], v[3]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field rail_move`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("rail_move", SPEC, PORTED, RECORDED), &call);
}
