use crate::handlers::testkit::{run_handler_suite, TestHost};

/// Every ported ability handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py abilities`).
#[test]
fn abilities_match_rom() {
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_handler_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap());
    assert_eq!(h, "5a6fdc63be9022b0");
}

/// Skip-check callbacks against the original (`tools/rom-diff/handlers.py skips`).
#[test]
fn skip_checks_match_rom() {
    use crate::battle_mon::tests::random_bpp;
    use crate::event_factor::EventSystem;
    use crate::flow::Battle;
    use crate::handlers::testkit::{TestHost, MONS};
    use crate::testutil::{Fnv, Rng};
    let mut h = Fnv::new();
    let mut r = Rng::new(0x5C1B);
    for addr in [
        0x021DAEC4u32,
        0x021DB1A8,
        0x021DB650,
        0x021DC820,
        0x021DCCEC,
    ] {
        let cb = super::skip_check::<TestHost>(addr).unwrap();
        for _ in 0..300 {
            let mut b = Battle::new(TestHost::default());
            b.mons = (0..MONS).map(|_| random_bpp(&mut r)).collect();
            let mut sys = EventSystem::new();
            // IDs up to 23 so both sides occur (poke_side is id >= 12; V-0017)
            sys.factor_mut(0).poke_id = (r.next() % 24) as u8;
            let ftype = r.next() % 8;
            let event = r.next() % 0x90;
            let sub = match r.next() % 4 {
                0 => 106,
                1 => r.next() % 600,
                3 => r.next() % 6, // 021dc820 tests sub_id <= 3
                _ => {
                    let o = 0x0B78 - 0x041C + 2 * (r.next() % 13) as usize;
                    u16::from_le_bytes([
                        crate::calc::BATTLE_CALC_RODATA[o],
                        crate::calc::BATTLE_CALC_RODATA[o + 1],
                    ]) as u32
                }
            };
            let owner = if r.next().is_multiple_of(4) {
                sys.factor_mut(0).poke_id // 021db650 needs owner == the factor's Pokemon
            } else {
                (r.next() % 24) as u8
            };
            h.w(cb(&sys, 0, &mut b, ftype, event, sub as u16, owner) as u32);
            for x in b.host.log.drain(..) {
                h.w(x);
            }
            for e in b.host.env.log.drain(..) {
                e.iter().for_each(|&v| h.w(v));
            }
        }
    }
    assert_eq!(h.hex(), "a7b0938a0e615b02");
}
