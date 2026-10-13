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

/// B7b2: the side-effect adder `06898c10` and remover `06898d10` (`SideEffects::add` /
/// `SideEffects::remove`) against the original code (`tools/rom-diff/handlers.py effect-sides`,
/// which documents the input distribution). EventSystem add / set_work / remove run as real code
/// on the ROM side and as `EventSystem` here; no stubs. A call whose entry lies outside the layer
/// array (the original accesses memory past it) must panic here; both sides then hash `OOB` and
/// continue from the state before the call. Needs the ROM build (registry); skipped without it.
#[test]
fn effect_sides_match_rom() {
    use super::{SideEffectEntry, SideEffects};
    use crate::event_factor::{EventSystem, FactorId, FACTOR_POOL};
    use crate::event_var::EventVarStack;
    use crate::testutil::{Fnv, Rng};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    const NONE: u32 = 0xFF;
    const OOB: u32 = 0xBAD;
    let Some(reg) = EffectRegistry::load() else {
        nds_rom::testenv::rom_missing("ROM build not found");
        return;
    };
    let idx = |f: Option<FactorId>| f.map_or(NONE, |v| v as u32);
    let mut h = Fnv::new();
    for seed in 0..200u32 {
        let mut r = Rng::new(0xC10 + seed);
        let mut sys = EventSystem::new();
        sys.init(&mut EventVarStack::new());
        let n = r.next();
        let n = if n.is_multiple_of(4) {
            0x80 + r.next() % 5
        } else {
            r.next() % 12
        };
        for _ in 0..n {
            let (a, b, d) = (r.next(), r.next(), r.next());
            sys.add(
                a % 8,
                (b & 0xFFFF) as u16,
                (b >> 16) % 8,
                d & 0xFF_FFFF,
                ((a >> 8) % 6) as u8,
                d,
                ((a >> 24) % 5) as u16,
            );
        }
        for _ in 0..r.next() % 4 {
            sys.remove((r.next() % FACTOR_POOL as u32) as FactorId);
        }
        let mut fx = SideEffects::default();
        for e in fx.entries.iter_mut() {
            let f = r.next();
            let factor = if f.is_multiple_of(3) {
                None
            } else {
                Some(((f >> 8) % FACTOR_POOL as u32) as FactorId)
            };
            let (cont, w8, lr) = (r.next(), r.next(), r.next());
            let layers = if lr % 8 != 0 { lr % 5 } else { lr };
            *e = SideEffectEntry {
                factor,
                cont,
                w8,
                layers,
            };
        }
        for _ in 0..16 {
            let add = r.next() % 5 < 3;
            let (s, k) = (r.next(), r.next());
            let side = if s % 8 != 0 { s % 2 } else { 2 + s % 19 };
            let key = if k % 8 != 0 { k % 14 } else { 14 + k % 87 };
            let cont = r.next();
            h.w(add as u32);
            h.w(side);
            h.w(key);
            let ret = if add {
                h.w(cont);
                catch_unwind(AssertUnwindSafe(|| {
                    idx(fx.add(&mut sys, &reg, side, key, cont))
                }))
            } else {
                catch_unwind(AssertUnwindSafe(|| fx.remove(&mut sys, side, key) as u32))
            };
            h.w(ret.unwrap_or(OOB));
            for e in &fx.entries {
                h.w(idx(e.factor));
                h.w(e.cont);
                h.w(e.w8);
                h.w(e.layers);
            }
            for f in &sys.pool {
                h.w(idx(f.prev));
                h.w(idx(f.next));
                h.w(f.table);
                h.w(f.skip_check);
                h.w(f.ftype);
                h.w(f.priority);
                h.w(f.flags);
                f.work.iter().for_each(|&w| h.w(w));
                h.w(f.sub_id as u32);
                h.b(f.owner);
                h.b(f.poke_id);
            }
            h.w(sys.depth);
            h.w(idx(sys.head));
            h.w(sys.count);
            sys.free.iter().for_each(|&f| h.w(f as u32));
        }
    }
    assert_eq!(h.hex(), "b426e443c888a200");
}
