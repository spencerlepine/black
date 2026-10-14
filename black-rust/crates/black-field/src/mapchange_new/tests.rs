use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02184CEC, "035aa73045a2d742"),
    (0x02184D38, "b5deb035f2726ffb"),
    (0x02184D90, "939ead13d23e5b8e"),
    (0x02184DE8, "cd164dc60f0fcd2b"),
    (0x02184E40, "e5478d1a1d85c310"),
    (0x02184E94, "377d27b7c4d8e61f"),
    (0x02184EEC, "12566f3303b06e91"),
    (0x02184F40, "682a1edced94128d"),
    (0x02184F88, "fe4644531c75a808"),
    (0x02184FD4, "e88b86cff3d79593"),
    (0x02185050, "a1059847efe2a595"),
    (0x021850A8, "a3b7ddaa46ce8c0a"),
    (0x021850FC, "ba68db4f04a5f524"),
    (0x02185140, "0d1a32350737a3ed"),
    (0x02185214, "982a05ec932359a9"),
    (0x021852F0, "ea6d6997b76b1eb5"),
    (0x02185360, "ee9c98f612739820"),
    (0x02185410, "4338d388cdcebe69"),
    (0x02185458, "fafde1a713c9a596"),
    (0x021854B0, "e613369cf88b0c1a"),
    (0x021854CC, "74ccaaa195691bb6"),
    (0x02185534, "1f304c85ff6a2f88"),
    (0x02185588, "94adb3bd8337f26a"),
    (0x0218562C, "8ece6174d9dd006c"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mapchange_new.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0218_4CEC => {
            work_init(e, v[0], v[1]);
            0
        }
        0x0218_4D38 => door_new(e, v[0], v[2], v[3], v[4], v[5]) as u64,
        0x0218_4D90 => door_rail_new(e, v[0], v[2], v[3], v[4], v[5]) as u64,
        0x0218_4DE8 => warp_rail_new(e, v[0], v[2], v[3], v[4], v[5]) as u64,
        0x0218_4E40 => pos_new(e, v[0], v[2], v[3], v[4]) as u64,
        0x0218_4E94 => special_a_new(e, v[0], v[1]) as u64,
        0x0218_4EEC => special_b_new(e, v[0]) as u64,
        0x0218_4F40 => zone_c_new(e, v[0]) as u64,
        0x0218_4F88 => special_d_new(e, v[0]) as u64,
        0x0218_4FD4 => zone_e_new(e, v[0], v[1]) as u64,
        0x0218_5050 => grid_f_new(e, v[0], v[2], v[3], v[4]) as u64,
        0x0218_50A8 => union_out_new(e, v[0]) as u64,
        0x0218_50FC => union_in_new(e, v[0]) as u64,
        0x0218_5140 => comm_pos_new(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_5214 => comm_return_new(e, v[0]) as u64,
        0x0218_52F0 => {
            player_loc(e, v[0], v[1]);
            0
        }
        0x0218_5360 => door_warp_new(e, v[0], v[2], v[3]) as u64,
        0x0218_5410 => door_comm_new(e, v[0], v[2], v[3]) as u64,
        0x0218_5458 => warp_new(e, v[0], v[2], v[3], v[4], v[5]) as u64,
        0x0218_54B0 => warp0_new(e, v[0], v[2], v[3], v[4]) as u64,
        0x0218_54CC => warp_loc_new(e, v[0], v[3], v[4]) as u64,
        0x0218_5534 => fade_new(e, v[0], v[2], v[3], v[4]) as u64,
        0x0218_5588 => wait_new(e, v[0], v[1], v[2]) as u64,
        0x0218_562C => {
            special_enter(e, v[0]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mapchange_new`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mapchange_new", SPEC, PORTED, RECORDED), &call);
}
