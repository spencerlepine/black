//! Flow-owned event handlers and helpers after the HANDEX procs (0x021CA8CC-0x021CC8D8; pool of 23 `(event, fn)` pairs at 0x021EFDE4; B10d).

use super::cmd_queue::{
    advance_fields, put_cmd, put_cmd_at, put_cmd_list, put_cmd_placeholder, set_word,
};
use super::env::{Arg, FlowEnv};
use super::SvFlowWork;
use crate::battle_mon::{value, BattleMon, BppEnv};
use crate::sick_cont::SickCont;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_B274,
    0x021C_C768,
    0x021C_C8A0,
    0x021C_C8B0,
    0x021C_C8B8,
    0x021C_C8C0,
    0x021C_C8C8,
    0x021C_B174,
    0x021C_B1A4,
    0x021C_B214,
    0x021C_B1D4,
    0x021C_BFA0,
    0x021C_C008,
    0x021C_C0F8,
    0x021C_C178,
    0x021C_C214,
    0x021C_C2C0,
    0x021C_C350,
    0x021C_C398,
    0x021C_C438,
    0x021C_C4AC,
    0x021C_C508,
    0x021C_C558,
    0x021C_C5F4,
    0x021C_BF30,
    0x021C_BF40,
    0x021C_BF50,
    0x021C_BF60,
    0x021C_BF70,
    0x021C_BF80,
    0x021C_BF90,
    0x021C_C098,
    0x021C_C0A8,
    0x021C_C0B8,
    0x021C_C0C8,
    0x021C_C0D8,
    0x021C_C0E8,
    0x021C_C668,
    0x021C_BCF4,
    0x021C_BEA8,
    0x021C_BD48,
    0x021C_B55C,
    0x021C_B5F0,
    0x021C_B2D4,
    0x021C_BB34,
    0x021C_BA40,
    0x021C_A8F8,
    0x021C_AA24,
    0x021C_AA58,
    0x021C_ACF4,
    0x021C_AFD8,
    0x021C_B730,
    0x021C_C800,
];

/// C: `FUN_overlay_d_93__021cb274`: asks the event system what happens to `value` for the
/// pokemon `mon` and the event record `rec` (`u16` at +0, `u8` at +6): pushes an event-variable
/// frame (0x290), sets variables 3 (poke id), 0x16, 0x12 and 4 (`value`), dispatches event 0x2A
/// on `work`, reads variable 4 back (low byte), pops the frame and returns 0x1F when the value
/// did not change. The event-variable stack is reached through [`FlowEnv`].
pub fn dispatch_value<E: FlowEnv>(
    env: &mut E,
    work: u32,
    mon: &BattleMon,
    rec: &[u8],
    value: u32,
) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x290)]);
    env.call(0x021D_75EC, &mut [Arg::V(3), Arg::V(mon.poke_id() as u32)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x16), Arg::V(rec[6] as u32)]);
    let word = u16::from_le_bytes([rec[0], rec[1]]) as u32;
    env.call(0x021D_75EC, &mut [Arg::V(0x12), Arg::V(word)]);
    env.call(0x021D_7630, &mut [Arg::V(4), Arg::V(value)]);
    env.call(0x021D_70F0, &mut [Arg::V(work), Arg::V(0x2A)]);
    let result = env.call(0x021D_775C, &mut [Arg::V(4)]) & 0xFF;
    env.call(0x021D_7540, &mut [Arg::V(0x297)]);
    if result == value {
        0x1F
    } else {
        result
    }
}

/// C: `FUN_overlay_d_93__021cc768`: which side is ahead? `a` and `b` are `{ x, y }` byte pairs,
/// `w` the flow work. Compares `b - a` for x and y; when equal the two
/// totals of `021cc800` ([`side_totals`]; sides 0 and 1; each yields a signed pair) decide.
/// Returns 1 when the outcome equals `side`, 0 when not, 2 when everything ties. `mons`
/// resolves the BattleMon addresses `021b9c00` returns.
pub fn compare_sides<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    mons: &dyn Fn(u32) -> BattleMon,
    a: &[u8],
    b: &[u8],
    side: u32,
) -> u32 {
    let dx = b[0] as i32 - a[0] as i32;
    let dy = b[1] as i32 - a[1] as i32;
    let outcome = |flag: bool| u32::from(flag as u32 == side);
    if dx != dy {
        return outcome(dx >= dy);
    }
    let (mut hp0, mut pct0) = ([0u8; 4], [0u8; 4]);
    let (mut hp1, mut pct1) = ([0u8; 4], [0u8; 4]);
    side_totals(env, w, mons, &mut hp0, &mut pct0, 0);
    side_totals(env, w, mons, &mut hp1, &mut pct1, 1);
    let (pct0, pct1) = (i32::from_le_bytes(pct0), i32::from_le_bytes(pct1));
    if pct0 != pct1 {
        return outcome(pct0 <= pct1);
    }
    let (hp0, hp1) = (i32::from_le_bytes(hp0), i32::from_le_bytes(hp1));
    if hp0 == hp1 {
        return 2;
    }
    outcome(hp0 <= hp1)
}

// ---- plumbing shared by the HANDEX record makers ---------------------------------------------------

/// Work offset of the command-queue header word that brackets a record maker (`P + 0x1D74`).
const OFS_QHEAD: usize = 0x1D74;
/// Work offset of the veneer-owned table `021cc418`/`021cc49c` read (`P + 0x1AB8`).
const OFS_TABLE: u32 = 0x1AB8;

fn rd32(b: &[u8]) -> u32 {
    u32::from_le_bytes(b[..4].try_into().unwrap())
}

fn wr32(b: &mut [u8], v: u32) {
    b[..4].copy_from_slice(&v.to_le_bytes());
}

/// `FUN_overlay_d_93__021c88a4`: allocates a HANDEX record of kind `kind` for pokemon `id`; the
/// result is its offset in the work image (the test environment returns an address inside it).
fn rec_new<E: FlowEnv>(env: &mut E, waddr: u32, kind: u32, id: u32) -> usize {
    env.call(0x021C_88A4, &mut [Arg::V(waddr), Arg::V(kind), Arg::V(id)])
        .wrapping_sub(waddr) as usize
}

/// `FUN_overlay_d_93__021c88c8`: commits a record made by [`rec_new`].
fn rec_commit<E: FlowEnv>(env: &mut E, waddr: u32, rec: usize) {
    env.call(
        0x021C_88C8,
        &mut [Arg::V(waddr), Arg::V(waddr.wrapping_add(rec as u32))],
    );
}

/// `FUN_overlay_d_93__021c8838` on the str record at `w[p..]` (ported in `hem`:
/// [`super::hem::str_init`]).
fn str_init(w: &mut [u8], p: usize, kind: u32, msg: u32) {
    super::hem::str_init(&mut w[p..], kind, msg);
}

/// `FUN_overlay_d_93__021c8860` on the str record at `w[p..]` (ported in `hem`:
/// [`super::hem::str_push`]).
fn str_push(w: &mut [u8], p: usize, v: u32) {
    super::hem::str_push(&mut w[p..], v);
}

/// `FUN_overlay_d_93__021d7b74`: max HP / `div`, at least 1.
fn max_hp_div_min1<E: BppEnv>(env: &mut E, mon: &BattleMon, div: u32) -> u32 {
    match mon.get(value::MAX_HP, env) / div {
        0 => 1,
        v => v,
    }
}

/// `FUN_overlay_d_93__021d805c(item, param)`: item data parameter (an unported ARM9 routine).
fn item_param<E: FlowEnv>(env: &mut E, item: u32, param: u32) -> u32 {
    env.call(0x021D_805C, &mut [Arg::V(item), Arg::V(param)])
}

/// Item parameter 0x3A (HP to restore): 0xFD = max HP / 4, 0xFE = max HP / 2, 0xFF = max HP,
/// else the value; `reread` = the revive path reads the parameter a second time for that case.
fn heal_amount<E: FlowEnv + BppEnv>(env: &mut E, mon: &BattleMon, item: u32, reread: bool) -> u32 {
    match item_param(env, item, 0x3A) {
        0xFD => max_hp_div_min1(env, mon, 4),
        0xFE => max_hp_div_min1(env, mon, 2),
        0xFF => mon.get(value::MAX_HP, env),
        _ if reread => item_param(env, item, 0x3A),
        v => v,
    }
}

// ---- tiny word-table helpers (021cc8a0..021cc8c8) -------------------------------------------------

/// C: `FUN_overlay_d_93__021cc8a0`: sets the 24 words of `p` to 6.
pub fn table_fill_6(p: &mut [u8]) {
    for i in 0..0x18 {
        wr32(&mut p[4 * i..], 6);
    }
}

/// C: `FUN_overlay_d_93__021cc8b0`: `p[i] = v` (word table).
pub fn table_set(p: &mut [u8], i: u32, v: u32) {
    wr32(&mut p[4 * i as usize..], v);
}

/// C: `FUN_overlay_d_93__021cc8b8` and `FUN_overlay_d_93__021cc8c0`: `p[i]` (word table).
pub fn table_get(p: &[u8], i: u32) -> u32 {
    rd32(&p[4 * i as usize..])
}

/// C: `FUN_overlay_d_93__021cc8c8`: clears the count at +0 and the 500 bytes after it
/// (`MI_CpuFill8`, real code).
pub fn queue_clear(p: &mut [u8]) {
    wr32(p, 0);
    p[4..504].fill(0);
}

// ---- client slot lookup (021cb174 / 021cb1a4) -------------------------------------------------------

