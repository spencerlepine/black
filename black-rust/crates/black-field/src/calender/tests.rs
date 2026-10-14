use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021647A0, "5d6c668f1fb2a243"),
    (0x021647D8, "bb97aba2f0180775"),
    (0x021647E8, "e814efdda5b969b1"),
    (0x021647F0, "f2599977f866f574"),
    (0x021647FC, "77e1bfdb161de606"),
    (0x0216480C, "20debd6f36e5b111"),
    (0x0216481C, "cb9fc71391532f03"),
    (0x02164844, "538aead1754ec789"),
    (0x02164890, "150774c3e0596856"),
    (0x021648A8, "7a98e905fa32fb38"),
    (0x021648CC, "2d359d498461f709"),
    (0x021649C8, "6b96d1d4a0c4a3ff"),
    (0x02164A2C, "cfe7ca3445e2b385"),
    (0x02164B38, "5fce7dfbd0e825e1"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/calender.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0216_47A0 => new(e, v[0], v[1]) as u64,
        0x0216_47D8 => {
            delete(e, v[0]);
            0
        }
        0x0216_47E8 => today_thunk(e, v[0], v[1]) as u64,
        0x0216_47F0 => {
            clear(e, v[0]);
            0
        }
        0x0216_47FC => {
            open(e, v[0]);
            0
        }
        0x0216_480C => {
            close(e, v[0]);
            0
        }
        0x0216_481C => today(e, v[0], v[1]) as u64,
        0x0216_4844 => special_day(e, v[0], v[1], v[2], v[3]) as u64,
        0x0216_4890 => {
            read_records(e, v[0]);
            0
        }
        0x0216_48A8 => zone_offset(e, v[0], v[1]) as u64,
        0x0216_48CC => day_of_year(v[0], v[1]) as u64,
        0x0216_49C8 => cmd467_end(e, v[0]) as u64,
        0x0216_4A2C => event_proc(e, v[0], v[1], v[2]) as u64,
        0x0216_4B38 => event_new(e, v[0], v[2]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field calender`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("calender", SPEC, PORTED, RECORDED), &call);
}

/// The embedded month-length table is the ROM's (`0x02171C20`, overlay 10 at `0x02155160`).
#[test]
fn rom_tables_match() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(ov) = dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
        return;
    };
    let o = (0x0217_1C20 - 0x0215_5160) as usize;
    assert_eq!(&ov[o..o + 13], &MONTH_DAYS[..]);
}

#[test]
fn days_of_the_year() {
    assert_eq!(day_of_year(1, 1), 0);
    assert_eq!(day_of_year(3, 1), 60); // February has 29 days in the table
    assert_eq!(day_of_year(13, 1), 366);
    assert_eq!(day_of_year(0, 0), 0xFFFF);
    assert_eq!(day_of_year(0xFFFF_FFFF, 5), 4);
}
