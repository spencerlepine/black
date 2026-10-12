//! Flow core, part 5 (0x021C4838-0x021C65A8; B10j).
//!
//! The turn-end / per-mon passes (`4838`, `4d1c`, `4f64`, `5108`, `5200`, `5308`, `5350`), the message
//! round trips of the damage tail (`4b3c`, `4e38`, `4ea4`, `62d8`, `6350`, `63d0`, `6458`, ...) and the
//! command / message emitters (`5480`..`6134`). Specs: `tools/rom-diff/svflow_specs/core5.spec`
//! (66 functions; the 24 `bx pc` veneers of the range, their thunks and `4b8c` are listed at the top of
//! the spec).
//!
//! B10j: all 66 specced functions are ported and verified (in [`PORTED`], 66/66 on
//! `tools/check-svflow.sh core5`). Callees outside this area go through [`FlowEnv`]; `cmd_queue` ones
//! and the in-area functions that are in `PORTED` are called directly (they run as real code on both
//! sides of the diff).
//!
//! B10-spec4: the 12 functions that were ported but unverified (`5620`, `5718`, `572c`, `5798`, `5dac`,
//! `62d8`, `6350`, `6394`, `63d0`, `6458`, `64b4`, `6564`) are registered now that their caller stubs in
//! core1-core4 are corrected; `5418` calls `5620` directly.

use super::cmd_queue;
use super::core2::Cx;
use super::env::{Arg, FlowEnv};
use super::SvFlowWork;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_5A1C,
    0x021C_56A8,
    0x021C_5098,
    0x021C_4BAC,
    0x021C_4B3C,
    0x021C_4910,
    0x021C_51D0,
    0x021C_4E38,
    0x021C_4EA4,
    0x021C_5108,
    0x021C_4F64,
    0x021C_5350,
    0x021C_5308,
    0x021C_5200,
    0x021C_4838,
    0x021C_4D1C,
    0x021C_4C24,
    0x021C_4C50,
    0x021C_4C40,
    0x021C_4B9C,
    0x021C_4A64,
    0x021C_5418,
    0x021C_5474,
    0x021C_5480,
    0x021C_54BC,
    0x021C_54C8,
    0x021C_54FC,
    0x021C_5540,
    0x021C_557C,
    0x021C_5604,
    0x021C_5750,
    0x021C_5774,
    0x021C_5840,
    0x021C_59C4,
    0x021C_5A34,
    0x021C_5A4C,
    0x021C_5A8C,
    0x021C_5B74,
    0x021C_5BE4,
    0x021C_5C1C,
    0x021C_5C74,
    0x021C_5D88,
    0x021C_5E48,
    0x021C_5EFC,
    0x021C_5F98,
    0x021C_604C,
    0x021C_6080,
    0x021C_60D4,
    0x021C_60EC,
    0x021C_6110,
    0x021C_6134,
    0x021C_6158,
    0x021C_6190,
    0x021C_493C,
    // B10-spec4: the callers' stubs in core1-core4 were fixed (arity / kinds / `force`)
    0x021C_5620,
    0x021C_5718,
    0x021C_572C,
    0x021C_5798,
    0x021C_5DAC,
    0x021C_62D8,
    0x021C_6350,
    0x021C_6394,
    0x021C_63D0,
    0x021C_6458,
    0x021C_64B4,
    0x021C_6564,
];

/// `work + 0x4`: the main module (address only).
const OFS_MAIN: usize = 0x4;
/// `work + 0x8`: the party container (address only).
const OFS_POKECON: usize = 0x8;
/// `work + 0x1D74`: the packed queue-header word that the record brackets reserve / restore.
const OFS_QUEUE_HEADER: usize = 0x1D74;
/// `work + 0x78A`: flag bits (`|= 2` by the end-of-turn pass, `|= 0x10` by `5c74`).
const OFS_FLAGS_78A: usize = 0x78A;
/// `work + 0x7A9`: one flag byte per mon id (`4f64`).
const OFS_MON_DONE: usize = 0x7A9;
/// `work + 0x789`: a mon id byte (`4f64`).
const OFS_ID_789: usize = 0x789;
/// `work + 0x868`: list object (value only).
const OFS_LIST_868: usize = 0x868;
/// `work + 0x1B08`: six 12-byte records (`5350`).
const OFS_RECORDS: usize = 0x1B08;
/// `work + 0x220`: u32 cleared by `5474`.
const OFS_WORD_220: usize = 0x220;
/// `work + 0x782`: u8 count of the 16-byte entries at `work + 0x7E0`; `+0x788`: u8 result of `6190`.
const OFS_ENTRY_COUNT: usize = 0x782;
const OFS_ENTRIES: usize = 0x7E0;
const OFS_RESULT_788: usize = 0x788;
/// End-of-list marker of the variable-length commands (`cmd_queue::put_cmd_list`).
const LIST_END: u32 = 0xFFFF_0000;

/// Puts command `cmd` with `args` (`021cd4b0`) on the queue of `cx`.
fn put<E: FlowEnv>(env: &mut E, cx: &mut Cx, cmd: u32, args: &[u32]) {
    cmd_queue::put_cmd(env, cx.q, cmd, args, cx.scratch);
}

/// Puts the list-terminated command `cmd` (`021cd64c`): `w`, then `rest` (which holds the terminator).
fn put_list<E: FlowEnv>(env: &mut E, cx: &mut Cx, cmd: u32, w: u32, rest: &[u32]) {
    cmd_queue::put_cmd_list(env, cx.q, cmd, w, rest);
}

/// `021cc8d8(work + 0x1D74, _)`: opens a record bracket on the queue-header word, returns the old word.
fn bracket_open(w: &mut SvFlowWork) -> u32 {
    cmd_queue::advance_fields(&mut w.mem[OFS_QUEUE_HEADER..])
}

/// `021cc95c(work + 0x1D74, old, _)`: closes the bracket.
fn bracket_close(w: &mut SvFlowWork, old: u32) {
    cmd_queue::set_word(&mut w.mem[OFS_QUEUE_HEADER..], old);
}

/// Message ids of [`put_selector_message`]: selector 0..=8 -> 0x7E + 4 * selector, 11..=13 -> 0xA6 + 4 * (selector - 11);
/// selectors 9 and 10 (and anything above 13) have none.
fn message_id(sel: u32) -> Option<u32> {
    match sel {
        0..=8 => Some(0x7E + 4 * sel),
        11..=13 => Some(0xA6 + 4 * (sel - 11)),
        _ => None,
    }
}

/// `FUN_overlay_d_93__021d5440`: the id (0..5) of a BattleMon.
fn mon_id<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// C: `FUN_overlay_d_93__021c5a1c`: puts command 0x57 (the mon's id) on the command queue `q`
/// (`scratch` = the 0x40 bytes at 0x021F3B44 that `cmd_queue::put_cmd` fills).
pub fn cmd_57<E: FlowEnv>(env: &mut E, q: &mut [u8], scratch: &mut [u8], mon: u32) {
    let id = mon_id(env, mon);
    cmd_queue::put_cmd(env, q, 0x57, &[id], scratch);
}

/// C: `FUN_overlay_d_93__021c56a8`: decodes the event-variable word `*word` for `mon` (`62a8(mon)` = the
/// stored value `x`). Word 0x24: `x` when non-zero; 0x25: `x` (and the word becomes 6) when non-zero,
/// else 6 when `62c4(mon, 6)` says so; 0x26: the result of `7e10(mon)` when non-zero (the word is
/// left alone); any other word: the word itself when `62c4(mon, word)` says so. The word is cleared
/// (0, or 6 in the second case) except for 0x26; the result is also returned.
pub fn decode_event_var<E: FlowEnv>(env: &mut E, mon: u32, word: &mut [u8]) -> u32 {
    let stored = env.call(0x021D_62A8, &mut [Arg::V(mon)]);
    let w = u32::from_le_bytes(word[..4].try_into().unwrap());
    let mut result = 0u32;
    let mut new_word = 0u32;
    if (w as i32) == 0 {
        return 0;
    }
    if (w as i32) > 0 && (0x24..=0x26).contains(&w) {
        match w {
            0x24 => {
                if stored != 0 {
                    result = stored;
                }
            }
            0x25 => {
                if stored != 0 {
                    result = stored;
                    new_word = 6;
                } else if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(6)]) != 0 {
                    result = 6;
                }
            }
            _ => {
                let r = env.call(0x021D_7E10, &mut [Arg::V(mon)]);
                if r != 0 {
                    return r;
                }
                word[..4].copy_from_slice(&result.to_le_bytes());
                return result;
            }
        }
    } else if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(w)]) != 0 {
        result = w;
    }
    word[..4].copy_from_slice(&new_word.to_le_bytes());
    result
}

