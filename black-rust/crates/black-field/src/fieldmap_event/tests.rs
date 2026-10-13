use super::*;
use crate::hooks::Mem;
use crate::testenv::{run, WORK};
use black_fndiff::{run_suite, Case, Suite};

// Written by tools/check-fn.sh only, after the Rust hashes matched the ROM run.
// fndiff-recorded-begin
const RECORDED: &[(u32, &str)] = &[
    (0x02189BD0, "f446ade461b25521"),
    (0x0218A074, "2426c6099fe47e10"),
    (0x0218A0E4, "763e22d178e5249f"),
    (0x0218A240, "582b391865c6a210"),
    (0x0218A278, "81c3e03f2f0fda0e"),
    (0x0218A4BC, "ba6a0134901610aa"),
    (0x0218A4E0, "fb3e60ec17226ca5"),
    (0x0218A6E8, "b7326f548241febe"),
    (0x0218A728, "95938fb0be371603"),
    (0x0218A738, "e7756aada8bf8cd9"),
    (0x0218A76C, "5b18db8d4055006f"),
    (0x0218A784, "0c5c14489608ae06"),
    (0x0218A7F0, "24660538ec3a8737"),
    (0x0218A824, "e32fe7422e0a5346"),
    (0x0218A878, "a97dede35372682d"),
    (0x0218A8BC, "0219c0525ae4d71d"),
    (0x0218A9B0, "3502664c1d115e9f"),
    (0x0218AA2C, "693ae3269f265064"),
    (0x0218AA74, "6eb842dd147631b5"),
    (0x0218AAD0, "600cb4c0100300cb"),
    (0x0218AB0C, "ad01a6bc2e363ab8"),
    (0x0218AB50, "864f445502cd5070"),
    (0x0218AB9C, "14640363807814cf"),
    (0x0218AC08, "4c6e06abc5ecb588"),
    (0x0218AC4C, "e4dcec5f8e017580"),
    (0x0218AC84, "b62c97be1d4b43dc"),
    (0x0218ACA8, "31d03879db41724f"),
    (0x0218ACD0, "e46119e3b83223a7"),
    (0x0218AD28, "58119f988aec8d18"),
    (0x0218AD80, "120fb3515f4af197"),
    (0x0218ADC8, "448c0271d76eb379"),
    (0x0218AE04, "4e4b95c5c01cadb6"),
    (0x0218AE48, "dbde03b23eab1ea5"),
    (0x0218AEAC, "91d7403faadc1cd1"),
    (0x0218B208, "c9847e23bdeae0a9"),
    (0x0218B24C, "3935207239014192"),
    (0x0218B2A8, "44a0b316046301ab"),
    (0x0218B2E8, "187403372bad4756"),
    (0x0218B310, "b82090ab2b0168f4"),
    (0x0218B32C, "16b64611490564b2"),
    (0x0218B3DC, "920985fdfefd3073"),
    (0x0218B408, "9e36c42167ab5870"),
    (0x0218B56C, "9aa511423277a43e"),
    (0x0218B630, "7101fde808882528"),
    (0x0218B6AC, "f6a6f1908bdbc85d"),
    (0x0218B6F8, "d69ae53c7dc51643"),
];
// fndiff-recorded-end

const SPEC: &str = include_str!("../../../../tools/rom-diff/specs/black-field/fieldmap_event.spec");

