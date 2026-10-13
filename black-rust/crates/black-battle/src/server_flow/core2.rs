//! Flow core, part 2 (0x021BD1D0-0x021BF324; B10g).
//!
//! The "message builders" (one server->client command each), the queue wrappers that bracket a
//! builder with the packed header word at work + 0x1D74 (`cmd_queue::advance_fields` /
//! `set_word`), the party / switch helpers and the two big routines `021bde98` (use a move) and
//! `021bec58` (apply one move category). Specs: `tools/rom-diff/svflow_specs/core2.spec` (66
//! functions; the 42 `bx pc` veneers of the range are listed at the top of the spec).
//!
//! B10g-spec ports (smoke ports that prove the spec grammar; the rest is for the B10g port job).
//! Callees outside this area go through [`FlowEnv`]; `cmd_queue` ones are called directly (they are
//! ported and run as real code), and so is a core2 callee once it is in [`PORTED`].

use super::cmd_queue;
use super::core1;
use super::env::{Arg, FlowEnv, Lend, Lent};
use super::events;
use super::server_side;
use super::SvFlowWork;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021B_EC08,
    0x021B_EC2C,
    0x021B_DC78,
    0x021B_E8C4,
    0x021B_D1D0,
    0x021B_D1E0,
    0x021B_E518,
    0x021B_E52C,
    0x021B_D338,
    0x021B_DBEC,
    0x021B_DC0C,
    0x021B_DC58,
    0x021B_D554,
    0x021B_DB88,
    0x021B_DE18,
    0x021B_DBB8,
    0x021B_E8F4,
    0x021B_F2EC,
    0x021B_E95C,
    0x021B_D8AC,
    0x021B_E6A8,
    0x021B_E9D0,
    0x021B_E620,
    0x021B_EA14,
    0x021B_E878,
    0x021B_F168,
    0x021B_D6B8,
    0x021B_EA60,
    0x021B_D850,
    0x021B_EB14,
    0x021B_E924,
    0x021B_DC2C,
    0x021B_D518,
    0x021B_E66C,
    0x021B_E994,
    0x021B_F244,
    0x021B_F27C,
    0x021B_F2B4,
    0x021B_D2B4,
    0x021B_D34C,
    0x021B_D200,
    0x021B_D43C,
    0x021B_D4E4,
    0x021B_D3E8,
    0x021B_D8F0,
    0x021B_D9D4,
    0x021B_EBD8,
    0x021B_DA00,
    0x021B_DD7C,
    0x021B_DD00,
    0x021B_DC98,
    0x021B_D708,
    0x021B_D60C,
    0x021B_D584,
    0x021B_DE48,
    0x021B_E3A8,
    0x021B_E400,
    0x021B_E550,
    0x021B_EAB4,
    0x021B_E6EC,
    0x021B_E7E4,
    0x021B_F04C,
    0x021B_F0E4,
    0x021B_F1B4,
    0x021B_EC58,
    0x021B_DE98,
];

/// `work + 0x4`: the main module (address only).
const OFS_MAIN: usize = 0x4;
/// `work + 0x1D74`: the packed queue header word (`cmd_queue::advance_fields` / `set_word`).
const OFS_QUEUE_HEADER: usize = 0x1D74;

/// `FUN_overlay_d_93__021d5440`: the id (0..5) of a BattleMon.
fn mon_id<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// `thunk_FUN_overlay_d_93__021b8d08` (its target): the client that owns the mon with id `id`.
fn client_of_id<E: FlowEnv>(env: &mut E, id: u32) -> u32 {
    env.call(0x021B_8D08, &mut [Arg::V(id)])
}

/// C: `FUN_overlay_d_93__021bec08`: 1 when `mon` belongs to our client (the one `021b8ff4` names
/// for the main module at work + 4).
pub fn mon_is_ours<E: FlowEnv>(env: &mut E, w: &SvFlowWork, mon: u32) -> u32 {
    let id = mon_id(&mut Lend::new_ref(env, w), mon);
    let client = client_of_id(&mut Lend::new_ref(env, w), id);
    let ours = env.callwr(0x021B_8FF4, &mut [Arg::V(w.u32_at(OFS_MAIN))], w);
    (client == ours) as u32
}

/// C: `FUN_overlay_d_93__021bec2c`: for a mon on our side and ability-like ids 0x96 / 0xA5, tells
/// the client (`021ba1b4` with the main module and 0x49 / 0x4A).
pub fn notify_special_id<E: FlowEnv>(env: &mut E, w: &SvFlowWork, mon: u32, id: u32) {
    if mon_is_ours(env, w, mon) == 0 {
        return;
    }
    let code = match id {
        0x96 => 0x49,
        0xA5 => 0x4A,
        _ => return,
    };
    env.callwr(
        0x021B_A1B4,
        &mut [Arg::V(w.u32_at(OFS_MAIN)), Arg::V(code)],
        w,
    );
}

/// C: `FUN_overlay_d_93__021bdc78`: puts command 0x3B (client of the mon, mon id) on the command
/// queue `q` (`scratch` = the 0x40 bytes at 0x021F3B44 that `cmd_queue::put_cmd` fills).
pub fn put_mon_client_cmd<E: FlowEnv>(env: &mut E, q: &mut [u8], scratch: &mut [u8], mon: u32) {
    let id = mon_id(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), mon);
    let client = client_of_id(&mut Lend::hv(env, vec![&mut *q, &mut *scratch]), id);
    cmd_queue::put_cmd(env, q, 0x3B, &[client, id], scratch);
}

/// Brackets a message builder with the queue header word (`021cc8d8` / `021cc95c`):
/// `old = advance_fields(header)`, builder, `header = old`.
fn bracket<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, f: impl FnOnce(&mut Lend<'_, E>)) {
    let old = cmd_queue::advance_fields(&mut w.mem[OFS_QUEUE_HEADER..]);
    f(&mut Lend::new(env, w));
    cmd_queue::set_word(&mut w.mem[OFS_QUEUE_HEADER..], old);
}

/// C: `FUN_overlay_d_93__021be8c4`: brackets the builder `021be8f4` with the queue header word.
pub fn queue_wrap_mon_builder<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32) {
    bracket(env, w, |env| build_be8f4(env, mon));
}

// ---- message builder primitives (start / add argument / send / read back / end) ----------------

/// `FUN_overlay_d_93__021d74f8`: starts a message of `n` bytes.
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

/// `FUN_overlay_d_93__021d75a8`: adds argument `id` = `v` (second flavour).
fn arg_a8<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_75A8, &mut [Arg::V(id), Arg::V(v)]);
}

/// `FUN_overlay_d_93__021d7630`: adds the reply slot `id` = `v`.
fn arg_630<E: FlowEnv>(env: &mut E, id: u32, v: u32) {
    env.call(0x021D_7630, &mut [Arg::V(id), Arg::V(v)]);
}

/// `FUN_overlay_d_93__021d775c`: reads reply slot `id` back.
fn read_back<E: FlowEnv>(env: &mut E, id: u32) -> u32 {
    env.call(0x021D_775C, &mut [Arg::V(id)])
}

/// C: `FUN_overlay_d_93__021bd1d0`: sets the done flag (+0xD) of the action record of mon id `id`.
pub fn mark_done<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, id: u32) {
    if let Some(rec) = core1::record_by_mon(env, w, id) {
        w.mem[rec + 0xD] = 1;
    }
}

/// C: `FUN_overlay_d_93__021bd1e0`: resets a 6-byte record: `u16 +0 = 0`, `+2 = +3 = 6`, `+4 = 0`,
/// `+5 &= ~3`.
pub fn reset_record(b: &mut [u8]) {
    b[0] = 0;
    b[1] = 0;
    b[4] = 0;
    b[2] = 6;
    b[3] = 6;
    b[5] &= !1;
    b[5] &= !2;
}

/// C: `FUN_overlay_d_93__021be518`: clears a list `{u8 count; u8 a[6]}`: count 0, `a[i] = 0x1F`.
pub fn list_clear(b: &mut [u8]) {
    b[0] = 0;
    for x in &mut b[1..7] {
        *x = 0x1F;
    }
}

/// C: `FUN_overlay_d_93__021be52c`: appends `(a, b, c)` to the list `{u8 count; u8 a[6] at +1;
/// u8 b[6] at +7; u8 c[6] at +0xD}` unless it holds 6 entries.
pub fn list_push(l: &mut [u8], a: u8, b: u8, c: u8) {
    let n = l[0] as usize;
    if n < 6 {
        l[n + 1] = a;
        l[n + 0xD] = b;
        l[n + 7] = c;
        l[0] += 1;
    }
}

/// C: `FUN_overlay_d_93__021bd338`: `rec[3] = 1` when the battle type of the main module (`021b8588`
/// of its word at +4) is 3.
pub fn set_flag_if_type3<E: FlowEnv>(env: &mut E, rec: &mut [u8], main: &[u8]) {
    let m = u32::from_le_bytes(main[4..8].try_into().unwrap());
    if env.call(0x021B_8588, &mut [Arg::V(m)]) == 3 {
        rec[3] = 1;
    }
}

// ---- message builders -----------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bdbec`: message 0x56, no arguments.
pub fn build_bdbec<E: FlowEnv>(env: &mut E) {
    msg_start(env, 0xC65);
    msg_send(env, 0x56);
    msg_end(env, 0xC67);
}

/// C: `FUN_overlay_d_93__021bdc0c`: message 0x57, no arguments.
pub fn build_bdc0c<E: FlowEnv>(env: &mut E) {
    msg_start(env, 0xC73);
    msg_send(env, 0x57);
    msg_end(env, 0xC75);
}

/// C: `FUN_overlay_d_93__021bdc58`: message 0x58, no arguments.
pub fn build_bdc58<E: FlowEnv>(env: &mut E) {
    msg_start(env, 0xC91);
    msg_send(env, 0x58);
    msg_end(env, 0xC93);
}

/// C: `FUN_overlay_d_93__021bd554`: message 0xA4 with the id of `mon`.
pub fn build_bd554<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0xA98);
    let id = mon_id(env, mon);
    arg_a8(env, 2, id);
    msg_send(env, 0xA4);
    msg_end(env, 0xA9B);
}

/// C: `FUN_overlay_d_93__021bdb88`: message 0x55 with the id of `mon`.
pub fn build_bdb88<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0xC43);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    msg_send(env, 0x55);
    msg_end(env, 0xC46);
}

/// C: `FUN_overlay_d_93__021bde18`: message 0x54 with the id of `mon`.
pub fn build_bde18<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0xD14);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    msg_send(env, 0x54);
    msg_end(env, 0xD17);
}

/// C: `FUN_overlay_d_93__021bdbb8`: message 0x55 (sent with the extra word 4) with the id of `mon`.
pub fn build_bdbb8<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0xC55);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    env.call(0x021D_70FC, &mut [Arg::P, Arg::V(0x55), Arg::V(4)]);
    msg_end(env, 0xC58);
}

