use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A19E8, "d0091d4fa58bb400"),
    (0x021A1AAC, "76d4bb4712b8b2f7"),
    (0x021A1AF4, "47600f569fde65d0"),
    (0x021A1B38, "19f8a552d1b17bbc"),
    (0x021A1B50, "6838fb3b7ede3e5c"),
    (0x021A1B74, "a8196a0e4ad559aa"),
    (0x021A1BDC, "30d4ad572e524dca"),
    (0x021A1BE4, "b9d4dd55cc4159c3"),
    (0x021A1BF4, "deca1f679ad1450e"),
    (0x021A1BF8, "9fe1522b4399bd61"),
    (0x021A1C04, "d06dab59600de928"),
    (0x021A1C1C, "9f4ca4af6504dc85"),
    (0x021A1C34, "7b0290e26bd2117f"),
    (0x021A1C68, "31b945086fc45d3b"),
    (0x021A1D04, "47a2dde7b12c337f"),
    (0x021A1D78, "8154c8bb736a1352"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/weather.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        match addr {
            0x021A_19E8 => return new(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6]) as u64,
            0x021A_1AAC => delete(e, v[0]),
            0x021A_1AF4 => update(e, v[0], v[1]),
            0x021A_1B38 => draw(e, v[0]),
            0x021A_1B50 => return starting(e, v[0]) as u64,
            0x021A_1B74 => set(e, v[0], v[1], v[2]),
            0x021A_1BDC => return request_thunk(e, v[0], v[1]) as u64,
            0x021A_1BE4 => request_quiet(e, v[0], v[1]),
            0x021A_1BF4 => return current(e, v[0]) as u64,
            0x021A_1BF8 => load_overlay(e, v[0], v[1]),
            0x021A_1C04 => unload_overlay(e, v[0]),
            0x021A_1C1C => vblank(e, v[1]),
            0x021A_1C34 => return request(e, v[0], v[1]) as u64,
            0x021A_1C68 => change_plain(e, v[0], v[1]),
            0x021A_1D04 => change_crossfade(e, v[0], v[1]),
            0x021A_1D78 => return crossfade_pair(v[0], v[1]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
        0
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field weather`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("weather", SPEC, PORTED, RECORDED), &call);
}

/// The embedded weather table is the ROM's (`0x021D8868`, overlay 21 at `0x02187F00`), and so is the change-proc table
/// `0x021DDE6C`.
#[test]
fn rom_tables_match() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    for (i, &(d, o)) in WEATHERS.iter().enumerate() {
        let a = WEATHER_TABLE + 8 * i as u32;
        assert_eq!((w(a), w(a + 4)), (d, o), "weather {i}");
    }
    assert_eq!((w(0x021D_DE6C), w(0x021D_DE70)), (0x021A_1C69, 0x021A_1D05));
}

#[test]
fn crossfade_pairs() {
    let pairs: Vec<(u32, u32)> = (0..15)
        .flat_map(|c| (0..15).map(move |n| (c, n)))
        .filter(|&(c, n)| crossfade_pair(c, n) == 1)
        .collect();
    assert_eq!(pairs, [(1, 4), (1, 5), (4, 1), (4, 5), (5, 1), (5, 4)]);
    assert_eq!(crossfade_pair(1, 0x1_0004), 1);
    assert_eq!(crossfade_pair(4, 0x1_0001), 0);
}
