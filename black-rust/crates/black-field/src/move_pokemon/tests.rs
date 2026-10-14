use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215E440, "3e6529e140a423a9"),
    (0x0215E580, "79a822a9c47a2d9a"),
    (0x0215E5B0, "eeca481976a4e768"),
    (0x0215E6A8, "151d5cbe25eca484"),
    (0x0215E6F8, "285e11e8e8eb7cf7"),
    (0x0215E724, "fc73e7319debe7ca"),
    (0x0215E740, "6ea50612b14c9bd2"),
    (0x0215E7DC, "1c8104b5d8a6e74c"),
    (0x0215E804, "a6396752e378f5d7"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/move_pokemon.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0215_E440 => {
            create_roamer(e, v[0], v[1]);
            0
        }
        0x0215_E580 => {
            respawn_roamers(e, v[0]);
            0
        }
        0x0215_E5B0 => {
            after_battle(e, v[0], v[1]);
            0
        }
        0x0215_E6A8 => roamer_in_zone(e, v[0], v[1]) as u64,
        0x0215_E6F8 => any_active(e, v[0]) as u64,
        0x0215_E724 => roamer_index(v[0]) as u64,
        0x0215_E740 => {
            move_roamer(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0215_E7DC => {
            set_location(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0215_E804 => roamer_record(e, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field move_pokemon`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("move_pokemon", SPEC, PORTED, RECORDED), &call);
}

/// The embedded group table is the ROM's (`0x021711A8`, overlay 10 at `0x02155160`), and the zone table is the one
/// that follows it (`0x021711C6`, `encount_st::SWARM_ZONES`).
#[test]
fn rom_tables_match() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(ov) = dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
        return;
    };
    let at = |a: u32| (a - 0x0215_5160) as usize;
    for (k, (count, locs)) in ROAM_GROUPS.iter().enumerate() {
        let o = at(0x0217_11A8) + 6 * k;
        assert_eq!(u16::from_le_bytes([ov[o], ov[o + 1]]), *count);
        assert_eq!(&ov[o + 2..o + 6], &locs[..]);
    }
    for (k, &z) in SWARM_ZONES.iter().enumerate() {
        let o = at(0x0217_11C6) + 2 * k;
        assert_eq!(u16::from_le_bytes([ov[o], ov[o + 1]]), z);
    }
}

#[test]
fn roamer_index_by_species() {
    assert_eq!(roamer_index(0x282), 0);
    assert_eq!(roamer_index(0x281), 1);
    assert_eq!(roamer_index(0x280), 2);
}
