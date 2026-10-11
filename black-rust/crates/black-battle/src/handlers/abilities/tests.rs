use crate::handlers::testkit::{run_handler_suite, TestHost};

/// Every ported ability handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py abilities`).
#[test]
fn abilities_match_rom() {
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_handler_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap());
    assert_eq!(h, "cfaa46dda4bf9b8e");
}