/// C: `FUN_overlay_d_93__021c5098`: the highest BattleMon field 0xF (`59b4`) over the party members
/// whose slot `0..=8640(main)` passes `902c(8ff4(main), 8de4(main, slot)) == 0` and whose `50f8(work + 0x1AB8,
/// slot)` is not 0x1F; at least 1.
pub fn max_level<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let pokecon = w.u32_at(OFS_POKECON);
    let mut best = 1;
    let last = env.call(0x021B_8640, &mut [Arg::V(main)]);
    let side = env.call(0x021B_8FF4, &mut [Arg::V(main)]);
    let mut slot = 0u32;
    loop {
        let idx = slot & 0xFF;
        let other = env.call(0x021B_8DE4, &mut [Arg::V(main), Arg::V(idx)]);
        if env.call(0x021B_902C, &mut [Arg::V(side), Arg::V(other)]) == 0 {
            let r = env.call(0x021C_50F8, &mut [Arg::P, Arg::V(idx)]);
            if r != 0x1F {
                let mon = env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(r)]);
                let v = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xF)]);
                if best < v {
                    best = v;
                }
            }
        }
        slot = slot.wrapping_add(1);
        if slot > last {
            break;
        }
    }
    best
}

/// C: `FUN_overlay_d_93__021c4bac`: puts the 0x5A message for selector `sel` (ids in [`message_id`]) with `param`
/// on the command queue `q` (the buffer at `work + 0xC`); nothing for the selectors without a message.
pub fn put_selector_message<E: FlowEnv>(env: &mut E, q: &mut [u8], sel: u32, param: u32) {
    if let Some(id) = message_id(sel) {
        cmd_queue::put_cmd_list(env, q, 0x5A, id, &[param, LIST_END]);
    }
}

/// C: `FUN_overlay_d_93__021c4b3c`: message 0x2649 round trip: argument 2 = the id of `mon`, 0x1D = `a`,
/// word slot 0x32 = `b`, sent with 0x6B; returns the reply in slot 0x32.
pub fn message_round_trip<E: FlowEnv>(env: &mut E, mon: u32, a: u32, b: u32) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x2649)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x1D), Arg::V(a)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x32), Arg::V(b)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x6B)]);
    let reply = env.call(0x021D_775C, &mut [Arg::V(0x32)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2649 + 6)]);
    reply
}

/// C: `FUN_overlay_d_93__021c4b9c`: callback trampoline: tail-jumps to `4bac(work, sel, param)` with
/// its registers rotated (`param` first, `sel` second, work third).
pub fn put_selector_message_cb<E: FlowEnv>(env: &mut E, q: &mut [u8], param: u32, sel: u32) {
    put_selector_message(env, q, sel, param);
}

/// C: `FUN_overlay_d_93__021c4c40`: callback trampoline: tail-jumps to `4c50(work, sel)` (swapped
/// arguments).
pub fn sel_message_cb<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, sel: u32) {
    sel_message(env, w, cx, sel);
}

// ---- message round trips ---------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c4910`: message 0x25C3 with argument 2 = `id`, sent to `target` (`70f0`).
pub fn mon_message<E: FlowEnv>(env: &mut E, id: u32, target: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(0x25C3)]);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(target)]);
    env.call(0x021D_7540, &mut [Arg::V(0x25C3 + 3)]);
}

/// C: `FUN_overlay_d_93__021c51d0`: message 0x27DF with slot 2 = the id of `mon`, sent with 0xA3.
pub fn mon_message_a3<E: FlowEnv>(env: &mut E, mon: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(0x27DF)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75A8, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0xA3)]);
    env.call(0x021D_7540, &mut [Arg::V(0x27DF + 3)]);
}

/// C: `FUN_overlay_d_93__021c4e38`: message 0x26E8 round trip (argument 2 = the id of `mon`, 0x39 = `a`,
/// 0x41 = 0 as byte, slot 0x32 = `b`, sent with 0x7F): the reply of slot 0x32 when the byte slot
/// 0x41 came back as 0, else 0.
pub fn message_26e8<E: FlowEnv>(env: &mut E, mon: u32, a: u32, b: u32) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x26E8)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x39), Arg::V(a)]);
    env.call(0x021D_7630, &mut [Arg::V(0x41), Arg::V(0)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x32), Arg::V(b)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x7F)]);
    let reply = env.call(0x021D_775C, &mut [Arg::V(0x32)]);
    let flag = env.call(0x021D_775C, &mut [Arg::V(0x41)]);
    env.call(0x021D_7540, &mut [Arg::V(0x26E8 + 9)]);
    if flag & 0xFF == 0 {
        reply
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021c4ea4`: sets up the hem record (`work + 0x1AE4`) by `kind` (3 and 4 push
/// the message 399 / 0x18C through `8838` / `8860`, anything else resets it with `8810`); then, for
/// `damage > 0` when `20bc(work, mon, damage)` agrees, runs the 0x290 / 0x255 message commands
/// (`70e4`) and `20d0`.
pub fn hem_damage_messages<E: FlowEnv>(env: &mut E, mon: u32, kind: u32, damage: i32) {
    let id = mon_id(env, mon);
    let msg = match kind {
        3 => Some(399),
        4 => Some(0x18C),
        _ => None,
    };
    match msg {
        Some(m) => {
            env.call(0x021C_8838, &mut [Arg::P, Arg::V(2), Arg::V(m)]);
            env.call(0x021C_8860, &mut [Arg::P, Arg::V(id)]);
        }
        None => {
            env.call(0x021C_8810, &mut [Arg::P]);
        }
    }
    if damage > 0
        && env.call(
            0x021C_20BC,
            &mut [Arg::P, Arg::V(mon), Arg::V(damage as u32)],
        ) != 0
    {
        env.call(0x021C_99A0, &mut [Arg::P, Arg::P]);
        env.call(0x021C_8810, &mut [Arg::P]);
        let text = env.call(0x021C_8010, &mut [Arg::P, Arg::V(id)]);
        env.call(
            0x021C_70E4,
            &mut [
                Arg::P,
                Arg::V(0x290),
                Arg::V(text),
                Arg::V(6),
                Arg::V(0),
                Arg::V(0),
            ],
        );
        if (env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)]) as i32) <= damage {
            env.call(
                0x021C_70E4,
                &mut [
                    Arg::P,
                    Arg::V(0x255),
                    Arg::V(6),
                    Arg::V(6),
                    Arg::V(0),
                    Arg::V(0),
                ],
            );
        }
        env.call(
            0x021C_20D0,
            &mut [Arg::P, Arg::V(mon), Arg::V(damage as u32), Arg::P],
        );
    }
}

// ---- per-mon pass --------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c5108`: the per-mon follow-up pass: message `51d0`, `bff60`, the per-species
/// calls (`d8510` / `dcf24` when `flag == 0`, else `d8538` / `dcfa4`; then `e0434`), the `bd2b4` /
/// `bd338` / `bd34c` walk (command 0x2C for each mon it yields), `ef90c`, command 0x23 and, when the
/// battle is not over and BattleMon field 0x11 is 0x7F, `3fdc`.
pub fn mon_pass<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, flag: u32) {
    let id = mon_id(env, mon);
    mon_message_a3(env, mon);
    env.call(0x021B_FF60, &mut [Arg::P, Arg::V(mon), Arg::V(0)]);
    if flag == 0 {
        env.call(0x021D_8510, &mut [Arg::V(mon)]);
        env.call(0x021D_CF24, &mut [Arg::V(mon)]);
    } else {
        env.call(0x021D_8538, &mut [Arg::V(mon)]);
        env.call(0x021D_CFA4, &mut [Arg::V(mon)]);
    }
    env.call(0x021E_0434, &mut [Arg::V(mon)]);
    env.call(0x021B_D2B4, &mut [Arg::P, Arg::P]);
    env.call(0x021B_D338, &mut [Arg::P, Arg::P]);
    loop {
        let mut out = [0u8; 4];
        let more = env.call(0x021B_D34C, &mut [Arg::P, Arg::P, Arg::Out(&mut out)]);
        if more == 0 {
            break;
        }
        let other = u32::from_le_bytes(out);
        env.call(0x021D_6264, &mut [Arg::V(other), Arg::V(id)]);
        let other_id = mon_id(env, other);
        put(env, cx, 0x2C, &[other_id, id]);
    }
    env.call(0x021E_F90C, &mut [Arg::V(id)]);
    put(env, cx, 0x23, &[id]);
    if env.call(0x021B_C2B0, &mut [Arg::P]) == 0
        && env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0x11)]) as u16 as i16 == 0x7F
    {
        env.call(0x021C_3FDC, &mut [Arg::P, Arg::V(mon)]);
    }
}

