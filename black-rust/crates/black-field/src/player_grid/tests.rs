use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A4140, "f8999a948d3b9c8d"),
    (0x021A41B0, "481e4d643f3359fc"),
    (0x021A4224, "9e174687da6bfbfd"),
    (0x021A4254, "40e03bd5afb98561"),
    (0x021A428C, "e25c05977444568f"),
    (0x021A43D8, "6aae018e9327aec3"),
    (0x021A4444, "f77285c906fbd3dd"),
    (0x021A448C, "0b5dfbf7dd238f6f"),
    (0x021A44BC, "db140ed574086c99"),
    (0x021A44C8, "d8b26c7067a31c60"),
    (0x021A4544, "726135365ff195af"),
    (0x021A4598, "50f606e315410521"),
    (0x021A4664, "5eb4cc092b179fb4"),
    (0x021A4698, "3659e70bd4bc2c19"),
    (0x021A46F0, "2c661643fb126074"),
    (0x021A4768, "0e2451e0dad9f723"),
    (0x021A47B4, "e910eedba3f4c662"),
    (0x021A482C, "ba3f9c88914a884c"),
    (0x021A4890, "3a563f94b3758f5f"),
    (0x021A48A4, "9739211beeee3303"),
    (0x021A48C8, "4d56485a10061b78"),
    (0x021A491C, "c6f482eb8036d163"),
    (0x021A4960, "a8cee0eee822f60c"),
    (0x021A4994, "a4cd271c269a95c8"),
    (0x021A4A74, "d12415af14a22d25"),
    (0x021A4AA8, "02c3e0790e7964c3"),
    (0x021A4B00, "0e86e541e3c3e607"),
    (0x021A4B54, "711a20cb0ebdc84e"),
    (0x021A4B98, "0d516090cbf76b8e"),
    (0x021A4C1C, "023c0a60b2de6353"),
    (0x021A4C50, "af7c6b50461b0054"),
    (0x021A4CA8, "f0192a3b6c98b12f"),
    (0x021A4CF4, "bd3fba1902cb522a"),
    (0x021A4D8C, "0ce1245638f6f26f"),
    (0x021A4D90, "05365ed170fc1a15"),
    (0x021A4DC0, "738705aa53607b4b"),
    (0x021A4E78, "c5ff65bb4377bbe2"),
    (0x021A4E9C, "71d94812772cfb2c"),
    (0x021A4ED8, "2200b0652b2196f8"),
    (0x021A4F04, "c4827b913b9e853f"),
    (0x021A4F0C, "5589c01d4ac5c439"),
    (0x021A4F50, "4a056dbe5be55347"),
    (0x021A4F58, "52b7fed24daec62a"),
    (0x021A4FA8, "13f366dbe3b577f9"),
    (0x021A5010, "299aef140e4f78b5"),
    (0x021A5014, "5625e48aa0aa9977"),
    (0x021A5044, "be10d9455c724669"),
    (0x021A50BC, "ba68514a2cab261a"),
    (0x021A50E0, "a166edfc533dfcbd"),
    (0x021A511C, "26ba9da1360565b8"),
    (0x021A5148, "def73b2c3aba07be"),
    (0x021A51A4, "7c1bb7b22d69f739"),
    (0x021A51A8, "f65fe3c1ad5ceb42"),
    (0x021A51D8, "9358af4b733c7728"),
    (0x021A5204, "0539814da42ba40d"),
    (0x021A5228, "9f151154b130c394"),
    (0x021A5264, "924530dd1dfbf2f6"),
    (0x021A5304, "5178a9ea306cc7e6"),
    (0x021A530C, "53f85f95155c0c9a"),
    (0x021A5318, "1c2a9a6f8ff3d57f"),
    (0x021A5328, "49e1d973c74c8b84"),
    (0x021A5334, "3ccb71731c64f6a2"),
    (0x021A5340, "45f9548f0eb669ca"),
    (0x021A5354, "afac01eead4f8b38"),
    (0x021A5360, "0a909d0118f260b6"),
    (0x021A536C, "008c272bdbb6cd0b"),
    (0x021A5378, "e272299a02b617dd"),
    (0x021A5390, "cd05faea4e67f8e8"),
    (0x021A53A4, "48b889b80d2b3ee5"),
    (0x021A53BC, "7eba74ff18acb282"),
    (0x021A53D0, "b3490da8db4be65f"),
    (0x021A5408, "7e970b65325c167f"),
    (0x021A5458, "1ef3babe7a171779"),
    (0x021A548C, "48c877b3db02728c"),
    (0x021A54A0, "419de3bf642f831e"),
    (0x021A54B4, "67c0dc61e63961d1"),
    (0x021A550C, "2ca75e2acdfca370"),
    (0x021A552C, "0f7075e8ca762c91"),
    (0x021A5538, "75892fb2ae9d6dfb"),
    (0x021A55A8, "5a31a5573f4d0d3d"),
    (0x021A55DC, "d7169a5ea1b38490"),
    (0x021A55FC, "59b54556acf1f5f1"),
    (0x021A5610, "0cb3b7d0efd9be4e"),
    (0x021A5630, "514f7723baa83557"),
    (0x021A569C, "e372df2725d8529e"),
    (0x021A56C0, "32b1ef3c367e6ce7"),
    (0x021A56F4, "5d0b57413257cac4"),
    (0x021A5708, "d5f5d068abcad0f4"),
    (0x021A5728, "db4d215c7a91c5a0"),
    (0x021A572C, "373a2bdad872cd1a"),
    (0x021A5738, "c47b50d87fe8ee77"),
    (0x021A5764, "c5d669ef866c890c"),
    (0x021A5768, "68f72573ed797dcc"),
    (0x021A5778, "fb7ae193d4545242"),
    (0x021A5788, "63ec6852b961ad9b"),
    (0x021A57C0, "a26391e22fa9af3e"),
    (0x021A5810, "c97542b7f0552a46"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_grid.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_4140 => create(e, v[0], v[1]) as u64,
        0x021A_41B0 => {
            frame(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021A_4224 => {
            apply_value(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4254 => can_act(e, v[0], v[1]) as u64,
        0x021A_428C => footstep_se(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021A_43D8 => {
            step_se(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4444 => move_value(e, v[0], v[1]) as u64,
        0x021A_448C => dir_kind(e, v[0], v[1]) as u64,
        0x021A_44BC => acmd_ok(e, v[0]) as u64,
        0x021A_44C8 => walk_value(e, v[0], v[1]) as u64,
        0x021A_4544 => walk_stop(e, v[0], v[1], v[2]) as u64,
        0x021A_4598 => walk_step(e, v[0], v[1], v[2]) as u64,
        0x021A_4664 => walk_turning(e, v[0], v[1], v[2]) as u64,
        0x021A_4698 => walk_bumping(e, v[0], v[1], v[2]) as u64,
        0x021A_46F0 => ice_value(e, v[0], v[1]) as u64,
        0x021A_4768 => ice_stop(e, v[0], v[1], v[2]) as u64,
        0x021A_47B4 => ice_step(e, v[0], v[1], v[2]) as u64,
        0x021A_482C => ice_spin(e, v[0], v[1], v[2]) as u64,
        0x021A_4890 => ice_wait(e, v[0], v[1], v[2]) as u64,
        0x021A_48A4 => ice_fall(e, v[0], v[1], v[2]) as u64,
        0x021A_48C8 => cycle_value(e, v[0], v[1]) as u64,
        0x021A_491C => cycle_stop(e, v[0], v[1], v[2]) as u64,
        0x021A_4960 => cycle_ok_attr(e, v[0]) as u64,
        0x021A_4994 => cycle_step(e, v[0], v[1], v[2]) as u64,
        0x021A_4A74 => cycle_turning(e, v[0], v[1], v[2]) as u64,
        0x021A_4AA8 => cycle_bumping(e, v[0], v[1], v[2]) as u64,
        0x021A_4B00 => surf_value(e, v[0], v[1]) as u64,
        0x021A_4B54 => surf_stop(e, v[0], v[1], v[2]) as u64,
        0x021A_4B98 => surf_step(e, v[0], v[1], v[2]) as u64,
        0x021A_4C1C => surf_turning(e, v[0], v[1], v[2]) as u64,
        0x021A_4C50 => surf_bumping(e, v[0], v[1], v[2]) as u64,
        0x021A_4CA8 => {
            act(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4CF4 => {
            walk_act(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4D8C => {
            act_none(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4D90 => {
            act_stand(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4DC0 => {
            act_walk(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4E78 => {
            act_turn(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4E9C => {
            act_bump(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4ED8 => {
            act_jump(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4F04 => {
            act_state_4(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4F0C => {
            act_slide(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4F50 => {
            act_state_6(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4F58 => {
            act_fall(e, v[0], v[1], v[2]);
            0
        }
        0x021A_4FA8 => {
            cycle_act(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5010 => {
            cycle_none(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5014 => {
            cycle_stand(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5044 => {
            cycle_ride(e, v[0], v[1], v[2]);
            0
        }
        0x021A_50BC => {
            cycle_turn(e, v[0], v[1], v[2]);
            0
        }
        0x021A_50E0 => {
            cycle_bump(e, v[0], v[1], v[2]);
            0
        }
        0x021A_511C => {
            cycle_jump(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5148 => {
            surf_act(e, v[0], v[1], v[2]);
            0
        }
        0x021A_51A4 => {
            surf_none(e, v[0], v[1], v[2]);
            0
        }
        0x021A_51A8 => {
            surf_stand(e, v[0], v[1], v[2]);
            0
        }
        0x021A_51D8 => {
            surf_swim(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5204 => {
            surf_turn(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5228 => {
            surf_bump(e, v[0], v[1], v[2]);
            0
        }
        0x021A_5264 => hit_check(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_5304 => {
            set_flags(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_530C => {
            clear_flags(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_5318 => has_flags(&e.mem, v[0], v[1]) as u64,
        0x021A_5328 => {
            set_flag_1(e, v[0]);
            0
        }
        0x021A_5334 => {
            clear_flag_1(e, v[0]);
            0
        }
        0x021A_5340 => flag_1(e, v[0]) as u64,
        0x021A_5354 => {
            set_flag_2(e, v[0]);
            0
        }
        0x021A_5360 => {
            clear_flag_2(e, v[0]);
            0
        }
        0x021A_536C => {
            clear_flag_4(e, v[0]);
            0
        }
        0x021A_5378 => {
            set_spinning(e, v[0], v[1]);
            0
        }
        0x021A_5390 => spinning(e, v[0]) as u64,
        0x021A_53A4 => {
            set_sliding(e, v[0], v[1]);
            0
        }
        0x021A_53BC => sliding(e, v[0]) as u64,
        0x021A_53D0 => {
            end_turn(e, v[0]);
            0
        }
        0x021A_5408 => stop(e, v[0]) as u64,
        0x021A_5458 => on_forced_tile(e, v[0]) as u64,
        0x021A_548C => forced_tile(e, v[0]) as u64,
        0x021A_54A0 => is_sliding(e, v[0]) as u64,
        0x021A_54B4 => {
            stop_sliding(e, v[0]);
            0
        }
        0x021A_550C => {
            end_forced(e, v[0]);
            0
        }
        0x021A_552C => forced_none(e, v[0], v[1]) as u64,
        0x021A_5538 => forced_step(e, v[0], v[1]) as u64,
        0x021A_55A8 => forced_move(e, v[0], v[1], v[2]) as u64,
        0x021A_55DC => is_ice(e, v[0]) as u64,
        0x021A_55FC => is_ice_404(e, v[0]) as u64,
        0x021A_5610 => on_ice(e, v[0]) as u64,
        0x021A_5630 => free_dir(e, v[0]) as u64,
        0x021A_569C => hit_check_ice(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_56C0 => grounded(e, v[0]) as u64,
        0x021A_56F4 => {
            spin_start(e, v[0]);
            0
        }
        0x021A_5708 => {
            spin_end(e, v[0]);
            0
        }
        0x021A_5728 => core(e, v[0]) as u64,
        0x021A_572C => mmdl(e, v[0]) as u64,
        0x021A_5738 => {
            spin_reset(e, v[0], v[1]);
            0
        }
        0x021A_5764 => get_1c(e, v[0]) as u64,
        0x021A_5768 => {
            set_1c(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_5778 => state_is_3(&e.mem, v[0]) as u64,
        0x021A_5788 => attr_ahead(e, v[0], v[1], v[2]) as u64,
        0x021A_57C0 => slope(e, v[0], v[1]) as u64,
        0x021A_5810 => warp_ahead(e, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_grid`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_grid", SPEC, PORTED, RECORDED), &call);
}
