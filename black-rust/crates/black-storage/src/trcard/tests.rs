use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020089A8, "daf55a95bd9d8f4d"),
    (0x020089B0, "2bb21c1df0cb6819"),
    (0x020089CC, "a46234fa603f8127"),
    (0x020089D0, "62e3a3212c6a3815"),
    (0x020089F4, "e392cc4a3bfc45a9"),
    (0x02008A00, "f99600c864f75120"),
    (0x02008A0C, "85cf72d4089a038a"),
    (0x02008A14, "14533a37a1e785fd"),
    (0x02008A1C, "a5348d09f32c2b57"),
    (0x02008A28, "f853ad5cc0740a3a"),
    (0x02008A34, "6a58adf949bc6ffe"),
    (0x02008A44, "644d82e725e3bdec"),
    (0x02008A54, "83f2fedbcb20c852"),
    (0x02008A7C, "1e42390b5aac6552"),
    (0x02008ACC, "4158fbd912b11b8a"),
    (0x02008ADC, "8f00ae0edfe441cd"),
];
// fndiff-recorded-end

/// The RTC calls routed to the spec's stub lines.
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

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_89A8 {
        return TrCard::size() as u64;
    }
    let mut t = TrCard::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0200_89B0 => {
            t.init();
            0
        }
        0x0200_89CC => {
            let _ = t.sign();
            0
        }
        0x0200_89D0 => t.is_signed() as u64,
        0x0200_89F4 => t.b602() as u64,
        0x0200_8A00 => {
            t.set_b602();
            0
        }
        0x0200_8A0C => t.w600() as u64,
        0x0200_8A14 => {
            t.set_w600(v[1] as u16);
            0
        }
        0x0200_8A1C => {
            t.set_b603(v[1] as u8);
            0
        }
        0x0200_8A28 => t.b603() as u64,
        0x0200_8A34 => {
            t.set_time(v[1] as u64 | (v[2] as u64) << 32);
            0
        }
        0x0200_8A44 => t.time(),
        0x0200_8A54 => t.entry(v[1] as usize) as u64,
        0x0200_8A7C => {
            t.set_entry(
                v[1] as usize,
                v[2] as u8,
                v[3] as u8,
                v[4] as u8,
                &mut Rom(&mut c.env),
            );
            0
        }
        0x0200_8ACC => t.half(v[1] as usize) as u64,
        0x0200_8ADC => {
            t.set_half(v[1] as usize, v[2] as u16);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = t.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage trcard`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "trcard",
            include_str!("../../../../tools/rom-diff/specs/black-storage/trcard.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn blank_signature() {
    let mut t = TrCard::default();
    assert!(!t.is_signed());
    t.sign[0x5FF] = 1;
    assert!(t.is_signed());
    assert_eq!(TrCard::from_bytes(&t.to_bytes()), t);
}
