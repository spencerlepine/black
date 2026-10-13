use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x0218AE48, "dbde03b23eab1ea5")];
// fndiff-recorded-end

/// Address of the spec's `work` arena.
const WORK: u32 = 0x02E0_0000;

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0218_AE48 => {
            let mut mem = Mem::new(WORK, std::mem::take(&mut c.work));
            front_position(&mut mem, c.vals[0], c.vals[1]);
            c.work = mem.bytes;
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field fieldmap_event`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "fieldmap_event",
            include_str!("../../../../tools/rom-diff/specs/black-field/fieldmap_event.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn front_tile_moves_one_unit() {
    let mut bytes = vec![0u8; 0x100];
    bytes[REQ_POS as usize..REQ_POS as usize + 4].copy_from_slice(&(WORK + 0x40).to_le_bytes());
    for (i, v) in [0x0002_0000u32, 0x10, 0x0003_0000].iter().enumerate() {
        bytes[0x40 + 4 * i..0x44 + 4 * i].copy_from_slice(&v.to_le_bytes());
    }
    let mut mem = Mem::new(WORK, bytes);
    mem.w16(WORK + REQ_FACING, 2);
    front_position(&mut mem, WORK, WORK + 0x80);
    assert_eq!(
        (
            mem.r32(WORK + 0x80),
            mem.r32(WORK + 0x84),
            mem.r32(WORK + 0x88)
        ),
        (0x0001_0000, 0x10, 0x0003_0000)
    );
    mem.w16(WORK + REQ_FACING, 1);
    front_position(&mut mem, WORK, WORK + 0x80);
    assert_eq!(mem.r32(WORK + 0x88), 0x0004_0000);
}
