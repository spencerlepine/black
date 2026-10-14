use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200BC10, "fcea3105ba126118"),
    (0x0200BC20, "a243ffc8438fd940"),
    (0x0200BC2C, "2b8d631e83be4e73"),
    (0x0200BC4C, "1b2cdb7b4369662e"),
    (0x0200BC88, "1d5efbd1726d3ce0"),
    (0x0200BCA0, "d09029407243698b"),
    (0x0200BCA8, "3277b09ec6f46664"),
    (0x0200BCB8, "9d48dad6f302405a"),
    (0x0200BCD8, "d64119e1f2e2b3e4"),
    (0x0200BCEC, "f028c445be5be386"),
    (0x0200BD48, "6b3dcff00cd9069f"),
    (0x0200BD98, "71d01d82ed2f6f81"),
    (0x0200BDA0, "93fcd5b70ae594f9"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_BD98 {
        return Daycare::size() as u64;
    }
    let mut d = Daycare::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0200_BC10 => {
            d.set_egg();
            0
        }
        0x0200_BC20 => {
            d.clear_egg();
            0
        }
        0x0200_BC2C => {
            let _ = d.mon(v[1]);
            (d.mon_slot(v[1]) * SLOT_SIZE + 4) as u64
        }
        0x0200_BC4C => {
            let mon = c.bufs[2].clone();
            d.put(v[1], &mon);
            0
        }
        0x0200_BC88 => {
            d.take(v[1]);
            0
        }
        0x0200_BCA0 => d.egg() as u64,
        0x0200_BCA8 => d.occupied(v[1]) as u64,
        0x0200_BCB8 => d.count() as u64,
        0x0200_BCD8 => d.exp(v[1]) as u64,
        0x0200_BCEC => {
            d.swap(v[1], v[2]);
            0
        }
        0x0200_BD48 => {
            d.add_exp(v[1]);
            0
        }
        0x0200_BDA0 => {
            d.init();
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = d.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage daycare`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "daycare",
            include_str!("../../../../tools/rom-diff/specs/black-storage/daycare.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn put_take_and_exp() {
    let mut d = Daycare::default();
    d.put(1, &[7; MON_SIZE]);
    assert_eq!((d.count(), d.occupied(1), d.mon_slot(1)), (1, 1, 1));
    d.put(1, &[9; MON_SIZE]);
    assert_eq!(d.mon(1)[0], 7);
    d.add_exp(u32::MAX - 1);
    d.add_exp(5);
    assert_eq!(d.exp(1), u32::MAX);
    d.swap(0, 1);
    assert_eq!(d.occupied(0), 1);
    d.take(0);
    assert_eq!(d.count(), 0);
    assert_eq!(Daycare::from_bytes(&d.to_bytes()), d);
}
