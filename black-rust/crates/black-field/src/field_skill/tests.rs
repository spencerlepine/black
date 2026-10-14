use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0215E840, "bae1ac41bed9d8e0"),
    (0x0215EA14, "ac5eaf3a48a4b0a8"),
    (0x0215EA30, "0ec30e1af7043cf5"),
    (0x0215EA38, "5a2cdde47072575f"),
    (0x0215EA58, "072162aadb2bee40"),
    (0x0215EA6C, "d366e36b726f187e"),
    (0x0215EA94, "fd4e0bfff5e63ac4"),
    (0x0215EAC8, "16ce273d49a48d56"),
    (0x0215EAF0, "cc77a36f33f5e422"),
    (0x0215EB18, "63c7230e770a21a4"),
    (0x0215EB4C, "340af1f032e422ec"),
    (0x0215EB60, "4b7b50600a17aae1"),
    (0x0215EB88, "2d1d5879f07093ea"),
    (0x0215EBBC, "e675fadbadd6c4ae"),
    (0x0215EBD0, "3849d0abdde903d0"),
    (0x0215EBF8, "8ac89bb12ec0f984"),
    (0x0215EC2C, "980191e3c89511cf"),
    (0x0215EC40, "cba0d14bb19196ef"),
    (0x0215EC54, "802c3174c925d882"),
    (0x0215EC6C, "bd311d082ce4139f"),
    (0x0215EC80, "b5c9b4d1d0589888"),
    (0x0215EC94, "00a6e1be81713e17"),
    (0x0215ECBC, "bd04aafcbe1bf07a"),
    (0x0215ED8C, "bf0f124ece40aed2"),
    (0x0215EDA0, "8de8ef4ccc63199f"),
    (0x0215EDD0, "101a24d24950f719"),
    (0x0215EE24, "85911e8c850f0dbe"),
    (0x0215EE38, "a1ea4f8ef076daee"),
    (0x0215EE68, "2cf85bfa4a7c1588"),
    (0x0215EEBC, "50771e99610b86a0"),
    (0x0215EED0, "827c8fc007f55a61"),
    (0x0215EEF8, "da28c1253d8a6121"),
    (0x0215EF2C, "28a07858e0ae203d"),
    (0x0215EF40, "7f4188f43b828f2b"),
    (0x0215EF60, "b40d7a3960018c74"),
    (0x0215EFCC, "b511e10ce5f7a33f"),
    (0x0215EFF4, "7e95d1fad9fcfddf"),
    (0x0215F060, "cf58b04410fa55cf"),
    (0x0215F088, "6ffbeeba5537625a"),
    (0x0215F0B8, "7ee64dfd355a637e"),
    (0x0215F0E8, "6867f424c3aeae9b"),
    (0x0215F104, "e88eae31d668e7d8"),
    (0x0215F118, "e5abb960a2a4cea0"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/field_skill.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0215_E840 => {
            ctx_init(e, v[0], v[1]);
            0
        }
        0x0215_EA14 => can_use(e, v[0], v[1]) as u64,
        0x0215_EA30 => {
            set_param(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x0215_EA38 => use_skill(e, v[0], v[1], v[2]) as u64,
        0x0215_EA58 => check_bit(e, v[0], 0) as u64,
        0x0215_EAC8 => check_skill1(e, v[0]) as u64,
        0x0215_EB4C => check_bit(e, v[0], 2) as u64,
        0x0215_EBBC => check_bit(e, v[0], 3) as u64,
        0x0215_EC2C => check_bit(e, v[0], 8) as u64,
        0x0215_EC6C => check_bit(e, v[0], 4) as u64,
        0x0215_EC80 => check_bit(e, v[0], 5) as u64,
        0x0215_ED8C => check_bit(e, v[0], 7) as u64,
        0x0215_EE24 => check_bit(e, v[0], 6) as u64,
        0x0215_EEBC => check_bit(e, v[0], 10) as u64,
        0x0215_EF2C => check_bit(e, v[0], 9) as u64,
        0x0215_EA6C => script_event(e, v[0], v[1], 0x0215_EA95) as u64,
        0x0215_EAF0 => script_event(e, v[0], v[1], 0x0215_EB19) as u64,
        0x0215_EB60 => script_event(e, v[0], v[1], 0x0215_EB89) as u64,
        0x0215_EBD0 => script_event(e, v[0], v[1], 0x0215_EBF9) as u64,
        0x0215_EC94 => script_event(e, v[0], v[1], 0x0215_ECBD) as u64,
        0x0215_EED0 => script_event(e, v[0], v[1], 0x0215_EEF9) as u64,
        0x0215_EFCC => script_event(e, v[0], v[1], 0x0215_EF61) as u64,
        0x0215_F060 => script_event(e, v[0], v[1], 0x0215_EFF5) as u64,
        0x0215_EDA0 => beacon_script_event(e, v[0], v[1], 0x5B, 0x0215_EDD1) as u64,
        0x0215_EE38 => beacon_script_event(e, v[0], v[1], 100, 0x0215_EE69) as u64,
        0x0215_EC40 => event_c6078(e, v[0], v[1]) as u64,
        0x0215_EC54 => event_c8268(e, v[0], v[1]) as u64,
        0x0215_EF40 => event_d1c24(e, v[0], v[1]) as u64,
        0x0215_EA94 => script_proc(e, v[0], v[1], v[2], 0x2715) as u64,
        0x0215_EB18 => script_proc(e, v[0], v[1], v[2], 0x2713) as u64,
        0x0215_EB88 => script_proc(e, v[0], v[1], v[2], 0x2717) as u64,
        0x0215_EBF8 => script_proc(e, v[0], v[1], v[2], 0x2711) as u64,
        0x0215_EEF8 => script_proc(e, v[0], v[1], v[2], 0x271C) as u64,
        0x0215_ECBC => flash_proc(e, v[0], v[1], v[2]) as u64,
        0x0215_EDD0 => change_proc(e, v[0], v[1], v[2], 0x271A, 0x0218_4EEC) as u64,
        0x0215_EE68 => change_proc(e, v[0], v[1], v[2], 0x2719, 0x0218_4F40) as u64,
        0x0215_EF60 => two_script_proc(e, v[0], v[1], v[2], 0x271E, 0x28E3) as u64,
        0x0215_EFF4 => two_script_proc(e, v[0], v[1], v[2], 0x271D, 0x28E2) as u64,
        0x0215_F088 => check_fn(e, v[0], v[1]) as u64,
        0x0215_F0B8 => event_fn(e, v[0], v[1]) as u64,
        0x0215_F0E8 => {
            work_init(e, v[0], v[1], v[2]);
            0
        }
        0x0215_F104 => flag(e, v[0], v[1]) as u64,
        0x0215_F118 => in_rect(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field field_skill`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("field_skill", SPEC, PORTED, RECORDED), &call);
}

/// The embedded tables are the ROM's (`0x021711EC` rects, `0x021711FC` / `0x02171254` skill tables).
#[test]
fn rom_tables_match() {
    let dir = std::env::var("BLACK_ROM_BUILD").ok();
    let Some(ov) = dir.and_then(|d| std::fs::read(format!("{d}/OVY_10.sbin")).ok()) else {
        assert!(std::env::var("REQUIRE_ROM").is_err(), "OVY_10.sbin missing");
        return;
    };
    let at = |a: u32| (a - 0x0215_5160) as usize;
    let w = |o: usize| u32::from_le_bytes(ov[o..o + 4].try_into().unwrap());
    for (base, t) in [(0x0217_11FC, &SKILL_TABLE), (0x0217_1254, &SKILL_TABLE_B)] {
        for (k, &(ev, ck)) in t.iter().enumerate() {
            assert_eq!(w(at(base) + 8 * k), ev);
            assert_eq!(w(at(base) + 8 * k + 4), ck);
        }
    }
    for (addr, r, _) in RECTS {
        for (i, &v) in r.iter().enumerate() {
            let o = at(addr) + 2 * i;
            assert_eq!(u16::from_le_bytes([ov[o], ov[o + 1]]), v);
        }
    }
}

#[test]
fn every_table_entry_is_dispatched() {
    // the run_check / run_event dispatch covers every table entry (no fallback to an address call)
    use crate::hooks::{CArg, Mem, MemEnv};
    struct Rec(Mem, Vec<u32>);
    impl MemEnv for Rec {
        fn mem(&mut self) -> &mut Mem {
            &mut self.0
        }
    }
    impl FieldEnv for Rec {
        fn call(&mut self, addr: u32, _: &mut [CArg<'_>]) -> u32 {
            self.1.push(addr);
            0x140 // a GMEVENT work inside the window
        }
    }
    for t in [&SKILL_TABLE, &SKILL_TABLE_B] {
        for &(ev, ck) in t.iter() {
            let mut env = Rec(Mem::new(0x100, vec![0; 0x100]), Vec::new());
            run_check(&mut env, ck, 0x100);
            assert!(!env.1.contains(&(ck & !1)));
            run_event(&mut env, ev, 0x180, 0x100);
            assert!(!env.1.contains(&(ev & !1)));
        }
    }
}
