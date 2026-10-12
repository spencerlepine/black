use crate::handlers::testkit::{run_item_suite, ItemRegistry, TestHost};

/// Every ported held-item handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py items`). The item registry is read from the local ROM build
/// (`BLACK_ROM_BUILD`); skipped when it is missing.
#[test]
fn items_match_rom() {
    let Some(reg) = ItemRegistry::load() else {
        eprintln!("skipped: ROM build not found");
        return;
    };
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_item_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap(), &reg);
    assert_eq!(h, "fff17f74394c0f1b");
}

/// The registry read by the harness: 171 items, 229 distinct handlers, keys start at 149.
#[test]
fn item_registry_shape() {
    let Some(reg) = ItemRegistry::load() else {
        eprintln!("skipped: ROM build not found");
        return;
    };
    assert_eq!(reg.keys.len(), 171);
    assert_eq!(reg.keys[0], 149);
    assert_eq!(reg.by_fn.len(), 229);
    assert!(reg
        .by_fn
        .keys()
        .all(|&a| (0x021D_D05C..=0x021E_02C0).contains(&a)));
}
