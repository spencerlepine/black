use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0219B41C, "c58b3d020a2a6bb0"),
    (0x0219B440, "99c7dabafa5275c6"),
    (0x0219B45C, "47893e53d9a46df8"),
    (0x0219B478, "519a2fa4986f5026"),
    (0x0219B494, "6a1c25e7763334ed"),
    (0x0219B4C8, "b05e8b0879482c0b"),
    (0x0219B4FC, "0fd71cdfc00080de"),
    (0x0219B500, "756e74bc508c4e0d"),
    (0x0219B504, "93df54f6f0b228b2"),
    (0x0219B508, "2569610849461695"),
    (0x0219B520, "52b6ebf439fb34ed"),
    (0x0219B59C, "7354c5bfa5d99661"),
    (0x0219B6B0, "886a7c712aad5c3c"),
    (0x0219B6E0, "c83178bd981ccee2"),
    (0x0219B75C, "626e5e6e93a5704a"),
    (0x0219B8A8, "f63cd098f187b7bc"),
    (0x0219B8BC, "197544fd7b3eed4a"),
    (0x0219B8E8, "9e1bf7b8df2bfe9b"),
    (0x0219B920, "27c03d03e56bb574"),
    (0x0219B924, "fb9373d31983685b"),
    (0x0219B968, "09f5eabb5378a320"),
    (0x0219B984, "68aa9197f9d4c88f"),
    (0x0219B9B0, "5ee982f09b1bd801"),
    (0x0219B9D8, "426df89ebbf4333b"),
    (0x0219BA0C, "16ec9cc58a24be0e"),
    (0x0219BA2C, "0fa04383e09d4c8e"),
    (0x0219BA6C, "43de890d5dfc234d"),
    (0x0219BAE0, "bc7c2bdfb18478bc"),
    (0x0219BB08, "b6580e3d6ceeb248"),
    (0x0219BB38, "bc702d184999f65a"),
    (0x0219BB3C, "099f721b64564b42"),
    (0x0219BB48, "7f407d52c7f865ee"),
    (0x0219BB84, "d018a97445ad9b18"),
    (0x0219BBA4, "89d0ff49ebec3329"),
    (0x0219BC48, "9399f82d6237dbcb"),
    (0x0219BCC0, "6af8f0560c6e6cef"),
    (0x0219BCE4, "82cabd42a58facb5"),
    (0x0219BD30, "2a3cccf440ebc74d"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/mvproc3c.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| {
        let m = v.first().copied().unwrap_or(0);
        let unit = |f: &mut dyn FnMut()| {
            f();
            0
        };
        match addr {
            0x0219_B41C => ev_busy(e, m) as u64,
            0x0219_B440 => unit(&mut || ev_init(e, m)),
            0x0219_B45C => ev_busy_query(e, m) as u64,
            0x0219_B478 => ev_spin_query(e, m) as u64,
            0x0219_B494 => moved(e, m) as u64,
            0x0219_B4C8 => standing(e, m) as u64,
            0x0219_B4FC => unit(&mut || ev_init_none(e, m)),
            0x0219_B500 => ev_busy_none(e, m) as u64,
            0x0219_B504 => ev_spin_none(e, m) as u64,
            0x0219_B508 => unit(&mut || ev_steps_init(e, m)),
            0x0219_B520 => ev_steps_busy(e, m) as u64,
            0x0219_B59C => ev_steps_spin(e, m) as u64,
            0x0219_B6B0 => unit(&mut || ev_look_init(e, m)),
            0x0219_B6E0 => ev_look_busy(e, m) as u64,
            0x0219_B75C => ev_look_spin(e, m) as u64,
            0x0219_B8A8 => pad_trigger(e) as u64,
            0x0219_B8BC => unit(&mut || follow_init(e, m)),
            0x0219_B8E8 => unit(&mut || follow_move(e, m)),
            0x0219_B920 => unit(&mut || follow_delete(e, m)),
            0x0219_B924 => follow_step(e, m, v[1]) as u64,
            0x0219_B968 => follow_wait(e, m, v[1]) as u64,
            0x0219_B984 => follow_check(e, m, v[1]) as u64,
            0x0219_B9B0 => unit(&mut || follow_start(e, m, v[1])),
            0x0219_B9D8 => player_moved(e, m, v[1]) as u64,
            0x0219_BA0C => unit(&mut || follow_record(e, m, v[1])),
            0x0219_BA2C => follower_base(e, m) as u64,
            0x0219_BA6C => follow_start_step(e, m) as u64,
            0x0219_BAE0 => unit(&mut || partner_init(e, m)),
            0x0219_BB08 => unit(&mut || partner_move(e, m)),
            0x0219_BB38 => unit(&mut || partner_delete(e, m)),
            0x0219_BB3C => unit(&mut || partner_return(e, m)),
            0x0219_BB48 => partner_step(e, m, v[1]) as u64,
            0x0219_BB84 => partner_wait(e, m, v[1]) as u64,
            0x0219_BBA4 => find_partner(e, m) as u64,
            0x0219_BC48 => partner_check(e, m, v[1]) as u64,
            0x0219_BCC0 => unit(&mut || partner_record(e, m, v[1], v[2])),
            0x0219_BCE4 => partner_due(e, m, v[1]) as u64,
            0x0219_BD30 => partner_start_step(e, m, v[1]) as u64,
            _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
        }
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field mvproc3c`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("mvproc3c", SPEC, PORTED, RECORDED), &call);
}

/// The state tables embedded here equal the ROM block.
#[test]
fn rom_tables_match() {
    for (base, t) in [
        (0x021D_5374, &FOLLOW_STATES[..]),
        (0x021D_538C, &PARTNER_STATES[..]),
    ] {
        for (i, &a) in t.iter().enumerate() {
            assert_eq!(rom32(base + 4 * i as u32), a);
        }
    }
}
