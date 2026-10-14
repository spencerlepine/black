use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02020F34, "6022a0e55081d8da"),
    (0x02020FE4, "db2568dd8d4b17ad"),
    (0x02021014, "1a68fbcd32e80d84"),
    (0x020210C8, "e7f234382465b37a"),
    (0x020212B0, "321f1d8586da2087"),
    (0x020212E4, "6a8a116f9b22d834"),
    (0x02021350, "0bae0ec8a8fd003e"),
    (0x0202136C, "51d6d8898f381000"),
    (0x0202138C, "68de0f1c2b1812b9"),
    (0x020213AC, "d354920cb4b4a03d"),
    (0x020213C0, "72f1d510989b7975"),
    (0x020213E0, "59f76e437ace48fe"),
    (0x020213F8, "f32188a70bb3da2e"),
];
// fndiff-recorded-end

/// The loader callees, routed to the spec's stub lines (same model as the ROM side). `rec` is the
/// record the stubbed loader "returns" (the ROM side reads it through the fixed buffer address).
struct Rom<'a> {
    env: &'a mut StubEnv,
    rec: [u8; RECORD_SIZE],
}

impl ItemHooks for Rom<'_> {
    fn archive_load(&mut self, archive: u32, member: u32, heap: u32, r3: u32) -> u32 {
        self.env.call(
            0x0204_8F60,
            &mut [Arg::V(archive), Arg::V(member), Arg::V(heap), Arg::V(r3)],
        )
    }
    fn block(&mut self, _handle: u32) -> [u8; RECORD_SIZE] {
        self.rec
    }
    fn free(&mut self, handle: u32) {
        self.env.call(0x0203_07B0, &mut [Arg::V(handle)]);
    }
}

fn words(b: &[u8]) -> Vec<u16> {
    b.as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes(*c))
        .collect()
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0202_0F34 => {
            let mut h = Rom {
                env: &mut c.env,
                rec: [0; RECORD_SIZE],
            };
            load_item_asset(&mut h, &c.globals[0], v[0], v[1], v[2]) as u64
        }
        0x0202_0FE4 => {
            let rec: [u8; RECORD_SIZE] = c.bufs[3][..RECORD_SIZE].try_into().unwrap();
            let mut h = Rom {
                env: &mut c.env,
                rec,
            };
            item_param(&mut h, v[0], v[1], v[2] as u16) as u64
        }
        0x0202_1014 => record_param(&c.bufs[0], v[1]) as u64,
        0x0202_10C8 => usage_param(&c.bufs[0], v[1]) as u64,
        0x0202_12B0 => tm_move(v[0], &words(&c.globals[0])) as u64,
        0x0202_12E4 => is_hm_move(v[0], &words(&c.globals[0])) as u64,
        0x0202_1350 => hm_index(v[0]) as u64,
        0x0202_136C => is_mail(v[0], &words(&c.globals[0])) as u64,
        0x0202_138C => mail_index(v[0], &words(&c.globals[0])) as u64,
        0x0202_13AC => mail_item(v[0], &words(&c.globals[0])) as u64,
        0x0202_13C0 => is_berry(v[0], &words(&c.globals[0])) as u64,
        0x0202_13E0 => outside_1ec_range(v[0]) as u64,
        0x0202_13F8 => pair_lookup(v[0], v[1], &c.globals[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-storage item`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "item",
            include_str!("../../../../tools/rom-diff/specs/black-storage/item.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

fn potion() -> [u8; RECORD_SIZE] {
    // price 300/10, usage kind 1, HP restore 20 at usage byte 13 (record byte 0x1D)
    let mut r = [0u8; RECORD_SIZE];
    r[0] = 30;
    r[0xC] = 1;
    r[0x1D] = 20;
    r
}

#[test]
fn record_and_usage_fields() {
    let r = potion();
    assert_eq!(record_param(&r, 0), 300);
    assert_eq!(record_param(&r, 0xE), 1);
    assert_eq!(usage_param(&r[USAGE_OFS..], 0x3A), 20);
    assert_eq!(record_param(&r, 0x3A), 20);
    // kind 0: the single byte whatever the parameter; unknown kind: 0
    let mut k0 = r;
    k0[0xC] = 0;
    k0[USAGE_OFS] = 0x5A;
    assert_eq!(record_param(&k0, 0x12), 0x5A);
    assert_eq!(record_param(&k0, 0x3F), 0x5A);
    k0[0xC] = 2;
    assert_eq!(record_param(&k0, 0x12), 0);
    // signed fields sign-extend, out-of-range parameters give 0
    let mut s = r;
    s[USAGE_OFS + 7] = 0xFE;
    assert_eq!(usage_param(&s[USAGE_OFS..], 0x34), 0xFFFF_FFFE);
    assert_eq!(usage_param(&s[USAGE_OFS..], 0x11), 0);
    assert_eq!(usage_param(&s[USAGE_OFS..], 0x3F), 0);
}

#[test]
fn classification() {
    assert_eq!(hm_index(0x1A4), 0);
    assert_eq!(hm_index(0x1A9), 5);
    assert_eq!(hm_index(0x1AA), 0xFF);
    assert_eq!(outside_1ec_range(0x1EC), 0);
    assert_eq!(outside_1ec_range(0x1F4), 0);
    assert_eq!(outside_1ec_range(0x1F5), 1);
    assert_eq!(outside_1ec_range(0x1EB), 1);
    let mails: Vec<u16> = (137..149).collect();
    assert!(is_mail(137, &mails) && !is_mail(149, &mails));
    assert_eq!((mail_index(139, &mails), mail_index(1, &mails)), (2, 0));
    assert_eq!((mail_item(11, &mails), mail_item(12, &mails)), (148, 0));
}
