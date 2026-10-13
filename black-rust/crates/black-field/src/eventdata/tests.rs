use super::*;
use crate::testenv::{ofs_work, run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021625CC, "3fdf6ac27877002c"),
    (0x02162C54, "045d3001fed6ce4e"),
    (0x02162D2C, "aa222acd171682c8"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/eventdata.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0216_25CC => ofs_work(warp_at(&e.mem, v[0], v[1])),
        0x0216_2C54 => warp_exit_dir(&e.mem, v[0]) as u64,
        0x0216_2D2C => warp_contains(&e.mem, v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field eventdata`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("eventdata", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn warp_lookup_and_bounds() {
    let mut mem = Mem::new(WORK, vec![0u8; 0x400]);
    mem.w16(WORK + 0x16, 2);
    assert_eq!(warp_at(&mem, WORK, 0), 0); // no records
    mem.w32(WORK + 0x24, WORK + 0x100);
    assert_eq!(warp_at(&mem, WORK, 1), WORK + 0x114);
    assert_eq!(warp_at(&mem, WORK, 2), 0);
    // a 2-tile wide door at x 0x18 (tile 1 centre), y 0, z 0x28
    let (w, p) = (WORK + 0x100, WORK + 0x200);
    mem.w16(w + 8, 0x18);
    mem.w16(w + 0xC, 0x28);
    mem.w16(w + 0xE, 2);
    mem.w16(w + 0x10, 1);
    mem.w32(p, 0x28 << 12);
    mem.w32(p + 4, 2 << 12);
    mem.w32(p + 8, 0x28 << 12);
    assert!(warp_contains(&mem, w, p));
    mem.w32(p, 0x29 << 12);
    assert!(!warp_contains(&mem, w, p));
    mem.w8(w + 4, 3);
    assert_eq!(warp_exit_dir(&mem, w), 2);
}
