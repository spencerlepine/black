use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02008C20, "7f5ba36c7a705825"),
    (0x02008C28, "fb3e0b6900a280f3"),
    (0x02008C64, "fb223df900645477"),
    (0x02008C94, "cc343ae56bcf92a1"),
    (0x02008CB4, "5d704cd2059bfa12"),
    (0x02008CD4, "25765b62cec19ff2"),
    (0x02008CF8, "3a2db66ac6af05b9"),
    (0x02008D1C, "027dd66eaa9c99cd"),
    (0x02008D58, "7a81513c052880ff"),
    (0x02008DAC, "9bd7e3044b93f548"),
    (0x02008DF0, "28437860c64c1f88"),
    (0x02008DFC, "dc8b98ed1092d9e0"),
    (0x02008E30, "e36bb90ab225270b"),
    (0x02008E68, "b75cacf4a47b14a5"),
    (0x02008E78, "e7802726aa4f709d"),
    (0x02008E88, "dc6063c14aaa01f5"),
    (0x02008EA0, "10c0260327b0328f"),
];
// fndiff-recorded-end

/// The cipher calls routed to the spec's stub lines; the V-count is the spec's global.
struct Rom<'a> {
    env: &'a mut StubEnv,
    vcount: u32,
    /// B18b: the enciphered bytes the original hands the cipher (`ptr`): block (`arg0`) + CRYPT_OFS.
    data: u32,
}

impl RecordCrypt for Rom<'_> {
    fn checksum(&mut self, data: &[u8]) -> u16 {
        self.env
            .call(0x0203_F2FC, &mut [Arg::In(data), Arg::V(CRYPT_LEN as u32)]) as u16
    }
    fn encrypt(&mut self, seed: u32) {
        self.env.call(
            0x0203_F328,
            &mut [Arg::Ptr(self.data), Arg::V(CRYPT_LEN as u32), Arg::V(seed)],
        );
    }
    fn decrypt(&mut self, seed: u32) {
        self.env.call(
            0x0203_F37C,
            &mut [Arg::Ptr(self.data), Arg::V(CRYPT_LEN as u32), Arg::V(seed)],
        );
    }
    fn vcount(&mut self) -> u32 {
        self.vcount
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_8C20 => return Record::size() as u64,
        0x0200_8CF8 => return Record::limit(v[0]) as u64,
        _ => {}
    }
    let vcount = c
        .globals
        .first()
        .map_or(0, |g| u32::from_le_bytes(g[..4].try_into().unwrap()));
    let mut r = Record::from_bytes(&c.bufs[0]);
    let h = &mut Rom {
        env: &mut c.env,
        vcount,
        data: black_fndiff::buf_addr(0) + CRYPT_OFS as u32,
    };
    let ret: u64 = match addr {
        0x0200_8C28 => {
            r.init(h);
            0
        }
        0x0200_8C64 => {
            r.seal(v[1], h);
            0
        }
        0x0200_8C94 => {
            r.open(v[1], h);
            0
        }
        0x0200_8CB4 => r.get_raw(v[1]) as u64,
        0x0200_8CD4 => r.set_raw(v[1], v[2]) as u64,
        0x0200_8D1C => r.set(v[1], v[2], h) as u64,
        0x0200_8D58 => r.set_max(v[1], v[2], h) as u64,
        0x0200_8DAC => r.add(v[1], v[2], h) as u64,
        0x0200_8DF0 => r.inc(v[1], h) as u64,
        0x0200_8DFC => r.get(v[1], h) as u64,
        0x0200_8E30 => {
            r.fold_current(h);
            0
        }
        0x0200_8E68 => {
            r.set_max_123(v[1], h);
            0
        }
        0x0200_8E78 => r.get_123(h) as u64,
        0x0200_8E88 => {
            r.set_max_124(v[1], h);
            0
        }
        0x0200_8EA0 => r.get_124(h) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = r.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage record`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "record",
            include_str!("../../../../tools/rom-diff/specs/black-storage/record.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn caps_and_maxima() {
    let mut r = Record::default();
    r.init(&mut Plain);
    assert_eq!(r.add(2, 2_000_000, &mut Plain), 999_999);
    assert_eq!(r.set(71, 70_000, &mut Plain), 9999);
    assert_eq!(r.set(91, 70_000, &mut Plain), 0xFFFF);
    r.set(27, 50, &mut Plain);
    r.set(26, 20, &mut Plain);
    r.fold_current(&mut Plain);
    assert_eq!((r.get(26, &mut Plain), r.get(27, &mut Plain)), (50, 0));
    assert_eq!(Record::from_bytes(&r.to_bytes()), r);
}