/// Shared body of `021cb174` and `021cb1a4`: `p` = `{ +0 server, +4 main }` (the flow work),
/// `p_addr` its address; `getter` is the main-module query that yields the pokemon id.
/// `021b8e28(main, id, &client, &index)` splits it, `021bb85c(server, client)` (ported in
/// `server_side`, so real code on both sides) returns the client slot whose word at +4 is the
/// party. `server` is the address of the server-side object; the environment keeps it inside `p`
/// (`p_addr <= server`), so its bytes are `p[server - p_addr..]`. A null slot (unused client or
/// `client >= 4`) makes the original read address 4, which has no model: it panics here.
fn slot_party<E: FlowEnv>(env: &mut E, getter: u32, p_addr: u32, p: &[u8]) -> u32 {
    let server = rd32(p);
    let main = rd32(&p[4..]);
    let id = env.call(getter, &mut [Arg::V(main)]);
    let mut a = [0u8; 1]; // client (r2)
    let mut b = [0u8; 1]; // index (r3)
    env.call(
        0x021B_8E28,
        &mut [Arg::V(main), Arg::V(id), Arg::Out(&mut a), Arg::Out(&mut b)],
    );
    let sbase = server.wrapping_sub(p_addr) as usize;
    let slot = super::server_side::slot_checked(&p[sbase..], a[0] as u32)
        .expect("021bb85c returned null: the original reads address 4");
    let party = rd32(&p[sbase + slot + 4..]);
    env.call(0x021B_9BF0, &mut [Arg::V(party), Arg::V(b[0] as u32)])
}

/// C: `FUN_overlay_d_93__021cb174`: the BattleMon of the pokemon `021b8c50(main)` names.
pub fn mon_of_main_a<E: FlowEnv>(env: &mut E, p_addr: u32, p: &[u8]) -> u32 {
    slot_party(env, 0x021B_8C50, p_addr, p)
}

/// C: `FUN_overlay_d_93__021cb1a4`: as [`mon_of_main_a`] for `021b8cb0(main)`.
pub fn mon_of_main_b<E: FlowEnv>(env: &mut E, p_addr: u32, p: &[u8]) -> u32 {
    slot_party(env, 0x021B_8CB0, p_addr, p)
}

// ---- event-variable stack users ----------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021cb214`: asks the event system whether `mon` is affected by the
/// record `rec` (`u16` at +0, `u8` at +6): pushes a frame (0x277), sets variables 3 (poke id),
/// 0x16, 0x12, 4 (0x1F) and 0x50 (0), dispatches event 0x29 on `work` and returns the low byte
/// of variable 4 after popping the frame.
pub fn dispatch_check<E: FlowEnv>(env: &mut E, work: u32, mon: &BattleMon, rec: &[u8]) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x277)]);
    env.call(0x021D_75EC, &mut [Arg::V(3), Arg::V(mon.poke_id() as u32)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x16), Arg::V(rec[6] as u32)]);
    let word = u16::from_le_bytes([rec[0], rec[1]]) as u32;
    env.call(0x021D_75EC, &mut [Arg::V(0x12), Arg::V(word)]);
    env.call(0x021D_7630, &mut [Arg::V(4), Arg::V(0x1F)]);
    env.call(0x021D_7630, &mut [Arg::V(0x50), Arg::V(0)]);
    env.call(0x021D_70F0, &mut [Arg::V(work), Arg::V(0x29)]);
    let result = env.call(0x021D_775C, &mut [Arg::V(4)]) & 0xFF;
    env.call(0x021D_7540, &mut [Arg::V(0x27F)]);
    result
}

/// C: `FUN_overlay_d_93__021cb1d4`: [`dispatch_value`] bracketed by `advance_fields` /
/// `set_word` on the queue header word at `P + 0x1D74`.
pub fn dispatch_value_bracketed<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    rec: &[u8],
    value: u32,
) -> u32 {
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    let r = dispatch_value(env, waddr, mon, rec, value);
    set_word(&mut w.mem[OFS_QHEAD..], head);
    r
}

// ---- HANDEX record makers ------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021cbfa0`: when `021cbff8(side, 3, turn(5))` says so, queues record
/// kind 4 (message 0x88 with the side of `mon`). Returns whether it did.
pub fn msg_side<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32, mon: &BattleMon) -> u32 {
    let side = u32::from(mon.poke_id() as u32 >= 12);
    let cont = SickCont::turn(5).0;
    let hit = env.call(0x021C_BFF8, &mut [Arg::V(side), Arg::V(3), Arg::V(cont)]);
    if hit == 0 {
        return 0;
    }
    let rec = rec_new(env, waddr, 4, 0x1F);
    str_init(&mut w.mem[..], rec + 4, 1, 0x88);
    str_push(&mut w.mem[..], rec + 4, side);
    rec_commit(env, waddr, rec);
    1
}

/// C: `FUN_overlay_d_93__021cc008`: revives a fainted `mon`: record kind 0x2C (+4 id, +6 HP from
/// item parameter 0x3A) with message 3, then `021c84f4(P, id)`.
pub fn revive<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    item: u32,
) -> u32 {
    if !mon.is_fainted(env) {
        return 0;
    }
    let id = mon.poke_id() as u32;
    let rec = rec_new(env, waddr, 0x2C, id);
    w.mem[rec + 4] = id as u8;
    let hp = heal_amount(env, mon, item, true);
    w.mem[rec + 6..rec + 8].copy_from_slice(&(hp as u16).to_le_bytes());
    str_init(&mut w.mem[..], rec + 8, 2, 3);
    str_push(&mut w.mem[..], rec + 8, id);
    rec_commit(env, waddr, rec);
    env.call(0x021C_84F4, &mut [Arg::V(waddr), Arg::V(id)]);
    1
}

/// C: `FUN_overlay_d_93__021cc0f8`: sets flag-B bit 9 (queue command 0x19) and adds
/// `amount - 1` crit-boost stages (command 0xE) through the command queue `q`; when either
/// happened, puts the list command 0x5B. Returns whether it did.
pub fn raise_crit<E: FlowEnv>(
    env: &mut E,
    q: &mut [u8],
    scratch: &mut [u8],
    mon: &mut BattleMon,
    amount: u32,
) -> u32 {
    let mut done = false;
    if !mon.flag_b(9) {
        mon.set_flag_b(9);
        put_cmd(env, q, 0x19, &[mon.poke_id() as u32, 9], scratch);
        done = true;
    }
    if (amount as i32) > 1 && mon.add_crit_boost(amount as i32 - 1) {
        put_cmd(
            env,
            q,
            0xE,
            &[mon.poke_id() as u32, amount.wrapping_sub(1)],
            scratch,
        );
        done = true;
    }
    if !done {
        return 0;
    }
    put_cmd_list(env, q, 0x5B, 0x411, &[mon.poke_id() as u32, 0xFFFF_0000]);
    1
}

/// C: `FUN_overlay_d_93__021cc178`: restores PP of move `slot` (original move set) by the amount
/// that is missing, at most item parameter 0x3B (0x7F = no limit): record kind 9 with
/// message 0x186 (poke id, move id). Returns whether anything was restored.
pub fn pp_restore_one<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    item: u32,
    slot: u32,
) -> u32 {
    if mon.orig_move_count() as u32 <= slot {
        return 0;
    }
    let id = mon.poke_id() as u32;
    let mut n = mon.orig_move_pp_missing(slot) as u32;
    let lim = item_param(env, item, 0x3B) & 0xFF;
    if lim != 0x7F && n > lim {
        n = lim;
    }
    if n == 0 {
        return 0;
    }
    let rec = rec_new(env, waddr, 9, id);
    w.mem[rec + 4] = n as u8;
    w.mem[rec + 5] = id as u8;
    w.mem[rec + 6] = slot as u8;
    w.mem[rec + 7] |= 2;
    str_init(&mut w.mem[..], rec + 8, 2, 0x186);
    str_push(&mut w.mem[..], rec + 8, id);
    str_push(&mut w.mem[..], rec + 8, mon.orig_move_id(slot) as u32);
    rec_commit(env, waddr, rec);
    1
}

/// C: `FUN_overlay_d_93__021cc214`: restores the PP of every original move (one record kind 9
/// each, at most item parameter 0x3B), then queues message 0x189 once. Returns whether
/// anything was restored.
pub fn pp_restore_all<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    item: u32,
) -> u32 {
    let id = mon.poke_id() as u32;
    let lim = item_param(env, item, 0x3B) & 0xFF;
    let mut any = 0;
    for i in 0..mon.orig_move_count() as u32 {
        let mut n = mon.orig_move_pp_missing(i) as u32;
        if n > lim {
            n = lim;
        }
        if n != 0 {
            let rec = rec_new(env, waddr, 9, id);
            w.mem[rec + 6] = i as u8;
            w.mem[rec + 5] = id as u8;
            w.mem[rec + 4] = n as u8;
            w.mem[rec + 7] |= 2;
            rec_commit(env, waddr, rec);
            any = 1;
        }
    }
    if any != 0 {
        let rec = rec_new(env, waddr, 4, id);
        str_init(&mut w.mem[..], rec + 4, 2, 0x189);
        str_push(&mut w.mem[..], rec + 4, id);
        rec_commit(env, waddr, rec);
    }
    any
}

/// C: `FUN_overlay_d_93__021cc2c0`: heals `mon` (not at full HP, not fainted) by the amount of
/// item parameter 0x3A: record kind 5 with message 0x183. Returns whether it did.
pub fn heal<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    item: u32,
) -> u32 {
    if mon.is_full_hp(env) || mon.is_fainted(env) {
        return 0;
    }
    let id = mon.poke_id() as u32;
    let rec = rec_new(env, waddr, 5, id);
    w.mem[rec + 6] = id as u8;
    w.mem[rec + 7] = 1;
    let hp = heal_amount(env, mon, item, false);
    w.mem[rec + 4..rec + 6].copy_from_slice(&(hp as u16).to_le_bytes());
    str_init(&mut w.mem[..], rec + 8, 2, 0x183);
    str_push(&mut w.mem[..], rec + 8, id);
    rec_commit(env, waddr, rec);
    1
}