/// C: `FUN_overlay_d_93__021be8f4`: message 7 with the id of `mon` (argument 3).
pub fn build_be8f4<E: FlowEnv>(env: &mut E, mon: u32) {
    msg_start(env, 0xF80);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    msg_send(env, 7);
    msg_end(env, 0xF83);
}

/// C: `FUN_overlay_d_93__021bf2ec`: message `cmd` with argument 2 = `a`, argument 0x12 = `b`.
pub fn build_bf2ec<E: FlowEnv>(env: &mut E, a: u32, b: u32, cmd: u32) {
    msg_start(env, 0x1214);
    arg_ec(env, 2, a);
    arg_ec(env, 0x12, b);
    msg_send(env, cmd);
    msg_end(env, 0x1218);
}

/// C: `FUN_overlay_d_93__021be95c`: message 3 with the id of `mon` (argument 3) and `x` (0x12).
pub fn build_be95c<E: FlowEnv>(env: &mut E, mon: u32, x: u32) {
    msg_start(env, 0xFA0);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x12, x);
    msg_send(env, 3);
    msg_end(env, 0xFA4);
}

/// C: `FUN_overlay_d_93__021bd8ac`: message 0xD (reply slot 0x51); returns the reply.
pub fn build_bd8ac<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    msg_start(env, 0xBA3);
    let id = mon_id(env, mon);
    arg_a8(env, 2, id);
    arg_630(env, 0x51, 0);
    msg_send(env, 0xD);
    let r = read_back(env, 0x51);
    msg_end(env, 0xBA8);
    r
}

/// C: `FUN_overlay_d_93__021be6a8`: message `cmd` with the id of `mon` (3), `rec.u16[0]` (0x12)
/// and `rec[6]` (0x16).
pub fn build_be6a8<E: FlowEnv>(env: &mut E, mon: u32, rec: &[u8], cmd: u32) {
    msg_start(env, 0xECF);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x12, u16::from_le_bytes([rec[0], rec[1]]) as u32);
    arg_ec(env, 0x16, rec[6] as u32);
    msg_send(env, cmd);
    msg_end(env, 0xED4);
}

/// C: `FUN_overlay_d_93__021be9d0`: message 4 with the id of `mon` (2), `a` (0x12) and `b` (0x51).
pub fn build_be9d0<E: FlowEnv>(env: &mut E, mon: u32, a: u32, b: u32) {
    msg_start(env, 0xFC0);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_ec(env, 0x12, a);
    arg_ec(env, 0x51, b);
    msg_send(env, 4);
    msg_end(env, 0xFC5);
}

/// C: `FUN_overlay_d_93__021be620`: message 0xA2 like `be6a8`, but `rec[6]` is a reply slot and is
/// overwritten with the reply.
pub fn build_be620<E: FlowEnv>(env: &mut E, mon: u32, rec: &mut [u8]) {
    msg_start(env, 0xEAB);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0x12, u16::from_le_bytes([rec[0], rec[1]]) as u32);
    arg_630(env, 0x16, rec[6] as u32);
    msg_send(env, 0xA2);
    rec[6] = read_back(env, 0x16) as u8;
    msg_end(env, 0xEB1);
}

/// C: `FUN_overlay_d_93__021bea14`: message 8 with the ids of two mons (2, 3) and `x` (0x12).
pub fn build_bea14<E: FlowEnv>(env: &mut E, a: u32, b: u32, x: u32) {
    msg_start(env, 0xFD3);
    let ia = mon_id(env, a);
    arg_ec(env, 2, ia);
    let ib = mon_id(env, b);
    arg_ec(env, 3, ib);
    arg_ec(env, 0x12, x);
    msg_send(env, 8);
    msg_end(env, 0xFD8);
}

/// C: `FUN_overlay_d_93__021be878`: message 6 (reply slot 0x51) with the id of `mon` (3) and `x` (0xD).
pub fn build_be878<E: FlowEnv>(env: &mut E, mon: u32, x: u32) -> u32 {
    msg_start(env, 0xF5B);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 0xD, x);
    arg_630(env, 0x51, 0);
    msg_send(env, 6);
    let r = read_back(env, 0x51);
    msg_end(env, 0xF61);
    r
}

/// C: `FUN_overlay_d_93__021bf168`: message 0x3D (reply slot 0x51) with the id of `mon` (3) and
/// `rec.u16[0]` (0x12); returns the reply.
pub fn build_bf168<E: FlowEnv>(env: &mut E, rec: &[u8], mon: u32) -> u32 {
    msg_start(env, 0x1194);
    let id = mon_id(env, mon);
    arg_a8(env, 3, id);
    arg_a8(env, 0x12, u16::from_le_bytes([rec[0], rec[1]]) as u32);
    arg_630(env, 0x51, 0);
    msg_send(env, 0x3D);
    let r = read_back(env, 0x51);
    msg_end(env, 0x119A);
    r
}

/// C: `FUN_overlay_d_93__021bd6b8`: message 0xB (reply slot 0x51) for a mon that is not fainted
/// (`021d5b68` is 0); returns the reply, 0 for a fainted mon.
pub fn build_bd6b8<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    if env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0 {
        return 0;
    }
    msg_start(env, 0xB21);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_630(env, 0x51, 0);
    msg_send(env, 0xB);
    let r = read_back(env, 0x51);
    msg_end(env, 0xB26);
    r
}

/// C: `FUN_overlay_d_93__021bea60`: message 9 with two mon ids (2, 3), `x` (0x12) and reply slot 0x51.
pub fn build_bea60<E: FlowEnv>(env: &mut E, a: u32, b: u32, x: u32) {
    msg_start(env, 0xFE5);
    let ia = mon_id(env, a);
    arg_ec(env, 2, ia);
    let ib = mon_id(env, b);
    arg_ec(env, 3, ib);
    arg_ec(env, 0x12, x);
    arg_630(env, 0x51, 0);
    msg_send(env, 9);
    msg_end(env, 0xFEB);
}

/// C: `FUN_overlay_d_93__021bd850`: message 0xC (reply slot 0x41, `021bd89c` also sees the work
/// and mon); returns the reply.
pub fn build_bd850<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    msg_start(env, 0xB8D);
    let id = mon_id(env, mon);
    arg_ec(env, 2, id);
    arg_630(env, 0x41, 0);
    env.call(0x021B_D89C, &mut [Arg::P, Arg::V(mon)]);
    msg_send(env, 0xC);
    let r = read_back(env, 0x41);
    msg_end(env, 0xB93);
    r
}

/// C: `FUN_overlay_d_93__021beb14`: message 0x1A listing the mons of `021bebb8(list)`; stores the
/// replies of slots 2 and 4 in `out_a` / `out_b`; returns 1 when `out_a != 0x1F`.
pub fn build_beb14<E: FlowEnv>(
    env: &mut E,
    mon: u32,
    x: u32,
    list: u32,
    out_a: &mut u8,
    out_b: &mut u8,
) -> u32 {
    let n = env.call(0x021B_EBB8, &mut [Arg::V(list)]);
    msg_start(env, 0x1028);
    arg_ec(env, 0x12, x);
    let id = mon_id(env, mon);
    arg_ec(env, 3, id);
    arg_ec(env, 5, n);
    let mut i = 0;
    while i < n {
        let m = env.call(0x021B_EBC8, &mut [Arg::V(list), Arg::V(i)]);
        let mid = mon_id(env, m);
        arg_ec(env, i + 6, mid);
        i += 1;
    }
    arg_630(env, 2, 0x1F);
    arg_a8(env, 4, 0x1F);
    msg_send(env, 0x1A);
    *out_a = read_back(env, 2) as u8;
    *out_b = read_back(env, 4) as u8;
    msg_end(env, 0x103B);
    (*out_a != 0x1F) as u32
}

// ---- queue wrappers (builder bracketed with the header word) ---------------------------------------

/// C: `FUN_overlay_d_93__021be924`: brackets `be95c(mon, x)`.
pub fn queue_wrap_be95c<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, x: u32) {
    bracket(env, w, |env| build_be95c(env, mon, x));
}

/// C: `FUN_overlay_d_93__021bdc2c`: brackets `bdc58`.
pub fn queue_wrap_bdc58<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork) {
    bracket(env, w, |env| build_bdc58(env));
}

/// C: `FUN_overlay_d_93__021bd518`: brackets `bd554(mon)` unless the mon is fainted (`021d5b68`).
pub fn queue_wrap_bd554<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32) {
    if env.callw(0x021D_5B68, &mut [Arg::V(mon)], w) == 0 {
        bracket(env, w, |env| build_bd554(env, mon));
    }
}

/// C: `FUN_overlay_d_93__021be66c`: brackets `be6a8(mon, rec, cmd)`.
pub fn queue_wrap_be6a8<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    mon: u32,
    rec: &[u8],
    cmd: u32,
) {
    bracket(env, w, |env| build_be6a8(env, mon, rec, cmd));
}

/// C: `FUN_overlay_d_93__021be994`: brackets `be9d0(mon, a, b)`.
pub fn queue_wrap_be9d0<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, a: u32, b: u32) {
    bracket(env, w, |env| build_be9d0(env, mon, a, b));
}

/// C: `FUN_overlay_d_93__021bf244`: brackets `bf2ec(a, b, 0x25)`.
pub fn queue_wrap_bf244<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, a: u32, b: u32) {
    bracket(env, w, |env| build_bf2ec(env, a, b, 0x25));
}

/// C: `FUN_overlay_d_93__021bf27c`: brackets `bf2ec(a, b, 0x26)`.
pub fn queue_wrap_bf27c<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, a: u32, b: u32) {
    bracket(env, w, |env| build_bf2ec(env, a, b, 0x26));
}

/// C: `FUN_overlay_d_93__021bf2b4`: brackets `bf2ec(a, b, 0x27)`.
pub fn queue_wrap_bf2b4<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, a: u32, b: u32) {
    bracket(env, w, |env| build_bf2ec(env, a, b, 0x27));
}

// ---- command queue context ----------------------------------------------------------------------

/// What the command-writing functions need besides the work image: the command queue buffer
/// (the region `work[0xC]` points at), the scratch words at 0x021F3B44 that `put_cmd` fills, and
/// the work's own address (stubbed callees that take a pointer into the work see it).
pub struct Cx<'a> {
    pub q: &'a mut [u8],
    pub scratch: &'a mut [u8],
    pub w_addr: u32,
}

/// `work + 4`: the main module, `+8`: the party container, `+0x10`: a u16-used word.
const OFS_POKECON: usize = 0x8;
const OFS_WORD_10: usize = 0x10;
/// `work + 0x1AB8` / `0x1A68` / `0x4B8`: helper structs (address only).
const OFS_HELPER_1AB8: u32 = 0x1AB8;
const OFS_HELPER_1A68: u32 = 0x1A68;
const OFS_HELPER_4B8: u32 = 0x4B8;
/// `work + 0x7C1`: u8 flag per mon id; `+0x78A`: u8 flag bits; `+0x786`: u8 counter.
const OFS_MON_FLAGS: usize = 0x7C1;
const OFS_FLAGS_78A: usize = 0x78A;
const OFS_COUNTER_786: usize = 0x786;

