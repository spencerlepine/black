//! Flow core, part 3 (0x021BF324-0x021C0F24; B10h).
//!
//! Mon-list helpers (the list object lives in unloaded code behind `bx pc` veneers), the event
//! wrappers, the ability gate (`021c001c`) and the damage-record builders and readers
//! (`021c091c` ... `021c0d90`). Specs: `tools/rom-diff/svflow_specs/core3.spec` (39 functions; the
//! 50 `bx pc` veneers of the range are listed at the top of the spec).
//!
//! B10h-spec ports (smoke ports that prove the spec grammar; the rest is for the B10h port job).
//! Callees outside this area go through [`FlowEnv`]; `cmd_queue` ones are called directly (they are
//! ported and run as real code).

use super::cmd_queue;
use super::core1;
use super::core2::{self, Cx};
use super::env::{Arg, FlowEnv};
use super::events;
use super::SvFlowWork;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021B_F324,
    0x021C_08CC,
    0x021C_0CB4,
    0x021C_0D70,
    0x021C_0C34,
    0x021C_0780,
    0x021C_0898,
    0x021B_FBD0,
    0x021C_02CC,
    0x021C_0274,
    0x021B_F9A4,
    0x021B_FA00,
    0x021C_03D8,
    0x021C_03AC,
    0x021C_0404,
    0x021C_0C38,
    0x021C_0CE8,
    0x021C_0520,
    0x021B_FE60,
    0x021C_031C,
    0x021C_07AC,
    0x021C_07D4,
    0x021C_024C,
    0x021B_FA60,
    0x021B_FEAC,
    0x021C_0450,
    0x021B_FD24,
    0x021B_F83C,
    0x021B_FF60,
    0x021B_FC08,
    0x021C_0A4C,
    0x021B_F3D8,
    0x021C_0D90,
    0x021B_F8D4,
    0x021C_001C,
    0x021C_091C,
    0x021B_F6E8,
    0x021B_F4AC,
    0x021B_F78C,
];

/// `work + 0x4`: the main module (address only).
const OFS_MAIN: usize = 0x4;
/// `work + 0x8`: the party container (address only).
const OFS_POKECON: usize = 0x8;

/// Entries of a damage record (`rec + 2`, u8).
fn record_count(rec: &[u8]) -> usize {
    rec[2] as usize
}

/// `FUN_overlay_d_93__021d5440`: the id (0..5) of a BattleMon.
fn mon_id<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// C: `FUN_overlay_d_93__021bf324`: 1 when either mon is hit-immune through the helper-struct
/// predicates `021bf3b8` (mon `a`) / `021bf3c8` (mon `b`) and its BattleMon field 0x11 is 99, or
/// `a` has flag 0x1c, or `a` has flag 0x1d and its stored id (`021d62d8`, low byte) is `b`'s id.
pub fn pair_blocked<E: FlowEnv>(env: &mut E, w: &SvFlowWork, a: u32, b: u32) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let id_a = mon_id(env, a);
    if env.call(0x021B_F3B8, &mut [Arg::P, Arg::V(main), Arg::V(id_a)]) != 0
        && env.call(0x021D_59B4, &mut [Arg::V(a), Arg::V(0x11)]) == 99
    {
        return 1;
    }
    let id_b = mon_id(env, b);
    if env.call(0x021B_F3C8, &mut [Arg::P, Arg::V(main), Arg::V(id_b)]) != 0
        && env.call(0x021D_59B4, &mut [Arg::V(b), Arg::V(0x11)]) == 99
    {
        return 1;
    }
    if env.call(0x021D_62C4, &mut [Arg::V(a), Arg::V(0x1C)]) != 0 {
        return 1;
    }
    if env.call(0x021D_62C4, &mut [Arg::V(a), Arg::V(0x1D)]) != 0 {
        let stored = env.call(0x021D_62D8, &mut [Arg::V(a), Arg::V(0x1D)]) & 0xFF;
        if stored == mon_id(env, b) {
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021c08cc`: unless the main module says no (`021b911c(main, 5)`), applies
/// `(mon, a, b)` to the BattleMon (`021d5608`), puts command 4 (mon id, `a`, `b`) on the command queue
/// `q` (`scratch` = the 0x40 bytes at 0x021F3B44 that `cmd_queue::put_cmd` fills), applies `021d56cc(mon,
/// a)` and puts command 6 (mon id, `a`).
pub fn apply_and_queue<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    q: &mut [u8],
    scratch: &mut [u8],
    mon: u32,
    a: u32,
    b: u32,
) {
    if env.call(0x021B_911C, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(5)]) != 0 {
        return;
    }
    let id = mon_id(env, mon);
    env.call(0x021D_5608, &mut [Arg::V(mon), Arg::V(a), Arg::V(b)]);
    cmd_queue::put_cmd(env, q, 4, &[id, a, b], scratch);
    env.call(0x021D_56CC, &mut [Arg::V(mon), Arg::V(a)]);
    cmd_queue::put_cmd(env, q, 6, &[id, a], scratch);
}

/// C: `FUN_overlay_d_93__021c0cb4`: for each entry of the damage record `rec` (count at `+2`,
/// u16 flag/id word at `+6 + 4 * i`, id in bits 0-4) stores the BattleMon pointer
/// `021b9934(pokecon, id)` at `out[4 * i]`; returns the count.
pub fn record_mons<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], out: &mut [u8]) -> u32 {
    let pokecon = w.u32_at(OFS_POKECON);
    for i in 0..record_count(rec) {
        let word = u16::from_le_bytes([rec[6 + 4 * i], rec[7 + 4 * i]]);
        let mon = env.call(
            0x021B_9934,
            &mut [Arg::V(pokecon), Arg::V((word & 0x1F) as u32)],
        );
        out[4 * i..4 * i + 4].copy_from_slice(&mon.to_le_bytes());
    }
    record_count(rec) as u32
}

/// C: `FUN_overlay_d_93__021c0d70`: sum of the u16 values (`+4 + 4 * i`) of a damage record.
pub fn record_total(rec: &[u8]) -> u32 {
    (0..record_count(rec)).fold(0u32, |sum, i| {
        sum.wrapping_add(u16::from_le_bytes([rec[4 + 4 * i], rec[5 + 4 * i]]) as u32)
    })
}

// ---- shared constants and primitives -------------------------------------------------------------

/// `work + 0x1D74`: the packed queue header word (`cmd_queue::advance_fields` / `set_word`).
const OFS_QUEUE_HEADER: usize = 0x1D74;
/// End-of-list marker of the variable-length commands (`cmd_queue::put_cmd_list`).
const LIST_END: u32 = 0xFFFF_0000;
/// `work + 0x1F78`: u32 state (2, 3 and 4 are tested).
const OFS_STATE_1F78: usize = 0x1F78;
/// `work + 0x858` / `+0x85C`: the two mon lists `021c0d90` builds records for; `+0x86C` / `+0x870`
/// are the damage records that belong to them (pointers).
const OFS_LIST_858: usize = 0x858;
const OFS_LIST_85C: usize = 0x85C;
const OFS_REC_86C: usize = 0x86C;
const OFS_REC_870: usize = 0x870;
/// `work + 0x860`: pointer handed to `021c0a1c` / `021c1a54`.
const OFS_PTR_860: usize = 0x860;
/// `work + 0x4B4`: pointer handed to `021c71e0` / `021c1194` / `021c11a4`.
const OFS_PTR_4B4: usize = 0x4B4;
/// `work + 0x1FE9`: u8 that `021c091c` clears.
const OFS_BYTE_1FE9: usize = 0x1FE9;
/// `work + 0x77F`: u8 counter.
const OFS_BYTE_77F: usize = 0x77F;

fn msg_start<E: FlowEnv>(env: &mut E, n: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(n)]);
}