/// Record kind 0xB: cure condition `sick` of pokemon `id`.
fn cure_record<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32, id: u32, sick: u32) {
    let rec = rec_new(env, waddr, 0xB, id);
    w.mem[rec + 0x14] = 1;
    w.mem[rec + 8] = id as u8;
    wr32(&mut w.mem[rec + 4..], sick);
    rec_commit(env, waddr, rec);
}

/// C: `FUN_overlay_d_93__021cc350`: cures condition `sick` of a living `mon` that has it.
pub fn cure_status<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    sick: u32,
) -> u32 {
    if mon.is_fainted(env) || !mon.has_sick(sick) {
        return 0;
    }
    cure_record(env, w, waddr, mon.poke_id() as u32, sick);
    1
}

/// C: `FUN_overlay_d_93__021cc398`: cures every condition `021cc428(0..)` lists (until it
/// answers 0) that a living `mon` has, unless `021cc418(P + 0x1AB8, id)` forbids it. Returns
/// whether anything was cured.
pub fn cure_all_statuses<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
) -> u32 {
    if mon.is_fainted(env) {
        return 0;
    }
    let id = mon.poke_id() as u32;
    if env.call(0x021C_C418, &mut [Arg::V(waddr + OFS_TABLE), Arg::V(id)]) != 0 {
        return 0;
    }
    let mut any = 0;
    let mut i = 0;
    loop {
        let sick = env.call(0x021C_C428, &mut [Arg::V(i)]);
        i += 1;
        if sick == 0 {
            break;
        }
        if !mon.has_sick(sick) {
            continue;
        }
        cure_record(env, w, waddr, mon.poke_id() as u32, sick);
        any = 1;
    }
    any
}

/// C: `FUN_overlay_d_93__021cc438`: changes stage `stat` of a living `mon` by `amount` (record
/// kind 0xE), unless `021cc49c(P + 0x1AB8, id)` answers 6 or the stage cannot move that way.
pub fn change_stage<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    amount: u32,
    stat: u32,
) -> u32 {
    let id = mon.poke_id() as u32;
    if env.call(0x021C_C49C, &mut [Arg::V(waddr + OFS_TABLE), Arg::V(id)]) == 6 {
        return 0;
    }
    if mon.is_fainted(env) || !mon.can_change_stage(stat, amount as i32) {
        return 0;
    }
    let rec = rec_new(env, waddr, 0xE, id);
    w.mem[rec + 0xF] = 1;
    w.mem[rec + 0x10] = id as u8;
    wr32(&mut w.mem[rec + 4..], stat);
    w.mem[rec + 0xC] = amount as u8;
    w.mem[rec + 0xE] = 1;
    rec_commit(env, waddr, rec);
    1
}

