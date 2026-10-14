use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200BDBC, "7d6ab9df3b81ff3d"),
    (0x0200BDC4, "d5aea5c57eaf7735"),
    (0x0200BDE4, "902af48c6e3767c8"),
    (0x0200BDF8, "4d460bf4a41dcd80"),
    (0x0200BDFC, "7fa13cd199b0b4e3"),
    (0x0200BE00, "43b5daf40f43d18b"),
    (0x0200BE24, "0c1d3a455dc20472"),
    (0x0200BE54, "5f3d7e1b6e28fa4d"),
    (0x0200BE6C, "46e99f0c21adaea1"),
    (0x0200BE84, "194202c7de7ee880"),
    (0x0200BF08, "9afeadf919086ac9"),
    (0x0200BF10, "4333045012aafe83"),
    (0x0200BF18, "b7570c71c9bd7efa"),
    (0x0200BF54, "ea2d5d8c19987c17"),
    (0x0200BF70, "6bb554219294762f"),
    (0x0200BF78, "c10fe1b911a74251"),
    (0x0200BF94, "96bf67507529638b"),
    (0x0200BFA4, "14c9d87805098278"),
];
// fndiff-recorded-end

/// The callees routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl IntrudeHooks for Rom<'_> {
    fn power_init(&mut self, rec: &mut [u8; 16]) {
        self.0.call(0x0217_63BC, &mut [Arg::In(rec)]);
    }
    fn play_time(&mut self, pt: u32) -> (u32, u32, u32) {
        let h = self.0.call(0x0200_864C, &mut [Arg::V(pt)]);
        let m = self.0.call(0x0200_8650, &mut [Arg::V(pt)]);
        let s = self.0.call(0x0200_8654, &mut [Arg::V(pt)]);
        (h, m, s)
    }
}

fn entry(b: &[u8]) -> [u16; 10] {
    crate::le::r16s(b, 0)
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_BDBC => return IntrudeSave::size() as u64,
        0x0200_BE54 | 0x0200_BE6C => {
            // FUN_020071CC(sav, 53): the block fetch the port replaces by ownership.
            c.env
                .call(0x0200_71CC, &mut [Arg::V(v[0]), Arg::V(BLOCK_ID)]);
            let mut s = IntrudeSave::from_bytes(&c.bufs[2]);
            if addr == 0x0200_BE54 {
                s.load_power(&c.bufs[1].clone());
            } else {
                s.store_power(&mut c.bufs[1]);
            }
            c.bufs[2] = s.to_bytes();
            return 0;
        }
        _ => {}
    }
    let mut s = IntrudeSave::from_bytes(&c.bufs[0]);
    let mut start = c
        .globals
        .first()
        .map_or(0, |g| u64::from_le_bytes(g[..8].try_into().unwrap()));
    let h = &mut Rom(&mut c.env);
    let ret: u64 = match addr {
        0x0200_BDC4 => {
            s.init(h);
            0
        }
        0x0200_BDE4 => {
            s.reset_power(h);
            0
        }
        0x0200_BDF8 => s.points_c() as u64,
        0x0200_BDFC => s.points_e() as u64,
        0x0200_BE00 => {
            s.add_points_c(v[1]);
            0
        }
        0x0200_BE24 => {
            s.add_points_e(v[1]);
            0
        }
        0x0200_BE84 => {
            s.add_entry(&entry(&c.bufs[1]));
            0
        }
        0x0200_BF08 => {
            s.set_b1a0(v[1] as u8);
            0
        }
        0x0200_BF10 => s.b1a0() as u64,
        0x0200_BF18 => {
            s.set_flag(v[1]);
            0
        }
        0x0200_BF54 => {
            c.bufs[1][..2].copy_from_slice(&s.flags());
            0
        }
        0x0200_BF70 => s.counter() as u64,
        0x0200_BF78 => s.inc_counter() as u64,
        0x0200_BF94 => s.seconds(),
        0x0200_BFA4 => {
            let r = s.play_seconds(v[1], v[2], &mut start, h);
            c.globals[0].copy_from_slice(&start.to_le_bytes());
            r
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage intrude_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "intrude_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/intrude_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn entries_roll_over() {
    let mut s = IntrudeSave::default();
    for k in 0..21u16 {
        let mut e = [0u16; 10];
        e[0] = k;
        e[1] = EOS;
        e[8] = 1;
        s.add_entry(&e);
    }
    assert_eq!((s.entries[0][0], s.entries[19][0]), (1, 20));
    let mut bad = [0u16; 10];
    bad[8] = 1;
    s.add_entry(&bad);
    assert_eq!(s.entries[19][0], 20);
    assert_eq!(IntrudeSave::from_bytes(&s.to_bytes()), s);
}
