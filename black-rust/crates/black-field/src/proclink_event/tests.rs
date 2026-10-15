use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216075C, "82dea0c784d51d5a"),
    (0x021607E0, "2427059c58a29253"),
    (0x02160A28, "0a3f7c0535aa1265"),
    (0x02160A4C, "7dcda46c99c937ae"),
    (0x02160A64, "b78db03aac55e2ca"),
    (0x02160A7C, "782db5f54d845ae7"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/proclink_event.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let r = match addr {
            0x0216_075C => new(e, v[0], v[1]),
            0x0216_07E0 => proc_(e, v[0], v[1], v[2]),
            0x0216_0A28 => {
                rec_init(e, v[0], v[1], v[2], v[3], v[4], v[5]);
                0
            }
            0x0216_0A4C => rec_a(e, v[0]),
            0x0216_0A64 => rec_b(e, v[0]),
            0x0216_0A7C => rec_c(e, v[0]),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        };
        u64::from(r)
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field proclink_event`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("proclink_event", SPEC, PORTED, RECORDED), &call);
}

/// The table words the spec fakes are the real ones' layout: [`TABLE`] holds 12 entries whose `init` / `return` / `free`
/// callbacks are the callbacks of `proclink.rs` and whose proc data is non-null (ROM 0x021712CC, overlay 10 at
/// 0x02155160).
#[test]
fn table_matches_rom() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(ov) = dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
        return;
    };
    let word = |a: u32| {
        let o = (a - 0x0215_5160) as usize;
        u32::from_le_bytes(ov[o..o + 4].try_into().unwrap())
    };
    // the first entry is the party list: init 0x02160A94, return 0x02160C88
    assert_eq!(word(TABLE + 8), 0x0216_0A95);
    assert_eq!(word(TABLE + 0xC), 0x0216_0C89);
    // entry 4 is the "report" entry: only a `special` callback (`report_special`, 0x02161350); every other entry has
    // an init / return in the proclink file and no `special`
    assert_eq!(word(TABLE + 4 * ENTRY + 0x10), 0x0216_1351);
    for k in (0..12).filter(|&k| k != 4) {
        let e = TABLE + k * ENTRY;
        assert_eq!(word(e + 0x10), 0, "entry {k} special");
        for off in [8, 0xC] {
            let w = word(e + off);
            assert!(
                w & 1 == 1 && (0x0216_0A94..0x0216_1D98).contains(&w),
                "entry {k} +{off:#x}: {w:#x}"
            );
        }
    }
}
