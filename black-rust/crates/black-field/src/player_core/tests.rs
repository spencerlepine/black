use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A36E4, "a1f3ac34a6df862e"),
    (0x021A3818, "30e1f58c471f7ae2"),
    (0x021A3898, "e5810010d189d818"),
    (0x021A38A4, "af805dd2b3dbda7d"),
    (0x021A3920, "4ee2d3c561ea7558"),
    (0x021A392C, "76bc807b3d1413b4"),
    (0x021A3938, "eccfcd18c67ba01c"),
    (0x021A394C, "cad25108c8e25144"),
    (0x021A395C, "ebd744ccf09b79d1"),
    (0x021A3960, "c06b83e774ef4dd9"),
    (0x021A3964, "9959c539fda24585"),
    (0x021A3968, "4aee90e4791c81cc"),
    (0x021A396C, "0e1e2199d1020a6e"),
    (0x021A3970, "522ba69ee7a86abf"),
    (0x021A3974, "3a2b568d36070dae"),
    (0x021A3980, "af93aeaa2063743c"),
    (0x021A398C, "3e4e486dc7512fdb"),
    (0x021A3990, "5c0fd2a222da3a7d"),
    (0x021A3994, "c46ba6606a975e9e"),
    (0x021A3998, "b657173625b4d8be"),
    (0x021A39AC, "0a89cde136a70625"),
    (0x021A39E4, "a2b9a7621782bb58"),
    (0x021A3A10, "481e0dc1eae5b4fc"),
    (0x021A3A20, "90e9c7a399a8c37d"),
    (0x021A3A3C, "af9fb1ec4132e076"),
    (0x021A3AA0, "d901cfbbc41d8815"),
    (0x021A3AB8, "6064c03043e040a7"),
    (0x021A3AC4, "7a30d275cf2df949"),
    (0x021A3AD0, "078d5561c0eae03b"),
    (0x021A3AD4, "af461574aaa490ad"),
    (0x021A3AD8, "8a579aa50a9ebee8"),
    (0x021A3B3C, "ec1d1ba51fb338af"),
    (0x021A3B78, "dd98c4f3ec2c880c"),
    (0x021A3B94, "ab8a033bda5bfa15"),
    (0x021A3BB4, "a3aed5ecfeaa6748"),
    (0x021A3C24, "2b8a55792ed82023"),
    (0x021A3C3C, "b1666604ca238329"),
    (0x021A3C54, "8483be8f63ca7b5a"),
    (0x021A3C6C, "966f6a89f3218bb3"),
    (0x021A3C94, "32fc474f22c818fb"),
    (0x021A3CB0, "58ef84c83e497712"),
    (0x021A3CC0, "e6a2d948161dbd4c"),
    (0x021A3CE8, "568ba958fed7e4a0"),
    (0x021A3D08, "e497140e7a19808c"),
    (0x021A3D50, "e951a9174c3e1dd8"),
    (0x021A3D6C, "e8093f2e6c9a6f36"),
    (0x021A3DA0, "90eab7c09b69177d"),
    (0x021A3E14, "6ae4c6deb11b27ef"),
    (0x021A3E48, "790b54b02569eaa4"),
    (0x021A3E54, "bcfd6a97e4d95956"),
    (0x021A3EE0, "645b048f0f402473"),
    (0x021A3F10, "a5897bc9b1ed590e"),
    (0x021A3F48, "0892cc7411f2d790"),
    (0x021A3F64, "7775d4ba365da462"),
    (0x021A3F70, "c4f406ba474c912a"),
    (0x021A3F8C, "08aee78fae94cd4b"),
    (0x021A3F98, "cf4bad54c918f147"),
    (0x021A3FB4, "f0d4ed65f2f5448e"),
    (0x021A3FC0, "5ef908fe4381afc3"),
    (0x021A3FDC, "85027eebf9e5a3b9"),
    (0x021A3FE8, "b12939cf86fc6e6a"),
    (0x021A408C, "3b9543f40023d08b"),
    (0x021A40A8, "5316450c3e7b9b4b"),
    (0x021A4114, "c1d4c366e5917c61"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_core.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let r = match addr {
            0x021A_36E4 => create(e, v[0], v[1], v[2], v[3], v[4]),
            0x021A_3818 => {
                update_state(e, v[0]);
                0
            }
            0x021A_3898 => {
                get_pos(e, v[0], v[1]);
                0
            }
            0x021A_38A4 => {
                warp(e, v[0], v[1]);
                0
            }
            0x021A_3920 => disp_dir(e, v[0]),
            0x021A_392C => {
                set_disp_dir(e, v[0], v[1]);
                0
            }
            0x021A_3938 => {
                change_and_save_code(e, v[0], v[1]);
                0
            }
            0x021A_394C => {
                clear_saved_code(e, v[0]);
                0
            }
            0x021A_395C => fieldmap(&mut e.mem, v[0]),
            0x021A_3960 => mmdl(&mut e.mem, v[0]),
            0x021A_3964 => {
                set_move_value(&mut e.mem, v[0], v[1]);
                0
            }
            0x021A_3968 => move_value(&mut e.mem, v[0]),
            0x021A_396C => {
                set_move_state(&mut e.mem, v[0], v[1]);
                0
            }
            0x021A_3970 => move_state(&mut e.mem, v[0]),
            0x021A_3974 => form(e, v[0]),
            0x021A_3980 => {
                set_form(e, v[0], v[1]);
                0
            }
            0x021A_398C => sex(&mut e.mem, v[0]),
            0x021A_3990 => gamesys(&mut e.mem, v[0]),
            0x021A_3994 => record(&mut e.mem, v[0]),
            0x021A_3998 => {
                set_form_code(e, v[0]);
                0
            }
            0x021A_39AC => {
                set_entry_code(e, v[0], v[1]);
                0
            }
            0x021A_39E4 => param_wait(e, v[0]),
            0x021A_3A10 => code_entry(e, v[0]),
            0x021A_3A20 => has_form_code(e, v[0]),
            0x021A_3A3C => busy(e, v[0]),
            0x021A_3AA0 => acmd_settable(e, v[0]),
            0x021A_3AB8 => dir_of_keys16(e, v[0], v[1]),
            0x021A_3AC4 => {
                remember_keys16(e, v[0], v[1]);
                0
            }
            0x021A_3AD0 => {
                set_surf_task(&mut e.mem, v[0], v[1]);
                0
            }
            0x021A_3AD4 => surf_task(&mut e.mem, v[0]),
            0x021A_3AD8 => {
                change_code(e, v[0], v[1]);
                0
            }
            0x021A_3B3C => {
                set_form_and_code(e, v[0], v[1]);
                0
            }
            0x021A_3B78 => is_walk_code(e, v[0]),
            0x021A_3B94 => has_param(e, v[0]),
            0x021A_3BB4 => dir_of_keys(e, v[0], v[1]),
            0x021A_3C24 => {
                remember_keys(&mut e.mem, v[0], v[1]);
                0
            }
            0x021A_3C3C => key_dir_x(v[0]),
            0x021A_3C54 => key_dir_z(v[0]),
            0x021A_3C6C => run_request(e, v[0], v[1]),
            0x021A_3C94 => first_request(v[0]),
            0x021A_3CB0 => {
                set_requests(&mut e.mem, v[0], v[1]);
                0
            }
            0x021A_3CC0 => requests_done(e, v[0]),
            0x021A_3CE8 => {
                run_requests(e, v[0]);
                0
            }
            0x021A_3D08 => switch_code(e, v[0], v[1]),
            0x021A_3D50 => switch_entry(e, v[0], v[1]),
            0x021A_3D6C => param_entry(e, v[0], v[1]),
            0x021A_3DA0 => req_walk(e, v[0]),
            0x021A_3E14 => req_cycle(e, v[0]),
            0x021A_3E48 => req_cycle_wait(e, v[0]),
            0x021A_3E54 => req_surf(e, v[0]),
            0x021A_3EE0 => req_saved(e, v[0]),
            0x021A_3F10 => req_saved_wait(e, v[0]),
            0x021A_3F48 => req_entry4(e, v[0]),
            0x021A_3F64 => req_entry4_wait(e, v[0]),
            0x021A_3F70 => req_entry5(e, v[0]),
            0x021A_3F8C => req_entry5_wait(e, v[0]),
            0x021A_3F98 => req_entry6(e, v[0]),
            0x021A_3FB4 => req_entry6_wait(e, v[0]),
            0x021A_3FC0 => req_entry9(e, v[0]),
            0x021A_3FDC => req_entry9_wait(e, v[0]),
            0x021A_3FE8 => req_surf_variant(e, v[0]),
            0x021A_408C => req_entry8(e, v[0]),
            0x021A_40A8 => req_entry8_wait(e, v[0]),
            0x021A_4114 => {
                zone_update(e, v[0]);
                0
            }
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        };
        r as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_core`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_core", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn pad_directions() {
    assert_eq!(key_dir_x(0x30), 2);
    assert_eq!(key_dir_x(0x10), 3);
    assert_eq!(key_dir_z(0xC0), 0);
    assert_eq!(key_dir_z(0x80), 1);
    assert_eq!(key_dir_z(0x30), DIR_NONE);
    assert_eq!(first_request(0), 10);
    assert_eq!(first_request(0x400), 10);
    assert_eq!(first_request(0x104), 2);
}