fn msg_end<E: FlowEnv>(env: &mut E, n: u32) {
    env.call(0x021D_7540, &mut [Arg::V(n)]);
}

fn msg_send<E: FlowEnv>(env: &mut E, cmd: u32) {
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(cmd)]);
}

fn arg_ec<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_75EC, &mut [Arg::V(id), Arg::V(v)]);
}

fn arg_a8<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_75A8, &mut [Arg::V(id), Arg::V(v)]);
}

fn arg_630<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_7630, &mut [Arg::V(id), Arg::V(v)]);
}

fn read_back<E: FlowEnv>(env: &mut E, id: u32) -> u32 {
    env.call(0x021D_775C, &mut [Arg::V(id)])
}

/// `FUN_overlay_d_93__021d62c4`: BattleMon flag test.
fn mon_flag<E: FlowEnv>(env: &mut E, mon: u32, flag: u32) -> u32 {
    env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(flag)])
}

/// `FUN_overlay_d_93__021d62a8`: the state (0..3) of a BattleMon.
fn mon_state<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_62A8, &mut [Arg::V(mon)])
}

/// `FUN_0201beb8`: flag `bit` of the move data record of move `id` (`black-pokemon::waza`; B10-wire:
/// the callers pass move IDs, not item / ability IDs).
fn id_has_flag<E: FlowEnv>(env: &mut E, id: u32, bit: u32) -> u32 {
    env.call(0x0201_BEB8, &mut [Arg::V(id), Arg::V(bit)])
}

fn u16_at(b: &[u8], ofs: usize) -> u16 {
    u16::from_le_bytes([b[ofs], b[ofs + 1]])
}

fn set_u16(b: &mut [u8], ofs: usize, v: u16) {
    b[ofs..ofs + 2].copy_from_slice(&v.to_le_bytes());
}

fn set_u32(b: &mut [u8], ofs: usize, v: u32) {
    b[ofs..ofs + 4].copy_from_slice(&v.to_le_bytes());
}

/// The queue header word of the work (`work + 0x1D74`).
fn header(w: &mut SvFlowWork) -> &mut [u8] {
    &mut w.mem[OFS_QUEUE_HEADER..]
}

/// The move being used, as most functions of this area see it: its record (`u16 +0` = move id),
/// the attacker, the list of targets (an object in unloaded code) and the 24-word table
/// (`events::table_*`).
#[derive(Clone, Copy)]
pub struct Move<'a> {
    pub rec: &'a [u8],
    pub mon: u32,
    pub list: u32,
    pub table: &'a [u8],
}

// ---- small leaves -------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c0c34`: the entry count (byte at `+2`) of a damage record.
pub fn record_len(rec: &[u8]) -> u32 {
    rec[2] as u32
}

/// C: `FUN_overlay_d_93__021c0780`: puts command 0x25 (mon id, `a & 0xff`, `b`) and applies the
/// change (`021d69a0`).
pub fn put_cmd_25<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, a: u32, b: u32) {
    let id = mon_id(env, mon);
    cmd_queue::put_cmd(env, cx.q, 0x25, &[id, a & 0xFF, b], cx.scratch);
    env.call(0x021D_69A0, &mut [Arg::V(mon), Arg::V(a), Arg::V(b)]);
}

/// C: `FUN_overlay_d_93__021c0898`: `n = min(n, 021d5584(mon, a) & 0xff)`; when `n != 0` applies
/// `021c08cc(mon, a, n)` and returns 1.
pub fn apply_up_to<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    a: u32,
    n: u32,
) -> u32 {
    let limit = env.call(0x021D_5584, &mut [Arg::V(mon), Arg::V(a)]) & 0xFF;
    let n = n.min(limit);
    if n == 0 {
        return 0;
    }
    apply_and_queue(env, w, cx.q, cx.scratch, mon, a, n);
    1
}

/// C: `FUN_overlay_d_93__021bfbd0`: rewrites the queued command 0x30 at position `pos` from the
/// record `rec` (`u16 +0` or `id_fallback`, `u8 +2 +3 +4`), then clears `rec[4]` and sets bit 1 of
/// `rec[5]`.
pub fn rewrite_cmd_30<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    id_fallback: u32,
    rec: &mut [u8],
    pos: u32,
) {
    let mut id = u16_at(rec, 0) as u32;
    if id == 0 {
        id = id_fallback;
    }
    let args = [rec[2] as u32, rec[3] as u32, id, rec[4] as u32];
    cmd_queue::put_cmd_at(env, cx.q, pos & 0xFFFF, 0x30, &args, cx.scratch);
    rec[4] = 0;
    rec[5] |= 2;
}

/// C: `FUN_overlay_d_93__021c02cc`: message `kind`: argument 0x12 = `id`, 2 = mon id, reply slot
/// 0x22; returns the reply.
pub fn build_02cc<E: FlowEnv>(env: &mut E, mon: u32, id: u32, kind: u32) -> u32 {
    msg_start(env, 0x15BB);
    arg_ec(env, 0x12, id);
    let m = mon_id(env, mon);
    arg_ec(env, 2, m);
    arg_630(env, 0x22, 0);
    msg_send(env, kind);
    let r = read_back(env, 0x22);
    msg_end(env, 0x15BB + 6);
    r
}

/// C: `FUN_overlay_d_93__021c0274`: message 0x1D: mon id (2), `b` (0x22), `a` (0x12), reply slot
/// 0x41; returns the reply.
pub fn build_0274<E: FlowEnv>(env: &mut E, mon: u32, a: u32, b: u32) -> u32 {
    msg_start(env, 0x15A1);
    let m = mon_id(env, mon);
    arg_ec(env, 2, m);
    arg_ec(env, 0x22, b);
    arg_ec(env, 0x12, a);
    arg_630(env, 0x41, 0);
    msg_send(env, 0x1D);
    let r = read_back(env, 0x41);
    msg_end(env, 0x15A1 + 7);
    r
}

/// C: `FUN_overlay_d_93__021bf9a4`: message 0x1C: the ids of two mons (3, 4), the u16 at `rec + 0`
/// (0x12), reply slot 0x51; returns the reply.
pub fn build_f9a4<E: FlowEnv>(env: &mut E, a: u32, b: u32, rec: &[u8]) -> u32 {
    msg_start(env, 0x1341);
    let ia = mon_id(env, a);
    arg_ec(env, 3, ia);
    let ib = mon_id(env, b);
    arg_ec(env, 4, ib);
    arg_ec(env, 0x12, u16_at(rec, 0) as u32);
    arg_630(env, 0x51, 0);
    msg_send(env, 0x1C);
    let r = read_back(env, 0x51);
    msg_end(env, 0x1341 + 7);
    r
}

/// C: `FUN_overlay_d_93__021bfa00`: in rule 2 (triple; `021b8588` of the main module): 0 when the
/// move `item` (nonzero) has move flag 0xb; else 1 when `021b9a6c` of the two mons' positions
/// (`021b8d8c`) is 0 (out of reach). 0 under any other rule.
pub fn same_side_gate<E: FlowEnv>(env: &mut E, w: &SvFlowWork, a: u32, b: u32, item: u32) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    if env.call(0x021B_8588, &mut [Arg::V(main)]) != 2 {
        return 0;
    }
    if item != 0 && id_has_flag(env, item, 0xB) != 0 {
        return 0;
    }
    let pokecon = w.u32_at(OFS_POKECON);
    let ia = mon_id(env, a);
    let ca = env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(ia)],
    );
    let ib = mon_id(env, b);
    let cb = env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(ib)],
    );
    u32::from(env.call(0x021B_9A6C, &mut [Arg::V(ca), Arg::V(cb)]) == 0)
}

