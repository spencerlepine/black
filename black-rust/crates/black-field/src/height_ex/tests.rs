use super::*;
use crate::testenv::{ofs_arg, run};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C0598, "cd3ede0886dfd55c"),
    (0x021C05D4, "3cefc0bd5ebb897e"),
    (0x021C0628, "fb41a2c124fc30f4"),
    (0x021C0674, "3d017b5f377cc125"),
    (0x021C0688, "0d725dd5e1bf09d1"),
    (0x021C068C, "b84e81969871669e"),
    (0x021C06A8, "78dc3b05ac38d1ab"),
    (0x021C06B4, "be1a0871baa7f7f0"),
    (0x021C06C0, "3c4217fb2ef0f47f"),
    (0x021C06CC, "c99f94064211077d"),
    (0x021C06E0, "a777200d6b72c19e"),
    (0x021C0780, "518cb01529749c5e"),
    (0x021C0808, "a14d00f43872b22d"),
    (0x021C080C, "a14d00f43872b22d"),
    (0x021C0810, "a609dd06d609b0fa"),
    (0x021C0820, "a837a3263b622298"),
    (0x021C0844, "ba7d413a8d3018d6"),
    (0x021C0904, "1eb890948ebd7876"),
    (0x021C0988, "bd1ab7d806da3f5a"),
    (0x021C09A8, "aa2e22fcd9e1afd3"),
    (0x021C0A10, "8415d574df16626e"),
    (0x021C0A48, "7ec9f5ff47448558"),
    (0x021C0A70, "466587186f345e68"),
    (0x021C0AA0, "1ea4bfa1378ca507"),
    (0x021C0AC0, "f1dcead2e49e8914"),
    (0x021C0B3C, "caad22644333b7a2"),
    (0x021C0BD8, "9ab721862ef206ad"),
    (0x021C0C48, "a119bbcb3db05780"),
    (0x021C0C80, "97669b374bbea917"),
    (0x021C0C94, "e585f9c2bf8f03d6"),
    (0x021C0CA4, "3eeba7e602461049"),
    (0x021C0CE4, "907098a4cdc2c9f5"),
    (0x021C0D08, "65a2a755edcd0228"),
    (0x021C0DDC, "f72ad45ed4aebed7"),
    (0x021C0E4C, "21a3df0d549d9792"),
    (0x021C0E98, "ad9fba5d4093d5af"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/height_ex.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        (match addr {
            0x021C_0598 => rect_contains(&e.mem, v[0], v[1], v[2]) as u32,
            0x021C_05D4 => return ofs_arg(create(e, v[0], v[1]), 2),
            0x021C_0628 => {
                set_record(&mut e.mem, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
                0
            }
            0x021C_0674 => {
                delete(e, v[0]);
                0
            }
            0x021C_0688 => count(&e.mem, v[0]),
            0x021C_068C => record_contains(&e.mem, v[0], v[1], v[2], v[3]) as u32,
            0x021C_06A8 => height(&e.mem, v[0], v[1]),
            0x021C_06B4 => extra(&e.mem, v[0], v[1]),
            0x021C_06C0 => {
                set_height(&mut e.mem, v[0], v[1], v[2]);
                0
            }
            0x021C_06CC => callback_event_new(e, v[0]),
            0x021C_06E0 => callback_event(e, v[0], v[1]),
            0x021C_0780 => callback_child(e, v[0], v[1]),
            0x021C_0808 => callback_true(),
            0x021C_080C => callback_true_2(),
            0x021C_0810 => callback_end(e, v[2]),
            0x021C_0820 => models_event_new(e, v[0], v[1], v[2]),
            0x021C_0844 => models_event(e, v[0], v[1], v[2]),
            0x021C_0904 => {
                models_init(e, v[0], v[1], v[2]);
                0
            }
            0x021C_0988 => {
                models_delete(e, v[0]);
                0
            }
            0x021C_09A8 => {
                set_step(e, v[0], v[1], v[2]);
                0
            }
            0x021C_09B0 => {
                models_add(e, v[0]);
                0
            }
            0x021C_0A10 => {
                models_anim_start(e, v[0]);
                0
            }
            0x021C_0A48 => {
                models_anim_end(e, v[0]);
                0
            }
            0x021C_0A70 => models_anim_done(e, v[0]),
            0x021C_0AA0 => models_event_2_new(e, v[0], v[1]),
            0x021C_0AC0 => models_event_2(e, v[0], v[1], v[2]),
            0x021C_0B3C => {
                models_init_2(e, v[0], v[1]);
                0
            }
            0x021C_0BD8 => {
                models_delete_2(e, v[0]);
                0
            }
            0x021C_0BE8 => {
                models_add_2(e, v[0]);
                0
            }
            0x021C_0C48 => {
                models_anim_start_2(e, v[0]);
                0
            }
            0x021C_0C80 => models_anim_done_2(e, v[0]),
            0x021C_0C94 => party_count(e, v[0]),
            0x021C_0CA4 => step_check(e, v[0], v[1], v[2], v[3]),
            0x021C_0CE4 => push_check(e, v[0], v[1], v[2]),
            0x021C_0D08 => footprint(e, v[0], v[1], v[2]),
            0x021C_0DDC => push_start(e, v[0], v[1], v[2]),
            0x021C_0E4C => push_event_new(e, v[0], v[1], v[2], v[3]),
            0x021C_0E98 => push_event(e, v[0], v[1], v[2]),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }) as u64
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field height_ex`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("height_ex", SPEC, PORTED, RECORDED), &call);
}