/// C: `FUN_overlay_d_93__021c4f64`: the once-per-mon pass: when the flag byte `work[0x7A9 + id]` is 0
/// and `d5b68(mon)` says so, sets it, runs `5078`, puts command 0x5B (unless `ba1a8` / `8d08` say the
/// mon needs no message or `work[0x789]` is the mon) and command 0x39, then `5108(mon, 0)`, `d646c`,
/// the level cap bookkeeping (`b92b0`) for the main side, `5088`, and the `ba1b4` side notifications.
/// Returns 1 when it ran.
pub fn mon_once<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, mon: u32) -> u32 {
    let id = mon_id(env, mon);
    let main = w.u32_at(OFS_MAIN);
    if w.u8_at(OFS_MON_DONE + id as usize) != 0 || env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0 {
        return 0;
    }
    w.set_u8_at(OFS_MON_DONE + id as usize, 1);
    env.call(0x021C_5078, &mut [Arg::P, Arg::V(id)]);
    let silent = env.call(0x021B_A1A8, &mut [Arg::V(main), Arg::V(0x200)]) != 0
        && env.call(0x021B_8D08, &mut [Arg::V(id)]) != 0;
    if !silent && w.u8_at(OFS_ID_789) as u32 != id {
        put_list(env, cx, 0x5B, 0, &[id, LIST_END]);
    }
    put(env, cx, 0x39, &[id]);
    mon_pass(env, cx, mon, 0);
    env.call(0x021D_646C, &mut [Arg::V(mon)]);
    let side = env.call(0x021B_8D08, &mut [Arg::V(id)]);
    let main_side = env.call(0x021B_8FF4, &mut [Arg::V(main)]);
    if side == main_side {
        let level = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xF)]);
        let cap = max_level(env, w);
        env.call(
            0x021B_92B0,
            &mut [
                Arg::V(main),
                Arg::V(mon),
                Arg::V(((level as i32).wrapping_add(0x1E) <= cap as i32) as u32),
            ],
        );
    }
    env.call(0x021C_5088, &mut [Arg::P, Arg::V(id)]);
    let side = env.call(0x021B_8D08, &mut [Arg::V(id)]);
    let main_side = env.call(0x021B_8FF4, &mut [Arg::V(main)]);
    let code = if side == main_side {
        0x4C
    } else {
        if env.call(0x021B_902C, &mut [Arg::V(side), Arg::V(main_side)]) != 0 {
            return 1;
        }
        env.call(0x021B_A1B4, &mut [Arg::V(main), Arg::V(0x19)]);
        0x52
    };
    env.call(0x021B_A1B4, &mut [Arg::V(main), Arg::V(code)]);
    1
}

/// C: `FUN_overlay_d_93__021c5350`: for each of the six 12-byte records at `work + rec` with a non-zero
/// word at `+0` (and BattleMon field 0xF of the mon `b9bf0(side, i)` below 100): message 0x5A
/// (0x2A / 0x2B by the byte at `+4`) and command 0x3E / 0x45 with the bytes at `+5`..`+10` of the
/// **first** record, the `69b8` loop and `ba1cc`. Returns 1 when one fired, else the last record
/// word read (0 at the start).
pub fn record_messages<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    side: u32,
    rec: usize,
) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let mut ret = 0u32;
    for i in 0..6usize {
        let at = rec + i * 12;
        if w.u32_at(at) == 0 {
            continue;
        }
        let mon = env.call(0x021B_9BF0, &mut [Arg::V(side), Arg::V(i as u32)]);
        if (env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xF)]) as i32) < 100 {
            ret = w.u32_at(at);
            let msg = if w.u8_at(at + 4) == 0 { 0x2A } else { 0x2B };
            let id = mon_id(env, mon);
            put_list(env, cx, 0x5A, msg, &[id, ret, LIST_END]);
            let b: [u32; 6] = [5, 6, 7, 8, 9, 10].map(|k| w.u8_at(rec + k) as u32);
            put(env, cx, 0x3E, &[id, b[0], b[1], b[2], b[3], b[4], b[5]]);
            loop {
                let word = ret.to_le_bytes();
                let r = env.call(0x021D_69B8, &mut [Arg::V(mon), Arg::In(&word), Arg::P]);
                if r == 0 {
                    break;
                }
            }
            put(env, cx, 0x45, &[id, ret, b[1], b[2], b[3], b[4], b[5]]);
            env.call(0x021B_A1CC, &mut [Arg::V(main), Arg::V(0x1B), Arg::V(ret)]);
            ret = 1;
        }
    }
    ret
}

/// C: `FUN_overlay_d_93__021c5308`: for a mon whose species check `b9a40(id) == 1` passes: `cb2d4`
/// fills the six records at `work + 0x1B08` for the side `b9aec(pokecon, b8ff4(main))`, then
/// [`record_messages`] reports them.
pub fn species_gate<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, mon: u32) -> u32 {
    let id = mon_id(env, mon);
    if env.call(0x021B_9A40, &mut [Arg::V(id)]) & 0xFF != 1 {
        return 0;
    }
    let main_side = env.call(0x021B_8FF4, &mut [Arg::V(w.u32_at(OFS_MAIN))]);
    let side = env.call(
        0x021B_9AEC,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(main_side)],
    );
    env.call(
        0x021C_B2D4,
        &mut [Arg::P, Arg::V(side), Arg::V(mon), Arg::P],
    );
    record_messages(env, w, cx, side, OFS_RECORDS)
}

/// C: `FUN_overlay_d_93__021c5200`: the end-of-turn flag pass over the entries of the list at
/// `work + 0x3E0` (when `b85ec(main)` allows it): for each entry that `52d8` does not skip, puts
/// command 0x55 once (sets `work[0x78A] |= 2`) when the main side conditions hold, runs
/// [`species_gate`] on its mon and clears the entry (`52f8`). Returns 1 if any gate fired.
pub fn end_of_turn_pass<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let mut ret = 0;
    if env.call(0x021B_85EC, &mut [Arg::V(main)]) == 0 {
        return 0;
    }
    let count = env.call(0x021C_52C8, &mut [Arg::P, Arg::V(0)]);
    let mut i = 0u32;
    if count != 0 {
        loop {
            let idx = i & 0xFF;
            if env.call(0x021C_52D8, &mut [Arg::P, Arg::V(0), Arg::V(idx)]) == 0 {
                let slot = env.call(0x021C_52E8, &mut [Arg::P, Arg::V(0), Arg::V(idx)]);
                let mon = env.call(
                    0x021B_9934,
                    &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(slot)],
                );
                if env.call(0x021B_86B8, &mut [Arg::V(main)]) == 0
                    && w.u8_at(OFS_FLAGS_78A) & 2 == 0
                    && env.call(0x021B_C2B0, &mut [Arg::P]) != 0
                    && env.call(0x021B_C310, &mut [Arg::P]) != 0
                {
                    let side = env.call(0x021B_86DC, &mut [Arg::V(main)]);
                    put(env, cx, 0x55, &[side]);
                    let f = w.u8_at(OFS_FLAGS_78A);
                    w.set_u8_at(OFS_FLAGS_78A, f | 2);
                }
                if species_gate(env, w, cx, mon) != 0 {
                    ret = 1;
                }
                env.call(0x021C_52F8, &mut [Arg::P, Arg::V(0), Arg::V(idx)]);
            }
            i = i.wrapping_add(1);
            if i >= count {
                break;
            }
        }
    }
    ret
}