/// C: `FUN_overlay_d_93__021cc4ac`: 0 when `021cc4f8(held item)` objects, 1 when
/// `021c84ac(P, mon, &flag)` says so, else 2 if it set the flag.
pub fn held_item_check<E: FlowEnv>(env: &mut E, waddr: u32, mon: &BattleMon, maddr: u32) -> u32 {
    if env.call(0x021C_C4F8, &mut [Arg::V(mon.held_item() as u32)]) != 0 {
        return 0;
    }
    let mut flag = [0u8; 1];
    let r = env.call(
        0x021C_84AC,
        &mut [Arg::V(waddr), Arg::V(maddr), Arg::Out(&mut flag)],
    );
    if r & 0xFF != 0 {
        1
    } else if flag[0] != 0 {
        2
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021cc508`: unless the ability (raw value 0x10) is 150, runs
/// `021bdbb8(P, mon)` bracketed by the queue header word; returns whether `021c88e4(P)` is 2.
pub fn ability_trigger<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    maddr: u32,
) -> u32 {
    if mon.get(0x10, env) == 150 {
        return 0;
    }
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    env.call(0x021B_DBB8, &mut [Arg::V(waddr), Arg::V(maddr)]);
    let r = env.call(0x021C_88E4, &mut [Arg::V(waddr)]);
    set_word(&mut w.mem[OFS_QHEAD..], head);
    u32::from(r == 2)
}

/// C: `FUN_overlay_d_93__021cc558`: unless `021e869c(P, id)` objects, announces the held item
/// of `mon` (record kind 0x20 with message 0xE1: poke id, item) when it has one.
pub fn show_held_item<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
) -> u32 {
    let args = &mut [Arg::V(waddr), Arg::V(mon.poke_id() as u32)];
    if env.call(0x021E_869C, args) != 0 {
        return 0;
    }
    let item = mon.held_item() as u32;
    if item == 0 {
        return 0;
    }
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    let id = mon.poke_id() as u32;
    let rec = rec_new(env, waddr, 0x20, id);
    w.mem[rec + 6] = id as u8;
    w.mem[rec + 4..rec + 6].copy_from_slice(&0u16.to_le_bytes());
    str_init(&mut w.mem[..], rec + 0xC, 2, 0xE1);
    str_push(&mut w.mem[..], rec + 0xC, id);
    str_push(&mut w.mem[..], rec + 0xC, item);
    rec_commit(env, waddr, rec);
    set_word(&mut w.mem[OFS_QHEAD..], head);
    1
}

/// C: `FUN_overlay_d_93__021cc5f4`: record kind 0x10 (+4 = 1, +5 id) followed by record kind 4
/// with message 0xE4, bracketed by the queue header word. Always returns 1.
pub fn flinch_message<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
) -> u32 {
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    let id = mon.poke_id() as u32;
    let rec = rec_new(env, waddr, 0x10, id);
    w.mem[rec + 4] = 1;
    w.mem[rec + 5] = id as u8;
    rec_commit(env, waddr, rec);
    let rec = rec_new(env, waddr, 4, id);
    str_init(&mut w.mem[..], rec + 4, 2, 0xE4);
    str_push(&mut w.mem[..], rec + 4, id);
    rec_commit(env, waddr, rec);
    set_word(&mut w.mem[OFS_QHEAD..], head);
    1
}

// ---- item-effect table entries (pool 0x021EFDE4): a fixed argument, then the shared body ----------

macro_rules! cure_entry {
    ($name:ident, $addr:literal, $sick:literal) => {
        #[doc = concat!("C: `FUN_overlay_d_93__", $addr, "`: `cure_status(.., ", stringify!($sick), ")`.")]
        pub fn $name<E: FlowEnv + BppEnv>(
            env: &mut E,
            w: &mut SvFlowWork,
            waddr: u32,
            mon: &BattleMon,
        ) -> u32 {
            cure_status(env, w, waddr, mon, $sick)
        }
    };
}
macro_rules! stage_entry {
    ($name:ident, $addr:literal, $stat:literal) => {
        #[doc = concat!("C: `FUN_overlay_d_93__", $addr, "`: `change_stage(.., amount, ", stringify!($stat), ")`.")]
        pub fn $name<E: FlowEnv + BppEnv>(
            env: &mut E,
            w: &mut SvFlowWork,
            waddr: u32,
            mon: &BattleMon,
            amount: u32,
        ) -> u32 {
            change_stage(env, w, waddr, mon, amount, $stat)
        }
    };
}
cure_entry!(cure_entry_2, "021cbf30", 2);
cure_entry!(cure_entry_5, "021cbf40", 5);
cure_entry!(cure_entry_4, "021cbf50", 4);
cure_entry!(cure_entry_3, "021cbf60", 3);
cure_entry!(cure_entry_1, "021cbf70", 1);
cure_entry!(cure_entry_6, "021cbf80", 6);
stage_entry!(stage_entry_1, "021cc098", 1);
stage_entry!(stage_entry_2, "021cc0a8", 2);
stage_entry!(stage_entry_3, "021cc0b8", 3);
stage_entry!(stage_entry_4, "021cc0c8", 4);
stage_entry!(stage_entry_5, "021cc0d8", 5);
stage_entry!(stage_entry_6, "021cc0e8", 6);

/// C: `FUN_overlay_d_93__021cbf90`: [`cure_all_statuses`] (the stack constant 7 is dead).
pub fn cure_all_entry<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
) -> u32 {
    cure_all_statuses(env, w, waddr, mon)
}

// ---- who is ahead (021cc668 / 021cc800) --------------------------------------------------------------

/// C: `FUN_overlay_d_93__021cc668`: compares the two sides of the battle. Per client the party
/// size (`021b9b94`) and the number of alive members (`021b9b98`) are added up for the client's
/// side. Returns 0 or 1 for the side that has alive members while the other has none, 2 when
/// neither does or `021ba534` says so; otherwise the tie is settled by `021cc768` (both sides
/// have members) or by the veneer `021cc758` (no side `021b8ff4` names has any).
pub fn who_leads<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    waddr: u32,
    mons: &dyn Fn(u32) -> BattleMon,
) -> u32 {
    let main = w.u32_at(4);
    let pokecon = w.u32_at(8);
    // stack bytes: [0..2) party size per side, [2..4) alive count per side
    let mut st = [0u8; 4];
    for i in 0..4u32 {
        let used = env.call(0x021B_8B9C, &mut [Arg::V(main), Arg::V(i & 0xFF)]);
        if used == 0 {
            continue;
        }
        let list = env.call(0x021B_9AEC, &mut [Arg::V(pokecon), Arg::V(i)]);
        if env.call(0x021B_9B94, &mut [Arg::V(list)]) == 0 {
            continue;
        }
        let side = env.call(0x021B_8BC4, &mut [Arg::V(main), Arg::V(i & 0xFF)]) as usize;
        let alive = env.call(0x021B_9B98, &mut [Arg::V(list)]);
        st[2 + side] = st[2 + side].wrapping_add(alive as u8);
        let size = env.call(0x021B_9B94, &mut [Arg::V(list)]);
        st[side] = st[side].wrapping_add(size as u8);
    }
    let client = env.call(0x021B_8FF4, &mut [Arg::V(main)]);
    let me = env.call(0x021B_8BC4, &mut [Arg::V(main), Arg::V(client)]);
    let other = ((1 ^ me) & 0xFF) as usize;
    let mine = st[2 + me as usize];
    let theirs = st[2 + other];
    if mine == 0 && theirs != 0 {
        return 0;
    }
    if theirs == 0 && mine != 0 {
        return 1;
    }
    if env.call(0x021B_A534, &mut [Arg::V(main)]) == 0 {
        return 2;
    }
    if st[2] == 0 {
        let r = env.call(0x021C_C758, &mut [Arg::V(waddr + 0x3E0)]);
        if r == 0x1F {
            return 2;
        }
        return u32::from(u32::from(r >= 12) == me);
    }
    compare_sides(env, w, mons, &st[2..4], &st[0..2], me)
}

/// C: `FUN_overlay_d_93__021cc800`: for the clients of side `side`: `hp` = sum of the current
/// HP of every party member, `pct` = `100 * hp / (sum of max HP)`. `mons` resolves the address
/// `021b9c00` returns to the BattleMon image behind it.
pub fn side_totals<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    mons: &dyn Fn(u32) -> BattleMon,
    hp: &mut [u8],
    pct: &mut [u8],
    side: u32,
) {
    let main = w.u32_at(4);
    let pokecon = w.u32_at(8);
    let mut total_hp = 0u32;
    let mut total_max = 0u32;
    for i in 0..4u32 {
        if env.call(0x021B_8B9C, &mut [Arg::V(main), Arg::V(i & 0xFF)]) == 0 {
            continue;
        }
        if side != env.call(0x021B_8BC4, &mut [Arg::V(main), Arg::V(i & 0xFF)]) {
            continue;
        }
        let list = env.call(0x021B_9AEC, &mut [Arg::V(pokecon), Arg::V(i)]);
        let n = env.call(0x021B_9B94, &mut [Arg::V(list)]);
        for k in 0..n {
            let at = env.call(0x021B_9C00, &mut [Arg::V(list), Arg::V(k & 0xFF)]);
            let m = mons(at);
            total_max = total_max.wrapping_add(m.get(value::MAX_HP, env));
            total_hp = total_hp.wrapping_add(m.get(value::HP, env));
        }
    }
    wr32(hp, total_hp);
    wr32(
        pct,
        crate::calc::u32_div(total_hp.wrapping_mul(100), total_max),
    );
}

// ---- capture arithmetic ---------------------------------------------------------------------------------

/// Ball-independent bonus steps by `021b873c(main)` (a count of 30/150/300/450/600 thresholds):
/// the first element is the value above 600, then each lower band.
fn band(v: u32) -> usize {
    if v > 600 {
        0
    } else if v > 450 {
        1
    } else if v > 300 {
        2
    } else if v > 150 {
        3
    } else if v > 30 {
        4
    } else {
        5
    }
}

/// C: `FUN_overlay_d_93__021cbcf4`: 4.12 fixed-point bonus by the main module's value
/// `021b873c(main)`: 1.0 above 600, then 0.9, 0.8, 0.7, 0.5 and 0.3.
pub fn capture_bonus<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    let v = env.call(0x021B_873C, &mut [Arg::V(w.u32_at(4))]);
    [0x1000, 0xE66, 0xCCD, 0xB33, 0x800, 0x4CD][band(v)]
}

/// C: `FUN_overlay_d_93__021cbea8`: the shake roll: scales `x` (at most 0xFF000, a 20.12 value)
/// by a factor chosen from `021b873c(main)` (bands as in [`capture_bonus`]: 10, 2, 1.5, 1, 0.5 and
/// none, in 4.12), divides by 6 and compares `rand(0x100)` with the integer part. Returns 1
/// when the roll is below it.
pub fn capture_shake<E: FlowEnv>(env: &mut E, w: &SvFlowWork, x: i32) -> u32 {
    let v = env.call(0x021B_873C, &mut [Arg::V(w.u32_at(4))]);
    let factor: i64 = match band(v) {
        0 => 10 << 10,
        1 => 2 << 12,
        2 => 6 << 10,
        3 => 1 << 12,
        4 => 2 << 10,
        _ => return 0,
    };
    let x = x.min(0xFF000) as i64;
    let scaled = (((x * factor) + 0x800) >> 12) as i32;
    let q = scaled.wrapping_div(6);
    let r = env.call(0x021D_78AC, &mut [Arg::V(0x100)]);
    u32::from(r < (q >> 12) as u32)
}

/// C: `FUN_overlay_d_93__021cbd48`: 4.12 catch-rate multiplier of ball `ball` (< 16) against
/// `mon` (`mon_addr` is its address). `blob` is the structure `021b8714(main)` returns (at
/// `blob_addr`): +0 a word, +4 a word, +9 and +0xC bytes.
pub fn ball_multiplier<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    mon: &BattleMon,
    mon_addr: u32,
    ball: u32,
    blob_addr: u32,
    blob: &[u8],
) -> u32 {
    use black_core::softfloat::{f2i, fadd, fsub, i2f};
    let main = w.u32_at(4);
    let blob_at = |env: &mut E| {
        env.call(0x021B_8714, &mut [Arg::V(main)])
            .wrapping_sub(blob_addr) as usize
    };
    if ball > 15 {
        return 0x1000;
    }
    match ball {
        2 => 0x2000,
        3 => 0x1800,
        6 => {
            if mon.has_type(10) || mon.has_type(6) {
                0x3000
            } else {
                0x1000
            }
        }
        7 => {
            if env.call(0x021B_A1A8, &mut [Arg::V(main), Arg::V(1)]) != 0 {
                return 0x3800;
            }
            let s = blob_at(env);
            if rd32(&blob[s + 4..]) == 6 {
                0x3800
            } else {
                0x1000
            }
        }
        8 => {
            let level = mon.get(value::LEVEL, env) & 0xFFFF;
            if level >= 30 {
                return 0x1000;
            }
            let mut v = (0x29 - level) & 0xFFFF;
            if v > 0x28 {
                v = 0x28;
            }
            let f = if v != 0 {
                fadd(0x3F00_0000, i2f((v << 12) as i32))
            } else {
                fsub(i2f(0), 0x3F00_0000)
            };
            f2i(f).wrapping_div(10) as u32
        }
        9 => {
            if env.call(0x021B_8724, &mut [Arg::V(main), Arg::V(mon_addr)]) != 0 {
                0x3000
            } else {
                0x1000
            }
        }
        10 => {
            let v = (w.u32_at(0x10).wrapping_mul(0x4CD).wrapping_add(0x1000)) as i32;
            if v <= 0x4000 {
                v as u32
            } else {
                0x4000
            }
        }
        13 => {
            let s = blob_at(env);
            let r = env.call(
                0x0201_13F0,
                &mut [Arg::V(blob[s + 9] as u32), Arg::V(blob[s + 0xC] as u32)],
            ) & 0xFF;
            let kind = rd32(&blob[s..]);
            if kind.wrapping_sub(4) <= 1 {
                return 0x3800;
            }
            if env.call(0x021C_BE98, &mut [Arg::V(kind & 0xFFFF)]) == 0 {
                return 0x1000;
            }
            if (r + 0xFD) & 0xFF <= 1 {
                0x3800
            } else {
                0x1000
            }
        }
        15 if w.u32_at(0x10) == 0 => 0x5000,
        _ => 0x1000,
    }
}

// ---- experience and effort values -------------------------------------------------------------------

/// `(personal-data parameter, party parameter, power item)` per stat: the EV yield read from
/// the fainted pokemon's personal data, the EV field of the winner, and the held item that
/// adds EVs (table at 0x021EFDC0).
const EV_TABLE: [(u8, u16, u16); 6] = [
    (0x0A, 0x0D, 0x126),
    (0x0B, 0x0E, 0x121),
    (0x0C, 0x0F, 0x122),
    (0x0D, 0x10, 0x125),
    (0x0E, 0x11, 0x123),
    (0x0F, 0x12, 0x124),
];

/// The integer square root helper `0207c7b4` of a value rounded through single precision:
/// `sqrt(f2i(x << 12 + 0.5))`.
fn sqrt_of<E: FlowEnv>(env: &mut E, x: u32) -> u32 {
    use black_core::softfloat::{f2i, fadd, fsub, u2f};
    let f = if x != 0 {
        fadd(0x3F00_0000, u2f(x << 12))
    } else {
        fsub(u2f(0), 0x3F00_0000)
    };
    env.call(0x0207_C7B4, &mut [Arg::V(f2i(f) as u32)])
}

/// C: `FUN_overlay_d_93__021cb55c`: scales `exp` (earned from a pokemon of level `lvl_b`) for
/// the receiver `mon` of level `lvl_a`: `exp * (2b+10)^2.5 / (a+b+10)^2.5 + 1`, at most the
/// EXP of level 100.
pub fn exp_scale<E: FlowEnv + BppEnv>(
    env: &mut E,
    mon: &BattleMon,
    exp: u32,
    lvl_a: u32,
    lvl_b: u32,
) -> u32 {
    let r4 = lvl_b.wrapping_mul(2).wrapping_add(10);
    let r5 = lvl_b.wrapping_add(lvl_a).wrapping_add(10);
    let s4 = sqrt_of(env, r4);
    let num = r4.wrapping_mul(r4).wrapping_mul(s4) >> 12;
    let s5 = sqrt_of(env, r5);
    let den = r5.wrapping_mul(r5).wrapping_mul(s5) >> 12;
    let q = crate::calc::u32_div(exp.wrapping_mul(num), den).wrapping_add(1);
    q.min(mon.exp_for_max_level(env))
}

/// C: `FUN_overlay_d_93__021cb5f0`: adds the effort values the `loser` yields (personal data,
/// doubled by the Macho Brace item 215 and by party flag 0x97, plus a power item's bonus) to
/// `winner`'s party pokemon (each EV capped at 255) and records every gain in
/// `out[5 + stat]`.
pub fn ev_gain<E: FlowEnv + BppEnv>(
    env: &mut E,
    winner: &BattleMon,
    loser: &BattleMon,
    out: &mut [u8],
) {
    let species = loser.species() as u32;
    let form = loser.get(0x13, env) & 0xFFFF;
    let mut gain = [0u8; 6];
    for (i, e) in EV_TABLE.iter().enumerate() {
        gain[i] = env.personal(species, form, e.0 as u32) as u8;
    }
    if winner.held_item() == 215 {
        gain.iter_mut().for_each(|g| *g = g.wrapping_shl(1));
    }
    for (i, e) in EV_TABLE.iter().enumerate() {
        let item = e.2 as u32;
        if item == winner.held_item() as u32 {
            gain[i] = gain[i].wrapping_add(item_param(env, item, 2) as u8);
        }
    }
    let pp = winner.get_pokemon();
    let token = env.pp_fast_begin(pp);
    if env.pp_get(pp, 0x97) != 0 {
        gain.iter_mut().for_each(|g| *g = g.wrapping_shl(1));
    }
    for (i, e) in EV_TABLE.iter().enumerate() {
        let v = gain[i] as u32;
        if v == 0 {
            continue;
        }
        let id = e.1 as u32;
        let new = (v + env.pp_get(pp, id)).min(0xFF);
        env.pp_put(pp, id, new);
        out[5 + i] = v as u8;
    }
    env.pp_fast_end(pp, token);
}

/// C: `FUN_overlay_d_93__021cb2d4`: experience for the party of `list` after `fainted` went
/// down. `out` = 6 entries of 12 bytes (`u32` exp, `u8` boosted flag at +4, EV gains at +5..).
/// `mons` resolves the addresses the party queries (`021b9c00`, `021b9934`, `021b9bf0`)
/// return to BattleMon images.
pub fn exp_distribute<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    list: u32,
    fainted: &BattleMon,
    out: &mut [u8],
    mons: &dyn Fn(u32) -> BattleMon,
) {
    use crate::calc::{fx_mul, u32_div};
    let main = w.u32_at(4);
    let pokecon = w.u32_at(8);
    // exp_yield (FUN_overlay_d_93__021d7eb4)
    let species = fainted.species() as u32;
    let form = fainted.get(value::FORM, env) & 0xFFFF;
    let level = fainted.get(value::LEVEL, env) & 0xFFFF;
    let base = env.personal(species, form, 9);
    let mut exp = u32_div(base.wrapping_mul(level), 5);
    let count = env.call(0x021B_9B94, &mut [Arg::V(list)]);
    let mut lucky = 0u32;
    if env.call(0x021B_86B8, &mut [Arg::V(main)]) == 1 {
        exp = u32_div(exp.wrapping_mul(15), 10);
    }
    out[..72].fill(0);
    let member =
        |env: &mut E, k: u32| mons(env.call(0x021B_9C00, &mut [Arg::V(list), Arg::V(k & 0xFF)]));
    // holders of item 216 (Exp. Share)
    let mut i = 0u32;
    while i < count {
        let m = member(env, i);
        if !m.is_fainted(env) && m.held_item() == 216 {
            lucky = (lucky + 1) & 0xFFFF;
        }
        i = (i + 1) & 0xFFFF;
    }
    if lucky != 0 {
        // the original divides the half that was taken away (r0 is still `exp >> 1`)
        let half = exp >> 1;
        exp -= half;
        let mut share = u32_div(half, lucky);
        if share == 0 {
            share = 1;
        }
        let mut i = 0u32;
        while i < count {
            let m = member(env, i);
            if !m.is_fainted(env) && m.held_item() == 216 {
                wr32(&mut out[12 * i as usize..], share);
            }
            i = (i + 1) & 0xFFFF;
        }
    }
    // pokemon that faced the fainted one
    let faced = fainted.faced_count() as i32;
    let mut alive = 0u32;
    let mut i = 0u32;
    while (i as i32) < faced {
        let id = fainted.faced(i & 0xFF) as u32;
        let m = mons(env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(id)]));
        if !m.is_fainted(env) {
            alive = (alive + 1) & 0xFF;
        }
        i = (i + 1) & 0xFFFF;
    }
    let mut share = u32_div(exp, alive);
    if share == 0 {
        share = 1;
    }
    let mut k = 0u32;
    while k < count {
        let m = member(env, k);
        if !m.is_fainted(env) {
            let pid = m.poke_id() as u32;
            let mut j = 0u32;
            while j < faced as u32 {
                if pid == fainted.faced(j) as u32 {
                    let at = 12 * k as usize;
                    let v = rd32(&out[at..]).wrapping_add(share);
                    wr32(&mut out[at..], v);
                }
                j = (j + 1) & 0xFF;
            }
        }
        k = (k + 1) & 0xFFFF;
    }
    // scale each share by the levels and the boosting conditions
    let mut i = 0u32;
    while i < count {
        let at = 12 * i as usize;
        if rd32(&out[at..]) != 0 {
            let trainer = env.call(0x021B_871C, &mut [Arg::V(main)]);
            let m = member(env, i);
            let pp = m.get_pokemon();
            let lvl_m = m.get(value::LEVEL, env) & 0xFFFF;
            let lvl_f = fainted.get(value::LEVEL, env) & 0xFFFF;
            let scaled = exp_scale(env, &m, rd32(&out[at..]), lvl_m, lvl_f);
            wr32(&mut out[at..], scaled);
            if env.call(0x0201_8480, &mut [Arg::V(pp), Arg::V(trainer)]) == 0 {
                let own = env.pp_get(pp, 0xC);
                let other = env.call(0x0200_8564, &mut [Arg::V(trainer)]);
                let fx = if own == other { 0x1800 } else { 0x1B33 };
                let v = fx_mul(rd32(&out[at..]), fx);
                wr32(&mut out[at..], v);
                out[at + 4] = 1;
            }
            if m.held_item() == 231 {
                let v = fx_mul(rd32(&out[at..]), 0x1800);
                wr32(&mut out[at..], v);
                out[at + 4] = 1;
            }
            let cur = rd32(&out[at..]);
            let v = env.call(0x0201_1168, &mut [Arg::V(cur)]);
            wr32(&mut out[at..], v);
        }
        i = (i + 1) & 0xFFFF;
    }
    // effort values
    let mut i = 0u32;
    while i < count {
        let at = 12 * i as usize;
        if rd32(&out[at..]) != 0 {
            let winner = mons(env.call(0x021B_9BF0, &mut [Arg::V(list), Arg::V(i & 0xFF)]));
            ev_gain(env, &winner, fainted, &mut out[at..at + 12]);
        }
        i = (i + 1) & 0xFFFF;
    }
}

// ---- capture roll ----------------------------------------------------------------------------------------

/// `(a * b + 0x800) >> 12` on signed 64-bit operands, low 32 bits (`_ll_mul` + rounding).
fn mul_round_fx(a: u32, b: u32) -> u32 {
    let p = (a as i32 as i64)
        .wrapping_mul(b as i32 as i64)
        .wrapping_add(0x800);
    ((p as u64) >> 12) as u32
}

/// C: `FUN_overlay_d_93__021cbb34`: the capture check of ball `ball` against `mon` (at
/// `mon_addr`). Returns 1 when the thrower may go on (the roll was made), 0 when a shake failed.
/// `count` receives the number of shakes (3 for a master ball), `flag` the critical-capture
/// roll. `blob` is the structure `021b8714(main)` returns (see [`ball_multiplier`]).
#[allow(clippy::too_many_arguments)]
pub fn capture_roll<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    mon: &BattleMon,
    mon_addr: u32,
    ball: u32,
    count: &mut [u8],
    flag: &mut [u8],
    blob_addr: u32,
    blob: &[u8],
) -> u32 {
    use black_core::softfloat::{f2i, fadd, fsub, u2f};
    flag[0] = 0;
    if ball == 1 {
        count[0] = 3;
        return 1;
    }
    let hp3 = mon.get(value::MAX_HP, env).wrapping_mul(3);
    let d = hp3.wrapping_sub(mon.get(value::HP, env).wrapping_mul(2));
    let f = if d != 0 {
        fadd(0x3F00_0000, u2f(d << 12))
    } else {
        fsub(u2f(0), 0x3F00_0000)
    };
    let mut r6 = f2i(f) as u32;
    if env.call(0x021B_A1A8, &mut [Arg::V(w.u32_at(4)), Arg::V(0x20)]) != 0 {
        r6 = mul_round_fx(r6, capture_bonus(env, w));
    }
    let species = mon.species() as u32;
    let form = mon.get(0x13, env) & 0xFFFF;
    let rate = env.personal(species, form, 8) & 0xFFFF;
    r6 = r6.wrapping_mul(rate);
    let mult = ball_multiplier(env, w, mon, mon_addr, ball, blob_addr, blob);
    r6 = crate::calc::u32_div(mul_round_fx(r6, mult), hp3);
    let factor = match mon.major_status() {
        1 | 4 | 5 => Some(0x1800),
        2 | 3 => Some(0x2800),
        _ => None,
    };
    if let Some(factor) = factor {
        r6 = mul_round_fx(r6, factor);
    }
    let x = env.call(0x0201_11BC, &mut [Arg::V(r6)]);
    flag[0] = capture_shake(env, w, x as i32) as u8;
    if (x as i32) >= 0xFF000 {
        count[0] = if flag[0] == 0 { 3 } else { 1 };
        return 1;
    }
    let shakes = if flag[0] == 0 { 3 } else { 1 };
    let a = env.call(0x0207_C768, &mut [Arg::V(0xFF000), Arg::V(x)]);
    let a = env.call(0x0207_C7B4, &mut [Arg::V(a)]);
    let a = env.call(0x0207_C7B4, &mut [Arg::V(a)]);
    let limit = (env.call(0x0207_C768, &mut [Arg::V(1 << 28), Arg::V(a)]) as i32 >> 12) as u32;
    count[0] = 0;
    for _ in 0..shakes {
        if env.call(0x021D_78AC, &mut [Arg::V((1u32 << 28) >> 12)]) >= limit {
            return 0;
        }
        count[0] = count[0].wrapping_add(1);
    }
    1
}

// ---- throwing a ball -------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021cba40`: `mon` throws ball `ball` at the first living target of the
/// side `021b8778` lists. In a wild battle (`021b86b8(main) == 0`) the capture check
/// ([`capture_roll`]) runs and command 0x46 is put on the command queue `q` (with the shake
/// count and flags); otherwise command 0x47 is put when a target exists. Returns 1 in the first
/// case, 0 in the second. `mons` resolves the addresses `021b98ac` returns to BattleMon images
/// (`target_blob` is the same image, used by [`ball_multiplier`] for `021b8714`'s structure).
#[allow(clippy::too_many_arguments)]
pub fn throw_ball<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    mon: &BattleMon,
    ball: u32,
    q: &mut [u8],
    scratch: &mut [u8],
    mons: &dyn Fn(u32) -> BattleMon,
    blob_addr: u32,
    blob: &[u8],
) -> u32 {
    let main = w.u32_at(4);
    let pokecon = w.u32_at(8);
    let mut target_addr = 0u32;
    let mut best = 6u32;
    // the original keeps them in one stack array: flag, shake count, target ids
    let (mut flag, mut count, mut ids) = ([0u8; 1], [0u8; 1], [0u8; 6]);
    let id = mon.poke_id() as u32;
    let r2 = env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(id)],
    );
    let r1 = ((best << 8) | r2) & 0xFFFF;
    let n = env.call(
        0x021B_8778,
        &mut [Arg::V(main), Arg::V(r1), Arg::Out(&mut ids)],
    );
    let mut i = 0u32;
    while i < n {
        target_addr = env.call(
            0x021B_98AC,
            &mut [Arg::V(pokecon), Arg::V(ids[i as usize & 0xFF] as u32)],
        );
        if !mons(target_addr).is_fainted(env) {
            best = ids[i as usize] as u32;
            break;
        }
        i = (i + 1) & 0xFF;
    }
    if env.call(0x021B_86B8, &mut [Arg::V(main)]) != 0 {
        if best != 6 {
            put_cmd(env, q, 0x47, &[best, ball], scratch);
        }
        return 0;
    }
    if best == 6 {
        return 1;
    }
    let target = mons(target_addr);
    let ok = capture_roll(
        env,
        w,
        &target,
        target_addr,
        ball,
        &mut count,
        &mut flag,
        blob_addr,
        blob,
    ) & 0xFF;
    let mut r4 = 0;
    if ok != 0 {
        w.set_u32_at(0x14, 6);
        w.set_u8_at(0x784, best as u8);
        r4 = u32::from(env.call(0x021B_8724, &mut [Arg::V(main), Arg::V(target_addr)]) == 0);
        target.set_pp_0x98(ball, env);
    }
    put_cmd(
        env,
        q,
        0x46,
        &[best, count[0] as u32, ok, r4, flag[0] as u32, ball],
        scratch,
    );
    1
}

