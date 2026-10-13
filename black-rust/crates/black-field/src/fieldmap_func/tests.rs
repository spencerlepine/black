use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x021BBBD8, "9e53e76363c37cdd")];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        // `ret=ofs:arg0`: the port reports the offset into the ring buffer
        0x021B_BBD8 => ring_take(&mut c.bufs[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field fieldmap_func`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "fieldmap_func",
            include_str!("../../../../tools/rom-diff/specs/black-field/fieldmap_func.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn ring_wraps_and_pushes_the_oldest() {
    let mut ring = vec![0u8; 0x444];
    assert_eq!(ring_take(&mut ring), 0);
    assert_eq!((ring[RING_NEXT], ring[RING_OLDEST]), (1, 0));
    ring[RING_NEXT] = 29;
    assert_eq!(ring_take(&mut ring), 29 * RING_ENTRY);
    assert_eq!(ring[RING_NEXT], 0);
    // full ring: next == oldest with entries pushes the oldest on
    ring[RING_COUNT] = 30;
    ring[RING_OLDEST] = 0;
    assert_eq!(ring_take(&mut ring), 0);
    assert_eq!((ring[RING_NEXT], ring[RING_OLDEST]), (1, 1));
}
