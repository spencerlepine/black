use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219A87C, "4163dea0ec720fde"),
    (0x0219A89C, "9e460db06e11443c"),
    (0x0219A8C0, "9d6e4269d15a24d1"),
    (0x0219A900, "ce3853f1507e6e97"),
    (0x0219AA08, "20110ce6e5b158cc"),
    (0x0219AA38, "792ba237f200f9f5"),
    (0x0219AA68, "85ff9595735eaa1b"),
    (0x0219AA78, "7d9d573eb4a91fa6"),
    (0x0219AA88, "44e3c6dec1ce383e"),
    (0x0219AA98, "5ede8373eadb08f4"),
    (0x0219AAA8, "d6620d996e579ea1"),
    (0x0219AAB8, "96e36da6b9d96642"),
    (0x0219AAC8, "2fecbf60bfb8b61f"),
    (0x0219AAD8, "f69cf4f2a0f982af"),
    (0x0219AAE8, "4f990298fcf5fafe"),
    (0x0219AAF8, "0c76f24606412115"),
    (0x0219AB08, "0a46d9caff0c2cd3"),
    (0x0219AB18, "61bf55214caff7f7"),
    (0x0219AB28, "4039010dd84cbf7c"),
    (0x0219AB38, "bf4f08e805c00828"),
    (0x0219AB48, "5cc10e949fc45f3f"),
    (0x0219AB58, "05d3bb848a9eb0e6"),
    (0x0219AB68, "6ddd2597fa627f92"),
    (0x0219AB8C, "18003d9d000f4d5c"),
    (0x0219ACE8, "784610dfabd56e37"),
    (0x0219AD18, "32a5bf6eb0209349"),
    (0x0219AD48, "37655854dce22109"),
    (0x0219AD58, "96c9edbc9f30b423"),
    (0x0219AD68, "eef22a00fe62f3b5"),
    (0x0219AD78, "79752b4cce813755"),
    (0x0219AD88, "c77dfd206fea39d8"),
    (0x0219AD98, "4c8ba7a23efedb2b"),
    (0x0219ADA8, "9370be1126b28497"),
    (0x0219ADB8, "2804cb9ed2772b48"),
    (0x0219ADC8, "0f091c4d490851c8"),
    (0x0219ADEC, "2c2ffbeff3e88045"),
    (0x0219AE10, "9f9ae5c196e708d2"),
    (0x0219AE68, "e75357cc697f009f"),
    (0x0219AEC8, "e7cb0e2b86777a1c"),
    (0x0219B030, "01aa7ca5b9127655"),
    (0x0219B060, "2adffaaf2b68fd3d"),
    (0x0219B074, "714e9cf57adf173d"),
    (0x0219B0C4, "a4c208580a39e92d"),
    (0x0219B0D8, "3632ded535ce1e37"),
    (0x0219B0F0, "4db95e36ac0c2a15"),
    (0x0219B104, "f7b49f8fdc8fd46b"),
    (0x0219B120, "bb4aa1ef44335460"),
    (0x0219B13C, "608762c2d9d79701"),
    (0x0219B200, "3719d1eaa06b3d71"),
    (0x0219B2E8, "b1f61a13a60665b3"),
    (0x0219B334, "250c416edf81a8b6"),
    (0x0219B358, "6a6d15862cfe4603"),
    (0x0219B374, "293133a3389b5f3e"),
    (0x0219B390, "7e3e679f1b6d83da"),
    (0x0219B404, "02aa2a3ce47774e1"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3b.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = v.first().copied().unwrap_or(0);
        let unit = |f: &mut dyn FnMut()| {
            f();
            0
        };
        match addr {
            0x0219_A87C => unit(&mut || walk_init(e, m)),
            0x0219_A89C => unit(&mut || walk_move(e, m)),
            0x0219_A8C0 => walk_start(e, m, v[1]) as u64,
            0x0219_A900 => walk_step(e, m, v[1]) as u64,
            0x0219_AA08 => walk_wait(e, m, v[1]) as u64,
            0x0219_AA38 => unit(&mut || route_init(e, m, v[1], v[2], v[3])),
            0x0219_AA68 => unit(&mut || route_init_15(e, m)),
            0x0219_AA78 => unit(&mut || route_init_16(e, m)),
            0x0219_AA88 => unit(&mut || route_init_17(e, m)),
            0x0219_AA98 => unit(&mut || route_init_18(e, m)),
            0x0219_AAA8 => unit(&mut || route_init_19(e, m)),
            0x0219_AAB8 => unit(&mut || route_init_1a(e, m)),
            0x0219_AAC8 => unit(&mut || route_init_1b(e, m)),
            0x0219_AAD8 => unit(&mut || route_init_1c(e, m)),
            0x0219_AAE8 => unit(&mut || route_init_1d(e, m)),
            0x0219_AAF8 => unit(&mut || route_init_1e(e, m)),
            0x0219_AB08 => unit(&mut || route_init_1f(e, m)),
            0x0219_AB18 => unit(&mut || route_init_20(e, m)),
            0x0219_AB28 => unit(&mut || route_init_21(e, m)),
            0x0219_AB38 => unit(&mut || route_init_22(e, m)),
            0x0219_AB48 => unit(&mut || route_init_23(e, m)),
            0x0219_AB58 => unit(&mut || route_init_24(e, m)),
            0x0219_AB68 => unit(&mut || route_move(e, m)),
            0x0219_AB8C => route_step(e, m, v[1]) as u64,
            0x0219_ACE8 => route_wait(e, m, v[1]) as u64,
            0x0219_AD18 => unit(&mut || rroute_init(e, m, v[1], v[2], v[3])),
            0x0219_AD48 => unit(&mut || rroute_init_25(e, m)),
            0x0219_AD58 => unit(&mut || rroute_init_26(e, m)),
            0x0219_AD68 => unit(&mut || rroute_init_27(e, m)),
            0x0219_AD78 => unit(&mut || rroute_init_28(e, m)),
            0x0219_AD88 => unit(&mut || rroute_init_29(e, m)),
            0x0219_AD98 => unit(&mut || rroute_init_2a(e, m)),
            0x0219_ADA8 => unit(&mut || rroute_init_2b(e, m)),
            0x0219_ADB8 => unit(&mut || rroute_init_2c(e, m)),
            0x0219_ADC8 => unit(&mut || rroute_move(e, m)),
            0x0219_ADEC => unit(&mut || rroute_next_leg(e, m)),
            0x0219_AE10 => rroute_sight(e, m, v[1]) as u64,
            0x0219_AE68 => rroute_reverse(e, m, v[1]) as u64,
            0x0219_AEC8 => rroute_step(e, m, v[1]) as u64,
            0x0219_B030 => rroute_wait(e, m, v[1]) as u64,
            0x0219_B060 => pad_trigger(e) as u64,
            0x0219_B074 => player_walks_fast(e, m) as u64,
            0x0219_B0C4 => pad_held(e) as u64,
            0x0219_B0D8 => table_len(v[0], v[1]) as u64,
            0x0219_B0F0 => random_entry(e, v[0], v[1]) as u64,
            0x0219_B104 => random_dir(e, v[0], v[1]) as u64,
            0x0219_B120 => dir_table(v[0]) as u64,
            0x0219_B13C => player_dir_in_sight(e, m) as u64,
            0x0219_B200 => sight_dir(e, m, v[1], v[2]) as u64,
            0x0219_B2E8 => may_move(e, m, v[1]) as u64,
            0x0219_B334 => walks_fast_near(e, m) as u64,
            0x0219_B358 => spins_while_walking(e, m) as u64,
            0x0219_B374 => unit(&mut || spin_order(e, m, v[1])),
            0x0219_B390 => unit(&mut || spin_look_start(e, m, v[1])),
            0x0219_B404 => unit(&mut || spin_unlock(e, m, v[1])),
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3b`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3b", SPEC, PORTED, RECORDED), &call);
}

/// The embedded ROM block and the state tables equal the build's (skipped without the ROM build unless `REQUIRE_ROM`).
#[test]
fn rom_block_matches() {
    let Some(rd) = crate::testenv::overlay21() else {
        return;
    };
    for (i, &w) in rom::ROM_WORDS.iter().enumerate() {
        assert_eq!(rd(rom::ROM_BASE + 4 * i as u32), w);
    }
    for (base, t) in [
        (0x021D_4D60, &WALK_STATES[..]),
        (0x021D_4D04, &ROUTE_STATES[..]),
        (0x021D_4D78, &RROUTE_STATES[..]),
    ] {
        for (i, &a) in t.iter().enumerate() {
            assert_eq!(rom32(base + 4 * i as u32), a);
        }
    }
}

/// `fldmmdl::MOVE_PROC_TABLES` (0x55 entries, F4b-port2) equals the ROM table `0x021D5E04`, and every proc of a record
/// `{id, init, move, delete, return}` inside `fldmmdl_acmd.c` part 3 (`0x0219A098..0x0219C4AC`) is a ported function of
/// `mvproc3a`..`mvproc3d` (skipped without the ROM build unless `REQUIRE_ROM`).
#[test]
fn move_proc_tables_match_rom() {
    let Some(rd) = crate::testenv::overlay21() else {
        return;
    };
    let ported: Vec<u32> = [
        crate::mvproc3a::PORTED,
        PORTED,
        crate::mvproc3c::PORTED,
        crate::mvproc3d::PORTED,
    ]
    .concat();
    let mut in_area = 0;
    for (code, &rec) in crate::fldmmdl::MOVE_PROC_TABLES.iter().enumerate() {
        assert_eq!(
            rd(0x021D_5E04 + 4 * code as u32),
            rec,
            "move code {code:#x}"
        );
        if rec == 0 {
            assert_eq!(code, 0x2F);
            continue;
        }
        for k in 1..5 {
            let p = rd(rec + 4 * k) & !1;
            if (0x0219_A098..0x0219_C4AC).contains(&p) {
                assert!(
                    ported.contains(&p),
                    "move code {code:#x} proc {k}: {p:#010x} not ported"
                );
                in_area += 1;
            }
        }
    }
    assert!(in_area > 0);
}
