use crate::handlers::testkit::{run_effect_suite, EffectRegistry, TestHost};

/// Every ported overlay-95 effect handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py effects`). The registries are read from the local ROM build
/// (`BLACK_ROM_BUILD`); skipped when it is missing.
#[test]
fn effects_match_rom() {
    let Some(reg) = EffectRegistry::load() else {
        nds_rom::testenv::rom_missing("ROM build not found");
        return;
    };
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_effect_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap(), &reg);
    assert_eq!(h, "b41542c55f713fca");
}

/// The registries read by the harness: 8 field keys, 5 position keys, 14 side keys and 30
/// distinct handlers (8 + 5 + 17), all in 0x06898094-0x06899AA0.
#[test]
fn effect_registry_shape() {
    let Some(reg) = EffectRegistry::load() else {
        nds_rom::testenv::rom_missing("ROM build not found");
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

// B7b: The 06898de0 stub (tag 0x155) was dropped. The real ROM function now runs, reading from
// the layer array at 0x0689E7E0 which is seeded by the harness before each handler call. This
// allows end-to-end verification of the Spikes/Toxic Spikes layer branches (h_068995b0/h_068996d4).
