use super::*;
use black_fndiff::{run_suite, Arg, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008700, "456e3f750c0549b9"),
    (0x02008704, "d3ffd171fba1e635"),
    (0x02008730, "7d6333040a63cd9f"),
    (0x0200873C, "62ac4cd16ae54c55"),
    (0x02008748, "31d5c432fc65f387"),
    (0x02008754, "1b07a6e3a770bdac"),
    (0x02008764, "6c4962d446526409"),
    (0x02008774, "a3dc4cc165cb3ecd"),
    (0x02008784, "09af596049825622"),
    (0x020087A8, "061cfe089a9b3954"),
    (0x020087C0, "26e58101a54bc2bd"),
    (0x020087D4, "22ae0c5f5f71b5c5"),
    (0x020087E8, "4f65ac4cc4e452cc"),
    (0x02008808, "6ea7b530e1a28cb4"),
];
// fndiff-recorded-end

/// Fixed emulator address of pointer argument `k`.
fn arg_addr(k: u32) -> u32 {
    0x02E1_0000 + k * 0x10000
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_8700 => return SaveInfo::size() as u64,
        0x0200_8704 => {
            let mut s = SaveInfo::from_bytes(&c.bufs[0]);
            s.init();
            c.bufs[0] = s.to_bytes().to_vec();
            return 0;
        }
        _ => {}
    }
    // The block argument: arg1 for the one-argument accessors, else arg2.
    let k = if v.len() == 2 { 1 } else { 2 };
    // FUN_020071CC(sav, 27): the block fetch the port replaces by ownership (logged by the stub).
    let got = c
        .env
        .call(0x0200_71CC, &mut [Arg::V(v[0]), Arg::V(BLOCK_ID)]);
    assert_eq!(got, arg_addr(k));
    let mut s = SaveInfo::from_bytes(&c.bufs[k as usize]);
    let ret = match addr {
        0x0200_8730 => {
            let _ = s.my_status_mut();
            MYSTATUS_OFS
        }
        0x0200_873C => {
            let _ = s.config_mut();
            0
        }
        0x0200_8748 => {
            let _ = s.play_time_mut();
            PLAYTIME_OFS
        }
        0x0200_8754 => s.flag(),
        0x0200_8764 => {
            s.set_flag();
            0
        }
        0x0200_8774 => {
            s.clear_flag();
            0
        }
        0x0200_8784 => s.extra_marked(v[1] as usize) as u32,
        0x0200_87A8 => {
            s.mark_extra(v[1] as usize);
            0
        }
        0x0200_87C0 => {
            s.unmark_extra(v[1] as usize);
            0
        }
        0x0200_87D4 => {
            let _ = s.extra_word_mut(v[1] as usize);
            0x44 + 4 * v[1]
        }
        0x0200_87E8 => {
            let src = MyStatus::from_bytes(&c.bufs[1][0x20..0x40]);
            s.load_my_status(&src);
            0
        }
        0x0200_8808 => {
            let mut dst = MyStatus::from_bytes(&c.bufs[1][0x20..0x40]);
            s.store_my_status(&mut dst);
            c.bufs[1][0x20..0x40].copy_from_slice(&dst.to_bytes());
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[k as usize] = s.to_bytes().to_vec();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage save_info`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "save_info",
            include_str!("../../../../tools/rom-diff/specs/black-storage/save_info.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_matches_init() {
    let b = SaveInfo::new_game().to_bytes();
    let mut want = [0u8; SIZE];
    want[0] = 1; // config: text speed 1
    want[1] = 2; // config: bit 9
    want[4 + 0x1A] = 2; // mystatus: language
    want[4 + 0x1B] = 0x15; // mystatus: version
    assert_eq!(b, want);
}

#[test]
fn image_round_trip() {
    let b: Vec<u8> = (0..SIZE as u32).map(|i| (i * 91 + 7) as u8).collect();
    assert_eq!(SaveInfo::from_bytes(&b).to_bytes().to_vec(), b);
}
