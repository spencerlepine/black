use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200C054, "7e98dd46f63b3c6d"),
    (0x0200C058, "3ce3db3bb09917c4"),
    (0x0200C0FC, "53a94068aa70bdfe"),
    (0x0200C104, "14d386a6c2dc518d"),
    (0x0200C10C, "5d436826612cd204"),
    (0x0200C11C, "e229341fcb9f20ce"),
    (0x0200C164, "447be5ebb063f21e"),
    (0x0200C184, "cd4bb06b0ddeee0e"),
    (0x0200C18C, "a4ea791311bafd2b"),
    (0x0200C1A4, "21b4f32e58a17674"),
    (0x0200C1B0, "68c47c68002ec2cf"),
    (0x0200C1C4, "14cfb818f9ea0be1"),
    (0x0200C1C8, "c4b15caa513b56e8"),
    (0x0200C1DC, "b3da3cf41d395b34"),
    (0x0200C1E0, "dfd808914e4ff1ad"),
    (0x0200C1F4, "124501b95b451403"),
    (0x0200C20C, "70d4d9c5acd14200"),
    (0x0200C224, "35f00ac5797fd745"),
    (0x0200C22C, "c5ba1590b9dbf43c"),
    (0x0200C234, "66b7f1def4cdc110"),
    (0x0200C248, "7dd3353e70b814ec"),
    (0x0200C258, "6fbe65c443c28896"),
    (0x0200C274, "4787e2329f2e6812"),
    (0x0200C278, "88623a11868017f6"),
    (0x0200C29C, "2e6277951eaebaeb"),
    (0x0200C2B0, "90abb1001dd8793b"),
    (0x0200C2D0, "75c6f02568a613cd"),
    (0x0200C2F0, "8e33c8cac173acd8"),
    (0x0200C308, "b5fe168f4186bb90"),
    (0x0200C31C, "f64e8388906c8075"),
    (0x0200C324, "1c9e179819363f8f"),
    (0x0200C32C, "ba1ff841741332c9"),
    (0x0200C334, "612f17f5ef578316"),
    (0x0200C33C, "b6be170a9626e176"),
    (0x0200C344, "1fa3830da83f94af"),
    (0x0200C34C, "2c359c73a97b5a64"),
    (0x0200C358, "ec1e0c5a01ead822"),
    (0x0200C364, "2626cbe07c022ede"),
    (0x0200C37C, "e1d4b2547ecd0bc1"),
    (0x0200C384, "3937bd87cf3360d1"),
    (0x0200C38C, "114544b3bc1193da"),
    (0x0200C39C, "4f4c1af29a8b995a"),
    (0x0200C3B0, "0669c11d302222f0"),
    (0x0200C3B8, "9df9d71893a5af2c"),
    (0x0200C3C0, "edb78be17c8a1417"),
    (0x0200C3E0, "201102cfcb26805a"),
    (0x0200C3EC, "2dbb14b74502e338"),
];
// fndiff-recorded-end

/// The message-bank callees routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl MiscTextHooks for Rom<'_> {
    fn msg_open(&mut self) -> u32 {
        self.0.call(
            0x0204_5B38,
            &mut [Arg::V(0), Arg::V(3), Arg::V(0x2F), Arg::V(0x8007)],
        )
    }
    fn msg_string(&mut self, msg: u32, id: u32) -> u32 {
        self.0.call(0x0204_5EC0, &mut [Arg::V(msg), Arg::V(id)])
    }
    fn str_to_name(&mut self, s: u32, dst: &mut [u16; 9]) {
        let mut b = [0u8; 18];
        self.0
            .call(0x0204_5A38, &mut [Arg::V(s), Arg::Out(&mut b), Arg::V(9)]);
        *dst = crate::le::r16s(&b, 0);
    }
    fn str_free(&mut self, s: u32) {
        self.0.call(0x0204_5808, &mut [Arg::V(s)]);
    }
    fn msg_close(&mut self, msg: u32) {
        self.0.call(0x0204_5C04, &mut [Arg::V(msg)]);
    }
}

