use super::*;
use crate::testenv::{ofs_work, run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021C939C, "a84f4fc01e7b7344"),
    (0x021C93D0, "3be231d67f39d0e0"),
    (0x021C93E4, "7c64e3d56b21d989"),
    (0x021C9450, "a942de302aa5722d"),
    (0x021C94AC, "2cd30edea097d968"),
    (0x021C94B4, "8e9ea5ef58e98df9"),
];
// fndiff-recorded-end

const SPEC: &str =
    include_str!("../../../../tools/rom-diff/specs/black-field/land_data_patch.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021C_939C => ofs_work(create(e, v[0], v[1])),
        0x021C_93D0 => {
            free(e, v[0]);
            0
        }
        0x021C_93E4 => {
            copy_attributes(e, v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7]);
            0
        }
        0x021C_9450 => place_buildings(e, v[0], v[1], v[2], v[3], v[4], v[5]) as u64,
        // `ret=ofs:work`: the port reports the offset into the arena
        0x021C_94AC => ofs_work(attr_section(&e.mem, v[0])),
        0x021C_94B4 => ofs_work(build_section(&e.mem, v[0])),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field land_data_patch`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new("land_data_patch", SPEC, PORTED, RECORDED),
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