fn call(addr: u32, c: &mut Case) -> u64 {
    run(c, SPEC, addr, |e, v| match addr {
        0x0218_9BD0 => grid_check(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_A074 => grid_entry(e, v[0], v[1]) as u64,
        0x0218_A0E4 => comm_check(e, v[0], v[1]) as u64,
        0x0218_A240 => rail_entry(e, v[0], v[1]) as u64,
        0x0218_A278 => rail_check(e, v[0], v[1]) as u64,
        0x0218_A4BC => hybrid_entry(e, v[0], v[1]) as u64,
        0x0218_A4E0 => {
            request_init(e, v[0], v[1], v[2]);
            0
        }
        0x0218_A6E8 => {
            after_event(e, v[0], v[1]);
            0
        }
        0x0218_A728 => {
            clear_02008df0(e, v[0]);
            0
        }
        0x0218_A738 => pending_script(e, v[0]) as u64,
        0x0218_A76C => talk(e, v[0]) as u64,
        0x0218_A784 => talk_dir(e, v[0], v[1]) as u64,
        0x0218_A7F0 => talk_facing(e, v[0]) as u64,
        0x0218_A824 => talk_trigger(e, v[0]) as u64,
        0x0218_A878 => push_check(e, v[0], v[1], v[2]) as u64,
        0x0218_A8BC => talk_bg(e, v[0]) as u64,
        0x0218_A9B0 => step_checks(e, v[0], v[1]) as u64,
        0x0218_AA2C => hatch_speed(e, v[0]) as u64,
        0x0218_AA74 => {
            eggs_step(e, v[0]);
            0
        }
        0x0218_AB0C => egg_ready(e, v[0]) as u64,
        0x0218_AB50 => egg_step(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_AB9C => {
            step_counters(e, v[0], v[1]);
            0
        }
        0x0218_AC08 => step_zone_event(e, v[0], v[1], v[2]) as u64,
        0x0218_AC4C => step_0200d564(e, v[0], v[1], v[2]) as u64,
        0x0218_AC84 => encounter_check(e, v[0], v[1], v[2]) as u64,
        0x0218_ACA8 => event_0218aca8(e, v[0]) as u64,
        0x0218_ACD0 => warp_check(e, v[0], v[1], v[2]) as u64,
        0x0218_AD28 => tile_event(e, v[0], v[1]) as u64,
        0x0218_AD80 => warp_event(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_ADC8 => {
            set_return(e, v[0], v[1], v[2]);
            0
        }
        0x0218_AE04 => warp_at(e, v[0], v[1]) as u64,
        0x0218_AE48 => {
            front_position(&mut e.mem, v[0], v[1]);
            0
        }
        0x0218_AEAC => is_warp_kind(e, v[0]) as u64,
        0x0218_AED4 => bump_warp(e, v[0], v[1], v[2]) as u64,
        0x0218_AF90 => bump_checks(e, v[0], v[1], v[2]) as u64,
        0x0218_B0CC => rail_warp_check(e, v[0], v[1], v[2]) as u64,
        0x0218_B13C => rail_bump_warp(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_B208 => step_encounter(e, v[0], v[1], v[2]) as u64,
        0x0218_B24C => push_down_event(e, v[0], v[1], v[2]) as u64,
        0x0218_B2A8 => {
            rail_set_return(e, v[0], v[1], v[2]);
            0
        }
        0x0218_B2E8 => rail_warp_event(e, v[0], v[1], v[2], v[3]) as u64,
        0x0218_B310 => facing_free(e, v[0], v[1]) as u64,
        0x0218_B32C => jump_step(e, v[0], v[1], v[2]) as u64,
        0x0218_B3DC => jump_event(e, v[0], v[1], v[2], v[3], v[4]) as u64,
        0x0218_B408 => ledge_check(e, v[0], v[1], v[2]) as u64,
        0x0218_B4B0 => talk_target(e, v[0], v[1]) as u64,
        0x0218_B53C => rail_talk_target(e, v[0], v[1]) as u64,
        0x0218_B56C => attribute_script(e, v[0]) as u64,
        0x0218_B630 => party_move_slot(e, v[0], v[1]) as u64,
        0x0218_B6AC => party_count_a0(e, v[0]) as u64,
        0x0218_B6F8 => jump_tiles_free(e, v[0], v[1]) as u64,
        0x0218_AAD0 => {
            egg_counter(e, v[0], v[1], 0);
            0
        }
        _ => unreachable!("{addr:#010x} is PORTED but not wired into the test"),
    })
}

/// Every ported function against the original (`tools/check-fn.sh black-field fieldmap_event`).
#[test]
fn fndiff_match_rom() {
    run_suite(&Suite::new("fieldmap_event", SPEC, PORTED, RECORDED), &call);
}

#[test]
fn front_tile_moves_one_unit() {
    let mut bytes = vec![0u8; 0x100];
    bytes[REQ_POS as usize..REQ_POS as usize + 4].copy_from_slice(&(WORK + 0x40).to_le_bytes());
    for (i, v) in [0x0002_0000u32, 0x10, 0x0003_0000].iter().enumerate() {
        bytes[0x40 + 4 * i..0x44 + 4 * i].copy_from_slice(&v.to_le_bytes());
    }
    let mut mem = Mem::new(WORK, bytes);
    mem.w16(WORK + REQ_FACING, 2);
    front_position(&mut mem, WORK, WORK + 0x80);
    assert_eq!(
        (
            mem.r32(WORK + 0x80),
            mem.r32(WORK + 0x84),
            mem.r32(WORK + 0x88)
        ),
        (0x0001_0000, 0x10, 0x0003_0000)
    );
    mem.w16(WORK + REQ_FACING, 1);
    front_position(&mut mem, WORK, WORK + 0x80);
    assert_eq!(mem.r32(WORK + 0x88), 0x0004_0000);
}
