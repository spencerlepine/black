//! Flow core, part 4 (0x021C0F24-0x021C4838; B10i).
//!
//! The per-target damage pipeline and its side-effect helpers (`0f24` -> `11a4` -> `11f0`, the base
//! damage `1e74`, the effect roll `236c`, the multi-hit loop `2e64`, the per-mon effect loops
//! `3438` / `3d08` / `3e0c`, the turn-end state machine `446c`). Specs:
//! `tools/rom-diff/svflow_specs/core4.spec` (102 functions; the 62 `bx pc` veneers of the range
//! and their thunks are listed at the top of the spec).
//!
//! Callees outside this area go through [`FlowEnv`]; `cmd_queue` ones are called directly (they are
//! ported and run as real code), and so are the functions of this area (the spec stubs of a
//! core4 function are inactive as soon as it is in [`PORTED`]).
//!
//! Conventions: a "message" is the builder protocol `74f8` (start, argument = work-relative slot
//! constant) / `75ec` (add argument) / `7630` (reply slot) / `70f0` (send) / `775c` (read the reply)
//! / `7540` (end); a "record bracket" reserves a record with `cc8d8` on the packed queue-header
//! word at `work + 0x1D74` and closes it with `cc95c`. A pointer argument of the original is a
//! slice here; where a stub receives the pointer value itself the caller passes the address.

use super::cmd_queue;
use super::env::{Arg, FlowEnv, Lend, PassPtr};
use super::SvFlowWork;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_1184,
    0x021C_1194,
    0x021C_1198,
    0x021C_14E0,
    0x021C_15AC,
    0x021C_15D4,
    0x021C_15E8,
    0x021C_1624,
    0x021C_1678,
    0x021C_16F8,
    0x021C_1758,
    0x021C_1788,
    0x021C_17CC,
    0x021C_1828,
    0x021C_18D4,
    0x021C_1910,
    0x021C_195C,
    0x021C_199C,
    0x021C_19D8,
    0x021C_1A10,
    0x021C_1A54,
    0x021C_1B5C,
    0x021C_1C88,
    0x021C_1CCC,
    0x021C_1CE4,
    0x021C_1D60,
    0x021C_1DCC,
    0x021C_1E14,
    0x021C_20BC,
    0x021C_2E4C,
    0x021C_1E74,
    0x021C_2048,
    0x021C_20D0,
    0x021C_2128,
    0x021C_2204,
    0x021C_2250,
    0x021C_2314,
    0x021C_2AB0,
    0x021C_2AF0,
    0x021C_2B3C,
    0x021C_2BA4,
    0x021C_2BF0,
    0x021C_2C50,
    0x021C_2CA8,
    0x021C_2D10,
    0x021C_236C,
    0x021C_2488,
    0x021C_25A4,
    0x021C_264C,
    0x021C_26A8,
    0x021C_26FC,
    0x021C_2758,
    0x021C_27A0,
    0x021C_282C,
    0x021C_2910,
    0x021C_2DC4,
    0x021C_2E64,
    0x021C_2F70,
    0x021C_3008,
    0x021C_312C,
    0x021C_3FDC,
    0x021C_232C,
    0x021C_2CD8,
    0x021C_1880,
    0x021C_0F24,
    0x021C_11A4,
    0x021C_11F0,
    0x021C_3184,
    0x021C_3274,
    0x021C_334C,
    0x021C_3370,
    0x021C_3394,
    0x021C_33E8,
    0x021C_3438,
    0x021C_36A0,
    0x021C_36E8,
    0x021C_3720,
    0x021C_376C,
    0x021C_3810,
    0x021C_39CC,
    0x021C_39F4,
    0x021C_3A4C,
    0x021C_3AA8,
    0x021C_3B68,
    0x021C_3B8C,
    0x021C_3BB8,
    0x021C_3BE0,
    0x021C_3C10,
    0x021C_3C60,
    0x021C_3CCC,
    0x021C_3D08,
    0x021C_3E0C,
    0x021C_4034,
    0x021C_4104,
    0x021C_41C0,
    0x021C_4218,
    0x021C_4268,
    0x021C_4304,
    0x021C_43F0,
    0x021C_446C,
    0x021C_4614,
    0x021C_4744,
];

/// `work + 0x1D74`: the packed queue-header word that the record brackets reserve / restore.
const OFS_QUEUE_HEADER: usize = 0x1D74;
/// `work + 0x778`: u32 counter bumped by [`counter_next`].
const OFS_COUNTER: usize = 0x778;
/// End-of-list marker of the variable-length commands (`cmd_queue::put_cmd_list`).
const LIST_END: u32 = 0xFFFF_0000;

/// What the command-writing functions need besides the work image: the command queue buffer
/// (the region `work[0xC]` points at) and the globals of the original: either the 0x40 scratch
/// bytes at 0x021F3B44 that `put_cmd` fills (`gb == 0`), or the 0x64 bytes at 0x021F3B20 of the
/// damage pipeline (`gb == 0x24`: the put_cmd scratch is the tail of it, the per-target arrays
/// live at the front). `w_addr` is the work's own address (stubbed callees that take a pointer
/// into the work see it).
pub struct Cx<'a> {
    pub q: &'a mut [u8],
    pub g: &'a mut [u8],
    pub gb: usize,
    pub w_addr: u32,
}

// ---- builder primitives ---------------------------------------------------------------------

/// `FUN_overlay_d_93__021d5440`: the id (0..5) of a BattleMon.
fn mon_id<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// `FUN_overlay_d_93__021d59b4`: BattleMon field `id`.
fn mon_get<E: FlowEnv>(env: &mut E, mon: u32, id: u32) -> u32 {
    env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(id)])
}

/// `FUN_overlay_d_93__021d74f8`: starts a message (`n` = a work-relative slot constant).
fn msg_start<E: FlowEnv>(env: &mut E, n: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(n)]);
}

/// `FUN_overlay_d_93__021d7540`: ends the message.
fn msg_end<E: FlowEnv>(env: &mut E, n: u32) {
    env.call(0x021D_7540, &mut [Arg::V(n)]);
}

/// `FUN_overlay_d_93__021d70f0`: sends command `cmd` (the work is passed through).
fn msg_send<E: FlowEnv>(env: &mut E, cmd: u32) {
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(cmd)]);
}

/// `FUN_overlay_d_93__021d75ec`: adds argument `id` = `v`.
fn arg_ec<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_75EC, &mut [Arg::V(id), Arg::V(v)]);
}

/// `FUN_overlay_d_93__021d7630`: adds the reply slot `id` = `v`.
fn arg_630<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_7630, &mut [Arg::V(id), Arg::V(v)]);
}

/// `FUN_overlay_d_93__021d775c`: reads reply slot `id` back.
fn read_back<E: FlowEnv>(env: &mut E, id: u32) -> u32 {
    env.call(0x021D_775C, &mut [Arg::V(id)])
}

/// `FUN_overlay_d_93__021d5fc0`: BattleMon state update `(mon, kind)`.
fn mon_set5fc0<E: FlowEnv>(env: &mut E, mon: u32, kind: u32) {
    env.call(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(kind)]);
}

/// The u16 move id at `+0` of a move record.
fn rec_id(rec: &[u8]) -> u32 {
    u16::from_le_bytes([rec[0], rec[1]]) as u32
}

/// Opens a record bracket (`cc8d8` on the queue-header word); returns the old word.
fn bracket_open(w: &mut SvFlowWork) -> u32 {
    cmd_queue::advance_fields(&mut w.mem[OFS_QUEUE_HEADER..])
}

/// Closes a record bracket (`cc95c`): restores the old word.
fn bracket_close(w: &mut SvFlowWork, old: u32) {
    cmd_queue::set_word(&mut w.mem[OFS_QUEUE_HEADER..], old);
}

/// Iterator veneers (`bx pc` into the unloaded list object): `begin(list)`.
fn list_begin<E: FlowEnv>(env: &mut E, addr: u32, list: u32) {
    env.call(addr, &mut [Arg::V(list)]);
}

/// Iterator veneers: `next(list)` (0 at the end).
fn list_next<E: FlowEnv>(env: &mut E, addr: u32, list: u32) -> u32 {
    env.call(addr, &mut [Arg::V(list)])
}

// ---- small getters / setters of the per-turn and target-list structs --------------------------

/// C: `FUN_overlay_d_93__021c1184`: 1 when byte 1 of the struct is 0.
pub fn byte1_is_zero(b: &[u8]) -> u32 {
    (b[1] == 0) as u32
}

/// C: `FUN_overlay_d_93__021c1194`: byte 3 of the struct.
pub fn byte3(b: &[u8]) -> u32 {
    b[3] as u32
}

/// C: `FUN_overlay_d_93__021c1198`: if byte 5 of the struct is 3, sets it to `v`.
pub fn set_byte5_if_3(b: &mut [u8], v: u32) {
    if b[5] == 3 {
        b[5] = v as u8;
    }
}

/// C: `FUN_overlay_d_93__021c2e4c`: `++*(u32 *)(work + 0x778)`, a wrap to 0 becoming 1; returns
/// the new value.
pub fn counter_next(w: &mut SvFlowWork) -> u32 {
    let mut v = w.u32_at(OFS_COUNTER).wrapping_add(1);
    if v == 0 {
        v = 1;
    }
    w.set_u32_at(OFS_COUNTER, v);
    v
}

/// C: `FUN_overlay_d_93__021c15d4`: `min(limit, BattleMon field 0xd of mon)` (unsigned).
pub fn min_with_hp<E: FlowEnv>(env: &mut E, mon: u32, limit: u32) -> u32 {
    let hp = mon_get(env, mon, 0xD);
    if hp < limit {
        hp
    } else {
        limit
    }
}

/// C: `FUN_overlay_d_93__021c15ac`: puts message 0x4F (`level < 3`) or 0x4E (`level > 3`; `level`
/// is signed) on the command queue `q` (the buffer at `work + 0xC`) through the list-terminated
/// command 0x5A; nothing for 3.
pub fn level_message<E: FlowEnv>(env: &mut E, q: &mut [u8], level: i32) {
    match level.cmp(&3) {
        std::cmp::Ordering::Less => cmd_queue::put_cmd_list(env, q, 0x5A, 0x4F, &[LIST_END]),
        std::cmp::Ordering::Greater => cmd_queue::put_cmd_list(env, q, 0x5A, 0x4E, &[LIST_END]),
        std::cmp::Ordering::Equal => {}
    }
}

// ---- messages (start / arguments / send / end) --------------------------------------------------

/// C: `FUN_overlay_d_93__021c1624`: message 0x45 about two mons and the move record (u16 id at
/// +0, u8 at +6).
pub fn msg_1624<E: FlowEnv>(env: &mut E, mon_a: u32, mon_b: u32, rec: &[u8]) {
    msg_start(env, 0x19BD);
    let a = mon_id(env, mon_a);
    arg_ec(env, 3, a);
    let b = mon_id(env, mon_b);
    arg_ec(env, 4, b);
    arg_ec(env, 0x12, rec_id(rec));
    arg_ec(env, 0x16, rec[6] as u32);
    msg_send(env, 0x45);
    msg_end(env, 0x19BD + 7);
}

/// C: `FUN_overlay_d_93__021c15e8`: the `0x19aa` record bracket around the message builder `1624`
/// (two mons and the move record).
pub fn record_bracket_1624<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    mon_a: u32,
    mon_b: u32,
    rec: &[u8],
) {
    let old = bracket_open(w);
    msg_1624(&mut Lend::new(env, w), mon_a, mon_b, rec);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c1758`: message 0x75 about one mon.
pub fn msg_1758<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0x1A17);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    msg_send(env, 0x75);
    msg_end(env, 0x1A17 + 3);
}

/// C: `FUN_overlay_d_93__021c16f8`: message by mode: 1 -> message 0x202 through the command queue
/// (`cd64c`); otherwise a `0x1a04` record bracket around `1758`.
pub fn msg_16f8<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, q: &mut [u8], mon: u32, mode: u32) {
    let id = mon_id(&mut Lend::wh(env, w, vec![&mut *q]), mon);
    if mode != 1 {
        let old = bracket_open(w);
        msg_1758(&mut Lend::wh(env, w, vec![&mut *q]), mon);
        bracket_close(w, old);
        return;
    }
    cmd_queue::put_cmd_list(&mut Lend::new(env, w), q, 0x5B, 0x202, &[id, LIST_END]);
}

/// C: `FUN_overlay_d_93__021c17cc`: message 0x44: mon `mon` and the first `n` mon pointers of the
/// array `mons` (u32 each).
pub fn msg_17cc<E: FlowEnv>(env: &mut E, mon: u32, n: u32, mons: &[u8]) {
    msg_start(env, 0x1A3F);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 5, n);
    let mut i = 0;
    if n != 0 {
        loop {
            let m =
                u32::from_le_bytes(mons[4 * i as usize..4 * i as usize + 4].try_into().unwrap());
            let id = mon_id(env, m);
            arg_ec(env, i + 6, id);
            i += 1;
            if i >= n {
                break;
            }
        }
    }
    msg_send(env, 0x44);
    msg_end(env, 0x1A47);
}

/// C: `FUN_overlay_d_93__021c1788`: the `0x1a29` record bracket around `17cc`.
pub fn record_bracket_17cc<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    mon: u32,
    n: u32,
    mons: &[u8],
) {
    let old = bracket_open(w);
    msg_17cc(&mut Lend::new(env, w), mon, n, mons);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c1828`: copies a 7-byte struct (move id, `r6`, move type, ids) to
/// `6848` and puts command 0x2D (`rec` = the move record).
pub fn put_move_used<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    r2: u32,
    mon_a: u32,
    mon_b: u32,
    rec: &[u8],
    r6: u32,
) {
    let ia = mon_id(&mut Lend::c(env, cx), mon_a);
    let ib = mon_id(&mut Lend::c(env, cx), mon_b);
    let mut s = [0u8; 7];
    s[0..2].copy_from_slice(&rec[0..2]);
    s[2..4].copy_from_slice(&(r6 as u16).to_le_bytes());
    s[4] = rec[6];
    s[5] = ia as u8;
    s[6] = r2 as u8;
    env.callc(0x021D_6848, &mut [Arg::V(mon_b), Arg::In(&s)], cx);
    cmd_queue::put_cmd(
        env,
        cx.q,
        0x2D,
        &[ib, ia, r2, rec[6] as u32, rec_id(rec), r6 & 0xFFFF],
        &mut cx.g[cx.gb..],
    );
}

/// C: `FUN_overlay_d_93__021c1910`: message 0x4D about mon `mon`, the move record and `r3`.
pub fn msg_1910<E: FlowEnv>(env: &mut E, mon: u32, rec: &[u8], r3: u32) {
    msg_start(env, 0x1AA7);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x12, rec_id(rec));
    arg_ec(env, 0x16, rec[6] as u32);
    arg_ec(env, 0x32, r3);
    msg_send(env, 0x4D);
    msg_end(env, 0x1AA7 + 6);
}

/// C: `FUN_overlay_d_93__021c18d4`: the `0x1a92` record bracket around `1910`.
pub fn record_bracket_1910<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    mon: u32,
    rec: &[u8],
    r3: u32,
) {
    let old = bracket_open(w);
    msg_1910(&mut Lend::new(env, w), mon, rec, r3);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c199c`: message 0x91 about mon `mon` and value `v`.
pub fn msg_199c<E: FlowEnv>(env: &mut E, mon: u32, v: u32) {
    msg_start(env, 0x1ACF);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_ec(env, 0x2E, v);
    msg_send(env, 0x91);
    msg_end(env, 0x1ACF + 4);
}

/// C: `FUN_overlay_d_93__021c195c`: when `5b40(mon)` says so, the `0x1abc` record bracket around
/// `199c`.
pub fn record_bracket_199c<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, v: u32) {
    if (env.callw(0x021D_5B40, &mut [Arg::V(mon)], w) & 0xFFFF) != 0 {
        let old = bracket_open(w);
        msg_199c(&mut Lend::new(env, w), mon, v);
        bracket_close(w, old);
    }
}

/// C: `FUN_overlay_d_93__021c1a10`: message 0x81 about mon `mon` and the move record.
pub fn msg_1a10<E: FlowEnv>(env: &mut E, mon: u32, rec: &[u8]) {
    msg_start(env, 0x1AEA);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x12, rec_id(rec));
    arg_ec(env, 0x16, rec[6] as u32);
    msg_send(env, 0x81);
    msg_end(env, 0x1AEA + 7);
}

/// C: `FUN_overlay_d_93__021c19d8`: the `0x1ada` record bracket around `1a10`.
pub fn record_bracket_1a10<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, rec: &[u8]) {
    let old = bracket_open(w);
    msg_1a10(&mut Lend::new(env, w), mon, rec);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c1c88`: message 0x88 about mon `mon` and the move record (u16 id at
/// +0).
pub fn msg_1c88<E: FlowEnv>(env: &mut E, mon: u32, rec: &[u8], r5: u32) {
    msg_start(env, 0x1B61);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x4D, r5);
    arg_ec(env, 0x12, rec_id(rec));
    msg_send(env, 0x88);
    msg_end(env, 0x1B61 + 5);
}

/// C: `FUN_overlay_d_93__021c1b5c`: per-mon message loop over the list `list`: collects the mons
/// for which `1c68` (`only_hit == 1`) / `1c78` answers, then message 0x82 with their count and
/// total; when the reply slot 0x48 stays 0 sends `cmd` too.
pub fn msg_1b5c<E: FlowEnv>(
    env: &mut E,
    mon: u32,
    list: u32,
    rec: &[u8],
    r5: u32,
    mode: u32,
    cmd: u32,
) {
    let n = list_next(env, 0x021C_1C48, list);
    let mut total = 0u32;
    let mut count = 0u32;
    msg_start(env, 0x1B2B);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x4D, r5);
    arg_ec(env, 0x12, rec_id(rec));
    arg_ec(env, 0x16, rec[6] as u32);
    let mut i = 0;
    if n != 0 {
        loop {
            let m = env.call(0x021C_1C58, &mut [Arg::V(list), Arg::V(i)]);
            let mut out = [0u8; 4];
            let callee = if mode == 0 { 0x021C_1C78 } else { 0x021C_1C68 };
            let hit = env.call(callee, &mut [Arg::V(list), Arg::V(m), Arg::Out(&mut out)]);
            if hit != 0 {
                total = total.wrapping_add(u32::from_le_bytes(out));
                let id = mon_id(env, m);
                arg_ec(env, count + 6, id);
                count += 1;
            }
            i += 1;
            if i >= n {
                break;
            }
        }
    }
    if count != 0 {
        arg_ec(env, 5, count);
        arg_630(env, 0x47, 0);
        arg_630(env, 0x48, 0);
        arg_ec(env, 0x32, total);
        msg_send(env, 0x82);
        let r = read_back(env, 0x48);
        if r == 0 {
            msg_send(env, cmd);
        }
    }
    msg_end(env, 0x1B52);
}

