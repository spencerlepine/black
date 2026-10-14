use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008AEC, "d1544915e70b3b31"),
    (0x02008AF0, "47db10cc25c3728d"),
    (0x02008B20, "483c9d75b19029ba"),
    (0x02008B4C, "7ca6320c2596d16c"),
    (0x02008B78, "db15285d9fca54a9"),
    (0x02008B94, "175d85283a499c67"),
    (0x02008B98, "319c335c4327f268"),
    (0x02008B9C, "162c9fcfaffe675b"),
    (0x02008BD0, "98905f602b8677e3"),
    (0x02008BE0, "242148eaddc7abe7"),
    (0x02008C00, "605ed2945668cbcb"),
];
// fndiff-recorded-end

/// The OS / RTC queries routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl RtcHooks for Rom<'_> {
    fn date_time(&mut self, date: &mut [u8; 16], time: &mut [u8; 12]) {
        self.0
            .call(0x0203_F49C, &mut [Arg::Out(date), Arg::Out(time)]);
    }
    fn to_seconds(&mut self, date: &[u8; 16], time: &[u8; 12]) -> u64 {
        self.0
            .call64(0x0208_9004, &mut [Arg::In(date), Arg::In(time)])
    }
}

impl SysHooks for Rom<'_> {
    fn rtc_offset(&mut self) -> u64 {
        self.0.call64(0x0208_7D08, &mut [])
    }
    fn mac(&mut self, buf: &mut [u8; 6]) {
        self.0.call(0x0208_7C6C, &mut [Arg::Out(buf)]);
    }
    fn owner_info(&mut self, buf: &mut [u8; 0x54]) {
        self.0.call(0x0208_7C88, &mut [Arg::Out(buf)]);
    }
    fn date_to_days(&mut self, date: &[u8; 16]) -> u32 {
        self.0.call(0x0208_8F58, &mut [Arg::In(date)])
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_8AEC => return SysInfo::size() as u64,
        0x0200_8B9C | 0x0200_8BD0 | 0x0200_8BE0 | 0x0200_8C00 => {
            let mut g = GameTime::from_bytes(&c.bufs[0]);
            let h = &mut Rom(&mut c.env);
            let ret = match addr {
                0x0200_8B9C => {
                    g.start(h);
                    0
                }
                0x0200_8BD0 => g.has_penalty() as u64,
                0x0200_8BE0 => {
                    g.count_down(v[1]);
                    0
                }
                _ => {
                    g.clock_changed(h);
                    0
                }
            };
            c.bufs[0] = g.to_bytes();
            return ret;
        }
        _ => {}
    }
    let mut s = SysInfo::from_bytes(&c.bufs[0]);
    let h = &mut Rom(&mut c.env);
    let ret: u64 = match addr {
        0x0200_8AF0 => {
            s.init(h);
            0
        }
        0x0200_8B20 => {
            s.record_owner(h);
            0
        }
        0x0200_8B4C => s.same_mac(h) as u64,
        0x0200_8B78 => s.same_rtc_offset(h) as u64,
        0x0200_8B94 => s.birth_month() as u64,
        0x0200_8B98 => s.birth_day() as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage sysinfo`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "sysinfo",
            include_str!("../../../../tools/rom-diff/specs/black-storage/sysinfo.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn penalty_counts_down() {
    let mut g = GameTime {
        penalty: 5000,
        ..Default::default()
    };
    g.count_down(40);
    assert_eq!(g.penalty, PENALTY - 40);
    g.count_down(5000);
    assert!(!g.has_penalty());
    let s = SysInfo::default();
    assert_eq!(SysInfo::from_bytes(&s.to_bytes()), s);
}
