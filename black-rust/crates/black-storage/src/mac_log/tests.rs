use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200E8E8, "d1544915e70b3b31"),
    (0x0200E8EC, "45a44f48558de7fd"),
    (0x0200E904, "960d30d7a93dffed"),
    (0x0200E918, "3a727f5f93c157dd"),
    (0x0200E920, "2b20de0767db6f73"),
    (0x0200E938, "399fb104098d26d9"),
    (0x0200E958, "98d19c69b18cda47"),
    (0x0200E978, "da78a07fb5c66234"),
    (0x0200E994, "4c9efb898fb03cd6"),
    (0x0200E9DC, "cafa4924c9a9b7ed"),
    (0x0200E9E0, "a7472bc5e5e5185b"),
    (0x0200EA00, "123f61b2de98fb0c"),
    (0x0200EA04, "846095814d6dcfc3"),
];
// fndiff-recorded-end

/// The OS calls routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl MacHooks for Rom<'_> {
    fn mac(&mut self, buf: &mut [u8; 6]) {
        self.0.call(0x0208_7C6C, &mut [Arg::Out(buf)]);
    }
    fn compare(&mut self, a: &[u8; 6], b: &[u8; 6]) -> u32 {
        self.0
            .call(0x0203_EFC8, &mut [Arg::In(a), Arg::In(b), Arg::V(6)])
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_E8E8 => return TrialHouse::size() as u64,
        0x0200_E9DC => return MatchLog::size() as u64,
        0x0200_E8EC => {
            let mut t = TrialHouse::from_bytes(&c.bufs[0]);
            t.init();
            c.bufs[0] = t.to_bytes();
            return 0;
        }
        0x0200_E9E0 | 0x0200_EA00 | 0x0200_EA04 => {
            let mut m = MatchLog::from_bytes(&c.bufs[0]);
            let ret = if addr == 0x0200_E9E0 {
                m.init(&mut Rom(&mut c.env));
                0
            } else {
                let _ = m.counters();
                LOG_SIZE as u64
            };
            c.bufs[0] = m.to_bytes();
            return ret;
        }
        _ => {}
    }
    let mut l = MacLog::from_bytes(&c.bufs[0]);
    let h = &mut Rom(&mut c.env);
    let ret: u64 = match addr {
        0x0200_E904 => {
            l.init(h);
            0
        }
        0x0200_E918 => l.count(v[1] as usize) as u64,
        0x0200_E920 => {
            l.add_count(v[1] as usize, v[2]);
            0
        }
        0x0200_E938 => {
            c.bufs[2][..6].copy_from_slice(&l.entry_mac(v[1]));
            0
        }
        0x0200_E958 => {
            l.set_entry_mac(v[1], &c.bufs[2].clone());
            0
        }
        0x0200_E978 => {
            l.set_entry_byte(v[1], v[2] as usize, v[3] as u8);
            0
        }
        0x0200_E994 => {
            let mac: [u8; 6] = c.bufs[1][..6].try_into().unwrap();
            l.contains(&mac, h) as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0][..LOG_SIZE].copy_from_slice(&l.to_bytes());
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage mac_log`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "mac_log",
            include_str!("../../../../tools/rom-diff/specs/black-storage/mac_log.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

/// Plain byte compare.
struct Eq6;

impl MacHooks for Eq6 {
    fn mac(&mut self, buf: &mut [u8; 6]) {
        *buf = [1, 2, 3, 4, 5, 6];
    }
    fn compare(&mut self, a: &[u8; 6], b: &[u8; 6]) -> u32 {
        (a != b) as u32
    }
}

#[test]
fn log_entries() {
    let mut l = MacLog::default();
    l.init(&mut Eq6);
    l.set_entry_mac(13, &[9; 6]);
    l.add_count(0, 300);
    assert_eq!(l.count(0), 255);
    assert!(l.contains(&[9; 6], &mut Eq6));
    l.add_count(0, (-260i32) as u32);
    assert!(!l.contains(&[9; 6], &mut Eq6));
    let m = MatchLog::default();
    assert_eq!(MatchLog::from_bytes(&m.to_bytes()), m);
}
