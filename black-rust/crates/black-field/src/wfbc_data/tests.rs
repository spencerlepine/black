use super::*;
use crate::testenv::{ofs_work, run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02161D98, "a7986eac4f549d1b"),
    (0x02161DC4, "5f53840a9235b460"),
    (0x02161E4C, "5f44aa1d3f9bfb20"),
    (0x02161E60, "c69f347202925388"),
    (0x02161E78, "e42836be8cfee534"),
    (0x02161E84, "8b1e08e861b5a390"),
    (0x02161ECC, "b898bfc918d9a3ab"),
    (0x02161EEC, "5180fa498a5ddfce"),
    (0x02161F00, "912dc5ff6d1f08f2"),
    (0x02161F14, "e0ebcc7a484d78c6"),
    (0x02161F54, "378ee26e0b4b3249"),
    (0x02161F68, "46d3071a6611e491"),
    (0x02161F74, "a0a34056ca81a820"),
    (0x02161F90, "08ca282dc9d54813"),
    (0x02161FEC, "34ae3d314ab77120"),
    (0x02161FF0, "c5d5cf12d0f726fd"),
    (0x02162008, "1c386a5320694a1e"),
    (0x0216202C, "95643fea82c776e3"),
    (0x02162048, "c12f17bb5a9a5761"),
    (0x0216208C, "1e4990c3ffca2297"),
    (0x0216209C, "fec2d8dc3026c96b"),
    (0x021620A0, "41a86252c84c8f1b"),
    (0x021620B0, "e37748edd8b3ed21"),
    (0x0216210C, "7f28e86a02b4aa24"),
    (0x02162120, "bd5bda5b661afa53"),
    (0x02162198, "cc473eecfead8e3a"),
    (0x021621C0, "458553040838a6ca"),
    (0x021621E8, "9e268e8b641006ae"),
    (0x0216220C, "a5db889f8c9a8f75"),
    (0x02162230, "36f57c56c368b766"),
    (0x02162254, "28b863af3fd4d04e"),
    (0x02162278, "cd4c2193cb09f1a5"),
    (0x02162298, "2e07317b45101dbb"),
    (0x021622AC, "03b592330a990a95"),
    (0x021622D8, "2fc4327eb68fb2e4"),
    (0x021622F4, "88b9b2bab5e3ecd2"),
    (0x0216230C, "e1e9f49a51b818da"),
    (0x02162330, "bf6d372a0314aba5"),
    (0x02162358, "45ef199b51b3b6dd"),
    (0x02162370, "5fcc4a8f42da9cd2"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/wfbc_data.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = &mut e.mem;
        (match addr {
            0x0216_1D98 => {
                list_init(m, v[0]);
                0
            }
            0x0216_1DC4 => return ofs_work(list_find(e, v[0], v[1], v[2])),
            0x0216_1E4C => list_has(e, v[0], v[1], v[2]) as u32,
            0x0216_1E60 => {
                list_compact(e, v[0], v[1]);
                0
            }
            0x0216_1E78 => {
                rec_clear(m, v[0]);
                0
            }
            0x0216_1E84 => rec_fields_valid(e, v[0]) as u32,
            0x0216_1ECC => rec_valid(e, v[0]) as u32,
            0x0216_1EEC => {
                fill_none10(m, v[0]);
                0
            }
            0x0216_1F00 => {
                pair_init(m, v[0]);
                0
            }
            0x0216_1F14 => {
                people_info(e, v[0], v[1], v[2]);
                0
            }
            0x0216_1F54 => {
                comm_init(e, v[0]);
                0
            }
            0x0216_1F68 => {
                comm_clear(m, v[0]);
                0
            }
            0x0216_1F74 => {
                comm_apply_slots(e, v[0], v[1], v[2]);
                0
            }
            0x0216_1F90 => {
                rec_store(e, v[0], v[1], v[2], v[3], v[4] as u16, v[5] as u16);
                0
            }
            0x0216_1FEC => {
                comm_set_4(m, v[0], v[1]);
                0
            }
            0x0216_1FF0 => {
                comm_put_20(e, v[0], v[1]);
                0
            }
            0x0216_2008 => {
                comm_put_slot_a(e, v[0], v[1], v[2]);
                0
            }
            0x0216_202C => comm_get_28(m, v[0], v[1], v[2]) as u32,
            0x0216_2048 => {
                comm_request(e, v[0], v[1], v[2]);
                0
            }
            0x0216_208C => has_20(m, v[0]) as u32,
            0x0216_209C => get_26(m, v[0]),
            0x0216_20A0 => {
                clear_20_28(m, v[0]);
                0
            }
            0x0216_20B0 => {
                recs_compact(e, v[0]);
                0
            }
            0x0216_210C => comm_active(e) as u32,
            0x0216_2120 => {
                apply_slots(e, v[0], v[1], v[2]);
                0
            }
            0x0216_2198 => handler_0(e, v[0], v[1], v[2], v[3]),
            0x0216_21C0 => handler_1(e, v[0], v[1], v[2], v[3]),
            0x0216_21E8 => handler_2(e, v[0], v[1], v[2], v[3]),
            0x0216_220C => {
                put_20(m, v[0], v[1]);
                0
            }
            0x0216_2230 => {
                slot_a_set(m, v[0], v[1], v[2]);
                0
            }
            0x0216_2254 => slot_a_get(m, v[0], v[1], v[2]) as u32,
            0x0216_2278 => {
                slot_b_set(m, v[0], v[1], v[2], v[3]);
                0
            }
            0x0216_2298 => {
                set_28(m, v[0], v[1], v[2]);
                0
            }
            0x0216_22AC => slot_b_get(m, v[0], v[1], v[2]) as u32,
            0x0216_22D8 => get_28(m, v[0], v[1]) as u32,
            0x0216_22F4 => {
                clear_20(m, v[0]);
                0
            }
            0x0216_230C => {
                slot_a_clear(m, v[0], v[1]);
                0
            }
            0x0216_2330 => {
                slot_b_clear(m, v[0], v[1]);
                0
            }
            0x0216_2358 => {
                clear_28(m, v[0]);
                0
            }
            0x0216_2370 => at_least_7(v[0]) as u32,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }) as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field wfbc_data`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("wfbc_data", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn slot_copy_needs_the_flag() {
    let mut mem = Mem::new(WORK, vec![0u8; 0x100]);
    mem.w32(WORK + 0x18, 0x1122_3344);
    assert!(!slot_a_get(&mut mem, WORK, 2, WORK + 0x80));
    mem.w16(WORK + 2, 4);
    assert!(slot_a_get(&mut mem, WORK, 2, WORK + 0x80));
    assert_eq!(mem.r32(WORK + 0x80), 0x1122_3344);
    assert!(!slot_a_get(&mut mem, WORK, 34, WORK + 0x80));
}