/// C: `FUN_overlay_d_93__021c14e0`: for every element of the list `list`: runs `6b5c`; when it
/// hits, sets the per-turn flag (`p[5]` bit 0) and runs the `0x197b` record bracket around `6bb8`.
pub fn list_apply_6b5c<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    p: &mut [u8],
    r1: u32,
    list: u32,
    rec: u32,
) {
    list_begin(&mut Lend::new(env, w), 0x021C_156C, list);
    let mut el = list_next(&mut Lend::new(env, w), 0x021C_157C, list);
    while el != 0 {
        if env.callw(
            0x021C_6B5C,
            &mut [Arg::P, Arg::V(r1), Arg::V(el), Arg::PAt(rec)],
            w,
        ) != 0
        {
            if p[5] & 1 == 0 {
                p[5] = p[5] & 0xFE | 1;
            }
            let old = bracket_open(w);
            env.callw(0x021C_6BB8, &mut [Arg::P, Arg::V(r1), Arg::V(el)], w);
            bracket_close(w, old);
            env.callw(0x021C_158C, &mut [Arg::V(list), Arg::V(el)], w);
        }
        el = list_next(&mut Lend::new(env, w), 0x021C_159C, list);
    }
}

/// C: `FUN_overlay_d_93__021c1678`: event 0x3a (chance gate on the mon's HP): `*hp_slot` becomes
/// HP-1 when the event answers non-zero; returns the answer (0 when HP > `*hp_slot`).
pub fn hp_gate<E: FlowEnv>(
    env: &mut E,
    mon_a: u32,
    mon_b: u32,
    r4: u32,
    hp_slot: &mut [u8],
) -> u32 {
    let hp = mon_get(env, mon_b, 0xD);
    if (u16::from_le_bytes([hp_slot[0], hp_slot[1]]) as i32) < hp as i32 {
        return 0;
    }
    msg_start(env, 0x19DF);
    let a = mon_id(env, mon_a);
    arg_ec(env, 3, a);
    let b = mon_id(env, mon_b);
    arg_ec(env, 4, b);
    arg_ec(env, 0x51, r4);
    arg_630(env, 0x3A, 0);
    msg_send(env, 0x74);
    let r = read_back(env, 0x3A);
    msg_end(env, 0x19DF + 7);
    if r != 0 {
        let hp = mon_get(env, mon_b, 0xD) as u16;
        hp_slot[0..2].copy_from_slice(&hp.wrapping_sub(1).to_le_bytes());
    }
    r
}

/// C: `FUN_overlay_d_93__021c20bc`: 1 when `7330(work, mon)` answers non-zero.
pub fn effect_applies<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    (env.call(0x021C_7330, &mut [Arg::P, Arg::V(mon)]) != 0) as u32
}

/// C: `FUN_overlay_d_93__021c1ce4`: for a move whose byte 6 is 9: every mon of the list `list`
/// whose `62a8` is 3 and `6d7c` is 0 gets `557c(work, mon, 3, 1)`.
pub fn list_apply_557c<E: FlowEnv>(env: &mut E, rec: &[u8], list: u32) {
    if rec[6] != 9 {
        return;
    }
    list_begin(env, 0x021C_1D30, list);
    let mut mon = list_next(env, 0x021C_1D40, list);
    while mon != 0 {
        if env.call(0x021D_62A8, &mut [Arg::V(mon)]) == 3
            && env.call(0x021D_6D7C, &mut [Arg::V(mon)]) == 0
        {
            env.call(
                0x021C_557C,
                &mut [Arg::P, Arg::V(mon), Arg::V(3), Arg::V(1)],
            );
        }
        mon = list_next(env, 0x021C_1D50, list);
    }
}

/// C: `FUN_overlay_d_93__021c1a54`: the post-hit message sequence: `1ce4`, then inside a `0x1b07`
/// record bracket five `1b5c` passes (messages 0x84, 0x83, 0x85, 0x86, 0x87) with a loop over the
/// list `list` (`195c`) in between, then `1c88`.
pub fn post_hit_messages<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    mon: u32,
    list: u32,
    r6: u32,
) {
    list_apply_557c(&mut Lend::new(env, w), rec, list);
    let old = bracket_open(w);
    msg_1b5c(&mut Lend::new(env, w), mon, list, rec, r6, 0, 0x84);
    msg_1b5c(&mut Lend::new(env, w), mon, list, rec, r6, 1, 0x83);
    msg_1b5c(&mut Lend::new(env, w), mon, list, rec, r6, 0, 0x85);
    list_begin(&mut Lend::new(env, w), 0x021C_1B2C, list);
    let mut el = list_next(&mut Lend::new(env, w), 0x021C_1B3C, list);
    while el != 0 {
        record_bracket_199c(env, w, el, 1);
        el = list_next(&mut Lend::new(env, w), 0x021C_1B4C, list);
    }
    msg_1b5c(&mut Lend::new(env, w), mon, list, rec, r6, 0, 0x86);
    msg_1b5c(&mut Lend::new(env, w), mon, list, rec, r6, 1, 0x87);
    msg_1c88(&mut Lend::new(env, w), mon, rec, r6);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c1d60`: chance gate `chance`: `5bb0(mon, 5)` or `76f4` applying gives
/// 1 (state update 6 / 4 on the mon); with `chance >= 100` and neither, the `0x1bad` record
/// bracket around `7780`; returns 1 when one of the effects applied.
pub fn chance_gate<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, chance: u32) -> u32 {
    if env.callw(0x021D_5BB0, &mut [Arg::V(mon), Arg::V(5)], w) != 0 {
        mon_set5fc0(&mut Lend::new(env, w), mon, 6);
        return 1;
    }
    if env.callw(0x021C_76F4, &mut [Arg::P, Arg::V(mon), Arg::V(chance)], w) != 0 {
        mon_set5fc0(&mut Lend::new(env, w), mon, 4);
        return 1;
    }
    if chance > 99 {
        let old = bracket_open(w);
        env.callw(0x021C_7780, &mut [Arg::P, Arg::V(mon)], w);
        bracket_close(w, old);
    }
    0
}

/// C: `FUN_overlay_d_93__021c1ccc`: looks up `7664(move id)` and runs the gate `1d60` on mon
/// `mon`; returns its answer.
pub fn move_chance_gate<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    rec: &[u8],
    r2: u32,
    mon: u32,
) -> u32 {
    let chance = env.callw(
        0x021C_7664,
        &mut [Arg::P, Arg::V(rec_id(rec)), Arg::V(r2)],
        w,
    );
    chance_gate(env, w, mon, chance)
}

/// C: `FUN_overlay_d_93__021c1e14`: inside a `0x1bf2` record bracket `78f4` picks a target; when it
/// does and `3370` is clear returns `334c(work, a, target, 1)` (else 0).
pub fn pick_and_apply<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    a: u32,
    b: u32,
    c: u32,
) -> u32 {
    let old = bracket_open(w);
    let mut ret = 0;
    let t = env.callwc(
        0x021C_78F4,
        &mut [Arg::P, Arg::V(a), Arg::V(b), Arg::V(c)],
        w,
        cx,
    );
    if t != 0 && mon_unavailable(&mut Lend::wc(env, w, cx), a) == 0 {
        ret = gate_then_apply(env, w, cx, a, t, 1);
    }
    bracket_close(w, old);
    ret
}

/// C: `FUN_overlay_d_93__021c1dcc`: for move category 8 (`0201c1bc`): `1e14` on the result of
/// `7b3c(r5, 0201bd28(move, 0x19))`, then `5d88` when it answers.
pub fn category8_followup<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    r2: u32,
    r3: u32,
    r5: u32,
) {
    if env.callwc(0x0201_C1BC, &mut [Arg::V(rec_id(rec))], w, cx) != 8 {
        return;
    }
    let x = env.callwc(0x0201_BD28, &mut [Arg::V(rec_id(rec)), Arg::V(0x19)], w, cx);
    let y = env.callwc(0x021D_7B3C, &mut [Arg::V(r5), Arg::V(x)], w, cx);
    if pick_and_apply(env, w, cx, r2, r3, y & 0xFFFF) != 0 {
        env.callwc(0x021C_5D88, &mut [Arg::P, Arg::V(r3), Arg::V(0x383)], w, cx);
    }
}

/// `FUN_overlay_d_93__021d75a8`: adds argument `id` = `v` (second flavour).
fn arg_a8<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_75A8, &mut [Arg::V(id), Arg::V(v)]);
}

/// `FUN_overlay_d_93__021d7674`: adds the roll-range reply slot.
fn arg_674<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32, d: u32) {
    env.call(
        0x021D_7674,
        &mut [Arg::V(a), Arg::V(b), Arg::V(c), Arg::V(d)],
    );
}

/// `FUN_overlay_d_93__021d7b10`: fixed-point scale `(value, factor)`.
fn fx_mul<E: FlowEnv>(env: &mut E, v: u32, f: u32) -> u32 {
    env.call(0x021D_7B10, &mut [Arg::V(v), Arg::V(f)])
}

/// `work + 4`: the main module.
const OFS_MAIN: usize = 0x4;

/// C: `FUN_overlay_d_93__021c2ab0`: event 0x41 (message 0x7a): `ef884()` when the event answers 0,
/// else 0.
pub fn event_7a<E: FlowEnv>(env: &mut E) -> u32 {
    msg_start(env, 0x1F23);
    arg_630(env, 0x41, 0);
    msg_send(env, 0x7A);
    let r = read_back(env, 0x41);
    msg_end(env, 0x1F23 + 4);
    if r == 0 {
        env.call(0x021E_F884, &mut [])
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021c2af0`: message 0x7b about mon `mon`; returns the `775c(0x35)` answer.
pub fn msg_2af0<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    msg_start(env, 0x1F3D);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_674(env, 0x35, 0x1000, 0x19A, 0x20000);
    msg_send(env, 0x7B);
    let r = read_back(env, 0x35);
    msg_end(env, 0x1F3D + 5);
    r
}

/// The id of `mon`, or 0x1F when `mon` is null.
fn mon_id_or_none<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    if mon == 0 {
        0x1F
    } else {
        mon_id(env, mon)
    }
}

/// C: `FUN_overlay_d_93__021c2b3c`: event 0x65 message about (`a` or 0x1f), `b`, `c`; returns the
/// `775c(0x41)` answer.
pub fn msg_2b3c<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32) -> u32 {
    msg_start(env, 0x1F55);
    let u = mon_id_or_none(env, a);
    arg_ec(env, 3, u);
    let u = mon_id(env, b);
    arg_ec(env, 4, u);
    arg_ec(env, 0x1D, c);
    arg_630(env, 0x41, 0);
    msg_send(env, 0x65);
    let r = read_back(env, 0x41);
    msg_end(env, 0x1F5C);
    r
}

/// C: `FUN_overlay_d_93__021c2ba4`: event 0x67 message about three values.
pub fn msg_2ba4<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32) {
    msg_start(env, 0x1F6A);
    let u = mon_id(env, a);
    arg_a8(env, 4, u);
    let u = mon_id(env, b);
    arg_a8(env, 3, u);
    arg_a8(env, 0x1D, c);
    msg_send(env, 0x67);
    msg_end(env, 0x1F6A + 5);
}

/// C: `FUN_overlay_d_93__021c2bf0`: event 0x68 message about mon `a`, (`b` or 0x1f), `c` and `d`.
pub fn msg_2bf0<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32, d: u32) {
    msg_start(env, 0x1F7E);
    let u = mon_id(env, a);
    arg_ec(env, 4, u);
    let u = mon_id_or_none(env, b) & 0xFF;
    arg_ec(env, 3, u);
    arg_ec(env, 0x1D, c);
    arg_ec(env, 0x1E, d);
    msg_send(env, 0x68);
    msg_end(env, 0x1F87);
}

/// C: `FUN_overlay_d_93__021c2c50`: event 0x69 message about mon `a`, (`b` or 0x1f) and `c`.
pub fn msg_2c50<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32) {
    msg_start(env, 0x1F95);
    let u = mon_id(env, a);
    arg_ec(env, 4, u);
    let u = mon_id_or_none(env, b) & 0xFF;
    arg_ec(env, 3, u);
    arg_ec(env, 0x1D, c);
    msg_send(env, 0x69);
    msg_end(env, 0x1F9D);
}

/// C: `FUN_overlay_d_93__021c2ca8`: event 0x6a message about mon `mon` (sent with `70e4`).
pub fn msg_2ca8<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0x1FA9);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    env.call(0x021D_70E4, &mut [Arg::P, Arg::V(0x6A)]);
    msg_end(env, 0x1FA9 + 3);
}

