use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008314, "727bcb4e0305b838"),
    (0x02008320, "6844d0f4c5859cf5"),
    (0x02008374, "fc581891b9f05e8a"),
    (0x0200837C, "f810560489cf26e0"),
    (0x02008390, "4ff60af437561df1"),
    (0x02008398, "1eaad11fe7749e1a"),
    (0x020083AC, "4d1914c8d1ea677d"),
    (0x020083B4, "0ae312e7288df07e"),
    (0x020083C8, "4b694921f4933835"),
    (0x020083D0, "248d185b448e442e"),
    (0x020083E4, "1f9ed7355a060757"),
    (0x020083EC, "add7dc55ee84af31"),
    (0x02008414, "ad1cbfaf25e9610f"),
    (0x02008428, "68c1f802d2cf6fc8"),
    (0x02008430, "cc0045f04369b845"),
    (0x02008448, "75abcf73fc7740d7"),
    (0x02008450, "fcc59c79f16ac7a6"),
];
// fndiff-recorded-end

/// `FUN_020461D4` routed to the spec's stub line.
struct Rom<'a>(&'a mut StubEnv);

impl ConfigHooks for Rom<'_> {
    fn apply_sound_mode(&mut self, mode: u32) {
        self.0.call(0x0204_61D4, &mut [Arg::V(mode)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_8314 {
        let src = Config::from_bytes(&c.bufs[0]);
        let mut dst = Config::from_bytes(&c.bufs[1]);
        Config::copy(&src, &mut dst);
        c.bufs[1] = dst.to_bytes().to_vec();
        return 0;
    }
    let mut cfg = Config::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_8320 => {
            cfg.init();
            0
        }
        0x0200_8374 => cfg.text_speed(),
        0x0200_837C => {
            cfg.set_text_speed(v[1]);
            0
        }
        0x0200_8390 => cfg.button_mode(),
        0x0200_8398 => {
            cfg.set_button_mode(v[1]);
            0
        }
        0x0200_83AC => cfg.flag7(),
        0x0200_83B4 => {
            cfg.set_flag7(v[1]);
            0
        }
        0x0200_83C8 => cfg.flag6(),
        0x0200_83D0 => {
            cfg.set_flag6(v[1]);
            0
        }
        0x0200_83E4 => cfg.sound_mode(),
        0x0200_83EC => {
            cfg.set_sound_mode(v[1], &mut Rom(&mut c.env));
            0
        }
        0x0200_8414 => {
            cfg.apply_sound_mode(&mut Rom(&mut c.env));
            0
        }
        0x0200_8428 => cfg.flag9(),
        0x0200_8430 => {
            cfg.set_flag9(v[1]);
            0
        }
        0x0200_8448 => cfg.flag10(),
        0x0200_8450 => {
            cfg.set_flag10(v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = cfg.to_bytes().to_vec();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage config`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "config",
            include_str!("../../../../tools/rom-diff/specs/black-storage/config.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_matches_init() {
    let c = Config::new_game();
    assert_eq!(c.to_bytes(), [0x01, 0x02, 0, 0]);
    assert_eq!(c.text_speed(), 1);
    assert_eq!(c.flag9(), 1);
}

#[test]
fn image_round_trip() {
    for v in [0u32, 0xFFFF_FFFF, 0x1234_5678, 0x8001_7FFE] {
        let b = v.to_le_bytes();
        assert_eq!(Config::from_bytes(&b).to_bytes(), b);
    }
}