/// End-of-list marker of the variable-length commands (`cmd_queue::put_cmd_list`).
const LIST_END: u32 = 0xFFFF_0000;

// ---- party iterators (server-side object S = work[0]) ---------------------------------------------

fn slot_word(s: &[u8], slot: usize) -> u32 {
    u32::from_le_bytes(s[slot + 4..slot + 8].try_into().unwrap())
}

/// C: `FUN_overlay_d_93__021bd2b4`: starts an iteration over the mons of all clients: `out` =
/// `{u8 client; u8 index; u8 done; u8 mode}` positioned at the first mon that exists and is not
/// fainted (`021b9c00`, `021d5b68`); `done` stays 1 when there is none.
pub fn iter_first<E: FlowEnv>(env: &mut E, out: &mut [u8], s: &[u8]) {
    out[0] = 0;
    out[1] = 0;
    out[2] = 1;
    out[3] = 0;
    for i in 0..4u8 {
        let Some(slot) = server_side::slot_unchecked(s, i as i32) else {
            continue;
        };
        let count = s[slot + 9];
        if count == 0 {
            continue;
        }
        let party = slot_word(s, slot);
        let mut j = 0u8;
        loop {
            let m = env.call(0x021B_9C00, &mut [Arg::V(party), Arg::V(j as u32)]);
            let alive = m != 0 && env.call(0x021D_5B68, &mut [Arg::V(m)]) == 0;
            if alive {
                out[0] = i;
                out[2] = 0;
                out[1] = j;
                return;
            }
            j = j.wrapping_add(1);
            if j >= count {
                break;
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021bd34c`: iterator step: stores the mon at the current position in
/// `out` (a u32) and moves on to the next one that exists and is not fainted (in mode 1 only the
/// first 3 positions of a client are visited); `done` is set when the clients are exhausted.
/// Returns 0 when the iteration was already done, else 1.
pub fn iter_next<E: FlowEnv>(env: &mut E, it: &mut [u8], s: &[u8], out: &mut [u8]) -> u32 {
    if it[2] != 0 {
        return 0;
    }
    // the original reads slot 0's party word when the client has no slot; unreachable in a
    // consistent battle (the iterator only stops at used slots)
    let party = server_side::slot_checked(s, it[0] as u32).map_or(0, |slot| slot_word(s, slot));
    let m = env.call(0x021B_9BF0, &mut [Arg::V(party), Arg::V(it[1] as u32)]);
    out[..4].copy_from_slice(&m.to_le_bytes());
    it[1] = it[1].wrapping_add(1);
    loop {
        if it[0] >= 4 {
            it[2] = 1;
            return 1;
        }
        if let Some(slot) = server_side::slot_unchecked(s, it[0] as i32) {
            let bound = if it[3] == 0 { s[slot + 9] } else { 3 };
            let party = slot_word(s, slot);
            while it[1] < bound {
                let m = env.call(0x021B_9BF0, &mut [Arg::V(party), Arg::V(it[1] as u32)]);
                if m != 0 && env.call(0x021D_5B68, &mut [Arg::V(m)]) == 0 {
                    return 1;
                }
                it[1] = it[1].wrapping_add(1);
            }
        }
        it[0] = it[0].wrapping_add(1);
        it[1] = 0;
    }
}

// ---- party / switch bookkeeping -----------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bd200`: fills a 6-byte switch record `out` from the helper struct at
/// work + 0x1AB8 (`w_addr` is the work's address): `+2` = `021bd274(helper, id of mon)`, `+3` =
/// 6 (or, when `021bd264(r3) == 1` and `021bd284(r3)`, `021bd2a4(helper, id)` after
/// `021bd294(r3, 0)`), `+5 &= ~3`.
pub fn fill_switch_record<E: FlowEnv>(env: &mut E, out: &mut [u8], w_addr: u32, mon: u32, r3: u32) {
    let kind = env.call(0x021B_D264, &mut [Arg::V(r3)]);
    let id = mon_id(env, mon);
    let helper = w_addr.wrapping_add(OFS_HELPER_1AB8);
    out[2] = env.call(0x021B_D274, &mut [Arg::V(helper), Arg::V(id)]) as u8;
    out[3] = 6;
    out[5] &= !1;
    out[5] &= !2;
    if kind == 1 && env.call(0x021B_D284, &mut [Arg::V(r3)]) != 0 {
        let r = env.call(0x021B_D294, &mut [Arg::V(r3), Arg::V(0)]);
        let id2 = env.call(0x021D_5440, &mut [Arg::V(r)]);
        out[3] = env.call(0x021B_D2A4, &mut [Arg::V(helper), Arg::V(id2)]) as u8;
    }
}

/// C: `FUN_overlay_d_93__021bd43c`: swaps party slots `a`, `b` of `client` (`021b9c10`), updates
/// the helper struct at work + 0x1AB8 (`021bd4d4`) and, unless `skip`, puts command 0x44
/// `(client, slot a id, slot b id)` and asks `021d5fc0(mon, 0xC)` for each swapped mon that is
/// not fainted.
pub fn swap_party_slots<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    client: u32,
    slots: (u32, u32),
    skip: u32,
) {
    let (a, b) = slots;
    let party = env.callwrc(
        0x021B_9AEC,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(client)],
        w,
        cx,
    );
    let main = w.u32_at(OFS_MAIN);
    let ia = env.callwrc(
        0x021B_8BEC,
        &mut [Arg::V(main), Arg::V(client), Arg::V(a)],
        w,
        cx,
    );
    let ib = env.callwrc(
        0x021B_8BEC,
        &mut [Arg::V(main), Arg::V(client), Arg::V(b)],
        w,
        cx,
    );
    env.callwrc(
        0x021B_D4D4,
        &mut [
            Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_1AB8)),
            Arg::V(ia),
            Arg::V(ib),
        ],
        w,
        cx,
    );
    env.callwrc(
        0x021B_9C10,
        &mut [Arg::V(party), Arg::V(a), Arg::V(b)],
        w,
        cx,
    );
    if skip == 0 {
        cmd_queue::put_cmd(
            &mut Lend::new_ref(env, w),
            cx.q,
            0x44,
            &[client, ia, ib],
            cx.scratch,
        );
        let ma = env.callwrc(0x021B_9BF0, &mut [Arg::V(party), Arg::V(a)], w, cx);
        let mb = env.callwrc(0x021B_9BF0, &mut [Arg::V(party), Arg::V(b)], w, cx);
        if env.callwrc(0x021D_5B68, &mut [Arg::V(ma)], w, cx) == 0 {
            env.callwrc(0x021D_5FC0, &mut [Arg::V(ma), Arg::V(0xC)], w, cx);
        }
        if env.callwrc(0x021D_5B68, &mut [Arg::V(mb)], w, cx) == 0 {
            env.callwrc(0x021D_5FC0, &mut [Arg::V(mb), Arg::V(0xC)], w, cx);
        }
    }
}

/// C: `FUN_overlay_d_93__021bd4e4`: runs the builder wrapper `021bd518` for the party members
/// `i` and `j` of `client`.
pub fn announce_pair<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, client: u32, i: u32, j: u32) {
    let party = env.callw(
        0x021B_9AEC,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(client)],
        w,
    );
    let a = env.callw(0x021B_9C00, &mut [Arg::V(party), Arg::V(i)], w);
    let b = env.callw(0x021B_9C00, &mut [Arg::V(party), Arg::V(j)], w);
    queue_wrap_bd554(env, w, a);
    queue_wrap_bd554(env, w, b);
}

/// C: `FUN_overlay_d_93__021bd3e8`: when `mon` sits in party slot 0 or 2 of its client, swaps
/// it with slot 1 (`021bd43c`, forced stub: the caller has no command queue), tells the client
/// (`021c5d88(work, mon, 0xE7)`) and announces the pair (`021bd4e4`).
pub fn switch_slot_front<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32) {
    let id = mon_id(&mut Lend::new(env, w), mon);
    let client = client_of_id(&mut Lend::new(env, w), id);
    let party = env.callw(
        0x021B_9AEC,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(client)],
        w,
    );
    let idx = env.callw(0x021B_9C98, &mut [Arg::V(party), Arg::V(mon)], w);
    if idx == 0 || idx == 2 {
        env.callw(
            0x021B_D43C,
            &mut [
                Arg::P,
                Arg::V(client),
                Arg::V(idx & 0xFF),
                Arg::V(1),
                Arg::V(0),
            ],
            w,
        );
        env.callw(0x021C_5D88, &mut [Arg::P, Arg::V(mon), Arg::V(0xE7)], w);
        announce_pair(env, w, client, idx & 0xFF, 1);
    }
}

/// C: `FUN_overlay_d_93__021bd8f0`: switches the mon in party slot `r3` of `client` in:
/// puts command 0x2A, swaps slots `r2` / `r3` when they differ, prepares the new front mon
/// (`bpp`: `021b8750`, `021d8484`, `021dcf5c`, `021d63ec(bpp, work[0x10] & 0xFFFF)`, `021d6580`),
/// sets its flag at work + 0x7C1 + id (cleared again in battle type 3 when `021b6994 <= r2`),
/// puts command 0x12 and updates the helper struct at work + 0x1AB8 (`021bd9c4`).
pub fn switch_in<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    client: u32,
    r2: u32,
    r3: u32,
) {
    let main = w.u32_at(OFS_MAIN);
    let party = env.callwc(
        0x021B_9AEC,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(client)],
        w,
        cx,
    );
    cmd_queue::put_cmd(&mut Lend::new(env, w), cx.q, 0x2A, &[client], cx.scratch);
    env.callwc(
        0x021B_9D10,
        &mut [Arg::V(main), Arg::V(party), Arg::V(client)],
        w,
        cx,
    );
    if r2 != r3 {
        env.callwc(
            0x021B_9C10,
            &mut [Arg::V(party), Arg::V(r2), Arg::V(r3)],
            w,
            cx,
        );
    }
    let bpp = env.callwc(0x021B_9BF0, &mut [Arg::V(party), Arg::V(r2)], w, cx);
    let id = mon_id(&mut Lend::wc(env, w, cx), bpp);
    env.callwc(
        0x021B_8750,
        &mut [Arg::V(main), Arg::V(client), Arg::V(bpp)],
        w,
        cx,
    );
    env.callwc(0x021D_8484, &mut [Arg::V(bpp)], w, cx);
    env.callwc(0x021D_CF5C, &mut [Arg::V(bpp)], w, cx);
    let w10 = w.u32_at(OFS_WORD_10) & 0xFFFF;
    env.callwc(0x021D_63EC, &mut [Arg::V(bpp), Arg::V(w10)], w, cx);
    env.callwc(0x021D_6580, &mut [Arg::V(bpp)], w, cx);
    let flag = OFS_MON_FLAGS + id as usize;
    w.mem[flag] = 1;
    if env.callwc(0x021B_8588, &mut [Arg::V(main)], w, cx) == 3
        && env.callwc(0x021B_6994, &mut [Arg::V(main), Arg::V(client)], w, cx) <= r2
    {
        w.mem[flag] = 0;
    }
    cmd_queue::put_cmd(
        &mut Lend::new(env, w),
        cx.q,
        0x12,
        &[client, r2, r3, w10],
        cx.scratch,
    );
    let x = env.callwc(
        0x021B_8BEC,
        &mut [Arg::V(main), Arg::V(client), Arg::V(r2)],
        w,
        cx,
    );
    env.callwc(
        0x021B_D9C4,
        &mut [
            Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_1AB8)),
            Arg::V(x),
            Arg::V(id),
            Arg::V(w.u32_at(OFS_POKECON)),
        ],
        w,
        cx,
    );
}

