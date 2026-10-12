use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0201A858, "34daffb6efc3f4f2"),
    (0x0201A920, "379f97723cb92e46"),
    (0x0201A948, "20860185e21cd1bf"),
    (0x0201A954, "e8a7865b87106396"),
    (0x0201A98C, "dd960ce45879e9c7"),
    (0x0201A9C8, "5da22bf9fdac82c9"),
    (0x0201AA14, "e06c5e430991521c"),
    (0x0201AA18, "137264de3fb5f9c6"),
    (0x0201AA1C, "89910b4b8acbbbe7"),
    (0x0201AA50, "35836012888718c4"),
    (0x0201AA88, "8f833de683ec0586"),
    (0x0201AAD4, "a2a39cce94f0bd33"),
    (0x0201AB08, "c040aa119f837d71"),
    (0x0201AB18, "67a93d36cbcead5f"),
    (0x0201AB28, "d24f602279aec7e2"),
    (0x0201AB34, "4aafc410c0ca3b63"),
    (0x0201AB48, "3a4de5e8098c8781"),
    (0x0201AB8C, "e3383c3dd041c22a"),
    (0x0201AC2C, "6734dcb1adfecaf7"),
    (0x0201AC44, "efd0b640302face9"),
    (0x0201AC80, "91fc1f95e18276e8"),
    (0x0201ACC4, "0e2f93a8488c192a"),
    (0x0201ACDC, "6e3fd176c3b2424b"),
    (0x0201AD18, "8b7d0bd3c7892a6b"),
    (0x0201AD40, "65ce693c5dfdb966"),
    (0x0201AD58, "e45d028b1ce0d1c4"),
];
// fndiff-recorded-end

/// `FUN_02017E1C` routed to the spec's stub line (same model as the ROM side).
struct Rom<'a>(&'a mut StubEnv);

impl PartyHooks for Rom<'_> {
    fn mon_param(&mut self, mon_addr: u32, id: u32) -> u32 {
        self.0
            .call(0x0201_7E1C, &mut [Arg::V(mon_addr), Arg::V(id), Arg::V(0)])
    }
    fn clear_mon(&mut self, mon: &mut [u8]) {
        self.0.call(0x0201_7528, &mut [Arg::Out(mon)]);
    }
    fn alloc(&mut self, a: [u32; 5]) -> u32 {
        self.0.call(
            0x0203_0734,
            &mut [
                Arg::V(a[0]),
                Arg::V(a[1]),
                Arg::V(a[2]),
                Arg::V(a[3]),
                Arg::V(a[4]),
            ],
        )
    }
    fn free(&mut self, ptr: u32) {
        self.0.call(0x0203_07B0, &mut [Arg::V(ptr)]);
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let base = c.vals[0];
    let env = &mut Rom(&mut c.env);
    match addr {
        0x0201_A858 => next_digit_tuple(&mut c.bufs[0]) as u64,
        0x0201_A920 => create(c.vals[0], &mut c.bufs[1], c.vals[1], env) as u64,
        0x0201_A948 => {
            init_default(&mut c.bufs[0], env);
            0
        }
        0x0201_A954 => {
            init(&mut c.bufs[0], c.vals[1], env);
            0
        }
        0x0201_A98C => {
            let mon = c.bufs[1].clone();
            add(&mut c.bufs[0], &mon) as u64
        }
        0x0201_A9C8 => remove(&mut c.bufs[0], c.vals[1], env) as u64,
        0x0201_AA14 => capacity(&c.bufs[0]) as u64,
        0x0201_AA18 => count(&c.bufs[0]) as u64,
        0x0201_AB28 => mon_offset(c.vals[1]) as u64,
        0x0201_ACC4 => flag_get(&c.bufs[0], c.vals[1]) as u64,
        0x0201_ACDC => {
            let (idx, on) = (c.vals[1], c.vals[2]);
            flag_set(&mut c.bufs[0], idx, on);
            0
        }
        0x0201_AA1C => count_non_egg(&c.bufs[0], base, env) as u64,
        0x0201_AA50 => count_battle_able(&c.bufs[0], base, env) as u64,
        0x0201_AA88 => count_egg_and_field3_clear(&c.bufs[0], base, env) as u64,
        0x0201_AAD4 => count_field3_set(&c.bufs[0], base, env) as u64,
        0x0201_AB08 => find_battle_able(&c.bufs[0], base, env) as u64,
        0x0201_AB18 => find_not_egg(&c.bufs[0], base, env) as u64,
        0x0201_AB34 => battle_able_at(c.vals[1], c.vals[0], env) as u64,
        0x0201_AB48 => {
            let src = c.bufs[2].clone();
            replace_from(&mut c.bufs[0], base, c.vals[1], &src, c.vals[2], env);
            0
        }
        0x0201_AB8C => {
            let mut tmp = c.bufs[4].clone();
            let r = swap_slots(
                &mut c.bufs[0],
                c.vals[1],
                c.vals[2],
                c.vals[3],
                &mut tmp,
                env,
            );
            c.bufs[4] = tmp;
            r as u64
        }
        0x0201_AC2C => {
            let src = c.bufs[0].clone();
            copy(&src, &mut c.bufs[1]);
            0
        }
        0x0201_AC44 => any_field5(&c.bufs[0], base, c.vals[1], env) as u64,
        0x0201_AC80 => count_pending(&c.bufs[0], base, env) as u64,
        0x0201_AD18 => battle_able(base, env) as u64,
        0x0201_AD40 => not_egg(base, env) as u64,
        0x0201_AD58 => match c.vals[1] {
            0x0201_AD19 => find_first(&c.bufs[0], base, battle_able::<Rom>, env) as u64,
            0x0201_AD41 => find_first(&c.bufs[0], base, not_egg::<Rom>, env) as u64,
            p => unreachable!("find_first: unknown predicate {p:#010x}"),
        },
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-storage party`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "party",
            include_str!("../../../../tools/rom-diff/specs/black-storage/party.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn flags_and_add() {
    let mut p = vec![0u8; PARTY_SIZE];
    wr32(&mut p, 0, 6);
    assert!(add(&mut p, &[7u8; MON_SIZE]));
    assert_eq!(count(&p), 1);
    assert!(flag_get(&p, 0));
    assert_eq!(p[8], 7);
    flag_set(&mut p, 0, 0);
    assert!(!flag_get(&p, 0));
    assert!(!flag_get(&p, 40));
    wr32(&mut p, 0, 1);
    assert!(!add(&mut p, &[0u8; MON_SIZE]));
}
