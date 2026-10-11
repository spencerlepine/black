use crate::handlers::testkit::{run_handler_suite, TestHost};

/// Every ported ability handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py abilities`).
#[test]
fn abilities_match_rom() {
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_handler_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap());
    assert_eq!(h, "be5f7a2275f7ebcf");
}

/// Skip-check callbacks against the original (`tools/rom-diff/handlers.py skips`).
#[test]
fn skip_checks_match_rom() {
    use crate::testutil::{Fnv, Rng};
    let mut h = Fnv::new();
    let mut r = Rng::new(0x5C1B);
    for _ in 0..500 {
        let (ftype, _ev, mut sub, _owner) =
            (r.next() % 8, r.next() % 0x90, r.next() % 200, r.next() % 24);
        if r.next() & 1 != 0 {
            sub = 106;
        }
        h.w(super::skip_021daec4(ftype, sub as u16) as u32);
    }
    assert_eq!(h.hex(), "0a3ea9d739ced545");
}