/// C: `FUN_overlay_d_93__021c2d10`: event 0x51 roll (move data 0x12; reply slots 0x41 / 0x26;
/// `78ac(100)`); returns 1 on success.
pub fn event_51_roll<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    mon_a: u32,
    mon_b: u32,
) -> u32 {
    let data = env.callwr(0x0201_BD28, &mut [Arg::V(rec_id(rec)), Arg::V(0x12)], w);
    msg_start(&mut Lend::new_ref(env, w), 0x1FCE);
    let a = mon_id(&mut Lend::new_ref(env, w), mon_a);
    arg_ec(&mut Lend::new_ref(env, w), 3, a);
    let b = mon_id(&mut Lend::new_ref(env, w), mon_b);
    arg_ec(&mut Lend::new_ref(env, w), 4, b);
    arg_ec(&mut Lend::new_ref(env, w), 0x12, rec_id(rec));
    arg_630(&mut Lend::new_ref(env, w), 0x41, 0);
    arg_a8(&mut Lend::new_ref(env, w), 0x26, data & 0xFF);
    msg_send(&mut Lend::new_ref(env, w), 0x51);
    let r1 = read_back(&mut Lend::new_ref(env, w), 0x41);
    let r2 = read_back(&mut Lend::new_ref(env, w), 0x26);
    msg_end(&mut Lend::new_ref(env, w), 0x1FCE + 0xA);
    if r1 & 0xFF == 0 {
        let roll = env.callwr(0x021D_78AC, &mut [Arg::V(100)], w);
        if roll < (r2 & 0xFF) {
            return 1;
        }
        if env.callwr(0x021B_911C, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(0)], w) != 0 {
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021c2204`: event 0x90 message about mon `mon` and value `v`; returns the
/// answer of `775c(0x41)`.
pub fn event_90<E: FlowEnv>(env: &mut E, mon: u32, v: u32) -> u32 {
    msg_start(env, 0x1D19);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_ec(env, 0x2D, v);
    arg_630(env, 0x41, 0);
    msg_send(env, 0x90);
    let r = read_back(env, 0x41);
    msg_end(env, 0x1D19 + 6);
    r
}

/// C: `FUN_overlay_d_93__021c2314`: `5be4(work, mon, 0)` then `4f64(work, mon)`.
pub fn run_5be4_4f64<E: FlowEnv>(env: &mut E, mon: u32) {
    env.call(0x021C_5BE4, &mut [Arg::P, Arg::V(mon), Arg::V(0)]);
    env.call(0x021C_4F64, &mut [Arg::P, Arg::V(mon)]);
}

/// C: `FUN_overlay_d_93__021c20d0`: applies `-amount` to mon `mon` (0 = nothing): `5b74`, state
/// update 2, the HANDEX record `hem` (when not null: `99a0(work, hem)`, `8810(hem)`), `195c`, the
/// `4f64` chain; returns 1 when applied.
pub fn apply_amount<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    mon: u32,
    amount: u32,
    hem: PassPtr,
) -> u32 {
    if amount == 0 {
        return 0;
    }
    env.callw(
        0x021C_5B74,
        &mut [
            Arg::P,
            Arg::V(mon),
            Arg::V(amount.wrapping_neg()),
            Arg::V(1),
        ],
        w,
    );
    mon_set5fc0(&mut Lend::new(env, w), mon, 2);
    if !hem.is_null() {
        env.callw(0x021C_99A0, &mut [Arg::P, hem.arg()], w);
        env.callw(0x021C_8810, &mut [hem.arg()], w);
    }
    record_bracket_199c(env, w, mon, 1);
    if env.callw(0x021C_4F64, &mut [Arg::P, Arg::V(mon)], w) != 0 {
        env.callw(0x021B_C2B0, &mut [Arg::P], w);
        return 1;
    }
    1
}

/// C: `FUN_overlay_d_93__021c2048`: tries the damage side effect on mon `mon`: `6cac` decides,
/// then the HANDEX record at `work + 0x1ae4` is filled (`8838` / `8860`), `20bc` / `20d0` apply it.
pub fn try_damage_effect<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, r2: u32, r3: u32) {
    if env.callw(0x021D_5B68, &mut [Arg::V(mon)], w) != 0 {
        return;
    }
    let mut out = [0u8; 4];
    let found = env.callw(
        0x021C_6CAC,
        &mut [
            Arg::P,
            Arg::V(mon),
            Arg::V(r2),
            Arg::V(r3),
            Arg::Out(&mut out),
        ],
        w,
    );
    if found == 0 {
        return;
    }
    let hem = PassPtr::Host(w.mem[OFS_HEM_1AE4 as usize..].as_ptr() as usize);
    env.callw(0x021C_8838, &mut [hem.arg(), Arg::V(2), Arg::V(0x17A)], w);
    let id = mon_id(&mut Lend::new(env, w), mon);
    env.callw(0x021C_8860, &mut [hem.arg(), Arg::V(id)], w);
    if u32::from_le_bytes(out) != 0 || effect_applies(&mut Lend::new(env, w), mon) != 0 {
        apply_amount(env, w, mon, found, hem);
    }
}

/// C: `FUN_overlay_d_93__021c2250`: state change of mon `mon`: record brackets around `7ad0` and
/// `7b0c`, command 0x1e, `5b44(mon, r2)`; with `r3` also `6730`, command 0x17 and `6134`.
pub fn mon_state_change<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    r2: u32,
    r3: u32,
) {
    let id = mon_id(&mut Lend::wc(env, w, cx), mon);
    let v = env.callwc(0x021D_5B40, &mut [Arg::V(mon)], w, cx) & 0xFFFF;
    let old = bracket_open(w);
    env.callwc(0x021C_7AD0, &mut [Arg::P, Arg::V(mon), Arg::V(r2)], w, cx);
    bracket_close(w, old);
    if r2 == 0 {
        env.callwc(0x021C_60EC, &mut [Arg::P, Arg::V(mon), Arg::V(0xF)], w, cx);
    }
    env.callwc(0x021D_CF24, &mut [Arg::V(mon)], w, cx);
    cmd_queue::put_cmd(
        &mut Lend::new(env, w),
        cx.q,
        0x1E,
        &[id, r2],
        &mut cx.g[cx.gb..],
    );
    env.callwc(0x021D_5B44, &mut [Arg::V(mon), Arg::V(r2)], w, cx);
    if r2 != 0 {
        env.callwc(0x021D_CF5C, &mut [Arg::V(mon)], w, cx);
    }
    let old = bracket_open(w);
    env.callwc(0x021C_7B0C, &mut [Arg::P, Arg::V(mon)], w, cx);
    bracket_close(w, old);
    if r3 != 0 {
        env.callwc(0x021D_6730, &mut [Arg::V(mon), Arg::V(v)], w, cx);
        cmd_queue::put_cmd(
            &mut Lend::new(env, w),
            cx.q,
            0x17,
            &[id, v],
            &mut cx.g[cx.gb..],
        );
        env.callwc(0x021C_6134, &mut [Arg::P, Arg::V(mon), Arg::V(8)], w, cx);
    }
}

/// C: `FUN_overlay_d_93__021c2128`: side effect on mon `mon` keyed by its `5b40` id: placeholder
/// command 0x42 (`cd4fc` / `cd568`), the `cc918` record, event 0x90 (`2204`), `6dd0`, the `88e4`
/// gate, the `805c` check and `2250`; returns 1 when `88e4` answered 2.
pub fn placeholder_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
) -> u32 {
    let key = env.callwc(0x021D_5B40, &mut [Arg::V(mon)], w, cx) & 0xFFFF;
    let mut done = false;
    let id = mon_id(&mut Lend::wc(env, w, cx), mon);
    if env.callwc(
        0x021C_21F4,
        &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(id)],
        w,
        cx,
    ) != 0
    {
        let outer = bracket_open(w);
        if event_90(&mut Lend::wc(env, w, cx), mon, key) == 0 {
            let pos = cmd_queue::put_cmd_placeholder(
                &mut Lend::new(env, w),
                cx.q,
                0x42,
                &mut cx.g[cx.gb..],
            );
            let old = cmd_queue::advance_fields_set_low10(&mut w.mem[OFS_QUEUE_HEADER..], key);
            env.callwc(0x021C_6DD0, &mut [Arg::P, Arg::V(mon)], w, cx);
            done = env.callwc(0x021C_88E4, &mut [Arg::P], w, cx) == 2;
            bracket_close(w, old);
            if done {
                let id = mon_id(&mut Lend::wc(env, w, cx), mon);
                cmd_queue::put_cmd_at(
                    &mut Lend::new(env, w),
                    cx.q,
                    pos,
                    0x42,
                    &[id],
                    &mut cx.g[cx.gb..],
                );
                if env.callwc(0x021D_805C, &mut [Arg::V(key), Arg::V(0x10)], w, cx) != 0 {
                    mon_state_change(env, w, cx, mon, 0, 1);
                }
            }
        }
        bracket_close(w, outer);
    }
    done as u32
}

/// C: `FUN_overlay_d_93__021c1e74`: the base damage computation (event 0x38 ... 0x37, `CalcBaseDamage`
/// (`021d7a44`), STAB / type / random / critical scaling): `out` (u16) = the damage; returns 1 when
/// the event answered (no damage computed). The stat getters
/// get the address of the move record `rec` (`rec_addr`, B16).
#[allow(clippy::too_many_arguments)]
pub fn base_damage<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    mon_a: u32,
    mon_b: u32,
    rec: &[u8],
    rec_addr: u32,
    type_eff: u32,
    scale: u32,
    crit: u32,
    no_random: u32,
    out: &mut [u8],
) -> u32 {
    let cat = env.callwr(0x0201_C1E8, &mut [Arg::V(rec_id(rec))], w);
    let mut flag = 0;
    msg_start(&mut Lend::new_ref(env, w), 0x1C21);
    arg_ec(&mut Lend::new_ref(env, w), 0x38, type_eff);
    let a = mon_id(&mut Lend::new_ref(env, w), mon_a);
    arg_ec(&mut Lend::new_ref(env, w), 3, a);
    let b = mon_id(&mut Lend::new_ref(env, w), mon_b);
    arg_ec(&mut Lend::new_ref(env, w), 4, b);
    arg_ec(&mut Lend::new_ref(env, w), 0x45, crit);
    arg_ec(&mut Lend::new_ref(env, w), 0x16, rec[6] as u32);
    arg_ec(&mut Lend::new_ref(env, w), 0x12, rec_id(rec));
    arg_ec(&mut Lend::new_ref(env, w), 0x1A, cat);
    arg_a8(&mut Lend::new_ref(env, w), 0x37, 0);
    msg_send(&mut Lend::new_ref(env, w), 0x46);
    let mut dmg = read_back(&mut Lend::new_ref(env, w), 0x37);
    if dmg == 0 {
        let atk = env.callwr(
            0x021C_728C,
            &mut [Arg::P, Arg::V(mon_a), Arg::V(mon_b), Arg::PAt(rec_addr)],
            w,
        );
        let pow = env.callwr(
            0x021C_7374,
            &mut [
                Arg::P,
                Arg::V(mon_a),
                Arg::V(mon_b),
                Arg::PAt(rec_addr),
                Arg::V(crit),
            ],
            w,
        );
        let def = env.callwr(
            0x021C_7474,
            &mut [
                Arg::P,
                Arg::V(mon_a),
                Arg::V(mon_b),
                Arg::PAt(rec_addr),
                Arg::V(crit),
            ],
            w,
        );
        let level = mon_get(&mut Lend::new_ref(env, w), mon_a, 0xF) & 0xFF;
        let mut d = env.callwr(
            0x021D_7A44,
            &mut [Arg::V(atk), Arg::V(pow), Arg::V(level), Arg::V(def)],
            w,
        );
        if scale != 0x1000 {
            d = fx_mul(&mut Lend::new_ref(env, w), d, scale);
        }
        let weather = event_7a(&mut Lend::new_ref(env, w));
        let f = env.callwr(
            0x021D_7C38,
            &mut [Arg::V(weather), Arg::V(rec[6] as u32)],
            w,
        );
        if f != 0x1000 {
            d = fx_mul(&mut Lend::new_ref(env, w), d, f);
        }
        if crit != 0 {
            d <<= 1;
        }
        if env.callwr(0x021B_911C, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(7)], w) == 0 {
            let pct = if no_random == 0 {
                let r = env.callwr(0x021D_78AC, &mut [Arg::V(0x10)], w);
                100u32.wrapping_sub(r) & 0xFFFF
            } else {
                0x55
            };
            d = crate::calc::u32_div(d.wrapping_mul(pct), 100);
        }
        if rec[6] != 0x11 {
            let f = env.callwr(
                0x021C_75EC,
                &mut [Arg::P, Arg::V(mon_a), Arg::V(rec[6] as u32)],
                w,
            );
            d = fx_mul(&mut Lend::new_ref(env, w), d, f);
        }
        d = env.callwr(0x021D_7A6C, &mut [Arg::V(d), Arg::V(type_eff)], w);
        if cat == 1
            && env.callwr(0x021D_62A8, &mut [Arg::V(mon_a)], w) == 4
            && mon_get(&mut Lend::new_ref(env, w), mon_a, 0x11) != 0x3E
        {
            d = crate::calc::u32_div(d.wrapping_mul(0x32), 100);
        }
        if d == 0 {
            d = 1;
        }
        arg_674(&mut Lend::new_ref(env, w), 0x35, 0x1000, 0x29, 0x20000);
        arg_a8(&mut Lend::new_ref(env, w), 0x32, d);
        msg_send(&mut Lend::new_ref(env, w), 0x47);
        let r1 = read_back(&mut Lend::new_ref(env, w), 0x35);
        let r3 = read_back(&mut Lend::new_ref(env, w), 0x32);
        dmg = fx_mul(&mut Lend::new_ref(env, w), r3, r1);
    } else {
        flag = 1;
    }
    msg_send(&mut Lend::new_ref(env, w), 0x48);
    msg_end(&mut Lend::new_ref(env, w), 0x1C8D);
    out[0..2].copy_from_slice(&(dmg as u16).to_le_bytes());
    flag
}

/// `work + 0x78A`: flag bits (bit 4 = "failure message already shown").
const OFS_FLAGS_78A: usize = 0x78A;
/// `work + 0x1AE4`: the HANDEX record (address only).
const OFS_HEM_1AE4: u32 = 0x1AE4;

/// Sets bit 0 of the per-turn struct's flag byte (byte 5) unless it is set.
fn set_turn_flag(p: &mut [u8]) {
    if p[5] & 1 == 0 {
        p[5] = p[5] & 0xFE | 1;
    }
}

/// C: `FUN_overlay_d_93__021c236c`: the move-effect roll: move data 0xb / 0xc, `7c70`, the event
/// answers 0x1d / 0x26 / 0x41, a 0..100 roll against the percentage; `out` (u32) = the answer;
/// returns the effect id or 0.
pub fn effect_roll<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    move_id: u32,
    mon_b: u32,
    mon_a: u32,
    out: &mut [u8],
) -> u32 {
    let data_b = env.callwr(0x0201_BD28, &mut [Arg::V(move_id), Arg::V(0xB)], w);
    let ty = env.callwr(0x0201_BF58, &mut [Arg::V(move_id)], w) & 0xFFFF;
    let data_c = env.callwr(0x0201_BD28, &mut [Arg::V(move_id), Arg::V(0xC)], w);
    let mut o = [0u8; 4];
    env.callwr(
        0x021D_7C70,
        &mut [Arg::V(ty), Arg::V(mon_b), Arg::Out(&mut o)],
        w,
    );
    let mut local18 = u32::from_le_bytes(o);
    msg_start(&mut Lend::new_ref(env, w), 0x1D7C);
    let a = mon_id(&mut Lend::new_ref(env, w), mon_a);
    arg_ec(&mut Lend::new_ref(env, w), 4, a);
    let b = mon_id(&mut Lend::new_ref(env, w), mon_b);
    arg_ec(&mut Lend::new_ref(env, w), 3, b);
    arg_ec(&mut Lend::new_ref(env, w), 0x12, move_id);
    arg_a8(&mut Lend::new_ref(env, w), 0x1D, data_b);
    arg_630(&mut Lend::new_ref(env, w), 0x41, 0);
    arg_630(&mut Lend::new_ref(env, w), 0x1E, local18);
    arg_a8(&mut Lend::new_ref(env, w), 0x26, data_c & 0xFF);
    msg_send(&mut Lend::new_ref(env, w), 100);
    let mut effect = read_back(&mut Lend::new_ref(env, w), 0x1D);
    let pct = read_back(&mut Lend::new_ref(env, w), 0x26);
    let block = read_back(&mut Lend::new_ref(env, w), 0x41);
    if effect == 0xFFFF {
        effect = 0;
    }
    local18 = read_back(&mut Lend::new_ref(env, w), 0x1E);
    msg_end(&mut Lend::new_ref(env, w), 0x1D8F);
    if block & 0xFF == 0 && effect != 0 {
        let roll = env.callwr(0x021D_78AC, &mut [Arg::V(100)], w);
        if roll < (pct & 0xFF) {
            out[0..4].copy_from_slice(&local18.to_le_bytes());
            return effect;
        }
        if env.callwr(0x021B_911C, &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(0)], w) != 0 {
            out[0..4].copy_from_slice(&local18.to_le_bytes());
            return effect;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021c264c`: message 0x60 about two mons and `r3`; returns the
/// `775c(0x1d)` answer.
pub fn msg_264c<E: FlowEnv>(env: &mut E, mon_a: u32, mon_b: u32, r3: u32) -> u32 {
    msg_start(env, 0x1E01);
    let a = mon_id(env, mon_a);
    arg_ec(env, 3, a);
    let b = mon_id(env, mon_b);
    arg_ec(env, 4, b);
    arg_ec(env, 0x3F, r3);
    arg_a8(env, 0x1D, 0);
    msg_send(env, 0x60);
    let r = read_back(env, 0x1D);
    msg_end(env, 0x1E01 + 7);
    r
}

/// C: `FUN_overlay_d_93__021c26a8`: message 0x61 about two mons, `r5` and `r1`.
pub fn msg_26a8<E: FlowEnv>(env: &mut E, r1: u32, mon_a: u32, mon_b: u32, r5: u32) {
    msg_start(env, 0x1E18);
    let a = mon_id(env, mon_a);
    arg_ec(env, 3, a);
    let b = mon_id(env, mon_b);
    arg_ec(env, 4, b);
    arg_ec(env, 0x3F, r5);
    arg_ec(env, 0x1D, r1);
    msg_send(env, 0x61);
    msg_end(env, 0x1E18 + 6);
}

/// C: `FUN_overlay_d_93__021c26fc`: message 0x62: `*slot` (u32) is both an argument and, after,
/// the answer of `775c(0x1e)`.
pub fn msg_26fc<E: FlowEnv>(env: &mut E, mon_a: u32, mon_b: u32, r4: u32, slot: &mut [u8]) {
    msg_start(env, 0x1E2E);
    let a = mon_id(env, mon_a);
    arg_ec(env, 3, a);
    let b = mon_id(env, mon_b);
    arg_ec(env, 4, b);
    arg_ec(env, 0x1D, r4);
    arg_a8(
        env,
        0x1E,
        u32::from_le_bytes(slot[0..4].try_into().unwrap()),
    );
    msg_send(env, 0x62);
    let r = read_back(env, 0x1E);
    slot[0..4].copy_from_slice(&r.to_le_bytes());
    msg_end(env, 0x1E2E + 7);
}

/// C: `FUN_overlay_d_93__021c282c`: the immunity check by reason `reason` (3, 4, 5, 0x10, 0x12,
/// 0xe, 0..5): 0 = none, 1 = hard, 2 = absorbed (ability checks), 3 = other; `r5` is a byte
/// compared with 0 / 2.
pub fn immunity_check<E: FlowEnv>(env: &mut E, mon: u32, reason: u32, _r4: u32, r5: u32) -> u32 {
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(reason)]) != 0 && r5 as u8 != 2 {
        return 1;
    }
    if (reason as i32) < 6 && env.call(0x021D_62A8, &mut [Arg::V(mon)]) != 0 && r5 as u8 == 0 {
        return 3;
    }
    if event_7a(env) == 1 && reason == 3 {
        return 3;
    }
    let checks: [(u32, &[u32]); 4] = [(5, &[8, 3]), (4, &[9]), (3, &[0xE]), (0x12, &[0xB])];
    for (r, ids) in checks {
        if reason == r {
            let s = env.call(0x021D_57FC, &mut [Arg::V(mon)]) & 0xFFFF;
            for &id in ids {
                if env.call(0x021E_8D24, &mut [Arg::V(s), Arg::V(id)]) != 0 {
                    return 2;
                }
            }
        }
    }
    if reason == 0xE && env.call(0x021D_62A8, &mut [Arg::V(mon)]) != 0 {
        return 3;
    }
    if reason == 0x10 && mon_get(env, mon, 0x10) == 0x79 {
        return 3;
    }
    0
}

