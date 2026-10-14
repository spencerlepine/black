use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200D858, "af26a72bbc241805"),
    (0x0200D85C, "ed139a77cc9082b9"),
    (0x0200D868, "bc2e5efe1cebab13"),
    (0x0200D87C, "4c03fc4af2604e39"),
    (0x0200D884, "2736ad0457c8ab5b"),
    (0x0200D914, "03663c305a566289"),
    (0x0200D9E8, "cb224d2824ade773"),
    (0x0200DA14, "4d0cf1c47fb39229"),
    (0x0200DA28, "daf9292c0803a840"),
    (0x0200DA54, "7082a82d1e5020b1"),
    (0x0200DA64, "cde789730fb2700d"),
    (0x0200DA6C, "95e165e8d8385ded"),
    (0x0200DA80, "4a37d5f7b3a05a35"),
    (0x0200DA98, "f5e3d2d8d3f8d5d3"),
    (0x0200DABC, "fe6852c434cb9958"),
    (0x0200DAC0, "67287b203f72b5e6"),
    (0x0200DAC4, "99feb43173e62735"),
    (0x0200DAD8, "55218f7b3ae52975"),
    (0x0200DAEC, "ec41dc17db32b378"),
    (0x0200DB08, "fe468201cffbebf9"),
    (0x0200DB1C, "c4ae160432c9ff2b"),
    (0x0200DB30, "62db5417f550a735"),
    (0x0200DB44, "04bf241296160f37"),
    (0x0200DB60, "72d0b97455623564"),
    (0x0200DB80, "6cde87c3ef1ba267"),
    (0x0200DBA0, "a38f2d3ec68b92d9"),
    (0x0200DBF0, "a042ac0b8c0ab2a9"),
    (0x0200DBFC, "fd61c8e87b713395"),
    (0x0200DC08, "1424bf9a11099a4f"),
    (0x0200DC0C, "d2f06ad32c00a129"),
    (0x0200DC50, "b64c933e9c5b216a"),
    (0x0200DC74, "d7ed9a68a49e0645"),
    (0x0200DC94, "96e749a759db3248"),
    (0x0200DCF8, "02b3416583d43c20"),
    (0x0200DCFC, "24f7ecee74a18bab"),
    (0x0200DFCC, "f1034fcd48959694"),
    (0x0200DFD8, "c738ae604130cfae"),
    (0x0200DFE4, "05e36732fb13063e"),
    (0x0200E008, "f723af1cd22f1155"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_D858 => return BsubwayPlay::size() as u64,
        0x0200_DA64 => return BsubwayScore::size() as u64,
        0x0200_DAC0 => return BsubwayScore::mode_index(v[0]) as u64,
        0x0200_D85C..=0x0200_DA54 => {
            let mut p = BsubwayPlay::from_bytes(&c.bufs[0]);
            let ret = match addr {
                0x0200_D85C => {
                    p.init();
                    0
                }
                0x0200_D868 => {
                    p.set_bit1(v[1]);
                    0
                }
                0x0200_D87C => p.bit1(),
                0x0200_D884 => p.get(v[1], &mut c.bufs[2]),
                0x0200_D914 => {
                    p.set(v[1], &c.bufs[2]);
                    0
                }
                0x0200_D9E8 => {
                    p.add_counters(v[1], v[2], v[3]);
                    0
                }
                0x0200_DA14 => {
                    p.clear_b2();
                    0
                }
                0x0200_DA28 => {
                    p.inc_b2();
                    0
                }
                0x0200_DA54 => p.b2(),
                _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
            };
            c.bufs[0] = p.to_bytes().to_vec();
            return ret as u64;
        }
        0x0200_DFCC..=0x0200_E008 => {
            let mut k = BsubwayCounters::from_bytes(&c.bufs[0]);
            let (i, j) = (
                v.get(1).copied().unwrap_or(0) as usize,
                v.get(2).copied().unwrap_or(0) as usize,
            );
            let ret = match addr {
                0x0200_DFCC => {
                    k.clear();
                    0
                }
                0x0200_DFD8 => k.get(i, j),
                0x0200_DFE4 => {
                    k.set(i, j, v[3]);
                    0
                }
                0x0200_E008 => {
                    k.inc(i, j);
                    0
                }
                _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
            };
            c.bufs[0][..COUNTERS_SIZE].copy_from_slice(&k.to_bytes());
            return ret as u64;
        }
        _ => {}
    }
    let mut s = BsubwayScore::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_DA6C => {
            s.init();
            0
        }
        0x0200_DA80 => {
            s.add_bp(v[1]);
            0
        }
        0x0200_DA98 => {
            s.sub_bp(v[1]);
            0
        }
        0x0200_DABC => s.bp(),
        0x0200_DAC4 => s.h08(v[1]),
        0x0200_DAD8 => s.h1a(v[1]),
        0x0200_DAEC => {
            s.raise_h1a(v[1], v[2]);
            0
        }
        0x0200_DB08 => {
            s.set_h08(v[1], v[2]);
            0
        }
        0x0200_DB1C => {
            s.clear_h2c(v[1]);
            0
        }
        0x0200_DB30 => {
            s.start_h2c(v[1]);
            0
        }
        0x0200_DB44 => s.has_h2c(v[1]) as u32,
        0x0200_DB60 => {
            s.inc_h2c(v[1]);
            0
        }
        0x0200_DB80 => s.h2c_minus_one(v[1]),
        0x0200_DBA0 => s.flag(v[1], v[2]),
        0x0200_DBF0 => {
            s.inc_b3();
            0
        }
        0x0200_DBFC => {
            s.dec_b3();
            0
        }
        0x0200_DC08 => s.b3(),
        0x0200_DC0C => s.update_streak(v[1]),
        0x0200_DC74 => {
            s.read_data(v[1], &mut c.bufs[2]);
            0
        }
        0x0200_DC50 => {
            s.write_data(v[1], &c.bufs[2]);
            0
        }
        0x0200_DC94 => {
            s.compute_points(&BsubwayPlay::from_bytes(&c.bufs[1]));
            0
        }
        0x0200_DCF8 => s.points(),
        0x0200_DCFC => s.points_thousands(),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage bsubway`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "bsubway",
            include_str!("../../../../tools/rom-diff/specs/black-storage/bsubway.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_blocks() {
    assert_eq!(BsubwayPlay::default().to_bytes(), [0; PLAY_SIZE]);
    let s = BsubwayScore::new_game().to_bytes();
    assert_eq!(s[3], 1);
    assert_eq!(s.iter().map(|&x| x as u32).sum::<u32>(), 1);
}
