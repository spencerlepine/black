use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021BB1F4, "2d0814933ded82ba"),
    (0x021BB240, "e0f7b33d344eb7db"),
    (0x021BB260, "70d1fe8c7e3be57b"),
    (0x021BB278, "9f077e9c622a9048"),
    (0x021BB2F8, "f9de9295e67aaa51"),
    (0x021BB348, "03b76f305059995c"),
    (0x021BB36C, "7702b38f7f37da3e"),
    (0x021BB388, "5b78c29dee1a08ed"),
    (0x021BB394, "e105e6ff84b5e3bd"),
    (0x021BB3A0, "859981d17b337de8"),
    (0x021BB3AC, "455a389b841b562d"),
    (0x021BB3B8, "a07a5bfd5f386fab"),
    (0x021BB3C4, "0e5d728505393f98"),
    (0x021BB3D0, "7a02cd2619615e96"),
    (0x021BB3DC, "c12a40f3b9c18b59"),
    (0x021BB3E8, "d19de869f4a78868"),
    (0x021BB480, "508480af1227291e"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/nogrid_mapper.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021B_B1F4 => u64::from(create(e, v[0], v[1], v[2], v[3])),
        0x021B_B240 => {
            delete(e, v[0]);
            0
        }
        0x021B_B260 => {
            set_rail_frame(e, v[0], v[1]);
            0
        }
        0x021B_B278 => {
            load_record(e, v[0], v[1], v[2]);
            0
        }
        0x021B_B2F8 => {
            load(e, v[0], v[1], v[2]);
            0
        }
        0x021B_B348 => {
            unload(e, v[0]);
            0
        }
        0x021B_B36C => {
            frame(e, v[0]);
            0
        }
        0x021B_B388 => u64::from(frame_forced(e, v[0])),
        0x021B_B394 => {
            rail_7970(e, v[0]);
            0
        }
        0x021B_B3A0 => {
            rail_79ac(e, v[0], v[1]);
            0
        }
        0x021B_B3AC => {
            rail_79c4(e, v[0], v[1]);
            0
        }
        0x021B_B3B8 => {
            rail_79cc(e, v[0]);
            0
        }
        0x021B_B3C4 => u64::from(attr_at(e, v[0], v[1])),
        0x021B_B3D0 => {
            rail_7d68(e, v[0], v[1]);
            0
        }
        0x021B_B3DC => u64::from(rail_7c04(e, v[0], v[1], v[2])),
        0x021B_B3E8 => {
            location_in_zone(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]);
            0
        }
        0x021B_B480 => u64::from(rail(e, v[0])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field nogrid_mapper`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("nogrid_mapper", SPEC, PORTED, RECORDED), &call);
}
