use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200ADE8, "ae37f407a10fb799"),
    (0x0200AE08, "382dad5c50289aa5"),
    (0x0200AE14, "674db6f5b25bbc56"),
    (0x0200AE54, "823e49ea8e308ca9"),
    (0x0200AEE8, "781dfc24c484a25a"),
    (0x0200AFD4, "77043fd8c0714644"),
    (0x0200B040, "51b731073c1322a3"),
    (0x0200B074, "098b4d1c12519e1f"),
    (0x0200B078, "e37c79ebfb887780"),
    (0x0200B110, "8c5a6986ab73877b"),
    (0x0200B1A8, "c1dd234519b27cc4"),
    (0x0200B1C4, "dfad58cf2deb7806"),
    (0x0200B1D4, "36a149cb976cc0d3"),
    (0x0200B1E0, "9d2a2b8cc1c84c12"),
    (0x0200B1F8, "1e6caf8d9e5a456f"),
    (0x0200B204, "78fdbcec83bbdb14"),
    (0x0200B214, "d077269abf01b11a"),
    (0x0200B254, "9bb38111b5a0ece7"),
    (0x0200B2A4, "0bde09820eb052b8"),
    (0x0200B348, "ed2c72e7d3803ee5"),
];
// fndiff-recorded-end

/// Fixed emulator address of pointer argument 0.
const ARG0: u32 = 0x02E1_0000;

/// `FUN_0203F1FC` routed to the spec's stub line.
struct Rom<'a>(&'a mut StubEnv);

impl RegulationHooks for Rom<'_> {
    fn crc16(&mut self, data: &[u8]) -> u16 {
        self.0
            .call(0x0203_F1FC, &mut [Arg::In(data), Arg::V(data.len() as u32)]) as u16
    }
}

/// The party callees routed to the spec's stub lines.
struct Party<'a> {
    env: &'a mut StubEnv,
    party: u32,
}

impl PartySource for Party<'_> {
    fn count(&mut self) -> u32 {
        self.env.call(0x0201_AA18, &mut [Arg::V(self.party)])
    }
    fn mon(&mut self, i: u32) -> u32 {
        self.env
            .call(0x0201_AB28, &mut [Arg::V(self.party), Arg::V(i)])
    }
    fn mon_param(&mut self, mon: u32, id: u32) -> u32 {
        self.env
            .call(0x0201_7E1C, &mut [Arg::V(mon), Arg::V(id), Arg::V(0)])
    }
}

/// Writes a summary back into its (0x20-byte) test buffer, keeping the 2 bytes past it.
fn put_summary(buf: &mut [u8], s: &PartySummary) {
    buf[..SUMMARY_SIZE].copy_from_slice(&s.to_bytes());
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_ADE8 => RegulationBlock::size() as u64,
        0x0200_AE08 => {
            let src = Regulation::from_bytes(&c.bufs[0]);
            let mut dst = Regulation::from_bytes(&c.bufs[1]);
            Regulation::copy(&src, &mut dst);
            c.bufs[1] = dst.to_bytes().to_vec();
            0
        }
        0x0200_AE14 | 0x0200_AE54 | 0x0200_AEE8 | 0x0200_AFD4 | 0x0200_B040 => {
            let mut r = Regulation::from_bytes(&c.bufs[0]);
            let ret = match addr {
                0x0200_AE14 => {
                    r.noop();
                    0
                }
                0x0200_AE54 => r.get(v[1]),
                0x0200_AEE8 => {
                    r.set(v[1], v[2]);
                    0
                }
                0x0200_AFD4 => r.flag(v[1], v[2]) as u32,
                _ => {
                    c.bufs[1] = r.rule_bytes().to_vec();
                    0
                }
            };
            c.bufs[0] = r.to_bytes().to_vec();
            ret as u64
        }
        0x0200_B074..=0x0200_B1C4 => {
            let mut card = RegulationCard::from_bytes(&c.bufs[0]);
            let ret = match addr {
                0x0200_B074 => {
                    let _ = card.text();
                    CARD_TEXT_OFS
                }
                0x0200_B078 => match card.param(v[1]) {
                    CardParam::Card => ARG0,
                    CardParam::Value(x) => x,
                },
                0x0200_B110 => {
                    card.set_param(v[1], v[2]);
                    0
                }
                0x0200_B1A8 => card.is_valid(&mut Rom(&mut c.env)) as u32,
                0x0200_B1C4 => card.compute_crc(&mut Rom(&mut c.env)) as u32,
                _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
            };
            c.bufs[0] = card.to_bytes();
            ret as u64
        }
        0x0200_B1D4 | 0x0200_B1E0 | 0x0200_B204 | 0x0200_B348 => {
            let mut b = RegulationBlock::from_bytes(&c.bufs[0]);
            let i = v.get(1).copied().unwrap_or(0) as usize;
            let ret = match addr {
                0x0200_B1D4 => {
                    let _ = b.card_mut(i);
                    (i * CARD_SIZE) as u32
                }
                0x0200_B1E0 => {
                    b.set_card(i, &RegulationCard::from_bytes(&c.bufs[2]));
                    0
                }
                0x0200_B204 => {
                    let _ = b.summary_mut(i);
                    (SUMMARY_OFS + i * SUMMARY_SIZE) as u32
                }
                _ => {
                    b.init();
                    0
                }
            };
            c.bufs[0] = b.to_bytes();
            ret as u64
        }
        0x0200_B1F8 | 0x0200_B214 | 0x0200_B254 | 0x0200_B2A4 => {
            let mut s = PartySummary::from_bytes(&c.bufs[0]);
            let ret = match addr {
                0x0200_B1F8 => {
                    s.clear();
                    0
                }
                0x0200_B214 => s.get(v[1], v[2]),
                0x0200_B254 => {
                    s.set(v[1], v[2], v[3]);
                    0
                }
                _ => {
                    s.build(&mut Party {
                        env: &mut c.env,
                        party: v[1],
                    });
                    0
                }
            };
            put_summary(&mut c.bufs[0], &s);
            ret as u64
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-storage regulation`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "regulation",
            include_str!("../../../../tools/rom-diff/specs/black-storage/regulation.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_block_is_zero() {
    let mut b = RegulationBlock::from_bytes(&[0xAB; BLOCK_SIZE]);
    b.init();
    assert_eq!(b.to_bytes(), vec![0; BLOCK_SIZE]);
    let img: Vec<u8> = (0..BLOCK_SIZE as u32).map(|i| (i * 13 + 5) as u8).collect();
    assert_eq!(RegulationBlock::from_bytes(&img).to_bytes(), img);
}
