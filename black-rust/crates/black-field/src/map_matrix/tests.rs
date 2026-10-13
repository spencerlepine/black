use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02159BC4, "b5615728646bbd48"),
    (0x02159D68, "a09289ab0a748193"),
];
// fndiff-recorded-end

/// Address of the spec's `work` arena (the MAP MATRIX record lives at its start).
const WORK: u32 = 0x02E0_0000;

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0215_9D68 => block_of(c.vals[0] as i32) as u32 as u64,
        0x0215_9BC4 => {
            let mem = Mem::new(WORK, std::mem::take(&mut c.work));
            let r = zone_at(&mem, c.vals[0], c.vals[1], c.vals[2]);
            c.work = mem.bytes;
            r as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field map_matrix`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "map_matrix",
            include_str!("../../../../tools/rom-diff/specs/black-field/map_matrix.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn blocks_round_toward_zero() {
    assert_eq!(block_of(0), 0);
    assert_eq!(block_of(0x1F_FFFF), 0);
    assert_eq!(block_of(0x20_0000), 1);
    assert_eq!(block_of(-1), 0);
    assert_eq!(block_of(-0x20_0000), -1);
    assert_eq!(block_of(-0x20_0001), -1);
}

#[test]
fn zone_cells_are_row_major() {
    let mut bytes = vec![0u8; MATRIX_SIZE as usize];
    bytes[8] = 3; // width 3
    let cell = 2 + 3 * 3;
    bytes[(ZONE_OFS + 2 * cell) as usize] = 0x34;
    bytes[(ZONE_OFS + 2 * cell + 1) as usize] = 0x12;
    let mem = Mem::new(WORK, bytes);
    assert_eq!(zone_at(&mem, WORK, 2, 3), 0x1234);
    assert_eq!(zone_at(&mem, WORK, 0, 0), 0);
}
