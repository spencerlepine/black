use super::*;
use crate::testenv::{ofs_work, run};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02187F00, "3008a7a98b567285"),
    (0x02187FD0, "28e815f0267c3d0f"),
    (0x02187FE8, "adef1e75405af47f"),
    (0x02188030, "6fb18ab4e7a633cd"),
    (0x021880A4, "99ba30f3d4d6710c"),
    (0x021880C8, "bf0dca5aaf8dd995"),
    (0x021881D8, "a84d0aa53929f0e9"),
    (0x02188758, "6082a08fb423fccb"),
    (0x02188910, "c1d81a96cd162340"),
    (0x02188990, "a8efaaced0b70c7c"),
    (0x02188B58, "648c9ee5add3f21c"),
    (0x02188BD8, "4fd251deeab682b6"),
    (0x02188BE8, "2ea3f879aa3e438b"),
    (0x02188BF8, "54f64bb4f416cd2b"),
    (0x02188C54, "45ec3959fd3f5b8e"),
    (0x02188C58, "b0a30cbc069e02ae"),
    (0x02188C5C, "4620870f900cdddc"),
    (0x02188C60, "6b07e956e05dee96"),
    (0x02188C64, "a7c33861a904b038"),
    (0x02188C68, "7332a2c6cc00be53"),
    (0x02188C6C, "14b018ca5280a47a"),
    (0x02188C70, "05954030d671da42"),
    (0x02188C7C, "f96ffb33f6673627"),
    (0x02188C80, "c1a9a7dfe9eea848"),
    (0x02188C84, "cc4d1d679032de1e"),
    (0x02188C88, "3fac5466b8809055"),
    (0x02188C90, "3ce2ed2bb96bd0f6"),
    (0x02188C98, "1b46e60f4e769760"),
    (0x02188C9C, "fde2104b875563f9"),
    (0x02188CA4, "b3af2e1e3c606cb4"),
    (0x02188CAC, "6ec65979360bb4c4"),
    (0x02188CB4, "a20dae2d438673e2"),
    (0x02188CBC, "d9925d760fa39784"),
    (0x02188CCC, "0adaf9843143c983"),
    (0x02188CD0, "d6c172a87a873cda"),
    (0x02188CD8, "8bedba71a8b461bd"),
    (0x02188CE0, "487e9b04820b6f56"),
    (0x02188CE8, "458fc15bf8feed13"),
    (0x02188CF0, "1aacfdfb4b99c953"),
    (0x02188CFC, "c1f7a3f130391a6d"),
    (0x02188D20, "d6d2a9c244b2dee4"),
    (0x02188D24, "8cdbdbc7e3f58c80"),
    (0x02188D2C, "e82ae95648ec2cf8"),
    (0x02188D30, "b4bd76e18c9d2e4f"),
    (0x02188D38, "ea945221cf1e8813"),
    (0x02188D40, "591fea4f37ee05b4"),
    (0x02188D48, "27e1b9ec363f8d7c"),
    (0x02188D4C, "b113b2aae45be125"),
    (0x02188D50, "9a2e86d0cb445363"),
    (0x02188D58, "32a0203927a4156a"),
    (0x02188E30, "b6fed20763e4d9f6"),
    (0x02189004, "29977198241a2fc2"),
    (0x02189204, "52f6f5067883f16c"),
    (0x021892B4, "fe946647bb5967d5"),
    (0x021892CC, "20b4a8602dca4f1f"),
    (0x0218932C, "3bdd9bf3b075b123"),
    (0x02189370, "5c00469c627ffc59"),
    (0x021893C0, "6e0167401d4ab89b"),
    (0x021894E0, "52977cfd5f221e23"),
    (0x02189518, "72cb0b13374542a9"),
    (0x02189560, "7117671744463c4e"),
    (0x0218958C, "ec06403c37f6d682"),
    (0x021895BC, "57ca7bcc985a81f1"),
    (0x021895D0, "f11f65158cba4c68"),
    (0x021895DC, "bb9114842b5999ad"),
    (0x021895E8, "86e23bfc4b3e0cc7"),
    (0x02189614, "d367e5f9d0c7227c"),
    (0x02189684, "da6010d2177b26ad"),
    (0x021896F4, "d4c345bae3a3ca66"),
    (0x02189734, "23cba83c8db61486"),
    (0x02189758, "5c3b66b0cbe9265e"),
    (0x02189764, "e93ecb6f8a649f14"),
    (0x021897A0, "7630e3485cceb0c4"),
    (0x021897BC, "0d0ccc6509adbada"),
    (0x021899F0, "7bce2cd74e5974a7"),
    (0x021899F8, "af88800a5e39e99b"),
    (0x02189A00, "2029c930b1b47452"),
    (0x02189A08, "1dd43fa8db69cb78"),
    (0x02189A10, "066ce926326ce82b"),
    (0x02189A18, "cd63741c51721bed"),
    (0x02189A1C, "61e0250a8c142ee6"),
    (0x02189A24, "219ca45491677b04"),
    (0x02189A2C, "4e34f65bb7f0b831"),
    (0x02189A38, "405d2f2e5733de44"),
    (0x02189A48, "b8d8170f6dd00ecc"),
    (0x02189A50, "7e5a3e8fa2d4c198"),
    (0x02189A58, "bdba32792d93bc43"),
    (0x02189A60, "ebe9303a46f7a76b"),
    (0x02189A68, "50c029ed82b71f58"),
    (0x02189A70, "89d36b5678d45a6d"),
    (0x02189A78, "7246a862e67b4786"),
    (0x02189A80, "09db3c1b0f079e29"),
    (0x02189ACC, "2f7b840690e04a7c"),
    (0x02189AE8, "cbc86f8161ba2383"),
    (0x02189B04, "8ff8981a934eb2dd"),
    (0x02189B30, "4fbad7a66b52d2bd"),
    (0x02189BA8, "f1e3f874db9d5ef8"),
    (0x02189BBC, "431ac89073013430"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/fieldmap.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0218_7F00 => proc_init(e, v[0], v[1], v[2]) as u64,
        0x0218_7FD0 => proc_main(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_7FE8 => proc_end(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_8030 => ofs_work(create(e, v[0], v[1])),
        0x0218_80A4 => {
            destroy(e, v[0]);
            0
        }
        0x0218_80C8 => proc_graphics_init(e, v[0], v[1]) as u64,
        0x0218_81D8 => proc_setup(e, v[0], v[1]) as u64,
        0x0218_85E8 => proc_start(e, v[0], v[1]) as u64,
        0x0218_8758 => proc_frame(e, v[0], v[1]) as u64,
        0x0218_88C8 => {
            draw_step(e, v[0]);
            0
        }
        0x0218_8910 => proc_draw(e, v[0], v[1]) as u64,
        0x0218_8990 => proc_teardown(e, v[0], v[1]) as u64,
        0x0218_8AD4 => proc_graphics_exit(e, v[0], v[1]) as u64,
        0x0218_8B58 => proc_step(e, v[0], v[1]) as u64,
        0x0218_9004 => {
            frame_map_work(e, v[0]);
            0
        }
        0x0218_9204 => {
            mmdl_setup(e, v[0]);
            0
        }
        0x0218_8BD8 => {
            request_end(&mut e.mem, v[0]);
            0
        }
        0x0218_8BE8 => is_running(&e.mem, v[0]) as u64,
        0x0218_8BF8 => end_player_state(e, v[0]) as u64,
        0x0218_8C54 => get_28(&e.mem, v[0]) as u64,
        0x0218_8C58 => get_10(&e.mem, v[0]) as u64,
        0x0218_8C5C => get_3c(&e.mem, v[0]) as u64,
        0x0218_8C60 => get_14(&e.mem, v[0]) as u64,
        0x0218_8C64 => get_18(&e.mem, v[0]) as u64,
        0x0218_8C68 => get_1c(&e.mem, v[0]) as u64,
        0x0218_8C6C => get_20(&e.mem, v[0]) as u64,
        0x0218_8C70 => gamesys_0202c794(e, v[0]) as u64,
        0x0218_8C7C => mmdl_sys(&e.mem, v[0]) as u64,
        0x0218_8C80 => gamesys(&e.mem, v[0]) as u64,
        0x0218_8C84 => heap(&e.mem, v[0]) as u64,
        0x0218_8C88 => get_b4(&e.mem, v[0]) as u64,
        0x0218_8C90 => get_b8(&e.mem, v[0]) as u64,
        0x0218_8C98 => get_48(&e.mem, v[0]) as u64,
        0x0218_8C9C => zone(&e.mem, v[0]) as u64,
        0x0218_8CA4 => ctrl_work(&e.mem, v[0]) as u64,
        0x0218_8CAC => {
            set_ctrl_work(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_8CB4 => player(&e.mem, v[0]) as u64,
        0x0218_8CBC => has_player(&e.mem, v[0]) as u64,
        0x0218_8CCC => get_24(&e.mem, v[0]) as u64,
        0x0218_8CD0 => get_94(&e.mem, v[0]) as u64,
        0x0218_8CD8 => get_130(&e.mem, v[0]) as u64,
        0x0218_8CE0 => get_98(&e.mem, v[0]) as u64,
        0x0218_8CE8 => encount(&e.mem, v[0]) as u64,
        0x0218_8CF0 => ctrl_kind(&e.mem, v[0]) as u64,
        0x0218_8CFC => map_kind(e, v[0]) as u64,
        0x0218_8D20 => get_30(&e.mem, v[0]) as u64,
        0x0218_8D24 => func_sys(&e.mem, v[0]) as u64,
        0x0218_8D2C => get_34(&e.mem, v[0]) as u64,
        0x0218_8D30 => tasks(&e.mem, v[0]) as u64,
        0x0218_8D38 => get_134(&e.mem, v[0]) as u64,
        0x0218_8D40 => {
            set_position_src(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_8D48 => get_2c(&e.mem, v[0]) as u64,
        0x0218_8D4C => {
            set_2c(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_8D50 => get_128(&e.mem, v[0]) as u64,
        0x0218_8D58 => area(&e.mem, v[0]) as u64,
        0x0218_8E30 => get_14c(&e.mem, v[0]) as u64,
        0x0218_92B4 => {
            mmdl_delete(e, v[0]);
            0
        }
        0x0218_92CC => zone_check(e, v[0]) as u64,
        0x0218_932C => player_moved(e, v[0]) as u64,
        0x0218_9370 => zone_changed(e, v[0]) as u64,
        0x0218_93C0 => {
            zone_change(e, v[0]);
            0
        }
        0x0218_94E0 => {
            zone_change_mmdl(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0218_9518 => {
            zone_change_save(e, v[0], v[1]);
            0
        }
        0x0218_9560 => {
            zone_change_20(e, v[0], v[1]);
            0
        }
        0x0218_958C => {
            zone_change_1c(e, v[0], v[1]);
            0
        }
        0x0218_95BC => {
            zone_change_record(e, v[0], v[1]);
            0
        }
        0x0218_95D0 => {
            zone_change_10(e, v[0], v[1]);
            0
        }
        0x0218_95DC => {
            clear_ctrl_work(&mut e.mem, v[0]);
            0
        }
        0x0218_95E8 => {
            zone_change_10_at(e, v[0], v[1], v[2]);
            0
        }
        0x0218_9614 => {
            zone_light(e, v[0], v[1]);
            0
        }
        0x0218_9684 => {
            zone_change_scene(e, v[0], v[1]);
            0
        }
        0x0218_96F4 => comm_mode(e, v[0]) as u64,
        0x0218_9734 => comm_idle(e, v[0]) as u64,
        0x0218_9758 => {
            slot_init(&mut e.mem, v[0]);
            0
        }
        0x0218_9764 => slot_alloc(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_97A0 => {
            slot_free(e, v[0], v[1]);
            0
        }
        0x0218_97BC => slot_block(&e.mem, v[0], v[1]) as u64,
        0x0218_99F0 => draw_mode(&e.mem, v[0]) as u64,
        0x0218_99F8 => {
            set_draw_mode(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_9A00 => get_124(&e.mem, v[0]) as u64,
        0x0218_9A08 => get_13c(&e.mem, v[0]) as u64,
        0x0218_9A10 => get_a0(&e.mem, v[0]) as u64,
        0x0218_9A18 => get_40(&e.mem, v[0]) as u64,
        0x0218_9A1C => {
            set_138(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_9A24 => get_138(&e.mem, v[0]) as u64,
        0x0218_9A2C => gamesys_020120d8(e, v[0]) as u64,
        0x0218_9A38 => gamedata_020113d8(e, v[0]) as u64,
        0x0218_9A48 => get_144(&e.mem, v[0]) as u64,
        0x0218_9A50 => {
            set_144(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_9A58 => {
            set_140(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_9A60 => ofs_work(addr_148(v[0])),
        0x0218_9A68 => {
            set_150(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_9A70 => get_150(&e.mem, v[0]) as u64,
        0x0218_9A78 => get_100(&e.mem, v[0]) as u64,
        0x0218_9A80 => camera_distance(e, v[0]) as u64,
        0x0218_9ACC => zone_494(e, v[0]) as u64,
        0x0218_9AE8 => zone_not_3f(e, v[0]) as u64,
        0x0218_9B04 => is_hold_zone(v[0]) as u64,
        0x0218_9B30 => {
            map_setup(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0218_9BA8 => map_ctrl_of(e, v[0]) as u64,
        0x0218_9BBC => heap_size_of(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field fieldmap`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("fieldmap", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn hold_zones() {
    for z in [0x3F, 0x8C, 0x8D, 0x8E] {
        assert!(is_hold_zone(z));
    }
    for z in [0, 0x3E, 0x40, 0x8B, 0x8F, 0x1_003F] {
        assert!(!is_hold_zone(z));
    }
}

#[test]
fn cos_table_edges() {
    assert_eq!(fx_cos(0), 4096);
    assert_eq!(fx_cos(1024), 0);
    assert_eq!(fx_cos(2048), -4096);
    assert_eq!(fx_cos(1), 4096);
}