// ---- using an item on a party pokemon -------------------------------------------------------------------

/// The 23 entries at 0x021EFDE4: `(item parameter id, kind, flag, handler address)`. Kind 0:
/// always usable, 1: only in battle slots, 2: after the use; flag 1 marks the effects that
/// run once the item is chosen (the ones that need the item to match by id).
const ITEM_TABLE: [(u16, u8, u8, u32); 23] = [
    (0x12, 0, 0, 0x021C_BF30),
    (0x13, 0, 0, 0x021C_BF40),
    (0x14, 0, 0, 0x021C_BF50),
    (0x15, 0, 0, 0x021C_BF60),
    (0x16, 0, 0, 0x021C_BF70),
    (0x17, 0, 0, 0x021C_BF80),
    (0x18, 0, 0, 0x021C_BF90),
    (0x19, 1, 0, 0x021C_BFA0),
    (0x1A, 0, 0, 0x021C_C008),
    (0x1E, 1, 0, 0x021C_C098),
    (0x1F, 1, 0, 0x021C_C0A8),
    (0x20, 1, 0, 0x021C_C0B8),
    (0x21, 1, 0, 0x021C_C0C8),
    (0x22, 1, 0, 0x021C_C0D8),
    (0x23, 1, 0, 0x021C_C0E8),
    (0x24, 1, 0, 0x021C_C0F8),
    (0x27, 0, 0, 0x021C_C178),
    (0x28, 0, 0, 0x021C_C214),
    (0x29, 0, 0, 0x021C_C2C0),
    (0x265, 2, 1, 0x021C_C4AC),
    (0x263, 2, 1, 0x021C_C508),
    (0x264, 2, 1, 0x021C_C558),
    (0x266, 2, 1, 0x021C_C5F4),
];

