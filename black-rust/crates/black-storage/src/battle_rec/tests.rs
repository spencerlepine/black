use super::*;
use black_fndiff::{run_suite, Arg, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x0200B364, "4d665cb5ab68315d"),
    (0x0200B368, "4b50538a805c77b5"),
    (0x0200B37C, "ea2ced60cb432e1d"),
    (0x0200B394, "0464c65f6eaebb0f"),
    (0x0200B3A4, "2e61f73c69e7efe5"),
    (0x0200B3A8, "e3858b05f2be0020"),
    (0x0200B3AC, "1ba588ff482f377f"),
    (0x0200B3B0, "d21f021500cee494"),
    (0x0200B3C8, "40ef2da979e87674"),
    (0x0200B784, "4ff8bdf37d102db9"),
    (0x0200B98C, "6e2d20c9a8d8a754"),
    (0x0200BA38, "cc702979bf9cf229"),
];
// fndiff-recorded-end

fn call(addr: u32, c: &mut Case) -> u64 {
    let v = c.vals.clone();
    match addr {
        0x0200_B364 => return BattleRecBlock::size() as u64,
        0x0200_B784 => {
            let (n, per) = party_layout(v[0]);
            c.bufs[1] = n.to_le_bytes().to_vec();
            c.bufs[2] = per.to_le_bytes().to_vec();
            return 0;
        }
        0x0200_B98C => return VideoHeader::from_bytes(&c.bufs[0]).param(v[1], v[2]),
        0x0200_BA38 => return level_ok(v[0]) as u64,
        0x0200_B3B0 | 0x0200_B3C8 => {
            // FUN_020071CC(sav, 48): the block fetch the port replaces by ownership.
            let sav = if addr == 0x0200_B3B0 { v[1] } else { v[0] };
            c.env
                .call(0x0200_71CC, &mut [Arg::V(sav), Arg::V(BLOCK_ID)]);
            let mut block = BattleRecBlock::from_bytes(&c.bufs[2]);
            if addr == 0x0200_B3B0 {
                block.load(&BattleRecBlock::from_bytes(&c.bufs[0]));
            } else {
                let mut dst = BattleRecBlock::from_bytes(&c.bufs[1]);
                block.store(&mut dst);
                c.bufs[1] = dst.to_bytes();
            }
            c.bufs[2] = block.to_bytes();
            return 0;
        }
        _ => {}
    }
    let mut b = BattleRecBlock::from_bytes(&c.bufs[0]);
    let ret = match addr {
        0x0200_B368 => {
            b.init();
            0
        }
        0x0200_B37C => {
            b.reset();
            0
        }
        0x0200_B394 => {
            let _ = b.start(v[1]);
            4
        }
        0x0200_B3A4 => {
            let _ = b.words_mut();
            4
        }
        0x0200_B3A8 => {
            let _ = b.tail_mut();
            0x84
        }
        0x0200_B3AC => b.id(),
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    };
    c.bufs[0] = b.to_bytes();
    ret as u64
}

/// Every ported function against the original (`tools/check-fn.sh black-storage battle_rec`).
#[test]
fn fndiff_match_rom() {
    run_suite(
        &Suite::new(
            "battle_rec",
            include_str!("../../../../tools/rom-diff/specs/black-storage/battle_rec.spec"),
            super::PORTED,
            RECORDED,
        ),
        &call,
    );
}

#[test]
fn new_game_block() {
    let b = BattleRecBlock::new_game().to_bytes();
    assert_eq!(&b[..4], &[0xFF; 4]);
    assert!(b[4..].iter().all(|&x| x == 0));
    assert_eq!(party_layout(28), (4, 3));
    assert_eq!(party_layout(0), (2, 6));
}
