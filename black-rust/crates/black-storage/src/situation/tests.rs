use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008828, "cafa4924c9a9b7ed"),
    (0x0200882C, "2d7641db58830729"),
    (0x02008844, "220a65c9177399ae"),
    (0x02008848, "807302e55cb18b3c"),
    (0x0200884C, "fd2ff8b5e76a5d87"),
    (0x02008850, "7ab5524d6a5619ab"),
    (0x02008860, "faa9b5f2176923de"),
    (0x020088B8, "50030d49e8255f2d"),
    (0x0200890C, "5dbf12be33ede355"),
    (0x0200892C, "cb003b4892c2e544"),
    (0x0200893C, "42b2a4668f13d5d8"),
    (0x0200894C, "555e4fcfab044cd0"),
    (0x02008954, "d0f06f0ecbf864d2"),
    (0x0200895C, "80b0a5da6658b9b9"),
    (0x02008964, "e60d1c44332f903f"),
    (0x0200896C, "eb68a6bbdbcd554e"),
    (0x02008970, "54322275058d4247"),
    (0x02008978, "2190c4cec997e82a"),
    (0x02008980, "9a59a52276e27fe2"),
    (0x02008988, "142390f1dec8ca0e"),
    (0x02008990, "1c0f08f08af59246"),
    (0x02008998, "a1d839b5324c6b95"),
    (0x020089A0, "d302357b87402b8c"),
];
// fndiff-recorded-end

/// The field-status accessors routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl FieldStatusHooks for Rom<'_> {
    fn status_6(&mut self, fs: u32) -> u8 {
        self.0.call(0x0201_4468, &mut [Arg::V(fs)]) as u8
    }
    fn set_status_6(&mut self, fs: u32, v: u8) {
        self.0
            .call(0x0201_4464, &mut [Arg::V(fs), Arg::V(v as u32)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_8828 => return Situation::size() as u64,
        0x0200_8860 | 0x0200_88B8 | 0x0200_890C => {
            // FUN_020071CC(sav, 28): the block fetch the port replaces by ownership.
            c.env
                .call(0x0200_71CC, &mut [Arg::V(v[0]), Arg::V(BLOCK_ID)]);
            let mut s = Situation::from_bytes(&c.bufs[2]);
            match addr {
                0x0200_8860 => s.store_pos(&c.bufs[1].clone()),
                0x0200_88B8 => s.load_pos(&mut c.bufs[1]),
                _ => s.copy_pos(&mut c.bufs[1]),
            }
            c.bufs[2] = s.to_bytes();
            return 0;
        }
        _ => {}
    }
    let mut s = Situation::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0200_882C => {
            s.init();
            0
        }
        0x0200_8844 => {
            let _ = s.loc_00();
            Situation::RECORD_OFFSETS[0] as u64
        }
        0x0200_8848 => {
            let _ = s.loc_38();
            Situation::RECORD_OFFSETS[1] as u64
        }
        0x0200_884C => {
            let _ = s.loc_54();
            Situation::RECORD_OFFSETS[2] as u64
        }
        0x0200_8850 => {
            let _ = s.w70();
            Situation::RECORD_OFFSETS[3] as u64
        }
        0x0200_892C => {
            s.save_status(v[1], &mut Rom(&mut c.env));
            0
        }
        0x0200_893C => {
            s.restore_status(v[1], &mut Rom(&mut c.env));
            0
        }
        0x0200_894C => {
            s.set_b73(v[1] as u8);
            0
        }
        0x0200_8954 => {
            c.bufs[1][0] = s.b73();
            0
        }
        0x0200_895C => {
            s.set_b74(v[1] as u8);
            0
        }
        0x0200_8964 => {
            c.bufs[1][0] = s.b74();
            0
        }
        0x0200_896C => {
            s.set_d78(v[1]);
            0
        }
        0x0200_8970 => {
            c.bufs[1].copy_from_slice(&s.d78().to_le_bytes());
            0
        }
        0x0200_8978 => s.egg_steps() as u64,
        0x0200_8980 => {
            s.set_egg_steps(v[1] as u16);
            0
        }
        0x0200_8988 => {
            s.set_w7e(v[1] as u16);
            0
        }
        0x0200_8990 => s.w7e() as u64,
        0x0200_8998 => s.w76() as u64,
        0x0200_89A0 => {
            s.set_w76(v[1] as u16);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage situation`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "situation",
            include_str!("../../../../tools/rom-diff/specs/black-storage/situation.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn position_round_trip() {
    let mut s = Situation::default();
    s.init();
    assert_eq!(s.w76(), INIT_76);
    let mut rec = vec![0u8; 0x44];
    for (i, b) in rec.iter_mut().enumerate() {
        *b = i as u8 + 1;
    }
    s.store_pos(&rec);
    let mut back = vec![0u8; 0x44];
    s.load_pos(&mut back);
    assert_eq!(back[0x1B], rec[0x1B]);
    assert_eq!(back[0x40], rec[0x40]);
    assert_eq!(&back[4..0x1A], &rec[4..0x1A]);
    assert_eq!(Situation::from_bytes(&s.to_bytes()), s);
}
