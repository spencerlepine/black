use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02019A7C, "9de0a3d0d252667f"),
    (0x02019A9C, "2aa042e0137f4fc0"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    match addr {
        0x0201_9A7C => {
            let st = c.region_mut("state");
            let mut x = u32::from_le_bytes(st[..4].try_into().unwrap());
            let r = super::lcg32_step(&mut x);
            st[..4].copy_from_slice(&x.to_le_bytes());
            r as u64
        }
        0x0201_9A9C => super::checksum(c.region("data"), c.vals[1]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    }
}

/// Every ported function against the original (`tools/check-fn.sh black-core poke_crypt`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "poke_crypt",
            include_str!("../../../../tools/rom-diff/specs/black-core/poke_crypt.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn lcg32_known_values() {
    let mut x = 0;
    assert_eq!(super::lcg32_step(&mut x), 0);
    assert_eq!(x, 0x6073);
    assert_eq!(super::lcg32_step(&mut x), 0xE97E);
}

#[test]
fn checksum_ignores_odd_tail() {
    assert_eq!(super::checksum(&[1, 0, 2, 0, 0xFF], 5), 3);
    assert_eq!(super::checksum(&[0xFF, 0xFF, 1, 0], 4), 0);
}