// ---- list walks ----------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c4838`: the first list walk (`list` = the veneer-iterated list object):
/// record bracket 0x25AC around `mon_message(0x1f, sel)`, then for each entry (until the battle
/// ends) a bracket around `mon_message(id, sel)` and [`mon_once`]; finally [`end_of_turn_pass`].
pub fn walk_list<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, list: u32, sel: u32) {
    env.call(0x021C_48E0, &mut [Arg::V(list)]);
    let old = bracket_open(w);
    mon_message(env, 0x1F, sel);
    bracket_close(w, old);
    if env.call(0x021B_C2B0, &mut [Arg::P]) == 0 {
        let mut mon = env.call(0x021C_48F0, &mut [Arg::V(list)]);
        while mon != 0 {
            let old = bracket_open(w);
            let id = mon_id(env, mon);
            mon_message(env, id, sel);
            bracket_close(w, old);
            mon_once(env, w, cx, mon);
            if env.call(0x021B_C2B0, &mut [Arg::P]) != 0 {
                break;
            }
            mon = env.call(0x021C_4900, &mut [Arg::V(list)]);
        }
    }
    end_of_turn_pass(env, w, cx);
}

/// C: `FUN_overlay_d_93__021c493c`: the per-list damage record: the ids of the `4a24(list)` mons of `list` (`4a34`) in a
/// byte array, then for every entry `e` of the `4a44(k)` walk (until the battle ends) and every one of those mons that
/// `d5b68` does not skip and `d6064(mon, e, &a, &b)` accepts, `4a54(mon, e, a, b, work)` inside a record bracket 0x25F7;
/// finally command 0x15 (count, walk steps, `d806c(ids)`) and [`end_of_turn_pass`].
pub fn pair_pass<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, list: u32) {
    // 8 bytes on the original's stack: bytes 4 = 3 and the rest 0 (the last two are never written)
    let mut ids = [0u8, 0, 0, 0, 3, 0, 0, 0];
    let count = env.call(0x021C_4A24, &mut [Arg::V(list)]);
    for i in 0..count {
        let mon = env.call(0x021C_4A34, &mut [Arg::V(list), Arg::V(i)]);
        ids[i as usize] = mon_id(env, mon) as u8;
    }
    let pokecon = w.u32_at(OFS_POKECON);
    let mut step = 0u32;
    loop {
        let entry = env.call(0x021C_4A44, &mut [Arg::V(step)]);
        step += 1;
        if entry == 0 {
            break;
        }
        for &id in ids.iter().take(count as usize) {
            let mon = env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(id as u32)]);
            if env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0 {
                continue;
            }
            let (mut a, mut b) = ([0u8; 4], [0u8; 4]);
            if env.call(
                0x021D_6064,
                &mut [
                    Arg::V(mon),
                    Arg::V(entry),
                    Arg::Out(&mut a),
                    Arg::Out(&mut b),
                ],
            ) != 0
            {
                let old = bracket_open(w);
                env.call(
                    0x021C_4A54,
                    &mut [
                        Arg::V(mon),
                        Arg::V(entry),
                        Arg::V(u32::from_le_bytes(a)),
                        Arg::V(u32::from_le_bytes(b)),
                        Arg::V(cx.w_addr),
                    ],
                );
                bracket_close(w, old);
            }
        }
        if env.call(0x021B_C2B0, &mut [Arg::P]) != 0 {
            break;
        }
    }
    let packed = env.call(0x021D_806C, &mut [Arg::In(&ids)]);
    put(env, cx, 0x15, &[count & 0xFF, step & 0xFF, packed]);
    end_of_turn_pass(env, w, cx);
}

/// C: `FUN_overlay_d_93__021c4d1c`: the second list walk: when `ef8b8` says the walk is blocked puts
/// command 0x40 and `3be0(work, 0)` and returns 0; otherwise for every listed mon that is not
/// `d5b68` and has neither status 5 nor 4 (`d5bdc`), a bracket 0x26C6 around [`message_26e8`] with
/// the `2ab0` mode and `d7bd4`, [`hem_damage_messages`] when it answered, then [`mon_once`]; ends
/// with `70e4(0x255, 6, 6, 0, 0)` if any fired and [`end_of_turn_pass`] (returned).
pub fn walk_list2<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, list: u32) -> u32 {
    let blocked = env.call(0x021E_F8B8, &mut [Arg::P]);
    if blocked != 0 {
        put(env, cx, 0x40, &[blocked]);
        env.call(0x021C_3BE0, &mut [Arg::P, Arg::V(0)]);
        return 0;
    }
    let mut fired = false;
    let mode = env.call(0x021C_2AB0, &mut [Arg::P]);
    env.call(0x021C_4E08, &mut [Arg::V(list)]);
    let mut mon = env.call(0x021C_4E18, &mut [Arg::V(list)]);
    while mon != 0 {
        if env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0
            && env.call(0x021D_5BDC, &mut [Arg::V(mon), Arg::V(5)]) == 0
            && env.call(0x021D_5BDC, &mut [Arg::V(mon), Arg::V(4)]) == 0
        {
            let old = bracket_open(w);
            let b = env.call(0x021D_7BD4, &mut [Arg::V(mon), Arg::V(mode)]);
            let r = message_26e8(env, mon, mode, b);
            if r != 0 {
                hem_damage_messages(env, mon, mode, r as i32);
                fired = true;
            }
            bracket_close(w, old);
            mon_once(env, w, cx, mon);
        }
        mon = env.call(0x021C_4E28, &mut [Arg::V(list)]);
    }
    if fired {
        env.call(
            0x021C_70E4,
            &mut [
                Arg::P,
                Arg::V(0x255),
                Arg::V(6),
                Arg::V(6),
                Arg::V(0),
                Arg::V(0),
            ],
        );
    }
    end_of_turn_pass(env, w, cx)
}

/// C: `FUN_overlay_d_93__021c4c24`: registers the callback `0x021C4C41` (`ef934`, with the work
/// address) and puts command 0x2F with argument 0.
pub fn register_callback<E: FlowEnv>(env: &mut E, cx: &mut Cx) {
    env.call(0x021E_F934, &mut [Arg::V(0x021C_4C41), Arg::V(cx.w_addr)]);
    put(env, cx, 0x2F, &[0]);
}

/// C: `FUN_021C4C50`: message `0x5A` by `sel` (1 -> 0x74, 2 -> 0x76, 6 -> 0xB3, 7 -> 0xB5, other
/// none), command 0x24 with `sel & 0xff` and, for 7, `43f0` on the list object at `work + 0x868`,
/// then `195c(mon, 0)` for each entry `5b7c` accepts (list walked with the `4cec` / `4cfc` / `4d0c`
/// veneers).
pub fn sel_message<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, sel: u32) {
    let msg: i32 = match sel {
        1 => 0x74,
        2 => 0x76,
        6 => 0xB3,
        7 => 0xB5,
        _ => -1,
    };
    if msg >= 0 {
        put_list(env, cx, 0x5A, msg as u32, &[LIST_END]);
    }
    put(env, cx, 0x24, &[sel & 0xFF]);
    if sel == 7 {
        let list = w.u32_at(OFS_LIST_868);
        env.call(0x021C_43F0, &mut [Arg::P, Arg::V(list)]);
        env.call(0x021C_4CEC, &mut [Arg::V(list)]);
        let mut mon = env.call(0x021C_4CFC, &mut [Arg::V(list)]);
        while mon != 0 {
            if env.call(0x021D_5B7C, &mut [Arg::V(mon)]) != 0 {
                env.call(0x021C_195C, &mut [Arg::P, Arg::V(mon), Arg::V(0)]);
            }
            mon = env.call(0x021C_4D0C, &mut [Arg::V(list)]);
        }
    }
}

// ---- small command emitters ----------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c5474`: `*(u32 *)(work + 0x220) = 0`.
pub fn clear_220(w: &mut SvFlowWork) {
    w.set_u32_at(OFS_WORD_220, 0);
}

/// C: `FUN_overlay_d_93__021c5480`: when `54ac(work + 0x1AB8, id)` (a slot, 6 = none) is not 6, puts
/// command 0x4D (slot, `msg`).
pub fn put_slot_message<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, msg: u32) {
    let id = mon_id(env, mon);
    let slot = env.call(0x021C_54AC, &mut [Arg::P, Arg::V(id)]);
    if slot != 6 {
        put(env, cx, 0x4D, &[slot, msg]);
    }
}

