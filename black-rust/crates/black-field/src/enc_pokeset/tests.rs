use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021A9350, "321a1660f9580902"),
    (0x021A94B0, "faa9a0fa40414d9b"),
    (0x021A9508, "02e6d0d0abe9023a"),
    (0x021A954C, "472b73a62e38281d"),
    (0x021A957C, "3d41ec1080543d9e"),
    (0x021A9604, "bbf23a2b4f0c682d"),
    (0x021A96B8, "10f07558a4526412"),
    (0x021A9780, "aae5e699546aa277"),
    (0x021A9810, "dbcd85bbc3735763"),
    (0x021A9828, "59c7d5c9f1deeb8d"),
    (0x021A996C, "07e1c7f19f7738d0"),
    (0x021A99A8, "c05025e4cbe92e39"),
    (0x021A99E4, "71ad6afb07dd6287"),
    (0x021A9A5C, "8ed27ab01b21dbb5"),
    (0x021A9B00, "48a481b56aad812b"),
    (0x021A9B68, "3228be56ff7e83ff"),
    (0x021A9BE8, "0000247aa84f4cda"),
    (0x021A9C58, "56d543a32c61500a"),
    (0x021A9CAC, "d8767a27c7d772ea"),
    (0x021A9D30, "5d554748a5942f5e"),
    (0x021A9DC8, "ec3cc29e686f3cd0"),
    (0x021A9DE0, "2af26d6b476b6055"),
    (0x021A9E64, "74773d933136a7bb"),
    (0x021A9E98, "cce9ca220a45f360"),
    (0x021A9EC4, "f98d4e390585e451"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/enc_pokeset.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_9350 => {
            efp_init(e, v[0], v[1], v[2], v[3], v[4]);
            0
        }
        0x021A_94B0 => rate_modifier(e, v[0], v[2]) as u64,
        0x021A_9508 => pick_mons(e, v[0], v[1], v[2]) as u64,
        0x021A_954C => wild_mons(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_957C => roamer(e, v[0], v[1], v[2]) as u64,
        // r5 of the caller: 0 under the harness entry state
        0x021A_9604 => wfbc_mon(e, v[0], v[1], v[2], 0) as u64,
        0x021A_96B8 => {
            wild_pokeparam(e, v[0], v[1], v[2]);
            0
        }
        0x021A_9780 => {
            roamer_pokeparam(e, v[0], v[1], v[2]);
            0
        }
        0x021A_9810 => swarm_species(e, v[0]) as u64,
        0x021A_9828 => {
            abilities(e, v[0], v[1]);
            0
        }
        0x021A_99A8 => swarm_record(e, v[0]) as u64,
        0x021A_99E4 => swarm_slot(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_9A5C => slot_group(e, v[0], v[1], v[2], v[3]) as u64,
        0x021A_9B00 => {
            roll_mon(e, v[0], v[1]);
            0
        }
        0x021A_9B68 => typed_slot(e, v[0], v[1], v[2]) as u64,
        0x021A_9BE8 => {
            max_level(e, v[0], v[1], v[2]);
            0
        }
        0x021A_9C58 => {
            slot_pick(e, v[0], v[1], v[2]);
            0
        }
        0x021A_9CAC => {
            held_item(e, v[0], v[1], v[2], v[3]);
            0
        }
        0x021A_9D30 => pid(e, v[0], v[1], v[2]) as u64,
        0x021A_9EC4 => uniform_slot(e, v[0]) as u64,
        0x021A_996C => rejected(&e.mem, v[0], v[1]) as u64,
        0x021A_9DC8 => roll100(e) as u64,
        0x021A_9DE0 => land_slot(e) as u64,
        0x021A_9E64 => water_slot(e) as u64,
        0x021A_9E98 => fishing_slot(e) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field enc_pokeset`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("enc_pokeset", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn slot_thresholds() {
    let land = [20, 40, 50, 60, 70, 80, 85, 90, 94, 98];
    assert_eq!(slot_by(0, &land), 0);
    assert_eq!(slot_by(19, &land), 0);
    assert_eq!(slot_by(20, &land), 1);
    assert_eq!(slot_by(97, &land), 9);
    assert_eq!(slot_by(99, &[60, 90, 95, 99]), 4);
    assert_eq!(slot_by(94, &[40, 80, 95, 99]), 2);
}

#[test]
fn level_filter() {
    let base = crate::testenv::WORK;
    let mut mem = Mem::new(base, vec![0u8; 0x40]);
    mem.w8(base + 0x1E, 20);
    mem.w32(base + 0x28, F_REPEL);
    assert!(rejected(&mem, base, 19));
    assert!(!rejected(&mem, base, 20));
    mem.w32(base + 0x28, F_REPEL | F_FISHING);
    assert!(!rejected(&mem, base, 19));
    mem.w32(base + 0x28, F_LOW_LEVEL_SKIP);
    assert!(rejected(&mem, base, 15));
    assert!(!rejected(&mem, base, 16));
    mem.w32(base + 0x28, F_LOW_LEVEL_SKIP | F_NO_FILTER);
    assert!(!rejected(&mem, base, 1));
}