/// C: `FUN_overlay_d_93__021c03d8`: in state 2 (`021d62a8`), for a mon with `021d6130`, tells the
/// client (`021c557c(work, mon, state, 1)`).
pub fn state2_notify<E: FlowEnv>(env: &mut E, mon: u32) {
    let s = mon_state(env, mon);
    if s == 2 && env.call(0x021D_6130, &mut [Arg::V(mon)]) != 0 {
        env.call(
            0x021C_557C,
            &mut [Arg::P, Arg::V(mon), Arg::V(s), Arg::V(1)],
        );
    }
}

/// C: `FUN_overlay_d_93__021c03ac`: flag 7 of the mon (then `021c54fc`) and a 50 % roll.
pub fn flag7_roll<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    if mon_flag(env, mon, 7) != 0 {
        env.call(0x021C_54FC, &mut [Arg::P, Arg::V(mon)]);
        if env.call(0x021D_7AF0, &mut [Arg::V(0x32)]) != 0 {
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021c0404`: in state 3: 1 when the id has flag 0xa, else a 20 % roll that
/// (on success) tells the client (`021c557c(work, mon, state, 1)`) and still returns 0.
pub fn state3_roll<E: FlowEnv>(env: &mut E, mon: u32, id: u32) -> u32 {
    let state = mon_state(env, mon);
    let mut hit = 0;
    let mut roll = false;
    if state == 3 {
        if id_has_flag(env, id, 0xA) != 0 {
            hit = 1;
        } else if env.call(0x021D_7AF0, &mut [Arg::V(0x14)]) != 0 {
            roll = true;
        }
    }
    if roll {
        env.call(
            0x021C_557C,
            &mut [Arg::P, Arg::V(mon), Arg::V(state), Arg::V(1)],
        );
    }
    hit
}

// ---- damage record readers ----------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c0c38`: copies the entries of the damage record `rec` whose bit 15 is
/// set to the output arrays (mon pointer via `021b9934`, u16 value, table value (bits 5-8), flag
/// (bit 13)); returns how many.
pub fn record_copy_hit<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    mons: &mut [u8],
    values: &mut [u8],
    tables: &mut [u8],
    flags: &mut [u8],
) -> u32 {
    let pokecon = w.u32_at(OFS_POKECON);
    let mut n = 0usize;
    for i in 0..rec[2] as usize {
        let word = u16_at(rec, 6 + 4 * i);
        if word & 0x8000 == 0 {
            continue;
        }
        let mon = env.call(
            0x021B_9934,
            &mut [Arg::V(pokecon), Arg::V((word & 0x1F) as u32)],
        );
        set_u32(mons, 4 * n, mon);
        set_u16(values, 2 * n, u16_at(rec, 4 + 4 * i));
        set_u32(tables, 4 * n, (word as u32 >> 5) & 0xF);
        flags[n] = ((word >> 13) & 1) as u8;
        n += 1;
    }
    n as u32
}

/// Outputs of [`record_copy_miss`] besides the mon pointers.
pub struct MissOut<'a> {
    pub values: &'a mut [u8],
    pub tables: &'a mut [u8],
    pub flags: &'a mut [u8],
    pub rolls: &'a mut [u8],
}

/// C: `FUN_overlay_d_93__021c0ce8`: like [`record_copy_hit`] for the entries whose bit 15 is clear,
/// with one more output byte array (the roll, bits 9-12); returns how many.
pub fn record_copy_miss<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    mons: &mut [u8],
    out: &mut MissOut,
) -> u32 {
    let pokecon = w.u32_at(OFS_POKECON);
    let mut n = 0usize;
    for i in 0..rec[2] as usize {
        let word = u16_at(rec, 6 + 4 * i);
        if word & 0x8000 != 0 {
            continue;
        }
        let mon = env.call(
            0x021B_9934,
            &mut [Arg::V(pokecon), Arg::V((word & 0x1F) as u32)],
        );
        set_u32(mons, 4 * n, mon);
        set_u16(out.values, 2 * n, u16_at(rec, 4 + 4 * i));
        set_u32(out.tables, 4 * n, (word as u32 >> 5) & 0xF);
        out.flags[n] = ((word >> 13) & 1) as u8;
        out.rolls[n] = ((word >> 9) & 0xF) as u8;
        n += 1;
    }
    n as u32
}

// ---- failure messages -----------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c0520`: the failure-message table: case `case` (0..0x19; others the
/// default 0x5a / 0x47) puts a 0x5a / 0x5b text message for the mon; `r6` is an extra value some
/// cases append.
pub fn failure_message<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, r6: u32, case: u32) {
    let id = mon_id(env, mon);
    // (command, message, append r6)
    let (cmd, msg, with_r6) = match case {
        2 => (0x5B, 0x135, false),
        3 => (0x5B, 0x114, false),
        4 => (0x5B, 0x123, false),
        6 => (0x5B, 363, false),
        7 => (0x5B, 366, false),
        8 => (0x5B, 0x150, false),
        9 => (0x5B, 0x253, true),
        10 => (0x5B, 0x23B, true),
        11 => (0x5B, 0x244, true),
        12 => (0x5B, 0x24D, true),
        13 => (0x5B, 0x37A, true),
        14 => (0x5B, 0x37D, false),
        18 => (0x5B, 445, false),
        20 => (0x5B, 0x43E, true),
        23 => (0x5A, 0xC1, false),
        1 => {
            env.call(0x021C_5604, &mut [Arg::P, Arg::V(mon), Arg::V(r6)]);
            (0x5A, 0x52, false)
        }
        15 => {
            cmd_queue::put_cmd(env, cx.q, 0x57, &[id], cx.scratch);
            cmd_queue::put_cmd_list(env, cx.q, 0x5B, 451, &[id, LIST_END]);
            cmd_queue::put_cmd(env, cx.q, 0x58, &[id], cx.scratch);
            return;
        }
        19 | 25 => return,
        22 => {
            if mon_flag(env, mon, 2) != 0 {
                (0x5A, 0xC2, false)
            } else {
                let r = env.call(0x021D_78AC, &mut [Arg::V(4)]) & 0xFF;
                (0x5A, 0xBC + r, false)
            }
        }
        _ => (0x5A, 0x47, false),
    };
    if with_r6 {
        cmd_queue::put_cmd_list(env, cx.q, cmd, msg, &[id, r6, LIST_END]);
    } else {
        cmd_queue::put_cmd_list(env, cx.q, cmd, msg, &[id, LIST_END]);
    }
}

/// C: `FUN_overlay_d_93__021c0450`: prints the failure reason `reason` of using `id` on `mon`:
/// the message table (`021c0520`) plus the extra client calls some reasons need, then a queue
/// bracket around `021c6564`. Reason 5 (party-wide) returns early when `021c4f64` and the
/// team check (`021bc2b0`) both say so.
pub fn failure_print<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    id: u32,
    reason: u32,
) {
    if reason == 5 {
        env.call(0x021C_63D0, &mut [Arg::P, Arg::V(mon)]);
        if env.call(0x021C_4F64, &mut [Arg::P, Arg::V(mon)]) != 0
            && core1::some_team_unusable(env, w) != 0
        {
            return;
        }
    } else {
        failure_message(env, cx, mon, id, reason);
        match reason {
            2 => {
                env.call(0x021C_5480, &mut [Arg::P, Arg::V(mon), Arg::V(0x256)]);
            }
            3 => {
                env.call(0x021C_5480, &mut [Arg::P, Arg::V(mon), Arg::V(0x25A)]);
            }
            4 => {
                env.call(0x021C_5480, &mut [Arg::P, Arg::V(mon), Arg::V(0x259)]);
            }
            0x17 => {
                let sleep = env.call(0x021D_7D18, &mut [Arg::V(2)]);
                let m = mon_id(env, mon);
                cmd_queue::put_cmd(env, cx.q, 0x31, &[m, 0], cx.scratch);
                env.call(
                    0x021C_5C1C,
                    &mut [Arg::P, Arg::V(mon), Arg::V(2), Arg::V(sleep)],
                );
                env.call(0x021C_5480, &mut [Arg::P, Arg::V(mon), Arg::V(0x256)]);
            }
            _ => {}
        }
    }
    let old = cmd_queue::advance_fields(header(w));
    env.call(
        0x021C_6564,
        &mut [Arg::V(cx.w_addr), Arg::V(mon), Arg::V(id), Arg::V(reason)],
    );
    cmd_queue::set_word(header(w), old);
}

