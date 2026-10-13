use super::*;
use crate::testenv::run;
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02198E44, "5ef0dcb370eb7cb1"),
    (0x02198E60, "f0c77d6bae37f841"),
    (0x02198E7C, "ac2a2ded2e8b87ce"),
    (0x02198E98, "da3ed0b93de2f16a"),
    (0x02198EB0, "7e2389a37f8ca080"),
    (0x02198EC8, "b15f888878391784"),
    (0x02198F20, "703910dcb0f2c431"),
    (0x02198F68, "16da1e4ee541ee58"),
    (0x02198FB0, "667be61bfcf9aba8"),
    (0x02199034, "3338227809b367ca"),
    (0x02199050, "181a6c5c35d8a2f1"),
    (0x02199070, "79289329fd962775"),
    (0x02199088, "3f6a78632cf7f622"),
    (0x021990A0, "b3948bdade3703be"),
    (0x021990B8, "0f04355fdc031972"),
    (0x021990D4, "584bdae6b4c22a0c"),
    (0x0219912C, "e2a8d05fa2545b69"),
    (0x021991F0, "bfded177c3e4d027"),
    (0x02199234, "b6853246acd9bc64"),
    (0x02199268, "5d3ebef2d518be17"),
    (0x02199298, "977eab85922706e6"),
    (0x021992CC, "998b3c27bc3aaf25"),
    (0x021992FC, "41827941902ae50c"),
    (0x0219932C, "dd3f1ad30f8d8462"),
    (0x02199360, "bf0867e27017d716"),
    (0x02199394, "6f034e98bd80247d"),
    (0x021993C8, "7cee1436fead92b8"),
    (0x021993FC, "98d554813de55f25"),
    (0x02199430, "a166cb6b31e77824"),
    (0x02199460, "6fde322cb24ecf25"),
    (0x02199490, "dd7d075df7f1c866"),
    (0x021994C0, "3353f3ef9570e7ee"),
    (0x021994F0, "7fec70e0f08a76c1"),
    (0x02199520, "112fad61a0b3e643"),
    (0x02199550, "a36f83b38c977681"),
    (0x02199584, "982a344380a82a25"),
    (0x021995B8, "7e47a2a480a3ffb5"),
    (0x021995E8, "9fd21f0f8dfde849"),
    (0x02199618, "dc402d5dee651afe"),
    (0x0219964C, "cfdcfa2d89440108"),
    (0x02199680, "12193d906f3da0d6"),
    (0x021996B4, "ee368c036e8a378a"),
    (0x021996E8, "d031a6618783b345"),
    (0x0219971C, "8953293731bc36b8"),
    (0x02199750, "20a9b63757055799"),
    (0x02199784, "7ed23c8ae1a1b82c"),
    (0x021997B8, "483a07efdc5815a6"),
    (0x021997EC, "a1a6daae9887b36a"),
    (0x02199820, "339086ac42780a7f"),
    (0x02199854, "3588cf97221dc0ea"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/acmd2d.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0219_8E44 => anim10_start(e, v[0]) as u64,
        0x0219_8E98 => anim10_start_se(e, v[0]) as u64,
        0x0219_8EC8 => anim_wait(e, v[0]) as u64,
        0x0219_90D4 => bounce_update(e, v[0]) as u64,
        0x0219_912C => {
            vec_move_start(e, v[0], v[1], v[2], v[3], v[4], v[5]);
            0
        }
        0x0219_8E60 => anim11_start(e, v[0]) as u64,
        0x0219_8E7C => anim12_start(e, v[0]) as u64,
        0x0219_8EB0 => anim11_start_se(e, v[0]) as u64,
        0x0219_8F20 => anim_wait_keep(e, v[0]) as u64,
        0x0219_8F68 => hop_start(e, v[0]) as u64,
        0x0219_8FB0 => hop_update(e, v[0]) as u64,
        0x0219_9034 => {
            timed_anim_start(e, v[0], v[1], v[2]);
            0
        }
        0x0219_9050 => timed_anim_update(e, v[0]) as u64,
        0x0219_9070 => timed_anim_66(e, v[0]) as u64,
        0x0219_9088 => timed_anim_68(e, v[0]) as u64,
        0x0219_90A0 => timed_anim_a2(e, v[0]) as u64,
        0x0219_90B8 => bounce_start(e, v[0]) as u64,
        0x0219_91F0 => vec_move_update(e, v[0]) as u64,
        0x0219_9234 => vec_move_69(e, v[0]) as u64,
        0x0219_9268 => vec_move_6a(e, v[0]) as u64,
        0x0219_9298 => vec_move_6b(e, v[0]) as u64,
        0x0219_92CC => vec_move_6c(e, v[0]) as u64,
        0x0219_92FC => vec_move_6d(e, v[0]) as u64,
        0x0219_932C => vec_move_6e(e, v[0]) as u64,
        0x0219_9360 => vec_move_6f(e, v[0]) as u64,
        0x0219_9394 => vec_move_70(e, v[0]) as u64,
        0x0219_93C8 => vec_move_71(e, v[0]) as u64,
        0x0219_93FC => vec_move_72(e, v[0]) as u64,
        0x0219_9430 => vec_move_73(e, v[0]) as u64,
        0x0219_9460 => vec_move_74(e, v[0]) as u64,
        0x0219_9490 => vec_move_91(e, v[0]) as u64,
        0x0219_94C0 => vec_move_92(e, v[0]) as u64,
        0x0219_94F0 => vec_move_93(e, v[0]) as u64,
        0x0219_9520 => vec_move_94(e, v[0]) as u64,
        0x0219_9550 => vec_move_95(e, v[0]) as u64,
        0x0219_9584 => vec_move_96(e, v[0]) as u64,
        0x0219_95B8 => vec_move_97(e, v[0]) as u64,
        0x0219_95E8 => vec_move_98(e, v[0]) as u64,
        0x0219_9618 => vec_move_79(e, v[0]) as u64,
        0x0219_964C => vec_move_7a(e, v[0]) as u64,
        0x0219_9680 => vec_move_7b(e, v[0]) as u64,
        0x0219_96B4 => vec_move_7c(e, v[0]) as u64,
        0x0219_96E8 => vec_move_7d(e, v[0]) as u64,
        0x0219_971C => vec_move_7e(e, v[0]) as u64,
        0x0219_9750 => vec_move_7f(e, v[0]) as u64,
        0x0219_9784 => vec_move_80(e, v[0]) as u64,
        0x0219_97B8 => vec_move_81(e, v[0]) as u64,
        0x0219_97EC => vec_move_82(e, v[0]) as u64,
        0x0219_9820 => vec_move_83(e, v[0]) as u64,
        0x0219_9854 => vec_move_84(e, v[0]) as u64,
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field acmd2d`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("acmd2d", SPEC, PORTED, RECORDED), &call);
}

/// [`BOUNCE`] and [`VEC_MOVES`] are the build's.
#[test]
fn tables_match_rom() {
    let Some(w) = crate::testenv::overlay21() else {
        return;
    };
    for (k, &y) in BOUNCE.iter().enumerate() {
        assert_eq!(w(0x021D_4C1C + 4 * k as u32), y, "bounce {k}");
    }
    for (a, v) in VEC_MOVES.iter() {
        for (k, &c) in v.iter().enumerate() {
            assert_eq!(w(a + 4 * k as u32), c, "{a:#x} component {k}");
        }
    }
}
