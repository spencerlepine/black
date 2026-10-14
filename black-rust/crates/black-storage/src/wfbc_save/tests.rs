use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200A840, "e20026c75fbf9fc9"),
    (0x0200A848, "69685130b807f754"),
    (0x0200A870, "5393dbe2a1234179"),
    (0x0200A884, "d84b78c2799105ab"),
    (0x0200A890, "09bbab5f40d102a1"),
    (0x0200A898, "7631fafbc51ab355"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    if addr == 0x0200_A840 {
        return WfbcSave::size() as u64;
    }
    let mut w = WfbcSave::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_A848 => {
            w.init();
            0
        }
        0x0200_A870 => {
            w.load_town(&c.bufs[1].clone());
            0
        }
        0x0200_A884 => {
            w.store_town(&mut c.bufs[1]);
            0
        }
        0x0200_A890 => {
            let _ = w.ids();
            IDS_OFS as u64
        }
        0x0200_A898 => {
            let _ = w.tail();
            TAIL_OFS as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = w.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage wfbc_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "wfbc_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/wfbc_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn init_defaults() {
    let mut w = WfbcSave::default();
    w.init();
    assert_eq!((w.town[2], w.ids[9], w.tail[2]), (2, 0xFFFF, 10));
    assert_eq!(WfbcSave::from_bytes(&w.to_bytes()), w);
}
