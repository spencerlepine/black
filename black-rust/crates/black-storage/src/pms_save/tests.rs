use super::*;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200E6B0, "64ecdbf49491b84d"),
    (0x0200E6B4, "6e98297502bde36d"),
    (0x0200E6FC, "c13bae6a136e7bd2"),
    (0x0200E714, "30a5328f4686ed6e"),
    (0x02024910, "7859da0bb0dce2c9"),
    (0x02024918, "483086173385f27c"),
    (0x02024934, "3f10d8117ecb07e1"),
    (0x02024938, "b39417a0f082a758"),
    (0x02024994, "c6c24610a889b269"),
    (0x020249A0, "ebc0e6108842fa28"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_E6B0 => return PmsMessages::size() as u64,
        0x0202_4934 => return PmsSave::size() as u64,
        0x0200_E6B4 | 0x0200_E6FC | 0x0200_E714 => {
            let mut m = PmsMessages::from_bytes(&c.bufs[0]);
            match addr {
                0x0200_E6B4 => m.init(),
                0x0200_E6FC => c.bufs[2].copy_from_slice(&m.get(v[1] as usize).to_bytes()),
                _ => m.set(v[1] as usize, &Pms::from_bytes(&c.bufs[2])),
            }
            c.bufs[0] = m.to_bytes();
            return 0;
        }
        _ => {}
    }
    let mut s = PmsSave::from_bytes(&c.bufs[0]);
    let ret: u64 = match addr {
        0x0202_4910 => {
            let _ = s.sentence(v[1] as usize);
            8 + 8 * v[1] as u64
        }
        0x0202_4918 => {
            s.set_sentence(v[1], &Pms::from_bytes(&c.bufs[2]));
            0
        }
        0x0202_4938 => {
            s.init();
            0
        }
        0x0202_4994 => s.is_unlocked(v[1]) as u64,
        0x0202_49A0 => {
            s.unlock(v[1]);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = s.to_bytes();
    ret
}

/// Every ported function against the original (`tools/check-fn.sh black-storage pms_save`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "pms_save",
            include_str!("../../../../tools/rom-diff/specs/black-storage/pms_save.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn defaults() {
    let mut s = PmsSave::default();
    s.init();
    assert!(s.is_unlocked(1) && !s.is_unlocked(0));
    let mut m = PmsMessages::default();
    m.init();
    assert_eq!(m.get(4).words, [0x573, 0xFFFF]);
    assert_eq!(PmsMessages::from_bytes(&m.to_bytes()), m);
    assert_eq!(PmsSave::from_bytes(&s.to_bytes()), s);
}