/// C: `FUN_overlay_d_93__021bd9d4`: `021bd8f0(r1, r2, r3)` then puts command 0x3D
/// `(r1, r2, r3, r4 & 0xFF)`.
pub fn switch_in_cmd<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    r1: u32,
    r2: u32,
    r3: u32,
    r4: u32,
) {
    switch_in(env, w, cx, r1, r2, r3);
    cmd_queue::put_cmd(
        &mut Lend::new(env, w),
        cx.q,
        0x3D,
        &[r1, r2, r3, r4 & 0xFF],
        cx.scratch,
    );
}

/// C: `FUN_overlay_d_93__021bebd8`: puts command 0x30 `(021b8d8c(main, pokecon, id of mon), r3, r2, 0)`.
pub fn put_cmd_30<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    r2: u32,
    r3: u32,
) {
    let id = mon_id(&mut Lend::wrc(env, w, cx), mon);
    let x = env.callwrc(
        0x021B_8D8C,
        &mut [
            Arg::V(w.u32_at(OFS_MAIN)),
            Arg::V(w.u32_at(OFS_POKECON)),
            Arg::V(id),
        ],
        w,
        cx,
    );
    cmd_queue::put_cmd(
        &mut Lend::new_ref(env, w),
        cx.q,
        0x30,
        &[x, r3, r2, 0],
        cx.scratch,
    );
}

// ---- per-client send loop, faint messages ------------------------------------------------------

/// C: `FUN_overlay_d_93__021bda00`: sends the per-client messages of the helper struct at
/// work + 0x1A68 (veneers `021bdb28`..`021bdb78`): clears the flag at work + 0x7C1 + id of every
/// mon of the iteration (`021bd2b4` / `021bd34c`, forced stubs: the spec has no server-side
/// object) whose flag is set, then the three message groups, each bracketed with the queue
/// header word. Returns 1 when `021c88e4` answered non-zero for any of them.
pub fn send_client_messages<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, w_addr: u32) -> u32 {
    let helper = w_addr.wrapping_add(OFS_HELPER_1A68);
    let mut ret = 0;
    env.callw(0x021B_DB28, &mut [Arg::V(helper)], w);
    // the iterator state lives in the caller's frame
    let mut it = [0u8; 4];
    env.callw(0x021B_D2B4, &mut [Arg::PBuf(&mut it), Arg::P], w);
    let mut mon = [0u8; 4];
    let mut more = env.callw(
        0x021B_D34C,
        &mut [Arg::PBuf(&mut it), Arg::P, Arg::Out(&mut mon)],
        w,
    );
    while more != 0 {
        let m = u32::from_le_bytes(mon);
        let id = mon_id(&mut Lend::new(env, w), m);
        let flag = OFS_MON_FLAGS + id as usize;
        if w.mem[flag] != 0 {
            env.callw(0x021B_DB38, &mut [Arg::V(helper), Arg::V(m)], w);
            w.mem[flag] = 0;
        }
        more = env.callw(
            0x021B_D34C,
            &mut [Arg::PBuf(&mut it), Arg::P, Arg::Out(&mut mon)],
            w,
        );
    }
    env.callw(0x021B_DB48, &mut [Arg::V(helper), Arg::V(w_addr)], w);
    bracket(env, w, |env| {
        build_bdbec(env);
        if env.call(0x021C_88E4, &mut [Arg::P]) != 0 {
            ret = 1;
        }
    });
    env.callw(0x021B_DB58, &mut [Arg::V(helper)], w);
    let mut m = env.callw(0x021B_DB68, &mut [Arg::V(helper)], w);
    while m != 0 {
        bracket(env, w, |env| {
            build_bdb88(env, m);
            if env.call(0x021C_88E4, &mut [Arg::P]) != 0 {
                ret = 1;
            }
        });
        m = env.callw(0x021B_DB78, &mut [Arg::V(helper)], w);
    }
    bracket(env, w, |env| {
        build_bdc0c(env);
        if env.call(0x021C_88E4, &mut [Arg::P]) != 0 {
            ret = 1;
        }
    });
    ret
}

/// C: `FUN_overlay_d_93__021bdd7c`: a mon faints: puts command 0x3C `(021b8d8c(..), r2)` unless
/// that client is 6, marks its action record done (`021bd1d0`), builds message 0x54
/// (`021bde18`) bracketed with the header word, `021c5108(work, mon, 0)`, `021d64f0(mon)`, puts
/// command 0x20 (id of mon) and `021bde08(work + 0x1AB8, id)`.
pub fn faint_message<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, cx: &mut Cx, mon: u32, r2: u32) {
    let id = mon_id(&mut Lend::wc(env, w, cx), mon);
    let x = env.callwc(
        0x021B_8D8C,
        &mut [
            Arg::V(w.u32_at(OFS_MAIN)),
            Arg::V(w.u32_at(OFS_POKECON)),
            Arg::V(id),
        ],
        w,
        cx,
    );
    if x != 6 {
        cmd_queue::put_cmd(&mut Lend::new(env, w), cx.q, 0x3C, &[x, r2], cx.scratch);
    }
    mark_done(&mut Lend::c(env, cx), w, id);
    bracket(&mut Lend::c(env, cx), w, |env| build_bde18(env, mon));
    env.callwc(0x021C_5108, &mut [Arg::P, Arg::V(mon), Arg::V(0)], w, cx);
    env.callwc(0x021D_64F0, &mut [Arg::V(mon)], w, cx);
    let id2 = mon_id(&mut Lend::wc(env, w, cx), mon);
    cmd_queue::put_cmd(&mut Lend::new(env, w), cx.q, 0x20, &[id2], cx.scratch);
    env.callwc(
        0x021B_DE08,
        &mut [Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_1AB8)), Arg::V(id)],
        w,
        cx,
    );
}

/// C: `FUN_overlay_d_93__021bdd00`: `flag == 0`: with `n = 021c6190(work, mon)` non-zero, sets
/// bit 0 of work + 0x78A and runs `021bd0c0(work, work[0x78B + i], id)` for `i < n` while the
/// mon lives, then clears the bit. Then, if the mon is not fainted, `021bdd7c(work, mon,
/// 0x26C)` (forced stub) and returns 1; else 0.
pub fn faint_if_alive<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, flag: u32) -> u32 {
    if flag == 0 {
        let n = env.callw(0x021C_6190, &mut [Arg::P, Arg::V(mon)], w);
        if n != 0 {
            w.mem[OFS_FLAGS_78A] = (w.mem[OFS_FLAGS_78A] & 0xFE) | 1;
            let mut i = 0;
            loop {
                let id = mon_id(&mut Lend::new(env, w), mon);
                let b = w.mem[OFS_FLAGS_78A + 1 + i as usize] as u32;
                env.callw(0x021B_D0C0, &mut [Arg::P, Arg::V(b), Arg::V(id)], w);
                if env.callw(0x021D_5B68, &mut [Arg::V(mon)], w) != 0 {
                    break;
                }
                i += 1;
                if i >= n {
                    break;
                }
            }
            w.mem[OFS_FLAGS_78A] &= 0xFE;
        }
    }
    if env.callw(0x021D_5B68, &mut [Arg::V(mon)], w) == 0 {
        env.callw(0x021B_DD7C, &mut [Arg::P, Arg::V(mon), Arg::V(0x26C)], w);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021bdc98`: puts the client command of `mon` (`021bdc78`, forced stub)
/// and, when `faint_if_alive(mon, 0)` says it fainted, looks up its client / slot
/// (`021b8e28(main, client, &slot, &byte)`), counts it in `work[0x7D9 + slot]` (capped at 0xFF)
/// and runs `021bd9d4(work, slot, byte, r2, 1)` (forced stub).
pub fn faint_notify<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, r2: u32) {
    env.callw(0x021B_DC78, &mut [Arg::P, Arg::V(mon)], w);
    if faint_if_alive(env, w, mon, 0) != 0 {
        let id = mon_id(&mut Lend::new(env, w), mon);
        let client = env.callw(
            0x021B_8D8C,
            &mut [
                Arg::V(w.u32_at(OFS_MAIN)),
                Arg::V(w.u32_at(OFS_POKECON)),
                Arg::V(id),
            ],
            w,
        );
        let mut slot = [0u8; 1];
        let mut byte = [0u8; 1];
        env.callw(
            0x021B_8E28,
            &mut [
                Arg::V(w.u32_at(OFS_MAIN)),
                Arg::V(client),
                Arg::Out(&mut slot),
                Arg::Out(&mut byte),
            ],
            w,
        );
        let ofs = 0x7D9 + slot[0] as usize;
        w.mem[ofs] = w.mem[ofs].saturating_add(1);
        env.callw(
            0x021B_D9D4,
            &mut [
                Arg::P,
                Arg::V(slot[0] as u32),
                Arg::V(byte[0] as u32),
                Arg::V(r2),
                Arg::V(1),
            ],
            w,
        );
    }
}

// ---- switch decisions ------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bd708`: the messages when `mon` switches / is sent out: in a
/// battle with `021b86e4` set only the notice 0xC6 (command 0x5A, once, bit 2 of work + 0x78A);
/// otherwise the builders `021bd850` / `021bd8ac` (bracketed with the header word) and
/// command 0x5C notices 0x4B / 0x48 (`021b902c`, `021b86b8`). Always ends with
/// `021d8244(work + 0x4B8, client)`; returns 0 only when `021bd850` answered non-zero.
pub fn switch_messages<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    a: u32,
    b: u32,
) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let id = mon_id(&mut Lend::wc(env, w, cx), mon);
    let client = client_of_id(&mut Lend::wc(env, w, cx), id);
    let finish = |env: &mut E, cx: &Cx| {
        env.call(
            0x021D_8244,
            &mut [
                Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_4B8)),
                Arg::V(client),
            ],
        );
        1
    };
    if env.callwc(0x021B_86E4, &mut [Arg::V(main)], w, cx) != 0 {
        if env.callwc(0x021B_86C0, &mut [Arg::V(main)], w, cx) != 0 && w.mem[OFS_FLAGS_78A] & 4 == 0
        {
            cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5A, 0xC6, &[LIST_END]);
            w.mem[OFS_FLAGS_78A] |= 4;
        }
        return finish(env, cx);
    }
    let ours = env.callwc(0x021B_8FF4, &mut [Arg::V(main)], w, cx);
    let same = env.callwc(0x021B_902C, &mut [Arg::V(client), Arg::V(ours)], w, cx) == 0;
    if a == 0 && env.callwc(0x021D_5B68, &mut [Arg::V(mon)], w, cx) == 0 {
        let mut r = 0;
        bracket(&mut Lend::c(env, cx), w, |env| r = build_bd850(env, mon));
        if r != 0 {
            return 0;
        }
    }
    if w.mem[OFS_FLAGS_78A] & 4 != 0 {
        return finish(env, cx);
    }
    let mut r = 0;
    if b == 0 {
        bracket(&mut Lend::c(env, cx), w, |env| r = build_bd8ac(env, mon));
    }
    if r == 0 {
        if same {
            if env.callwc(0x021B_86B8, &mut [Arg::V(main)], w, cx) == 0 {
                let id2 = mon_id(&mut Lend::wc(env, w, cx), mon);
                cmd_queue::put_cmd_list(
                    &mut Lend::new(env, w),
                    cx.q,
                    0x5C,
                    0x4B,
                    &[0x56A, id2, LIST_END],
                );
            }
        } else {
            cmd_queue::put_cmd_list(&mut Lend::new(env, w), cx.q, 0x5C, 0x48, &[0x56A, LIST_END]);
        }
    }
    w.mem[OFS_FLAGS_78A] |= 4;
    finish(env, cx)
}

