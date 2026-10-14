use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200C3F4, "869794007d619f41"),
    (0x0200C3F8, "0c97fb3ad820651c"),
    (0x0200C418, "04b9a98e4bba7271"),
    (0x0200C44C, "7a351ecc134b9a04"),
    (0x0200C464, "adf14d49ee756e27"),
    (0x0200C468, "0bb8f27b827c50e4"),
    (0x0200C480, "436511afb78328ce"),
    (0x0200C4E8, "9e987ba190d8b2f7"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_C3F4 {
        return Shortcut::size() as u64;
    }
    let mut s = Shortcut::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0200_C3F8 => {
            s.init();
            0
        }
        0x0200_C418 => {
            s.set(v[1] as u8, v[2] != 0);
            0
        }
        0x0200_C44C => s.contains(v[1] as u8) as u64,
        0x0200_C464 => s.get(v[1] as usize) as u64,
        0x0200_C468 => s.count() as u64,
        0x0200_C480 => s.insert(v[1] as u8, v[2]) as u64,
        0x0200_C4E8 => {
            s.pack();
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0][..SIZE].copy_from_slice(&s.to_bytes());
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage shortcut`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "shortcut",
            include_str!("../../../../tools/rom-diff/specs/black-storage/shortcut.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn add_move_remove() {
    let mut s = Shortcut::default();
    s.set(3, true);
    s.set(5, true);
    s.set(3, true);
    assert_eq!((s.count(), s.get(0), s.get(1)), (2, 3, 5));
    assert_eq!(s.insert(5, 0), 0);
    assert_eq!((s.get(0), s.get(1)), (5, 3));
    s.set(5, false);
    assert_eq!((s.count(), s.get(0), s.contains(5)), (1, 3, false));
}
