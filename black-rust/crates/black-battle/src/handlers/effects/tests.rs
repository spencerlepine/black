use crate::handlers::testkit::{run_effect_suite, EffectRegistry, TestHost};

/// Every ported overlay-95 effect handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py effects`). The registries are read from the local ROM build
/// (`BLACK_ROM_BUILD`); skipped when it is missing.
#[test]
fn effects_match_rom() {
    let Some(reg) = EffectRegistry::load() else {
        eprintln!("skipped: ROM build not found");
        return;
    };
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_effect_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap(), &reg);
    assert_eq!(h, "da197eae5f3d6238");
}

/// The registries read by the harness: 8 field keys, 5 position keys, 14 side keys and 30
/// distinct handlers (8 + 5 + 17), all in 0x06898094-0x06899AA0.
#[test]
fn effect_registry_shape() {
    let Some(reg) = EffectRegistry::load() else {
        eprintln!("skipped: ROM build not found");
        return;
    };
    assert_eq!(reg.keys[0], (0..8).collect::<Vec<_>>());
    assert_eq!(reg.keys[1], (0..5).collect::<Vec<_>>());
    assert_eq!(reg.keys[2], (0..14).collect::<Vec<_>>());
    assert_eq!(reg.by_fn.len(), 30);
    for kind in 0..3 {
        let n = reg.by_fn.values().filter(|(k, _)| *k == kind).count();
        assert_eq!(n, [8, 5, 17][kind]);
    }
    assert!(reg
        .by_fn
        .keys()
        .all(|&a| (0x0689_8094..=0x0689_9AA0).contains(&a)));
}

/// The one stub added for effect handlers (`06898de0`, tag 0x155) against the Python stub, which
/// the script calls through the emulator with the same random arguments
/// (`tools/rom-diff/handlers.py effectstubs`). Does not need the ROM build.
#[test]
fn effect_stubs_match_python() {
    use crate::flow::FlowHost;
    use crate::testutil::{Fnv, Rng};
    let mut h = Fnv::new();
    let mut r = Rng::new(0xDE0);
    let mut host = TestHost::default();
    for _ in 0..200 {
        let sub = if r.next() & 1 != 0 {
            r.next() % 0x20
        } else {
            r.next() & 0xFFFF
        };
        let (a, b, c) = (r.next() % 8, r.next(), r.next() % 5);
        let side = [0, 1, 2, a, b][c as usize];
        h.w(sub);
        h.w(side);
        h.w(host.fun_06898de0(sub, side));
        host.log.drain(..).for_each(|v| h.w(v));
    }
    assert_eq!(h.hex(), "471d19c1d654a2e0");
}