/// C: `FUN_overlay_d_93__021c024c`: for ids other than 0xa5, when the message builder
/// `021c02cc(mon, id, 0x1f)` answers nonzero prints that failure reason and returns 1.
pub fn failure_check<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    id: u32,
) -> u32 {
    if id == 0xA5 {
        return 0;
    }
    let r = build_02cc(env, mon, id, 0x1F);
    if r == 0 {
        return 0;
    }
    failure_print(env, w, cx, mon, id, r);
    1
}

/// C: `FUN_overlay_d_93__021bfe60`: clears flag 0x1a (with `021c5620` / `021d5fc0(mon, 0xb)`), then
/// runs `021c6110` until the mon's state (`021d5c08`) is 0x10.
pub fn clear_flag_1a<E: FlowEnv>(env: &mut E, mon: u32) {
    if mon_flag(env, mon, 0x1A) != 0 {
        env.call(
            0x021C_5620,
            &mut [Arg::P, Arg::V(mon), Arg::V(0x1A), Arg::V(0)],
        );
        env.call(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(0xB)]);
    }
    loop {
        let s = env.call(0x021D_5C08, &mut [Arg::V(mon)]);
        if s == 0x10 {
            return;
        }
        env.call(0x021C_6110, &mut [Arg::P, Arg::V(mon), Arg::V(s)]);
    }
}

/// C: `FUN_overlay_d_93__021c031c`: in state 3 (`work + 0x1F78`) puts message 0x5a / 0xbc and
/// returns 1; else for a mon with flag 6: when `021d61a8` clears it (`021c5620`) with message
/// 0x5b / 351 and returns 0, otherwise `021c54c8` and a 50 % roll (its raw result).
pub fn flag6_check<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, mon: u32) -> u32 {
    if w.u32_at(OFS_STATE_1F78) == 3 {
        let id = mon_id(env, mon);
        cmd_queue::put_cmd_list(env, cx.q, 0x5A, 0xBC, &[id, LIST_END]);
        return 1;
    }
    if mon_flag(env, mon, 6) == 0 {
        return 0;
    }
    if env.call(0x021D_61A8, &mut [Arg::V(mon)]) != 0 {
        env.call(
            0x021C_5620,
            &mut [Arg::P, Arg::V(mon), Arg::V(6), Arg::V(0)],
        );
        let id = mon_id(env, mon);
        cmd_queue::put_cmd_list(env, cx.q, 0x5B, 351, &[id, LIST_END]);
        return 0;
    }
    env.call(0x021C_54C8, &mut [Arg::P, Arg::V(mon)]);
    env.call(0x021D_7AF0, &mut [Arg::V(0x32)])
}

// ---- message builders with lists -------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c07d4`: message 0x4e: the count of the list (`021c0868`) at key 5, the
/// ids of its mons at keys 6.. (`021c0878` / `021c0888`), the mon id (3), `r3` (0x12), `r2`
/// (0x17), key 0x20 = 1; returns the reply of key 0x20.
pub fn build_07d4<E: FlowEnv>(env: &mut E, mon: u32, r2: u32, r3: u32, list: u32) -> u32 {
    msg_start(env, 0x16EF);
    let n = env.call(0x021C_0868, &mut [Arg::V(list)]);
    arg_ec(env, 5, n);
    let mut i = 0;
    let mut m = env.call(0x021C_0878, &mut [Arg::V(list), Arg::V(0)]);
    while m != 0 {
        let id = mon_id(env, m);
        arg_ec(env, i + 6, id);
        i += 1;
        m = env.call(0x021C_0888, &mut [Arg::V(list), Arg::V(i)]);
    }
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x12, r3);
    arg_ec(env, 0x17, r2);
    arg_a8(env, 0x20, 1);
    msg_send(env, 0x4E);
    let r = read_back(env, 0x20);
    msg_end(env, 0x1704);
    r
}

/// C: `FUN_overlay_d_93__021c07ac`: `021c07d4(mon, r3, r2, list)` then `021c0898(mon, r3, its
/// low byte)`.
pub fn apply_listed<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    r2: u32,
    r3: u32,
    list: u32,
) -> u32 {
    let n = build_07d4(env, mon, r3, r2, list) & 0xFF;
    apply_up_to(env, w, cx, mon, r3, n)
}

// ---- hit checks ----------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bfa60`: can the attack `rec` of mon `a` hit mon `b`: 1 early when
/// `021bf324` / `021c69b8` / flag 0x20 say so, else the accuracy builder (message 0x33: stages at
/// keys 0x27 / 0x28, key 0x4b, result 0x35) and a percent roll.
pub fn can_hit<E: FlowEnv>(env: &mut E, w: &SvFlowWork, a: u32, b: u32, rec: &[u8]) -> u32 {
    if pair_blocked(env, w, a, b) != 0 {
        return 1;
    }
    let move_id = u16_at(rec, 0) as u32;
    if env.call(
        0x021C_69B8,
        &mut [Arg::P, Arg::V(a), Arg::V(b), Arg::V(move_id)],
    ) != 0
    {
        return 1;
    }
    if mon_flag(env, b, 0x20) != 0 {
        return 1;
    }
    let acc = env.call(0x021C_6A04, &mut [Arg::P, Arg::V(a), Arg::V(b), Arg::P]);
    msg_start(env, 0x1396);
    let ia = mon_id(env, a);
    arg_ec(env, 3, ia);
    let ib = mon_id(env, b);
    arg_ec(env, 4, ib);
    arg_630(env, 0x4B, 0);
    let sa = env.call(0x021D_59B4, &mut [Arg::V(a), Arg::V(6)]);
    arg_a8(env, 0x27, sa);
    let sb = env.call(0x021D_59B4, &mut [Arg::V(b), Arg::V(7)]);
    arg_a8(env, 0x28, sb);
    env.call(
        0x021D_7674,
        &mut [Arg::V(0x35), Arg::V(0x1000), Arg::V(0x29), Arg::V(0x2_0000)],
    );
    msg_send(env, 0x33);
    let stage_a = read_back(env, 0x27) as u8 as i8 as i32;
    let mut stage_b = read_back(env, 0x28) as u8 as i8 as i32;
    let ignore = read_back(env, 0x4B) & 0xFF;
    if (mon_flag(env, b, 0x11) != 0 && stage_b > 6) || ignore != 0 {
        stage_b = 6;
    }
    if env.call(0x021D_5BB0, &mut [Arg::V(a), Arg::V(0xE)]) != 0 {
        env.call(0x021D_7704, &mut [Arg::V(0x35), Arg::V(0x1333)]);
    }
    let base = read_back(env, 0x35);
    msg_end(env, 0x13AF);
    let idx = (stage_a + 6 - stage_b).clamp(0, 12) as u32;
    let t = env.call(0x021D_7908, &mut [Arg::V(acc & 0xFF), Arg::V(idx & 0xFF)]);
    let mut pct = env.call(0x021D_7B10, &mut [Arg::V(t), Arg::V(base)]);
    if pct > 100 {
        pct = 100;
    }
    if env.call(0x021B_911C, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(6)]) != 0 {
        pct = 100;
    }
    let roll = env.call(0x021D_78AC, &mut [Arg::V(100)]);
    u32::from(roll < (pct & 0xFF))
}

