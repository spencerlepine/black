use super::{Mt19937, SeedHost};
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x020056EC, "0dea5ef41764685b"),
    (0x02005728, "165e1120705e454e"),
    (0x0203F034, "e317bf3487daab09"),
    (0x0203F0A8, "55922d8517d2852c"),
    (0x0203F258, "87985a6a8f9b7a2c"),
    (0x0203F2FC, "5294994ef9c690bc"),
    (0x0203F328, "f4d05ad7301abcef"),
    (0x0203F388, "c87c7a14ab541526"),
];
// fndiff-recorded-end

/// `FUN_0203F258` callees routed to the spec's stubs.
struct Host<'a>(&'a mut StubEnv);

impl SeedHost for Host<'_> {
    fn read_entropy(&mut self, out: &mut [u8; 32]) {
        self.0.call(0x0208_8168, &mut [Arg::Out(out)]);
    }
    fn sha1_init(&mut self) {
        self.0.call(0x0208_13D8, &mut [Arg::P]);
    }
    fn sha1_update(&mut self, data: &[u8]) {
        self.0.call(
            0x0208_142C,
            &mut [Arg::P, Arg::P, Arg::V(data.len() as u32)],
        );
    }
    fn sha1_final(&mut self, out: &mut [u8; 20]) {
        self.0.call(0x0208_14C0, &mut [Arg::P, Arg::Out(out)]);
    }
}

fn ctx_of(b: &[u8]) -> super::MathRandContext32 {
    super::MathRandContext32::from_bytes(b[..24].try_into().unwrap())
}

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0203_F034 => {
            let mut m = Mt19937::from_bytes(c.region("st"));
            m.seed(c.vals[0]);
            m.write_bytes(c.region_mut("st"));
            0
        }
        0x0203_F0A8 => {
            let mut m = Mt19937::from_bytes(c.region("st"));
            let r = m.next_u32();
            m.write_bytes(c.region_mut("st"));
            r as u64
        }
        0x0200_56EC | 0x0200_5728 => {
            let mut x = ctx_of(c.region("ctx"));
            let r = if addr == 0x0200_56EC {
                x.next(c.vals[0])
            } else {
                x.next_scaled(c.vals[0])
            };
            c.region_mut("ctx")[..24].copy_from_slice(&x.to_bytes());
            r as u64
        }
        0x0203_F258 => {
            let mut x = ctx_of(c.region("ctx"));
            super::seed_lcg64(&mut Host(&mut c.env), &mut x);
            c.region_mut("ctx")[..24].copy_from_slice(&x.to_bytes());
            0
        }
        0x0203_F2FC => super::byte_sum(c.region("data"), c.vals[1]) as u64,
        0x0203_F328 => {
            let (size, seed) = (c.vals[1], c.vals[2]);
            super::lcg32_xor(c.region_mut("data"), size, seed);
            0
        }
        0x0203_F388 => {
            let st = c.region_mut("state");
            let mut x = u32::from_le_bytes(st[..4].try_into().unwrap());
            let r = super::lcg32_next(&mut x);
            st[..4].copy_from_slice(&x.to_le_bytes());
            r as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-core rng`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "rng",
            include_str!("../../../../tools/rom-diff/specs/black-core/rng.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn mt19937_reference_vector() {
    // Standard init_genrand(5489): first output 3499211612, then 581869302.
    let mut m = Mt19937::new();
    assert_eq!(m.next_u32(), 3_499_211_612);
    assert_eq!(m.next_u32(), 581_869_302);
    let mut s = Mt19937::new();
    s.seed(5489);
    assert_eq!(s.next_u32(), 3_499_211_612);
}

#[test]
fn mt19937_bytes_roundtrip() {
    let mut m = Mt19937::new();
    m.seed(1);
    m.next_u32();
    let mut b = vec![0u8; 0xA00];
    m.write_bytes(&mut b);
    assert_eq!(Mt19937::from_bytes(&b), m);
}

#[test]
fn lcg_helpers() {
    let mut x = 0;
    assert_eq!(super::lcg32_next(&mut x), 0);
    assert_eq!(x, 0x6073);
    let mut d = [1u8, 0, 2, 0, 9];
    super::lcg32_xor(&mut d, 5, 0);
    super::lcg32_xor(&mut d, 5, 0);
    assert_eq!(d, [1, 0, 2, 0, 9]);
    assert_eq!(super::byte_sum(&[0xFF, 0xFF, 1], 3), 0x1FF);
}