/// C: `FUN_overlay_d_93__021c3fdc`: `195c(work, mon, 0)` for every mon the iterator
/// (`bd2b4` / `bd34c`) yields that `9a24(id of mon, id of that mon)` rejects.
pub fn notify_others<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32) {
    let a = mon_id(&mut Lend::new(env, w), mon);
    // the iterator state lives in the caller's frame
    let mut it = [0u8; 4];
    env.callw(0x021B_D2B4, &mut [Arg::PBuf(&mut it), Arg::P], w);
    loop {
        let mut o = [0u8; 4];
        let more = env.callw(
            0x021B_D34C,
            &mut [Arg::PBuf(&mut it), Arg::P, Arg::Out(&mut o)],
            w,
        );
        if more == 0 {
            break;
        }
        let other = u32::from_le_bytes(o);
        let b = mon_id(&mut Lend::new(env, w), other);
        if env.callw(0x021B_9A24, &mut [Arg::V(a), Arg::V(b)], w) == 0 {
            record_bracket_199c(env, w, other, 0);
        }
    }
}

/// C: `FUN_overlay_d_93__021c2910`: the failure-message switch on reason `reason` (1..7: messages
/// 0x25a / 0x256 / 0x259 / 0x258 / 0x257 / 0x25b / 0x25c via `5480`), then `2aa0` / the HANDEX
/// record, the Shaymin special case (`5444` = 0x1ec), `2c50` / `2bf0` / `2ca8` inside a `0x1f00`
/// record bracket, and `195c`.
#[allow(clippy::too_many_arguments)]
pub fn failure_message<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    r3: u32,
    reason: u32,
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
) {
    env.callwc(
        0x021C_5C1C,
        &mut [Arg::P, Arg::V(mon), Arg::V(reason), Arg::V(r5)],
        w,
        cx,
    );
    let msg = match reason {
        1 => Some(0x25A),
        2 => Some(0x256),
        3 => Some(0x259),
        4 => Some(0x96 << 2),
        5 => Some(0x257),
        6 => Some(0x25B),
        7 => Some(0x97 << 2),
        _ => None,
    };
    if let Some(id) = msg {
        env.callwc(0x021C_5480, &mut [Arg::P, Arg::V(mon), Arg::V(id)], w, cx);
    }
    if r6 == 0 {
        if r8 != 0 {
            // r1 still holds `r8` (the original reloads it for the test)
            env.callwc(0x021C_99A0, &mut [Arg::P, Arg::PAt(r8)], w, cx);
        }
    } else {
        let hem = cx.w_addr.wrapping_add(OFS_HEM_1AE4);
        env.callwc(
            0x021C_2AA0,
            &mut [Arg::V(reason), Arg::V(r5), Arg::V(mon), Arg::PAt(hem)],
            w,
            cx,
        );
        env.callwc(0x021C_99A0, &mut [Arg::P, Arg::PAt(hem)], w, cx);
        env.callwc(0x021C_8810, &mut [Arg::PAt(hem)], w, cx);
    }
    if reason == 3
        && env.callwc(0x021D_5444, &mut [Arg::V(mon)], w, cx) == 0x1EC
        && mon_get(&mut Lend::wc(env, w, cx), mon, 0x13) == 1
    {
        env.callwc(0x021D_8510, &mut [Arg::V(mon)], w, cx);
        env.callwc(0x021D_66D8, &mut [Arg::V(mon), Arg::V(0)], w, cx);
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        cmd_queue::put_cmd(
            &mut Lend::new(env, w),
            cx.q,
            0x4F,
            &[id, 0],
            &mut cx.g[cx.gb..],
        );
        env.callwc(0x021D_8484, &mut [Arg::V(mon)], w, cx);
        env.callwc(0x021C_5D88, &mut [Arg::P, Arg::V(mon), Arg::V(0xDE)], w, cx);
    }
    let old = bracket_open(w);
    if env.callwc(0x021D_7D6C, &mut [Arg::V(reason)], w, cx) == 0 {
        if reason == 0x10 {
            msg_2ca8(&mut Lend::wc(env, w, cx), mon);
            if mon_get(&mut Lend::wc(env, w, cx), mon, 0x11) == 0x7F {
                notify_others(&mut Lend::c(env, cx), w, mon);
            }
        } else {
            msg_2c50(&mut Lend::wc(env, w, cx), mon, r3, reason);
        }
    } else {
        msg_2bf0(&mut Lend::wc(env, w, cx), mon, r3, reason, r5);
    }
    bracket_close(w, old);
    if r7 == 0 {
        record_bracket_199c(&mut Lend::c(env, cx), w, mon, 3);
    }
}

/// C: `FUN_overlay_d_93__021c27a0`: the `282c` immunity check -> `5c74` when `f`; else the `2b3c` /
/// `2ba4` messages (inside a `0x1fef` record bracket) and the `0x78a` flag; returns the check result.
#[allow(clippy::too_many_arguments)]
pub fn immunity_or_message<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
) -> u32 {
    let r = immunity_check(&mut Lend::new(env, w), a, c, d, e);
    if r != 0 {
        if f != 0 {
            env.callw(
                0x021C_5C74,
                &mut [Arg::P, Arg::V(a), Arg::V(r), Arg::V(c)],
                w,
            );
        }
        return 1;
    }
    let old = bracket_open(w);
    let r = msg_2b3c(&mut Lend::new(env, w), b, a, c);
    if r != 0 && f != 0 {
        msg_2ba4(&mut Lend::new(env, w), a, b, c);
        w.mem[OFS_FLAGS_78A] |= 0x10;
    }
    bracket_close(w, old);
    r
}

/// C: `FUN_overlay_d_93__021c2758`: when `27a0` answers 0: `2910` (failure message) and return 1,
/// else 0.
#[allow(clippy::too_many_arguments)]
pub fn check_then_fail<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
) -> u32 {
    if immunity_or_message(&mut Lend::c(env, cx), w, a, b, c, d, 0, e) == 0 {
        failure_message(env, w, cx, a, b, c, d, f, 0, 0);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021c25a4`: the effect message round: `8810` on the HANDEX record at
/// `work + 0x1ae4`, then `r5` (0xffff = ask `264c`, else `26a8`), `26fc`, `2758`; the record is
/// flushed (`99a0`, `8810`) when `8828` said so and the check passes; returns 1 when the effect
/// went through.
#[allow(clippy::too_many_arguments)]
pub fn effect_round<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon_a: u32,
    mon_b: u32,
    _move_id: u32,
    r5: u32,
    r6: u32,
    r7: u32,
) -> u32 {
    let hem = cx.w_addr.wrapping_add(OFS_HEM_1AE4);
    env.callwc(0x021C_8810, &mut [Arg::PAt(hem)], w, cx);
    let mut r5 = r5;
    if r5 == 0xFFFF {
        r5 = msg_264c(&mut Lend::wc(env, w, cx), mon_a, mon_b, hem);
        if r5 == 0 || r5 == 0xFFFF {
            return 0;
        }
    } else {
        msg_26a8(&mut Lend::wc(env, w, cx), r5, mon_a, mon_b, hem);
    }
    let mut slot = r6.to_le_bytes();
    msg_26fc(&mut Lend::wc(env, w, cx), mon_a, mon_b, r5, &mut slot);
    let r6 = u32::from_le_bytes(slot);
    let busy = env.callwc(0x021C_8828, &mut [Arg::PAt(hem)], w, cx);
    let ok = check_then_fail(env, w, cx, mon_b, mon_a, r5, r6, r7, (busy == 0) as u32);
    if ok != 0 {
        if busy != 0 {
            env.callwc(0x021C_99A0, &mut [Arg::P, Arg::PAt(hem)], w, cx);
            env.callwc(0x021C_8810, &mut [Arg::PAt(hem)], w, cx);
        }
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021c2488`: the same effect for every mon of the list `list`: `25a4` with
/// the roll results (the per-turn flag in `p` on success), else `5718` unless flag 4 of
/// `work + 0x78a` is set.
pub fn effect_each<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    move_id: u32,
    mon: u32,
    list: u32,
) {
    let data_b = env.callwc(0x0201_BD28, &mut [Arg::V(move_id), Arg::V(0xB)], w, cx);
    let ty = env.callwc(0x0201_BF58, &mut [Arg::V(move_id)], w, cx) & 0xFFFF;
    // The original stores the u16 into the low half of a stack word BELOW sp (`sp - 4`) and
    // hands the whole word to `7c70`; the high half is whatever an earlier callee left there:
    // zero before the first `25a4` call, afterwards that callee's saved `lr` (a return address
    // into this function, 0x021Cxxxx). Reproduced like that (see README deviations).
    let mut ty_word = ty;
    let mut hit = false;
    if env.callwc(0x021C_2564, &mut [Arg::V(list)], w, cx) == 0 {
        env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
        return;
    }
    env.callwc(0x021C_2574, &mut [Arg::V(list)], w, cx);
    let mut el = env.callwc(0x021C_2584, &mut [Arg::V(list)], w, cx);
    while el != 0 {
        let mut o = [0u8; 4];
        env.callwc(
            0x021D_7C70,
            &mut [Arg::V(ty_word), Arg::V(mon), Arg::Out(&mut o)],
            w,
            cx,
        );
        let roll = u32::from_le_bytes(o);
        let ok = effect_round(env, w, cx, mon, el, move_id, data_b, roll, 1);
        ty_word = 0x021C_0000 | ty;
        if ok != 0 {
            set_turn_flag(p);
            hit = true;
        }
        el = env.callwc(0x021C_2594, &mut [Arg::V(list)], w, cx);
    }
    if !hit && w.mem[OFS_FLAGS_78A] & 0x10 == 0 {
        env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
    }
}

/// C: `FUN_overlay_d_93__021c2f70`: event 0x59: hit/target counts for hit index `idx`
/// (`0201c0bc`), answers 0x1f / 0x20 / 0x35; `out1` (u32) = the answer (8 -> 0), `out2` (u32) =
/// the answer times the factor.
pub fn hit_event<E: FlowEnv>(
    env: &mut E,
    move_id: u32,
    idx: u32,
    mon_a: u32,
    mon_b: u32,
    out1: &mut [u8],
    out2: &mut [u8],
) {
    let r = env.call(
        0x0201_C0BC,
        &mut [Arg::V(move_id), Arg::V(idx), Arg::Out(&mut out2[0..4])],
    );
    out1[0..4].copy_from_slice(&r.to_le_bytes());
    msg_start(env, 0x2053);
    let a = mon_id(env, mon_a);
    arg_ec(env, 3, a);
    let b = mon_id(env, mon_b);
    arg_ec(env, 4, b);
    arg_a8(
        env,
        0x1F,
        u32::from_le_bytes(out1[0..4].try_into().unwrap()),
    );
    arg_a8(
        env,
        0x20,
        u32::from_le_bytes(out2[0..4].try_into().unwrap()),
    );
    arg_a8(env, 0x35, 1);
    msg_send(env, 0x59);
    let r1 = read_back(env, 0x1F);
    out1[0..4].copy_from_slice(&r1.to_le_bytes());
    let r2 = read_back(env, 0x20);
    out2[0..4].copy_from_slice(&r2.to_le_bytes());
    let r3 = read_back(env, 0x35);
    if r3 & 0xFF > 1 {
        let v = r2.wrapping_mul(r3 & 0xFF);
        out2[0..4].copy_from_slice(&v.to_le_bytes());
    }
    msg_end(env, 0x2062);
    if r1 == 8 {
        out1[0..4].copy_from_slice(&0u32.to_le_bytes());
    }
}

/// C: `FUN_overlay_d_93__021c312c`: event 0x5a message about three values and the reply slot 0x20
/// (`r6`); returns the `775c(0x20)` answer.
pub fn msg_312c<E: FlowEnv>(env: &mut E, mon: u32, r2: u32, r3: u32, r5: u32) -> u32 {
    msg_start(env, 0x20C1);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_ec(env, 3, r3);
    arg_ec(env, 0x1F, r2);
    arg_630(env, 0x20, r5);
    msg_send(env, 0x5A);
    let r = read_back(env, 0x20);
    msg_end(env, 0x20C8);
    r
}

/// C: `FUN_overlay_d_93__021c3008`: accuracy / effect gate for one hit: `312c`, `5d14`, then
/// `77b0`, `5a8c` and `785c` (hit) or `7820` (miss) inside the `0x209d` / `0x20a6` records;
/// `loud` = print the messages. Returns 1 when the hit landed.
#[allow(clippy::too_many_arguments)]
pub fn hit_gate<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    target_id: u32,
    mon: u32,
    p4: u32,
    p5: u32,
    p6: u32,
    p7: u32,
    p8: u32,
    loud: u32,
    p10: u32,
) -> u32 {
    let delta = msg_312c(&mut Lend::wc(env, w, cx), mon, p4, p6, p5);
    if env.callwc(
        0x021D_5D14,
        &mut [Arg::V(mon), Arg::V(p4), Arg::V(delta)],
        w,
        cx,
    ) == 0
    {
        if loud != 0 {
            env.callwc(
                0x021C_5A4C,
                &mut [Arg::P, Arg::V(mon), Arg::V(p4), Arg::V(delta)],
                w,
                cx,
            );
            w.mem[OFS_FLAGS_78A] |= 0x10;
        }
        return 0;
    }
    if env.callwc(0x021D_6D7C, &mut [Arg::V(mon)], w, cx) != 0 {
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        if target_id != id {
            if loud != 0 {
                cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0xD2, &[id, LIST_END]);
            }
            return 0;
        }
    }
    let mut ret = 1;
    let hit = env.callwc(
        0x021C_77B0,
        &mut [
            Arg::P,
            Arg::V(mon),
            Arg::V(p4),
            Arg::V(target_id),
            Arg::V(delta),
            Arg::V(p8),
        ],
        w,
        cx,
    );
    if hit == 0 {
        if loud != 0 {
            let old = bracket_open(w);
            env.callwc(0x021C_7820, &mut [Arg::P, Arg::V(mon), Arg::V(p8)], w, cx);
            bracket_close(w, old);
        }
        ret = 0;
    } else {
        env.callwc(
            0x021C_5A8C,
            &mut [
                Arg::P,
                Arg::V(mon),
                Arg::V(p4),
                Arg::V(delta),
                Arg::V(p7),
                Arg::V(p10),
            ],
            w,
            cx,
        );
        let old = bracket_open(w);
        env.callwc(
            0x021C_785C,
            &mut [
                Arg::P,
                Arg::V(target_id),
                Arg::V(mon),
                Arg::V(p4),
                Arg::V(delta),
            ],
            w,
            cx,
        );
        bracket_close(w, old);
    }
    ret
}

