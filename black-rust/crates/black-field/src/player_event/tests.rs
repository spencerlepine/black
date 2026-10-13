use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C1024, "c96987687b399674"),
    (0x021C1094, "c18ea68039a0a196"),
    (0x021C10A0, "fd8ac52e091b8203"),
    (0x021C10B4, "75ffa314c811c90e"),
    (0x021C1228, "c680fbc201d9b274"),
    (0x021C12C4, "e99d310f2b862f16"),
    (0x021C12D0, "567e9a5406aa55b1"),
    (0x021C12E4, "58f3e36eaae0db5c"),
    (0x021C1318, "13bedc49895392c6"),
    (0x021C146C, "51f8d3d09a482407"),
    (0x021C14F8, "1211c1172376daa9"),
    (0x021C157C, "f853155876b1b383"),
    (0x021C161C, "a3a23de5b559e7ba"),
    (0x021C1670, "676c95985542310d"),
    (0x021C16AC, "560d42e67deea43f"),
    (0x021C1988, "a95f96e0030ed0b3"),
    (0x021C19B4, "3f42223b39f2b45c"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/player_event.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021C_1024 => surf_start_new(e, v[0], v[1], v[2], v[3]) as u64,
        0x021C_10B4 => {
            surf_start_proc(e, v[0], v[1]);
            0
        }
        0x021C_1228 => surf_end_new(e, v[0], v[1], v[2], v[3]) as u64,
        0x021C_1318 => surf_end_jump(e, v[0]) as u64,
        0x021C_146C => surf_end_hop(e, v[0]) as u64,
        0x021C_14F8 => surf_end_rise(e, v[0]) as u64,
        0x021C_157C => surf_end_arc(e, v[0]) as u64,
        0x021C_161C => surf_end_land(e, v[0]) as u64,
        0x021C_1670 => surf_end_wait(e, v[0]) as u64,
        0x021C_16AC => {
            surf_end_down(e, v[0]);
            0
        }
        0x021C_1988 => {
            surf_end_proc(e, v[0], v[1]);
            0
        }
        0x021C_19B4 => mmdl_in_front(e, v[0], v[1]) as u64,
        0x021C_1094 => surf_start_done(e, v[0]) as u64,
        0x021C_10A0 => {
            surf_start_delete(e, v[0]);
            0
        }
        0x021C_12C4 => surf_end_done(e, v[0]) as u64,
        0x021C_12D0 => {
            surf_end_delete(e, v[0]);
            0
        }
        0x021C_12E4 => {
            set_ride_offset(e, v[0], v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field player_event`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("player_event", SPEC, PORTED, RECORDED), &call);
}
