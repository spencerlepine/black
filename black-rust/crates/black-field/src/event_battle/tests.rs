use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216E95C, "494ea828518d665a"),
    (0x0216EAB0, "c5451bd6c411d2a9"),
    (0x0216EB98, "9dd7c3729b67e200"),
    (0x0216EDFC, "c0927a56ce961ce8"),
    (0x0216EE20, "d3c17022e102e291"),
    (0x0216EE40, "ed9a20e34c0a9a50"),
    (0x0216EE7C, "1b9b53b679a049f0"),
    (0x0216EF84, "8381773aab09dccb"),
    (0x0216EFA8, "245ae0aa8604563d"),
    (0x0216EFCC, "20ace57a6277329b"),
    (0x0216EFDC, "92d381b3253ec53f"),
    (0x0216EFF0, "31dad00118fb16f7"),
    (0x0216F02C, "828ec3cfa8f49b4a"),
    (0x0216F054, "0814d042a590f272"),
    (0x0216F064, "ac8052bd61ca5ced"),
    (0x0216F070, "ac8052bd61ca5ced"),
    (0x0216F07C, "ac8052bd61ca5ced"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/event_battle.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0216_E95C => capture_demo_new(e, v[0], v[1], v[2]) as u64,
        0x0216_EAB0 => special_battle_new(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]) as u64,
        0x0216_EB98 => battle_proc(e, v[0], v[1], v[2]) as u64,
        0x0216_EDFC => trainer_class(e, v[0]) as u64,
        0x0216_EE20 => {
            notify_trainer(e, v[0], v[1]);
            0
        }
        0x0216_EE40 => {
            notify_species(e, v[0], v[1]);
            0
        }
        0x0216_EE7C => {
            after_battle_notify(e, v[0]);
            0
        }
        0x0216_EF84 => {
            count_escape(e, v[0]);
            0
        }
        0x0216_EFA8 => lost(v[0], v[1]) as u64,
        0x0216_EFCC => kind_of(v[0]) as u64,
        0x0216_EFDC => lost_of(e, v[0]) as u64,
        0x0216_EFF0 => {
            after_battle(e, v[0], v[1]);
            0
        }
        0x0216_F02C => {
            work_init(e, v[0], v[1], v[2]);
            0
        }
        0x0216_F054 => {
            free_setup(e, v[0]);
            0
        }
        0x0216_F064 | 0x0216_F070 | 0x0216_F07C => {
            check_callback(e);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field event_battle`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("event_battle", SPEC, PORTED, RECORDED), &call);
}

/// Overlay 10 of the ROM build (`$BLACK_ROM_BUILD/OVY_10.sbin`, loaded at `0x02155160`), or `None` (skip) when it is
/// missing and `REQUIRE_ROM` is not set.
fn overlay10() -> Option<Vec<u8>> {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    match dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) {
        Some(b) => Some(b),
        None => {
            assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
            None
        }
    }
}

/// The embedded tables are the ROM's (`0x021725B8`, `0x021725C0`, `0x021725D8`).
#[test]
fn rom_tables_match() {
    let Some(ov) = overlay10() else { return };
    let at = |a: u32| (a - 0x0215_5160) as usize;
    assert_eq!(&ov[at(0x0217_25B8)..at(0x0217_25C0)], &KIND_TABLE[..]);
    for (k, row) in NOTIFY_TABLE.iter().enumerate() {
        for (i, &f) in row.iter().enumerate() {
            let o = at(0x0217_25C0) + 8 * k + 4 * i;
            assert_eq!(u32::from_le_bytes(ov[o..o + 4].try_into().unwrap()), f);
        }
    }
    for (r, row) in LOST_TABLE.iter().enumerate() {
        let o = at(0x0217_25D8) + 5 * r;
        assert_eq!(&ov[o..o + 5], &row[..]);
    }
}

#[test]
fn lost_reads_the_table() {
    assert_eq!(lost(0, 0), 1);
    assert_eq!(lost(1, 0), 0);
    assert_eq!(lost(3, 3), 1);
    assert_eq!(lost(5, 4), 0);
    assert_eq!(kind_of(0), 1);
    assert_eq!(kind_of(1), 2);
    assert_eq!(kind_of(5), 0);
}