/// The command-queue and scratch buffers `use_item` writes to.
pub struct ItemIo<'a> {
    pub q: &'a mut [u8],
    pub scratch: &'a mut [u8],
}

/// How [`use_item`] reaches the item-effect handlers of [`ITEM_TABLE`]: `(P, target, item, r3,
/// extra)` with the handler's address `h`. [`PortedEffects`] runs the ports of this module; the
/// differential test of `021cb730` stubs them by address instead (`force` stubs in its spec).
pub trait ItemEffects<E> {
    #[allow(clippy::too_many_arguments)]
    fn run(
        &mut self,
        env: &mut E,
        w: &mut SvFlowWork,
        waddr: u32,
        h: u32,
        target: &mut Option<BattleMon>,
        target_addr: u32,
        item: u32,
        r3: u32,
        extra: u32,
        io: &mut ItemIo<'_>,
    ) -> u32;
}

/// The handlers ported in this module.
pub struct PortedEffects;

impl<E: FlowEnv + BppEnv> ItemEffects<E> for PortedEffects {
    fn run(
        &mut self,
        env: &mut E,
        w: &mut SvFlowWork,
        waddr: u32,
        h: u32,
        target: &mut Option<BattleMon>,
        target_addr: u32,
        item: u32,
        r3: u32,
        extra: u32,
        io: &mut ItemIo<'_>,
    ) -> u32 {
        run_entry(env, w, waddr, h, target, target_addr, item, r3, extra, io)
    }
}

/// Calls the item-effect handler at `h` the way `use_item` does: `(P, target, item, r3, extra)`.
#[allow(clippy::too_many_arguments)]
fn run_entry<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    h: u32,
    target: &mut Option<BattleMon>,
    target_addr: u32,
    item: u32,
    r3: u32,
    extra: u32,
    io: &mut ItemIo<'_>,
) -> u32 {
    let m = target.as_mut().expect("item effect on a null BattleMon");
    match h {
        0x021C_BF30 => cure_entry_2(env, w, waddr, m),
        0x021C_BF40 => cure_entry_5(env, w, waddr, m),
        0x021C_BF50 => cure_entry_4(env, w, waddr, m),
        0x021C_BF60 => cure_entry_3(env, w, waddr, m),
        0x021C_BF70 => cure_entry_1(env, w, waddr, m),
        0x021C_BF80 => cure_entry_6(env, w, waddr, m),
        0x021C_BF90 => cure_all_entry(env, w, waddr, m),
        0x021C_BFA0 => msg_side(env, w, waddr, m),
        0x021C_C008 => revive(env, w, waddr, m, item),
        0x021C_C098 => stage_entry_1(env, w, waddr, m, r3),
        0x021C_C0A8 => stage_entry_2(env, w, waddr, m, r3),
        0x021C_C0B8 => stage_entry_3(env, w, waddr, m, r3),
        0x021C_C0C8 => stage_entry_4(env, w, waddr, m, r3),
        0x021C_C0D8 => stage_entry_5(env, w, waddr, m, r3),
        0x021C_C0E8 => stage_entry_6(env, w, waddr, m, r3),
        0x021C_C0F8 => raise_crit(env, io.q, io.scratch, m, r3),
        0x021C_C178 => pp_restore_one(env, w, waddr, m, item, extra),
        0x021C_C214 => pp_restore_all(env, w, waddr, m, item),
        0x021C_C2C0 => heal(env, w, waddr, m, item),
        0x021C_C4AC => held_item_check(env, waddr, m, target_addr),
        0x021C_C508 => ability_trigger(env, w, waddr, m, target_addr),
        0x021C_C558 => show_held_item(env, w, waddr, m),
        0x021C_C5F4 => flinch_message(env, w, waddr, m),
        _ => unreachable!("item effect handler {h:#x}"),
    }
}

/// C: `FUN_overlay_d_93__021cb730`: `user` uses `item` on the party member in slot `slot`
/// (6 = none; `extra` is handed to the effects, the PP slot of an Elixir-like item). Puts the
/// result on the command queue `io.q`. `img` is the BattleMon image `021b9bf0` returns (at
/// `img_base`): the effects may change it. Returns the "item was used up" flag (0 or 1).
///
/// The spec (B10-spec2) keeps the 23 handlers as `force` stubs by address: they are ported and
/// verified on their own, and as real code they would read a null BattleMon whenever the slot is
/// 6 (a reachable input), which no port can reproduce. The test wiring therefore runs it with
/// [`ItemEffects`] stubbing them by address.
#[allow(clippy::too_many_arguments)]
pub fn use_item<E: FlowEnv + BppEnv, X: ItemEffects<E>>(
    env: &mut E,
    fx: &mut X,
    w: &mut SvFlowWork,
    waddr: u32,
    user: &BattleMon,
    item: u32,
    extra: u32,
    slot: u32,
    io: &mut ItemIo<'_>,
    img: &mut [u8],
    img_base: u32,
) -> u32 {
    let snapshot = img.to_vec();
    let mut target: Option<BattleMon> = None;
    let mut target_off = 0usize;
    let r = use_item_body(
        env,
        fx,
        w,
        waddr,
        user,
        item,
        extra,
        slot,
        io,
        &snapshot,
        img_base,
        &mut target,
        &mut target_off,
    );
    if let Some(t) = target {
        img[target_off..target_off + t.raw.len()].copy_from_slice(&t.raw);
    }
    r
}

