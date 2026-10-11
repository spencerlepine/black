use crate::handlers::testkit::{run_handler_suite, TestHost};

/// Every ported ability handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py abilities`).
#[test]
fn abilities_match_rom() {
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_handler_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap());
    assert_eq!(h, "1ff8de4b55be7537");
}

/// Skip-check callbacks against the original (`tools/rom-diff/handlers.py skips`).
#[test]
fn skip_checks_match_rom() {
    use crate::handlers::testkit::TestHost;
    use crate::testutil::{Fnv, Rng};
    let mut h = Fnv::new();
    let mut r = Rng::new(0x5C1B);
    let mut host = TestHost::default();
    for which in 0..2 {
        for _ in 0..500 {
            let ftype = r.next() % 8;
            let _event = r.next() % 0x90;
            let mut sub = r.next() % 200;
            let _owner = r.next() % 24;
            if r.next() & 1 != 0 {
                sub = 106;
            }
            let v = if which == 0 {
                super::skip_021daec4(ftype, sub as u16)
            } else {
                super::skip_021db1a8(&mut host, ftype, sub as u16)
            };
            h.w(v as u32);
            for x in host.log.drain(..) {
                h.w(x);
            }
        }
    }
    assert_eq!(h.hex(), "2cf0fe0796b1fd80");
}