/// C: `FUN_overlay_d_93__021c54bc`: tail jump to `5d88(work, mon, 0x350)`.
pub fn put_message_350<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    put_mon_message(env, cx, mon, 0x350);
}

/// C: `FUN_overlay_d_93__021c54c8`: [`put_slot_message`] with 0x25B, then message 0x15C for the mon.
pub fn put_slot_message_25b<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    put_slot_message(env, cx, mon, 0x25B);
    let id = mon_id(env, mon);
    put_list(env, cx, 0x5B, 0x15C, &[id, LIST_END]);
}

/// C: `FUN_overlay_d_93__021c54fc`: reads the stored value `62d8(mon, 7)` (low byte), [`put_slot_message`]
/// with 0x25C, then message 0x14D with the mon id and that byte.
pub fn put_slot_message_25c<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    let v = env.call(0x021D_62D8, &mut [Arg::V(mon), Arg::V(7)]);
    put_slot_message(env, cx, mon, 0x25C);
    let id = mon_id(env, mon);
    put_list(env, cx, 0x5B, 0x14D, &[id, v & 0xFF, LIST_END]);
}

/// C: `FUN_overlay_d_93__021c5d88`: message `msg` (0x5B) for the mon.
pub fn put_mon_message<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, msg: u32) {
    let id = mon_id(env, mon);
    put_list(env, cx, 0x5B, msg, &[id, LIST_END]);
}

/// C: `FUN_overlay_d_93__021c5604`: command 0x59 (mon id, `a`; the fourth argument is not used by the
/// two-word command).
pub fn cmd_59<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, a: u32) {
    let id = mon_id(env, mon);
    put(env, cx, 0x59, &[id, a]);
}

/// C: `FUN_overlay_d_93__021c5a34`: command 0x58 (mon id).
pub fn cmd_58<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    let id = mon_id(env, mon);
    put(env, cx, 0x58, &[id]);
}

/// C: `FUN_overlay_d_93__021c60d4`: command 0x42 (mon id).
pub fn cmd_42<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    let id = mon_id(env, mon);
    put(env, cx, 0x42, &[id]);
}

/// C: `FUN_overlay_d_93__021c5750`: message 0xD2 for the mon.
pub fn message_d2<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    put_mon_message(env, cx, mon, 0xD2);
}

/// C: `FUN_overlay_d_93__021c5774`: message 0xD8 for the mon.
pub fn message_d8<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    put_mon_message(env, cx, mon, 0xD8);
}

/// C: `FUN_overlay_d_93__021c572c`: message 0xD5 for the mon.
pub fn message_d5<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    put_mon_message(env, cx, mon, 0xD5);
}

/// C: `FUN_overlay_d_93__021c5718`: message 0x47 (no mon).
pub fn message_47<E: FlowEnv>(env: &mut E, cx: &mut Cx) {
    put_list(env, cx, 0x5A, 0x47, &[LIST_END]);
}

/// C: `FUN_overlay_d_93__021c557c`: `d61f0(mon)`, commands 0x10 and 0x35 (id, 0), and for `extra != 0`
/// the message 0x138 / 0x126 / 0x156 chosen by `kind` (2 / 3 / other) with the mon id.
pub fn cmd_10_35<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, kind: u32, extra: u32) {
    let id = mon_id(env, mon);
    env.call(0x021D_61F0, &mut [Arg::V(mon)]);
    put(env, cx, 0x10, &[id]);
    put(env, cx, 0x35, &[id, 0]);
    if extra != 0 {
        let msg = match kind {
            2 => 0x138,
            3 => 0x126,
            _ => 0x156,
        };
        put_list(env, cx, 0x5B, msg, &[id, LIST_END]);
    }
}

/// C: `FUN_overlay_d_93__021c5a4c`: message 0x99 (`dir > 0`) or 0xAE with the mon id and `a`.
pub fn message_99_ae<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, a: u32, dir: i32) {
    let id = mon_id(env, mon);
    let msg = if dir > 0 { 0x99 } else { 0xAE };
    put_list(env, cx, 0x5B, msg, &[id, a, LIST_END]);
}

/// C: `FUN_overlay_d_93__021c60ec`: `d5fe4(mon, v)`, command 0x19 (mon id, `v`).
pub fn cmd_19<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, v: u32) {
    env.call(0x021D_5FE4, &mut [Arg::V(mon), Arg::V(v)]);
    let id = mon_id(env, mon);
    put(env, cx, 0x19, &[id, v]);
}

/// C: `FUN_overlay_d_93__021c6110`: `d6008(mon, v)`, command 0x1A (mon id, `v`).
pub fn cmd_1a<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, v: u32) {
    env.call(0x021D_6008, &mut [Arg::V(mon), Arg::V(v)]);
    let id = mon_id(env, mon);
    put(env, cx, 0x1A, &[id, v]);
}

/// C: `FUN_overlay_d_93__021c6134`: `d5fc0(mon, v)`, command 0x1B (mon id, `v`).
pub fn cmd_1b<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, v: u32) {
    env.call(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(v)]);
    let id = mon_id(env, mon);
    put(env, cx, 0x1B, &[id, v]);
}

/// C: `FUN_overlay_d_93__021c604c`: unless `b911c(main, 4)`, `d5f90(mon, v & 0xffff)` and command 0x01
/// (mon id, `v & 0xffff`).
pub fn cmd_01<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, mon: u32, v: u32) {
    if env.call(0x021B_911C, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(4)]) == 0 {
        let v = v & 0xFFFF;
        env.call(0x021D_5F90, &mut [Arg::V(mon), Arg::V(v)]);
        let id = mon_id(env, mon);
        put(env, cx, 0x01, &[id, v]);
    }
}

/// C: `FUN_overlay_d_93__021c5b74`: `amount > 0`: `d5fa4(mon, amount & 0xffff)` and command 2 (id,
/// `amount & 0xffff`); `amount < 0`: `604c(mon, -amount)`; then, for `flag != 0` when `5bd4(work + 0x1AB8,
/// id) != 6`, command 0x41 (id).
pub fn hp_change<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    amount: i32,
    flag: u32,
) {
    let id = mon_id(env, mon);
    if amount > 0 {
        let a = amount as u32 & 0xFFFF;
        env.call(0x021D_5FA4, &mut [Arg::V(mon), Arg::V(a)]);
        put(env, cx, 2, &[id, a]);
    } else if amount < 0 {
        cmd_01(env, w, cx, mon, (amount as u32).wrapping_neg());
    }
    if flag != 0 && env.call(0x021C_5BD4, &mut [Arg::P, Arg::V(id)]) != 6 {
        put(env, cx, 0x41, &[id]);
    }
}

/// C: `FUN_overlay_d_93__021c5540`: command 0x36 (id), [`hp_change`] with `-amount` and flag 1, then
/// message 0x50 with the mon id.
pub fn damage_commands<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    amount: i32,
) {
    let id = mon_id(env, mon);
    put(env, cx, 0x36, &[id]);
    hp_change(env, w, cx, mon, amount.wrapping_neg(), 1);
    put_list(env, cx, 0x5A, 0x50, &[id, LIST_END]);
}

/// C: `FUN_overlay_d_93__021c5be4`: `d59b4(mon, 0xd)` (result unused), `d5fb8(mon)`, commands 3 (id) and
/// 0x43 (id, `v`).
pub fn cmd_03_43<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, v: u32) {
    env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)]);
    let id = mon_id(env, mon);
    env.call(0x021D_5FB8, &mut [Arg::V(mon)]);
    put(env, cx, 3, &[id]);
    put(env, cx, 0x43, &[id, v]);
}

/// C: `FUN_overlay_d_93__021c5c1c`: `d6034(mon, n, extra)`, command 0xF (id, `n & 0xff`, `extra`) and, for
/// `n < 6` (signed), command 0x35 (id, `n`, `extra`) where n 5 becomes 6 if `e8a78(extra)`.
pub fn cmd_0f<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, n: u32, extra: u32) {
    let id = mon_id(env, mon);
    env.call(0x021D_6034, &mut [Arg::V(mon), Arg::V(n), Arg::V(extra)]);
    put(env, cx, 0xF, &[id, n & 0xFF, extra]);
    if (n as i32) < 6 {
        let mut n = n;
        if n == 5 && env.call(0x021E_8A78, &mut [Arg::V(extra)]) != 0 {
            n = 6;
        }
        put(env, cx, 0x35, &[id, n, extra]);
    }
}