/// C: `FUN_overlay_d_93__021bf8d4`: unless the list holds exactly one mon and it is `mon`, for every
/// mon of the list that fails both `021bf9a4` and `can_hit`, removes it from the list and
/// applies `021c572c(work, mon, move id)`.
pub fn filter_list_hits<E: FlowEnv>(env: &mut E, w: &SvFlowWork, rec: &[u8], mon: u32, list: u32) {
    if env.call(0x021B_F944, &mut [Arg::V(list)]) == 1
        && env.call(0x021B_F954, &mut [Arg::V(list), Arg::V(0)]) == mon
    {
        return;
    }
    env.call(0x021B_F964, &mut [Arg::V(list)]);
    let mut m = env.call(0x021B_F974, &mut [Arg::V(list)]);
    while m != 0 {
        if build_f9a4(env, mon, m, rec) == 0 && can_hit(env, w, mon, m, rec) == 0 {
            env.call(0x021B_F984, &mut [Arg::V(list), Arg::V(m)]);
            env.call(0x021C_572C, &mut [Arg::P, Arg::V(m)]);
        }
        m = env.call(0x021B_F994, &mut [Arg::V(list)]);
    }
}

/// C: `FUN_overlay_d_93__021bf83c`: one pass over the list: for every mon not excluded by
/// `021bf324` that passes `021c6948`, removes it from the list and applies `021c572c`.
pub fn filter_list_pass<E: FlowEnv>(env: &mut E, w: &SvFlowWork, mon: u32, list: u32) {
    env.call(0x021B_F894, &mut [Arg::V(list)]);
    let mut m = env.call(0x021B_F8A4, &mut [Arg::V(list)]);
    while m != 0 {
        if pair_blocked(env, w, mon, m) == 0
            && env.call(0x021C_6948, &mut [Arg::P, Arg::V(mon), Arg::V(m)]) != 0
        {
            env.call(0x021B_F8B4, &mut [Arg::V(list), Arg::V(m)]);
            env.call(0x021C_572C, &mut [Arg::P, Arg::V(m)]);
        }
        m = env.call(0x021B_F8C4, &mut [Arg::V(list)]);
    }
}

// ---- per-mon effects -------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bfeac`: `*out = 0`; for a mon `b` that is not fainted, locked or in
/// state flag (`021d5b68`, `021d6d7c`, `021d5c30`): without flag 7 does the swap bookkeeping
/// (`021c60ec`, command 0x25, `021c2910`) and returns 1; with flag 7 puts message 0x5b / 0x20b and
/// sets `*out = 1`. Returns 0 otherwise.
pub fn swap_effect<E: FlowEnv>(env: &mut E, cx: &mut Cx, a: u32, b: u32, out: &mut [u8]) -> u32 {
    set_u32(out, 0, 0);
    if env.call(0x021D_5B68, &mut [Arg::V(b)]) != 0
        || env.call(0x021D_6D7C, &mut [Arg::V(b)]) != 0
        || env.call(0x021D_5C30, &mut [Arg::V(b)]) != 0
    {
        return 0;
    }
    if env.call(0x021D_5BB0, &mut [Arg::V(b), Arg::V(7)]) != 0 {
        let id = mon_id(env, b);
        cmd_queue::put_cmd_list(env, cx.q, 0x5B, 0x20B, &[id, LIST_END]);
        set_u32(out, 0, 1);
        return 0;
    }
    let idb = mon_id(env, b);
    env.call(0x021C_60EC, &mut [Arg::P, Arg::V(a), Arg::V(3)]);
    env.call(0x021C_60EC, &mut [Arg::P, Arg::V(b), Arg::V(3)]);
    put_cmd_25(env, cx, a, 4, idb.wrapping_add(1) & 0xFF);
    let ida = mon_id(env, a);
    let e = env.call(0x021E_899C, &mut [Arg::V(ida)]);
    env.call(
        0x021C_2910,
        &mut [
            Arg::P,
            Arg::V(b),
            Arg::V(a),
            Arg::V(0x21),
            Arg::V(e),
            Arg::V(0),
            Arg::V(0),
            Arg::V(0),
        ],
    );
    1
}

/// C: `FUN_overlay_d_93__021bff60`: end of a bind-like effect: finds the partner of `mon`
/// (`021d69ac(mon, 4) - 1 < 24`; none = return), clears its flag 0x21 (with messages), tells the
/// clients (`021c0780`) and, when `keep` is 0 and `mon` is not fainted, puts command 0x31.
pub fn release_partner<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx, mon: u32, keep: u32) {
    let t = env.call(0x021D_69AC, &mut [Arg::V(mon), Arg::V(4)]);
    let mut idx = 0x1F;
    if t != 0 {
        let k = t.wrapping_sub(1) & 0xFF;
        if k < 0x18 {
            idx = k;
        }
    }
    if idx == 0x1F {
        return;
    }
    let partner = env.call(
        0x021B_9934,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(idx)],
    );
    if mon_flag(env, partner, 0x21) != 0 {
        env.call(
            0x021C_5620,
            &mut [Arg::P, Arg::V(partner), Arg::V(0x21), Arg::V(0)],
        );
        env.call(0x021C_6110, &mut [Arg::P, Arg::V(partner), Arg::V(3)]);
        if env.call(0x021C_000C, &mut [Arg::P, Arg::V(idx)]) != 0 {
            cmd_queue::put_cmd(env, cx.q, 0x31, &[idx, 0], cx.scratch);
            env.call(0x021C_5D88, &mut [Arg::P, Arg::V(partner), Arg::V(0x465)]);
        }
    }
    put_cmd_25(env, cx, mon, 4, 0);
    if keep == 0 && env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0 {
        let id = mon_id(env, mon);
        cmd_queue::put_cmd(env, cx.q, 0x31, &[id, 0], cx.scratch);
    }
}

// ---- queue brackets -------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bfd24`: queue bracket around `021c676c(mon, list, move id,
/// &target, &miss)`: on success puts command 0x30 (`client`, the counter byte at `work + 0x77F`
/// or `021bfe50` of the list's first mon, the move id, 0), then in a second bracket `021c6848`
/// and commands 0x31 for the target; without success and no miss flag runs `021c5718`.
/// Returns the success flag.
pub fn run_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    client: u32,
    list: u32,
    move_id: u32,
) -> u32 {
    let move_id = move_id & 0xFFFF;
    let old = cmd_queue::advance_fields(header(w));
    let mut target = [0x1Fu8];
    let mut miss = [0u8; 4];
    let ok = env.call(
        0x021C_676C,
        &mut [
            Arg::P,
            Arg::V(mon),
            Arg::V(list),
            Arg::V(move_id),
            Arg::Out(&mut target),
            Arg::Out(&mut miss),
        ],
    );
    if ok != 0 {
        let mut counter = w.mem[OFS_BYTE_77F] as u32;
        if env.call(0x021B_FE30, &mut [Arg::V(list)]) != 0 {
            let first = env.call(0x021B_FE40, &mut [Arg::V(list), Arg::V(0)]);
            let id = mon_id(env, first);
            counter = env.call(0x021B_FE50, &mut [Arg::P, Arg::V(id)]);
        }
        cmd_queue::put_cmd(env, cx.q, 0x30, &[client, counter, move_id, 0], cx.scratch);
    }
    cmd_queue::set_word(header(w), old);
    if ok != 0 {
        let old = cmd_queue::advance_fields(header(w));
        env.call(0x021C_6848, &mut [Arg::P, Arg::V(mon)]);
        cmd_queue::set_word(header(w), old);
        if env.call(0x021D_5C30, &mut [Arg::V(mon)]) != 0 {
            let id = mon_id(env, mon);
            cmd_queue::put_cmd(env, cx.q, 0x31, &[id, 1], cx.scratch);
        }
        if target[0] != 0x1F {
            cmd_queue::put_cmd(env, cx.q, 0x31, &[target[0] as u32, 1], cx.scratch);
        }
    } else if miss == [0; 4] {
        env.call(0x021C_5718, &mut [Arg::P]);
    }
    ok
}

