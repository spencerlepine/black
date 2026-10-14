use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x021AA77C, "700ac4ea7a54204c"),
    (0x021AA794, "0d0a8e1e4efcb18d"),
    (0x021AA7AC, "e335a299aefc5471"),
    (0x021AA7CC, "b63f44cc8519fb0e"),
    (0x021AA7DC, "101957db2c123694"),
    (0x021AA80C, "f2b2bfc3e30d83f8"),
    (0x021AA8EC, "6c51bb9b5ec74113"),
    (0x021AA900, "292422ddd0e7d5e2"),
    (0x021AA944, "4ffc9cebdd3cec3b"),
    (0x021AAA4C, "413dd573cfd0a53c"),
    (0x021AAABC, "b9c3f4cecee0bc5c"),
    (0x021AAAF0, "666e980dbd8b38c4"),
    (0x021AAB1C, "3768dc4fe75a959d"),
    (0x021AAB48, "c1a269747cc5d424"),
    (0x021AAB54, "f50afb275b8b88ad"),
    (0x021AAB74, "76f5c4eb6742c358"),
    (0x021AABC4, "2de542984a587430"),
    (0x021AABEC, "9ae9caf275a47fbc"),
    (0x021AABF4, "a4acef6b76c0d217"),
    (0x021AACD4, "f0ebafc2f64b7146"),
    (0x021AAE34, "fa9b39e76def3b7a"),
    (0x021AAEC0, "a3a2b70d606c4d59"),
    (0x021AAF54, "097b0764f823e46d"),
    (0x021AAFA0, "d723ffb50e0cfd15"),
    (0x021AAFC0, "756c81d6879cb2b0"),
    (0x021AAFFC, "f779221691809c86"),
    (0x021AB018, "761f0306f8695761"),
    (0x021AB070, "43816e9c3e1eac63"),
    (0x021AB0AC, "9983c5f5d08a1950"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/effect_encount.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x021A_A77C => create(e, v[0]) as u64,
        0x021A_A794 => {
            delete(e, v[0]);
            0
        }
        0x021A_A7AC => {
            init(e, v[0], v[1]);
            0
        }
        0x021A_A7CC => {
            exit(e, v[0], v[1]);
            0
        }
        0x021A_A80C => {
            step(e, v[0]);
            0
        }
        0x021A_A8EC => {
            end(e, v[0]);
            0
        }
        0x021A_A900 => at(e, v[0], v[1]) as u64,
        0x021A_A944 => stepped_on(e, v[0]) as u64,
        0x021A_AA4C => distance(e, v[0], v[1]) as u64,
        0x021A_AABC => {
            mark_stepped(e, v[0]);
            0
        }
        0x021A_AAF0 => {
            anim_a(e, v[0], v[1]);
            0
        }
        0x021A_AB1C => {
            anim_b(e, v[0], v[1]);
            0
        }
        0x021A_AB54 => {
            mark_restore(e, v[0]);
            0
        }
        0x021A_AB74 => {
            restore(e, v[0], v[1]);
            0
        }
        0x021A_ABF4 => {
            window(e, v[0], v[1], v[2]);
            0
        }
        0x021A_ACD4 => {
            search(e, v[0], v[1]);
            0
        }
        0x021A_AE34 => {
            place(e, v[0], v[1]);
            0
        }
        0x021A_AEC0 => mmdl_on(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x021A_AF54 => mmdl_on_effect(e, v[0]) as u64,
        0x021A_AFA0 => {
            end_effect(e, v[0], v[1]);
            0
        }
        0x021A_AFC0 => {
            make_object(e, v[0], v[1], v[2]);
            0
        }
        0x021A_AFFC => {
            delete_object(e, v[0]);
            0
        }
        0x021A_B018 => dust_item(e) as u64,
        0x021A_B0AC => give_item(e, v[0], v[1]) as u64,
        0x021A_A7DC => {
            set_params(&mut e.mem, v[0], v[1]);
            0
        }
        0x021A_AB48 => item(&e.mem, v[0]) as u64,
        0x021A_ABC4 => step_due(&mut e.mem, v[0], v[1]) as u64,
        0x021A_ABEC => {
            reset_counter(&mut e.mem, v[0]);
            0
        }
        0x021A_B070 => shadow_item(e) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field effect_encount`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("effect_encount", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn counter_saturates() {
    let base = crate::testenv::WORK;
    let mut mem = Mem::new(base, vec![0u8; 0x600]);
    let (ewk, eff) = (base, base + 0x20);
    mem.w16(eff + EFF_INTERVAL, 2);
    assert!(!step_due(&mut mem, ewk, eff));
    assert!(step_due(&mut mem, ewk, eff));
    mem.w32(ewk + 0xC, u32::MAX);
    assert!(step_due(&mut mem, ewk, eff));
    assert_eq!(mem.r32(ewk + 0xC), u32::MAX);
    reset_counter(&mut mem, ewk);
    assert_eq!(mem.r32(ewk + 0xC), 0);
}
