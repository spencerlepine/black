use super::*;
use crate::testenv::{run, Env};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02183B60, "81c3bd364dc5405b"),
    (0x02183B6C, "207dd551928bd83d"),
    (0x02183B90, "a132a07d33d6faee"),
    (0x02183C34, "34eb2f26893289ad"),
    (0x02183C58, "c94fd5a2d9973b92"),
    (0x02183C9C, "fc304bcdf5f693e4"),
    (0x02183D68, "b3ff7f8c6c62bb39"),
    (0x02183DD4, "f6bbdbbc03e1d739"),
    (0x02183DE4, "86a53d6cbee2fc0a"),
    (0x02183F5C, "c0806fd18f294585"),
    (0x0218403C, "a97568d563459cb9"),
    (0x0218409C, "053fb33629e8f947"),
    (0x02184120, "b8b5b81e8dc9b65b"),
    (0x021841C0, "8b4b1773ef22e890"),
    (0x02184360, "5d4551dc18dab2e5"),
    (0x02184388, "eac5ed28b1739dbe"),
    (0x02184410, "c629bed159bb9d17"),
    (0x02184504, "0e651cf29eeb77f6"),
    (0x02184584, "3f004ebfcf91c688"),
    (0x0218466C, "45c77fc140dde948"),
    (0x02184730, "cf8d2c351d1c2185"),
    (0x021847CC, "ca800192c044743f"),
    (0x02184894, "8d8d5f5738120740"),
    (0x0218495C, "24dc8df00306032f"),
    (0x02184A14, "fae2490cc1a449fe"),
    (0x02184AB4, "3444a1c26fd1bce7"),
    (0x02184B54, "6220e6da8caa6e18"),
    (0x02184BF8, "2b182baccb40d78f"),
    (0x02184C78, "4a2cd0a10a3b2b93"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mapchange_proc.spec");

impl MapChangeEnv for Env<'_> {
    fn call64(&mut self, addr: u32) -> u64 {
        self.stubs.call64(addr, &mut [])
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0218_3B60 => call_cb(e, v[0]) as u64,
        0x0218_3B6C => start_event(e, v[0], v[1]) as u64,
        0x0218_3B90 => start_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_3C34 => start_new(e, v[0], v[1]) as u64,
        0x0218_3C58 => {
            wfbc_reset(e, v[0]);
            0
        }
        0x0218_3C9C => new_game_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_3D68 => new_game_new(e, v[0], v[1]) as u64,
        0x0218_3DD4 => {
            wfbc_post(e, v[0]);
            0
        }
        0x0218_3DE4 => continue_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_3F5C => continue_new(e, v[0], v[1]) as u64,
        0x0218_403C => {
            season_check(e, v[0]);
            0
        }
        0x0218_409C => {
            door_sub(e, v[0], v[1]);
            0
        }
        0x0218_4120 => {
            door_sub_exit(e, v[0], v[1]);
            0
        }
        0x0218_41C0 => zone_change_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4360 => zone_change_new(e, v[0], v[1]) as u64,
        0x0218_4388 => door_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4410 => door_comm_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4504 => warp_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4584 => fade_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_466C => wait_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4730 => pos_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_47CC => special_a_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4894 => special_b_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_495C => zone_c_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4A14 => special_d_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4AB4 => zone_e_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4B54 => grid_f_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4BF8 => union_in_proc(e, v[0], v[1], v[2]) as u64,
        0x0218_4C78 => union_out_proc(e, v[0], v[1], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mapchange_proc`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mapchange_proc", SPEC, PORTED, RECORDED), &call);
}
