//! HANDEX execution (0x021C8918-0x021CA8CC; B10b): the executor `021c8918` (switch over the
//! record id 0x00-0x3B), its 59 procs, the message helper `021c99a0` and `021ca7f4`. Specs and
//! the record layout: `tools/rom-diff/svflow_specs/handex.spec` (header + the case table on the
//! `021c8918` block).
//!
//! A proc gets the work, the record bytes `rec` (word 0: id bits 0-7, pokeID bits 8-12, flags
//! 23-26; proc fields from +4) and, when it needs them, the command queue `q` behind
//! `work + 0xC` plus the put_cmd scratch words at 0x021F3B44 (`scratch`), or the record `d`
//! behind `work + 0x4A8`. Callees outside the area (core1..core6, events, api, hem, BattleMon,
//! main module, `bx pc` veneers into overlay 95) go through [`FlowEnv`]; cmd_queue functions and
//! the message helper [`message`] are called directly. Proc names are the record id (`proc_XX`).

use super::cmd_queue::{
    self, advance_fields, put_cmd, put_cmd_at, put_cmd_list, put_cmd_placeholder, set_mode_flag30,
    set_word, word_bits28_29, word_flag30, word_low10,
};
use super::env::{Arg, FlowEnv, Lend, Lent};
use super::hem::HANDEX_MGR;
use super::SvFlowWork;

/// Terminator of a `put_cmd_list` argument list (literal at 0x021C8DE0).
const END: u32 = 0xFFFF_0000;

fn rd16(rec: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([rec[o], rec[o + 1]]) as u32
}

fn rd32(rec: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(rec[o..o + 4].try_into().unwrap())
}

/// pokeID field of a HANDEX record header (bits 8-12 of word 0).
fn rec_poke_id(rec: &[u8]) -> u32 {
    (rd32(rec, 0) >> 8) & 0x1F
}

/// Header bit 23: bracket the effect with core5 `5a1c` / `5a34` on a mon.
fn bit23(rec: &[u8]) -> bool {
    (rd32(rec, 0) >> 23) & 1 != 0
}

/// The str record (hem layout, 0x24 bytes) at `ofs` of the HANDEX record.
fn str_at(rec: &[u8], ofs: usize) -> &[u8] {
    &rec[ofs..ofs + 0x24]
}

/// `021b9934(work[8], id)`: the BattleMon of a pokeID.
fn mon_of<E: FlowEnv>(env: &mut E, w: &SvFlowWork, id: u32) -> u32 {
    env.callwr(0x021B_9934, &mut [Arg::V(w.u32_at(8)), Arg::V(id)], w)
}

/// `021d5b68(mon)`: nonzero when the mon has fainted.
fn fainted<E: FlowEnv>(env: &mut E, mon: u32) -> bool {
    env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0
}

/// A `bx pc` veneer into overlay 95 taking `(work + 0x1AB8, id)`.
///
/// `helper`: the host pointer of `work + 0x1AB8` (B16: the veneer's first argument).
fn veneer_ab8<E: FlowEnv>(env: &mut E, addr: u32, id: u32, helper: usize) -> bool {
    env.call(addr, &mut [Arg::PHost(helper), Arg::V(id)]) != 0
}

/// core5 `021c5a1c(work, mon)` (opens the effect bracket of header bit 23).
fn bracket_open<E: FlowEnv>(env: &mut E, mon: u32) {
    env.call(0x021C_5A1C, &mut [Arg::P, Arg::V(mon)]);
}

/// core5 `021c5a34(work, mon)` (closes the effect bracket of header bit 23).
fn bracket_close<E: FlowEnv>(env: &mut E, mon: u32) {
    env.call(0x021C_5A34, &mut [Arg::P, Arg::V(mon)]);
}

/// The mon of the record's pokeID, or 0 when the pokeID is 0x1F (none).
fn mon_or_none<E: FlowEnv>(env: &mut E, w: &SvFlowWork, id: u32) -> u32 {
    if id == 0x1F {
        0
    } else {
        mon_of(env, w, id)
    }
}