/// C: `FUN_overlay_d_93__021c2e64`: the multi-hit loop: for every hit of the move (`0201c020`)
/// runs `2f70`, then `3008` for the target (`answer` = 10: for each of the 6 slots), logs the hit
/// inside a `0x2037` record around `78a8`; returns 1 when any hit landed.
pub fn multi_hit<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    mon_a: u32,
    mon_b: u32,
    flag: u32,
) -> u32 {
    let mut any = 0;
    let id_a = mon_id(&mut Lend::wc(env, w, cx), mon_a);
    let counter = counter_next(w);
    let hits = env.callwc(0x0201_C020, &mut [Arg::V(rec_id(rec))], w, cx);
    let mut h = 0;
    if hits != 0 {
        loop {
            let mut landed = 0;
            let mut o1 = [0u8; 4];
            let mut o2 = [0u8; 4];
            hit_event(
                &mut Lend::wc(env, w, cx),
                rec_id(rec),
                h,
                mon_a,
                mon_b,
                &mut o1,
                &mut o2,
            );
            let n = u32::from_le_bytes(o1);
            let m = u32::from_le_bytes(o2);
            if n != 0 {
                if n == 10 {
                    let mut slot = 1u32;
                    loop {
                        let r =
                            hit_gate(env, w, cx, id_a, mon_b, slot, m, id_a, 0, counter, flag, 1);
                        if r != 0 {
                            landed = 1;
                        }
                        slot = (slot + 1) & 0xFF;
                        if slot >= 6 {
                            break;
                        }
                    }
                } else {
                    landed = hit_gate(env, w, cx, id_a, mon_b, n, m, id_a, 0, counter, flag, 1);
                }
                if landed != 0 {
                    let old = bracket_open(w);
                    env.callwc(
                        0x021C_78A8,
                        &mut [
                            Arg::P,
                            Arg::V(mon_b),
                            Arg::V(rec_id(rec)),
                            Arg::V(n),
                            Arg::V(m),
                        ],
                        w,
                        cx,
                    );
                    bracket_close(w, old);
                    any = 1;
                }
            }
            h += 1;
            if h >= hits {
                break;
            }
        }
    }
    any
}

/// C: `FUN_overlay_d_93__021c2dc4`: for every mon of the list `list`: `2e64` with flag 1 (the
/// per-turn flag in `p` on success).
pub fn multi_hit_each<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    rec: &[u8],
    mon: u32,
    list: u32,
) {
    list_begin(&mut Lend::wc(env, w, cx), 0x021C_2E1C, list);
    let mut el = list_next(&mut Lend::wc(env, w, cx), 0x021C_2E2C, list);
    while el != 0 {
        if multi_hit(env, w, cx, rec, mon, el, 1) != 0 {
            set_turn_flag(p);
        }
        el = list_next(&mut Lend::wc(env, w, cx), 0x021C_2E3C, list);
    }
}

/// C: `FUN_overlay_d_93__021c232c`: move-effect gate: `236c(move id)` then `25a4` unless
/// `5b68(mon_b)`.
pub fn effect_gate<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    mon_a: u32,
    mon_b: u32,
) {
    let mut out = [0u8; 4];
    let effect = effect_roll(
        &mut Lend::c(env, cx),
        w,
        rec_id(rec),
        mon_a,
        mon_b,
        &mut out,
    );
    if effect != 0 && env.callwc(0x021D_5B68, &mut [Arg::V(mon_b)], w, cx) == 0 {
        effect_round(
            env,
            w,
            cx,
            mon_a,
            mon_b,
            rec_id(rec),
            effect,
            u32::from_le_bytes(out),
            0,
        );
    }
}

/// C: `FUN_overlay_d_93__021c2cd8`: when `5b68(mon_b)` is clear and `2d10` passes: `2e64` (multi-hit
/// with flag 0).
pub fn multi_hit_if_rolled<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    mon_a: u32,
    mon_b: u32,
) {
    if env.callwc(0x021D_5B68, &mut [Arg::V(mon_b)], w, cx) == 0
        && event_51_roll(&mut Lend::c(env, cx), w, rec, mon_a, mon_b) != 0
    {
        multi_hit(env, w, cx, rec, mon_a, mon_b, 0);
    }
}

/// C: `FUN_overlay_d_93__021c1880`: by the move's category (`0201c1bc` of the record's id): 7 ->
/// `2cd8` on (a, a); 6 -> `2cd8` and 4 -> `232c` when `flag` is 0.
pub fn category_dispatch<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    mon_a: u32,
    mon_b: u32,
    flag: u32,
) {
    match env.callwc(0x0201_C1BC, &mut [Arg::V(rec_id(rec))], w, cx) {
        4 => {
            if flag == 0 {
                effect_gate(env, w, cx, rec, mon_a, mon_b);
            }
        }
        6 => {
            if flag == 0 {
                multi_hit_if_rolled(env, w, cx, rec, mon_a, mon_b);
            }
        }
        7 => multi_hit_if_rolled(env, w, cx, rec, mon_a, mon_a),
        _ => {}
    }
}

/// `work + 8`: the party container (address only).
const OFS_POKECON: usize = 0x8;
/// `work + 0x4B8`: helper struct (address only).
const OFS_HELPER_4B8: u32 = 0x4B8;
/// `work + 0x860`: value handed to `14c0` / `14d0`.
const OFS_VAL_860: usize = 0x860;
/// `work + 0x870`: value handed to `0a4c` / `11a4`.
const OFS_VAL_870: usize = 0x870;
/// `work + 0x789`: u8 (id of the attacker while the damage pipeline runs; 0x1f afterwards).
const OFS_BYTE_789: usize = 0x789;
/// `work + 0x77E`: u8 state of [`turn_end`].
const OFS_STATE_77E: usize = 0x77E;

fn w_u32(g: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(g[ofs..ofs + 4].try_into().unwrap())
}

fn w_u16(g: &[u8], ofs: usize) -> u32 {
    u16::from_le_bytes([g[ofs], g[ofs + 1]]) as u32
}

/// C: `FUN_overlay_d_93__021c3370`: 1 when `5b7c(mon) == 0` or `5b48(mon) != 0`.
pub fn mon_unavailable<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    if env.call(0x021D_5B7C, &mut [Arg::V(mon)]) == 0 {
        return 1;
    }
    (env.call(0x021D_5B48, &mut [Arg::V(mon)]) != 0) as u32
}

/// C: `FUN_overlay_d_93__021c3394`: when mon `mon` has flag 0xf: message 0x377 (if `loud`) and
/// flag 0x20 at `work + 0x78a`; returns the flag.
pub fn flag_f_gate<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    loud: u32,
) -> u32 {
    if env.callwc(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0xF)], w, cx) != 0 {
        if loud != 0 {
            let id = mon_id(&mut Lend::wc(env, w, cx), mon);
            cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0x377, &[id, LIST_END]);
            w.mem[OFS_FLAGS_78A] |= 0x20;
        }
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021c33e8`: command 0x4d for mon `mon` (when `3428` says its slot is not
/// 6), then `5b74(work, mon, r2, 1)`.
pub fn cmd_4d_then_5b74<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, r2: u32) {
    let id = mon_id(&mut Lend::c(env, cx), mon);
    let slot = env.callc(
        0x021C_3428,
        &mut [Arg::PAt(cx.w_addr + 0x1AB8), Arg::V(id)],
        cx,
    );
    if slot != 6 {
        cmd_queue::put_cmd(env, cx.q, 0x4D, &[slot, 0x263], &mut cx.g[cx.gb..]);
    }
    env.callc(
        0x021C_5B74,
        &mut [Arg::P, Arg::V(mon), Arg::V(r2), Arg::V(1)],
        cx,
    );
}

/// C: `FUN_overlay_d_93__021c334c`: the `3394` gate (`loud` = its message flag), then `33e8`
/// (returns 1) or 0.
pub fn gate_then_apply<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    r2: u32,
    loud: u32,
) -> u32 {
    if flag_f_gate(env, w, cx, mon, loud) == 0 {
        cmd_4d_then_5b74(&mut Lend::new(env, w), cx, mon, r2);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021c3184`: move-effect for every mon of the list `list`:
/// `0201bd28` / `0201bf58`, `7c70`, then `2e64` and `25a4` per mon (the per-turn flag in `p`).
pub fn effect_each_with_hits<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    rec: &[u8],
    mon: u32,
    list: u32,
) {
    let data_b = env.callwc(0x0201_BD28, &mut [Arg::V(rec_id(rec)), Arg::V(0xB)], w, cx);
    let ty = env.callwc(0x0201_BF58, &mut [Arg::V(rec_id(rec))], w, cx) & 0xFFFF;
    let mut o = [0u8; 4];
    env.callwc(
        0x021D_7C70,
        &mut [Arg::V(ty), Arg::V(mon), Arg::Out(&mut o)],
        w,
        cx,
    );
    let roll = u32::from_le_bytes(o);
    list_begin(&mut Lend::wc(env, w, cx), 0x021C_3244, list);
    let mut el = list_next(&mut Lend::wc(env, w, cx), 0x021C_3254, list);
    while el != 0 {
        if multi_hit(env, w, cx, rec, mon, el, 1) != 0 {
            set_turn_flag(p);
        }
        if effect_round(env, w, cx, mon, el, rec_id(rec), data_b, roll, 1) != 0 {
            set_turn_flag(p);
        }
        el = list_next(&mut Lend::wc(env, w, cx), 0x021C_3264, list);
    }
}

/// C: `FUN_overlay_d_93__021c3274`: for every mon of the list `list`: `3370` / `79e0` + `334c`
/// (command 0x5b message 0x183, per-turn flag), else the `5b48` message 0x37d.
pub fn each_apply_79e0<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    r2: u32,
    list: u32,
) {
    list_begin(&mut Lend::wc(env, w, cx), 0x021C_331C, list);
    let mut el = list_next(&mut Lend::wc(env, w, cx), 0x021C_332C, list);
    while el != 0 {
        let id = mon_id(&mut Lend::wc(env, w, cx), el);
        if mon_unavailable(&mut Lend::wc(env, w, cx), el) == 0 {
            let s = env.callwc(0x021C_79E0, &mut [Arg::P, Arg::V(r2), Arg::V(el)], w, cx) & 0xFFFF;
            if gate_then_apply(env, w, cx, el, s, 1) != 0 {
                set_turn_flag(p);
                cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0x183, &[id, LIST_END]);
            }
        } else if env.callwc(0x021D_5B48, &mut [Arg::V(el)], w, cx) != 0 {
            cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0x37D, &[id, LIST_END]);
        }
        el = list_next(&mut Lend::wc(env, w, cx), 0x021C_333C, list);
    }
}

/// C: `FUN_overlay_d_93__021c3b8c`: 0 when `r1 > 4`, or when `r1 == ef884()` and (`r2 != 0xff`
/// or `ef894() == 0xff`); else 1.
pub fn slot_gate<E: FlowEnv>(env: &mut E, r1: u32, r2: u32) -> u32 {
    if r1 > 4 {
        return 0;
    }
    let cur = env.call(0x021E_F884, &mut []);
    if r1 == cur && (r2 != 0xFF || env.call(0x021E_F894, &mut []) == 0xFF) {
        return 0;
    }
    1
}

/// C: `FUN_overlay_d_93__021c3be0`: the `0x2303` record bracket around `7998`.
pub fn record_bracket_7998<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, r1: u32) {
    let old = bracket_open(w);
    env.callw(0x021C_7998, &mut [Arg::P, Arg::V(r1)], w);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c3bb8`: `ef8a4(a, b)`, command 0x3f (a, b), `3be0`.
pub fn cmd_3f<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, a: u32, b: u32) {
    env.callwc(0x021E_F8A4, &mut [Arg::V(a), Arg::V(b)], w, cx);
    cmd_queue::put_cmd(
        &mut Lend::new(env, w),
        cx.q,
        0x3F,
        &[a, b],
        &mut cx.g[cx.gb..],
    );
    record_bracket_7998(&mut Lend::c(env, cx), w, a);
}

/// C: `FUN_overlay_d_93__021c3b68`: `3b8c` then `3bb8`; returns `3b8c`.
pub fn slot_gate_apply<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    r1: u32,
    r2: u32,
) -> u32 {
    if slot_gate(&mut Lend::wc(env, w, cx), r1, r2) != 0 {
        cmd_3f(env, w, cx, r1, r2);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021c3c10`: event 0x7c message about value `r1` and mon `mon`; returns the
/// `775c(0x24)` answer (u8).
pub fn event_7c<E: FlowEnv>(env: &mut E, r1: u32, mon: u32) -> u32 {
    msg_start(env, 0x2317);
    arg_ec(env, 0x39, r1);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_a8(env, 0x24, 0);
    msg_send(env, 0x7C);
    let r = read_back(env, 0x24);
    msg_end(env, 0x2317 + 6);
    r & 0xFF
}

/// C: `FUN_overlay_d_93__021c3ccc`: event 0x9e message about mon `mon` and `r2`.
pub fn event_9e<E: FlowEnv>(env: &mut E, mon: u32, r2: u32) {
    msg_start(env, 0x2360);
    let id = mon_id(env, mon);
    arg_a8(env, 3, id);
    arg_a8(env, 0x12, r2);
    msg_send(env, 0x9E);
    msg_end(env, 0x2360 + 4);
}

/// C: `FUN_overlay_d_93__021c3aa8`: effect by move: `0201c184(move)` non-zero -> `3c10` + `3b68`
/// (per-turn flag) or `5718`; else the `0x22c8` record around `3ccc` with the `88e4` gate.
pub fn effect_by_move<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    rec: &[u8],
    mon: u32,
) {
    let c = env.callwc(0x0201_C184, &mut [Arg::V(rec_id(rec))], w, cx) as u8;
    if c == 0 {
        let old = bracket_open(w);
        event_9e(&mut Lend::wc(env, w, cx), mon, rec_id(rec));
        let r = env.callwc(0x021C_88E4, &mut [Arg::P], w, cx);
        bracket_close(w, old);
        if r == 2 {
            set_turn_flag(p);
        } else {
            env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
        }
    } else {
        let c3 = event_7c(&mut Lend::wc(env, w, cx), c as i8 as i32 as u32, mon) as u8;
        let r2 = (c3 as i8 as i32 + 5) as u32;
        if slot_gate_apply(env, w, cx, c as i8 as i32 as u32, r2) == 0 {
            env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
            return;
        }
        set_turn_flag(p);
    }
}