/// C: `FUN_overlay_d_93__021bd60c`: whether `mon` can be switched out: `local` = 1 and `flag` = 1
/// when `021b86e4`; else when the mon is on our client `local = 021bd6b8`. With `flag` still 0
/// and a mon from `021b9920(pokecon, 1, 0)`: compares the two mons' `021d58a8(.., 0xC)` values
/// (u16) and, when the first is not larger, a random draw (`021d78ac(0x100)`, divide by
/// `0209c0a4`, scaled by the counter at work + 0x786) decides: draw >= bound returns 0.
/// Otherwise `021bd708(work, mon, local, 0)` (forced stub).
pub fn can_switch_out<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32, flag: u32) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let mut flag = flag;
    let mut local = 0;
    if env.callw(0x021B_86E4, &mut [Arg::V(main)], w) == 0 {
        let id = mon_id(&mut Lend::new(env, w), mon);
        let client = client_of_id(&mut Lend::new(env, w), id);
        let ours = env.callw(0x021B_8FF4, &mut [Arg::V(main)], w);
        if client != ours {
            flag = 1;
        } else {
            local = build_bd6b8(&mut Lend::new(env, w), mon);
            if local != 0 {
                flag = 1;
            }
        }
    } else {
        flag = 1;
        local = 1;
    }
    if flag == 0 {
        let m = env.callw(
            0x021B_9920,
            &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(1), Arg::V(0)],
            w,
        );
        if m != 0 {
            let u3 = env.callw(0x021D_58A8, &mut [Arg::V(mon), Arg::V(0xC)], w) & 0xFFFF;
            let u4 = env.callw(0x021D_58A8, &mut [Arg::V(m), Arg::V(0xC)], w) & 0xFFFF;
            if u3 <= u4 {
                let q = env.callw(0x0209_C0A4, &mut [Arg::V(u3 << 12), Arg::V(u4)], w);
                let scaled = ((q << 7) as i32) >> 12;
                let cnt = w.mem[OFS_COUNTER_786] as u32;
                let bound = (scaled as u32).wrapping_add(cnt * 0x1E);
                if env.callw(0x021D_78AC, &mut [Arg::V(0x100)], w) >= bound {
                    return 0;
                }
            }
        }
    }
    env.callw(
        0x021B_D708,
        &mut [Arg::P, Arg::V(mon), Arg::V(local), Arg::V(0)],
        w,
    )
}

/// C: `FUN_overlay_d_93__021bd584`: `can_switch_out(mon, 0)`; non-zero returns 1. Otherwise
/// puts a notice: for our own client `021ba1b4(main, 0x4E)` and command 0x5A (0x49), else command
/// 0x5B (0x36B, id of mon); for our own client also counts it in work + 0x786 (cap 0x1E).
/// Returns 0.
pub fn switch_out_notice<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    q: &mut [u8],
    mon: u32,
) -> u32 {
    let main = w.u32_at(OFS_MAIN);
    let id = mon_id(&mut Lend::wh(env, w, vec![&mut *q]), mon);
    let client = client_of_id(&mut Lend::wh(env, w, vec![&mut *q]), id);
    let ours = env.call_l(
        0x021B_8FF4,
        &mut [Arg::V(main)],
        Lent::of(Some(&mut *w), None, vec![&mut *q]),
    );
    let r = can_switch_out(&mut Lend::hv(env, vec![&mut *q]), w, mon, 0);
    if r != 0 {
        return 1;
    }
    if client == ours {
        env.call_l(
            0x021B_A1B4,
            &mut [Arg::V(main), Arg::V(0x4E)],
            Lent::of(Some(&mut *w), None, vec![&mut *q]),
        );
        cmd_queue::put_cmd_list(&mut Lend::new(env, w), q, 0x5A, 0x49, &[LIST_END]);
    } else {
        let id2 = mon_id(&mut Lend::wh(env, w, vec![&mut *q]), mon);
        cmd_queue::put_cmd_list(&mut Lend::new(env, w), q, 0x5B, 0x36B, &[id2, LIST_END]);
    }
    if client == ours {
        let c = w.mem[OFS_COUNTER_786].wrapping_add(1);
        w.mem[OFS_COUNTER_786] = if c > 0x1E { 0x1E } else { c };
    }
    r
}

// ---- move-effect helpers ------------------------------------------------------------------------

/// `work + 0x4A8`: pointer to the flag struct `apply_move_category` works on; `+0x1AB0`: pointer to a
/// 20-byte record; `+0x860` / `0x864`: pointers passed on.
const OFS_PTR_4A8: usize = 0x4A8;
const OFS_PTR_1AB0: usize = 0x1AB0;
const OFS_PTR_864: usize = 0x864;
const OFS_PTR_860: usize = 0x860;
/// `work + 0x1F88`: a 24-word table (`events::table_fill_6`), `+0x1C`: a helper struct.
const OFS_TABLE_1F88: usize = 0x1F88;
const OFS_HELPER_1C: u32 = 0x1C;

/// The move id in bits 7..22 of a move word.
fn move_id_of(word: u32) -> u32 {
    (word & 0x7F_FFFF) >> 7
}

fn u16_at(b: &[u8]) -> u32 {
    u16::from_le_bytes([b[0], b[1]]) as u32
}

/// C: `FUN_overlay_d_93__021bde48`: when `mon` has the status 0x17 (`021d62c4`) and the move word
/// does not name move 0xA5, asks for a replacement move (`021e8b80(021d6314(mon, 0x17))`);
/// returns it if it differs from the word's move and `021d55a0(mon, move)` accepts it, else 0.
pub fn replacement_move<E: FlowEnv>(env: &mut E, mon: u32, word: u32) -> u32 {
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x17)]) != 0 && move_id_of(word) != 0xA5 {
        let r = env.call(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x17)]);
        let mv = env.call(0x021E_8B80, &mut [Arg::V(r)]);
        if mv != move_id_of(word) && env.call(0x021D_55A0, &mut [Arg::V(mon), Arg::V(mv)]) != 0 {
            return mv;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021be3a8`: picks the side effect 0xD (mon has status 0xF and
/// `0201beb8(move, 0xC)`) or 0x14 (`021ef948(2)` and `0201beb8(move, 9)`) for the move `rec.u16[0]`
/// and runs `021c0450(work, mon, move, effect)`; returns 1 when there is one.
pub fn side_effect<E: FlowEnv>(env: &mut E, mon: u32, rec: &[u8]) -> u32 {
    let m = u16_at(rec);
    let mut effect = 0;
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0xF)]) != 0
        && env.call(0x0201_BEB8, &mut [Arg::V(m), Arg::V(0xC)]) != 0
    {
        effect = 0xD;
    } else if env.call(0x021E_F948, &mut [Arg::V(2)]) != 0
        && env.call(0x0201_BEB8, &mut [Arg::V(m), Arg::V(9)]) != 0
    {
        effect = 0x14;
    }
    if effect != 0 {
        env.call(
            0x021C_0450,
            &mut [Arg::P, Arg::V(mon), Arg::V(m), Arg::V(effect)],
        );
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021be400`: for every entry of the per-turn list `l`
/// (`{u8 n; u8 mons[6]; u8 b[6]; u8 c[6]}`, work + 0x4B0): the mon `a` (list id in `+1+i`) and `b`
/// (`+0xD+i`) from the party container; when `021d5b7c(a)` runs the message `021bea60(a, b, x)`
/// (bracketed with the queue header word), `021c64b4`, sets bit 0 of the u32 at `d + 0x10` (the
/// 20-byte record at work + 0x1AB0), `021ca7f4`, `021d7394`, `021e02dc`, `021bec58`
/// (forced stub), `021e03d8` and `021d73d4`.
pub fn run_pending_list<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    w_addr: u32,
    l: &[u8],
    d: &mut [u8],
    x: u32,
) {
    if l[0] == 0 {
        return;
    }
    let mut i = 0usize;
    loop {
        let a = env.callw(
            0x021B_9934,
            &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(l[i + 1] as u32)],
            w,
        );
        let b = env.callw(
            0x021B_9934,
            &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(l[i + 0xD] as u32)],
            w,
        );
        if env.callw(0x021D_5B7C, &mut [Arg::V(a)], w) != 0 {
            bracket(env, w, |env| build_bea60(env, a, b, x));
            let dp = w.u32_at(OFS_PTR_1AB0);
            env.callw(
                0x021C_64B4,
                &mut [Arg::V(w_addr), Arg::V(x), Arg::V(a), Arg::V(dp)],
                w,
            );
            let v = u32::from_le_bytes(d[0x10..0x14].try_into().unwrap());
            d[0x10..0x14].copy_from_slice(&((v & !1) | 1).to_le_bytes());
            env.callw(
                0x021C_A7F4,
                &mut [
                    Arg::P,
                    Arg::V(a),
                    Arg::V(l[i + 7] as u32),
                    Arg::V(dp),
                    Arg::V(w.u32_at(OFS_PTR_864)),
                ],
                w,
            );
            let id = mon_id(&mut Lend::new(env, w), a);
            env.callw(0x021D_7394, &mut [Arg::V(id)], w);
            let g = env.callw(0x021D_59B4, &mut [Arg::V(a), Arg::V(0xC)], w);
            env.callw(0x021E_02DC, &mut [Arg::V(a), Arg::V(x), Arg::V(g)], w);
            env.callw(
                0x021B_EC58,
                &mut [Arg::P, Arg::V(a), Arg::V(x), Arg::V(w.u32_at(OFS_PTR_864))],
                w,
            );
            env.callw(0x021E_03D8, &mut [Arg::V(a), Arg::V(x)], w);
            let id = mon_id(&mut Lend::new(env, w), a);
            env.callw(0x021D_73D4, &mut [Arg::V(id)], w);
        }
        i += 1;
        if i >= l[0] as usize {
            break;
        }
    }
}

