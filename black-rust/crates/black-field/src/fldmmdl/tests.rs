use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0216D4F0, "b52340c0d3950151"),
    (0x0216D55C, "079615e140f95902"),
    (0x0216D5F8, "283f3ed95c1be88c"),
    (0x0216D67C, "4250f23b5d2a364a"),
    (0x0216DD1C, "d7636c31b57a843c"),
    (0x0216E4B0, "dde00de11a994a2a"),
];
// fndiff-recorded-end

/// Address of the spec's `work` arena (MMDLSYS at +0, MMDL[0..8] from +0x100).
const WORK: u32 = 0x02E0_0000;

/// Runs `f` on the work image as a [`Mem`] window at its original address.
fn with_work<R>(c: &mut Case, f: impl FnOnce(&mut Mem) -> R) -> R {
    let mut mem = Mem::new(WORK, std::mem::take(&mut c.work));
    let r = f(&mut mem);
    c.work = mem.bytes;
    r
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0216_D4F0 => with_work(c, |m| count(m, v[0])) as u64,
        0x0216_D55C => with_work(c, |m| status(m, v[0], v[1])) as u64,
        0x0216_D5F8 => with_work(c, |m| dir_disp(m, v[0])) as u64,
        0x0216_D67C => with_work(c, |m| sys_of(m, v[0])) as u64,
        0x0216_DD1C => with_work(c, |m| next_in_use(m, v[0], v[1], v[2])) as u64,
        0x0216_E4B0 => save_slot(v[0], v[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field fldmmdl`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "fldmmdl",
            include_str!("../../../../tools/rom-diff/specs/black-field/fldmmdl.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn iterator_skips_free_objects() {
    let mut mem = Mem::new(WORK, vec![0u8; 0x1000]);
    mem.w16(WORK + 4, 3); // three MMDLs
    mem.w32(WORK + 0x18, WORK + 0x100);
    mem.w32(WORK + 0x200, ST_IN_USE); // MMDL 1 in use
    let (out, idx) = (WORK + 0x800, WORK + 0x804);
    assert!(next_in_use(&mut mem, WORK, out, idx));
    assert_eq!((mem.r32(out), mem.r32(idx)), (WORK + 0x200, 2));
    assert!(!next_in_use(&mut mem, WORK, out, idx));
    assert_eq!(mem.r32(idx), 3);
}

#[test]
fn save_slots_by_zone_and_id() {
    assert_eq!(save_slot(0x14D, 1), 2);
    assert_eq!(save_slot(0x15A, 5), 25);
    assert_eq!(save_slot(0x14D, 3), NO_SLOT);
    assert_eq!(save_slot(0x1_014D, 1), NO_SLOT);
}
