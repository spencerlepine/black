use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02007B00, "481dc77864dce891"),
    (0x02007B08, "bc5514d06fd7b83d"),
    (0x02007B50, "bdd933b8d0ab51d2"),
    (0x02007B54, "b8d3e87f473e1f3b"),
    (0x02007B5C, "ef2fb7ae0de65c1c"),
    (0x02007B64, "ed637ae0595b78fc"),
    (0x02007B68, "471afadd110b23e9"),
    (0x02007BD0, "ea0ed278b38a5b5a"),
    (0x02007C20, "8f5cf18f74c9ee6b"),
    (0x02007C34, "2ba32afa37e302ea"),
    (0x02007C70, "78772d91c0b7fdc1"),
];
// fndiff-recorded-end

/// The party queries routed to the spec's stub lines.
struct Rom<'a>(&'a mut StubEnv);

impl PerapHooks for Rom<'_> {
    fn party_count(&mut self, party: u32) -> u32 {
        self.0.call(0x0201_AA18, &mut [Arg::V(party)])
    }
    fn party_mon(&mut self, party: u32, i: u32) -> u32 {
        self.0.call(0x0201_AB28, &mut [Arg::V(party), Arg::V(i)])
    }
    fn species(&mut self, mon: u32) -> u32 {
        self.0
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(5), Arg::V(0)])
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_7B00 => return Perap::size() as u64,
        0x0200_7B68 => {
            let src = c.bufs[1].clone();
            unpack(&src, &mut c.bufs[0]);
            return 0;
        }
        _ => {}
    }
    let mut p = Perap::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0200_7B08 => {
            p.init();
            0
        }
        0x0200_7B50 => p.exists() as u64,
        0x0200_7B54 => {
            p.clear();
            0
        }
        0x0200_7B5C => {
            p.set_exists();
            0
        }
        0x0200_7B64 => {
            let _ = p.data();
            4
        }
        0x0200_7BD0 => {
            let src = c.bufs[1].clone();
            p.pack(&src);
            0
        }
        0x0200_7C20 => {
            let src = Perap::from_bytes(&c.bufs[1]);
            p.copy_from(&src);
            0
        }
        0x0200_7C34 => {
            p.check_party(v[1], &mut Rom(&mut c.env));
            0
        }
        0x0200_7C70 => p.cry_kind() as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = p.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage perapvoice`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "perapvoice",
            include_str!("../../../../tools/rom-diff/specs/black-storage/perapvoice.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn pack_then_unpack_keeps_the_nibbles() {
    let mut p = Perap::default();
    let src: Vec<u8> = (0..SAMPLES).map(|i| (i * 37) as u8).collect();
    p.pack(&src);
    let mut out = vec![0xAAu8; UNPACKED_SIZE];
    unpack(&p.data, &mut out);
    for i in 0..DATA_SIZE {
        // pack stores s / 16 + 8, unpack adds 8 again: the nibble of s / 16, as the high nibble
        let q = |s: u8| ((((s as i8 as i32) / 16) as u8) & 0xF) << 4;
        assert_eq!(out[8 * i], q(src[8 * i]));
        assert_eq!(out[8 * i + 7], q(src[8 * i + 4]));
    }
    assert!(out[SAMPLES..].iter().all(|&b| b == 0));
}
