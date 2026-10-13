use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A2D88, "f36bce98ad36bed8"),
    (0x021A2DE4, "0a2318b1a268d21c"),
    (0x021A2DF0, "8d20511540315d5f"),
    (0x021A2E34, "eee13fbdbdcdb5a9"),
    (0x021A2E44, "7023bf5c13b46420"),
    (0x021A2E50, "1014cf741696fa50"),
    (0x021A2E90, "7cf354b7bd7316ff"),
    (0x021A2EB8, "bd6589ccddbe3e87"),
    (0x021A2F20, "89f14c33c3df25a1"),
    (0x021A2F48, "d4eb9c986b578c0f"),
    (0x021A2F54, "0a4c6121c10a9791"),
    (0x021A2F60, "bb5c5200986dcaa8"),
    (0x021A2F6C, "76913568801d5dd4"),
    (0x021A2F8C, "97f26bdb8f623263"),
    (0x021A2F98, "1dbec06488b40018"),
    (0x021A2FA4, "e6dee5e536b96800"),
    (0x021A2FB0, "2c12a0b5280d0bfd"),
    (0x021A2FD0, "7f76606b0831e2cc"),
    (0x021A2FE4, "5d7b0e7ceefa5741"),
    (0x021A3038, "83b8da74ace86f2d"),
    (0x021A3044, "7807f159069cad0e"),
    (0x021A3050, "4bb7056d16dc97a9"),
    (0x021A305C, "9ef58cb83d9e5cca"),
    (0x021A3068, "2c8355b5bff87690"),
    (0x021A3074, "0770e11828efdc32"),
    (0x021A3080, "027d1373133c3656"),
    (0x021A308C, "a4797dc7045585c6"),
    (0x021A3098, "60265d4f079478ff"),
    (0x021A30A4, "146d3b1517317b58"),
    (0x021A30B0, "02d003cfaa15af2b"),
    (0x021A30BC, "6547be628686d049"),
    (0x021A30C8, "0d8f43338d445744"),
    (0x021A3128, "61aff12a9c0de315"),
    (0x021A3190, "5a417f2cf8abdbd6"),
    (0x021A31A4, "c7e81effe976cd55"),
    (0x021A31D0, "dd5b97749a5b3fb8"),
    (0x021A3208, "170d47911ea96003"),
    (0x021A3220, "2d84a4b798d4ff85"),
    (0x021A322C, "510f66cff508a043"),
    (0x021A3238, "cf818610d15a9e89"),
    (0x021A3244, "d1cc5d4594130daa"),
    (0x021A3250, "b98f20f6ea92aef6"),
    (0x021A32A8, "287e65b4e9af837a"),
    (0x021A330C, "ea0b0fbe47d85257"),
    (0x021A3364, "8fa071ac0f54a4f8"),
    (0x021A3460, "44ccbadec8dea82d"),
    (0x021A3484, "8e7a973069bbb480"),
    (0x021A34EC, "d8b896da4b637c1f"),
    (0x021A34F8, "a9c64edd1ca82738"),
    (0x021A3504, "73f18e93dae3867b"),
    (0x021A3524, "df0097baf669535c"),
    (0x021A3550, "6070b202a43deb3e"),
    (0x021A356C, "6cfe38556556391b"),
    (0x021A3584, "2d44d54fdd00166c"),
    (0x021A35A0, "9327c1f6b74299bd"),
    (0x021A35BC, "719c152050867cf2"),
    (0x021A35E0, "7f1bfb861037b836"),
    (0x021A35F0, "37cc405ff9e94376"),
    (0x021A35FC, "65788424bc91f482"),
    (0x021A3624, "58c68428878a0cb1"),
    (0x021A3644, "0f221bb8953d51a1"),
    (0x021A3664, "165dda27956491dd"),
    (0x021A3674, "8a600f0630baa3ad"),
    (0x021A3680, "eb2a2a57a5de32a4"),
    (0x021A368C, "51c694790f25561c"),
    (0x021A3698, "04ad2266bb35f3e4"),
    (0x021A36A4, "bd15ca1d16e81f07"),
    (0x021A36B0, "2c6b9d3383aa2ef0"),
    (0x021A36BC, "a1d61437d45c106b"),
    (0x021A36C8, "f1bed9c616db0055"),
    (0x021A36D4, "42f90d7bd84fe3a7"),
    (0x021A36E0, "f93cdd2b02d44263"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_2D88 => {
            ctrl_create(e, v[0], v[1], v[2]);
            0
        }
        0x021A_2DE4 => {
            ctrl_delete(e, v[0]);
            0
        }
        0x021A_2DF0 => {
            ctrl_frame(e, v[0]);
            0
        }
        0x021A_2E34 => ctrl_pos(e, v[0]) as u64,
        0x021A_2E44 => {
            ctrl_hold(e, v[0], v[1]);
            0
        }
        0x021A_2E50 => create(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021A_2E90 => {
            delete(e, v[0]);
            0
        }
        0x021A_2EB8 => {
            save(e, v[0]);
            0
        }
        0x021A_2F20 => step_events(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_2F48 => {
            set_requests(e, v[0], v[1]);
            0
        }
        0x021A_2F54 => requests_done(e, v[0]) as u64,
        0x021A_2F60 => {
            run_requests(e, v[0]);
            0
        }
        0x021A_2F6C => {
            end_turn(e, v[0]);
            0
        }
        0x021A_2F8C => {
            set_surf_task(e, v[0], v[1]);
            0
        }
        0x021A_2F98 => surf_task(e, v[0]) as u64,
        0x021A_2FA4 => key_dir(e, v[0], v[1]) as u64,
        0x021A_2FB0 => state_check(e, v[0]) as u64,
        0x021A_2FD0 => {
            request_walk(e, v[0]);
            0
        }
        0x021A_2FE4 => {
            update_move_state(e, v[0]);
            0
        }
        0x021A_3038 => {
            mmdl_pos(e, v[0], v[1]);
            0
        }
        0x021A_3044 => {
            warp_to(e, v[0], v[1]);
            0
        }
        0x021A_3050 => dir(e, v[0]) as u64,
        0x021A_305C => {
            set_dir(e, v[0], v[1]);
            0
        }
        0x021A_3068 => fieldmap(e, v[0]) as u64,
        0x021A_3074 => mmdl(e, v[0]) as u64,
        0x021A_3080 => move_value(e, v[0]) as u64,
        0x021A_308C => move_state(e, v[0]) as u64,
        0x021A_3098 => form(e, v[0]) as u64,
        0x021A_30A4 => {
            set_form(e, v[0], v[1]);
            0
        }
        0x021A_30B0 => sex(e, v[0]) as u64,
        0x021A_30BC => gamesys(e, v[0]) as u64,
        0x021A_30C8 => attr_under(e, v[0], 0) as u64,
        0x021A_3128 => attr_front(e, v[0], v[1], 0) as u64,
        0x021A_3190 => entry_code(v[0], v[1]) as u64,
        0x021A_31A4 => form_code(v[0], v[1]) as u64,
        0x021A_31D0 => code_entry(v[0]) as u64,
        0x021A_3208 => not_surfing(e, v[0]) as u64,
        0x021A_3220 => {
            change_code(e, v[0], v[1]);
            0
        }
        0x021A_322C => {
            clear_saved_code(e, v[0]);
            0
        }
        0x021A_3238 => code_has_form(e, v[0]) as u64,
        0x021A_3244 => mmdl_busy(e, v[0]) as u64,
        0x021A_3250 => mmdl_in_front(e, v[0]) as u64,
        0x021A_32A8 => mmdl_in_front_at(e, v[0], v[1]) as u64,
        0x021A_330C => {
            tile_in_dir(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021A_3364 => {
            pos_in_dir(e, v[0], v[1], v[2]);
            0
        }
        0x021A_3460 => {
            front_tile(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021A_3484 => {
            dir_vector(e, v[0], v[1], v[2]);
            0
        }
        0x021A_34EC => code_form(e, v[0]) as u64,
        0x021A_34F8 => param_counter(e, v[0]) as u64,
        0x021A_3504 => attr_check(e, v[0], v[1], v[2]) as u64,
        0x021A_3524 => surf_attr_check(e, v[0], v[1], v[2]) as u64,
        0x021A_3550 => {
            grid_5738(e, v[0], v[1]);
            0
        }
        0x021A_356C => {
            grid_550c(e, v[0]);
            0
        }
        0x021A_3584 => one_shot(e, v[0]) as u64,
        0x021A_35A0 => {
            set_one_shot(e, v[0], v[1]);
            0
        }
        0x021A_35BC => running_shoes(e, v[0]) as u64,
        0x021A_35E0 => {
            create_grid(e, v[0], v[1]);
            0
        }
        0x021A_35F0 => {
            grid_frame(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021A_35FC => {
            grid_pos(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021A_3624 => sliding(e, v[0]) as u64,
        0x021A_3644 => {
            stop_sliding(e, v[0]);
            0
        }
        0x021A_3664 => {
            create_nogrid(e, v[0]);
            0
        }
        0x021A_3674 => {
            nogrid_596c(e, v[0], v[1], v[2]);
            0
        }
        0x021A_3680 => nogrid_5928(e, v[0], v[1]) as u64,
        0x021A_368C => {
            nogrid_5964(e, v[0]);
            0
        }
        0x021A_3698 => nogrid_59ec(e, v[0]) as u64,
        0x021A_36A4 => nogrid_5a30(e, v[0]) as u64,
        0x021A_36B0 => nogrid_5a00(e, v[0], v[1]) as u64,
        0x021A_36BC => nogrid_5a0c(e, v[0], v[1]) as u64,
        0x021A_36C8 => nogrid_5a18(e, v[0], v[1], v[2]) as u64,
        0x021A_36D4 => nogrid_5a24(e, v[0], v[1]) as u64,
        0x021A_36E0 => grid_of(&e.mem, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn form_table_lookups() {
    assert_eq!(form_code(0, 2), 3);
    assert_eq!(form_code(1, 1), 5);
    assert_eq!(form_code(0, 7), 1);
    assert_eq!(code_entry(0x8A), 5);
    assert_eq!(code_entry(6), 2);
    assert_eq!(code_entry(0x77), 10);
    assert_eq!(entry_code(1, 9), 0x9D);
}
