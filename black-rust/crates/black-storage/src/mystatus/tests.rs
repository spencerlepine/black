use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008468, "f39d9f48dea4c88d"),
    (0x02008494, "9ec015b5aaa25548"),
    (0x020084A0, "d9c0eb5bc4e75b25"),
    (0x020084BC, "8db555999b2b5f11"),
    (0x020084F4, "0697b0f85a65bd94"),
    (0x0200852C, "fdf58628f6101422"),
    (0x02008530, "f78fea72620ab083"),
    (0x02008534, "d01c1630840a4a8a"),
    (0x0200853C, "ef116986ac638087"),
    (0x02008540, "5920071452226838"),
    (0x0200854C, "061bc1aa04e9361e"),
    (0x02008550, "36538b8ca0380cad"),
    (0x02008554, "77718d26bc733981"),
    (0x02008558, "fa31c140201daf25"),
    (0x0200855C, "65389eecca5c64a6"),
    (0x02008560, "4c19727f35c8d584"),
    (0x02008564, "f796775c8a7a785d"),
    (0x02008568, "0bc639f64cc9027d"),
    (0x0200856C, "53b163fcc3c26513"),
    (0x02008570, "495bad804acf3a79"),
    (0x02008574, "eff39d229afd32c2"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_8468 => return MyStatus::size() as u64,
        0x0200_8494 => {
            let src = MyStatus::from_bytes(&c.bufs[0]);
            let mut dst = MyStatus::from_bytes(&c.bufs[1]);
            MyStatus::copy(&src, &mut dst);
            c.bufs[1] = dst.to_bytes().to_vec();
            return 0;
        }
        _ => {}
    }
    let mut s = MyStatus::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_84A0 => {
            s.init();
            0
        }
        0x0200_84BC => s.name_is_empty() as u32,
        // the name is at offset 0 of the block
        0x0200_84F4 => {
            let _ = s.name();
            0
        }
        0x0200_852C => {
            s.set_id(v[1]);
            0
        }
        0x0200_8530 => s.id(),
        0x0200_8534 => s.public_id(),
        0x0200_853C => s.sync_id(),
        0x0200_8540 => {
            s.set_sync_id_once(v[1]);
            0
        }
        0x0200_854C => {
            s.set_gender(v[1]);
            0
        }
        0x0200_8550 => s.gender(),
        0x0200_8554 => s.field_1c(),
        0x0200_8558 => {
            s.set_field_1c(v[1]);
            0
        }
        0x0200_855C => s.version(),
        0x0200_8560 => {
            s.set_version(v[1]);
            0
        }
        0x0200_8564 => s.language(),
        0x0200_8568 => {
            s.set_language(v[1]);
            0
        }
        0x0200_856C => s.country(),
        0x0200_8570 => s.region(),
        0x0200_8574 => {
            s.set_country_region(v[1], v[2]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes().to_vec();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage mystatus`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "mystatus",
            include_str!("../../../../tools/rom-diff/specs/black-storage/mystatus.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_matches_init() {
    let s = MyStatus::new_game();
    let mut want = [0u8; SIZE];
    want[0x1A] = 2;
    want[0x1B] = 0x15;
    assert_eq!(s.to_bytes(), want);
    assert!(s.name_is_empty());
}

#[test]
fn image_round_trip() {
    let b: Vec<u8> = (0..SIZE as u8).map(|i| i.wrapping_mul(37) ^ 0xA5).collect();
    assert_eq!(MyStatus::from_bytes(&b).to_bytes().to_vec(), b);
}
