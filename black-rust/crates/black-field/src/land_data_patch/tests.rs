use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[(0x021C94AC, "2cd30edea097d968")];
// fndiff-recorded-end

/// Address of the spec's `work` arena (the patch object lives at its start).
const WORK: u32 = 0x02E0_0000;

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        // `ret=ofs:work`: the port reports the offset into the arena
        0x021C_94AC => {
            let mem = Mem::new(WORK, std::mem::take(&mut c.work));
            let r = attr_section(&mem, c.vals[0]);
            c.work = mem.bytes;
            r.wrapping_sub(WORK) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-field land_data_patch`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "land_data_patch",
            include_str!("../../../../tools/rom-diff/specs/black-field/land_data_patch.spec"),
            PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn attr_section_is_file_plus_offset() {
    let mut bytes = vec![0u8; 0x200];
    bytes[0..4].copy_from_slice(&(WORK + 0x100).to_le_bytes());
    bytes[0x100] = 0x40;
    let mem = Mem::new(WORK, bytes);
    assert_eq!(attr_section(&mem, WORK), WORK + 0x140);
}