/// C: `FUN_overlay_d_93__021be550`: as one step of `run_pending_list` for the list `l` (work +
/// 0x4AC) with message `021bea14(a, r1, r2)`: effect 0xD of `021c0520` when mon `a` has status
/// 0xF and `0201beb8(r2, 0xC)`, else `021e02dc`, `021bec58` (forced stub) and `021e03d8`.
pub fn run_pending_one<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    w_addr: u32,
    l: &[u8],
    r1: u32,
    r2: u32,
) {
    if l[0] == 0 {
        return;
    }
    let a = env.callw(
        0x021B_9934,
        &mut [Arg::V(w.u32_at(OFS_POKECON)), Arg::V(l[1] as u32)],
        w,
    );
    bracket(env, w, |env| build_bea14(env, a, r1, r2));
    let dp = w.u32_at(OFS_PTR_1AB0);
    env.callw(
        0x021C_64B4,
        &mut [Arg::V(w_addr), Arg::V(r2), Arg::V(a), Arg::V(dp)],
        w,
    );
    env.callw(
        0x021C_A7F4,
        &mut [
            Arg::P,
            Arg::V(a),
            Arg::V(l[7] as u32),
            Arg::V(dp),
            Arg::V(w.u32_at(OFS_PTR_864)),
        ],
        w,
    );
    if env.callw(0x021D_62C4, &mut [Arg::V(a), Arg::V(0xF)], w) != 0
        && env.callw(0x0201_BEB8, &mut [Arg::V(r2), Arg::V(0xC)], w) != 0
    {
        env.callw(
            0x021C_0520,
            &mut [Arg::P, Arg::V(a), Arg::V(r2), Arg::V(0xD)],
            w,
        );
        return;
    }
    let g = env.callw(0x021D_59B4, &mut [Arg::V(a), Arg::V(0xC)], w);
    env.callw(0x021E_02DC, &mut [Arg::V(a), Arg::V(r2), Arg::V(g)], w);
    env.callw(
        0x021B_EC58,
        &mut [Arg::P, Arg::V(a), Arg::V(r2), Arg::V(w.u32_at(OFS_PTR_864))],
        w,
    );
    env.callw(0x021E_03D8, &mut [Arg::V(a), Arg::V(r2)], w);
}

/// C: `FUN_overlay_d_93__021beab4`: `021beb14` lists the mons for `list`; when its slot-2 reply
/// is not 0x1F, fills the record `out`: `+7` = `021beb04(work + 0x1AB8)` (6 when the slot-4
/// reply is 0x1F), `+1` = the slot-2 reply, `+0` = 1; returns 1. Otherwise 0.
pub fn list_mons_reply<E: FlowEnv>(
    env: &mut E,
    w_addr: u32,
    mon: u32,
    x: u32,
    list: u32,
    out: &mut [u8],
) -> u32 {
    let mut b9 = 0x1Fu8;
    let mut b8 = 0x1Fu8;
    build_beb14(env, mon, x, list, &mut b9, &mut b8);
    if b9 == 0x1F {
        return 0;
    }
    let v = if b8 != 0x1F {
        // the original also passes the byte `b8` in r1; the spec's stub takes one argument
        env.call(
            0x021B_EB04,
            &mut [Arg::V(w_addr.wrapping_add(OFS_HELPER_1AB8))],
        )
    } else {
        6
    };
    out[7] = v as u8;
    out[1] = b9;
    out[0] = 1;
    1
}

/// The table of three ids at 0x021EFDB8 (`u16`).
const TABLE_EFDB8: [u32; 3] = [0x207, 0x206, 0x208];

/// C: `FUN_overlay_d_93__021be6ec`: for the ids of `TABLE_EFDB8` (`r2` must be one of them) and a
/// mon without status 10 / `021d6e74`: collects up to two pending records of the other ids
/// (`core1::find_pending_alive(id, mon id)`), picks the one with the lowest index
/// (`core1::record_index`) and swaps its mon in: `021d5fc0(mon, 10)`, `021d6e40`, command 0x5B
/// and `core1::promote_record`. Returns 1 when it did.
pub fn swap_with_pending<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    w_addr: u32,
    q: &mut [u8],
    mon: u32,
    r2: u32,
) -> u32 {
    if !TABLE_EFDB8.contains(&r2) {
        return 0;
    }
    if env.call_l(
        0x021D_5BB0,
        &mut [Arg::V(mon), Arg::V(10)],
        Lent::of(Some(&mut *w), None, vec![&mut *q]),
    ) != 0
    {
        return 0;
    }
    if env.call_l(
        0x021D_6E74,
        &mut [Arg::V(mon)],
        Lent::of(Some(&mut *w), None, vec![&mut *q]),
    ) != 0
    {
        return 0;
    }
    let id = mon_id(&mut Lend::wh(env, w, vec![&mut *q]), mon);
    let mut found = [0usize; 2];
    let mut cnt = 0;
    for kind in TABLE_EFDB8 {
        if kind == r2 {
            continue;
        }
        if let Some(rec) =
            core1::find_pending_alive(&mut Lend::hv(env, vec![&mut *q]), w, w_addr, kind, id)
        {
            found[cnt] = rec;
            cnt += 1;
            if cnt >= 2 {
                break;
            }
        }
    }
    if cnt == 0 {
        return 0;
    }
    let mut best = 6u32;
    let mut sel = w.u32_at(found[0]);
    for &rec in &found[..cnt] {
        let idx = core1::record_index(w, rec as u32) as u32;
        if idx < best {
            best = idx;
            sel = w.u32_at(rec);
        }
    }
    let id2 = mon_id(&mut Lend::wh(env, w, vec![&mut *q]), sel);
    env.call_l(
        0x021D_5FC0,
        &mut [Arg::V(mon), Arg::V(10)],
        Lent::of(Some(&mut *w), None, vec![&mut *q]),
    );
    env.call_l(
        0x021D_6E40,
        &mut [Arg::V(sel), Arg::V(id), Arg::V(r2)],
        Lent::of(Some(&mut *w), None, vec![&mut *q]),
    );
    cmd_queue::put_cmd_list(&mut Lend::new(env, w), q, 0x5B, 0x47A, &[id, id2, LIST_END]);
    core1::promote_record(&mut Lend::hv(env, vec![&mut *q]), w, id2);
    1
}

/// C: `FUN_overlay_d_93__021be7e4`: whole call bracketed with the queue header word: `be878(mon,
/// r3)`; when it answers non-zero: in `021c88e4(work) == 2` puts command 0x30 `(021b8d8c(..), r3,
/// r2, 0)`, `021be8c4(mon)` and `*out = 1`, else `021c5718(work, mon, r2)`. `*out` is 0
/// otherwise. Returns the `be878` answer.
pub fn flag_mon_effect<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    mon: u32,
    r2: u32,
    r3: u32,
    out: &mut u8,
) -> u32 {
    let old = cmd_queue::advance_fields(&mut w.mem[OFS_QUEUE_HEADER..]);
    let rv = build_be878(&mut Lend::wc(env, w, cx), mon, r3);
    *out = 0;
    if rv != 0 {
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        let x = env.callwc(
            0x021B_8D8C,
            &mut [
                Arg::V(w.u32_at(OFS_MAIN)),
                Arg::V(w.u32_at(OFS_POKECON)),
                Arg::V(id),
            ],
            w,
            cx,
        );
        if env.callwc(0x021C_88E4, &mut [Arg::P], w, cx) == 2 {
            cmd_queue::put_cmd(
                &mut Lend::new(env, w),
                cx.q,
                0x30,
                &[x, r3, r2, 0],
                cx.scratch,
            );
            queue_wrap_mon_builder(&mut Lend::c(env, cx), w, mon);
            *out = 1;
        } else {
            env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
        }
    }
    cmd_queue::set_word(&mut w.mem[OFS_QUEUE_HEADER..], old);
    rv
}

// ---- loops over the targets of a veneer iterator ----------------------------------------------------

/// C: `FUN_overlay_d_93__021bf04c`: for every target `t` of the iterator `it` (veneers
/// `021bf0a4` / `021bf0b4` / `021bf0d4`) for which `021bfa00(work, r2, t, move)` answers non-zero:
/// `021bf0c4(it, t)`, `021c572c(work, t, move)` and bit 4 of work + 0x78A (`move` =
/// `mv.u16[0]`).
pub fn apply_to_targets<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mv: &[u8], r2: u32, it: u32) {
    env.callw(0x021B_F0A4, &mut [Arg::V(it)], w);
    let mut t = env.callw(0x021B_F0B4, &mut [Arg::V(it)], w);
    let m = u16_at(mv);
    while t != 0 {
        if env.callw(
            0x021B_FA00,
            &mut [Arg::P, Arg::V(r2), Arg::V(t), Arg::V(m)],
            w,
        ) != 0
        {
            env.callw(0x021B_F0C4, &mut [Arg::V(it), Arg::V(t)], w);
            env.callw(0x021C_572C, &mut [Arg::P, Arg::V(t)], w);
            w.mem[OFS_FLAGS_78A] |= 0x10;
        }
        t = env.callw(0x021B_F0D4, &mut [Arg::V(it)], w);
    }
}

/// C: `FUN_overlay_d_93__021bf0e4`: when `021bf138(it) == 1` and the first target `t`
/// (`021bf148(it, 0)`) has the same id as `mon`, `rec.u32[3] != 7` and (`021d5bb0(mon, 0xC)` or
/// `flag`): `021bf158(it, t)`.
pub fn add_first_target<E: FlowEnv>(env: &mut E, mon: u32, it: u32, rec: &[u8], flag: u32) {
    if env.call(0x021B_F138, &mut [Arg::V(it)]) != 1 {
        return;
    }
    let t = env.call(0x021B_F148, &mut [Arg::V(it), Arg::V(0)]);
    if t == 0 {
        return;
    }
    let a = mon_id(env, t);
    let b = mon_id(env, mon);
    let kind = u32::from_le_bytes(rec[0xC..0x10].try_into().unwrap());
    if a == b
        && kind != 7
        && (env.call(0x021D_5BB0, &mut [Arg::V(mon), Arg::V(0xC)]) != 0 || flag != 0)
    {
        env.call(0x021B_F158, &mut [Arg::V(it), Arg::V(t)]);
    }
}

/// C: `FUN_overlay_d_93__021bf1b4`: for every target `t` of the iterator `it` (veneers
/// `021bf204` / `021bf214` / `021bf234`) other than `mon` that `021d6d7c` accepts, when `flag` is 0
/// and `0201beb8(mv.u16[0], 0xD)` is 0: `021bf224(it, t)`.
pub fn add_other_targets<E: FlowEnv>(env: &mut E, mv: &[u8], mon: u32, it: u32, flag: u32) {
    env.call(0x021B_F204, &mut [Arg::V(it)]);
    let mut t = env.call(0x021B_F214, &mut [Arg::V(it)]);
    while t != 0 {
        if mon != t
            && env.call(0x021D_6D7C, &mut [Arg::V(t)]) != 0
            && flag == 0
            && env.call(0x0201_BEB8, &mut [Arg::V(u16_at(mv)), Arg::V(0xD)]) == 0
        {
            env.call(0x021B_F224, &mut [Arg::V(it), Arg::V(t)]);
        }
        t = env.call(0x021B_F234, &mut [Arg::V(it)]);
    }
}

