use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021ACA38, "0996a7b5b01ccb07"),
    (0x021ACA5C, "fa380ef2c7f2721c"),
    (0x021ACB38, "b1dca782ca4dcdf0"),
    (0x021ACBB8, "a69a84dd05f40bfe"),
    (0x021ACBC8, "6f32a9ce57d9119f"),
    (0x021ACBE4, "d931040f589761aa"),
    (0x021ACC04, "adf6c2eab41c6a30"),
    (0x021ACC3C, "662654a2eb45d3c3"),
    (0x021ACCAC, "15e96184e19bea96"),
    (0x021ACDC4, "e5d07f14795608c0"),
    (0x021ACFF8, "ec022425ddfb00c9"),
    (0x021AD3D4, "f8536b2e19aa3b68"),
    (0x021AD3FC, "0573f2bba4423bf1"),
    (0x021AD564, "7b42f2526abd2b1f"),
    (0x021AD598, "675b5497e42c0456"),
    (0x021AD5CC, "a116e9a09ee6abc4"),
    (0x021AD5E0, "000039eadce9283e"),
    (0x021AD6B8, "2dc75a7fe47c7ea4"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/namipoke.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_CA38 => res_create(e, v[0], v[1]) as u64,
        0x021A_CA5C => {
            res_delete(e, v[0], v[1]);
            0
        }
        0x021A_CB38 => add_surf_task(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021A_CBB8 => {
            set_mode(e, v[0], v[1]);
            0
        }
        0x021A_CBC8 => {
            set_visible(e, v[0], v[1]);
            0
        }
        0x021A_CBE4 => follow_speed(v[0] as i32) as u32 as u64,
        0x021A_CC04 => ahead_along(&e.mem, v[0], v[1], v[2]) as u64,
        0x021A_CC3C => {
            surf_init(e, v[0], v[1]);
            0
        }
        0x021A_CCAC => {
            draw_none();
            0
        }
        0x021A_CDC4 => {
            surf_update_grid(e, v[0], v[1]);
            0
        }
        0x021A_CFF8 => {
            surf_update_rail(e, v[0], v[1]);
            0
        }
        0x021A_D3D4 => splash_res_create(e, v[0], v[1]) as u64,
        0x021A_D3FC => {
            splash_res_delete(e, v[0], v[1]);
            0
        }
        0x021A_D564 => add_splash_follow(e, v[0], v[1], v[2]) as u64,
        0x021A_D598 => add_splash_once(e, v[0], v[1], v[2]) as u64,
        0x021A_D5CC => splash_done(e, v[0]) as u64,
        0x021A_D5E0 => {
            splash_init(e, v[0], v[1]);
            0
        }
        0x021A_D6B8 => {
            splash_update(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field namipoke`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("namipoke", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn follow_speed_clamps() {
    assert_eq!(follow_speed(0x5_0000), 0x4_0000 / 0xA3D * 0x28);
    assert_eq!(follow_speed(0xA3C), 0);
    assert_eq!(follow_speed(-0xA3D), -0x28);
}