/// C: `FUN_overlay_d_93__021c8918`: executes one HANDEX record. `v` = bits 0-9 and `st` = bits
/// 28-29 (or 1 when bit 30 is clear) of the manager word at `work + 0x1D74`. Nothing happens
/// (not even the final `set_mode_flag30`) when header bit 24 is set and `st` is 0, or when header
/// bit 25 is set and the pokeID's mon (`021b9940`) has fainted. Otherwise ids 0x00-0x3B (but
/// 0x0D) run their proc, whose result becomes `st`, and `set_mode_flag30(mgr, st)` records it.
/// `ra` is the record's address (handed to the callees that take a pointer as a value).
#[allow(clippy::too_many_arguments)]
pub fn execute<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    ra: u32,
    q: &mut [u8],
    scratch: &mut [u8],
    d: &mut [u8],
) {
    let v = word_low10(&w.mem[HANDEX_MGR..]);
    let mut st = if word_flag30(&w.mem[HANDEX_MGR..]) != 0 {
        word_bits28_29(&w.mem[HANDEX_MGR..]) & 0xFF
    } else {
        1
    };
    let h = rd32(rec, 0);
    if (h >> 24) & 1 != 0 && st == 0 {
        return;
    }
    if (h >> 25) & 1 != 0 {
        let mon = env.call_l(
            0x021B_9940,
            &mut [Arg::V(w.u32_at(8)), Arg::V(rec_poke_id(rec))],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        if fainted(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), mon) {
            return;
        }
    }
    let id = h & 0xFF;
    if id <= 0x3B {
        st = match id {
            0x00 => proc_00(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x01 => proc_01(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x02 => proc_02(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x03 => proc_03(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x04 => proc_04(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x05 => proc_05(env, w, rec, v, q),
            0x06 => proc_06(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x07 => proc_07(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x08 => proc_08(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x09 => proc_09(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x0A => proc_0a(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x0B => proc_0b(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec, v),
            0x0C => proc_0c(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec, ra),
            0x0D => st,
            0x0E => proc_0e(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec, v),
            0x0F => proc_0f(env, w, rec, q, scratch),
            0x10 => proc_10(env, w, rec, q, scratch),
            0x11 => proc_11(env, w, rec, q, scratch),
            0x12 => proc_12(env, w, rec, q, scratch),
            0x13 => proc_13(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x14 => proc_14(env, w, rec, q, scratch),
            0x15 => proc_15(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x16 => proc_16(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x17 => proc_17(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x18 => proc_18(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x19 => proc_19(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x1A => proc_1a(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x1B => proc_1b(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x1C => proc_1c(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x1D => proc_1d(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x1E => proc_1e(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x1F => proc_1f(env, w, rec, q, scratch),
            0x20 => proc_20(env, w, rec, q, scratch),
            0x21 => proc_21(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x22 => proc_22(env, w, rec, q, scratch),
            0x23 => proc_23(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x24 => proc_24(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x25 => proc_25(env, w, rec, q, scratch),
            0x26 => proc_26(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x27 => proc_27(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec, d),
            0x28 => proc_28(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x29 => proc_29(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x2A => proc_2a(env, w, rec, q, scratch),
            0x2B => proc_2b(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x2C => proc_2c(env, w, rec, q, scratch),
            0x2D => proc_2d(env, w, rec, q, scratch),
            0x2E => proc_2e(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x2F => proc_2f(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x30 => proc_30(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x31 => proc_31(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), rec),
            0x32 => proc_32(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x33 => proc_33(env, w, rec, q, scratch),
            0x34 => proc_34(env, w, rec, q, scratch),
            0x35 => proc_35(env, w, rec, q),
            0x36 => proc_36(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, rec),
            0x37 => proc_37(&mut Lend::new(env, w), rec, q, scratch),
            0x38 => proc_38(&mut Lend::new(env, w), q, scratch),
            0x39 => proc_39(env, w, rec, q, scratch),
            0x3A => proc_3a(rec, d),
            _ => proc_3b(d),
        };
    }
    set_mode_flag30(&mut w.mem[HANDEX_MGR..], st);
}

/// C: `FUN_overlay_d_93__021c9de8`: HANDEX proc 0x00: 0 when the pokeID's mon has fainted and
/// bit 1 of `rec[4]` is clear, or when bit 0 of `rec[4]` is set and `021d5b48(mon)` says so;
/// else whether core4 `021c2128(work, mon)` is nonzero.
pub fn proc_00<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec_poke_id(rec));
    let f = rd32(rec, 4);
    if fainted(&mut Lend::new_ref(env, w), mon) && f & 2 == 0 {
        return 0;
    }
    if f & 1 != 0 && env.callwr(0x021D_5B48, &mut [Arg::V(mon)], w) != 0 {
        return 0;
    }
    (env.callwr(0x021C_2128, &mut [Arg::P, Arg::V(mon)], w) != 0) as u32
}

/// C: `FUN_overlay_d_93__021c8c6c`: HANDEX proc 0x01: when the veneer `8cac(work + 0x1AB8,
/// pokeID)` agrees, core6 `021c60d4(work, mon)`; returns 1, else 0.
pub fn proc_01<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    if !veneer_ab8(
        &mut Lend::new_ref(env, w),
        0x021C_8CAC,
        rec_poke_id(rec),
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(env, w, rec_poke_id(rec));
    env.callwr(0x021C_60D4, &mut [Arg::P, Arg::V(mon)], w);
    1
}

/// C: `FUN_overlay_d_93__021c8cbc`: HANDEX proc 0x02: when the veneer `8cfc(work + 0x1AB8,
/// pokeID)` agrees, core5 `021c5a1c(work, mon)`; returns 1, else 0.
pub fn proc_02<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    if !veneer_ab8(
        &mut Lend::new_ref(env, w),
        0x021C_8CFC,
        rec_poke_id(rec),
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(env, w, rec_poke_id(rec));
    bracket_open(&mut Lend::new_ref(env, w), mon);
    1
}

/// C: `FUN_overlay_d_93__021c8d0c`: HANDEX proc 0x03: core5 `021c5a34(work, mon)` for the
/// record's pokeID; returns 1.
pub fn proc_03<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec_poke_id(rec));
    bracket_close(&mut Lend::new_ref(env, w), mon);
    1
}

/// C: `FUN_overlay_d_93__021c96d4`: HANDEX proc 0x04: the message at +4 (inside the bit-23
/// bracket on the pokeID's mon, unless the pokeID is 0x1F); returns 1.
pub fn proc_04<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let a = mon_or_none(env, w, rec_poke_id(rec));
    if bit23(rec) && a != 0 {
        bracket_open(&mut Lend::new_ref(env, w), a);
    }
    message(&mut Lend::new_ref(env, w), str_at(rec, 4));
    if bit23(rec) && a != 0 {
        bracket_close(&mut Lend::new_ref(env, w), a);
    }
    1
}

/// C: `FUN_overlay_d_93__021c8d2c`: HANDEX proc 0x05 (gets `v`): on target `t = mon(rec[6])`,
/// nothing (0) when core4 `3370(work, t)` says so; else (bit-23 bracket on the pokeID's mon)
/// `ok = rec[7] != 0 || !3394(work, t, 1)`, and when ok `33e8(work, t, u16 rec[4])` followed by
/// the message at +8 if it is set, or command list 0x5B (0x38C, rec[6], v) when `v` != 0.
/// Returns ok.
pub fn proc_05<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], v: u32, q: &mut [u8]) -> u32 {
    let a = mon_of(&mut Lend::hv(env, vec![&mut *q]), w, rec_poke_id(rec));
    let t = mon_of(&mut Lend::hv(env, vec![&mut *q]), w, rec[6] as u32);
    if env.call_l(
        0x021C_3370,
        &mut [Arg::P, Arg::V(t)],
        Lent::of(None, Some(w), vec![&mut *q]),
    ) != 0
    {
        return 0;
    }
    if bit23(rec) {
        bracket_open(&mut Lend::whr(env, w, vec![&mut *q]), a);
    }
    let ok = if rec[7] == 0 {
        env.call_l(
            0x021C_3394,
            &mut [Arg::P, Arg::V(t), Arg::V(1)],
            Lent::of(None, Some(w), vec![&mut *q]),
        ) == 0
    } else {
        true
    };
    if ok {
        env.call_l(
            0x021C_33E8,
            &mut [Arg::P, Arg::V(t), Arg::V(rd16(rec, 4))],
            Lent::of(None, Some(w), vec![&mut *q]),
        );
        if rec[0xA] != 0 {
            message(&mut Lend::whr(env, w, vec![&mut *q]), str_at(rec, 8));
        } else if v != 0 {
            put_cmd_list(
                &mut Lend::new_ref(env, w),
                q,
                0x5B,
                0x38C,
                &[rec[6] as u32, v, END],
            );
        }
    }
    if bit23(rec) {
        bracket_close(&mut Lend::whr(env, w, vec![&mut *q]), a);
    }
    ok as u32
}

/// C: `FUN_overlay_d_93__021c8de4`: HANDEX proc 0x06 (its `v` is not read): attacker
/// `a = mon(rec[7])` (0 for 0x1F); when the veneer `8e48(work + 0x1AB8, rec[6])` agrees, the
/// target `t = mon(rec[6])` is alive and core4 `1e14(work, t, a, u16 rec[4])` succeeds: the
/// message at +8 if it is set, returns 1. Else 0.
pub fn proc_06<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let a = mon_or_none(env, w, rec[7] as u32);
    if !veneer_ab8(
        &mut Lend::new_ref(env, w),
        0x021C_8E48,
        rec[6] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let t = mon_of(env, w, rec[6] as u32);
    if fainted(&mut Lend::new_ref(env, w), t) {
        return 0;
    }
    if env.callwr(
        0x021C_1E14,
        &mut [Arg::P, Arg::V(t), Arg::V(a), Arg::V(rd16(rec, 4))],
        w,
    ) == 0
    {
        return 0;
    }
    if rec[0xA] != 0 {
        message(&mut Lend::new_ref(env, w), str_at(rec, 8));
    }
    1
}

/// C: `FUN_overlay_d_93__021c8e58`: HANDEX proc 0x07: when the veneer `8f14(work + 0x1AB8,
/// rec[6])` agrees, `t = mon(rec[6])` is alive, (with `rec[7]` bit 0) `5c30(t)` is clear and
/// core4 `20bc(work, t, u16 rec[4])` succeeds: (bit-23 bracket on the pokeID's mon, 0 for 0x1F)
/// with `rec[7]` bit 1 core6 `70e4(work, u16 rec[8], rec[0xA], rec[0xB], 0, 0)`, then
/// `20d0(work, t, u16 rec[4], rec + 0xC)`; returns 1. Else 0.
pub fn proc_07<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    if !veneer_ab8(
        &mut Lend::new_ref(env, w),
        0x021C_8F14,
        rec[6] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let t = mon_of(env, w, rec[6] as u32);
    let a = mon_or_none(env, w, rec_poke_id(rec));
    if fainted(&mut Lend::new_ref(env, w), t) {
        return 0;
    }
    if rec[7] & 1 != 0 && env.callwr(0x021D_5C30, &mut [Arg::V(t)], w) != 0 {
        return 0;
    }
    if env.callwr(
        0x021C_20BC,
        &mut [Arg::P, Arg::V(t), Arg::V(rd16(rec, 4))],
        w,
    ) == 0
    {
        return 0;
    }
    if bit23(rec) {
        bracket_open(&mut Lend::new_ref(env, w), a);
    }
    if rec[7] & 2 != 0 {
        env.callwr(
            0x021C_70E4,
            &mut [
                Arg::P,
                Arg::V(rd16(rec, 8)),
                Arg::V(rec[0xA] as u32),
                Arg::V(rec[0xB] as u32),
                Arg::V(0),
                Arg::V(0),
            ],
            w,
        );
    }
    env.callwr(
        0x021C_20D0,
        &mut [Arg::P, Arg::V(t), Arg::V(rd16(rec, 4)), Arg::P],
        w,
    );
    if bit23(rec) {
        bracket_close(&mut Lend::new_ref(env, w), a);
    }
    1
}

/// C: `FUN_overlay_d_93__021c8f24`: HANDEX proc 0x08: for each of the `rec[4]` mon ids at +7:
/// when the veneer `8f94(work + 0x1AB8, id)` agrees and the mon is alive, core5 `5b74(work, mon,
/// u32 rec[0x10 + 4i], rec[5] == 0)` and (when `rec[6]` is 0) core4 `195c(work, mon, 1)`.
/// Returns 1 when any mon was handled.
pub fn proc_08<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mut r = 0;
    let mut i = 0usize;
    while i < rec[4] as usize {
        let id = rec[7 + i] as u32;
        if veneer_ab8(
            &mut Lend::new_ref(env, w),
            0x021C_8F94,
            id,
            w.mem[0x1AB8..].as_ptr() as usize,
        ) {
            let mon = mon_of(env, w, id);
            if !fainted(&mut Lend::new_ref(env, w), mon) {
                env.callwr(
                    0x021C_5B74,
                    &mut [
                        Arg::P,
                        Arg::V(mon),
                        Arg::V(rd32(rec, 0x10 + 4 * i)),
                        Arg::V((rec[5] == 0) as u32),
                    ],
                    w,
                );
                if rec[6] == 0 {
                    env.callwr(0x021C_195C, &mut [Arg::P, Arg::V(mon), Arg::V(1)], w);
                }
                r = 1;
            }
        }
        i += 1;
    }
    r
}

/// C: `FUN_overlay_d_93__021c8fa4`: HANDEX proc 0x09 (its `v` is not read; the pokeID's mon is
/// fetched and dropped): on `t = mon(rec[5])`, when `5b7c(t)` or `rec[7]` bit 1, with
/// `f = !(rec[7] bit 0)`: unless `55d8(t, rec[6], f)`, core6 `6080(work, t, rec[6], rec[4], f)`
/// and the message at +8; returns 1. Else 0.
pub fn proc_09<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    mon_of(env, w, rec_poke_id(rec));
    let t = mon_of(env, w, rec[5] as u32);
    if env.callwr(0x021D_5B7C, &mut [Arg::V(t)], w) == 0 && rec[7] & 2 == 0 {
        return 0;
    }
    let f = (rec[7] & 1 == 0) as u32;
    if env.callwr(
        0x021D_55D8,
        &mut [Arg::V(t), Arg::V(rec[6] as u32), Arg::V(f)],
        w,
    ) != 0
    {
        return 0;
    }
    env.callwr(
        0x021C_6080,
        &mut [
            Arg::P,
            Arg::V(t),
            Arg::V(rec[6] as u32),
            Arg::V(rec[4] as u32),
            Arg::V(f),
        ],
        w,
    );
    message(&mut Lend::new_ref(env, w), str_at(rec, 8));
    1
}

/// C: `FUN_overlay_d_93__021c9010`: HANDEX proc 0x0A (its `v` is not read): on `t = mon(rec[5])`,
/// alive or `rec[7]` bit 1: when core3 `0898(work, t, rec[6], rec[4])` succeeds, the message at
/// +8 and (when core6 `7194(work, t, rec[6])` says so) core4 `2128(work, t)`; returns 1. Else 0.
pub fn proc_0a<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let t = mon_of(env, w, rec[5] as u32);
    if fainted(&mut Lend::new_ref(env, w), t) && rec[7] & 2 == 0 {
        return 0;
    }
    if env.callwr(
        0x021C_0898,
        &mut [
            Arg::P,
            Arg::V(t),
            Arg::V(rec[6] as u32),
            Arg::V(rec[4] as u32),
        ],
        w,
    ) != 0
    {
        message(&mut Lend::new_ref(env, w), str_at(rec, 8));
        if env.callwr(
            0x021C_7194,
            &mut [Arg::P, Arg::V(t), Arg::V(rec[6] as u32)],
            w,
        ) != 0
        {
            env.callwr(0x021C_2128, &mut [Arg::P, Arg::V(t)], w);
        }
    }
    1
}

/// C: `FUN_overlay_d_93__021c9068`: HANDEX proc 0x0B (gets `v`): (bit-23 bracket on the
/// pokeID's mon) for each of the `rec[0x14]` mon ids at +8 whose mon passes `5b7c`: with a cursor
/// word = u32 `rec[4]`, every `k = 56a8(mon, &cursor)` (core5) until 0 gets core5 `5620(work,
/// mon, k, &x)` and then either (`rec[0x15]` = 0) the veneer `9168(k, x, mon, v, work + 0x1AE4)`,
/// which when it agrees is followed by the message at work + 0x1AE4 and hem `8810` on it, or the
/// message at +0x18; k = 0x13 adds core4 `195c(work, mon, 0)`. Returns 1 when any k was found.
/// (The cursor is handed to the stub as input only, as the spec models it.)
pub fn proc_0b<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], v: u32) -> u32 {
    let a = mon_of(env, w, rec_poke_id(rec));
    let mut r = 0;
    if bit23(rec) {
        bracket_open(&mut Lend::new_ref(env, w), a);
    }
    let mut i = 0usize;
    while i < rec[0x14] as usize {
        let t = mon_of(env, w, rec[8 + i] as u32);
        if env.callwr(0x021D_5B7C, &mut [Arg::V(t)], w) != 0 {
            let cursor = rec[4..8].to_vec();
            let mut k = env.callwr(0x021C_56A8, &mut [Arg::V(t), Arg::In(&cursor)], w);
            while k != 0 {
                r = 1;
                let mut x = [0u8; 4];
                env.callwr(
                    0x021C_5620,
                    &mut [Arg::P, Arg::V(t), Arg::V(k), Arg::Out(&mut x)],
                    w,
                );
                if rec[0x15] == 0 {
                    if env.callwr(
                        0x021C_9168,
                        &mut [
                            Arg::V(k),
                            Arg::V(u32::from_le_bytes(x)),
                            Arg::V(t),
                            Arg::V(v),
                            Arg::P,
                        ],
                        w,
                    ) != 0
                    {
                        message(env, &w.mem[0x1AE4..0x1AE4 + 0x24]);
                        env.callwr(0x021C_8810, &mut [Arg::P], w);
                    }
                } else {
                    message(&mut Lend::new_ref(env, w), str_at(rec, 0x18));
                }
                if k == 0x13 {
                    env.callwr(0x021C_195C, &mut [Arg::P, Arg::V(t), Arg::V(0)], w);
                }
                k = env.callwr(0x021C_56A8, &mut [Arg::V(t), Arg::In(&cursor)], w);
            }
        }
        i += 1;
    }
    if bit23(rec) {
        bracket_close(&mut Lend::new_ref(env, w), a);
    }
    r
}

/// C: `FUN_overlay_d_93__021c9178`: HANDEX proc 0x0C: attacker `a` = the pokeID's mon (0 for
/// 0x1F), `f = !hem 8828(rec + 0x14) && rec[0xD] == 0`; when the veneer `9240(work + 0x1AB8,
/// rec[0xF])` agrees, `t = mon(rec[0xF])` is alive and core4 `27a0(work, t, a, u32 rec[4],
/// u32 rec[8], rec[0x10], rec[0xC])` is 0: (bit-23 bracket on `a`) core4 `2910(work, t, a,
/// u32 rec[4], u32 rec[8], f, rec[0xE], rec + 0x14)`; returns 1. Else 0. `ra` = the record's
/// address (2910 gets `rec + 0x14` as a value).
pub fn proc_0c<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], ra: u32) -> u32 {
    let a = mon_or_none(env, w, rec_poke_id(rec));
    let f = (env.callwr(0x021C_8828, &mut [Arg::P], w) == 0 && rec[0xD] == 0) as u32;
    if !veneer_ab8(
        &mut Lend::new_ref(env, w),
        0x021C_9240,
        rec[0xF] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let t = mon_of(env, w, rec[0xF] as u32);
    if fainted(&mut Lend::new_ref(env, w), t) {
        return 0;
    }
    if env.callwr(
        0x021C_27A0,
        &mut [
            Arg::P,
            Arg::V(t),
            Arg::V(a),
            Arg::V(rd32(rec, 4)),
            Arg::V(rd32(rec, 8)),
            Arg::V(rec[0x10] as u32),
            Arg::V(rec[0xC] as u32),
        ],
        w,
    ) != 0
    {
        return 0;
    }
    if bit23(rec) {
        bracket_open(&mut Lend::new_ref(env, w), a);
    }
    env.callwr(
        0x021C_2910,
        &mut [
            Arg::P,
            Arg::V(t),
            Arg::V(a),
            Arg::V(rd32(rec, 4)),
            Arg::V(rd32(rec, 8)),
            Arg::V(f),
            Arg::V(rec[0xE] as u32),
            Arg::V(ra.wrapping_add(0x14)),
        ],
        w,
    );
    if bit23(rec) {
        bracket_close(&mut Lend::new_ref(env, w), a);
    }
    1
}

/// C: `FUN_overlay_d_93__021c9250`: HANDEX proc 0x0E (gets `v`): pass 1 over the `rec[0xF]`
/// mon ids at +0x10: the first one passing the veneer `9374(work + 0x1AB8, id)`, alive and
/// `5d14(mon, u32 rec[4], (s8) rec[0xC])` sets `any` (and opens the bit-23 bracket on the
/// pokeID's mon). Pass 2: every id passing the veneer `9384`, alive and core4 `3008(work, pokeID,
/// mon, u32 rec[4], (s8) rec[0xC], 0x1F, v, u32 rec[8], rec[0xE], rec[0xD] == 0)` gets the message
/// at +0x18 and makes the result 1. `any` closes the bracket.
pub fn proc_0e<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], v: u32) -> u32 {
    let a = mon_of(env, w, rec_poke_id(rec));
    let mut any = false;
    let mut r = 0;
    let sv = rec[0xC] as i8 as i32 as u32;
    let mut i = 0usize;
    while i < rec[0xF] as usize {
        let id = rec[0x10 + i] as u32;
        if veneer_ab8(
            &mut Lend::new_ref(env, w),
            0x021C_9374,
            id,
            w.mem[0x1AB8..].as_ptr() as usize,
        ) {
            let mon = mon_of(env, w, id);
            if !fainted(&mut Lend::new_ref(env, w), mon)
                && env.callwr(
                    0x021D_5D14,
                    &mut [Arg::V(mon), Arg::V(rd32(rec, 4)), Arg::V(sv)],
                    w,
                ) != 0
            {
                any = true;
                break;
            }
        }
        i += 1;
    }
    if any && bit23(rec) {
        bracket_open(&mut Lend::new_ref(env, w), a);
    }
    let mut i = 0usize;
    while i < rec[0xF] as usize {
        let id = rec[0x10 + i] as u32;
        if veneer_ab8(
            &mut Lend::new_ref(env, w),
            0x021C_9384,
            id,
            w.mem[0x1AB8..].as_ptr() as usize,
        ) {
            let mon = mon_of(env, w, id);
            if !fainted(&mut Lend::new_ref(env, w), mon) {
                let f = (rec[0xD] == 0) as u32;
                if env.callwr(
                    0x021C_3008,
                    &mut [
                        Arg::P,
                        Arg::V(rec_poke_id(rec)),
                        Arg::V(mon),
                        Arg::V(rd32(rec, 4)),
                        Arg::V(sv),
                        Arg::V(0x1F),
                        Arg::V(v),
                        Arg::V(rd32(rec, 8)),
                        Arg::V(rec[0xE] as u32),
                        Arg::V(f),
                    ],
                    w,
                ) != 0
                {
                    message(&mut Lend::new_ref(env, w), str_at(rec, 0x18));
                    r = 1;
                }
            }
        }
        i += 1;
    }
    if any && bit23(rec) {
        bracket_close(&mut Lend::new_ref(env, w), a);
    }
    r
}

/// C: `FUN_overlay_d_93__021c9394`: HANDEX proc 0x0F: when the veneer `9438(work + 0x1AB8,
/// rec[4])` agrees and `mon(rec[4])` is alive: `5ea8(mon, k, rec[4 + k])` for k = 1..7, then
/// command 0x0B (`rec[4]`..`rec[0xB]`); returns 1. Else 0.
pub fn proc_0f<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    if !veneer_ab8(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        0x021C_9438,
        rec[4] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    if fainted(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), mon) {
        return 0;
    }
    for k in 1..=7u32 {
        env.call_l(
            0x021D_5EA8,
            &mut [Arg::V(mon), Arg::V(k), Arg::V(rec[4 + k as usize] as u32)],
            Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
        );
    }
    let args: Vec<u32> = rec[4..0xC].iter().map(|&b| b as u32).collect();
    put_cmd(&mut Lend::new_ref(env, w), q, 0x0B, &args, scratch);
    1
}

/// C: `FUN_overlay_d_93__021c947c`: HANDEX proc 0x10: for each of the `rec[4]` mon ids at +5
/// whose mon is alive: command 0x0D (id) and `5f08(mon)`. Returns 1 when any was alive.
pub fn proc_10<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let mut r = 0;
    let mut i = 0usize;
    while i < rec[4] as usize {
        let mon = mon_of(
            &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
            w,
            rec[5 + i] as u32,
        );
        if !fainted(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), mon) {
            put_cmd(
                &mut Lend::new_ref(env, w),
                q,
                0x0D,
                &[rec[5 + i] as u32],
                scratch,
            );
            env.call_l(
                0x021D_5F08,
                &mut [Arg::V(mon)],
                Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
            );
            r = 1;
        }
        i += 1;
    }
    r
}

/// C: `FUN_overlay_d_93__021c94c8`: HANDEX proc 0x11: when the veneer `95c0(work + 0x1AB8,
/// rec[0xE])` agrees: for each bit k (0..4) of `rec[0xF]`, `5970(mon, 8 + k, u16 rec[4 + 2k])` and
/// command 0x13 (`rec[0xE]`, 8 + k, low byte of that value); then the message at +0x10; returns
/// 1. Else 0.
pub fn proc_11<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    if !veneer_ab8(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        0x021C_95C0,
        rec[0xE] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[0xE] as u32,
    );
    for k in 0..5usize {
        if (rec[0xF] >> k) & 1 != 0 {
            let val = rd16(rec, 4 + 2 * k);
            let field = 8 + k as u32;
            env.call_l(
                0x021D_5970,
                &mut [Arg::V(mon), Arg::V(field), Arg::V(val)],
                Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
            );
            put_cmd(
                &mut Lend::new_ref(env, w),
                q,
                0x13,
                &[rec[0xE] as u32, field, val & 0xFF],
                scratch,
            );
        }
    }
    message(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 0x10),
    );
    1
}

/// C: `FUN_overlay_d_93__021c9448`: HANDEX proc 0x12: when `mon(rec[4])` is alive: command 0x0C
/// (`rec[4]`), returns the low byte of the BattleMon function `021d5efc(mon)`. Else 0.
pub fn proc_12<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    if fainted(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), mon) {
        return 0;
    }
    put_cmd(
        &mut Lend::new_ref(env, w),
        q,
        0x0C,
        &[rec[4] as u32],
        scratch,
    );
    env.call_l(
        0x021D_5EFC,
        &mut [Arg::V(mon)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) & 0xFF
}

/// C: `FUN_overlay_d_93__021c95d0`: HANDEX proc 0x13: when the veneer `961c(work + 0x1AB8,
/// rec[4])` agrees and `mon(rec[4])` is alive or `rec[5]` != 0: the message at +8 and core4
/// `2314(work, mon)`; returns 1. Else 0.
pub fn proc_13<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    if !veneer_ab8(
        &mut Lend::new_ref(env, w),
        0x021C_961C,
        rec[4] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(env, w, rec[4] as u32);
    if fainted(&mut Lend::new_ref(env, w), mon) && rec[5] == 0 {
        return 0;
    }
    message(&mut Lend::new_ref(env, w), str_at(rec, 8));
    env.callwr(0x021C_2314, &mut [Arg::P, Arg::V(mon)], w);
    1
}

/// C: `FUN_overlay_d_93__021c962c`: HANDEX proc 0x14: when the veneer `96b4(work + 0x1AB8,
/// rec[6])` agrees, `mon(rec[6])` is alive and the veneer `96c4(5444(mon))` is 0: command 0x16
/// (`rec[6]`, u16 `rec[4]`), `66b0(mon, u16 rec[4])`, and when `rec[7]` is 0 and `8da8(u16 rec[4])`
/// command list 0x5B (0x380, `rec[6]`, `8cfc(u16 rec[4])`); returns 1. Else 0.
pub fn proc_14<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    if !veneer_ab8(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        0x021C_96B4,
        rec[6] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[6] as u32,
    );
    if fainted(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), mon) {
        return 0;
    }
    let s = env.call_l(
        0x021D_5444,
        &mut [Arg::V(mon)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    if env.call_l(
        0x021C_96C4,
        &mut [Arg::V(s)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        return 0;
    }
    let mv = rd16(rec, 4);
    put_cmd(
        &mut Lend::new_ref(env, w),
        q,
        0x16,
        &[rec[6] as u32, mv],
        scratch,
    );
    env.call_l(
        0x021D_66B0,
        &mut [Arg::V(mon), Arg::V(mv)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    if rec[7] == 0
        && env.call_l(
            0x021E_8DA8,
            &mut [Arg::V(mv)],
            Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
        ) != 0
    {
        let x = env.call_l(
            0x021E_8CFC,
            &mut [Arg::V(mv)],
            Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
        );
        put_cmd_list(
            &mut Lend::new_ref(env, w),
            q,
            0x5B,
            0x380,
            &[rec[6] as u32, x, END],
        );
    }
    1
}

/// Shared shape of procs 0x15 / 0x16: when `mon(rec[8])` is alive, the BattleMon function
/// `addr(mon, u32 rec[4])`; returns 1. Else 0.
fn mon_word_call<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], addr: u32) -> u32 {
    let mon = mon_of(env, w, rec[8] as u32);
    if fainted(&mut Lend::new_ref(env, w), mon) {
        return 0;
    }
    env.callwr(addr, &mut [Arg::V(mon), Arg::V(rd32(rec, 4))], w);
    1
}

/// Shared shape of procs 0x17 / 0x18: when `mon(rec[8])` is alive, the core6 function
/// `addr(work, mon, u32 rec[4])`; returns 1. Else 0.
fn work_mon_word_call<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], addr: u32) -> u32 {
    let mon = mon_of(env, w, rec[8] as u32);
    if fainted(&mut Lend::new_ref(env, w), mon) {
        return 0;
    }
    env.callwr(addr, &mut [Arg::P, Arg::V(mon), Arg::V(rd32(rec, 4))], w);
    1
}

/// C: `FUN_overlay_d_93__021c9728`: HANDEX proc 0x15: `5fc0(mon(rec[8]), u32 rec[4])` when
/// alive (1), else 0.
pub fn proc_15<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    mon_word_call(env, w, rec, 0x021D_5FC0)
}

/// C: `FUN_overlay_d_93__021c9750`: HANDEX proc 0x16: `6440(mon(rec[8]), u32 rec[4])` when
/// alive (1), else 0.
pub fn proc_16<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    mon_word_call(env, w, rec, 0x021D_6440)
}

/// C: `FUN_overlay_d_93__021c9778`: HANDEX proc 0x17: core6 `60ec(work, mon(rec[8]), u32
/// rec[4])` when alive (1), else 0.
pub fn proc_17<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    work_mon_word_call(env, w, rec, 0x021C_60EC)
}

/// C: `FUN_overlay_d_93__021c97a4`: HANDEX proc 0x18: core6 `6110(work, mon(rec[8]), u32
/// rec[4])` when alive (1), else 0.
pub fn proc_18<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    work_mon_word_call(env, w, rec, 0x021C_6110)
}

/// C: `FUN_overlay_d_93__021c97d0`: HANDEX proc 0x19: `x = u32 rec[8]`; core6 `7ba8(work,
/// pokeID, u32 rec[4], rec[0xC], &x)` (x in/out); when the veneer `9818(rec[0xC], u32 rec[4], x)`
/// agrees, the message at +0x10 and 1. Else 0.
pub fn proc_19<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    let mut x: [u8; 4] = rec[8..0xC].try_into().unwrap();
    env.call(
        0x021C_7BA8,
        &mut [
            Arg::P,
            Arg::V(rec_poke_id(rec)),
            Arg::V(rd32(rec, 4)),
            Arg::V(rec[0xC] as u32),
            Arg::Out(&mut x),
        ],
    );
    if env.call(
        0x021C_9818,
        &mut [
            Arg::V(rec[0xC] as u32),
            Arg::V(rd32(rec, 4)),
            Arg::V(u32::from_le_bytes(x)),
        ],
    ) == 0
    {
        return 0;
    }
    message(env, str_at(rec, 0x10));
    1
}

/// C: `FUN_overlay_d_93__021c9828`: HANDEX proc 0x1A: for i in 0..14, when byte
/// `(i >> 3) + 1` is below `rec[4]` and bit `i & 7` of `rec[4 + (i >> 3) + 1]` is set and the
/// veneer `9888(rec[7], i)` agrees: core5 `4bac(work, i, rec[7])`. Returns 1 when any.
pub fn proc_1a<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    let mut r = 0;
    for i in 0..14u32 {
        let j = ((i >> 3) + 1) as usize;
        let hit = j < rec[4] as usize && (1u8 << (i & 7)) & rec[4 + j] != 0;
        if hit && env.call(0x021C_9888, &mut [Arg::V(rec[7] as u32), Arg::V(i)]) != 0 {
            env.call(0x021C_4BAC, &mut [Arg::P, Arg::V(i), Arg::V(rec[7] as u32)]);
            r = 1;
        }
    }
    r
}

/// C: `FUN_overlay_d_93__021c9898`: HANDEX proc 0x1B: when core5 `3c60(work, u32 rec[4],
/// u32 rec[8], rec[0xC])` succeeds, the message at +0x10 and 1. Else 0.
pub fn proc_1b<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    if env.call(
        0x021C_3C60,
        &mut [
            Arg::P,
            Arg::V(rd32(rec, 4)),
            Arg::V(rd32(rec, 8)),
            Arg::V(rec[0xC] as u32),
        ],
    ) == 0
    {
        return 0;
    }
    message(env, str_at(rec, 0x10));
    1
}

/// C: `FUN_overlay_d_93__021c98c4`: HANDEX proc 0x1C: when `ef8e8(u32 rec[4])` agrees, core5
/// `4c50(work, u32 rec[4])` and 1. Else 0.
pub fn proc_1c<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    if env.call(0x021E_F8E8, &mut [Arg::V(rd32(rec, 4))]) == 0 {
        return 0;
    }
    env.call(0x021C_4C50, &mut [Arg::P, Arg::V(rd32(rec, 4))]);
    1
}

/// C: `FUN_overlay_d_93__021c98e4`: HANDEX proc 0x1D (bit-23 bracket on the pokeID's mon around
/// a success): with `rec[4]` != 0, when core4 `3b8c(work, rec[4], rec[5])` agrees, `3bb8(work,
/// rec[4], rec[5])` and the message at +8; with `rec[4]` = 0 and `rec[6]` != 0, the message at +8
/// and core6 `7bf8(work)` inside a manager header save/restore (`advance_fields` / `set_word` on
/// work + 0x1D74). Returns 1 on success, else 0.
pub fn proc_1d<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, rec: &[u8]) -> u32 {
    let a = mon_of(env, w, rec_poke_id(rec));
    if rec[4] != 0 {
        if env.callw(
            0x021C_3B8C,
            &mut [Arg::P, Arg::V(rec[4] as u32), Arg::V(rec[5] as u32)],
            w,
        ) == 0
        {
            return 0;
        }
        if bit23(rec) {
            bracket_open(&mut Lend::new(env, w), a);
        }
        env.callw(
            0x021C_3BB8,
            &mut [Arg::P, Arg::V(rec[4] as u32), Arg::V(rec[5] as u32)],
            w,
        );
        message(&mut Lend::new(env, w), str_at(rec, 8));
    } else {
        if rec[6] == 0 {
            return 0;
        }
        if bit23(rec) {
            bracket_open(&mut Lend::new(env, w), a);
        }
        message(&mut Lend::new(env, w), str_at(rec, 8));
        let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
        env.callw(0x021C_7BF8, &mut [Arg::P], w);
        set_word(&mut w.mem[HANDEX_MGR..], old);
    }
    if bit23(rec) {
        bracket_close(&mut Lend::new(env, w), a);
    }
    1
}

