use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020085A0, "db48e79298f38cef"),
    (0x020085D4, "22072a5eddf8a0c8"),
    (0x0200863C, "3e4a0f0b6c7ee896"),
    (0x0200864C, "d0646d425f0d4fed"),
    (0x02008650, "f51f11ceed22076c"),
    (0x02008654, "0a0c8f0641787905"),
    (0x02008658, "289503aaf406b9d2"),
    (0x020086C8, "2e497cf6f0d2c22e"),
    (0x020086D0, "35e5c083d93db380"),
    (0x020086D8, "452afc778ed864d9"),
    (0x020086E0, "4e49573f532d07b6"),
    (0x020086E8, "db1e0ab2380c82e1"),
    (0x020086F0, "add58520ba0892e2"),
    (0x020086F8, "42e761da75ce4192"),
];
// fndiff-recorded-end

/// The RTC callees routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl PlayTimeHooks for Rom<'_> {
    fn rtc_date(&mut self) -> [u32; 4] {
        let mut b = [0u8; 16];
        self.0.call(0x0208_8864, &mut [Arg::Out(&mut b)]);
        crate::le::r32s(&b, 0)
    }
    fn rtc_time(&mut self) -> [u32; 3] {
        let mut b = [0u8; 12];
        self.0.call(0x0208_890C, &mut [Arg::Out(&mut b)]);
        crate::le::r32s(&b, 0)
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_863C {
        let src = PlayTime::from_bytes(&c.bufs[1]);
        let mut dst = PlayTime::from_bytes(&c.bufs[0]);
        PlayTime::copy(&mut dst, &src);
        c.bufs[0] = dst.to_bytes().to_vec();
        return 0;
    }
    let mut p = PlayTime::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_85A0 => {
            p.init();
            0
        }
        0x0200_85D4 => {
            p.add_seconds(v[1]);
            0
        }
        0x0200_864C => p.hours(),
        0x0200_8650 => p.minutes(),
        0x0200_8654 => p.seconds(),
        0x0200_8658 => {
            p.stamp_now(&mut Rom(&mut c.env));
            0
        }
        0x0200_86C8 => p.stamp_year(),
        0x0200_86D0 => p.stamp_month(),
        0x0200_86D8 => p.stamp_day(),
        0x0200_86E0 => p.stamp_hour(),
        0x0200_86E8 => p.stamp_minute(),
        0x0200_86F0 => {
            c.bufs[1] = p.stamp_word().to_le_bytes().to_vec();
            0
        }
        0x0200_86F8 => {
            p.set_stamp_word(crate::le::r32(&c.bufs[1], 0));
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = p.to_bytes().to_vec();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage playtime`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "playtime",
            include_str!("../../../../tools/rom-diff/specs/black-storage/playtime.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn counter_stops_at_999_59_59() {
    let mut p = PlayTime::new_game();
    assert_eq!(p.to_bytes(), [0; SIZE]);
    p.add_seconds(3725);
    assert_eq!((p.hours(), p.minutes(), p.seconds()), (1, 2, 5));
    p.add_seconds(u32::MAX / 2);
    assert_eq!((p.hours(), p.minutes(), p.seconds()), (999, 59, 59));
    p.add_seconds(1);
    assert_eq!((p.hours(), p.minutes(), p.seconds()), (999, 59, 59));
}