/// C: `FUN_overlay_d_93__021bfc08`: uses an ability-triggered effect of `mon` on `list` with the
/// move `move_id`; returns 1..4. `p` is the small per-turn struct at `work + 0x4A8` (byte 4 is
/// set to 1 on the common path).
pub fn use_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    mon: u32,
    list: u32,
    move_id: u32,
) -> u32 {
    if mon_flag(env, mon, 0x1A) == 0 {
        let id = mon_id(env, mon);
        let client = env.call(
            0x021B_8D8C,
            &mut [
                Arg::V(w.u32_at(OFS_MAIN)),
                Arg::V(w.u32_at(OFS_POKECON)),
                Arg::V(id),
            ],
        );
        if env.call(0x021C_66AC, &mut [Arg::P, Arg::V(mon), Arg::V(list)]) != 0 {
            env.call(0x021C_5718, &mut [Arg::P]);
            return 1;
        }
        if env.call(0x021C_6720, &mut [Arg::P, Arg::V(mon), Arg::V(move_id)]) == 0 {
            if run_effect(env, w, cx, mon, client, list, move_id) != 0 {
                let e = env.call(0x021E_896C, &mut [Arg::V(2), Arg::V(move_id)]);
                env.call(
                    0x021C_5C1C,
                    &mut [Arg::P, Arg::V(mon), Arg::V(0x1A), Arg::V(e)],
                );
            }
            return 2;
        }
        if run_effect(env, w, cx, mon, client, list, move_id) == 0 {
            return 1;
        }
        let old = cmd_queue::advance_fields(header(w));
        env.call(0x021C_6878, &mut [Arg::P, Arg::V(mon)]);
        cmd_queue::set_word(header(w), old);
    }
    p[4] = 1;
    let old = cmd_queue::advance_fields(header(w));
    let r = env.call(0x021C_68A8, &mut [Arg::P, Arg::V(mon), Arg::V(list)]);
    cmd_queue::set_word(header(w), old);
    env.call(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(0xB)]);
    clear_flag_1a(env, mon);
    if r != 0 {
        4
    } else {
        3
    }
}

// ---- the ability gate ------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c001c`: can `mon` use the ability / effect `id` (`skip` = skip the
/// move-type test). Returns 1 and prints the failure reason (`021c0450`) when it cannot; else
/// 0 (after the success message / bookkeeping of states 2 and 3).
pub fn ability_gate<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    id: u32,
    skip: u32,
) -> u32 {
    state2_notify(env, mon);
    let mut state = mon_state(env, mon);
    let mut hit = 0;
    let reason = 'r: {
        if state == 2 && build_0274(env, mon, id, 2) == 0 {
            break 'r 2;
        }
        hit = state3_roll(env, mon, id);
        state = mon_state(env, mon);
        if state == 3 && hit == 0 {
            break 'r 4;
        }
        match w.u32_at(OFS_STATE_1F78) {
            4 => break 'r 0x16,
            2 => break 'r 0x17,
            _ => {}
        }
        if skip == 0 {
            let t = env.call(0x021D_5778, &mut [Arg::V(mon), Arg::V(id)]);
            if t != 4 && env.call(0x021D_5584, &mut [Arg::V(mon), Arg::V(t)]) == 0 {
                break 'r 1;
            }
        }
        let r = build_02cc(env, mon, id, 0x1E);
        if r != 0 {
            break 'r r;
        }
        if env.call(0x021D_5BB0, &mut [Arg::V(mon), Arg::V(6)]) != 0 {
            break 'r 7;
        }
        if env.call(0x021D_5BB0, &mut [Arg::V(mon), Arg::V(4)]) != 0 {
            break 'r 6;
        }
        if mon_flag(env, mon, 0xD) != 0 && id != 0xA5 {
            let a = env.call(0x021D_6314, &mut [Arg::V(mon), Arg::V(0xD)]);
            if id == env.call(0x021E_8B80, &mut [Arg::V(a)]) {
                break 'r 9;
            }
        }
        if mon_flag(env, mon, 0xF) != 0 && id_has_flag(env, id, 0xC) != 0 {
            break 'r 0xD;
        }
        if env.call(0x021E_F948, &mut [Arg::V(2)]) != 0 && id_has_flag(env, id, 9) != 0 {
            break 'r 0x14;
        }
        if id != 0xA5 {
            if mon_flag(env, mon, 0x17) != 0 {
                let a = env.call(0x021D_67B8, &mut [Arg::V(mon)]);
                if a != 0xA5 && a != id {
                    break 'r 0x11;
                }
            }
            if mon_flag(env, mon, 0x19) != 0 && id != env.call(0x021D_67A8, &mut [Arg::V(mon)]) {
                break 'r 0x11;
            }
            if mon_flag(env, mon, 0xB) != 0 && env.call(0x0201_C314, &mut [Arg::V(id)]) == 0 {
                break 'r 0xA;
            }
            if env.call(0x021E_F948, &mut [Arg::V(3)]) != 0
                && env.call(
                    0x021E_F91C,
                    &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(mon), Arg::V(id)],
                ) != 0
            {
                break 'r 0xC;
            }
        }
        if flag6_check(env, w, cx, mon) != 0 {
            break 'r 5;
        }
        if state == 1 && env.call(0x021D_7AF0, &mut [Arg::V(0x19)]) != 0 {
            break 'r 3;
        }
        if flag7_roll(env, mon) != 0 {
            break 'r 8;
        }
        0
    };
    if reason != 0 {
        failure_print(env, w, cx, mon, id, reason);
        return 1;
    }
    if state == 2 {
        failure_message(env, cx, mon, id, 2);
        env.call(0x021C_5480, &mut [Arg::P, Arg::V(mon), Arg::V(0x256)]);
    } else if hit != 0 && state == 3 {
        env.call(
            0x021C_557C,
            &mut [Arg::P, Arg::V(mon), Arg::V(state), Arg::V(0)],
        );
        let m = mon_id(env, mon);
        cmd_queue::put_cmd_list(env, cx.q, 0x5B, 0x12F, &[m, id, LIST_END]);
    }
    0
}

