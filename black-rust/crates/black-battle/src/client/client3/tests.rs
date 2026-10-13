use crate::server_flow::testkit::{run_area_suite, Case};

// Written by tools/check-svflow.sh only, after the Rust hashes matched the ROM run.
// svflow-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021CF5B0, "dda4e54deec8cc8e"),
    (0x021CF81C, "9d8c5a4e5707d615"),
    (0x021CF89C, "e2f1dc9250c01c2f"),
    (0x021CF8D4, "920a93bd895931fa"),
    (0x021CF918, "dfdfe0f2aedace8b"),
    (0x021CF9B4, "e3991e7730373a66"),
    (0x021CFAC4, "414c6a7a6ca0d00b"),
    (0x021CFADC, "ab13b59e0cfb1ac8"),
    (0x021CFB8C, "e682a7ef9d0fa6d4"),
    (0x021CFBA4, "1d0ec47739381bd3"),
    (0x021CFBBC, "53e5920dd3b052c2"),
    (0x021CFBE0, "e27fd0233df681d2"),
    (0x021CFBF0, "eb96330b629643b2"),
    (0x021CFD50, "3550a127d8351ba7"),
    (0x021CFDCC, "d8bc1bffffff2215"),
    (0x021CFE08, "b300a0ba36f403ef"),
    (0x021CFE98, "86d0b7794f29dc2c"),
    (0x021CFFAC, "c78093cf8170fffd"),
    (0x021D0100, "0eb74357cbcb8a1a"),
    (0x021D0248, "953e4308d397ccee"),
    (0x021D0304, "a63f1989aaf922b1"),
    (0x021D04E4, "9b01397a7bf1709c"),
    (0x021D055C, "47ee81898b3b8f5d"),
    (0x021D05C4, "d11039f01b7a120c"),
    (0x021D0644, "3a1ca967e7741d1c"),
    (0x021D06C8, "3c2261906756d07a"),
    (0x021D0770, "d0c3c2f112f56ae7"),
    (0x021D0980, "9ebff6aa88c7ee1a"),
    (0x021D09E0, "8c14ae1c5b1475ba"),
    (0x021D0B10, "4f1a7c9aeaa1766f"),
    (0x021D0B6C, "1c2f66e613891ed0"),
    (0x021D0BB0, "20724cedce8a5129"),
    (0x021D0C78, "9c1d5b156db0bb8c"),
    (0x021D0CC4, "9aee8d23a8c9dd13"),
    (0x021D0D38, "7348b01a87fa6bc7"),
    (0x021D0D44, "85c362142fb5a62b"),
    (0x021D0D50, "7d3ea84c50e749fd"),
    (0x021D0E74, "e01a0b624c684801"),
    (0x021D0E7C, "cc62bafe81d6d1a6"),
    (0x021D102C, "ba1496dbdc015a6c"),
    (0x021D1058, "9301a77f65df6042"),
];
// svflow-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let a0 = c.vals.first().copied().unwrap_or(0);
    let (env, vals, bufs) = (&mut c.env, &c.vals, &mut c.bufs);
    match addr {
        0x021C_FAC4 => return super::not_ability_17(env, vals[1]) as u64,
        0x021C_FB8C => return super::flag8_test(env, vals[1]) as u64,
        0x021C_FDCC => return super::counter_test(env, vals[1]) as u64,
        0x021D_0E74 => return super::switch_replay(env) as u64,
        0x021D_0980 => {
            let has_out = vals[1] != 0;
            let out = if has_out {
                Some(&mut c.regions[0][..])
            } else {
                None
            };
            return super::live_slots(env, &bufs[0], out) as u64;
        }
        _ => {}
    }
    let (o, rest) = bufs.split_at_mut(1);
    let o = &mut o[0];
    match addr {
        0x021C_F5B0 => super::move_blocked(env, o, vals[1], vals[2], vals[3]) as u64,
        0x021C_F81C => super::usable_moves(env, o, vals[1], &mut rest[1]) as u64,
        0x021C_F89C => super::can_use(env, o, vals[1]) as u64,
        0x021C_F8D4 => {
            let (b, h) = rest.split_at_mut(2);
            super::escape_check(env, o, vals[1], &mut b[1], &mut h[0]) as u64
        }
        0x021C_F918 => {
            let (b, h) = rest.split_at_mut(1);
            super::run_check(env, o, &mut b[0], &mut h[0]) as u64
        }
        0x021C_F9B4 => {
            let (b, h) = rest.split_at_mut(2);
            super::trap_check(env, o, vals[1], &mut b[1], &mut h[0]) as u64
        }
        0x021C_FADC => super::arena_trap_test(env, o, vals[1]) as u64,
        0x021C_FBA4 => {
            super::clear_slots(o);
            0
        }
        0x021C_FBBC => super::slot_holds(o, vals[1] as i32) as u64,
        0x021C_FBE0 => {
            super::set_slot(o, vals[1], vals[2]);
            0
        }
        0x021C_FBF0 => super::ai_switch(env, o, vals[1], vals[2], &mut rest[2]) as u64,
        0x021C_FD50 => super::random_opponent(env, o, vals[1]) as u64,
        0x021C_FE08 => super::wonder_guard_test(env, o, vals[1], vals[2]) as u64,
        0x021C_FE98 => super::super_effective_test(env, o, vals[1], vals[2]) as u64,
        0x021C_FFAC => super::encore_test(env, o, vals[1], vals[2]) as u64,
        0x021D_0100 => super::absorb_switch(env, o, vals[1], vals[2], &mut rest[2]) as u64,
        0x021D_0248 => super::natural_cure_switch(env, o, vals[1], &mut rest[1]) as u64,
        0x021D_0304 => super::stat_boost_switch(env, o, vals[1], vals[2], &mut rest[2]) as u64,
        0x021D_04E4 => super::find_ability_slot(env, o, vals[1], &mut rest[1]) as u64,
        0x021D_055C => super::any_hitter(env, o, vals[1], vals[2]) as u64,
        0x021D_05C4 => super::find_resist_slot(env, o, vals[1], vals[2], &mut rest[2]) as u64,
        0x021D_0644 => super::can_hit(env, o, vals[1], vals[2], vals[3]) as u64,
        0x021D_06C8 => super::random_partner_side(env, o) as u64,
        0x021D_0770 => super::ai_step(env, o, a0, &mut rest[0]) as u64,
        0x021D_09E0 => {
            super::sort_by_power(env, o, &mut rest[0], vals[2], vals[3]);
            0
        }
        0x021D_0B10 => {
            super::setup_party_block(env, o, vals[1], vals[2], vals[3], vals[4]);
            0
        }
        0x021D_0B6C => {
            super::write_switch_answers(env, o, a0, &rest[0]);
            0
        }
        0x021D_0BB0 => {
            super::forced_switch_answers(env, o, a0);
            0
        }
        0x021D_0C78 => {
            let (out, list) = rest.split_at_mut(1);
            super::switch_requests(env, o, &mut out[0], &list[0], vals[2]) as u64
        }
        0x021D_0CC4 => super::yes_no_step(env, o, a0, &mut rest[0]) as u64,
        0x021D_0D38 => {
            let (st, list) = rest.split_at_mut(2);
            super::req5_step(env, o, a0, &mut st[0], &list[0], vals[3]) as u64
        }
        0x021D_0D44 => {
            let (st, list) = rest.split_at_mut(2);
            super::req6_step(env, o, a0, &mut st[0], &list[0], vals[3]) as u64
        }
        0x021D_0D50 => super::ai_switch_in(env, o, a0, &rest[0], vals[1]) as u64,
        0x021D_0E7C => {
            let (st, list) = rest.split_at_mut(2);
            super::switch_in_step(env, o, a0, &mut st[0], vals[2], &list[0], vals[3]) as u64
        }
        0x021D_102C => super::partner_fainted(env, o) as u64,
        0x021D_1058 => super::check_partners(env, o) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/rom-diff/svflow.py client3`).
#[test]
fn svflow_match_rom() {
    run_area_suite(
        "client3",
        include_str!("../../../../../tools/rom-diff/svflow_specs/client3.spec"),
        super::PORTED,
        RECORDED,
        call,
    );
}