/// C: `FUN_overlay_d_93__021c6080`: `flag != 0`: `d56a0(mon, a, b)` and command 8 (id, a, b); otherwise
/// `d5674(mon, a, b)` and command 7.
pub fn cmd_07_08<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, a: u32, b: u32, flag: u32) {
    let id = mon_id(env, mon);
    if flag != 0 {
        env.call(0x021D_56A0, &mut [Arg::V(mon), Arg::V(a), Arg::V(b)]);
        put(env, cx, 8, &[id, a, b]);
    } else {
        env.call(0x021D_5674, &mut [Arg::V(mon), Arg::V(a), Arg::V(b)]);
        put(env, cx, 7, &[id, a, b]);
    }
}

/// C: `FUN_overlay_d_93__021c6158`: for each id of the byte list `list` (`[count, ids...]`): `b9aec(pokecon, id)`,
/// command 0x2A (id) and `b9d10(main, mon, id)`.
pub fn list_mons<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, list: &[u8]) {
    let mut i = 0usize;
    if list[0] != 0 {
        loop {
            let id = list[i + 1] as u32;
            let mon = env.call(
                0x021B_9AEC,
                &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(id)],
            );
            put(env, cx, 0x2A, &[id]);
            env.call(
                0x021B_9D10,
                &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(mon), Arg::V(id)],
            );
            i += 1;
            if i >= list[0] as usize {
                break;
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021c5418`: when `d5bdc(mon, status)` says so: [`cmd_1a`] (mon, status);
/// `62c4(mon, 0x1a)` -> `5620(mon, 0x1a, null)`; command 0x31 (id, 0); `bd1d0(id)`. Returns 1 when it ran.
pub fn apply_status<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, status: u32) -> u32 {
    if env.call(0x021D_5BDC, &mut [Arg::V(mon), Arg::V(status)]) == 0 {
        return 0;
    }
    let id = mon_id(env, mon);
    cmd_1a(env, cx, mon, status);
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x1A)]) != 0 {
        set_status_value(env, cx, mon, 0x1A, None);
    }
    put(env, cx, 0x31, &[id, 0]);
    env.call(0x021B_D1D0, &mut [Arg::P, Arg::V(id)]);
    1
}

/// C: `sub_021c4a64`: record bracket 0x261B around the 0x2649 round trip ([`message_round_trip`]
/// with `mon`, `sel`, `b`); when it answers with a value and `20bc(mon, value)` agrees, the selectors 4 / 5 / 9 /
/// 10 first put the slot message 0x258 / 0x257 / 0x274 / 0x273, then the hem record is reset (`8810`),
/// filled (`4b2c`) and used (`20d0`).
pub fn bracketed_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    sel: u32,
    b: u32,
) {
    let old = bracket_open(w);
    let r = message_round_trip(env, mon, sel, b);
    if r != 0 && env.call(0x021C_20BC, &mut [Arg::P, Arg::V(mon), Arg::V(r)]) != 0 {
        let msg = match sel {
            4 => Some(0x258),
            5 => Some(0x257),
            9 => Some(0x274),
            10 => Some(0x273),
            _ => None,
        };
        if let Some(m) = msg {
            put_slot_message(env, cx, mon, m);
        }
        env.call(0x021C_8810, &mut [Arg::P]);
        env.call(0x021C_4B2C, &mut [Arg::P, Arg::V(mon), Arg::V(sel)]);
        env.call(0x021C_20D0, &mut [Arg::P, Arg::V(mon), Arg::V(r), Arg::P]);
    }
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c5a8c`: stat change messages for `mon` / stat `stat`: `delta > 0` raises it
/// (`d5dc0(mon, stat, delta & 0xff)`; commands 9 and 0x37 with the stat and the applied amount) and, if `flag != 0`,
/// message 0x1B (`msg == 0`) or 0x3AA (with `msg`); otherwise lowers it (`d5e38(mon, stat, -delta & 0xff)`; commands 10
/// and 0x38, message 0x5B / 0x5A).
pub fn stat_change<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    mon: u32,
    stat: u32,
    delta: i32,
    msg: u32,
    flag: u32,
) {
    let id = mon_id(env, mon);
    if delta > 0 {
        let done = env.call(
            0x021D_5DC0,
            &mut [Arg::V(mon), Arg::V(stat), Arg::V(delta as u32 & 0xFF)],
        );
        put(env, cx, 9, &[id, stat & 0xFF, done & 0xFF]);
        put(env, cx, 0x37, &[id, stat & 0xFF, done & 0xFF]);
        if flag != 0 {
            if msg & 0xFFFF == 0 {
                put_list(env, cx, 0x5B, 0x1B, &[id, stat, done, LIST_END]);
            } else {
                put_list(
                    env,
                    cx,
                    0x5B,
                    0x3AA,
                    &[id, msg & 0xFFFF, stat, done, LIST_END],
                );
            }
        }
    } else {
        let done = env.call(
            0x021D_5E38,
            &mut [
                Arg::V(mon),
                Arg::V(stat),
                Arg::V((delta as u32).wrapping_neg() & 0xFF),
            ],
        );
        put(env, cx, 10, &[id, stat & 0xFF, done & 0xFF]);
        put(env, cx, 0x38, &[id, stat & 0xFF, done & 0xFF]);
        put_list(env, cx, 0x5B, 0x5A, &[id, stat, done, LIST_END]);
    }
}

/// The `i`-th little-endian word of `b`.
fn word(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap())
}