// ---- damage records --------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021c0a4c`: fills the damage record `out` (`+0` hit count, `+1` miss
/// count, `+2` total, then per mon of the list `{u16 +4+4i value, u16 +6+4i bits: 0-4 mon id,
/// 5-8 table value, 9-12 roll, 13 / 14 / 15 flags}`) for the move `mv`; `flags` is the word
/// that `021c1e74` is passed on.
pub fn fill_record<E: FlowEnv>(env: &mut E, mv: &Move, flags: u32, out: &mut [u8]) {
    let mon1 = mv.mon;
    let rec_id = u16_at(mv.rec, 0) as u32;
    out[0] = 0;
    out[1] = 0;
    let mut i = 0usize;
    env.call(0x021C_0C04, &mut [Arg::V(mv.list)]);
    let mut m = env.call(0x021C_0C14, &mut [Arg::V(mv.list)]);
    while m != 0 {
        let wo = 6 + 4 * i; // flag word
        let vo = 4 + 4 * i; // value
        let id = mon_id(env, m);
        let mut word = (u16_at(out, wo) & !0x1F) | (id as u16 & 0x1F);
        let r = env.call(
            0x021C_6BF8,
            &mut [Arg::P, Arg::V(mon1), Arg::V(m), Arg::V(rec_id)],
        );
        word = (word & !(1 << 13)) | ((r as u16 & 1) << 13);
        let t = events::table_get(mv.table, (word & 0x1F) as u32);
        word = (word & !(0xF << 5)) | ((t as u16 & 0xF) << 5);
        set_u16(out, wo, word);
        let mut hi = [0u8; 2];
        let r = env.call(
            0x021C_1E74,
            &mut [
                Arg::P,
                Arg::V(mon1),
                Arg::V(m),
                Arg::P,
                Arg::V(((word >> 5) & 0xF) as u32),
                Arg::V(flags),
                Arg::V(((word >> 13) & 1) as u32),
                Arg::V(0),
                Arg::Out(&mut hi),
            ],
        );
        word = (word & !(1 << 14)) | ((r as u16 & 1) << 14);
        out[vo..vo + 2].copy_from_slice(&hi);
        if (word >> 14) & 1 != 0 {
            word &= !(1 << 13);
            word = (word & !(0xF << 5)) | (3 << 5);
        }
        let r = env.call(0x021D_6D7C, &mut [Arg::V(m)]);
        word = (word & !(1 << 15)) | ((r as u16 & 1) << 15);
        set_u16(out, wo, word);
        if word >> 15 == 0 {
            let x = env.call(
                0x021C_15D4,
                &mut [Arg::V(m), Arg::V(u16_at(out, vo) as u32)],
            ) as u16;
            let mut lo = x.to_le_bytes();
            let r = env.call(
                0x021C_1678,
                &mut [
                    Arg::P,
                    Arg::V(mon1),
                    Arg::V(m),
                    Arg::V(1),
                    Arg::Out(&mut lo),
                ],
            );
            word = (word & !(0xF << 9)) | ((r as u16 & 0xF) << 9);
            out[vo..vo + 2].copy_from_slice(&lo);
            out[0] = out[0].wrapping_add(1);
        } else {
            word &= !(0xF << 9);
            out[1] = out[1].wrapping_add(1);
        }
        set_u16(out, wo, word);
        env.call(
            0x021C_7C24,
            &mut [
                Arg::P,
                Arg::P,
                Arg::V(mon1),
                Arg::V(m),
                Arg::V(((word >> 5) & 0xF) as u32),
            ],
        );
        i += 1;
        m = env.call(0x021C_0C24, &mut [Arg::V(mv.list)]);
    }
    out[2] = i as u8;
}

/// The two damage records of `work + 0x86C` and `work + 0x870` (the regions the work points at).
pub struct Records<'a> {
    pub r1: &'a mut [u8],
    pub r2: &'a mut [u8],
}

/// C: `FUN_overlay_d_93__021c0d90`: builds the two damage records (`021c0a4c`, for the lists at
/// `work + 0x858` and `+0x85C`) for the move `mv`, puts the placeholder command 0x30 (its
/// position goes to `*pos`) and runs `021c11a4` on each list; returns the sum of the results.
/// `extra` is the stack word whose bit 0 becomes bit 1 of the flag word.
pub fn build_records<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    recs: &mut Records,
    mv: &Move,
    (pos, extra): (&mut [u8], u32),
) -> u32 {
    let list = mv.list;
    let mon = mv.mon;
    let cap = if env.call(0x021C_0EC4, &mut [Arg::V(list)]) == 1 {
        0x1000
    } else {
        0xC00
    };
    let mut sum = 0u32;
    let t = env.call(0x021C_0ED4, &mut [Arg::V(list)]);
    let r1 = u32::from(t > 1);
    // bit 1 of the flag word comes from the stack argument, bit 0 from the list; the other
    // bits are an uninitialised stack slot of the original (0 in the emulator, garbage on the
    // cartridge): this port uses 0
    let flags = (r1 & 1) | ((extra & 1) << 1);
    env.call(
        0x021C_0EE4,
        &mut [Arg::V(list), Arg::V(mon), Arg::V(w.u32_at(OFS_LIST_858))],
    );
    env.call(
        0x021C_0EF4,
        &mut [Arg::V(list), Arg::V(mon), Arg::V(w.u32_at(OFS_LIST_85C))],
    );
    let (l1, l2) = (w.u32_at(OFS_LIST_858), w.u32_at(OFS_LIST_85C));
    fill_record(env, &Move { list: l1, ..*mv }, cap, recs.r1);
    fill_record(env, &Move { list: l2, ..*mv }, cap, recs.r2);
    let p = cmd_queue::put_cmd_placeholder(env, cx.q, 0x30, cx.scratch);
    set_u32(pos, 0, p);
    // 021c11a4(work, rec, mon, list, record pointer, work[0x4b4], cap, flags)
    let run = |env: &mut E, list: u32, record: u32| {
        env.call(
            0x021C_11A4,
            &mut [
                Arg::P,
                Arg::P,
                Arg::V(mon),
                Arg::V(list),
                Arg::V(record),
                Arg::V(w.u32_at(OFS_PTR_4B4)),
                Arg::V(cap),
                Arg::V(flags),
            ],
        )
    };
    if env.call(0x021C_0F04, &mut [Arg::V(l1)]) != 0 {
        sum = sum.wrapping_add(run(env, l1, w.u32_at(OFS_REC_86C)));
        if sum != 0 && core2::mon_is_ours(env, w, mon) != 0 {
            env.call(0x021B_A1B4, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(0x4D)]);
        }
    }
    if env.call(0x021C_0F14, &mut [Arg::V(l2)]) != 0 {
        sum = sum.wrapping_add(run(env, l2, w.u32_at(OFS_REC_870)));
    }
    sum
}

/// C: `FUN_overlay_d_93__021c091c`: builds the damage record(s) for one target list: clears
/// `work + 0x1FE9`, runs the preparation calls (`021c19d8`, `021c0a1c`, `021c71e0`, `021c14e0`),
/// then (list answers `021c0a2c`) either `021c0f24` or [`build_records`], applies `021c2048`
/// and finally `021c1a54`; after `build_records` marks the per-turn struct `p`
/// (`work + 0x4A8`: bits 0 and 1 of byte 5) and rewrites the queued command 0x30
/// ([`rewrite_cmd_30`]).
pub fn build_damage<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    recs: &mut Records,
    mv: &Move,
    flags: u32,
) {
    let rec_id = u16_at(mv.rec, 0) as u32;
    let (mon, list) = (mv.mon, mv.list);
    let mut result = 0;
    let mut built = false;
    let mut pos = [0u8; 4];
    w.mem[OFS_BYTE_1FE9] = 0;
    env.call(0x021C_19D8, &mut [Arg::P, Arg::V(mon), Arg::P]);
    env.call(0x021C_0A1C, &mut [Arg::V(w.u32_at(OFS_PTR_860))]);
    env.call(
        0x021C_71E0,
        &mut [Arg::P, Arg::V(mon), Arg::V(rec_id), Arg::P],
    );
    env.call(
        0x021C_14E0,
        &mut [Arg::P, Arg::V(mon), Arg::P, Arg::V(list)],
    );
    if env.call(0x021C_0A2C, &mut [Arg::V(list)]) != 0 {
        if env.call(0x021C_1194, &mut [Arg::V(w.u32_at(OFS_PTR_4B4))]) != 0
            && env.call(0x021C_0A3C, &mut [Arg::V(list)]) == 1
        {
            result = env.call(
                0x021C_0F24,
                &mut [Arg::P, Arg::P, Arg::V(mon), Arg::V(list), Arg::P],
            );
        } else {
            result = build_records(env, w, cx, recs, mv, (&mut pos, flags));
            built = true;
        }
        env.call(
            0x021C_2048,
            &mut [Arg::P, Arg::V(mon), Arg::V(rec_id), Arg::V(result)],
        );
    }
    env.call(
        0x021C_1A54,
        &mut [
            Arg::P,
            Arg::P,
            Arg::V(mon),
            Arg::V(w.u32_at(OFS_PTR_860)),
            Arg::V(result),
            Arg::V(flags),
        ],
    );
    if built {
        p[5] |= 3;
        rewrite_cmd_30(env, cx, rec_id, p, u32::from_le_bytes(pos));
    }
}