#[allow(clippy::too_many_arguments)]
fn use_item_body<E: FlowEnv + BppEnv, X: ItemEffects<E>>(
    env: &mut E,
    fx: &mut X,
    w: &mut SvFlowWork,
    waddr: u32,
    user: &BattleMon,
    item: u32,
    extra: u32,
    slot: u32,
    io: &mut ItemIo<'_>,
    snapshot: &[u8],
    img_base: u32,
    target: &mut Option<BattleMon>,
    target_off: &mut usize,
) -> u32 {
    use crate::battle_mon::BPP_SIZE;
    let main = w.u32_at(4);
    let pokecon = w.u32_at(8);
    let mons = |a: u32| {
        let o = a.wrapping_sub(img_base) as usize;
        BattleMon::from_bytes(snapshot[o..o + BPP_SIZE].try_into().unwrap())
    };
    let user_id = user.poke_id() as u32;
    let client = env.call(0x021B_8D08, &mut [Arg::V(user_id)]);
    let mut target_addr = 0u32;
    let mut target_id = 0u32;
    let mut cba30 = 0u32;
    let mut mode = 0u32; // var_2c
    if slot != 6 {
        let list = env.call(0x021B_9AEC, &mut [Arg::V(pokecon), Arg::V(client)]);
        let count = env.call(0x021B_6994, &mut [Arg::V(main), Arg::V(client)]);
        target_addr = env.call(0x021B_9BF0, &mut [Arg::V(list), Arg::V(slot)]);
        *target_off = target_addr.wrapping_sub(img_base) as usize;
        let t = mons(target_addr);
        target_id = t.poke_id() as u32;
        *target = Some(t);
        cba30 = env.call(
            0x021C_BA30,
            &mut [Arg::V(waddr + OFS_TABLE), Arg::V(target_id)],
        );
        let mut m = 0;
        if slot >= count {
            m = 2;
        }
        mode = m & 0xFF;
        if env.call(0x021B_8588, &mut [Arg::V(main)]) == 3 && mode == 2 && slot < 3 {
            mode = 1;
        }
    }
    let term = 0xFFFF_0000;
    if w.u32_at(0x18) != 1 {
        put_cmd_list(env, io.q, 0x5A, 0x21, &[client, item, term]);
        if item_param(env, item, 0xF) == 4 {
            let used = throw_ball(
                env, w, user, item, io.q, io.scratch, &mons, img_base, snapshot,
            );
            if used != 0 {
                env.call(
                    0x021B_9090,
                    &mut [Arg::V(main), Arg::V(client), Arg::V(item)],
                );
            }
            return 0;
        }
        if item_param(env, item, 7) == 3 {
            env.call(
                0x021B_9090,
                &mut [Arg::V(main), Arg::V(client), Arg::V(item)],
            );
            return 1;
        }
    } else {
        put_cmd_list(env, io.q, 0x5A, 0x24, &[client, target_id, item, term]);
        if env.call(0x021B_8FF4, &mut [Arg::V(main)]) == client {
            env.call(0x021B_A1B4, &mut [Arg::V(main), Arg::V(0x6F)]);
        }
    }
    let mut usable = mode == 0 && cba30 != 6;
    if !usable {
        for e in ITEM_TABLE.iter() {
            let (id, kind, flag) = (e.0 as u32, e.1, e.2);
            let hit = if flag != 0 {
                item == id
            } else {
                item_param(env, item, id) != 0
            };
            if !hit {
                continue;
            }
            if kind == 0 || (kind == 1 && mode < 2 && cba30 != 6) {
                usable = true;
                break;
            }
        }
    }
    if !usable {
        put_cmd_list(env, io.q, 0x5A, 0xC4, &[term]);
        return 0;
    }
    if w.u32_at(0x18) == 1 {
        if cba30 != 6 {
            put_cmd(env, io.q, 0x4D, &[cba30, 0x282], io.scratch);
        }
        for e in ITEM_TABLE.iter() {
            if e.2 == 0 || item != e.0 as u32 {
                continue;
            }
            let mut r7 = 0;
            if env.call(0x021C_8338, &mut [Arg::V(waddr), Arg::V(target_id)]) == 0 {
                let pos = put_cmd_placeholder(env, io.q, 0x4D, io.scratch);
                r7 = fx.run(env, w, waddr, e.3, target, target_addr, item, 0, extra, io);
                if r7 == 1 {
                    put_cmd_at(env, io.q, pos, 0x4D, &[cba30, 0x25E], io.scratch);
                    put_cmd(env, io.q, 0x4C, &[0x236], io.scratch);
                    return 0;
                }
            }
            if r7 == 0 {
                put_cmd_list(env, io.q, 0x5A, 0x44, &[term]);
            }
            put_cmd(env, io.q, 0x4C, &[0x236], io.scratch);
            return 0;
        }
    }
    let mut any = false;
    let pos = put_cmd_placeholder(env, io.q, 0x4D, io.scratch);
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    for e in ITEM_TABLE.iter() {
        let r3 = item_param(env, item, e.0 as u32);
        if r3 != 0 && fx.run(env, w, waddr, e.3, target, target_addr, item, r3, extra, io) != 0 {
            any = true;
        }
    }
    if any {
        let msg = if cba30 != 6 { 0x25E } else { 0x292 };
        put_cmd_at(env, io.q, pos, 0x4D, &[cba30, msg], io.scratch);
        if w.u32_at(0x18) != 1 {
            env.call(
                0x021B_9090,
                &mut [Arg::V(main), Arg::V(client), Arg::V(item)],
            );
            env.call(
                0x021B_A250,
                &mut [Arg::V(main), Arg::V(target_addr), Arg::V(item)],
            );
        }
    } else {
        put_cmd_list(env, io.q, 0x5A, 0x44, &[term]);
    }
    set_word(&mut w.mem[OFS_QHEAD..], head);
    put_cmd(env, io.q, 0x4C, &[0x236], io.scratch);
    0
}

// ---- message arguments ----------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021caa24`: the target id a message names. For the record kind 0xAE
/// (`rec` u16 at +0; its kind word at +0xC): when the kind is 0 and `id` is the `expected` id
/// (or `expected` is 6) it is `021d80bc(main, pokecon, r2, 0xAE, id)`; for kind 7 and a
/// different id it is `id`; otherwise `expected`.
pub fn target_override<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    rec: &[u8],
    r2: u32,
    id: u32,
    expected: u32,
) -> u32 {
    if u16::from_le_bytes([rec[0], rec[1]]) != 0xAE {
        return expected;
    }
    let kind = rd32(&rec[0xC..]);
    if kind == 0 {
        if id == expected || expected == 6 {
            // the callee's fifth word is whatever the caller's stack holds: the saved r3
            return env.call(
                0x021D_80BC,
                &mut [
                    Arg::V(w.u32_at(4)),
                    Arg::V(w.u32_at(8)),
                    Arg::V(r2),
                    Arg::V(0xAE),
                    Arg::V(id),
                ],
            );
        }
        return expected;
    }
    if kind == 7 && id != expected {
        return id;
    }
    expected
}

/// C: `FUN_overlay_d_93__021ca8f8`: pushes the message arguments of the record kind in
/// `rec[0xC]` onto the object `obj` (through the veneers `021ca9d4`..`021caa14`). `who` is a
/// party id or 0x1F (the pokemon `mon` itself). Returns the number of argument groups (0..2).
#[allow(clippy::too_many_arguments)]
pub fn message_args_a<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    mon_addr: u32,
    rec: &[u8],
    who: u32,
    obj: u32,
) -> u32 {
    let (main, pokecon) = (w.u32_at(4), w.u32_at(8));
    env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(mon.poke_id() as u32)],
    );
    match rd32(&rec[0xC..]) {
        0 | 3 | 4 | 5 | 9 => {
            let m = if who == 0x1F {
                mon_of_main_a(env, waddr, &w.mem[..])
            } else {
                env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)])
            };
            env.call(0x021C_A9D4, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        1 | 7 => {
            let m = if who == 0x1F {
                mon_addr
            } else {
                env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)])
            };
            env.call(0x021C_A9E4, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        8 => {
            env.call(0x021C_A9F4, &mut [Arg::V(obj), Arg::V(mon_addr)]);
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_AA04, &mut [Arg::V(obj), Arg::V(m)]);
            2
        }
        13 => {
            if who == 0x1F {
                return 0;
            }
            let m = env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)]);
            env.call(0x021C_AA14, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        _ => 0,
    }
}

/// The shared tail of [`message_args_b`] and [`message_args_c`]: when `r` (a BattleMon address)
/// is not null, asks the event system about it (`dispatch_value_bracketed`), follows the party
/// id it answers (anything but 0x1F) and pushes the result with the veneer `veneer`.
#[allow(clippy::too_many_arguments)]
fn message_tail<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    rec: &[u8],
    obj: u32,
    r: u32,
    veneer: u32,
    mons: &dyn Fn(u32) -> BattleMon,
) -> u32 {
    if r == 0 {
        return 0;
    }
    let id = mons(r).poke_id() as u32;
    let answer = dispatch_value_bracketed(env, w, waddr, mon, rec, id);
    let mut r = r;
    if answer != 0x1F {
        r = env.call(0x021B_9934, &mut [Arg::V(w.u32_at(8)), Arg::V(answer)]);
    }
    env.call(veneer, &mut [Arg::V(obj), Arg::V(r)]);
    1
}