/// C: `FUN_overlay_d_93__021c59c4`: for each of the `count` mons whose flag byte `flags[i]` is set: message 0x180 with
/// the mon id (`extra != 0`) or the plain message 0x51.
pub fn flagged_messages<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    count: u32,
    mons: &[u8],
    flags: &[u8],
    extra: u32,
) {
    let mut i = 0usize;
    if count != 0 {
        loop {
            if flags[i] != 0 {
                if extra == 0 {
                    put_list(env, cx, 0x5A, 0x51, &[LIST_END]);
                } else {
                    let id = mon_id(env, word(mons, i));
                    put_list(env, cx, 0x5B, 0x180, &[id, LIST_END]);
                }
            }
            i += 1;
            if i as u32 >= count {
                break;
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021c5840`: counts the `count` signed words `vals` above 3 and below 3; without `flag` puts
/// message 0x4E (any above) or 0x4F (any below); with it puts the per-class 0x5B messages (6 / 9 / 0xC for 1 / 2 / 3
/// mons above, 0xF / 0x12 / 0x15 below) naming the mon ids of `mons` (up to three).
pub fn count_messages<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    count: u32,
    vals: &[u8],
    mons: &[u8],
    flag: u32,
) {
    let mut above = 0u8;
    let mut below = 0u8;
    for i in 0..count as usize {
        let v = word(vals, i) as i32;
        if v > 3 {
            above = above.wrapping_add(1);
        }
        if v < 3 {
            below = below.wrapping_add(1);
        }
    }
    if flag == 0 {
        if above != 0 {
            put_list(env, cx, 0x5A, 0x4E, &[LIST_END]);
        } else if below != 0 {
            put_list(env, cx, 0x5A, 0x4F, &[LIST_END]);
        }
        return;
    }
    // the original keeps the ids in a 4-byte stack array (the bytes past it are scratch)
    let mut ids = [0u8; 8];
    if above != 0 {
        let mut n = 0usize;
        for i in 0..count as usize {
            if word(vals, i) as i32 > 3 {
                ids[n] = mon_id(env, word(mons, i)) as u8;
                n = (n + 1) & 0xFF;
            }
        }
        let id = |k: usize| ids[k] as u32;
        match above {
            1 => put_list(env, cx, 0x5B, 6, &[id(0), LIST_END]),
            2 => put_list(env, cx, 0x5B, 9, &[id(0), id(1), LIST_END]),
            3 => put_list(env, cx, 0x5B, 0xC, &[id(0), id(1), id(2), LIST_END]),
            _ => {}
        }
    }
    if below == 0 {
        return;
    }
    let mut n = 0usize;
    for i in 0..count as usize {
        if (word(vals, i) as i32) < 3 {
            ids[n] = mon_id(env, word(mons, i)) as u8;
            n = (n + 1) & 0xFF;
        }
    }
    let id = |k: usize| ids[k] as u32;
    match below {
        1 => put_list(env, cx, 0x5B, 0xF, &[id(0), LIST_END]),
        2 => put_list(env, cx, 0x5B, 0x12, &[id(0), id(1), LIST_END]),
        3 => put_list(env, cx, 0x5B, 0x15, &[id(0), id(1), id(2), LIST_END]),
        _ => {}
    }
}

/// C: `FUN_overlay_d_93__021c5798`: for each of `count` entries: `604c(mons[i], hp[i])`, counting the signed words `vals` above and
/// below 3; then command 0x33 (count, verdict 2 / 3 / 1, `first_hp`) and, per entry, the byte queue push of the mon id.
#[allow(clippy::too_many_arguments)]
pub fn entry_commands<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    first_hp: &[u8],
    count: u32,
    vals: &[u8],
    mons: &[u8],
    hp: &[u8],
) {
    let mut above = 0u8;
    let mut below = 0u8;
    for i in 0..count as usize {
        let h = u16::from_le_bytes([hp[2 * i], hp[2 * i + 1]]) as u32;
        cmd_01(env, w, cx, word(mons, i), h);
        let v = word(vals, i) as i32;
        if v > 3 {
            above = above.wrapping_add(1);
        }
        if v < 3 {
            below = below.wrapping_add(1);
        }
    }
    let verdict = if above != 0 {
        2
    } else if below != 0 {
        3
    } else {
        1
    };
    let first = u16::from_le_bytes([first_hp[0], first_hp[1]]) as u32;
    put(env, cx, 0x33, &[count & 0xFF, verdict, first]);
    for i in 0..count as usize {
        let id = mon_id(env, word(mons, i));
        cmd_queue::push_u8(env, cx.q, id as u8);
    }
}

/// C: `FUN_overlay_d_93__021c5c74`: message by mode `mode` (1: 0x5B messages for the sub-id `sub` -- 1 -> 0x11A, 2 -> 0x13B, 3 ->
/// 0x129, 4 -> 0x10B, 5 -> 0xF9, 6 -> 0x162, 0xF -> 0x18 -- or 0x5A / 0x47 for the others, nothing at all for 0x14; 2: 0x5B / 0xD2;
/// else 0x5A / 0x47) and `work[0x78A] |= 0x10` (except for sub-id 0x14).
pub fn mode_message<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    mode: u32,
    sub: u32,
) {
    let id = mon_id(env, mon);
    if mode == 1 {
        let msg = match sub {
            0x14 => return,
            1 => Some(0x11A),
            2 => Some(0x13B),
            3 => Some(0x129),
            4 => Some(0x10B),
            5 => Some(0xF9),
            6 => Some(0x162),
            0xF => Some(0x18),
            _ => None,
        };
        match msg {
            Some(m) => put_list(env, cx, 0x5B, m, &[id, LIST_END]),
            None => put_list(env, cx, 0x5A, 0x47, &[LIST_END]),
        }
    } else if mode == 2 {
        put_list(env, cx, 0x5B, 0xD2, &[id, LIST_END]);
    } else {
        put_list(env, cx, 0x5A, 0x47, &[LIST_END]);
    }
    let f = w.u8_at(OFS_FLAGS_78A);
    w.set_u8_at(OFS_FLAGS_78A, f | 0x10);
}

/// The first `n` (0 and above 4 give none) words of `b`, followed by the list terminator.
fn words_then_end(b: &[u8], n: u32, mut head: Vec<u32>) -> Vec<u32> {
    if (1..=4).contains(&n) {
        head.extend((0..n as usize).map(|i| word(b, i)));
    }
    head.push(LIST_END);
    head
}

/// C: `FUN_overlay_d_93__021c5dac`: message 0x5A `msg` followed by the first `n` (1..=4) words of `words`.
pub fn message_5a_words<E: FlowEnv>(env: &mut E, cx: &mut Cx, msg: u32, n: u32, words: &[u8]) {
    put_list(env, cx, 0x5A, msg, &words_then_end(words, n, Vec::new()));
}

/// C: `FUN_overlay_d_93__021c5e48`: message 0x5C (`a`, `b`) followed by the first `n` words.
pub fn message_5c_words<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    a: u32,
    b: u32,
    n: u32,
    words: &[u8],
) {
    put_list(env, cx, 0x5C, a, &words_then_end(words, n, vec![b]));
}

/// C: `FUN_overlay_d_93__021c5efc`: message 0x5B `msg` followed by the first `n` words.
pub fn message_5b_words<E: FlowEnv>(env: &mut E, cx: &mut Cx, msg: u32, n: u32, words: &[u8]) {
    put_list(env, cx, 0x5B, msg, &words_then_end(words, n, Vec::new()));
}

/// C: `FUN_overlay_d_93__021c5f98`: message 0x5D (`a`, `b`) followed by the first `n` words.
pub fn message_5d_words<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    a: u32,
    b: u32,
    n: u32,
    words: &[u8],
) {
    put_list(env, cx, 0x5D, a, &words_then_end(words, n, vec![b]));
}

/// C: `FUN_overlay_d_93__021c5620`: with `v != 0`: `*out = 6314(mon, v)` when `out` is given; `v < 6` (signed):
/// `d61f0`, command 0x10 and, unless `5698(work + 0x1AB8, id) == 6`, command 0x35 (id, 0); otherwise `d6238(mon, v)` and
/// command 0x11 (id, `v & 0xffff`).
pub fn set_status_value<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    mon: u32,
    v: u32,
    out: Option<&mut [u8]>,
) {
    if v == 0 {
        return;
    }
    let id = mon_id(env, mon);
    if let Some(o) = out {
        let r = env.call(0x021D_6314, &mut [Arg::V(mon), Arg::V(v)]);
        o[..4].copy_from_slice(&r.to_le_bytes());
    }
    if (v as i32) < 6 {
        env.call(0x021D_61F0, &mut [Arg::V(mon)]);
        put(env, cx, 0x10, &[id]);
        if env.call(0x021C_5698, &mut [Arg::P, Arg::V(id)]) != 6 {
            put(env, cx, 0x35, &[id, 0]);
        }
    } else {
        env.call(0x021D_6238, &mut [Arg::V(mon), Arg::V(v)]);
        put(env, cx, 0x11, &[id, v & 0xFFFF]);
    }
}

// ---- message round trips (first argument = the server object) -------------------------------------------

/// C: `FUN_overlay_d_93__021c62d8`: message 0x2C3F round trip (slot 2 = the id of `mon`, 0xE = `p`, 0x12 = 0 (word),
/// 0xD = 6 (word), 0x41 = 0 (byte), sent with 0x18): stores the replies of slots 0x12 (u16) and 0xD (u8) in `out`
/// (`+0`, `+2`); true when slot 0x41 came back 0.
pub fn round_trip_2c3f<E: FlowEnv>(env: &mut E, mon: u32, p: u32, out: &mut [u8]) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x2C3F)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0xE), Arg::V(p)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x12), Arg::V(0)]);
    env.call(0x021D_75A8, &mut [Arg::V(0xD), Arg::V(6)]);
    env.call(0x021D_7630, &mut [Arg::V(0x41), Arg::V(0)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x18)]);
    let a = env.call(0x021D_775C, &mut [Arg::V(0x12)]);
    out[..2].copy_from_slice(&(a as u16).to_le_bytes());
    let b = env.call(0x021D_775C, &mut [Arg::V(0xD)]);
    out[2] = b as u8;
    let c = env.call(0x021D_775C, &mut [Arg::V(0x41)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2C3F + 0xD)]);
    (c == 0) as u32
}

/// C: `FUN_overlay_d_93__021c6350`: message 0x2C5D round trip (slot 2 = id, 0x11 = 1 (byte), sent with 0xF): the low
/// byte of slot 0x11.
pub fn round_trip_2c5d<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x2C5D)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_7630, &mut [Arg::V(0x11), Arg::V(1)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0xF)]);
    let r = env.call(0x021D_775C, &mut [Arg::V(0x11)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2C5D + 5)]);
    r & 0xFF
}

/// C: `FUN_overlay_d_93__021c6394`: message 0x2C72 (slot 2 = id, 0x12 = `p`, sent with 0x15).
pub fn message_2c72<E: FlowEnv>(env: &mut E, mon: u32, p: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(0x2C72)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75A8, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x12), Arg::V(p)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x15)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2C72 + 4)]);
}

/// C: `FUN_overlay_d_93__021c6564`: message 0x2CF4 (slot 2 = id, 0x12 = `p`, 0x22 = `q`, sent with 0x21).
pub fn message_2cf4<E: FlowEnv>(env: &mut E, mon: u32, p: u32, q: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(0x2CF4)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x12), Arg::V(p)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x22), Arg::V(q)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x21)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2CF4 + 5)]);
}

/// C: `FUN_overlay_d_93__021c6458`: resets the hem record (`8810`), then message 0x2CB0 round trip (slot 2 = id, 0x12 = `p`,
/// 0x3F = the address of the hem record `hem`, 0x51 = 0 (byte), sent with 0x19): the reply of slot 0x51.
pub fn round_trip_2cb0<E: FlowEnv>(env: &mut E, mon: u32, p: u32, hem: u32) -> u32 {
    env.call(0x021C_8810, &mut [Arg::P]);
    env.call(0x021D_74F8, &mut [Arg::V(0x2CB0)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x12), Arg::V(p)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x3F), Arg::V(hem)]);
    env.call(0x021D_7630, &mut [Arg::V(0x51), Arg::V(0)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x19)]);
    let r = env.call(0x021D_775C, &mut [Arg::V(0x51)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2CB0 + 7)]);
    r
}

/// C: `FUN_overlay_d_93__021c64b4`: message 0x2CC8 round trip for species `species` and mon `mon` (slot 2 = id, 0x16 =
/// `201c1c8(species)` (byte), 0x1C = `d57fc(mon)`, 0x1A = `201c1e8(species)`, 0x1B = `201bd28(species, 0x1B)`, 0x4B = 0 (byte), sent
/// with 0x28): fills the 0x14-byte record `out` (`+0` / `+2` = species, `+4` = slot 0x1C, `+6` = slot 0x16 (0x11 if slot 0x4B), `+8` =
/// slot 0x1A, `+0xC` = slot 0x1B, `+0x10` = 0).
pub fn round_trip_2cc8<E: FlowEnv>(env: &mut E, species: u32, mon: u32, out: &mut [u8]) {
    env.call(0x021D_74F8, &mut [Arg::V(0x2CC8)]);
    let id = mon_id(env, mon);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    let v = env.call(0x0201_C1C8, &mut [Arg::V(species)]);
    env.call(0x021D_7630, &mut [Arg::V(0x16), Arg::V(v)]);
    let v = env.call(0x021D_57FC, &mut [Arg::V(mon)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x1C), Arg::V(v)]);
    let v = env.call(0x0201_C1E8, &mut [Arg::V(species)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x1A), Arg::V(v)]);
    let v = env.call(0x0201_BD28, &mut [Arg::V(species), Arg::V(0x1B)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x1B), Arg::V(v)]);
    env.call(0x021D_7630, &mut [Arg::V(0x4B), Arg::V(0)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x28)]);
    out[0..2].copy_from_slice(&(species as u16).to_le_bytes());
    out[2..4].copy_from_slice(&(species as u16).to_le_bytes());
    let r = env.call(0x021D_775C, &mut [Arg::V(0x16)]);
    out[6] = r as u8;
    let r = env.call(0x021D_775C, &mut [Arg::V(0x1C)]);
    out[4..6].copy_from_slice(&(r as u16).to_le_bytes());
    let r = env.call(0x021D_775C, &mut [Arg::V(0x1A)]);
    out[8..12].copy_from_slice(&r.to_le_bytes());
    let r = env.call(0x021D_775C, &mut [Arg::V(0x1B)]);
    out[12..16].copy_from_slice(&r.to_le_bytes());
    out[16..20].copy_from_slice(&0u32.to_le_bytes());
    if env.call(0x021D_775C, &mut [Arg::V(0x4B)]) != 0 {
        out[6] = 0x11;
    }
    env.call(0x021D_7540, &mut [Arg::V(0x2CC8 + 0x1D)]);
}

/// C: `FUN_overlay_d_93__021c63d0`: base damage `CalcBaseDamage(0x28, d59b4(mon, 8), d59b4(mon, 0xF), d59b4(mon, 9))` scaled by
/// `(100 - 78ac(0x10)) / 100` (at least 1, 16 bits); `1678(mon, mon, 0, &dmg)` may rewrite it; [`damage_commands`] with it,
/// `16f8(mon, r)` when `1678` returned non-zero. Returns the damage.
pub fn base_damage_effect<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, mon: u32) -> u32 {
    let atk = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(8)]);
    let def = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(9)]);
    let level = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xF)]);
    let base = env.call(
        0x021D_7A44,
        &mut [Arg::V(0x28), Arg::V(atk), Arg::V(level), Arg::V(def)],
    );
    let scale = env.call(0x021D_78AC, &mut [Arg::V(0x10)]);
    let mut dmg = crate::calc::u32_div(
        (base as u16 as u32).wrapping_mul(100u32.wrapping_sub(scale)),
        100,
    ) as u16;
    if dmg == 0 {
        dmg = 1;
    }
    let mut buf = dmg.to_le_bytes();
    let r = env.call(
        0x021C_1678,
        &mut [
            Arg::P,
            Arg::V(mon),
            Arg::V(mon),
            Arg::V(0),
            Arg::Out(&mut buf),
        ],
    );
    dmg = u16::from_le_bytes(buf);
    damage_commands(env, w, cx, mon, dmg as i32);
    if r != 0 {
        env.call(0x021C_16F8, &mut [Arg::P, Arg::V(mon), Arg::V(r)]);
    }
    dmg as u32
}

/// C: `FUN_overlay_d_93__021c6190`: for every one of the `work[0x782]` entries (16 bytes at `work + 0x7E0`: mon, packed
/// record, flag bytes at `+0xD` / `+0xE`) other than `except` with `+0xD == 0` and species 0xE4: if `e02dc(mon, 0xE4, d59b4(mon, 0xC))`
/// says so mark `+0xE`; then `work[0x788] = 0`, message 0x2C18 (slot 6 = id of `except`, sent with 0x53), and for each
/// marked entry of species 0xE4 `e03d8(mon)`, clearing the mark. Returns `work[0x788]`.
pub fn mark_species_entries<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, except: u32) -> u32 {
    fn species(packed: u32) -> u32 {
        if packed & 0xF == 1 {
            (packed & 0x7F_FFFF) >> 7
        } else {
            0
        }
    }
    if w.u8_at(OFS_ENTRY_COUNT) != 0 {
        let mut i = 0usize;
        loop {
            let base = OFS_ENTRIES + i * 16;
            let mon = w.u32_at(base);
            if mon != except && w.u8_at(base + 0xD) == 0 {
                let sp = species(w.u32_at(base + 4));
                if sp == 0xE4 {
                    let v = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xC)]);
                    if env.call(0x021E_02DC, &mut [Arg::V(mon), Arg::V(sp), Arg::V(v)]) != 0 {
                        w.set_u8_at(base + 0xE, 1);
                    }
                }
            }
            i += 1;
            if i >= w.u8_at(OFS_ENTRY_COUNT) as usize {
                break;
            }
        }
    }
    w.set_u8_at(OFS_RESULT_788, 0);
    env.call(0x021D_74F8, &mut [Arg::V(0x2C18)]);
    let id = mon_id(env, except);
    env.call(0x021D_75EC, &mut [Arg::V(6), Arg::V(id)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x53)]);
    env.call(0x021D_7540, &mut [Arg::V(0x2C18 + 3)]);
    if w.u8_at(OFS_ENTRY_COUNT) != 0 {
        let mut i = 0usize;
        loop {
            let base = OFS_ENTRIES + i * 16;
            if w.u8_at(base + 0xE) != 0 {
                if species(w.u32_at(base + 4)) == 0xE4 {
                    env.call(0x021E_03D8, &mut [Arg::V(w.u32_at(base))]);
                }
                w.set_u8_at(base + 0xE, 0);
            }
            i += 1;
            if i >= w.u8_at(OFS_ENTRY_COUNT) as usize {
                break;
            }
        }
    }
    w.u8_at(OFS_RESULT_788) as u32
}

#[cfg(test)]
mod tests;