// ---- mon list passes -------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bf3d8`: fills the 24-word `table` with 6 and sets `table[id] =
/// 021c6e3c(mon, m, <record>, 1)` for each mon `m` of the list; on result 0 the mon is removed from the
/// list and reported (`021c7c24`, and `021ba1b4(main, 0x4b)` when it is ours).
pub fn list_apply<E: FlowEnv>(env: &mut E, w: &SvFlowWork, mon: u32, list: u32, table: &mut [u8]) {
    events::table_fill_6(table);
    env.call(0x021B_F46C, &mut [Arg::V(list)]);
    let mut m = env.call(0x021B_F47C, &mut [Arg::V(list)]);
    while m != 0 {
        let r = env.call(
            0x021C_6E3C,
            &mut [Arg::P, Arg::V(mon), Arg::V(m), Arg::P, Arg::V(1)],
        );
        let id = mon_id(env, m);
        events::table_set(table, id, r);
        if r == 0 {
            env.call(0x021B_F48C, &mut [Arg::V(list), Arg::V(m)]);
            env.call(
                0x021C_7C24,
                &mut [Arg::P, Arg::P, Arg::V(mon), Arg::V(m), Arg::V(0)],
            );
            if core2::mon_is_ours(env, w, mon) != 0 {
                env.call(0x021B_A1B4, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(0x4B)]);
            }
        }
        m = env.call(0x021B_F49C, &mut [Arg::V(list)]);
    }
}

/// C: `FUN_overlay_d_93__021bf78c` (`021c8810` takes a pointer to the helper struct `hem`: it
/// zero-fills 0x24 bytes, so it is stubbed as `args=p` everywhere): message `event` for the
/// mons `a` and `b` with the move record `rec` (`u16 +0`, `u8 +6`, bit 0 of the word at `+0x10`),
/// the helper struct `hem` and `table[id of b]`; returns the reply of key 0x40 and stores the
/// reply of key 0x51 in `out`.
pub fn event_message<E: FlowEnv>(
    env: &mut E,
    rec: &[u8],
    event: u32,
    (a, b): (u32, u32),
    table: &[u8],
    hem: u32,
    out: &mut [u8],
) -> u32 {
    let idb = mon_id(env, b);
    let t = events::table_get(table, idb);
    env.call(0x021C_8810, &mut [Arg::P]);
    msg_start(env, 0x12E2);
    let ia = mon_id(env, a);
    arg_ec(env, 3, ia);
    let ib = mon_id(env, b);
    arg_ec(env, 4, ib);
    arg_ec(env, 0x12, u16_at(rec, 0) as u32);
    arg_ec(env, 0x16, rec[6] as u32);
    arg_ec(
        env,
        0x4E,
        u32::from_le_bytes(rec[0x10..0x14].try_into().unwrap()) & 1,
    );
    arg_ec(env, 0x3F, hem);
    arg_ec(env, 0x38, t);
    arg_630(env, 0x40, 0);
    arg_630(env, 0x51, 0);
    msg_send(env, event);
    let r = read_back(env, 0x40);
    let r51 = read_back(env, 0x51);
    set_u32(out, 0, r51);
    msg_end(env, 0x12E2 + 0xD);
    r
}

/// C: `FUN_overlay_d_93__021bf6e8`: runs the event `event` for the mons `a` and `b` inside a queue
/// bracket (it reaches `event_message` = `021bf78c` through `env.call`, a forced stub in the specs): on success flushes
/// the event helper (`021c99a0`, `021c8810`) or, when nothing was queued (`021c88e4` is 0 and
/// the reply of key 0x51 is 0), puts the message 0x5b / 0xd2 for `b`. Returns the result of
/// `021bf78c`. (The move record and the 24-word table are only passed through to it.)
pub fn run_event<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    (a, b): (u32, u32),
    event: u32,
) -> u32 {
    let old = cmd_queue::advance_fields(header(w));
    let mut out = [0u8; 4];
    let r = env.call(
        0x021B_F78C,
        &mut [
            Arg::P,
            Arg::P,
            Arg::V(event),
            Arg::V(a),
            Arg::V(b),
            Arg::P,
            Arg::P,
            Arg::Out(&mut out),
        ],
    );
    if r != 0 {
        if env.call(0x021C_8828, &mut [Arg::P]) != 0 {
            env.call(0x021C_99A0, &mut [Arg::P, Arg::P]);
            env.call(0x021C_8810, &mut [Arg::P]);
        } else if env.call(0x021C_88E4, &mut [Arg::P]) == 0 && out == [0; 4] {
            let id = mon_id(env, b);
            cmd_queue::put_cmd_list(env, cx.q, 0x5B, 0xD2, &[id, LIST_END]);
        }
    }
    cmd_queue::set_word(header(w), old);
    r
}

/// C: `FUN_overlay_d_93__021bf4ac`: four passes over the list that run `021bf6e8` for each mon:
/// events 0x2b (not for mons excluded by `021bf324`), 0x2c, then (when the move has flag 3)
/// the ability pass (mons with flag 7 and not `021c6668`: removed, message 0x5b / 0x20b) and 0x2d;
/// the mons it handled are removed from the list.
pub fn run_event_passes<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    mon: u32,
    list: u32,
) {
    env.call(0x021B_F5E8, &mut [Arg::V(list)]);
    let mut m = env.call(0x021B_F5F8, &mut [Arg::V(list)]);
    while m != 0 {
        if pair_blocked(env, w, mon, m) == 0 && run_event(env, w, cx, (mon, m), 0x2B) != 0 {
            env.call(0x021B_F608, &mut [Arg::V(list), Arg::V(m)]);
        }
        m = env.call(0x021B_F618, &mut [Arg::V(list)]);
    }
    env.call(0x021B_F628, &mut [Arg::V(list)]);
    let mut m = env.call(0x021B_F638, &mut [Arg::V(list)]);
    while m != 0 {
        if run_event(env, w, cx, (mon, m), 0x2C) != 0 {
            env.call(0x021B_F648, &mut [Arg::V(list), Arg::V(m)]);
        }
        m = env.call(0x021B_F658, &mut [Arg::V(list)]);
    }
    let move_id = u16_at(rec, 0) as u32;
    if id_has_flag(env, move_id, 3) != 0 {
        env.call(0x021B_F668, &mut [Arg::V(list)]);
        let mut m = env.call(0x021B_F678, &mut [Arg::V(list)]);
        while m != 0 {
            if env.call(0x021D_5BB0, &mut [Arg::V(m), Arg::V(7)]) != 0
                && env.call(0x021C_6668, &mut [Arg::P, Arg::V(mon)]) == 0
            {
                env.call(0x021B_F688, &mut [Arg::V(list), Arg::V(m)]);
                let id = mon_id(env, m);
                cmd_queue::put_cmd_list(env, cx.q, 0x5B, 0x20B, &[id, LIST_END]);
            }
            m = env.call(0x021B_F698, &mut [Arg::V(list)]);
        }
    }
    env.call(0x021B_F6A8, &mut [Arg::V(list)]);
    let mut m = env.call(0x021B_F6B8, &mut [Arg::V(list)]);
    while m != 0 {
        if run_event(env, w, cx, (mon, m), 0x2D) != 0 {
            env.call(0x021B_F6C8, &mut [Arg::V(list), Arg::V(m)]);
        }
        m = env.call(0x021B_F6D8, &mut [Arg::V(list)]);
    }
}

#[cfg(test)]
mod tests;