// ---- apply the effect of a move ------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021bec58`: applies the effect of the move / ability `r2` of `mon`
/// (`r3` is a target word); `p` is the flag struct at work + 0x4A8 (`+5`: bit 0 "started", bit 1)
/// and `D = work[0x1AB0]` the move record pointer the veneer functions get. The category is
/// `0201c1bc(r2)`; `021bfc08` may end the call early (1, 3: return 0; 2: return 1). Returns 1
/// when the effect is handled, 0 when a message was put instead. The loops over the targets
/// (`021bf04c`, `021bf0e4`, `021bf1b4`) and `021bf168` are forced stubs: their ports take the
/// record `D` points to.
pub fn apply_move_category<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    p: &mut [u8],
    mon: u32,
    r2: u32,
    r3: u32,
) -> u32 {
    let cat = env.callwc(0x0201_C1BC, &mut [Arg::V(r2)], w, cx);
    let id = mon_id(&mut Lend::wc(env, w, cx), mon);
    let mut handled = 1u32;
    let mut l2c = 0u32;
    let mut l30 = 0u32;
    notify_special_id(&mut Lend::c(env, cx), w, mon, r2);
    if env.callwc(0x0201_BEB8, &mut [Arg::V(r2), Arg::V(1)], w, cx) != 0 {
        let r = env.callwc(
            0x021B_FC08,
            &mut [Arg::P, Arg::V(mon), Arg::V(r3), Arg::V(r2)],
            w,
            cx,
        );
        match r {
            1 | 3 => return l2c,
            2 => return handled,
            _ => {}
        }
    }
    env.callwc(0x021B_EFEC, &mut [Arg::V(w.u32_at(OFS_PTR_860))], w, cx);
    let x = cmd_queue::put_cmd_placeholder(&mut Lend::new(env, w), cx.q, 0x30, cx.scratch);
    fill_switch_record(&mut Lend::new(env, w), p, cx.w_addr, mon, r3);
    let old = cmd_queue::advance_fields(&mut w.mem[OFS_QUEUE_HEADER..]);
    let iv = env.callwc(
        0x021C_65A8,
        &mut [Arg::P, Arg::V(mon), Arg::V(r2), Arg::V(r3)],
        w,
        cx,
    );
    cmd_queue::set_word(&mut w.mem[OFS_QUEUE_HEADER..], old);
    if iv != 0 {
        if p[5] & 1 == 0 {
            p[5] = (p[5] & 0xFE) | 1;
        }
        if iv == 2 {
            env.callwc(
                0x021B_FBD0,
                &mut [Arg::P, Arg::V(r2), Arg::V(w.u32_at(OFS_PTR_4A8)), Arg::V(x)],
                w,
                cx,
            );
            return 1;
        }
    }
    match cat {
        0 | 4 | 6 | 7 | 8 => {
            l2c = 1;
            l30 = 1;
        }
        9 => l30 = 1,
        _ => {}
    }
    env.callwc(0x021B_EFFC, &mut [Arg::V(r3)], w, cx);
    let d = w.u32_at(OFS_PTR_1AB0);
    let table = cx.w_addr.wrapping_add(OFS_TABLE_1F88 as u32);
    env.callwc(
        0x021B_F04C,
        &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
        w,
        cx,
    );
    env.callwc(
        0x021B_F0E4,
        &mut [Arg::P, Arg::V(mon), Arg::V(r3), Arg::V(d), Arg::V(l2c)],
        w,
        cx,
    );
    if env.callwc(0x021B_F00C, &mut [Arg::V(r3)], w, cx) != 0 {
        if w.mem[OFS_FLAGS_78A] & 0x10 == 0 {
            env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
        }
        handled = 0;
    }
    if handled != 0 {
        if cat != 9 {
            env.callwc(
                0x021B_F83C,
                &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
                w,
                cx,
            );
        }
        events::table_fill_6(&mut w.mem[OFS_TABLE_1F88..OFS_TABLE_1F88 + 0x60]);
        if l30 != 0 || env.callwc(0x021B_F168, &mut [Arg::P, Arg::V(d), Arg::V(mon)], w, cx) != 0 {
            env.callwc(
                0x021B_F3D8,
                &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3), Arg::V(table)],
                w,
                cx,
            );
        }
        env.callwc(
            0x021B_F4AC,
            &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3), Arg::V(table)],
            w,
            cx,
        );
        if cat != 9 {
            env.callwc(
                0x021B_F8D4,
                &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
                w,
                cx,
            );
        }
        let mut done = env.callwc(0x021B_F01C, &mut [Arg::V(r3)], w, cx) != 0;
        if !done {
            env.callwc(
                0x021B_F1B4,
                &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3), Arg::V(l30)],
                w,
                cx,
            );
            if env.callwc(0x021B_F02C, &mut [Arg::V(r3)], w, cx) != 0 {
                env.callwc(0x021C_5718, &mut [Arg::P], w, cx);
                done = true;
            }
        }
        if done {
            queue_wrap_bf27c(&mut Lend::c(env, cx), w, id, r2);
            handled = 0;
        }
    }
    if handled == 0 {
        if p[5] & 1 != 0 {
            env.callwc(
                0x021B_FBD0,
                &mut [Arg::P, Arg::V(r2), Arg::V(w.u32_at(OFS_PTR_4A8)), Arg::V(x)],
                w,
                cx,
            );
        }
    } else {
        if l2c != 0 {
            env.callwc(
                0x021C_091C,
                &mut [
                    Arg::P,
                    Arg::V(d),
                    Arg::V(mon),
                    Arg::V(r3),
                    Arg::V(table),
                    Arg::V(0),
                ],
                w,
                cx,
            );
        } else {
            match cat {
                1 => env.callwc(
                    0x021C_2488,
                    &mut [Arg::P, Arg::V(r2), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                2 => env.callwc(
                    0x021C_2DC4,
                    &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                3 => env.callwc(
                    0x021C_3274,
                    &mut [Arg::P, Arg::V(r2), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                5 => env.callwc(
                    0x021C_3184,
                    &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                9 => env.callwc(
                    0x021C_3438,
                    &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                10 => env.callwc(0x021C_3AA8, &mut [Arg::P, Arg::V(d), Arg::V(mon)], w, cx),
                11 | 13 => env.callwc(
                    0x021C_3D08,
                    &mut [Arg::P, Arg::V(d), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                12 => env.callwc(
                    0x021C_376C,
                    &mut [Arg::P, Arg::V(r2), Arg::V(mon), Arg::V(r3)],
                    w,
                    cx,
                ),
                _ => 0,
            };
        }
        let b = p[5];
        if b & 1 != 0 {
            if b & 2 == 0 {
                env.callwc(
                    0x021B_FBD0,
                    &mut [Arg::P, Arg::V(r2), Arg::V(w.u32_at(OFS_PTR_4A8)), Arg::V(x)],
                    w,
                    cx,
                );
            }
            env.callwc(
                0x021B_F03C,
                &mut [Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_1C))],
                w,
                cx,
            );
            if env.callwc(0x021D_5B68, &mut [Arg::V(mon)], w, cx) == 0
                && env.callwc(0x0201_BEB8, &mut [Arg::V(r2), Arg::V(2)], w, cx) != 0
            {
                env.callwc(0x021C_60EC, &mut [Arg::P, Arg::V(mon), Arg::V(0xC)], w, cx);
            }
            queue_wrap_bf244(&mut Lend::c(env, cx), w, id, r2);
        } else {
            queue_wrap_bf27c(&mut Lend::c(env, cx), w, id, r2);
            handled = 0;
        }
    }
    queue_wrap_bf2b4(&mut Lend::c(env, cx), w, id, r2);
    env.callwc(0x021C_4F64, &mut [Arg::P, Arg::V(mon)], w, cx);
    handled
}

// ---- use a move -----------------------------------------------------------------------------------------

/// The data `use_move` reads and writes through pointers in the work: the move word (the
/// function's third argument), the 20-byte records at work + 0x1AB0 (`d`, patched) and
/// 0x1AB4 (`e`, copied over `d` when there is no replacement move).
pub struct MoveCtx<'a> {
    pub d: &'a mut [u8],
    pub e: &'a mut [u8],
    /// The move word: bits 7..22 = move id, bits 4..6 = target slot.
    pub word: &'a mut [u8],
    /// The address of the move word (stubs see it).
    pub word_addr: u32,
}

const OFS_PTR_850: usize = 0x850;
const OFS_PTR_854: usize = 0x854;
const OFS_PTR_4B0: usize = 0x4B0;
const OFS_PTR_4AC: usize = 0x4AC;
const OFS_PTR_1AB4: usize = 0x1AB4;
const OFS_WORD_1F74: usize = 0x1F74;
const OFS_HELPER_1AE4: u32 = 0x1AE4;

/// B16: the move records `D` / `E` (`*(work + 0x1AB0)` / `+ 0x1AB4`) are part of the flow work in
/// the game (work + 0xD04 / 0xD18) and separate regions in the differential test
/// ([`SvFlowWork::pull`] / [`SvFlowWork::push`]).
fn rec_pull(w: &SvFlowWork, waddr: u32, p: u32, rec: &mut [u8]) {
    w.pull(waddr, p, rec);
}

fn rec_push(w: &mut SvFlowWork, waddr: u32, p: u32, rec: &[u8]) {
    w.push(waddr, p, rec);
}

/// C: `FUN_overlay_d_93__021bde98`: uses the move of the word `m.word` for `mon` (`x` is passed on to
/// `021e02dc`): the whole attack sequence. Resets the flag structs at work + 0x4A8 / 0x4B0
/// (forced stubs: the pointers are not declared regions), replaces the move (`replacement_move`),
/// puts the move-use message (bracketed), asks `021c001c` / `021c62d8` whether the move is
/// replaced (the u16 / u8 it fills in), patches the records `m.d` / `m.e`, runs the effect
/// (`flag_mon_effect`, then `021be6ec`, `021beab4`, `021be550` or `021bec58`; forced stubs: their
/// ports take regions the spec does not provide here) and finishes with `021d6744` and command
/// 0x18 (or `021c0780`), `021bfe60`, command 0x31, `021be994`, `021e0394` and `021be400`
/// (forced stub).
pub fn use_move<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    cx: &mut Cx,
    m: &mut MoveCtx,
    mon: u32,
    x: u32,
) {
    let main = w.u32_at(OFS_MAIN);
    let pokecon = w.u32_at(OFS_POKECON);
    let w850 = w.u32_at(OFS_PTR_850);
    let w854 = w.u32_at(OFS_PTR_854);
    let kind_before = env.callwc(0x021D_5C08, &mut [Arg::V(mon)], w, cx);
    env.callwc(0x021B_D1E0, &mut [Arg::V(w.u32_at(OFS_PTR_4A8))], w, cx);
    env.callwc(0x021B_E518, &mut [Arg::V(w.u32_at(OFS_PTR_4B0))], w, cx);
    let word = u32::from_le_bytes(m.word[..4].try_into().unwrap());
    let rep = replacement_move(&mut Lend::wc(env, w, cx), mon, word);
    if rep != 0 {
        let t = env.callwc(
            0x021D_80BC,
            &mut [Arg::V(main), Arg::V(pokecon), Arg::V(mon), Arg::V(rep)],
            w,
            cx,
        );
        env.callwc(
            0x021D_82D0,
            &mut [Arg::V(m.word_addr), Arg::V(rep), Arg::V(t)],
            w,
            cx,
        );
    }
    let word = u32::from_le_bytes(m.word[..4].try_into().unwrap());
    let mv = move_id_of(word);
    let mut r7 = mv;
    let kind = env.callwc(0x021D_5778, &mut [Arg::V(mon), Arg::V(mv)], w, cx);
    let mut slot = (word & 0x7F) >> 4;
    let mut done_flag = 0u32;
    let mut res = 0u8;
    let f19 = env.callwc(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x19)], w, cx) & 0xFF;
    let f1a = env.callwc(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x1A)], w, cx) & 0xFF;
    env.callwc(
        0x021E_02DC,
        &mut [Arg::V(mon), Arg::V(mv), Arg::V(x)],
        w,
        cx,
    );
    queue_wrap_be95c(&mut Lend::c(env, cx), w, mon, mv);
    let arg = if f19 != 0 || f1a != 0 { 1 } else { done_flag };
    let mut local_1a = 0u32;
    'body: {
        if env.callwc(
            0x021C_001C,
            &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(arg & 0xFF)],
            w,
            cx,
        ) != 0
        {
            break 'body;
        }
        let mut out3 = [0u8, 0, 6];
        let a = env.callwc(
            0x021C_62D8,
            &mut [
                Arg::V(cx.w_addr),
                Arg::V(mon),
                Arg::V(mv),
                Arg::V(slot),
                Arg::Out(&mut out3),
            ],
            w,
            cx,
        );
        local_1a = u16_at(&out3);
        let local_18 = out3[2] as u32;
        let e_ptr = w.u32_at(OFS_PTR_1AB4);
        let d_ptr = w.u32_at(OFS_PTR_1AB0);
        if a == 0 {
            env.callwc(0x021C_5604, &mut [Arg::P, Arg::V(mon), Arg::V(mv)], w, cx);
            env.callwc(
                0x021C_64B4,
                &mut [Arg::V(cx.w_addr), Arg::V(mv), Arg::V(mon), Arg::V(e_ptr)],
                w,
                cx,
            );
            env.callwc(
                0x021C_A7F4,
                &mut [
                    Arg::P,
                    Arg::V(mon),
                    Arg::V(slot),
                    Arg::V(e_ptr),
                    Arg::V(w850),
                ],
                w,
                cx,
            );
            env.callwc(
                0x021C_07AC,
                &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(kind), Arg::V(w850)],
                w,
                cx,
            );
            done_flag = 1;
            env.callwc(
                0x021C_0450,
                &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(0x1A)],
                w,
                cx,
            );
            break 'body;
        }
        let replaced = local_1a != 0;
        if replaced {
            env.callwc(0x021C_5604, &mut [Arg::P, Arg::V(mon), Arg::V(mv)], w, cx);
            put_cmd_30(env, w, cx, mon, mv, slot);
            env.callwc(
                0x021E_02DC,
                &mut [Arg::V(mon), Arg::V(local_1a), Arg::V(x)],
                w,
                cx,
            );
            r7 = local_1a;
            slot = local_18;
        }
        env.callwc(
            0x021C_64B4,
            &mut [Arg::V(cx.w_addr), Arg::V(mv), Arg::V(mon), Arg::V(e_ptr)],
            w,
            cx,
        );
        if replaced {
            env.callwc(
                0x021C_64B4,
                &mut [Arg::V(cx.w_addr), Arg::V(r7), Arg::V(mon), Arg::V(d_ptr)],
                w,
                cx,
            );
            rec_pull(w, cx.w_addr, d_ptr, m.d);
            let v = u32::from_le_bytes(m.d[0x10..0x14].try_into().unwrap());
            m.d[0x10..0x14].copy_from_slice(&(v | 4).to_le_bytes());
            m.d[2..4].copy_from_slice(&(mv as u16).to_le_bytes());
            rec_push(w, cx.w_addr, d_ptr, m.d);
            if side_effect(&mut Lend::wc(env, w, cx), mon, m.d) != 0 {
                env.callwc(
                    0x021C_07AC,
                    &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(kind), Arg::V(w850)],
                    w,
                    cx,
                );
                done_flag = 1;
                break 'body;
            }
        } else {
            rec_pull(w, cx.w_addr, e_ptr, m.e);
            rec_pull(w, cx.w_addr, d_ptr, m.d);
            m.d[..20].copy_from_slice(&m.e[..20]);
            rec_push(w, cx.w_addr, d_ptr, m.d);
        }
        env.callwc(0x021B_E338, &mut [Arg::V(w850)], w, cx);
        env.callwc(0x021B_E348, &mut [Arg::V(w854)], w, cx);
        env.callwc(
            0x021C_A7F4,
            &mut [
                Arg::P,
                Arg::V(mon),
                Arg::V(slot),
                Arg::V(d_ptr),
                Arg::V(w850),
            ],
            w,
            cx,
        );
        let b = env.callwc(0x021B_E358, &mut [Arg::V(w850)], w, cx) & 0xFF;
        env.callwc(0x021B_E368, &mut [Arg::V(w850)], w, cx);
        env.callwc(0x021B_E378, &mut [Arg::V(w850), Arg::V(w854)], w, cx);
        env.callwc(0x021B_E388, &mut [Arg::V(w854), Arg::V(b)], w, cx);
        if f19 == 0 && f1a == 0 && kind != 4 {
            env.callwc(
                0x021C_07AC,
                &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(kind), Arg::V(w850)],
                w,
                cx,
            );
            done_flag = 1;
        }
        rec_pull(w, cx.w_addr, e_ptr, m.e);
        queue_wrap_be6a8(&mut Lend::c(env, cx), w, mon, m.e, 0x22);
        rec_pull(w, cx.w_addr, d_ptr, m.d);
        build_be620(&mut Lend::wc(env, w, cx), mon, m.d);
        rec_push(w, cx.w_addr, d_ptr, m.d);
        let hb = cx.w_addr.wrapping_add(OFS_HELPER_1AE4);
        if env.callwc(
            0x021C_6458,
            &mut [
                Arg::V(cx.w_addr),
                Arg::V(mon),
                Arg::V(mv),
                Arg::V(r7),
                Arg::V(hb),
            ],
            w,
            cx,
        ) != 0
        {
            env.callwc(0x021C_99A0, &mut [Arg::P, Arg::V(hb)], w, cx);
            env.callwc(0x021C_8810, &mut [Arg::V(hb)], w, cx);
        } else {
            env.callwc(0x021C_5604, &mut [Arg::P, Arg::V(mon), Arg::V(r7)], w, cx);
        }
        w.set_u16_at(OFS_WORD_1F74, r7 as u16);
        if env.callwc(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x22)], w, cx) != 0 {
            env.callwc(
                0x021C_5620,
                &mut [Arg::P, Arg::V(mon), Arg::V(0x22), Arg::V(0)],
                w,
                cx,
            );
            env.callwc(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(0xE)], w, cx);
        }
        if env.callwc(0x021C_024C, &mut [Arg::P, Arg::V(mon), Arg::V(r7)], w, cx) != 0 {
            break 'body;
        }
        if flag_mon_effect(env, w, cx, mon, r7, slot, &mut res) != 0 {
            break 'body;
        }
        if env.callwc(
            0x021B_E6EC,
            &mut [Arg::P, Arg::V(mon), Arg::V(r7), Arg::V(slot)],
            w,
            cx,
        ) != 0
        {
            break 'body;
        }
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        env.callwc(
            0x021B_E398,
            &mut [
                Arg::V(cx.w_addr.wrapping_add(OFS_HELPER_1C)),
                Arg::V(r7),
                Arg::V(w.u32_at(OFS_WORD_10)),
                Arg::V(id),
            ],
            w,
            cx,
        );
        rec_pull(w, cx.w_addr, d_ptr, m.d);
        queue_wrap_be6a8(&mut Lend::c(env, cx), w, mon, m.d, 0x23);
        if env.callwc(
            0x021B_EAB4,
            &mut [
                Arg::P,
                Arg::V(mon),
                Arg::V(r7),
                Arg::V(w854),
                Arg::V(w.u32_at(OFS_PTR_4AC)),
            ],
            w,
            cx,
        ) != 0
        {
            env.callwc(0x021B_E550, &mut [Arg::P, Arg::V(mon), Arg::V(r7)], w, cx);
        } else {
            res = env.callwc(
                0x021B_EC58,
                &mut [Arg::P, Arg::V(mon), Arg::V(r7), Arg::V(w854)],
                w,
                cx,
            ) as u8;
        }
    }
    env.callwc(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(3)], w, cx);
    if done_flag != 0 || f19 != 0 || r7 == 0xA5 {
        rec_pull(w, cx.w_addr, w.u32_at(OFS_PTR_1AB0), m.d);
        let d6 = m.d[6] as u32;
        env.callwc(
            0x021D_6744,
            &mut [
                Arg::V(mon),
                Arg::V(slot),
                Arg::V(res as u32),
                Arg::V(d6),
                Arg::V(r7),
                Arg::V(mv),
            ],
            w,
            cx,
        );
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        cmd_queue::put_cmd(
            env,
            cx.q,
            0x18,
            &[id, slot, res as u32, d6, r7, mv],
            cx.scratch,
        );
    } else if env.callwc(0x021D_69AC, &mut [Arg::V(mon), Arg::V(3)], w, cx) != 0 {
        env.callwc(
            0x021C_0780,
            &mut [Arg::P, Arg::V(mon), Arg::V(3), Arg::V(0)],
            w,
            cx,
        );
    }
    if kind_before != 0x10 && env.callwc(0x021D_5C08, &mut [Arg::V(mon)], w, cx) != 0x10 {
        env.callwc(0x021B_FE60, &mut [Arg::P, Arg::V(mon)], w, cx);
    }
    if env.callwc(0x021D_5BB0, &mut [Arg::V(mon), Arg::V(0xB)], w, cx) != 0 {
        let id = mon_id(&mut Lend::wc(env, w, cx), mon);
        cmd_queue::put_cmd(&mut Lend::new(env, w), cx.q, 0x31, &[id, 0], cx.scratch);
        env.callwc(0x021B_FF60, &mut [Arg::P, Arg::V(mon), Arg::V(0)], w, cx);
    }
    queue_wrap_be9d0(&mut Lend::c(env, cx), w, mon, r7, res as u32);
    if local_1a != 0 {
        env.callwc(0x021E_0394, &mut [Arg::V(mon), Arg::V(local_1a)], w, cx);
    }
    env.callwc(0x021E_0394, &mut [Arg::V(mon), Arg::V(mv)], w, cx);
    env.callwc(0x021B_E400, &mut [Arg::P, Arg::V(r7)], w, cx);
}

#[cfg(test)]
pub(crate) mod tests;
