use super::*;
use crate::testenv::{ofs_arg, ofs_work, run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216237C, "130b0ce0722294dd"),
    (0x021623BC, "d4c1313b5ce9dc5d"),
    (0x021623DC, "26b550cf56f66c0f"),
    (0x02162408, "04064f6de5604c3b"),
    (0x0216242C, "59726c269a6bf96d"),
    (0x02162438, "23f86a34dbfcf1de"),
    (0x02162450, "fe7c9a6eb813459f"),
    (0x02162458, "6ec6ef43ad16ec00"),
    (0x0216245C, "51e96db3df438814"),
    (0x02162478, "b1e7e3f413ace6a8"),
    (0x021624B0, "992563840bc4a4ef"),
    (0x02162530, "9cc91fb2934d4f60"),
    (0x0216255C, "2a9dc7e01a4385d3"),
    (0x02162560, "706c4730d958ffad"),
    (0x02162564, "43da93cadaa9db9c"),
    (0x02162598, "0602a5e5b23f7e6e"),
    (0x021625CC, "3fdf6ac27877002c"),
    (0x021625E8, "24bc8eb7d39667a3"),
    (0x021625FC, "ab784f827279bb3d"),
    (0x02162694, "70c99baa44e64f61"),
    (0x021626B8, "0cd172a1f52900a5"),
    (0x021626E8, "b3e5a4eabce5a2f9"),
    (0x021626EC, "b895d17aa0066bc7"),
    (0x02162708, "32d5cb51b48e5ea7"),
    (0x02162740, "302d2cc77486054a"),
    (0x02162744, "3fff891f0b67a7f7"),
    (0x02162748, "d707d79cfc2c81ff"),
    (0x02162774, "c7906adb4913cc82"),
    (0x02162788, "55ff3741bf40fbe5"),
    (0x0216279C, "0e4d8f326727d608"),
    (0x021627C8, "522006af3f6cede0"),
    (0x0216282C, "05d839381fec6569"),
    (0x02162890, "bdee5837acff521f"),
    (0x021628E4, "9fec265369d95294"),
    (0x02162928, "4bb7efe72068dd74"),
    (0x02162940, "c1526b748c49ba25"),
    (0x02162958, "b7118135d68d0293"),
    (0x02162970, "15dcc3eb28979370"),
    (0x02162A54, "332ee3a066a680f6"),
    (0x02162A5C, "b3cdf1568aa93fa5"),
    (0x02162A74, "6547c657868e4fb1"),
    (0x02162AA4, "0c79d3f28fe4a9b8"),
    (0x02162B68, "ecf4c54daf0f6341"),
    (0x02162BCC, "8e7fd2bf9b882b79"),
    (0x02162C54, "045d3001fed6ce4e"),
    (0x02162C84, "a6a431744ee88247"),
    (0x02162CE8, "be853f02dffd78f4"),
    (0x02162D2C, "aa222acd171682c8"),
    (0x02162D90, "b36e04f70b645a2d"),
    (0x02162E00, "17dc55eca32ef4cc"),
    (0x02162E48, "62493e86cad98520"),
    (0x02162E9C, "29f98728492c9f1a"),
    (0x02162ECC, "7872ec6e74641dca"),
    (0x02162F0C, "8094117d59e55d88"),
    (0x02162F74, "df64d0ba6a34b433"),
    (0x02162FC8, "2365132c3f557969"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/eventdata.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = &mut e.mem;
        match addr {
            0x0216_237C => ofs_arg(create(e, v[0]), 1),
            0x0216_23BC => {
                delete(e, v[0]);
                0
            }
            0x0216_23DC => {
                reset(m, v[0]);
                0
            }
            0x0216_2408 => {
                load(e, v[0], v[1], v[2]);
                0
            }
            0x0216_242C => {
                set_zone(e, v[0], v[1]);
                0
            }
            0x0216_2438 => {
                encount_init(e, v[0], v[1], v[2]);
                0
            }
            0x0216_2450 => tail(m, v[0]) as u64,
            0x0216_2458 => encount_ready(m, v[0]) as u64,
            0x0216_245C => {
                warp_disable(m, v[0], v[1]);
                0
            }
            0x0216_2478 => kind6_trigger_at(m, v[0], v[1]) as u64,
            0x0216_24B0 => {
                load_member(e, v[0], v[1]);
                0
            }
            0x0216_2530 => {
                clear(m, v[0]);
                0
            }
            0x0216_255C => ofs_work(encount_work(v[0])),
            0x0216_2560 => warp_count(m, v[0]) as u64,
            0x0216_2564 => warp_index_at(m, v[0], v[1]) as u64,
            0x0216_2598 => warp_index_at_rail(m, v[0], v[1]) as u64,
            0x0216_25CC => ofs_work(warp_at(m, v[0], v[1])),
            0x0216_25E8 => warp_dest_is_0x100(m, v[0]) as u64,
            0x0216_25FC => warp_exit_location(e, v[0], v[1], v[2], v[3]) as u64,
            0x0216_2694 => {
                location_set(e, v[0], v[1], v[2]);
                0
            }
            0x0216_26B8 => warp_faces(m, v[0], v[1]) as u64,
            0x0216_26E8 => warp_door_kind(m, v[0]) as u64,
            0x0216_26EC => warp_is_disabled(m, v[0]) as u64,
            0x0216_2708 => {
                warp_set_tile(m, v[0], v[1], v[2], v[3], v[4] as u16);
                0
            }
            0x0216_2740 => npcs(m, v[0]) as u64,
            0x0216_2744 => npc_count(m, v[0]) as u64,
            0x0216_2748 => {
                npc_set_pos(m, v[0], v[1], v[2], v[3], v[4], v[5] as u16);
                0
            }
            0x0216_2774 => {
                npc_set_obj_code(m, v[0], v[1], v[2]);
                0
            }
            0x0216_2788 => {
                npc_set_script(m, v[0], v[1], v[2]);
                0
            }
            0x0216_279C => dir_trigger_kind(v[0]) as u64,
            0x0216_27C8 => ofs_work(trigger_at(e, v[0], v[1], v[2], v[3])),
            0x0216_282C => ofs_work(trigger_at_2d(e, v[0], v[1], v[2], v[3])),
            0x0216_2890 => ofs_work(trigger6_at(e, v[0], v[1], v[2])),
            0x0216_28E4 => ofs_work(trigger_at_rail(e, v[0], v[1], v[2])),
            0x0216_2928 => trigger_script_at(e, v[0], v[1], v[2], v[3]) as u64,
            0x0216_2940 => trigger6_script_at(e, v[0], v[1], v[2]) as u64,
            0x0216_2958 => trigger_script_at_rail(e, v[0], v[1], v[2]) as u64,
            0x0216_2970 => bg_script_at(e, v[0], v[1], v[2], v[3]) as u64,
            0x0216_2A54 => bg_script_at_a(e, v[0], v[1], v[2], v[3]) as u64,
            0x0216_2A5C => bg_script_at_b(e, v[0], v[1], v[2], v[3]) as u64,
            0x0216_2A74 => {
                bg_set_pos(m, v[0], v[1], v[2], v[3], v[4] as u16);
                0
            }
            0x0216_2AA4 => door_offset(v[0], v[1], v[2], v[3], v[4] as u16) as u64,
            0x0216_2B68 => warp_exit_code(m, v[0], v[1]) as u64,
            0x0216_2BCC => warp_exit_code_rail(m, v[0], v[1]) as u64,
            0x0216_2C54 => warp_exit_dir(m, v[0]) as u64,
            0x0216_2C84 => {
                warp_exit_pos(e, v[0], v[1], v[2]);
                0
            }
            0x0216_2CE8 => {
                warp_centre(m, v[0], v[1]);
                0
            }
            0x0216_2D2C => warp_contains(m, v[0], v[1]) as u64,
            0x0216_2D90 => {
                warp_exit_rail(e, v[0], v[1], v[2]);
                0
            }
            0x0216_2E00 => warp_contains_rail(m, v[0], v[1]) as u64,
            0x0216_2E48 => bg_contains(m, v[0], v[1]) as u64,
            0x0216_2E9C => bg_contains_rail(m, v[0], v[1]) as u64,
            0x0216_2ECC => {
                trigger_centre(e, v[0], v[1]);
                0
            }
            0x0216_2F0C => trigger_contains(m, v[0], v[1]) as u64,
            0x0216_2F74 => trigger_contains_2d(m, v[0], v[1]) as u64,
            0x0216_2FC8 => trigger_contains_rail(m, v[0], v[1]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field eventdata`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("eventdata", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn warp_lookup_and_bounds() {
    let mut mem = Mem::new(WORK, vec![0u8; 0x400]);
    mem.w16(WORK + 0x16, 2);
    assert_eq!(warp_at(&mem, WORK, 0), 0); // no records
    mem.w32(WORK + 0x24, WORK + 0x100);
    assert_eq!(warp_at(&mem, WORK, 1), WORK + 0x114);
    assert_eq!(warp_at(&mem, WORK, 2), 0);
    // a 2-tile wide door at x 0x18 (tile 1 centre), y 0, z 0x28
    let (w, p) = (WORK + 0x100, WORK + 0x200);
    mem.w16(w + 8, 0x18);
    mem.w16(w + 0xC, 0x28);
    mem.w16(w + 0xE, 2);
    mem.w16(w + 0x10, 1);
    mem.w32(p, 0x28 << 12);
    mem.w32(p + 4, 2 << 12);
    mem.w32(p + 8, 0x28 << 12);
    assert!(warp_contains(&mem, w, p));
    mem.w32(p, 0x29 << 12);
    assert!(!warp_contains(&mem, w, p));
    mem.w8(w + 4, 3);
    assert_eq!(warp_exit_dir(&mem, w), 2);
}
