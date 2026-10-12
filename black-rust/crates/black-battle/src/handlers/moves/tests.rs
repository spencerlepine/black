use crate::handlers::testkit::{run_move_suite, MoveRegistry, TestHost};

/// Every ported move handler, fuzzed against the original code
/// (`tools/rom-diff/handlers.py moves`). The move registry is read from the local ROM build
/// (`BLACK_ROM_BUILD`); skipped when it is missing.
#[test]
fn moves_match_rom() {
    let Some(reg) = MoveRegistry::load() else {
        eprintln!("skipped: ROM build not found");
        return;
    };
    let addrs: Vec<u32> = super::PORTED.to_vec();
    let h = run_move_suite(&addrs, |a| super::handler::<TestHost>(a).unwrap(), &reg);
    assert_eq!(h, "e9d0e632c5c9ffdf");
}

/// The registry read by the harness: 258 moves, 337 distinct handlers, handler code in
/// 0x021E0468-0x021E84FC.
#[test]
fn move_registry_shape() {
    let Some(reg) = MoveRegistry::load() else {
        eprintln!("skipped: ROM build not found");
        return;
    };
    assert_eq!(reg.keys.len(), 258);
    assert_eq!(reg.by_fn.len(), 337);
    assert!(reg
        .by_fn
        .keys()
        .all(|&a| (0x021E_02DC..=0x021E_8940).contains(&a)));
}

/// Number of scalar arguments of each stub, in the order of `move_stub_check` in
/// tools/rom-diff/handlers.py.
const STUB_ARGS: [usize; 39] = [
    1, 0, 1, 1, 1, 2, 0, 0, 1, 0, 0, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 0, 0, 2, 1, 1, 1, 1, 1, 1, 1, 1,
    0, 2, 2, 1, 1, 2, 0,
];

/// The stubs added for move handlers (tags 0x12F-0x153, plus `weather` at 021c2ab0) against
/// the Python stubs, which the script calls through the emulator with the same random
/// arguments (`tools/rom-diff/handlers.py movestubs`). Does not need the ROM build.
#[test]
fn move_stubs_match_python() {
    use crate::flow::FlowHost;
    use crate::testutil::{Fnv, Rng};
    let mut h = Fnv::new();
    let mut r = Rng::new(0x57B5);
    let mut host = TestHost {
        wseed: r.next(),
        ..Default::default()
    };
    let pick = |r: &mut Rng| -> u32 {
        match r.next() % 6 {
            0 => r.next() % 8,
            1 => 0x1F,
            2 => r.next() % 0x240,
            3 => r.next() % 16,
            4 => r.next() % 2,
            _ => r.next(),
        }
    };
    let b = |v: bool| v as u32;
    for (idx, &n) in STUB_ARGS.iter().enumerate() {
        for _ in 0..40 {
            let a: Vec<u32> = (0..n).map(|_| pick(&mut r)).collect();
            let mut out = [0u8; 8];
            let mut words: Vec<u32> = Vec::new();
            match idx {
                0 => words.push(host.fun_021c7d34(a[0])),
                1 => words.push(host.fun_021c7d44()),
                2 => words.push(host.fun_021c7ed8(a[0])),
                3 | 4 => {
                    let n = if idx == 3 {
                        Some(host.party_ids(a[0], &mut out))
                    } else {
                        host.partner_party_ids(a[0], &mut out)
                    };
                    match n {
                        None => words.push(0xFFFF_FFFF),
                        Some(n) => {
                            words.push(n);
                            words.extend(out.iter().take(n as usize).map(|&i| i as u32));
                        }
                    }
                }
                5 => words.push(host.fun_021c8030(a[0], a[1])),
                6 => words.push(host.fun_021c8114()),
                7 => words.push(host.fun_021c8138()),
                8 => words.push(b(host.fun_021c814c(a[0]))),
                9 => words.push(host.fun_021c8174()),
                10 => words.push(host.fun_021c8180()),
                11 => {
                    let pid = a[0] % 9;
                    let pid = if pid < 8 { pid } else { 0xFF };
                    words.push(host.fun_021c81c8(pid, a[1]))
                }
                12 => words.push(b(host.fun_021c824c(a[0]))),
                13 => words.push(b(host.fun_021c82c4(a[0], a[1]))),
                14 => words.push(host.fun_021c8300(a[0])),
                15 => host.fun_021c831c(a[0]),
                16 => words.push(host.fun_021c83a0(a[0])),
                17 => host.fun_021c83dc(a[0]),
                18 => words.push(b(host.fun_021c83e8(a[0]))),
                19 => {
                    let mut o = 0;
                    words.push(b(host.fun_021c846c(a[0], a[1], &mut o)));
                    words.push(o);
                }
                20 => host.fun_021c8494(a[0]),
                21 => words.push(host.fun_021c2e4c()),
                22 => words.push(host.weather()),
                23 => words.push(b(host.fun_06898cf4(a[0], a[1]))),
                24 => words.push(b(host.fun_0689c8ac(a[0]))),
                25 => words.push(b(host.fun_0689c8bc(a[0]))),
                26 => words.push(b(host.fun_0689c91c(a[0]))),
                27 => words.push(b(host.fun_0689c93c(a[0]))),
                28 => words.push(b(host.fun_0689ca1c(a[0]))),
                29 => words.push(b(host.fun_0689ca3c(a[0]))),
                30 => words.push(b(host.fun_0689ca5c(a[0]))),
                31 => words.push(b(host.fun_0689ca7c(a[0]))),
                32 => {
                    words.push(0x29);
                    words.extend(host.fun_0689ca8c().iter().map(|&m| m as u32));
                }
                33 => words.push(b(host.fun_0689d064(a[0], a[1]))),
                34 => words.push(host.fun_0689d098(a[0], a[1])),
                35 => words.push(host.fun_0689d0c8(a[0])),
                36 => words.push(host.fun_0689d174(a[0])),
                37 => words.push(host.fun_0689d184(a[0], a[1])),
                _ => words.push(b(host.fun_021c7f4c())),
            }
            words.into_iter().for_each(|v| h.w(v));
            host.log.drain(..).for_each(|v| h.w(v));
        }
    }
    assert_eq!(h.hex(), "e12192f6830dd600");
}