fn put_u32(b: &mut Vec<u8>, v: u32) {
    *b = v.to_le_bytes().to_vec();
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_C054 {
        return Misc::size() as u64;
    }
    let mut m = Misc::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0200_C058 => {
            m.init(&mut Rom(&mut c.env));
            0
        }
        0x0200_C0FC => {
            m.set_byte_0c(v[1] as usize, v[2]);
            0
        }
        0x0200_C104 => m.byte_0c(v[1] as usize) as u64,
        0x0200_C10C => {
            m.clear_0c();
            0
        }
        0x0200_C11C => {
            m.set_08(v[1], v[2], v[3], v[4]);
            0
        }
        0x0200_C164 => {
            let g = m.get_08();
            for (k, x) in g.iter().enumerate() {
                put_u32(&mut c.bufs[k + 1], *x);
            }
            0
        }
        0x0200_C184 => m.w14_low() as u64,
        0x0200_C18C => {
            m.set_w14_low(v[1]);
            0
        }
        0x0200_C1A4 => m.w14_high() as u64,
        0x0200_C1B0 => {
            m.set_w14_high(v[1]);
            0
        }
        0x0200_C1C4 => m.counter_18() as u64,
        0x0200_C1C8 => m.inc_counter_18() as u64,
        0x0200_C1DC => m.counter_1c() as u64,
        0x0200_C1E0 => m.inc_counter_1c() as u64,
        0x0200_C1F4 => {
            let _ = m.name_20();
            0x20
        }
        0x0200_C20C => {
            let _ = m.name_32();
            0x32
        }
        0x0200_C224 => m.b44() as u64,
        0x0200_C22C => {
            m.set_b44(v[1]);
            0
        }
        0x0200_C234 => m.has_badge(v[1]) as u64,
        0x0200_C248 => {
            m.give_badge(v[1]);
            0
        }
        0x0200_C258 => m.badge_count() as u64,
        0x0200_C274 => m.money() as u64,
        0x0200_C278 => m.add_money(v[1]) as u64,
        0x0200_C29C => m.sub_money(v[1]) as u64,
        0x0200_C2B0 => {
            m.set_record(v[1] as usize, &crate::le::r16s(&c.bufs[2], 0));
            0
        }
        0x0200_C2D0 => {
            let r = m.record(v[1] as usize);
            crate::le::w16s(&mut c.bufs[2], 0, &r);
            0
        }
        0x0200_C2F0 => {
            m.or_b45(v[1], v[2]);
            0
        }
        0x0200_C308 => m.b45_field(v[1]) as u64,
        0x0200_C31C => m.b45() as u64,
        0x0200_C324 => {
            m.set_b45(v[1]);
            0
        }
        0x0200_C32C => m.b46() as u64,
        0x0200_C334 => {
            m.set_b46(v[1]);
            0
        }
        0x0200_C33C => m.b47(v[1] as usize) as u64,
        0x0200_C344 => {
            m.set_b47(v[1] as usize, v[2]);
            0
        }
        0x0200_C34C => m.h4a(v[1] as usize) as u64,
        0x0200_C358 => {
            m.set_h4a(v[1] as usize, v[2]);
            0
        }
        0x0200_C364 => {
            m.add_h4a(v[1] as usize, v[2]);
            0
        }
        0x0200_C37C => m.q50(),
        0x0200_C384 => {
            m.set_q50(v[1] as u64 | (v[2] as u64) << 32);
            0
        }
        0x0200_C38C => {
            m.set_w_b8(v[1], v[2]);
            0
        }
        0x0200_C39C => m.w_b8(v[1]) as u64,
        0x0200_C3B0 => m.h_e0() as u64,
        0x0200_C3B8 => {
            m.set_h_e0(v[1]);
            0
        }
        0x0200_C3C0 => {
            m.sub_h_e0(v[1]);
            0
        }
        0x0200_C3E0 => m.w_dc() as u64,
        0x0200_C3EC => {
            m.set_w_dc(v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = m.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage misc`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "misc",
            include_str!("../../../../tools/rom-diff/specs/black-storage/misc.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn money_and_badges() {
    let mut m = Misc::default();
    assert_eq!(m.add_money(5000), 5000);
    assert_eq!(m.add_money(u32::MAX), MONEY_MAX);
    assert_eq!(m.sub_money(MONEY_MAX + 1), 0);
    m.give_badge(0);
    m.give_badge(7);
    assert!(m.has_badge(7) && !m.has_badge(3));
    assert_eq!(m.badge_count(), 2);
    let img: Vec<u8> = (0..SIZE as u32).map(|i| (i * 29 + 1) as u8).collect();
    assert_eq!(Misc::from_bytes(&img).to_bytes(), img);
}