/// C: `FUN_overlay_d_93__021c3c60`: (varargs) `ef8c8(r1, r2)` -> command 0x21, or (`r3 != 0` and
/// `8a98(r2) != 0x1f`) -> `ef8f8` + command 0x22; returns 1 when a command was queued.
pub fn cmd_21_22<E: FlowEnv>(env: &mut E, cx: &mut Cx, r1: u32, r2: u32, r3: u32) -> u32 {
    if env.callc(0x021E_F8C8, &mut [Arg::V(r1), Arg::V(r2)], cx) != 0 {
        cmd_queue::put_cmd(env, cx.q, 0x21, &[r1 & 0xFF, r2], &mut cx.g[cx.gb..]);
        return 1;
    }
    if r3 != 0 {
        let v = env.callc(0x021E_8A98, &mut [Arg::V(r2)], cx);
        if v != 0x1F {
            env.callc(0x021E_F8F8, &mut [Arg::V(r1), Arg::V(v)], cx);
            cmd_queue::put_cmd(env, cx.q, 0x22, &[r1 & 0xFF, v], &mut cx.g[cx.gb..]);
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021c36a0`: commands 3, 0x32, 0x34 for mon `mon` (`5fb8` first); `rec` =
/// the move record, `flag` a byte.
pub fn cmds_3_32_34<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32, rec: &[u8], flag: u32) {
    let id = mon_id(&mut Lend::c(env, cx), mon);
    env.callc(0x021D_5FB8, &mut [Arg::V(mon)], cx);
    cmd_queue::put_cmd(env, cx.q, 3, &[id], &mut cx.g[cx.gb..]);
    cmd_queue::put_cmd(
        env,
        cx.q,
        0x32,
        &[id, flag & 0xFF, rec_id(rec), flag, flag],
        &mut cx.g[cx.gb..],
    );
    cmd_queue::put_cmd(env, cx.q, 0x34, &[id], &mut cx.g[cx.gb..]);
}

/// C: `FUN_overlay_d_93__021c36e8`: `604c`, command 0x32 for mon `mon` (move record `rec`, flag
/// `flag`), then `16f8` with the mode `mode`.
#[allow(clippy::too_many_arguments)]
pub fn cmd_32_then_16f8<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    rec: &[u8],
    flag: u32,
    mode: u32,
    r6: u32,
) {
    env.callwc(
        0x021C_604C,
        &mut [Arg::P, Arg::V(mon), Arg::V(r6 & 0xFFFF)],
        w,
        cx,
    );
    let id = mon_id(&mut Lend::wc(env, w, cx), mon);
    cmd_queue::put_cmd(
        env,
        cx.q,
        0x32,
        &[id, flag & 0xFF, rec_id(rec)],
        &mut cx.g[cx.gb..],
    );
    msg_16f8(&mut Lend::hv(env, vec![&mut *cx.g]), w, cx.q, mon, mode);
}

/// C: `FUN_overlay_d_93__021c41c0`: commands 0x28 (mon id) / 0x52 (`4208` id), `5d88`, `6d70` for
/// mon `mon`.
pub fn cmds_28_52<E: FlowEnv>(env: &mut E, cx: &mut Cx, mon: u32) {
    let id = mon_id(&mut Lend::c(env, cx), mon);
    let v = env.callc(
        0x021C_4208,
        &mut [Arg::PAt(cx.w_addr + 0x1AB8), Arg::V(id)],
        cx,
    );
    env.callc(0x021C_5D88, &mut [Arg::P, Arg::V(mon), Arg::V(0x31A)], cx);
    env.callc(0x021D_6D70, &mut [Arg::V(mon)], cx);
    cmd_queue::put_cmd(env, cx.q, 0x28, &[id], &mut cx.g[cx.gb..]);
    cmd_queue::put_cmd(env, cx.q, 0x52, &[v], &mut cx.g[cx.gb..]);
}

/// C: `FUN_overlay_d_93__021c3720`: commands 0x34 / 0x32 for mon `mon`, then `41c0`; returns
/// `6d90(mon)` (u16).
pub fn cmds_34_32_41c0<E: FlowEnv>(
    env: &mut E,
    cx: &mut Cx,
    mon: u32,
    rec: &[u8],
    flag: u32,
) -> u32 {
    let id = mon_id(&mut Lend::c(env, cx), mon);
    let r = env.callc(0x021D_6D90, &mut [Arg::V(mon)], cx) & 0xFFFF;
    cmd_queue::put_cmd(env, cx.q, 0x34, &[id], &mut cx.g[cx.gb..]);
    cmd_queue::put_cmd(
        env,
        cx.q,
        0x32,
        &[id, flag & 0xFF, rec_id(rec)],
        &mut cx.g[cx.gb..],
    );
    cmds_28_52(env, cx, mon);
    r
}

/// C: `FUN_overlay_d_93__021c4268`: message 0x4a / 0x4b / 0x4c about mon `a`, mon `b` and the
/// move record `rec` (u16 +0/+2, u8 +6, u32 +8) and four values.
#[allow(clippy::too_many_arguments)]
pub fn msg_4268<E: FlowEnv>(
    env: &mut E,
    a: u32,
    b: u32,
    rec: &[u8],
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
) {
    let idb = mon_id(env, b);
    msg_start(env, 0x2477);
    let ida = mon_id(env, a);
    arg_ec(env, 3, ida);
    arg_ec(env, 4, idb);
    arg_ec(env, 0x14, u16::from_le_bytes([rec[2], rec[3]]) as u32);
    arg_ec(env, 0x12, rec_id(rec));
    arg_ec(env, 0x38, r5);
    arg_ec(env, 0x16, rec[6] as u32);
    arg_ec(
        env,
        0x1A,
        u32::from_le_bytes(rec[8..12].try_into().unwrap()),
    );
    arg_ec(env, 0x32, r6);
    arg_ec(env, 0x45, r7);
    arg_ec(env, 0x46, r8);
    arg_630(env, 0x47, 0);
    msg_send(env, 0x4A);
    msg_send(env, 0x4B);
    msg_send(env, 0x4C);
    msg_end(env, 0x2477 + 0x11);
}

/// C: `FUN_overlay_d_93__021c4218`: the `0x245e` record bracket around `4268`.
#[allow(clippy::too_many_arguments)]
pub fn record_bracket_4268<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    a: u32,
    b: u32,
    rec: &[u8],
    r5: u32,
    r6: u32,
    r7: u32,
    r8: u32,
) {
    let old = bracket_open(w);
    msg_4268(&mut Lend::new(env, w), a, b, rec, r5, r6, r7, r8);
    bracket_close(w, old);
}

/// C: `FUN_overlay_d_93__021c4034`: pays `7b74(mon, 4)` when the mon's HP (field 0xd) is above it
/// (`5b74`, `195c`, `6d5c`, commands 0x27 / 0x51, message 0x311) and returns 1; otherwise `5dac`
/// and 0 (also 0, with message 0x314, when `6d7c(mon)`).
pub fn pay_quarter_hp<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, mon: u32) -> u32 {
    if env.callwc(0x021D_6D7C, &mut [Arg::V(mon)], w, cx) == 0 {
        let cost = env.callwc(0x021D_7B74, &mut [Arg::V(mon), Arg::V(4)], w, cx);
        let hp = mon_get(&mut Lend::wc(env, w, cx), mon, 0xD);
        if (cost as i32) < hp as i32 {
            let id = mon_id(&mut Lend::wc(env, w, cx), mon);
            let slot = env.callwc(
                0x021C_40F4,
                &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(id)],
                w,
                cx,
            );
            if slot != 6 {
                env.callwc(
                    0x021C_5B74,
                    &mut [Arg::P, Arg::V(mon), Arg::V(cost.wrapping_neg()), Arg::V(1)],
                    w,
                    cx,
                );
                record_bracket_199c(&mut Lend::c(env, cx), w, mon, 1);
                env.callwc(
                    0x021D_6D5C,
                    &mut [Arg::V(mon), Arg::V(cost & 0xFFFF)],
                    w,
                    cx,
                );
                let id = mon_id(&mut Lend::wc(env, w, cx), mon);
                cmd_queue::put_cmd(
                    &mut Lend::new(env, w),
                    cx.q,
                    0x27,
                    &[id, cost & 0xFFFF],
                    &mut cx.g[cx.gb..],
                );
                cmd_queue::put_cmd(
                    &mut Lend::new(env, w),
                    cx.q,
                    0x51,
                    &[slot],
                    &mut cx.g[cx.gb..],
                );
                let id = mon_id(&mut Lend::wc(env, w, cx), mon);
                cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0x311, &[id, LIST_END]);
                return 1;
            }
        }
        env.callwc(
            0x021C_5DAC,
            &mut [Arg::P, Arg::V(0x7B), Arg::V(0), Arg::V(0)],
            w,
            cx,
        );
    } else {
        env.callwc(
            0x021C_5D88,
            &mut [Arg::P, Arg::V(mon), Arg::V(0x314)],
            w,
            cx,
        );
    }
    0
}

/// C: `FUN_overlay_d_93__021c4104`: (varargs) the per-target damage side effects: command 0x54,
/// `5d88`, `5840` / `59c4` (in/out slots), `6d9c` (rewrites the u16 slot), `41c0`, `1dcc`, `1880`,
/// `4218`; `a` = r1, `b` = the target (r2), `val` = r3 (u16 slot), `s0` / `s1` = the caller's stack
/// words 0 / 1, `rec` = stack word 2; returns the u16 slot.
#[allow(clippy::too_many_arguments)]
pub fn apply_target_effects<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    a: u32,
    b: u32,
    val: u32,
    s0: u32,
    s1: u32,
    rec: &[u8],
) -> u32 {
    let id = mon_id(&mut Lend::wc(env, w, cx), b);
    // the 4th word of command 0x54 is the (zero) stack word after the caller's 3 arguments
    cmd_queue::put_cmd(
        env,
        cx.q,
        0x54,
        &[id, s0 & 0xFF, rec_id(rec), 0],
        &mut cx.g[cx.gb..],
    );
    env.callwc(0x021C_5D88, &mut [Arg::P, Arg::V(b), Arg::V(0x317)], w, cx);
    // the original passes its stack words: `&s0` (an argument slot), `&b` (a local copy) and `&s1`;
    // `b` is read back from the local afterwards
    let mut s0w = s0.to_le_bytes();
    let mut bw = b.to_le_bytes();
    let mut s1w = s1.to_le_bytes();
    env.callwc(
        0x021C_5840,
        &mut [
            Arg::P,
            Arg::V(1),
            Arg::PBuf(&mut s0w),
            Arg::PBuf(&mut bw),
            Arg::V(0),
        ],
        w,
        cx,
    );
    env.callwc(
        0x021C_59C4,
        &mut [
            Arg::P,
            Arg::V(1),
            Arg::PBuf(&mut bw),
            Arg::PBuf(&mut s1w),
            Arg::V(0),
        ],
        w,
        cx,
    );
    let b = u32::from_le_bytes(bw);
    let mut slot = val.to_le_bytes();
    if env.callwc(0x021D_6D9C, &mut [Arg::V(b), Arg::Out(&mut slot)], w, cx) != 0 {
        cmds_28_52(&mut Lend::new(env, w), cx, b);
    }
    let v = u32::from_le_bytes(slot) & 0xFFFF;
    category8_followup(env, w, cx, rec, a, b, v);
    category_dispatch(env, w, cx, rec, a, b, 1);
    record_bracket_4268(&mut Lend::c(env, cx), w, a, b, rec, s0, v, s1 & 0xFF, 1);
    v
}

/// C: `FUN_overlay_d_93__021c43f0`: for every mon the iterator (`bd2b4` / `bd34c`) yields:
/// `444c(list, mon)`; `443c` before and `445c(list, work)` after.
pub fn collect_mons<E: FlowEnv>(env: &mut E, w_addr: u32, list: u32) {
    env.call(0x021C_443C, &mut [Arg::V(list)]);
    // the iterator state lives in the caller's frame
    let mut it = [0u8; 4];
    env.call(0x021B_D2B4, &mut [Arg::PBuf(&mut it), Arg::P]);
    loop {
        let mut o = [0u8; 4];
        let more = env.call(
            0x021B_D34C,
            &mut [Arg::PBuf(&mut it), Arg::P, Arg::Out(&mut o)],
        );
        if more == 0 {
            break;
        }
        env.call(
            0x021C_444C,
            &mut [Arg::V(list), Arg::V(u32::from_le_bytes(o))],
        );
    }
    env.call(0x021C_445C, &mut [Arg::V(list), Arg::V(w_addr)]);
}

/// C: `FUN_overlay_d_93__021c4304`: message 0xa0 / 0xa1 about the list `list` (`43c0` flag, `43d0`
/// count, `43e0` mons); `out` (u32) = the `775c(0x51)` answer; returns the `88e4` gate.
pub fn list_message<E: FlowEnv>(
    env: &mut E,
    rec: &[u8],
    mon: u32,
    list: u32,
    out: &mut [u8],
) -> u32 {
    msg_start(env, 0x249A);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    let flag = env.call(0x021C_43C0, &mut [Arg::V(list)]) as u8;
    let n = env.call(0x021C_43D0, &mut [Arg::V(list)]) & 0xFF;
    arg_ec(env, 5, n);
    arg_630(env, 0x51, 1);
    let mut i = 0;
    if n != 0 {
        loop {
            let m = env.call(0x021C_43E0, &mut [Arg::V(list), Arg::V(i)]);
            let id = mon_id(env, m);
            arg_ec(env, i + 6, id);
            i = (i + 1) & 0xFF;
            if i >= n {
                break;
            }
        }
    }
    arg_ec(env, 0x12, rec_id(rec));
    if flag == 0 {
        msg_send(env, 0xA1);
    } else if n != 0 {
        msg_send(env, 0xA0);
    }
    let r = read_back(env, 0x51);
    out[0..4].copy_from_slice(&r.to_le_bytes());
    msg_end(env, 0x24BA);
    (env.call(0x021C_88E4, &mut [Arg::P]) != 0) as u32
}

/// C: `FUN_overlay_d_93__021c3e0c`: Skill-Swap-style exchange between mon `mon` and the first mon
/// of the list `list` (`3fac`): BattleMon field 0x10 of both, commands 0x4a / 0x58, records
/// `0x23b4` around `7b3c` / `7b7c`, `66d0`, `8564`; Klutz (0x67) / Unnerve (0x7f) re-checks via
/// `195c` / `3fdc`; message 0x47 when refused.
pub fn swap_fields<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    mon: u32,
    list: u32,
) {
    let other = env.callwc(0x021C_3FAC, &mut [Arg::V(list), Arg::V(0)], w, cx);
    let a = mon_get(&mut Lend::wc(env, w, cx), mon, 0x10);
    let b = mon_get(&mut Lend::wc(env, w, cx), other, 0x10);
    if a == b
        || env.callwc(0x021C_3FBC, &mut [Arg::V(a & 0xFFFF)], w, cx) != 0
        || env.callwc(0x021C_3FCC, &mut [Arg::V(b & 0xFFFF)], w, cx) != 0
    {
        cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5A, 0x47, &[LIST_END]);
        return;
    }
    let ida = mon_id(&mut Lend::wc(env, w, cx), mon);
    let idb = mon_id(&mut Lend::wc(env, w, cx), other);
    set_turn_flag(p);
    cmd_queue::put_cmd(
        env,
        cx.q,
        0x4A,
        &[ida, idb, b & 0xFFFF, a & 0xFFFF],
        &mut cx.g[cx.gb..],
    );
    cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0x1FC, &[ida, LIST_END]);
    let old = bracket_open(w);
    env.callwc(
        0x021C_7B3C,
        &mut [Arg::P, Arg::V(ida), Arg::V(a & 0xFFFF), Arg::V(b & 0xFFFF)],
        w,
        cx,
    );
    env.callwc(
        0x021C_7B3C,
        &mut [Arg::P, Arg::V(idb), Arg::V(b & 0xFFFF), Arg::V(a & 0xFFFF)],
        w,
        cx,
    );
    bracket_close(w, old);
    env.callwc(0x021D_66D0, &mut [Arg::V(mon), Arg::V(b)], w, cx);
    env.callwc(0x021D_66D0, &mut [Arg::V(other), Arg::V(a)], w, cx);
    env.callwc(0x021D_8564, &mut [Arg::V(mon), Arg::V(other)], w, cx);
    cmd_queue::put_cmd(
        &mut Lend::new(env, w),
        cx.q,
        0x58,
        &[ida],
        &mut cx.g[cx.gb..],
    );
    cmd_queue::put_cmd(
        &mut Lend::new(env, w),
        cx.q,
        0x58,
        &[idb],
        &mut cx.g[cx.gb..],
    );
    if a != b {
        let old = bracket_open(w);
        env.callwc(0x021C_7B7C, &mut [Arg::P, Arg::V(ida)], w, cx);
        env.callwc(0x021C_7B7C, &mut [Arg::P, Arg::V(idb)], w, cx);
        bracket_close(w, old);
    }
    for (m, ability) in [(mon, a), (other, b)] {
        if env.callwc(0x021D_62C4, &mut [Arg::V(m), Arg::V(0x10)], w, cx) == 0 {
            if ability == 0x67 {
                record_bracket_199c(&mut Lend::c(env, cx), w, m, 0);
            }
            if ability == 0x7F {
                notify_others(&mut Lend::c(env, cx), w, m);
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021c3d08`: by the move id (u16 at +0): 0xa4 -> `4034` (+ per-turn flags),
/// 285 -> `3e0c`, else the `0x237f` record around `4304` with the `88e4` gate and message 0x47.
pub fn effect_by_id<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    rec: &[u8],
    mon: u32,
    list: u32,
) {
    if env.callwc(0x021C_3DFC, &mut [Arg::V(list)], w, cx) != 0 {
        return;
    }
    if rec_id(rec) == 0xA4 {
        if pay_quarter_hp(env, w, cx, mon) != 0 && p[5] & 1 == 0 {
            p[5] = p[5] & 0xFE | 1;
            p[5] |= 2;
        }
    } else {
        if rec_id(rec) == 0x11D {
            swap_fields(env, w, cx, p, mon, list);
            return;
        }
        let old = bracket_open(w);
        let mut gate = env.callwc(0x021C_88E4, &mut [Arg::P], w, cx);
        let mut out = [0u8; 4];
        if list_message(&mut Lend::wc(env, w, cx), rec, mon, list, &mut out) != 0 {
            gate = env.callwc(0x021C_88E4, &mut [Arg::P], w, cx);
            if gate == 2 {
                set_turn_flag(p);
            }
        }
        if gate < 2 && u32::from_le_bytes(out) != 0 && w.mem[OFS_FLAGS_78A] & 0x10 == 0 {
            cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5A, 0x47, &[LIST_END]);
        }
        bracket_close(w, old);
    }
}

/// C: `FUN_overlay_d_93__021c39cc`: 1 unless both `8588(main)` and `86b8(main)` are 0.
pub fn gate_39cc<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    let a = env.callwr(0x021B_8588, &mut [Arg::V(w.u32_at(OFS_MAIN))], w);
    let b = env.callwr(0x021B_86B8, &mut [Arg::V(w.u32_at(OFS_MAIN))], w);
    if a != 0 {
        return 1;
    }
    (b != 0) as u32
}

/// C: `FUN_overlay_d_93__021c3a4c`: event 0x8b message about mon `mon`; returns the `775c(0x41)`
/// answer.
pub fn event_8b<E: FlowEnv>(env: &mut E, w_addr: u32, mon: u32) -> u32 {
    msg_start(env, 0x22AD);
    arg_630(env, 0x41, 0);
    let id = mon_id(env, mon);
    arg_ec(env, 4, id);
    msg_send(env, 0x8B);
    env.call(0x021C_3A98, &mut [Arg::V(w_addr), Arg::V(mon)]);
    let r = read_back(env, 0x41);
    msg_end(env, 0x22AD + 6);
    r
}

