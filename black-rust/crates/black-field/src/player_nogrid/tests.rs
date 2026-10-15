use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A5880, "d633c003fe139f04"),
    (0x021A5920, "f5f09ba91d883cc8"),
    (0x021A5928, "1cafc5e8b232771c"),
    (0x021A5964, "ebec2f91ee78f57d"),
    (0x021A596C, "d1204ca9cfe6fc3a"),
    (0x021A59DC, "4188e3a771025c95"),
    (0x021A59EC, "55d150334179294c"),
    (0x021A5A00, "e616df9aa184cdf1"),
    (0x021A5A0C, "1a80c108f96294dd"),
    (0x021A5A18, "dbca7b94a14d6da9"),
    (0x021A5A24, "f8ddf9ab86b1887e"),
    (0x021A5A30, "b08f8e63ac509376"),
    (0x021A5A34, "3046f8cff3d10923"),
    (0x021A5A68, "4a13fa4e2a4892ce"),
    (0x021A5AD4, "97fc1413a7eff5c1"),
    (0x021A5B4C, "50c76611c6092d50"),
    (0x021A5B6C, "53f7b3cc3b2866c5"),
    (0x021A5B7C, "cbc7d9d09a868f11"),
    (0x021A5BC0, "f96c68a619cc2383"),
    (0x021A5C3C, "d429a636ef7033d4"),
    (0x021A5CA4, "9feac9584512530c"),
    (0x021A5D04, "b816d7584102fff6"),
    (0x021A5DD8, "569e0a973eda2348"),
    (0x021A5E28, "670b85e1897d22df"),
    (0x021A5EA0, "8e82f9b677de2534"),
    (0x021A5EA4, "f9a625d9600009a1"),
    (0x021A5EDC, "e0ed3c2591ec267b"),
    (0x021A5F14, "34bd6361b94e76ad"),
    (0x021A5F38, "58c2b1f5fcce30da"),
    (0x021A5F74, "917b3a4f81f62cff"),
    (0x021A5F98, "59d84344733f52d3"),
    (0x021A6014, "0d2a7763cf2d56cf"),
    (0x021A607C, "d95770f35b37c476"),
    (0x021A60DC, "81a58b32306c02a9"),
    (0x021A61B0, "e2f3c5683a36f0c5"),
    (0x021A6200, "8d7045922cc87a5d"),
    (0x021A6278, "a1288d2d83fa2adb"),
    (0x021A627C, "f887fc8e8d2a420a"),
    (0x021A62B4, "aa121af5d00a4d9d"),
    (0x021A62D8, "9117a06cb6fd83f6"),
    (0x021A62FC, "cb29d6fb66937a7b"),
    (0x021A6338, "6933cee879e74c32"),
    (0x021A635C, "fd593769a9de3b9f"),
    (0x021A63EC, "d9d969851d78a7dd"),
    (0x021A63F0, "32aa53a89c43e454"),
    (0x021A6428, "9f34a6e254c0a09e"),
    (0x021A6454, "a94af4abad0d607d"),
    (0x021A6478, "9e00bb3a9b64013e"),
    (0x021A64B4, "ba7234b334930ba9"),
    (0x021A652C, "c3f1e22d349bfe9f"),
    (0x021A658C, "72026358bf234922"),
    (0x021A661C, "9893a2625ba68881"),
    (0x021A666C, "4c1e2a3cd858044f"),
    (0x021A66E4, "78693c715f43aeea"),
    (0x021A66EC, "6db8bedb545d791d"),
    (0x021A66F8, "ab63d3359afbb899"),
    (0x021A6708, "d3b013f415c08ccd"),
    (0x021A6714, "a88dd97f388ce1c6"),
    (0x021A6720, "fc43bd707acf44ba"),
    (0x021A672C, "bfe6806fc37ecc6a"),
    (0x021A6738, "46506aa848efe368"),
    (0x021A6744, "d3db65938fc3cf9d"),
    (0x021A6750, "4d60dbe3585d227f"),
    (0x021A675C, "37e4e389971300bb"),
    (0x021A6768, "08a0f041d7922b2b"),
    (0x021A6774, "300d3140e7b1fb32"),
    (0x021A6780, "40797257dd001fb6"),
    (0x021A678C, "53120f031f71af2c"),
    (0x021A6798, "a12d606422ebd4c3"),
    (0x021A67A4, "63f15d9f50d3954e"),
    (0x021A682C, "973d005edb4ddd6c"),
    (0x021A6870, "e900a6325e7398c1"),
    (0x021A6908, "35f9ed45ea246a0e"),
    (0x021A6974, "8dc65179a3e3efbc"),
    (0x021A69D8, "0af55ea652d9e435"),
    (0x021A6A3C, "17c6f291f5f42a8c"),
    (0x021A6A64, "3d1c7a70d30ccd31"),
    (0x021A6A8C, "99c070b9abc9a90c"),
    (0x021A6AA0, "6830d041794fe99f"),
    (0x021A6B08, "e7f3d7f52e9d095a"),
    (0x021A6B30, "bd3e5c318823bd25"),
    (0x021A6B50, "9e15b71041e6947f"),
    (0x021A6B88, "50033e9efffa2eb7"),
    (0x021A6B9C, "a9d26d8780188142"),
    (0x021A6BAC, "445ea63ff19b38ca"),
    (0x021A6BB4, "708d76b22eefc04a"),
    (0x021A6C30, "1f8213c6aa7655c1"),
    (0x021A6C4C, "85fae754943df91a"),
    (0x021A6C64, "21b8e730f5c6da1b"),
    (0x021A6C6C, "51c3089f2928c036"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_nogrid.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        if let Some(t) = FLAG_THUNKS.iter().find(|t| t.0 == addr) {
            flag_thunk(e, t.0, v[0]);
            return 0;
        }
        match addr {
            0x021A_5880 => u64::from(create(e, v[0], v[1])),
            0x021A_5920 => {
                delete(e, v[0]);
                0
            }
            0x021A_5928 => {
                reset_rail(e, v[0], v[1]);
                0
            }
            0x021A_5964 => {
                clear_rail(e, v[0]);
                0
            }
            0x021A_596C => {
                frame(e, v[0], v[1], v[2]);
                0
            }
            0x021A_59DC => u64::from(is_bump(e, v[0])),
            0x021A_59EC => u64::from(is_sliding(e, v[0])),
            0x021A_5A00 => {
                set_loc(e, v[0], v[1]);
                0
            }
            0x021A_5A0C => {
                get_loc(e, v[0], v[1]);
                0
            }
            0x021A_5A18 => {
                front_loc(e, v[0], v[1], v[2]);
                0
            }
            0x021A_5A24 => {
                rail_pos(e, v[0], v[1]);
                0
            }
            0x021A_5A30 => u64::from(rail_work(e, v[0])),
            0x021A_5A34 => {
                end_bump(e, v[0]);
                0
            }
            0x021A_5A68 => u64::from(move_value(e, v[0], v[1], v[2], v[3], v[4])),
            0x021A_5AD4 => {
                action(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x021A_5B4C => {
                after_value(e, v[0], v[1], v[2]);
                0
            }
            0x021A_5B6C => u64::from(hit(e, v[1], v[2], v[3])),
            0x021A_5B7C => {
                eff_update(e, v[0]);
                0
            }
            0x021A_5BC0 => {
                walk_action(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x021A_5C3C => u64::from(walk_value(e, v[0], v[1], v[2], v[3], v[4])),
            0x021A_5CA4 => u64::from(walk_v_stand(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_5D04 => u64::from(v_walk(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_5DD8 => u64::from(walk_v_turn(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_5E28 => u64::from(walk_v_bump(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_5EA0 => {
                act_none();
                0
            }
            0x021A_5EA4 => {
                act_stand(e, v[0], v[1], v[4]);
                0
            }
            0x021A_5EDC => {
                act_walk(e, v[0], v[1], v[3], v[4], v[5]);
                0
            }
            0x021A_5F14 => {
                act_turn(e, v[0], v[1], v[4]);
                0
            }
            0x021A_5F38 => {
                act_bump(e, v[0], v[1], v[4]);
                0
            }
            0x021A_5F74 => {
                act_base(e, v[0], v[1], v[4], 0x38, 1);
                0
            }
            0x021A_5F98 => {
                cycle_action(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x021A_6014 => u64::from(cycle_value(e, v[0], v[1], v[2], v[3], v[4])),
            0x021A_607C => u64::from(cycle_v_stand(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_60DC => u64::from(cycle_v_walk(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_61B0 => u64::from(cycle_v_turn(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_6200 => u64::from(cycle_v_bump(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_6278 => {
                act_none();
                0
            }
            0x021A_627C => {
                act_stand(e, v[0], v[1], v[4]);
                0
            }
            0x021A_62B4 => {
                act_base(e, v[0], v[1], v[4], 0x14, 1);
                0
            }
            0x021A_62D8 => {
                act_turn(e, v[0], v[1], v[4]);
                0
            }
            0x021A_62FC => {
                act_bump(e, v[0], v[1], v[4]);
                0
            }
            0x021A_6338 => {
                act_base(e, v[0], v[1], v[4], 0x38, 1);
                0
            }
            0x021A_635C => {
                surf_action(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x021A_63EC => {
                act_none();
                0
            }
            0x021A_63F0 => {
                act_stand(e, v[0], v[1], v[4]);
                0
            }
            0x021A_6428 => {
                act_base(e, v[0], v[1], v[4], if v[5] == 1 { 0x14 } else { 0x10 }, 1);
                0
            }
            0x021A_6454 => {
                act_turn(e, v[0], v[1], v[4]);
                0
            }
            0x021A_6478 => {
                act_bump(e, v[0], v[1], v[4]);
                0
            }
            0x021A_64B4 => u64::from(surf_value(e, v[0], v[1], v[2], v[3], v[4])),
            0x021A_652C => u64::from(surf_v_stand(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_658C => u64::from(surf_v_walk(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_661C => u64::from(surf_v_turn(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_666C => u64::from(surf_v_bump(e, v[0], v[1], v[2], v[3], v[4], v[5])),
            0x021A_66E4 => {
                set_flags(e, v[0], v[1]);
                0
            }
            0x021A_66EC => {
                clear_flags(e, v[0], v[1]);
                0
            }
            0x021A_66F8 => u64::from(has_flags(e, v[0], v[1])),
            0x021A_67A4 => u64::from(slope_kind(e, v[0])),
            0x021A_682C => u64::from(slope(e, v[0], v[1], v[2])),
            0x021A_6870 => {
                slope_end(e, v[0]);
                0
            }
            0x021A_6908 => u64::from(slide_plain(e, v[0], v[1], v[2])),
            0x021A_6974 => u64::from(slide_a(e, v[0], v[1], v[2])),
            0x021A_69D8 => u64::from(slide_b(e, v[0], v[1], v[2])),
            0x021A_6A3C => u64::from(slide_se(e, v[0], v[1], v[2], 2)),
            0x021A_6A64 => u64::from(slide_se(e, v[0], v[1], v[2], 3)),
            0x021A_6A8C => u64::from(slide_bounce(e, v[0], v[1], v[2])),
            0x021A_6AA0 => u64::from(hold(e, v[0], v[1], v[2])),
            0x021A_6B08 => {
                eff_start(e, v[0], v[1]);
                0
            }
            0x021A_6B30 => {
                eff_stop(e, v[0]);
                0
            }
            0x021A_6B50 => {
                eff_frame(e, v[0], v[1]);
                0
            }
            0x021A_6B88 => {
                rec_init(e, v[0]);
                0
            }
            0x021A_6B9C => {
                rec_hold(e, v[0]);
                0
            }
            0x021A_6BAC => {
                rec_reset(e, v[0]);
                0
            }
            0x021A_6BB4 => {
                rec_input(e, v[0], v[1], v[2]);
                0
            }
            0x021A_6C30 => {
                rec_end(e, v[0]);
                0
            }
            0x021A_6C4C => u64::from(rec_dir(e, v[0], v[1])),
            0x021A_6C64 => u64::from(rec_bit1(e, v[0])),
            0x021A_6C6C => u64::from(bump_ok(e, v[0], v[1])),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_nogrid`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_nogrid", SPEC, PORTED, RECORDED), &call);
}

/// The embedded ROM table equals the overlay (`SPIN_DIRS`, ROM 0x021D8AB0).
#[test]
fn spin_dirs_match_overlay() {
    let Some(rd) = crate::testenv::overlay21() else {
        return;
    };
    for (k, row) in SPIN_DIRS.iter().enumerate() {
        let w = rd(0x021D_8AB0 + 4 * k as u32);
        assert_eq!(u32::from_le_bytes(*row), w, "SPIN_DIRS[{k}]");
    }
}
