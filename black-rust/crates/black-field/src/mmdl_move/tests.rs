use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mmdl_move.spec");

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02163010, "187b5875607dab70"),
    (0x02163028, "3fcee122d3f81880"),
    (0x02163090, "e469885aa711b70b"),
    (0x021630F4, "10babb8a124462ff"),
    (0x0216310C, "aaf55595df4bf934"),
    (0x02163130, "486f048724d25669"),
    (0x02163154, "0d876d772b63f6ce"),
    (0x02163188, "4c23dbf8235e0bcc"),
    (0x021631C0, "9ae78018945598ac"),
    (0x02163230, "78bb535bb4dd3504"),
    (0x02163288, "3a127456bd89f81a"),
    (0x021632F0, "ab6bb243634b3fb7"),
    (0x02163340, "14e4b64143689c8f"),
    (0x021633A8, "f68b32ef05c37127"),
    (0x02163418, "c3a6260e0de533de"),
    (0x02163464, "00aa010700aad3f2"),
    (0x021634D8, "744427a6a4f95a2f"),
    (0x021634FC, "45c7ed4d2d1fca0e"),
    (0x0216352C, "c7097c22a5697605"),
    (0x0216355C, "c2e4dcb01ec0e0c8"),
    (0x02163580, "9b7ec19d64742ffa"),
    (0x02163618, "de941dc0368acd57"),
    (0x02163650, "e9aeae32b7ae476a"),
    (0x0216369C, "be76ed55863f5ed6"),
    (0x021636A8, "faf6d5dc2219bd2b"),
    (0x021636C8, "5c6bd27804c811ed"),
    (0x021636F0, "7e2c7548e5af6d44"),
    (0x0216372C, "065745880a28d0a7"),
    (0x0216375C, "58aa1d365c176031"),
    (0x0216377C, "5f215c08680674e6"),
    (0x021637B0, "42932b311a109730"),
    (0x021637E4, "31beeadf30fe94e7"),
    (0x02163868, "159d45148c742fb5"),
    (0x021638AC, "5c74a986d91527d1"),
    (0x021638C8, "a725d65397f983fe"),
    (0x021638CC, "3ad8a27da1e75cd1"),
    (0x02163994, "6b62b007caa7af69"),
    (0x021639C0, "8bc05aaf180cd93c"),
    (0x02163A08, "9eb43eebaf8003d2"),
    (0x02163A40, "8d1f623a955e465f"),
    (0x02163AF0, "f52df17e7fcee6da"),
    (0x02163BAC, "b9377209d282bd38"),
    (0x02163C04, "c3e99bdfdf5b30d4"),
    (0x02163C9C, "8393a73cde471e46"),
    (0x02163D04, "559ae126d1b769b5"),
    (0x02163E1C, "474c0634192d181f"),
    (0x02163E34, "c027a94381b08ca8"),
    (0x02163E58, "32f6ec4302f4ea3c"),
    (0x02163E7C, "6537ef4bf571a281"),
    (0x02163F10, "6948a6a92d155816"),
    (0x02163F40, "8da538efddab411b"),
    (0x02163F8C, "5962082d2ae4427f"),
    (0x02163FC0, "020229001357f651"),
    (0x02164010, "b0ec9ecfb288c48a"),
    (0x021640B0, "ad17adfe90ae37ca"),
    (0x02164168, "57dbc737a4c41281"),
    (0x021641C8, "5e18606c4ee2e1d9"),
    (0x021641E8, "87f4b08dbfb966a5"),
    (0x021641F8, "dd1f2d5416519dec"),
    (0x02164208, "2ca8c2b7d95cfcc3"),
    (0x02164240, "47eb80929a00cdd7"),
    (0x02164280, "2ed627a46fd82fc3"),
    (0x02164294, "60af5baa7e6f7e74"),
    (0x021642A4, "cac0c66d983ab992"),
    (0x021642B4, "96a464d84c2e8a0a"),
    (0x021642D4, "ced3211f4c5372fe"),
    (0x021642F8, "870b3e19b352d325"),
    (0x02164304, "03b6bfe69b416cbd"),
    (0x0216435C, "03bb1986d54855dc"),
    (0x02164380, "f99d1b26b0ff7b64"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        (match addr {
            0x0216_3010 => {
                move_init(e, v[0]);
                0
            }
            0x0216_3028 => {
                move_proc(e, v[0]);
                0
            }
            0x0216_3090 => can_move(e, v[0]),
            0x0216_30F4 => {
                height_update(e, v[0]);
                0
            }
            0x0216_310C => {
                attr_update(e, v[0]);
                0
            }
            0x0216_3130 => {
                step_start_effects(e, v[0]);
                0
            }
            0x0216_3154 => {
                step_end_effects(e, v[0]);
                0
            }
            0x0216_3188 => {
                step_end_effects_2(e, v[0]);
                0
            }
            0x0216_31C0 => {
                attr_info(e, v[0], v[1]);
                0
            }
            0x0216_3230 => {
                effects_step_start(e, v[0]);
                0
            }
            0x0216_3288 => {
                effects_step_end(e, v[0]);
                0
            }
            0x0216_32F0 => {
                effects_jump_end(e, v[0]);
                0
            }
            0x0216_3340 => {
                effects_step_start_2(e, v[0]);
                0
            }
            0x0216_33A8 => {
                effects_step_end_2(e, v[0]);
                0
            }
            0x0216_3418 => {
                set_draw_offset(e, v[0], v[1]);
                0
            }
            0x0216_3464 => effect_kind(e, v[0], v[1]),
            0x0216_34D8 => {
                fx_effect_start(e, v[0], v[1]);
                0
            }
            0x0216_34FC => {
                fx_effect_moving(e, v[0], v[1]);
                0
            }
            0x0216_352C => {
                fx_effect_still(e, v[0], v[1]);
                0
            }
            0x0216_355C => {
                fx_effect_end(e, v[0], v[1]);
                0
            }
            0x0216_3580 => {
                fx_bbdd4(e, v[0], v[1]);
                0
            }
            0x0216_3618 => {
                fx_c4be0(e, v[0], v[1]);
                0
            }
            0x0216_3650 => {
                fx_c4be0_or_c4a10(e, v[0], v[1]);
                0
            }
            0x0216_369C => {
                clear_bit_40000(e, v[0], v[1]);
                0
            }
            0x0216_36A8 => has_shadow(e, v[0], v[1]),
            0x0216_36C8 => attr_shadow(e, v[0]),
            0x0216_36F0 => {
                fx_shadow(e, v[0], v[1]);
                0
            }
            0x0216_372C => {
                shadow_bit(e, v[0], v[1]);
                0
            }
            0x0216_375C => {
                fx_ac528(e, v[0], v[1]);
                0
            }
            0x0216_377C => {
                fx_c484c_old(e, v[0], v[1]);
                0
            }
            0x0216_37B0 => {
                fx_c484c(e, v[0], v[1]);
                0
            }
            0x0216_37E4 => {
                fx_bbfb8_on(e, v[0], v[1]);
                0
            }
            0x0216_3868 => {
                fx_bbfb8_off(e, v[0], v[1]);
                0
            }
            0x0216_38AC => {
                fx_c8f7c(e, v[0], v[1]);
                0
            }
            0x0216_38C8 => {
                nothing(e, v[0], v[1]);
                0
            }
            0x0216_38CC => step_hits(e, v[0], v[1], v[2], v[3], v[4], v[5] as u16),
            0x0216_3994 => step_hits_here(e, v[0], v[1], v[2], v[3], v[4] as u16),
            0x0216_39C0 => step_hits_dir(e, v[0], v[1]),
            0x0216_3A08 => {
                footprint(e, v[0], v[1], v[2], v[3]);
                0
            }
            0x0216_3A40 => rects_overlap(&e.mem, v[0], v[1]) as u32,
            0x0216_3AF0 => mmdl_blocks(e, v[0], v[1], v[2], v[3], v[4], v[5]),
            0x0216_3BAC => mmdl_at(e, v[0], v[1], v[2], v[3]),
            0x0216_3C04 => in_footprint(e, v[0], v[1], v[2], v[3]),
            0x0216_3C9C => outside_limit(e, v[0], v[1], v[2]),
            0x0216_3D04 => attr_blocked(e, v[0], v[1], v[2], v[3], v[4]),
            0x0216_3E1C => map_query(e, v[0], v[1], v[2]),
            0x0216_3E34 => attr_at(e, v[0], v[1], v[2]),
            0x0216_3E58 => height_at(e, v[0], v[1], v[2]),
            0x0216_3E7C => {
                grid_step(e, v[0], v[1]);
                0
            }
            0x0216_3F10 => {
                save_old_grid(e, v[0]);
                0
            }
            0x0216_3F40 => attr_ahead(e, v[0], v[1]),
            0x0216_3F8C => {
                pos_add(e, v[0], v[1]);
                0
            }
            0x0216_3FC0 => {
                pos_step(e, v[0], v[1], v[2]);
                0
            }
            0x0216_4010 => height_from_map(e, v[0]),
            0x0216_40B0 => attr_from_map(e, v[0]),
            0x0216_4168 => near_player(e, v[0]),
            0x0216_41C8 => {
                step_start_move(e, v[0]);
                0
            }
            0x0216_41E8 => dir_dx(v[0]),
            0x0216_41F8 => dir_dz(v[0]),
            0x0216_4208 => {
                vec_step(&mut e.mem, v[0], v[1], v[2]);
                0
            }
            0x0216_4240 => {
                grid_step_xz(&mut e.mem, v[0], v[1], v[2], v[3]);
                0
            }
            0x0216_4280 => {
                tile_centre(&mut e.mem, v[0], v[1], v[2]);
                0
            }
            0x0216_4294 => {
                grid_to_fx(&mut e.mem, v[0], v[1], v[2], v[3]);
                0
            }
            0x0216_42A4 => dir_opposite(v[0]),
            0x0216_42B4 => dir_between(v[0] as i32, v[1] as i32, v[2] as i32, v[3] as i32) as u32,
            0x0216_42D4 => dir_rotated(v[0], v[1]),
            0x0216_42F8 => {
                set_rail(e, v[0]);
                0
            }
            0x0216_4304 => {
                rail_from_header(e, v[0], v[1]);
                0
            }
            0x0216_435C => {
                rail_loc_store(e, v[0], v[1]);
                0
            }
            0x0216_4380 => rail_loc_load(e, v[0], v[1]),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }) as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mmdl_move`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mmdl_move", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn x_decides_before_z() {
    assert_eq!(dir_between(5, 0, 4, 9), 2);
    assert_eq!(dir_between(-1, 0, 0, -9), 3);
    assert_eq!(dir_between(3, 2, 3, 1), 0);
    assert_eq!(dir_between(3, 2, 3, 2), 1);
}