/// C: `FUN_overlay_d_93__021c39f4`: picks a random slot of the party side described by the struct
/// `side` (count at `+4` via `9b94`, first slot byte `+0xb`) whose `5b7c` answers non-zero;
/// returns the slot or 0xffffffff.
pub fn random_party_slot<E: FlowEnv>(env: &mut E, w: &SvFlowWork, side: &[u8]) -> u32 {
    let side4 = u32::from_le_bytes(side[4..8].try_into().unwrap());
    let count = env.callwr(0x021B_9B94, &mut [Arg::V(side4)], w);
    let mut slot = env.callwr(
        0x021B_99C0,
        &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(side[0xB] as u32)],
        w,
    );
    let mut found = [0u8; 8];
    let mut n = 0usize;
    while slot < count {
        let mon = env.callwr(0x021B_9C00, &mut [Arg::V(side4), Arg::V(slot & 0xFF)], w);
        if env.callwr(0x021D_5B7C, &mut [Arg::V(mon)], w) != 0 {
            found[n] = slot as u8;
            n += 1;
        }
        slot += 1;
    }
    if n == 0 {
        return 0xFFFF_FFFF;
    }
    let i = env.callwr(0x021D_78AC, &mut [Arg::V(n as u32)], w) as usize;
    found.get(i).copied().unwrap_or(0) as u32
}

/// C: `FUN_overlay_d_93__021c3810`: one mon's effect: reasons `39cc` / `8d8c` / `5b68` / `62c4` /
/// `69ac` gate it, record `0x221c` around `3a4c`, then either `39f4` + `bdd7c` / `bd9d4`
/// (`mode == 1`) or `8d08` + `8244` + `bdd7c`; `out5` (u32) = 1 when `3a4c` and `88e4` both said
/// so. `side` = the record `bb85c` returns (work + 0x1000).
#[allow(clippy::too_many_arguments)]
pub fn single_mon_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon_a: u32,
    mon_b: u32,
    mode: u32,
    out5: &mut [u8],
    r6: u32,
    r7: u32,
    r8: u32,
) -> u32 {
    out5[0..4].copy_from_slice(&0u32.to_le_bytes());
    let gate = if mode == 0 {
        gate_39cc(&mut Lend::c(env, cx), w)
    } else {
        1
    };
    if gate == 2 {
        return 0;
    }
    let main = w.u32_at(OFS_MAIN);
    let pokecon = w.u32_at(OFS_POKECON);
    let id = mon_id(&mut Lend::wc(env, w, cx), mon_b);
    let slot = env.callwc(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(id)],
        w,
        cx,
    );
    if slot == 6 {
        return 0;
    }
    if env.callwc(0x021D_5B68, &mut [Arg::V(mon_b)], w, cx) != 0 {
        return 0;
    }
    if env.callwc(0x021D_62C4, &mut [Arg::V(mon_b), Arg::V(0x21)], w, cx) != 0 {
        return 0;
    }
    let st = env.callwc(0x021D_69AC, &mut [Arg::V(mon_b), Arg::V(4)], w, cx);
    let kind = if st == 0 || (st.wrapping_sub(1) & 0xFF) > 0x17 {
        0x1F
    } else {
        st.wrapping_sub(1) & 0xFF
    };
    if kind != 0x1F {
        return 0;
    }
    let old = bracket_open(w);
    let c = event_8b(&mut Lend::new(env, w), cx.w_addr, mon_b) as u8;
    if c != 0 && env.callwc(0x021C_88E4, &mut [Arg::P], w, cx) != 0 {
        out5[0..4].copy_from_slice(&1u32.to_le_bytes());
    }
    bracket_close(w, old);
    if c != 0 {
        return 0;
    }
    let mut b1 = [0u8; 1];
    let mut b2 = [0u8; 1];
    env.callwc(
        0x021B_8E28,
        &mut [
            Arg::V(main),
            Arg::V(slot),
            Arg::Out(&mut b1),
            Arg::Out(&mut b2),
        ],
        w,
        cx,
    );
    if gate == 1 {
        let rec_ptr = env.callwc(
            0x021B_B85C,
            &mut [Arg::V(w.u32_at(0)), Arg::V(b1[0] as u32)],
            w,
            cx,
        );
        let side_ofs = rec_ptr.wrapping_sub(cx.w_addr) as usize;
        let side: Vec<u8> = w.mem[side_ofs..side_ofs + 0x10].to_vec();
        let pick = random_party_slot(&mut Lend::c(env, cx), w, &side);
        if (pick as i32) < 0 {
            return 0;
        }
        let switched = env.callwc(
            0x021B_990C,
            &mut [Arg::V(pokecon), Arg::V(b1[0] as u32), Arg::V(pick & 0xFF)],
            w,
            cx,
        );
        let id = mon_id(&mut Lend::wc(env, w, cx), switched);
        env.callwc(
            0x021B_DD7C,
            &mut [Arg::P, Arg::V(mon_b), Arg::V(r6 & 0xFFFF)],
            w,
            cx,
        );
        if r8 != 0 {
            // r1 still holds `r8` (the original reloads it for the test)
            env.callwc(0x021C_99A0, &mut [Arg::P, Arg::PAt(r8)], w, cx);
        }
        env.callwc(
            0x021B_D9D4,
            &mut [
                Arg::P,
                Arg::V(b1[0] as u32),
                Arg::V(b2[0] as u32),
                Arg::V(pick & 0xFF),
                Arg::V(0),
            ],
            w,
            cx,
        );
        cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0x34D, &[id, LIST_END]);
        env.callwc(0x021B_DA00, &mut [Arg::P], w, cx);
    } else {
        let id = mon_id(&mut Lend::wc(env, w, cx), mon_a);
        let client = env.callwc(0x021B_8D08, &mut [Arg::V(id)], w, cx);
        if r7 == 0 {
            let a = mon_get(&mut Lend::wc(env, w, cx), mon_a, 0xF) & 0xFF;
            let b = mon_get(&mut Lend::wc(env, w, cx), mon_b, 0xF) & 0xFF;
            if a < b {
                return 0;
            }
        }
        env.callwc(
            0x021D_8244,
            &mut [
                Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_4B8)),
                Arg::V(client),
            ],
            w,
            cx,
        );
        env.callwc(
            0x021B_DD7C,
            &mut [Arg::P, Arg::V(mon_b), Arg::V(r6 & 0xFFFF)],
            w,
            cx,
        );
        w.set_u32_at(0x14, 5);
    }
    1
}

/// C: `FUN_overlay_d_93__021c376c`: for every mon of the list `list`: `3810` (the full per-mon
/// effect); the per-turn flag on success, else `5718` unless `3810` flagged it.
pub fn each_single_mon_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    _r2: u32,
    mon: u32,
    list: u32,
) {
    let mut local = [0u8; 4];
    list_begin(&mut Lend::wc(env, w, cx), 0x021C_37E0, list);
    let mut el = list_next(&mut Lend::wc(env, w, cx), 0x021C_37F0, list);
    while el != 0 {
        if single_mon_effect(env, w, cx, mon, el, 0, &mut local, 0, 0, 0) == 0 {
            if u32::from_le_bytes(local) == 0 {
                env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
            }
        } else {
            set_turn_flag(p);
        }
        el = list_next(&mut Lend::wc(env, w, cx), 0x021C_3800, list);
    }
}

/// C: `FUN_overlay_d_93__021c3438`: damage side effects for every mon of the list `list`: the
/// immunity predicates (`bf324` / `6948` / `bfa00`) -> `572c`, the level / table gate -> `5774`,
/// else inside a `0x216b` record `6a9c`, the damage application (`3720` / `36a0` / `36e8`),
/// `1828`, `4218`, `1c88` and `4f64`.
pub fn damage_each<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    rec: &[u8],
    mon_a: u32,
    list: u32,
) {
    let mut idx = 0;
    let level_a = mon_get(&mut Lend::wc(env, w, cx), mon_a, 0xF) as u8;
    loop {
        let bpp = env.callwc(0x021C_3660, &mut [Arg::V(list), Arg::V(idx)], w, cx);
        idx += 1;
        if bpp == 0 {
            break;
        }
        if env.callwc(0x021D_5B68, &mut [Arg::V(bpp)], w, cx) != 0 {
            continue;
        }
        let id_b = mon_id(&mut Lend::wc(env, w, cx), bpp);
        let blocked = (env.callwc(
            0x021B_F324,
            &mut [Arg::P, Arg::V(mon_a), Arg::V(bpp)],
            w,
            cx,
        ) == 0
            && env.callwc(
                0x021C_6948,
                &mut [Arg::P, Arg::V(mon_a), Arg::V(bpp)],
                w,
                cx,
            ) != 0)
            || env.callwc(
                0x021B_FA00,
                &mut [Arg::P, Arg::V(mon_a), Arg::V(bpp), Arg::V(rec_id(rec))],
                w,
                cx,
            ) != 0;
        if blocked {
            env.callwc(0x021C_572C, &mut [Arg::P, Arg::V(bpp)], w, cx);
            continue;
        }
        let level_b = mon_get(&mut Lend::wc(env, w, cx), bpp, 0xF) as u8;
        let mut tbl = 0;
        let skip = level_a < level_b || {
            tbl = super::events::table_get(&w.mem[0x1F88..], id_b);
            tbl == 0
        };
        if skip {
            env.callwc(0x021C_5774, &mut [Arg::P, Arg::V(bpp)], w, cx);
            continue;
        }
        let old = bracket_open(w);
        let found = env.callwc(
            0x021C_6A9C,
            &mut [Arg::P, Arg::V(mon_a), Arg::V(bpp), Arg::V(rec_id(rec))],
            w,
            cx,
        );
        if found == 0 {
            if env.callwc(0x021C_88E4, &mut [Arg::P], w, cx) == 0 {
                env.callwc(0x021C_572C, &mut [Arg::P, Arg::V(bpp)], w, cx);
            }
        } else {
            let ida = mon_id(&mut Lend::wc(env, w, cx), mon_a);
            let client = env.callwc(
                0x021C_3670,
                &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(ida)],
                w,
                cx,
            );
            let mut hp = [0u8; 4];
            let v = mon_get(&mut Lend::wc(env, w, cx), bpp, 0xD) as u16;
            hp[0..2].copy_from_slice(&v.to_le_bytes());
            let class = env.callwc(0x021D_7AD4, &mut [Arg::V(tbl)], w, cx) & 0xFF;
            let mut fainted = 0;
            let mut chained = false;
            set_turn_flag(p);
            if env.callwc(0x021D_6D7C, &mut [Arg::V(bpp)], w, cx) == 0 {
                let r = hp_gate(&mut Lend::wc(env, w, cx), mon_a, bpp, 1, &mut hp);
                if r == 0 {
                    cmds_3_32_34(&mut Lend::new(env, w), cx, bpp, rec, class);
                } else {
                    let slot = u16::from_le_bytes([hp[0], hp[1]]) as u32;
                    cmd_32_then_16f8(env, w, cx, bpp, rec, class, r, slot);
                    let slot = u16::from_le_bytes([hp[0], hp[1]]) as u32;
                    env.callwc(
                        0x021C_3690,
                        &mut [Arg::V(list), Arg::V(bpp), Arg::V(slot), Arg::V(0)],
                        w,
                        cx,
                    );
                    chained = true;
                }
            } else {
                let v = cmds_34_32_41c0(&mut Lend::new(env, w), cx, bpp, rec, class) as u16;
                hp[0..2].copy_from_slice(&v.to_le_bytes());
                env.callwc(
                    0x021C_3680,
                    &mut [Arg::V(list), Arg::V(bpp), Arg::V(v as u32), Arg::V(1)],
                    w,
                    cx,
                );
                fainted = 1;
            }
            let slot = u16::from_le_bytes([hp[0], hp[1]]) as u32;
            if fainted == 0 {
                put_move_used(&mut Lend::new(env, w), cx, client, mon_a, bpp, rec, slot);
            }
            record_bracket_4268(
                &mut Lend::c(env, cx),
                w,
                mon_a,
                bpp,
                rec,
                tbl,
                slot,
                0,
                fainted,
            );
            if chained || fainted != 0 {
                msg_1c88(&mut Lend::wc(env, w, cx), mon_a, rec, 0);
            }
            env.callwc(0x021C_4F64, &mut [Arg::P, Arg::V(bpp)], w, cx);
        }
        bracket_close(w, old);
    }
}

/// C: `FUN_overlay_d_93__021c4614`: when `8588(main)` is 2 and both sides' party slots pass
/// (`9b98` is 1): `4724` / `4734` slots, command 0x50 and the four `bd43c` / `bd4e4` calls.
pub fn pair_sides<E: FlowEnv>(env: &mut E, w: &SvFlowWork, cx: &mut Cx) {
    let main = w.u32_at(OFS_MAIN);
    let pokecon = w.u32_at(OFS_POKECON);
    if env.callwrc(0x021B_8588, &mut [Arg::V(main)], w, cx) != 2 {
        return;
    }
    let mut lo = env.callwrc(0x021B_8FF4, &mut [Arg::V(main)], w, cx);
    let other = env.callwrc(
        0x021B_9064,
        &mut [Arg::V(main), Arg::V(lo), Arg::V(0)],
        w,
        cx,
    ) & 0xFF;
    let mut hi = other;
    if other < lo {
        hi = lo;
        lo = other;
    }
    let a = env.callwrc(0x021B_9AEC, &mut [Arg::V(pokecon), Arg::V(lo)], w, cx);
    let b = env.callwrc(0x021B_9AEC, &mut [Arg::V(pokecon), Arg::V(hi)], w, cx);
    if env.callwrc(0x021B_9B98, &mut [Arg::V(a)], w, cx) != 1 {
        return;
    }
    if env.callwrc(0x021B_9B98, &mut [Arg::V(b)], w, cx) != 1 {
        return;
    }
    let a = env.callwrc(0x021B_9CE8, &mut [Arg::V(a)], w, cx);
    let b = env.callwrc(0x021B_9CE8, &mut [Arg::V(b)], w, cx);
    let ida = mon_id(&mut Lend::wrc(env, w, cx), a);
    let idb = mon_id(&mut Lend::wrc(env, w, cx), b);
    let sa = env.callwrc(
        0x021C_4724,
        &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(ida)],
        w,
        cx,
    );
    let sb = env.callwrc(
        0x021C_4734,
        &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(idb)],
        w,
        cx,
    );
    if sa == 6 || sb == 6 {
        return;
    }
    let ca = env.callwrc(0x021B_8DEC, &mut [Arg::V(main), Arg::V(sa)], w, cx);
    let cb = env.callwrc(0x021B_8DEC, &mut [Arg::V(main), Arg::V(sb)], w, cx);
    if ca == cb && env.callwrc(0x021B_99F0, &mut [Arg::V(sa)], w, cx) == 0 {
        cmd_queue::put_cmd(
            &mut Lend::new_ref(env, w),
            cx.q,
            0x50,
            &[lo, ca, hi, cb],
            &mut cx.g[cx.gb..],
        );
        env.callwrc(
            0x021B_D43C,
            &mut [Arg::P, Arg::V(lo), Arg::V(ca), Arg::V(1), Arg::V(1)],
            w,
            cx,
        );
        env.callwrc(
            0x021B_D43C,
            &mut [Arg::P, Arg::V(hi), Arg::V(cb), Arg::V(1), Arg::V(1)],
            w,
            cx,
        );
        env.callwrc(
            0x021B_D4E4,
            &mut [Arg::P, Arg::V(lo), Arg::V(ca), Arg::V(1)],
            w,
            cx,
        );
        env.callwrc(
            0x021B_D4E4,
            &mut [Arg::P, Arg::V(hi), Arg::V(cb), Arg::V(1)],
            w,
            cx,
        );
    }
}

/// C: `FUN_overlay_d_93__021c4744`: picks a random alive mon of the side (the `bb85c` record,
/// byte 9 = count, at most 4) and applies half / full of its max HP through `334c`, message 0x53.
pub fn random_side_effect<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx) {
    let main = w.u32_at(OFS_MAIN);
    let pokecon = w.u32_at(OFS_POKECON);
    let handle = env.callwc(0x021B_8630, &mut [Arg::V(main)], w, cx);
    if handle == 0 {
        return;
    }
    let kind = env.callwc(0x0202_E7E0, &mut [], w, cx);
    if kind == 0 || kind == 3 {
        return;
    }
    let side = env.callwc(0x021B_8FF4, &mut [Arg::V(main)], w, cx);
    let rec_ptr = env.callwc(0x021B_B85C, &mut [Arg::V(w.u32_at(0)), Arg::V(side)], w, cx);
    let rec_ofs = rec_ptr.wrapping_sub(cx.w_addr) as usize;
    let mut found = [0u8; 4];
    let mut n = 0usize;
    let mut slot = 0u32;
    if w.mem[rec_ofs + 9] != 0 {
        loop {
            let m = env.callwc(
                0x021B_990C,
                &mut [Arg::V(pokecon), Arg::V(side), Arg::V(slot)],
                w,
                cx,
            );
            let ok = m != 0 && env.callwc(0x021D_5B68, &mut [Arg::V(m)], w, cx) == 0;
            if ok && env.callwc(0x021D_5B48, &mut [Arg::V(m)], w, cx) == 0 {
                let id = mon_id(&mut Lend::wc(env, w, cx), m);
                found[n] = id as u8;
                n += 1;
            }
            slot = (slot + 1) & 0xFF;
            if slot >= w.mem[rec_ofs + 9] as u32 {
                break;
            }
        }
    }
    if n == 0 {
        return;
    }
    let rnd = env.callwc(0x0203_F0A8, &mut [], w, cx);
    let idx = ((rnd as u64 * n as u64) >> 32) as usize;
    let id = found[idx] as u32;
    let mon = env.callwc(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(id)], w, cx);
    let mut amount = mon_get(&mut Lend::wc(env, w, cx), mon, 0xE);
    if kind == 1 {
        amount >>= 1;
    }
    if amount != 0 && gate_then_apply(env, w, cx, mon, amount & 0xFFFF, 0) != 0 {
        cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5A, 0x53, &[4, id, LIST_END]);
        env.callwc(0x0202_E7E4, &mut [Arg::V(handle)], w, cx);
    }
}