/// C: `FUN_overlay_d_93__021caa58`: like [`message_args_a`] for the other message kinds (1..2
/// groups pushed through `021cac24`..`021cace4`); `extra` is the id `021caa24` should expect.
/// `mons` resolves BattleMon addresses.
#[allow(clippy::too_many_arguments)]
pub fn message_args_b<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    mon_addr: u32,
    extra: u32,
    rec: &[u8],
    who: u32,
    obj: u32,
    mons: &dyn Fn(u32) -> BattleMon,
) -> u32 {
    let (main, pokecon) = (w.u32_at(4), w.u32_at(8));
    let client = env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(mon.poke_id() as u32)],
    );
    let tgt = target_override(env, w, rec, mon_addr, client, extra);
    let victim = |env: &mut E| env.call(0x021B_98AC, &mut [Arg::V(pokecon), Arg::V(tgt)]);
    let tail = |env: &mut E, w: &mut SvFlowWork, r: u32| {
        message_tail(env, w, waddr, mon, rec, obj, r, 0x021C_ACE4, mons)
    };
    match rd32(&rec[0xC..]) {
        0 | 3 => {
            let r = victim(env);
            tail(env, w, r)
        }
        9 => {
            env.call(0x021D_78AC, &mut [Arg::V(2)]);
            let r = mon_of_main_a(env, waddr, &w.mem[..]);
            tail(env, w, r)
        }
        5 => {
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_AC24, &mut [Arg::V(obj), Arg::V(m)]);
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_AC34, &mut [Arg::V(obj), Arg::V(m)]);
            2
        }
        4 => {
            let m = mon_of_main_b(env, waddr, &w.mem[..]);
            env.call(0x021C_AC44, &mut [Arg::V(obj), Arg::V(m)]);
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_AC54, &mut [Arg::V(obj), Arg::V(m)]);
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_AC64, &mut [Arg::V(obj), Arg::V(m)]);
            3
        }
        8 => {
            env.call(0x021C_AC74, &mut [Arg::V(obj), Arg::V(mon_addr)]);
            let m = mon_of_main_b(env, waddr, &w.mem[..]);
            env.call(0x021C_AC84, &mut [Arg::V(obj), Arg::V(m)]);
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_AC94, &mut [Arg::V(obj), Arg::V(m)]);
            let m = mon_of_main_a(env, waddr, &w.mem[..]);
            env.call(0x021C_ACA4, &mut [Arg::V(obj), Arg::V(m)]);
            3
        }
        7 => {
            let m = if who == 0x1F {
                mon_addr
            } else {
                env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)])
            };
            env.call(0x021C_ACB4, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        1 => {
            let m = victim(env);
            env.call(0x021C_ACC4, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        2 => {
            let m = mon_of_main_b(env, waddr, &w.mem[..]);
            env.call(0x021C_ACD4, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        13 => {
            if who == 0x1F {
                return 0;
            }
            let r = env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)]);
            tail(env, w, r)
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021cacf4`: the multi-pokemon message kinds (`list` = the structure
/// `021b99e0(client)` returns: `u16` counts at +0 and +2, party ids from +4 and +7).
#[allow(clippy::too_many_arguments)]
pub fn message_args_c<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    mon: &BattleMon,
    mon_addr: u32,
    extra: u32,
    rec: &[u8],
    who: u32,
    obj: u32,
    list: &[u8],
    mons: &dyn Fn(u32) -> BattleMon,
) -> u32 {
    let (main, pokecon) = (w.u32_at(4), w.u32_at(8));
    let client = env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(mon.poke_id() as u32)],
    );
    env.call(0x021B_99E0, &mut [Arg::V(client)]);
    let n0 = u16::from_le_bytes([list[0], list[1]]) as u32;
    let n1 = u16::from_le_bytes([list[2], list[3]]) as u32;
    let tgt = target_override(env, w, rec, mon_addr, client, extra);
    let party = |env: &mut E, id: u32| env.call(0x021B_98AC, &mut [Arg::V(pokecon), Arg::V(id)]);
    let tail = |env: &mut E, w: &mut SvFlowWork, r: u32| {
        message_tail(env, w, waddr, mon, rec, obj, r, 0x021C_AFC8, mons)
    };
    match rd32(&rec[0xC..]) {
        0 | 3 => {
            let r = party(env, tgt);
            tail(env, w, r)
        }
        9 => {
            let pick = env.call(0x021D_78AC, &mut [Arg::V(n0)]) & 0xFF;
            let r = party(env, list[4 + pick as usize] as u32);
            tail(env, w, r)
        }
        5 => {
            let mut i = 0;
            while i < n0 {
                let m = party(env, list[4 + i as usize] as u32);
                env.call(0x021C_AF18, &mut [Arg::V(obj), Arg::V(m)]);
                i += 1;
            }
            n0 & 0xFF
        }
        4 => {
            let mut count = 0u32;
            let mut i = 0;
            while i < n0 {
                let m = party(env, list[4 + i as usize] as u32);
                env.call(0x021C_AF28, &mut [Arg::V(obj), Arg::V(m)]);
                i += 1;
                count += 1;
            }
            let mut j = 0;
            while j < n1 {
                let other = list[7 + j as usize] as u32;
                if client != other {
                    let m = party(env, other);
                    env.call(0x021C_AF38, &mut [Arg::V(obj), Arg::V(m)]);
                    count += 1;
                }
                j += 1;
            }
            count & 0xFF
        }
        8 => {
            env.call(0x021C_AF48, &mut [Arg::V(obj), Arg::V(mon_addr)]);
            for i in 0..3u32 {
                let r1 = env.call(0x021B_8C38, &mut [Arg::V(client), Arg::V(i & 0xFF)]);
                if r1 != client {
                    let m = party(env, r1);
                    env.call(0x021C_AF58, &mut [Arg::V(obj), Arg::V(m)]);
                }
            }
            for veneer in [0x021C_AF68, 0x021C_AF78, 0x021C_AF88] {
                let m = mon_of_main_a(env, waddr, &w.mem[..]);
                env.call(veneer, &mut [Arg::V(obj), Arg::V(m)]);
            }
            6
        }
        7 => {
            let m = if who == 0x1F {
                mon_addr
            } else {
                env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)])
            };
            env.call(0x021C_AF98, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        1 => {
            let m = party(env, tgt);
            env.call(0x021C_AFA8, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        2 => {
            let m = party(env, tgt);
            env.call(0x021C_AFB8, &mut [Arg::V(obj), Arg::V(m)]);
            1
        }
        13 => {
            if who == 0x1F {
                return 0;
            }
            let r = env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(who)]);
            tail(env, w, r)
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021cafd8`: picks the ally that answers an attack on `mon` (needs the
/// fainted pokemon `021cb144` yields on the other side, the candidate list `021b8778` for the
/// side of `mon`, and the one of two candidates closer to `distance` or, on a tie, with less
/// HP, else random) and pushes it with the veneer `021cb164`. `mv` is the move id. Returns
/// whether it did.
#[allow(clippy::too_many_arguments)]
pub fn pick_ally<E: FlowEnv + BppEnv>(
    env: &mut E,
    w: &SvFlowWork,
    kind: u32,
    mon: &BattleMon,
    mv: u32,
    distance: u32,
    obj: u32,
    mons: &dyn Fn(u32) -> BattleMon,
) -> u32 {
    use crate::calc::fun_021d7f00;
    let (main, pokecon) = (w.u32_at(4), w.u32_at(8));
    if fun_021d7f00(kind) <= 1 {
        return 0;
    }
    if env.call(0x021C_B134, &mut [Arg::V(obj)]) != 1 {
        return 0;
    }
    let fainted_addr = env.call(0x021C_B144, &mut [Arg::V(obj), Arg::V(0)]);
    let fainted = mons(fainted_addr);
    if !fainted.is_fainted(env) {
        return 0;
    }
    let my_id = mon.poke_id() as u32;
    let their_id = fainted.poke_id() as u32;
    // FUN_overlay_d_93__021b9a24: same side
    if u32::from(my_id >= 12) == u32::from(their_id >= 12) {
        return 0;
    }
    let client = env.call(
        0x021B_8D8C,
        &mut [Arg::V(main), Arg::V(pokecon), Arg::V(mon.poke_id() as u32)],
    );
    env.call(0x021C_B154, &mut [Arg::V(obj), Arg::V(fainted_addr)]);
    let r1 = if env.call(0x0201_BEB8, &mut [Arg::V(mv), Arg::V(0xB)]) != 0 {
        (6 << 8) | client
    } else {
        client | 0x100
    } & 0xFFFF;
    let mut ids = [0u8; 8];
    let count = env.call(
        0x021B_8778,
        &mut [Arg::V(main), Arg::V(r1), Arg::Out(&mut ids)],
    );
    let mut slots = [0u32; 3];
    let (mut kept, mut i) = (0usize, 0u32);
    while i < count {
        let m = env.call(
            0x021B_98AC,
            &mut [Arg::V(pokecon), Arg::V(ids[i as usize] as u32)],
        );
        slots[kept] = m;
        if !mons(m).is_fainted(env) {
            ids[kept] = ids[i as usize];
            kept = (kept + 1) & 0xFF;
        }
        i = (i + 1) & 0xFF;
    }
    if kept == 0 {
        return 0;
    }
    let pick = if kept != 2 {
        0
    } else {
        let d = distance & 0xFF;
        let a = (ids[0] as i32 - d as i32).abs();
        let b = (ids[1] as i32 - d as i32).abs();
        let (a, b) = if a != b {
            (a, b)
        } else {
            (
                mons(slots[0]).get(value::HP, env) as i32,
                mons(slots[1]).get(value::HP, env) as i32,
            )
        };
        if a < b {
            0
        } else if b < a {
            1
        } else {
            env.call(0x021D_78AC, &mut [Arg::V(2)]) & 0xFF
        }
    };
    if slots[pick as usize] == 0 {
        return 0;
    }
    env.call(0x021C_B164, &mut [Arg::V(obj)]);
    1
}

#[cfg(test)]
mod tests;
