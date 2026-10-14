use super::*;
use black_fndiff::{run_suite, Arg, Case, StubEnv, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200E588, "1431427218b24f05"),
    (0x0200E590, "36f3d927403c500d"),
    (0x0200E5A0, "77f4c7ee7f8dff0b"),
    (0x0200E5CC, "67a9a84e50c6ac4e"),
    (0x0200E5E4, "a3f4adcac165fb1a"),
    (0x0200E5F8, "9f6e67c8107a46ea"),
    (0x0200E684, "5e59532cdb6c7963"),
];
// fndiff-recorded-end

/// `FUN_0203F1FC` routed to the spec's stub line.
struct Rom<'a>(&'a mut StubEnv);

impl RegulationHooks for Rom<'_> {
    fn crc16(&mut self, data: &[u8]) -> u16 {
        self.0
            .call(0x0203_F1FC, &mut [Arg::In(data), Arg::V(data.len() as u32)]) as u16
    }
}

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    if addr == 0x0200_E588 {
        return BattleExamination::size() as u64;
    }
    let mut d = BattleExamination::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_E590 => {
            d.init();
            0
        }
        0x0200_E5A0 => d.is_valid(&mut Rom(&mut c.env)) as u32,
        0x0200_E5CC => d.flag15_clear() as u32,
        0x0200_E5E4 => d.count(),
        0x0200_E5F8 => {
            let _ = d.entry(v[1]);
            (BattleExamination::entry_index(v[1]) * ENTRY_SIZE) as u32
        }
        0x0200_E684 => {
            d.mark_done(&mut Rom(&mut c.env));
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = d.to_bytes();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage battle_examination`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "battle_examination",
            include_str!("../../../../tools/rom-diff/specs/black-storage/battle_examination.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn image_round_trip() {
    let b: Vec<u8> = (0..SIZE as u32).map(|i| (i * 7 + 3) as u8).collect();
    assert_eq!(BattleExamination::from_bytes(&b).to_bytes(), b);
}