/// C: `FUN_overlay_d_93__021c9a24`: HANDEX proc 0x1E: whether the veneer `9a50(u32 rec[4],
/// rec[8], pokeID, rec + 0xC (0x10 bytes), rec[0x1C])` is nonzero.
pub fn proc_1e<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    (env.call(
        0x021C_9A50,
        &mut [
            Arg::V(rd32(rec, 4)),
            Arg::V(rec[8] as u32),
            Arg::V(rec_poke_id(rec)),
            Arg::In(&rec[0xC..0x1C]),
            Arg::V(rec[0x1C] as u32),
        ],
    ) != 0) as u32
}

/// C: `FUN_overlay_d_93__021c9a60`: HANDEX proc 0x1F (type change?): on `t = mon(rec[6])` with
/// `cur = (u16) BattleMon_Get(t, 0x10)`: 0 when the veneer `9ba8(cur)` agrees, `t` has fainted,
/// or `rec[7]` is 0 and u16 `rec[4]` equals `cur`. Else (commands 0x57 / 0x58 with the pokeID
/// around it when header bit 23 is set) command 0x49 (`rec[6]`, u16 `rec[4]`), the message at
/// +0xC, core6 `7b3c(work, rec[6], cur, u16 rec[4])` in a manager header save/restore, `8510(t)`,
/// `66d0(t, u16 rec[4])`, command 0x1D (`rec[6]`, u16 `rec[4]`), `8484(t)`, command 0x58
/// (`rec[6]`), and when the value changed core6 `7b7c(work, rec[6])` in another bracket; finally,
/// unless `62c4(t, 0x10)`, core4 `195c(work, t, 0)` for 0x67 and core5 `3fdc(work, t)` for 0x7F;
/// returns 1.
pub fn proc_1f<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let t = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[6] as u32,
    );
    let cur = env.call_l(
        0x021D_59B4,
        &mut [Arg::V(t), Arg::V(0x10)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) & 0xFFFF;
    if env.call_l(
        0x021C_9BA8,
        &mut [Arg::V(cur)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        return 0;
    }
    if fainted(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), t) {
        return 0;
    }
    let new = rd16(rec, 4);
    if rec[7] == 0 && new == cur {
        return 0;
    }
    if bit23(rec) {
        put_cmd(
            &mut Lend::new(env, w),
            q,
            0x57,
            &[rec_poke_id(rec)],
            scratch,
        );
    }
    put_cmd(
        &mut Lend::new(env, w),
        q,
        0x49,
        &[rec[6] as u32, new],
        scratch,
    );
    message(
        &mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 0xC),
    );
    let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
    env.call_l(
        0x021C_7B3C,
        &mut [Arg::P, Arg::V(rec[6] as u32), Arg::V(cur), Arg::V(new)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    set_word(&mut w.mem[HANDEX_MGR..], old);
    env.call_l(
        0x021D_8510,
        &mut [Arg::V(t)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    env.call_l(
        0x021D_66D0,
        &mut [Arg::V(t), Arg::V(new)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    put_cmd(
        &mut Lend::new(env, w),
        q,
        0x1D,
        &[rec[6] as u32, new],
        scratch,
    );
    env.call_l(
        0x021D_8484,
        &mut [Arg::V(t)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    put_cmd(&mut Lend::new(env, w), q, 0x58, &[rec[6] as u32], scratch);
    if cur != new {
        let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
        env.call_l(
            0x021C_7B7C,
            &mut [Arg::P, Arg::V(rec[6] as u32)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        set_word(&mut w.mem[HANDEX_MGR..], old);
    }
    if bit23(rec) {
        put_cmd(
            &mut Lend::new(env, w),
            q,
            0x58,
            &[rec_poke_id(rec)],
            scratch,
        );
    }
    if env.call_l(
        0x021D_62C4,
        &mut [Arg::V(t), Arg::V(0x10)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) == 0
    {
        if cur == 0x67 {
            env.call_l(
                0x021C_195C,
                &mut [Arg::P, Arg::V(t), Arg::V(0)],
                Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
            );
        }
        if cur == 0x7F {
            env.call_l(
                0x021C_3FDC,
                &mut [Arg::P, Arg::V(t)],
                Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
            );
        }
    }
    1
}

/// C: `FUN_overlay_d_93__021c9bb8`: HANDEX proc 0x20: on `t = mon(rec[6])`: when the pokeID is
/// not `rec[6]`, core6 asks `7a54(work, t, u16 rec[4])` inside a manager header save/restore; a
/// yes runs core6 `7aa0(work, t)` in another one and returns 0. Else (commands 0x57 / 0x58 with
/// the pokeID around it when header bit 23 is set) the message at +0xC, core4 `2250(work, t,
/// u16 rec[4], 0)`, then for `rec[7]` / `rec[8]` (second mon `rec[9]`) `6738(mon)` + command 0x2B
/// (its id), and core4 `195c(work, t, 0)`; returns 1.
pub fn proc_20<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let t = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[6] as u32,
    );
    if rec_poke_id(rec) != rec[6] as u32 {
        let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
        let yes = env.call_l(
            0x021C_7A54,
            &mut [Arg::P, Arg::V(t), Arg::V(rd16(rec, 4))],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        ) & 0xFF;
        set_word(&mut w.mem[HANDEX_MGR..], old);
        if yes != 0 {
            let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
            env.call_l(
                0x021C_7AA0,
                &mut [Arg::P, Arg::V(t)],
                Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
            );
            set_word(&mut w.mem[HANDEX_MGR..], old);
            return 0;
        }
    }
    if bit23(rec) {
        put_cmd(
            &mut Lend::new(env, w),
            q,
            0x57,
            &[rec_poke_id(rec)],
            scratch,
        );
    }
    message(
        &mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 0xC),
    );
    env.call_l(
        0x021C_2250,
        &mut [Arg::P, Arg::V(t), Arg::V(rd16(rec, 4)), Arg::V(0)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    if bit23(rec) {
        put_cmd(
            &mut Lend::new(env, w),
            q,
            0x58,
            &[rec_poke_id(rec)],
            scratch,
        );
    }
    if rec[7] != 0 {
        env.call_l(
            0x021D_6738,
            &mut [Arg::V(t)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        put_cmd(&mut Lend::new(env, w), q, 0x2B, &[rec[6] as u32], scratch);
    }
    if rec[8] != 0 {
        let m2 = mon_of(
            &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
            w,
            rec[9] as u32,
        );
        env.call_l(
            0x021D_6738,
            &mut [Arg::V(m2)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        put_cmd(&mut Lend::new(env, w), q, 0x2B, &[rec[9] as u32], scratch);
    }
    env.call_l(
        0x021C_195C,
        &mut [Arg::P, Arg::V(t), Arg::V(0)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    1
}

/// C: `FUN_overlay_d_93__021c9dcc`: HANDEX proc 0x21: core4 `195c(work, mon(rec[4]), u32
/// rec[8])`; returns 1.
pub fn proc_21<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec[4] as u32);
    env.callwr(
        0x021C_195C,
        &mut [Arg::P, Arg::V(mon), Arg::V(rd32(rec, 8))],
        w,
    );
    1
}

/// C: `FUN_overlay_d_93__021c9e3c`: HANDEX proc 0x22: on `t = mon(rec[4])` alive with
/// `p = dcf78(t, u16 rec[6])` nonzero: a placeholder command 0x42, core6 `6e00(work, t, pokeID)`
/// inside a manager header save/restore, and when hem's state `88e4(work)` is 2 the placeholder
/// is rewritten as command 0x42 (`rec[4]`); then `d7040(p)` (through the thunk 021dcf9c);
/// returns 1. Else 0.
pub fn proc_22<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let t = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    if fainted(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), t) {
        return 0;
    }
    let p = env.call_l(
        0x021D_CF78,
        &mut [Arg::V(t), Arg::V(rd16(rec, 6))],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    if p == 0 {
        return 0;
    }
    let pos = put_cmd_placeholder(&mut Lend::new(env, w), q, 0x42, scratch);
    let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
    env.call_l(
        0x021C_6E00,
        &mut [Arg::P, Arg::V(t), Arg::V(rec_poke_id(rec))],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    if env.call_l(
        0x021C_88E4,
        &mut [Arg::P],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) == 2
    {
        put_cmd_at(
            &mut Lend::new(env, w),
            q,
            pos,
            0x42,
            &[rec[4] as u32],
            scratch,
        );
    }
    set_word(&mut w.mem[HANDEX_MGR..], old);
    env.call_l(
        0x021D_7040,
        &mut [Arg::V(p)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    1
}

/// C: `FUN_overlay_d_93__021c9ecc`: HANDEX proc 0x23: on the pokeID's mon: when u32 `rec[4]` is
/// 0, core6 `60d4(work, mon)` and the message at +8; then core4 `2250(work, mon, 0, 1)`; returns
/// 1.
pub fn proc_23<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec_poke_id(rec));
    if rd32(rec, 4) == 0 {
        env.callwr(0x021C_60D4, &mut [Arg::P, Arg::V(mon)], w);
        message(&mut Lend::new_ref(env, w), str_at(rec, 8));
    }
    env.callwr(
        0x021C_2250,
        &mut [Arg::P, Arg::V(mon), Arg::V(0), Arg::V(1)],
        w,
    );
    1
}

/// C: `FUN_overlay_d_93__021c9cc0`: HANDEX proc 0x24 (swap-like): `a = mon(rec[4])`, `b` = the
/// pokeID's mon, `vb` / `va` = u16 `5b40` of each; core6 asks `7a54(work, a, vb)` inside a manager
/// header save/restore, a yes runs `7aa0(work, a)` in another one and returns 0. Else (bit-23
/// bracket on `b`) the messages at +8 / +0x2C / +0x50, core4 `2250(work, b, va, 0)`, `2250(work,
/// a, vb, 0)`, `195c(work, b, 0)`, `195c(work, a, 0)`; returns 1.
pub fn proc_24<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, rec: &[u8]) -> u32 {
    let a = mon_of(env, w, rec[4] as u32);
    let b = mon_of(env, w, rec_poke_id(rec));
    let vb = env.callw(0x021D_5B40, &mut [Arg::V(b)], w) & 0xFFFF;
    let va = env.callw(0x021D_5B40, &mut [Arg::V(a)], w) & 0xFFFF;
    let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
    let yes = env.callw(0x021C_7A54, &mut [Arg::P, Arg::V(a), Arg::V(vb)], w) & 0xFF;
    set_word(&mut w.mem[HANDEX_MGR..], old);
    if yes != 0 {
        let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
        env.callw(0x021C_7AA0, &mut [Arg::P, Arg::V(a)], w);
        set_word(&mut w.mem[HANDEX_MGR..], old);
        return 0;
    }
    if bit23(rec) {
        bracket_open(&mut Lend::new(env, w), b);
    }
    message(&mut Lend::new(env, w), str_at(rec, 8));
    message(&mut Lend::new(env, w), str_at(rec, 0x2C));
    message(&mut Lend::new(env, w), str_at(rec, 0x50));
    if bit23(rec) {
        bracket_close(&mut Lend::new(env, w), b);
    }
    env.callw(
        0x021C_2250,
        &mut [Arg::P, Arg::V(b), Arg::V(va), Arg::V(0)],
        w,
    );
    env.callw(
        0x021C_2250,
        &mut [Arg::P, Arg::V(a), Arg::V(vb), Arg::V(0)],
        w,
    );
    env.callw(0x021C_195C, &mut [Arg::P, Arg::V(b), Arg::V(0)], w);
    env.callw(0x021C_195C, &mut [Arg::P, Arg::V(a), Arg::V(0)], w);
    1
}

/// C: `FUN_overlay_d_93__021c9f0c`: HANDEX proc 0x25: puts command 0x1F (`rec[4]`, `rec[5]`,
/// `rec[6]`, `rec[7]`, u16 `rec[8]`) on the queue `q`, then `021d5734(mon(rec[4]), rec[5],
/// u16 rec[8], rec[6], rec[7])`; returns 1.
pub fn proc_25<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    let v8 = rd16(rec, 8);
    let args = [
        rec[4] as u32,
        rec[5] as u32,
        rec[6] as u32,
        rec[7] as u32,
        v8,
    ];
    cmd_queue::put_cmd(&mut Lend::new_ref(env, w), q, 0x1F, &args, scratch);
    env.call_l(
        0x021D_5734,
        &mut [
            Arg::V(mon),
            Arg::V(rec[5] as u32),
            Arg::V(v8),
            Arg::V(rec[6] as u32),
            Arg::V(rec[7] as u32),
        ],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    1
}

/// C: `FUN_overlay_d_93__021c9f4c`: HANDEX proc 0x26: core3 `0780(work, mon(rec[4]), rec[5],
/// rec[6])`; returns 1.
pub fn proc_26<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec[4] as u32);
    env.callwr(
        0x021C_0780,
        &mut [
            Arg::P,
            Arg::V(mon),
            Arg::V(rec[5] as u32),
            Arg::V(rec[6] as u32),
        ],
        w,
    );
    1
}

/// C: `FUN_overlay_d_93__021c9f6c`: HANDEX proc 0x27 (move-like): `a = mon(rec[4])`,
/// `b = mon(rec[5])`; core5 `64b4(work, u16 rec[6], a, &m)` fills a 0x14-byte record m; the
/// veneers `a0b8(work[0x860])`, `a0c8(o)`, `a0d8(o, b)`, `a0e8(o, 1)`, `a0f8(o)` on
/// `o = work[0x868]`; `a108(o)` -> core5 `5718` and 0. Else core3 `f83c` / `f3d8` / `f4ac` /
/// `f8d4(work, &m, a, o[, work + 0x1F88])`; `a118(o)` -> 0. Else, with the three halfwords of `d`
/// (the record behind work + 0x4A8) saved: core2 `d1e0(d)`, `d200(d, work, a, o)`, `d[4] = 1`,
/// core3 `c091c(work, &m, a, o, work + 0x1F88, 1)`; returns bit 0 of `d[5]` (read before the
/// halfwords are restored).
pub fn proc_27<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], d: &mut [u8]) -> u32 {
    let a = mon_of(env, w, rec[4] as u32);
    let b = mon_of(env, w, rec[5] as u32);
    let mut m = [0u8; 0x14];
    env.callwr(
        0x021C_64B4,
        &mut [Arg::P, Arg::V(rd16(rec, 6)), Arg::V(a), Arg::Out(&mut m)],
        w,
    );
    env.callwr(0x021C_A0B8, &mut [Arg::V(w.u32_at(0x860))], w);
    let o = w.u32_at(0x868);
    env.callwr(0x021C_A0C8, &mut [Arg::V(o)], w);
    env.callwr(0x021C_A0D8, &mut [Arg::V(o), Arg::V(b)], w);
    env.callwr(0x021C_A0E8, &mut [Arg::V(o), Arg::V(1)], w);
    env.callwr(0x021C_A0F8, &mut [Arg::V(o)], w);
    if env.callwr(0x021C_A108, &mut [Arg::V(o)], w) != 0 {
        env.callwr(0x021C_5718, &mut [Arg::P], w);
        return 0;
    }
    let mr = &m[..0x10];
    env.callwr(
        0x021B_F83C,
        &mut [Arg::P, Arg::In(mr), Arg::V(a), Arg::V(o)],
        w,
    );
    env.callwr(
        0x021B_F3D8,
        &mut [Arg::P, Arg::In(mr), Arg::V(a), Arg::V(o), Arg::P],
        w,
    );
    env.callwr(
        0x021B_F4AC,
        &mut [Arg::P, Arg::In(mr), Arg::V(a), Arg::V(o), Arg::P],
        w,
    );
    env.callwr(
        0x021B_F8D4,
        &mut [Arg::P, Arg::In(mr), Arg::V(a), Arg::V(o)],
        w,
    );
    if env.callwr(0x021C_A118, &mut [Arg::V(o)], w) != 0 {
        return 0;
    }
    let mut save = [0u8; 6];
    save.copy_from_slice(&d[..6]);
    env.callwr(0x021B_D1E0, &mut [Arg::Out(&mut d[..6])], w);
    env.callwr(
        0x021B_D200,
        &mut [Arg::Out(&mut d[..6]), Arg::P, Arg::V(a), Arg::V(o)],
        w,
    );
    d[4] = 1;
    // the original passes work; c091c reaches `d` through work + 0x4A8 (the spec's `outki`)
    env.callwr(
        0x021C_091C,
        &mut [
            Arg::Out(&mut d[..6]),
            Arg::In(mr),
            Arg::V(a),
            Arg::V(o),
            Arg::P,
            Arg::V(1),
        ],
        w,
    );
    let f = (d[5] & 1) as u32;
    d[..6].copy_from_slice(&save);
    f
}

/// C: `FUN_overlay_d_93__021ca128`: HANDEX proc 0x28: when core2 `d60c(work, mon(pokeID), 1)`
/// agrees, `work[0x14] = 5` and 1. Else 0.
pub fn proc_28<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec_poke_id(rec));
    if env.callw(0x021B_D60C, &mut [Arg::P, Arg::V(mon), Arg::V(1)], w) == 0 {
        return 0;
    }
    w.set_u32_at(0x14, 5);
    1
}

/// C: `FUN_overlay_d_93__021ca158`: HANDEX proc 0x29: on `t = mon(rec[0x4C])`: 0 when core1
/// `c2b0(work)`, api `8374(work, rec[0x4C])` or `work[0x14]` say so. Else the message at +4;
/// when core2 `dd00(work, t, rec[0x4D])` agrees, server_side `b930(server, b8d8c(work[4],
/// work[8], rec[0x4C]))`, the message at +0x28, `work[0x14] = 1`, returns 1. Else 0.
pub fn proc_29<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, rec: &[u8]) -> u32 {
    let id = rec[0x4C] as u32;
    let t = mon_of(env, w, id);
    if env.callw(0x021B_C2B0, &mut [Arg::P], w) != 0 {
        return 0;
    }
    if env.callw(0x021C_8374, &mut [Arg::P, Arg::V(id)], w) != 0 {
        return 0;
    }
    if w.u32_at(0x14) != 0 {
        return 0;
    }
    message(&mut Lend::new(env, w), str_at(rec, 4));
    if env.callw(
        0x021B_DD00,
        &mut [Arg::P, Arg::V(t), Arg::V(rec[0x4D] as u32)],
        w,
    ) == 0
    {
        return 0;
    }
    let pos = env.callw(
        0x021B_8D8C,
        &mut [Arg::V(w.u32_at(4)), Arg::V(w.u32_at(8)), Arg::V(id)],
        w,
    );
    env.callw(0x021B_B930, &mut [Arg::P, Arg::V(pos)], w);
    message(&mut Lend::new(env, w), str_at(rec, 0x28));
    w.set_u32_at(0x14, 1);
    1
}

/// C: `FUN_overlay_d_93__021ca1d0`: HANDEX proc 0x2A: `a = mon(rec[4])`, `b = mon(rec[5])`;
/// `62c4(a, 0x10)` -> core4 `2ca8(work, b)`; `65c8(b, a)`; command 0x26 (`rec[4]`, `rec[5]`);
/// `6d7c(b)` -> command 0x51 (veneer `a238(work + 0x1AB8, rec[5])`); returns 1.
pub fn proc_2a<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let a = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    let b = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[5] as u32,
    );
    if env.call_l(
        0x021D_62C4,
        &mut [Arg::V(a), Arg::V(0x10)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        env.call_l(
            0x021C_2CA8,
            &mut [Arg::P, Arg::V(b)],
            Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
        );
    }
    env.call_l(
        0x021D_65C8,
        &mut [Arg::V(b), Arg::V(a)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    put_cmd(
        &mut Lend::new_ref(env, w),
        q,
        0x26,
        &[rec[4] as u32, rec[5] as u32],
        scratch,
    );
    if env.call_l(
        0x021D_6D7C,
        &mut [Arg::V(b)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        let x = env.call_l(
            0x021C_A238,
            &mut [
                Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize),
                Arg::V(rec[5] as u32),
            ],
            Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
        );
        put_cmd(&mut Lend::new_ref(env, w), q, 0x51, &[x], scratch);
    }
    1
}

/// C: `FUN_overlay_d_93__021ca248`: HANDEX proc 0x2B: whether core4 `1d60(work, mon(rec[4]),
/// rec[5])` is nonzero.
pub fn proc_2b<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec[4] as u32);
    (env.callwr(
        0x021C_1D60,
        &mut [Arg::P, Arg::V(mon), Arg::V(rec[5] as u32)],
        w,
    ) != 0) as u32
}

/// C: `FUN_overlay_d_93__021ca270`: HANDEX proc 0x2C: `5fa4(mon(rec[4]), u16 rec[6])`, command
/// 2 (`rec[4]`, u16 `rec[6]`), `work[0x7A9 + rec[4]] = 0`, the message at +8; when the position
/// `b8d8c(work[4], work[8], rec[4])` is not 6: core2 `d9d4(work, b8d08(rec[4]), x, x, 1)` with
/// `x = b8dec(work[4], pos)`, then `da00(work)`; returns 1.
pub fn proc_2c<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let id = rec[4] as u32;
    let mon = mon_of(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), w, id);
    env.call_l(
        0x021D_5FA4,
        &mut [Arg::V(mon), Arg::V(rd16(rec, 6))],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    put_cmd(&mut Lend::new(env, w), q, 2, &[id, rd16(rec, 6)], scratch);
    w.set_u8_at(0x7A9 + id as usize, 0);
    message(
        &mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 8),
    );
    let pos = env.call_l(
        0x021B_8D8C,
        &mut [Arg::V(w.u32_at(4)), Arg::V(w.u32_at(8)), Arg::V(id)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    if pos != 6 {
        let c = env.call_l(
            0x021B_8D08,
            &mut [Arg::V(id)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        let x = env.call_l(
            0x021B_8DEC,
            &mut [Arg::V(w.u32_at(4)), Arg::V(pos)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        env.call_l(
            0x021B_D9D4,
            &mut [Arg::P, Arg::V(c), Arg::V(x), Arg::V(x), Arg::V(1)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        env.call_l(
            0x021B_DA00,
            &mut [Arg::P],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
    }
    1
}

/// C: `FUN_overlay_d_93__021ca2e8`: HANDEX proc 0x2D: `67d8(mon(rec[4]), u16 rec[6])`, command
/// 0x14 (`rec[4]`, u16 `rec[6]`), the message at +8; returns 1.
pub fn proc_2d<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    env.call_l(
        0x021D_67D8,
        &mut [Arg::V(mon), Arg::V(rd16(rec, 6))],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    put_cmd(
        &mut Lend::new_ref(env, w),
        q,
        0x14,
        &[rec[4] as u32, rd16(rec, 6)],
        scratch,
    );
    message(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 8),
    );
    1
}

/// C: `FUN_overlay_d_93__021ca318`: HANDEX proc 0x2E: whether core4 `3810(work, mon(pokeID),
/// mon(rec[6]), rec[7] & 0xF, &out, u16 rec[4], rec[7] >> 4, rec + 8)` is nonzero.
pub fn proc_2e<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let t = mon_of(env, w, rec[6] as u32);
    let p = mon_of(env, w, rec_poke_id(rec));
    let mut out = [0u8; 4];
    (env.callwr(
        0x021C_3810,
        &mut [
            Arg::P,
            Arg::V(p),
            Arg::V(t),
            Arg::V((rec[7] & 0xF) as u32),
            Arg::Out(&mut out),
            Arg::V(rd16(rec, 4)),
            Arg::V((rec[7] >> 4) as u32),
            Arg::In(&rec[8..0x2C]),
        ],
        w,
    ) != 0) as u32
}

/// Shared shape of procs 0x2F / 0x31: when core2 `addr(work, rec[4])` agrees, the message at +8
/// and 1. Else 0.
fn check_then_message<E: FlowEnv>(env: &mut E, rec: &[u8], addr: u32) -> u32 {
    if env.call(addr, &mut [Arg::P, Arg::V(rec[4] as u32)]) == 0 {
        return 0;
    }
    message(env, str_at(rec, 8));
    1
}

/// C: `FUN_overlay_d_93__021ca370`: HANDEX proc 0x2F: core2 `d148(work, rec[4])` -> the message
/// at +8 and 1, else 0.
pub fn proc_2f<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    check_then_message(env, rec, 0x021B_D148)
}

/// C: `FUN_overlay_d_93__021ca394`: HANDEX proc 0x30: whether core2 `d170(work, u16 rec[4])` is
/// nonzero.
pub fn proc_30<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    (env.call(0x021B_D170, &mut [Arg::P, Arg::V(rd16(rec, 4))]) != 0) as u32
}

/// C: `FUN_overlay_d_93__021ca3ac`: HANDEX proc 0x31: core2 `d1b0(work, rec[4])` -> the message
/// at +8 and 1, else 0.
pub fn proc_31<E: FlowEnv>(env: &mut E, rec: &[u8]) -> u32 {
    check_then_message(env, rec, 0x021B_D1B0)
}

/// C: `FUN_overlay_d_93__021ca3d0`: HANDEX proc 0x32 (party swap): `rec[4]` != `rec[5]` on the
/// same client `c = b8d08(id)`, both mons alive, both party indices `9c98(b9aec(work[8], c), mon)`
/// (as s16) non-negative: core2 `d43c(work, c, i, j, 0)`, the message at +8, `d4e4(work, c, i,
/// j)`; returns 1. Else 0.
pub fn proc_32<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    if rec[4] == rec[5] {
        return 0;
    }
    let c = env.callwr(0x021B_8D08, &mut [Arg::V(rec[4] as u32)], w);
    let c2 = env.callwr(0x021B_8D08, &mut [Arg::V(rec[5] as u32)], w);
    if c != c2 {
        return 0;
    }
    let a = mon_of(env, w, rec[4] as u32);
    let b = mon_of(env, w, rec[5] as u32);
    if fainted(&mut Lend::new_ref(env, w), a) || fainted(&mut Lend::new_ref(env, w), b) {
        return 0;
    }
    let party = env.callwr(0x021B_9AEC, &mut [Arg::V(w.u32_at(8)), Arg::V(c)], w);
    let i = env.callwr(0x021B_9C98, &mut [Arg::V(party), Arg::V(a)], w) as u16 as i16;
    let j = env.callwr(0x021B_9C98, &mut [Arg::V(party), Arg::V(b)], w) as u16 as i16;
    if i < 0 || j < 0 {
        return 0;
    }
    let (i, j) = ((i as u32) & 0xFF, (j as u32) & 0xFF);
    env.callwr(
        0x021B_D43C,
        &mut [Arg::P, Arg::V(c), Arg::V(i), Arg::V(j), Arg::V(0)],
        w,
    );
    message(&mut Lend::new_ref(env, w), str_at(rec, 8));
    env.callwr(
        0x021B_D4E4,
        &mut [Arg::P, Arg::V(c), Arg::V(i), Arg::V(j)],
        w,
    );
    1
}

/// C: `FUN_overlay_d_93__021ca47c`: HANDEX proc 0x33: `a` = the pokeID's mon, `b = mon(rec[4])`:
/// 0 when `6c24(a)`, `6c24(b)` or `5c30(a)` say so or `6c50(a, b)` is 0. Else with
/// `cur = (u16) Get(a, 0x10)`, `ia = 5440(a)`, `ib = 5440(b)` (bit-23 bracket on `a`):
/// `e0434(a)`, `8510(a)`, `8484(a)`, command 0x53 (ia, ib), the message at +8; when `Get(a,
/// 0x11)` differs from `cur`, core6 `7b7c(work, ia)` inside a manager header save/restore;
/// returns 1.
pub fn proc_33<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let a = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec_poke_id(rec),
    );
    let b = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    if env.call_l(
        0x021D_6C24,
        &mut [Arg::V(a)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        return 0;
    }
    if env.call_l(
        0x021D_6C24,
        &mut [Arg::V(b)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        return 0;
    }
    if env.call_l(
        0x021D_5C30,
        &mut [Arg::V(a)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        return 0;
    }
    let cur = env.call_l(
        0x021D_59B4,
        &mut [Arg::V(a), Arg::V(0x10)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) & 0xFFFF;
    if env.call_l(
        0x021D_6C50,
        &mut [Arg::V(a), Arg::V(b)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    ) == 0
    {
        return 0;
    }
    let ia = env.call_l(
        0x021D_5440,
        &mut [Arg::V(a)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    let ib = env.call_l(
        0x021D_5440,
        &mut [Arg::V(b)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    if bit23(rec) {
        bracket_open(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), a);
    }
    env.call_l(
        0x021E_0434,
        &mut [Arg::V(a)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    env.call_l(
        0x021D_8510,
        &mut [Arg::V(a)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    env.call_l(
        0x021D_8484,
        &mut [Arg::V(a)],
        Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
    );
    put_cmd(&mut Lend::new(env, w), q, 0x53, &[ia, ib], scratch);
    message(
        &mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 8),
    );
    if bit23(rec) {
        bracket_close(&mut Lend::wh(env, w, vec![&mut *q, &mut *scratch]), a);
    }
    if cur
        != env.call_l(
            0x021D_59B4,
            &mut [Arg::V(a), Arg::V(0x11)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        )
    {
        let old = advance_fields(&mut w.mem[HANDEX_MGR..]);
        env.call_l(
            0x021C_7B7C,
            &mut [Arg::P, Arg::V(ia)],
            Lent::of(Some(&mut *w), None, vec![&mut *q, &mut *scratch]),
        );
        set_word(&mut w.mem[HANDEX_MGR..], old);
    }
    1
}

/// C: `FUN_overlay_d_93__021ca570`: HANDEX proc 0x34: when the veneer `a5bc(work + 0x1AB8,
/// rec[4])` and `6c24(mon(rec[4]))` agree: `6c2c(mon)`, command 0x4B (`rec[4]`), the message at
/// +8; returns 1. Else 0.
pub fn proc_34<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    if !veneer_ab8(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        0x021C_A5BC,
        rec[4] as u32,
        w.mem[0x1AB8..].as_ptr() as usize,
    ) {
        return 0;
    }
    let mon = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    if env.call_l(
        0x021D_6C24,
        &mut [Arg::V(mon)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) == 0
    {
        return 0;
    }
    env.call_l(
        0x021D_6C2C,
        &mut [Arg::V(mon)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    put_cmd(
        &mut Lend::new_ref(env, w),
        q,
        0x4B,
        &[rec[4] as u32],
        scratch,
    );
    message(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 8),
    );
    1
}

/// C: `FUN_overlay_d_93__021ca5cc`: HANDEX proc 0x35: api `7f68(work, 0x800 | b8d8c(work[4],
/// work[8], pokeID), ids)` lists mon ids; for each: `5bdc(mon, 3)` -> core5 `5418(work, mon, 3)`
/// and, when `69ac(mon, 4)` - 1 is below 0x18, `0203efc8(work, mon, 1)`; core6 `6f34(work, mon,
/// 1)`; `62c4(mon, 0x1E)` / `62c4(mon, 0x20)` -> core5 `5620(work, mon, 0x1E / 0x20, null)`; any
/// of these -> command list 0x5B (0x43B, id). Returns 1.
pub fn proc_35<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], q: &mut [u8]) -> u32 {
    let pos = env.call_l(
        0x021B_8D8C,
        &mut [
            Arg::V(w.u32_at(4)),
            Arg::V(w.u32_at(8)),
            Arg::V(rec_poke_id(rec)),
        ],
        Lent::of(None, Some(w), vec![&mut *q]),
    );
    let mut ids = [0u8; 4];
    let n = env.call_l(
        0x021C_7F68,
        &mut [Arg::P, Arg::V((0x800 | pos) & 0xFFFF), Arg::Out(&mut ids)],
        Lent::of(None, Some(w), vec![&mut *q]),
    );
    let mut i = 0u32;
    while i < n {
        let id = ids[i as usize] as u32;
        let mon = mon_of(&mut Lend::hv(env, vec![&mut *q]), w, id);
        let mut any = false;
        if env.call_l(
            0x021D_5BDC,
            &mut [Arg::V(mon), Arg::V(3)],
            Lent::of(None, Some(w), vec![&mut *q]),
        ) != 0
        {
            env.call_l(
                0x021C_5418,
                &mut [Arg::P, Arg::V(mon), Arg::V(3)],
                Lent::of(None, Some(w), vec![&mut *q]),
            );
            let s = env.call_l(
                0x021D_69AC,
                &mut [Arg::V(mon), Arg::V(4)],
                Lent::of(None, Some(w), vec![&mut *q]),
            );
            let s = if s == 0 {
                0x1F
            } else {
                let x = s.wrapping_sub(1) & 0xFF;
                if x < 0x18 {
                    x
                } else {
                    0x1F
                }
            };
            if s != 0x1F {
                env.call_l(
                    0x0203_EFC8,
                    &mut [Arg::P, Arg::V(mon), Arg::V(1)],
                    Lent::of(None, Some(w), vec![&mut *q]),
                );
            }
            any = true;
        }
        if env.call_l(
            0x021C_6F34,
            &mut [Arg::P, Arg::V(mon), Arg::V(1)],
            Lent::of(None, Some(w), vec![&mut *q]),
        ) != 0
        {
            any = true;
        }
        for st in [0x1Eu32, 0x20] {
            if env.call_l(
                0x021D_62C4,
                &mut [Arg::V(mon), Arg::V(st)],
                Lent::of(None, Some(w), vec![&mut *q]),
            ) != 0
            {
                env.call_l(
                    0x021C_5620,
                    &mut [Arg::P, Arg::V(mon), Arg::V(st), Arg::V(0)],
                    Lent::of(None, Some(w), vec![&mut *q]),
                );
                any = true;
            }
        }
        if any {
            put_cmd_list(&mut Lend::new_ref(env, w), q, 0x5B, 0x43B, &[id, END]);
        }
        i = (i + 1) & 0xFF;
    }
    1
}

/// C: `FUN_overlay_d_93__021ca6c4`: HANDEX proc 0x36: when `mon(rec[4])` is alive and core5
/// `5418(work, mon, u32 rec[8])` agrees, the message at +0xC and 1. Else 0.
pub fn proc_36<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8]) -> u32 {
    let mon = mon_of(env, w, rec[4] as u32);
    if fainted(&mut Lend::new_ref(env, w), mon) {
        return 0;
    }
    if env.callwr(
        0x021C_5418,
        &mut [Arg::P, Arg::V(mon), Arg::V(rd32(rec, 8))],
        w,
    ) == 0
    {
        return 0;
    }
    message(&mut Lend::new_ref(env, w), str_at(rec, 0xC));
    1
}

/// C: `FUN_overlay_d_93__021ca6fc`: HANDEX proc 0x37: `rec[0xB]` != 0 -> command 0x56 (0);
/// core6 `70e4(work, u16 rec[4], rec[6], rec[7], rec[0xA], u16 rec[8])`; the message at +0xC;
/// returns 1.
pub fn proc_37<E: FlowEnv>(env: &mut E, rec: &[u8], q: &mut [u8], scratch: &mut [u8]) -> u32 {
    if rec[0xB] != 0 {
        put_cmd(env, q, 0x56, &[0], scratch);
    }
    env.call_l(
        0x021C_70E4,
        &mut [
            Arg::P,
            Arg::V(rd16(rec, 4)),
            Arg::V(rec[6] as u32),
            Arg::V(rec[7] as u32),
            Arg::V(rec[0xA] as u32),
            Arg::V(rd16(rec, 8)),
        ],
        Lent::of(None, None, vec![&mut *q, &mut *scratch]),
    );
    message(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        str_at(rec, 0xC),
    );
    1
}

/// C: `FUN_overlay_d_93__021ca738`: HANDEX proc 0x38: puts command 0x56 (0) on the queue `q`;
/// returns 1 (the record is not read).
pub fn proc_38<E: FlowEnv>(env: &mut E, q: &mut [u8], scratch: &mut [u8]) -> u32 {
    cmd_queue::put_cmd(env, q, 0x56, &[0], scratch);
    1
}

/// C: `FUN_overlay_d_93__021ca748`: HANDEX proc 0x39: on `t = mon(rec[4])` alive, `6d54(t)`
/// clear and `(u8) Get(t, 0x13)` != `rec[5]`: (bit-23 bracket on `t`) `66d8(t, rec[5])`, command
/// 0x4F (`rec[4]`, `rec[5]`), the message at +8; returns 1. Else 0.
pub fn proc_39<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    let t = mon_of(
        &mut Lend::hv(env, vec![&mut *q, &mut *scratch]),
        w,
        rec[4] as u32,
    );
    if fainted(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), t) {
        return 0;
    }
    if env.call_l(
        0x021D_6D54,
        &mut [Arg::V(t)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) != 0
    {
        return 0;
    }
    if env.call_l(
        0x021D_59B4,
        &mut [Arg::V(t), Arg::V(0x13)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    ) & 0xFF
        == rec[5] as u32
    {
        return 0;
    }
    if bit23(rec) {
        bracket_open(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), t);
    }
    env.call_l(
        0x021D_66D8,
        &mut [Arg::V(t), Arg::V(rec[5] as u32)],
        Lent::of(None, Some(w), vec![&mut *q, &mut *scratch]),
    );
    put_cmd(
        &mut Lend::new_ref(env, w),
        q,
        0x4F,
        &[rec[4] as u32, rec[5] as u32],
        scratch,
    );
    message(
        &mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]),
        str_at(rec, 8),
    );
    if bit23(rec) {
        bracket_close(&mut Lend::whr(env, w, vec![&mut *q, &mut *scratch]), t);
    }
    1
}

/// C: `FUN_overlay_d_93__021ca7c4`: HANDEX proc 0x3A: byte +4 of the record `d` behind
/// `work + 0x4A8` := `rec[4]`; returns 1.
pub fn proc_3a(rec: &[u8], d: &mut [u8]) -> u32 {
    d[4] = rec[4];
    1
}

/// C: `FUN_overlay_d_93__021ca7d4`: HANDEX proc 0x3B: sets bit 0 of byte +5 of the record `d`
/// behind `work + 0x4A8`; returns 1 (the record is not read).
pub fn proc_3b(d: &mut [u8]) -> u32 {
    if d[5] & 1 == 0 {
        d[5] = (d[5] & !1) | 1;
    }
    1
}

/// C: `FUN_overlay_d_93__021c99a0`: shows the message of the str record `s` (hem layout):
/// kind 1 -> core5 `5dac(work, id, count, args)`, kind 2 -> `5efc(...)`, or with flag bit 15
/// kind 1 -> `5e48(work, id, (u16) extra, count, args)`, kind 2 -> `5f98(...)` (`args` = the 8
/// words at +4, `extra` = the word at +0x20). Returns 1, or 0 for any other kind.
pub fn message<E: FlowEnv>(env: &mut E, s: &[u8]) -> u32 {
    let h = rd16(s, 2);
    let id = rd16(s, 0);
    let n = (h & 0x7FFF) >> 8;
    let args = &s[4..0x24];
    let (with_extra, plain) = match h & 0xFF {
        1 => (0x021C_5E48, 0x021C_5DAC),
        2 => (0x021C_5F98, 0x021C_5EFC),
        _ => return 0,
    };
    if h & 0x8000 == 0 {
        env.call(plain, &mut [Arg::P, Arg::V(id), Arg::V(n), Arg::In(args)]);
    } else {
        let extra = rd32(s, 0x20) & 0xFFFF;
        env.call(
            with_extra,
            &mut [Arg::P, Arg::V(id), Arg::V(extra), Arg::V(n), Arg::In(args)],
        );
    }
    1
}

/// C: `FUN_overlay_d_93__021ca7f4`: not a HANDEX proc (core2 calls it): `(work, a, b, rec, s)`
/// with `rec` a record whose word +0x10 bit 0 selects the extra id (0x1F when set, else events
/// `cb214(work, a, rec)`). The veneer `a8c8(s)`, `work[0x77F] = 6`, then by the battle rule
/// `b8588(work[4])`: 1 -> events `caa58`, 2 -> `cacf4`, else `ca8f8` (work, a, b, rec, extra, s,
/// and the stack words that hold a / b). A result of 1 sets `work[0x77F]` from the veneer
/// `a8e8(work + 0x1AB8, 5440(m))` when `m = a8d8(s, 0)` is nonzero, and becomes 0 when events
/// `cafd8(work, rule, a, rec, b, s, a, b)` is 0. Returns the result. `ra` = the record's address
/// (the callees get it as a value).
pub fn rule_dispatch<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    a: u32,
    b: u32,
    rec: &[u8],
    ra: u32,
    s: u32,
) -> u32 {
    let rule = env.callw(0x021B_8588, &mut [Arg::V(w.u32_at(4))], w);
    let mut extra = 0x1F;
    if rd32(rec, 0x10) & 1 == 0 {
        extra = env.callw(0x021C_B214, &mut [Arg::P, Arg::V(a), Arg::V(ra)], w);
    }
    env.callw(0x021C_A8C8, &mut [Arg::V(s)], w);
    w.set_u8_at(0x77F, 6);
    let mut r = match rule {
        1 => env.callw(
            0x021C_AA58,
            &mut [
                Arg::P,
                Arg::V(a),
                Arg::V(b),
                Arg::V(ra),
                Arg::V(extra),
                Arg::V(s),
                Arg::V(a),
            ],
            w,
        ),
        2 => env.callw(
            0x021C_ACF4,
            &mut [
                Arg::P,
                Arg::V(a),
                Arg::V(b),
                Arg::V(ra),
                Arg::V(extra),
                Arg::V(s),
                Arg::V(a),
                Arg::V(b),
            ],
            w,
        ),
        _ => env.callw(
            0x021C_A8F8,
            &mut [
                Arg::P,
                Arg::V(a),
                Arg::V(b),
                Arg::V(ra),
                Arg::V(extra),
                Arg::V(s),
            ],
            w,
        ),
    };
    if r == 1 {
        let m = env.callw(0x021C_A8D8, &mut [Arg::V(s), Arg::V(0)], w);
        if m != 0 {
            let id = env.callw(0x021D_5440, &mut [Arg::V(m)], w);
            let z = env.callw(
                0x021C_A8E8,
                &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(id)],
                w,
            );
            w.set_u8_at(0x77F, z as u8);
        }
        if env.callw(
            0x021C_AFD8,
            &mut [
                Arg::P,
                Arg::V(rule),
                Arg::V(a),
                Arg::V(ra),
                Arg::V(b),
                Arg::V(s),
                Arg::V(a),
                Arg::V(b),
            ],
            w,
        ) == 0
        {
            r = 0;
        }
    }
    r
}

/// Addresses of ported functions; stubs at these addresses become inactive.
pub const PORTED: &[u32] = &[
    0x021C8918, 0x021C8C6C, 0x021C8CBC, 0x021C8D0C, 0x021C8D2C, 0x021C8DE4, 0x021C8E58, 0x021C8F24,
    0x021C8FA4, 0x021C9010, 0x021C9068, 0x021C9178, 0x021C9250, 0x021C9394, 0x021C9448, 0x021C947C,
    0x021C94C8, 0x021C95D0, 0x021C962C, 0x021C96D4, 0x021C9728, 0x021C9750, 0x021C9778, 0x021C97A4,
    0x021C97D0, 0x021C9828, 0x021C9898, 0x021C98C4, 0x021C98E4, 0x021C99A0, 0x021C9A24, 0x021C9A60,
    0x021C9BB8, 0x021C9CC0, 0x021C9DCC, 0x021C9DE8, 0x021C9E3C, 0x021C9ECC, 0x021C9F0C, 0x021C9F4C,
    0x021C9F6C, 0x021CA128, 0x021CA158, 0x021CA1D0, 0x021CA248, 0x021CA270, 0x021CA2E8, 0x021CA318,
    0x021CA370, 0x021CA394, 0x021CA3AC, 0x021CA3D0, 0x021CA47C, 0x021CA570, 0x021CA5CC, 0x021CA6C4,
    0x021CA6FC, 0x021CA738, 0x021CA748, 0x021CA7C4, 0x021CA7D4, 0x021CA7F4,
];

#[cfg(test)]
pub(crate) mod tests;