/// C: `FUN_overlay_d_93__021c446c`: the turn-end state machine (state byte at `work + 0x77e`,
/// 0..6): `43f0` and `4744` once, then the stages `4d1c` / `4838(0x76)` / `493c` / `4b8c` +
/// `4c24` / `4838(0x77)` (each advances the state; a stage that answers 0 continues with the
/// next one unless `bc2b0` says to stop) and the tail (`4838(0x78)`, the per-mon loop, `4614`,
/// the turn counter); returns 1 when a stage asked to wait.
pub fn turn_end<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx) -> u32 {
    let list = w.u32_at(0x854);
    if w.mem[OFS_STATE_77E] == 0 {
        let wa = cx.w_addr;
        collect_mons(&mut Lend::wc(env, w, cx), wa, list);
        w.mem[OFS_STATE_77E] = 1;
        random_side_effect(env, w, cx);
    }
    let mut stage = w.mem[OFS_STATE_77E] as i8;
    loop {
        match stage {
            1 | 2 | 3 | 5 => {
                bump_state(w);
                let waiting = match stage {
                    1 => {
                        cmd_queue::put_cmd(
                            &mut Lend::new(env, w),
                            cx.q,
                            0x56,
                            &[0],
                            &mut cx.g[cx.gb..],
                        );
                        env.callwc(0x021C_4D1C, &mut [Arg::P, Arg::V(list)], w, cx)
                    }
                    2 => env.callwc(
                        0x021C_4838,
                        &mut [Arg::P, Arg::V(list), Arg::V(0x76)],
                        w,
                        cx,
                    ),
                    3 => env.callwc(0x021C_493C, &mut [Arg::P, Arg::V(list)], w, cx),
                    _ => env.callwc(
                        0x021C_4838,
                        &mut [Arg::P, Arg::V(list), Arg::V(0x77)],
                        w,
                        cx,
                    ),
                };
                if waiting != 0 {
                    return 1;
                }
                if env.callwc(0x021B_C2B0, &mut [Arg::P], w, cx) != 0 {
                    return 0;
                }
                stage += 1;
            }
            4 => {
                bump_state(w);
                env.callwc(0x021C_4B8C, &mut [Arg::P], w, cx);
                env.callwc(0x021C_4C24, &mut [Arg::P], w, cx);
                stage = 5;
            }
            6 => return turn_end_tail(env, w, cx, list),
            _ => return 0,
        }
    }
}

fn bump_state(w: &mut SvFlowWork) {
    w.mem[OFS_STATE_77E] = w.mem[OFS_STATE_77E].wrapping_add(1);
}

/// The last stage of [`turn_end`]: `4838(0x78)`, per-mon `6408` / `6e88` + command 0x2e, `4614`,
/// `d701c`, the turn counter (capped at 0x270f), state back to 0.
fn turn_end_tail<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, list: u32) -> u32 {
    env.callwc(
        0x021C_4838,
        &mut [Arg::P, Arg::V(list), Arg::V(0x78)],
        w,
        cx,
    );
    bump_state(w);
    env.callwc(0x021C_45E4, &mut [Arg::V(list)], w, cx);
    let mut mon = env.callwc(0x021C_45F4, &mut [Arg::V(list)], w, cx);
    while mon != 0 {
        env.callwc(0x021D_6408, &mut [Arg::V(mon)], w, cx);
        env.callwc(0x021D_6E88, &mut [Arg::V(mon)], w, cx);
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        cmd_queue::put_cmd(
            &mut Lend::new(env, w),
            cx.q,
            0x2E,
            &[id],
            &mut cx.g[cx.gb..],
        );
        mon = env.callwc(0x021C_4604, &mut [Arg::V(list)], w, cx);
    }
    pair_sides(env, w, cx);
    env.callwc(0x021D_701C, &mut [], w, cx);
    if w.u32_at(0x10) < 0x270F {
        let v = w.u32_at(0x10) + 1;
        w.set_u32_at(0x10, v);
    }
    w.mem[OFS_STATE_77E] = 0;
    w.set_u32_at(OFS_STATE_77E - 10, 0);
    0
}

/// C: `FUN_overlay_d_93__021c11f0`: the per-target computation of one move: for each target
/// (`0c38` fills the arrays at 0x021F3B20..) `4104`, the hit counter, `0cb4` / `15e8`, `0ce8`,
/// `1788`, `5798` / `5840` / `59c4`, `1828`, `16f8`, `1ccc` / `1dcc` / `1880` / `4218`, `4f64`;
/// returns the remaining HP total. The arrays are in `cx.g` (the global at 0x021F3B20, 0x64
/// bytes; the put_cmd scratch is its tail, `cx.gb == 0x24`): u8 `+0`, u8 `+3`, u16 `+6`, u32
/// `+0xC`, mon pointers `+0x18`.
#[allow(clippy::too_many_arguments)]
pub fn per_target_damage<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon_a: u32,
    rec: &[u8],
    b: &mut [u8],
    p8: u32,
    record: u32,
    rec_addr: u32,
) -> u32 {
    // B16: the arrays are the global 0x021F3B20 (`cx.g`), passed on by address
    let g = |o: u32| Arg::PAt(0x021F_3B20 + o);
    let mon_at = |g: &[u8], i: usize| w_u32(g, 0x18 + 4 * i);
    let id = mon_id(&mut Lend::wc(env, w, cx), mon_a);
    let client = env.callwc(
        0x021C_14B0,
        &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(id)],
        w,
        cx,
    );
    let mut total = env.callwc(0x021C_0D70, &mut [Arg::PAt(record)], w, cx);
    let n = env.callwc(
        0x021C_0C38,
        &mut [Arg::P, Arg::PAt(record), g(0x18), g(6), g(0xC), g(0)],
        w,
        cx,
    ) & 0xFF;
    if n != 0 {
        let mut i = 0usize;
        loop {
            let mon = mon_at(cx.g, i);
            if env.callwc(0x021D_6D7C, &mut [Arg::V(mon)], w, cx) != 0 {
                let u16_i = w_u16(cx.g, 6 + 2 * i);
                let u32_i = w_u32(cx.g, 0xC + 4 * i);
                let u8_i = cx.g[i] as u32;
                let r4 = apply_target_effects(env, w, cx, mon_a, mon, u16_i, u32_i, u8_i, rec);
                // the original re-reads the mon pointer: `4104`'s commands overwrote the
                // tail of the array (the put_cmd scratch overlaps it)
                let mon = mon_at(cx.g, i);
                env.callwc(
                    0x021C_14C0,
                    &mut [
                        Arg::V(w.u32_at(OFS_VAL_860)),
                        Arg::V(mon),
                        Arg::V(r4),
                        Arg::V(1),
                    ],
                    w,
                    cx,
                );
                total = total.wrapping_sub(u16_i.wrapping_sub(r4));
            }
            i += 1;
            if i as u32 >= n {
                break;
            }
        }
    }
    if byte1_is_zero(b) != 0 {
        let n2 = env.callwc(0x021C_0CB4, &mut [Arg::P, Arg::PAt(record), g(0x18)], w, cx) & 0xFF;
        if n2 != 0 {
            let mut i = 0usize;
            loop {
                let mon = mon_at(cx.g, i);
                record_bracket_1624(&mut Lend::c(env, cx), w, mon_a, mon, rec);
                i += 1;
                if i as u32 >= n2 {
                    break;
                }
            }
        }
    }
    b[1] = b[1].wrapping_add(1);
    let cnt = env.callwc(
        0x021C_0CE8,
        &mut [Arg::P, Arg::PAt(record), g(0x18), g(6), g(0xC), g(0), g(3)],
        w,
        cx,
    ) & 0xFF;
    if cnt == 0 {
        return total;
    }
    // the mon array is part of the global the callees also use: a copy, the global lent
    let mons = cx.g[0x18..].to_vec();
    record_bracket_17cc(&mut Lend::c(env, cx), w, mon_a, cnt, &mons);
    let flag = p8 & 1;
    env.callwc(
        0x021C_5798,
        &mut [
            Arg::P,
            Arg::PAt(rec_addr),
            Arg::V(cnt),
            g(0xC),
            g(0x18),
            g(6),
        ],
        w,
        cx,
    );
    if byte3(b) == 0 {
        env.callwc(
            0x021C_5840,
            &mut [Arg::P, Arg::V(cnt), g(0xC), g(0x18), Arg::V(flag)],
            w,
            cx,
        );
    } else {
        let v = w_u32(cx.g, 0xC);
        set_byte5_if_3(b, v);
    }
    env.callwc(
        0x021C_59C4,
        &mut [Arg::P, Arg::V(cnt), g(0x18), g(0), Arg::V(flag)],
        w,
        cx,
    );
    for i in 0..cnt as usize {
        let mon = mon_at(cx.g, i);
        let u16_i = w_u16(cx.g, 6 + 2 * i);
        env.callwc(
            0x021C_14D0,
            &mut [
                Arg::V(w.u32_at(OFS_VAL_860)),
                Arg::V(mon),
                Arg::V(u16_i),
                Arg::V(0),
            ],
            w,
            cx,
        );
        put_move_used(&mut Lend::new(env, w), cx, client, mon_a, mon, rec, u16_i);
        let mon = mon_at(cx.g, i);
        mon_set5fc0(&mut Lend::wc(env, w, cx), mon, 2);
    }
    for i in 0..cnt as usize {
        let mode = cx.g[3 + i] as u32;
        if mode != 0 {
            let mon = mon_at(cx.g, i);
            msg_16f8(&mut Lend::hv(env, vec![&mut *cx.g]), w, cx.q, mon, mode);
        }
    }
    for i in 0..cnt as usize {
        let mon = mon_at(cx.g, i);
        let u16_i = w_u16(cx.g, 6 + 2 * i);
        let u32_i = w_u32(cx.g, 0xC + 4 * i);
        let u8_i = cx.g[i] as u32;
        move_chance_gate(&mut Lend::c(env, cx), w, rec, mon_a, mon);
        let mon = mon_at(cx.g, i);
        category8_followup(env, w, cx, rec, mon_a, mon, u16_i);
        let mon = mon_at(cx.g, i);
        category_dispatch(env, w, cx, rec, mon_a, mon, 0);
        let mon = mon_at(cx.g, i);
        record_bracket_4268(
            &mut Lend::c(env, cx),
            w,
            mon_a,
            mon,
            rec,
            u32_i,
            u16_i,
            u8_i,
            0,
        );
    }
    for i in 0..cnt as usize {
        let mon = mon_at(cx.g, i);
        env.callwc(0x021C_4F64, &mut [Arg::P, Arg::V(mon)], w, cx);
    }
    env.callwc(0x021C_4F64, &mut [Arg::P, Arg::V(mon_a)], w, cx);
    total
}

/// C: `FUN_overlay_d_93__021c11a4`: damage on one target when `0c34(record)` says there is one:
/// `11f0` ([`per_target_damage`], the full per-target computation; `11a4` forwards its own `rec` and
/// `b` words and ignores the rest of the arguments `11f0` is handed), then `18d4` (the log record);
/// returns the damage.
#[allow(clippy::too_many_arguments)]
pub fn damage_with_log<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    rec: &[u8],
    mon_a: u32,
    p5: u32,
    b: &mut [u8],
    p8: u32,
    rec_addr: u32,
) -> u32 {
    if env.callwc(0x021C_0C34, &mut [Arg::V(p5)], w, cx) as u8 == 0 {
        return 0;
    }
    let d = per_target_damage(env, w, cx, mon_a, rec, b, p8, p5, rec_addr);
    record_bracket_1910(&mut Lend::c(env, cx), w, mon_a, rec, d);
    d
}

/// C: `FUN_overlay_d_93__021c0f24`: applies one move record to every target of the per-turn list
/// (`b[0]` = count): the attacker id into `work + 0x789`, the `4e` / `4d` commands, the loop of
/// `0a4c` + `11a4` (damage and log) with the faint / chain checks, then the closing commands;
/// returns the summed damage. `p` = the per-turn struct (`work + 0x4a8`), `b` = the target list
/// (`work + 0x4b4`).
#[allow(clippy::too_many_arguments)]
pub fn apply_move_to_targets<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    b: &mut [u8],
    rec: &[u8],
    mon_a: u32,
    p4: u32,
    rec_addr: u32,
    table: u32,
) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let mut total = 0u32;
    let mut slot = 6u32;
    let target = env.callwc(0x021C_1144, &mut [Arg::V(p4), Arg::V(0)], w, cx);
    let tid = mon_id(&mut Lend::wc(env, w, cx), target);
    w.mem[OFS_BYTE_789] = tid as u8;
    if env.callwc(0x021B_8590, &mut [Arg::V(main)], w, cx) != 0 {
        let idt = mon_id(&mut Lend::wc(env, w, cx), target);
        slot = env.callwc(
            0x021C_1154,
            &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(idt)],
            w,
            cx,
        );
        if slot != 6 {
            let ida = mon_id(&mut Lend::wc(env, w, cx), mon_a);
            let v = env.callwc(
                0x021C_1164,
                &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(ida)],
                w,
                cx,
            );
            cmd_queue::put_cmd(
                &mut Lend::new(env, w),
                cx.q,
                0x4E,
                &[v, slot, 0x27E],
                &mut cx.g[cx.gb..],
            );
        }
    }
    let mut rounds = 0u32;
    let mut count = 0u32;
    if b[0] != 0 {
        loop {
            let before = env.callwc(0x021D_62A8, &mut [Arg::V(mon_a)], w, cx);
            if p[5] & 1 == 0 {
                p[5] = p[5] & 0xFE | 1;
                p[5] |= 2;
            }
            cmd_queue::put_cmd(
                env,
                cx.q,
                0x30,
                &[p[2] as u32, p[3] as u32, rec_id(rec), 0],
                &mut cx.g[cx.gb..],
            );
            let v870 = w.u32_at(OFS_VAL_870);
            env.callwc(
                0x021C_0A4C,
                &mut [
                    Arg::P,
                    Arg::V(mon_a),
                    Arg::V(p4),
                    Arg::PAt(rec_addr),
                    Arg::PAt(table),
                    Arg::V(0x1000),
                    Arg::V(v870),
                ],
                w,
                cx,
            );
            let d = damage_with_log(env, w, cx, rec, mon_a, v870, b, 0, rec_addr);
            total = total.wrapping_add(d);
            rounds += 1;
            if env.callwc(0x021D_5B68, &mut [Arg::V(target)], w, cx) != 0
                || env.callwc(0x021D_5B68, &mut [Arg::V(mon_a)], w, cx) != 0
            {
                break;
            }
            record_bracket_199c(&mut Lend::c(env, cx), w, target, 1);
            let after = env.callwc(0x021D_62A8, &mut [Arg::V(mon_a)], w, cx);
            if after == 2 && before != 2 {
                break;
            }
            if b[2] != 0
                && env.callwc(
                    0x021B_FA60,
                    &mut [Arg::P, Arg::V(mon_a), Arg::V(target), Arg::PAt(rec_addr)],
                    w,
                    cx,
                ) == 0
            {
                break;
            }
            count += 1;
            if b[0] as u32 <= count {
                break;
            }
        }
    }
    if rounds != 0 {
        level_message(&mut Lend::new(env, w), cx.q, b[5] as i32);
        cmd_queue::put_cmd_list(
            &mut Lend::new(env, w),
            cx.q,
            0x5A,
            0x20,
            &[rounds, LIST_END],
        );
    }
    if env.callwc(0x021D_5B68, &mut [Arg::V(target)], w, cx) != 0 {
        let id = mon_id(&mut Lend::wc(env, w, cx), target);
        cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5B, 0, &[id, LIST_END]);
    }
    if slot != 6 {
        let ida = mon_id(&mut Lend::wc(env, w, cx), mon_a);
        let v = env.callwc(
            0x021C_1174,
            &mut [Arg::PHost(w.mem[0x1AB8..].as_ptr() as usize), Arg::V(ida)],
            w,
            cx,
        );
        cmd_queue::put_cmd(
            &mut Lend::new(env, w),
            cx.q,
            0x4D,
            &[v, 0x291],
            &mut cx.g[cx.gb..],
        );
    }
    w.mem[OFS_BYTE_789] = 0x1F;
    total
}

#[cfg(test)]
pub(crate) mod tests;
