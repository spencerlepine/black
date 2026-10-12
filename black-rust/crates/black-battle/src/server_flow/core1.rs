//! Flow core, part 1 (0x021BB968-0x021BD1D0; `021bb968` creates the work; B10f).
//!
//! The per-turn action list lives in the work: `+0x782` is the record count (u8) and `+0x7E0` are up
//! to six 16-byte records (`+0` BattleMon pointer, `+4` action word, `+8` priority word, `+0xC` client
//! id, `+0xD` done flag). Specs: `tools/rom-diff/svflow_specs/core1.spec` (45 functions; the veneers
//! and the thunk of the range are listed at the top of the spec).
//!
//! B10f-spec ports (smoke ports that prove the spec grammar; the rest is for the B10f port job).
//! Offsets and record addresses are offsets into the work image; callees outside this area go through
//! [`FlowEnv`], `server_side` ones are called directly (they are ported and run as real code).

use super::cmd_queue::{advance_fields, put_cmd, set_word};
use super::env::{Arg, FlowEnv};
use super::events;
use super::server_side;
use super::SvFlowWork;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021B_C550,
    0x021B_CB68,
    0x021B_CA08,
    0x021B_C874,
    0x021B_C4DC,
    0x021B_C55C,
    0x021B_BD20,
    0x021B_BD80,
    0x021B_BC34,
    0x021B_C8EC,
    0x021B_C948,
    0x021B_CF34,
    0x021B_CEFC,
    0x021B_CF70,
    0x021B_D080,
    0x021B_C2B0,
    0x021B_C310,
    0x021B_CAAC,
    0x021B_CA50,
    0x021B_CB14,
    0x021B_CB98,
    0x021B_CC50,
    0x021B_D148,
    0x021B_D170,
    0x021B_D1B0,
    0x021B_C0E8,
    0x021B_C3B0,
    0x021B_C360,
    0x021B_C390,
    0x021B_C568,
    0x021B_CFAC,
    0x021B_CCE0,
    0x021B_BCD4,
    0x021B_BC44,
    0x021B_BD2C,
    0x021B_BD8C,
    0x021B_B968,
    0x021B_B9D8,
    0x021B_BB48,
    0x021B_BB60,
    0x021B_C220,
    0x021B_C418,
    0x021B_C610,
    0x021B_BFA8,
    0x021B_D0C0,
];

const OFS_REC_COUNT: usize = 0x782;
const OFS_RECORDS: usize = 0x7E0;
const REC_SIZE: usize = 16;

/// C: `FUN_021bc550`: address of the helper struct at work + 0x4B8 (`work` is the work's address).
pub fn struct_0x4b8(work: u32) -> u32 {
    work.wrapping_add(0x4B8)
}

/// C: `FUN_overlay_d_93__021bcb68`: index of the record at work offset `rec` in the action list,
/// or the record count when it is not an element of the list.
pub fn record_index(w: &SvFlowWork, rec: u32) -> u8 {
    let n = w.u8_at(OFS_REC_COUNT);
    for i in 0..n {
        if (OFS_RECORDS + REC_SIZE * i as usize) as u32 == rec {
            return i;
        }
    }
    n
}

/// C: `FUN_overlay_d_93__021bca08`: work offset of the first record whose BattleMon id
/// (`021d5440`) is `id`.
pub fn record_by_mon<E: FlowEnv>(env: &mut E, w: &SvFlowWork, id: u32) -> Option<usize> {
    let n = w.u8_at(OFS_REC_COUNT);
    for i in 0..n {
        let rec = OFS_RECORDS + REC_SIZE * i as usize;
        let mon = w.u32_at(rec);
        if id == env.call(0x021D_5440, &mut [Arg::V(mon)]) {
            return Some(rec);
        }
    }
    None
}

/// C: `FUN_overlay_d_93__021bc874`: sorts the first `n` records of `recs` (the action list) by
/// priority word (`+8`, unsigned, larger first); records with equal priority swap when
/// `021d78ac(2)` is non-zero.
pub fn sort_records<E: FlowEnv>(env: &mut E, recs: &mut [u8], n: u32) {
    for i in 0..n as usize {
        for j in i + 1..n as usize {
            let pa = u32::from_le_bytes(
                recs[i * REC_SIZE + 8..i * REC_SIZE + 12]
                    .try_into()
                    .unwrap(),
            );
            let pb = u32::from_le_bytes(
                recs[j * REC_SIZE + 8..j * REC_SIZE + 12]
                    .try_into()
                    .unwrap(),
            );
            if pa > pb {
                continue;
            }
            if pa == pb && env.call(0x021D_78AC, &mut [Arg::V(2)]) == 0 {
                continue;
            }
            let (lo, hi) = recs.split_at_mut(j * REC_SIZE);
            lo[i * REC_SIZE..(i + 1) * REC_SIZE].swap_with_slice(&mut hi[..REC_SIZE]);
        }
    }
}

/// C: `FUN_overlay_d_93__021bc4dc`: rebuilds the id list `{u8 count; u8 ids[]}` at work offset
/// `list`: for each client `i` (0..4) `021bc540` returns how many ids it wrote into a 4-byte
/// buffer; each id is added to the server-side object's list (`021bb930` on `s`) and, when there
/// was at least one, `i` is appended to `list`. Returns 1 when any client had ids.
pub fn collect_list<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, s: &mut [u8], list: usize) -> u32 {
    w.mem[list] = 0;
    let mut any = 0;
    for i in 0..4u32 {
        let mut out = [0u8; 4];
        let n = env.call(0x021B_C540, &mut [Arg::P, Arg::V(i), Arg::Out(&mut out)]);
        if n == 0 {
            continue;
        }
        for k in 0..n as usize {
            server_side::list_add(s, out.get(k).copied().unwrap_or(0) as u32);
        }
        any = 1;
        let count = w.mem[list];
        w.mem[list] = count.wrapping_add(1);
        w.mem[list + 1 + count as usize] = i as u8;
    }
    any
}

const OFS_FLAG_77C: usize = 0x77C;
const OFS_QHEAD: usize = 0x1D74;

/// C: `FUN_021bc55c`: byte at work + 0x784.
pub fn byte_784(w: &SvFlowWork) -> u8 {
    w.u8_at(0x784)
}

/// C: `FUN_overlay_d_93__021bbd20`: clears the byte at work + 0x77C.
pub fn clear_77c_a(w: &mut SvFlowWork) {
    w.set_u8_at(OFS_FLAG_77C, 0);
}

/// C: `FUN_overlay_d_93__021bbd80`: same as [`clear_77c_a`] (a second copy in the original).
pub fn clear_77c_b(w: &mut SvFlowWork) {
    w.set_u8_at(OFS_FLAG_77C, 0);
}

/// C: `FUN_overlay_d_93__021bbc34`: clears the bytes at work + 0x77C and + 0x783.
pub fn clear_77c_783(w: &mut SvFlowWork) {
    w.set_u8_at(OFS_FLAG_77C, 0);
    w.set_u8_at(0x783, 0);
}

/// C: `FUN_overlay_d_93__021bc8ec`: priority bits of mon `mon_id`/`slot` (event 0x11): pushes an
/// event-variable frame (0x6C7), sets variables 3 (id of `slot`), 0x12 (`mon`) and 0x18
/// (`0201bd28(mon, 6) + 7`, low byte), dispatches and returns variable 0x18 (low byte).
pub fn priority_bits<E: FlowEnv>(env: &mut E, mon: u32, slot: u32) -> u32 {
    let k = env.call(0x0201_BD28, &mut [Arg::V(mon), Arg::V(6)]);
    let k = k.wrapping_add(7) & 0xFF;
    env.call(0x021D_74F8, &mut [Arg::V(0x6C7)]);
    let id = env.call(0x021D_5440, &mut [Arg::V(slot)]);
    env.call(0x021D_75EC, &mut [Arg::V(3), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x12), Arg::V(mon)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x18), Arg::V(k)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x11)]);
    let r = env.call(0x021D_775C, &mut [Arg::V(0x18)]) & 0xFF;
    env.call(0x021D_7540, &mut [Arg::V(0x6C7 + 6)]);
    r
}

/// C: `FUN_overlay_d_93__021bc948`: speed value (event 0x13, frame 0x6E0) of `mon`, limited to 10000;
/// `invert` != 0 returns `10000 - v` when variable 0x4A is set.
pub fn speed_value<E: FlowEnv>(env: &mut E, mon: u32, invert: u32) -> u32 {
    let base = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xC)]);
    env.call(0x021D_74F8, &mut [Arg::V(0x6E0)]);
    let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0x2F), Arg::V(base)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x51), Arg::V(1)]);
    env.call(0x021D_75A8, &mut [Arg::V(0x4A), Arg::V(0)]);
    env.call(
        0x021D_7674,
        &mut [Arg::V(0x35), Arg::V(0x1000), Arg::V(410), Arg::V(0x2_0000)],
    );
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0x13)]);
    let a = env.call(0x021D_775C, &mut [Arg::V(0x2F)]);
    let b = env.call(0x021D_775C, &mut [Arg::V(0x35)]);
    let mut v = env.call(0x021D_7B10, &mut [Arg::V(a), Arg::V(b)]);
    if env.call(0x021D_62A8, &mut [Arg::V(mon)]) == 1
        && env.call(0x021D_775C, &mut [Arg::V(0x51)]) != 0
    {
        v = v.wrapping_mul(0x19) / 100;
    }
    if v > 0x2710 {
        v = 0x2710;
    }
    if invert != 0 && env.call(0x021D_775C, &mut [Arg::V(0x4A)]) != 0 {
        v = 0x2710u32.wrapping_sub(v);
    }
    env.call(0x021D_7540, &mut [Arg::V(0x700)]);
    v & 0xFFFF
}

/// C: `FUN_overlay_d_93__021bcf34`: event 1, frame 0x864: variables 2 (id of `slot`) and 0xC (`val`).
pub fn event_1<E: FlowEnv>(env: &mut E, slot: u32, val: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(0x864)]);
    let id = env.call(0x021D_5440, &mut [Arg::V(slot)]);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0xC), Arg::V(val)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(1)]);
    env.call(0x021D_7540, &mut [Arg::V(0x864 + 4)]);
}

/// C: `FUN_overlay_d_93__021bcefc`: [`event_1`] bracketed by the command queue header word
/// (work + 0x1D74): advance its fields before, restore the old word after.
pub fn event_1_bracketed<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, slot: u32, val: u32) {
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    event_1(env, slot, val);
    set_word(&mut w.mem[OFS_QHEAD..], head);
}

/// C: `FUN_overlay_d_93__021bcf70`: event 2, frame 0x874; otherwise as [`event_1`].
pub fn event_2<E: FlowEnv>(env: &mut E, slot: u32, val: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(0x874)]);
    let id = env.call(0x021D_5440, &mut [Arg::V(slot)]);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_75EC, &mut [Arg::V(0xC), Arg::V(val)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(2)]);
    env.call(0x021D_7540, &mut [Arg::V(0x874 + 4)]);
}

/// C: `FUN_overlay_d_93__021bd080`: event 0xE (frame 0x8D0) for `slot` with variable 0x41 = 0;
/// returns variable 0x41.
pub fn event_0e<E: FlowEnv>(env: &mut E, slot: u32) -> u32 {
    env.call(0x021D_74F8, &mut [Arg::V(0x8D0)]);
    let id = env.call(0x021D_5440, &mut [Arg::V(slot)]);
    env.call(0x021D_75EC, &mut [Arg::V(2), Arg::V(id)]);
    env.call(0x021D_7630, &mut [Arg::V(0x41), Arg::V(0)]);
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(0xE)]);
    let r = env.call(0x021D_775C, &mut [Arg::V(0x41)]);
    env.call(0x021D_7540, &mut [Arg::V(0x8D5)]);
    r
}

/// C: `sub_021bc2b0`: 0 when both teams (client index from `021b8bc4`) have a client with a usable
/// party (`021b8b9c` and `021b9b98` of its party), else 1.
pub fn some_team_unusable<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    let mut flags = [0u8; 2];
    let main = w.u32_at(4);
    let pc = w.u32_at(8);
    for i in 0..4u32 {
        if env.call(0x021B_8B9C, &mut [Arg::V(main), Arg::V(i)]) == 0 {
            continue;
        }
        let party = env.call(0x021B_9AEC, &mut [Arg::V(pc), Arg::V(i)]);
        let team = env.call(0x021B_8BC4, &mut [Arg::V(main), Arg::V(i)]);
        if env.call(0x021B_9B98, &mut [Arg::V(party)]) != 0 {
            flags[team as usize] = 1;
        }
    }
    u32::from(flags[0] == 0 || flags[1] == 0)
}

/// C: `FUN_overlay_d_93__021bc310`: 1 when a client of the leading client's team has a usable party.
pub fn own_team_usable<E: FlowEnv>(env: &mut E, w: &SvFlowWork) -> u32 {
    let main = w.u32_at(4);
    let pc = w.u32_at(8);
    for i in 0..4u32 {
        if env.call(0x021B_8B9C, &mut [Arg::V(main), Arg::V(i)]) == 0 {
            continue;
        }
        let team = env.call(0x021B_8BC4, &mut [Arg::V(main), Arg::V(i)]);
        let lead = env.call(0x021B_8FF4, &mut [Arg::V(main)]);
        let team2 = env.call(0x021B_8BC4, &mut [Arg::V(main), Arg::V(lead)]);
        if team != team2 {
            continue;
        }
        let party = env.call(0x021B_9AEC, &mut [Arg::V(pc), Arg::V(i)]);
        if env.call(0x021B_9B98, &mut [Arg::V(party)]) != 0 {
            return 1;
        }
    }
    0
}

fn rd32(b: &[u8]) -> u32 {
    u32::from_le_bytes(b[..4].try_into().unwrap())
}

fn rec_ofs(i: usize) -> usize {
    OFS_RECORDS + REC_SIZE * i
}

/// Address of the scratch record the move helpers use (work + 0x840).
const OFS_SCRATCH: usize = 0x840;

/// C: `FUN_overlay_d_93__021bcaac`: first record from index `start` that is not done (`+0xD`),
/// whose action word (`+4`, `021d8454`) is of kind 1 and whose `021d8340` equals `id`. Work
/// offset of the record or `None`.
pub fn find_pending<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    waddr: u32,
    id: u32,
    start: u32,
) -> Option<usize> {
    let n = w.u8_at(OFS_REC_COUNT) as u32;
    let mut i = start;
    while i < n {
        let rec = rec_ofs(i as usize);
        let act = waddr.wrapping_add(rec as u32 + 4);
        if w.u8_at(rec + 0xD) == 0
            && env.call(0x021D_8454, &mut [Arg::V(act)]) == 1
            && id == env.call(0x021D_8340, &mut [Arg::V(act)])
        {
            return Some(rec);
        }
        i += 1;
    }
    None
}

/// C: `FUN_overlay_d_93__021bca50`: first record after the record at work offset `after` whose
/// BattleMon id (`021d5440`) is `id`.
pub fn next_record_by_mon<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    after: u32,
    id: u32,
) -> Option<usize> {
    let n = w.u8_at(OFS_REC_COUNT) as u32;
    let mut i = 0u32;
    if n != 0 {
        loop {
            let hit = rec_ofs(i as usize) as u32 == after;
            i += 1;
            if hit || i >= n {
                break;
            }
        }
    }
    if i < n {
        loop {
            let rec = rec_ofs(i as usize);
            let mon = w.u32_at(rec);
            if id == env.call(0x021D_5440, &mut [Arg::V(mon)]) {
                return Some(rec);
            }
            i += 1;
            if i >= w.u8_at(OFS_REC_COUNT) as u32 {
                break;
            }
        }
    }
    None
}

/// C: `FUN_overlay_d_93__021bcb14`: first pending record (see [`find_pending`], searching from
/// `kind_id`) whose mon passes `021b9a24(side, id)` and is not fainted (`021d5b68`).
pub fn find_pending_alive<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    waddr: u32,
    kind_id: u32,
    side: u32,
) -> Option<usize> {
    let mut rec = find_pending(env, w, waddr, kind_id, 0)?;
    loop {
        let mon = w.u32_at(rec);
        let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
        if env.call(0x021B_9A24, &mut [Arg::V(side), Arg::V(id)]) != 0
            && env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0
        {
            return Some(rec);
        }
        let idx = record_index(w, rec as u32) as u32;
        rec = find_pending(env, w, waddr, kind_id, idx.wrapping_add(1) & 0xFF)?;
    }
}

fn copy_rec(w: &mut SvFlowWork, dst: usize, src: usize) {
    w.mem.copy_within(src..src + REC_SIZE, dst);
}

/// C: `FUN_overlay_d_93__021bcb98`: moves the record at work offset `rec` (found at or after
/// index `from`) in front of the first done record at or after `from` (`+0xD` != 0 means done is
/// skipped: the first index with the flag clear); the records in between move up by one. Returns
/// the new index, or -1 when nothing moved.
pub fn move_record_up(w: &mut SvFlowWork, rec: u32, from: i32) -> i32 {
    let n = w.u8_at(OFS_REC_COUNT) as i32;
    let mut at = -1;
    if from < n {
        let mut i = from;
        loop {
            at = i;
            if rec_ofs(i as usize) as u32 == rec {
                break;
            }
            i += 1;
            at = -1;
            if i >= n {
                break;
            }
        }
    }
    let mut p = from;
    let first = loop {
        if p >= n {
            break -1;
        }
        if w.u8_at(rec_ofs(p as usize) + 0xD) == 0 {
            break p;
        }
        p += 1;
    };
    if at > -1 && first > -1 && first < at {
        copy_rec(w, OFS_SCRATCH, rec as usize);
        let mut j = at;
        while first < j {
            copy_rec(w, rec_ofs(j as usize), rec_ofs(j as usize - 1));
            j -= 1;
        }
        copy_rec(w, rec_ofs(first as usize), OFS_SCRATCH);
        return first;
    }
    -1
}

/// C: `FUN_overlay_d_93__021bcc50`: moves the record at work offset `rec` to the end of the list
/// (the records behind it move down by one; the record goes through the scratch at +0x840).
pub fn move_record_last(w: &mut SvFlowWork, rec: u32) {
    let n = w.u8_at(OFS_REC_COUNT) as i32;
    let mut at = -1;
    if n != 0 {
        let mut i = 0;
        loop {
            at = i;
            if rec_ofs(i as usize) as u32 == rec {
                break;
            }
            i += 1;
            at = -1;
            if i >= n {
                break;
            }
        }
    }
    if at > -1 {
        copy_rec(w, OFS_SCRATCH, rec as usize);
        while at < w.u8_at(OFS_REC_COUNT) as i32 - 1 {
            copy_rec(w, rec_ofs(at as usize), rec_ofs(at as usize + 1));
            at += 1;
        }
        copy_rec(w, rec_ofs(at as usize), OFS_SCRATCH);
    }
}

/// C: `FUN_overlay_d_93__021bd148`: 1 when the record of mon `id` exists, is not done and could
/// be moved by [`move_record_up`] (index >= 0).
pub fn promote_record<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, id: u32) -> u32 {
    if let Some(rec) = record_by_mon(env, w, id) {
        if w.u8_at(rec + 0xD) == 0 && move_record_up(w, rec as u32, 0) > -1 {
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021bd170`: repeatedly moves the pending records of `id` (see
/// [`find_pending`]) forward with [`move_record_up`]; 1 when at least one moved.
pub fn promote_pending<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32, id: u32) -> u32 {
    let mut next = 0u32;
    let mut cur = find_pending(env, w, waddr, id, 0);
    let mut r = 0;
    while let Some(rec) = cur {
        let idx = move_record_up(w, rec as u32, next as i32);
        if idx < 0 {
            break;
        }
        next = (idx + 1) as u32;
        cur = find_pending(env, w, waddr, id, next & 0xFF);
        r = 1;
    }
    r
}

/// C: `FUN_overlay_d_93__021bd1b0`: moves the not-done record of mon `id` to the end of the list.
pub fn demote_record<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, id: u32) -> u32 {
    if let Some(rec) = record_by_mon(env, w, id) {
        if w.u8_at(rec + 0xD) == 0 {
            move_record_last(w, rec as u32);
            return 1;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021bd0c0` (the original does not null-check `021bca50`: a done kind-6 record with
/// no later record of the mon faults in the ROM, here 0 is returned; the spec keeps two or more records of
/// one mon so that never happens): replaces the action of the (not done) record of mon `id`: when
/// the record found is done and its action kind (`021d8454`) is 6, the next record of that mon
/// (see [`next_record_by_mon`], `side` is the id) is used; unless `021bde48` objects or the kind
/// is 6, the action word becomes `021d82fc(.., 021bd138(work + 0x1AB8, arg))`, the record is run
/// (`021bcce0`) and 1 is returned.
pub fn replace_action<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    id: u32,
    arg: u32,
) -> u32 {
    let Some(mut rec) = record_by_mon(env, w, id) else {
        return 0;
    };
    if w.u8_at(rec + 0xD) != 0 {
        let act = waddr.wrapping_add(rec as u32 + 4);
        if env.call(0x021D_8454, &mut [Arg::V(act)]) == 6 {
            match next_record_by_mon(env, w, rec as u32, id) {
                Some(r) => rec = r,
                None => return 0,
            }
        }
    }
    if w.u8_at(rec + 0xD) != 0 {
        return 0;
    }
    let act = waddr.wrapping_add(rec as u32 + 4);
    let mon = w.u32_at(rec);
    if env.call(0x021B_DE48, &mut [Arg::V(mon), Arg::V(act)]) != 0 {
        return 0;
    }
    if env.call(0x021D_8454, &mut [Arg::V(act)]) == 6 {
        return 0;
    }
    let v = env.call(0x021B_D138, &mut [Arg::P, Arg::V(arg)]);
    env.call(0x021D_82FC, &mut [Arg::V(act), Arg::V(v)]);
    env.call(0x021B_CCE0, &mut [Arg::P, Arg::P]);
    1
}

/// C: `FUN_overlay_d_93__021bc0e8`: for each of the first `n` records at `recs` (work offset;
/// `waddr + recs` is the address the original receives) recomputes the priority word (`+8`): bits 0..12
/// from [`speed_value`]; for a kind-1 action (`021d8454`), bits 16..21 from [`priority_bits`] (the
/// move id is bits 7.. of the action word when its kind nibble is 1) and for kinds 1 and 5 bits
/// 13..15 from `021c6350`; each part bracketed by the queue header word like [`event_1_bracketed`].
/// Then the records are sorted ([`sort_records`]).
pub fn refresh_priorities<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    recs: usize,
    n: u32,
) {
    for i in 0..n as usize {
        let rec = recs + i * REC_SIZE;
        let mon = w.u32_at(rec);
        let spd = speed_value(env, mon, 1);
        let pri = w.u32_at(rec + 8);
        w.set_u32_at(rec + 8, (spd & 0x1FFF) | (pri & 0xFFFF_E000));
        let act_addr = waddr.wrapping_add(rec as u32 + 4);
        let kind = env.call(0x021D_8454, &mut [Arg::V(act_addr)]);
        if kind == 1 {
            let act = w.u32_at(rec + 4);
            let mv = if act & 0xF == 1 {
                (act & 0x7F_FFFF) >> 7
            } else {
                0
            };
            let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
            let bits = priority_bits(env, mv, mon);
            let pri = w.u32_at(rec + 8);
            w.set_u32_at(
                rec + 8,
                (pri & 0x1FFF) | (pri & 0xE000) | (pri & 0x01C0_0000) | ((bits & 0x3F) << 16),
            );
            set_word(&mut w.mem[OFS_QHEAD..], head);
        }
        if kind == 1 || kind == 5 {
            let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
            let v = env.call(0x021C_6350, &mut [Arg::V(waddr), Arg::V(mon)]) & 0xFF;
            let pri = w.u32_at(rec + 8);
            w.set_u32_at(rec + 8, ((v & 7) << 13) | (pri & 0xFFFF_1FFF));
            set_word(&mut w.mem[OFS_QHEAD..], head);
        }
    }
    sort_records(env, &mut w.mem[recs..], n);
}

/// C: `FUN_overlay_d_93__021bc220`: for every client with a used slot (`s` = server-side object)
/// and every action word of its list in `buf` (counts at `+0x30`, words at `client * 12 + j * 4`) with
/// kind 3 and bit 10 clear, calls `021bd9d4`; then `021bda00` and `021c5200`.
pub fn queue_switch_ins<E: FlowEnv>(env: &mut E, s: &[u8], buf: &[u8]) {
    for i in 0..4u32 {
        if server_side::slot_checked(s, i).is_none() {
            continue;
        }
        let n = server_side::byte_0x30(buf, i as i32) as u32;
        for k in 0..n {
            let v = server_side::word_at(buf, i as i32, k as i32);
            if v & 0xF == 3 && v & 0x400 == 0 {
                env.call(
                    0x021B_D9D4,
                    &mut [
                        Arg::P,
                        Arg::V(i),
                        Arg::V((v & 0x7F) >> 4),
                        Arg::V((v & 0x3FF) >> 7),
                        Arg::V(1),
                    ],
                );
            }
        }
    }
    env.call(0x021B_DA00, &mut [Arg::P]);
    env.call(0x021C_5200, &mut [Arg::P]);
}

/// C: `FUN_overlay_d_93__021bc3b0`: in mode 1 (work + 0x18), pushes command 0x29 `(client, n)` for
/// every client with a used slot, `n` being 1 (or, in rule 2, 1 + `021bc408(work + 0x1AB8,
/// client, &tmp)`).
pub fn push_client_counts<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    s: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) {
    if w.u32_at(0x18) != 1 {
        return;
    }
    for i in 0..4u32 {
        if !server_side::slot_used_idx(s, i) {
            continue;
        }
        let n = if env.call(0x021B_8588, &mut [Arg::V(w.u32_at(4))]) == 2 {
            let mut out = [0u8; 4];
            let r = env.call(0x021B_C408, &mut [Arg::P, Arg::V(i), Arg::Out(&mut out)]);
            r.wrapping_add(1) & 0xFF
        } else {
            1
        };
        put_cmd(env, q, 0x29, &[i, n], scratch);
    }
}

/// C: `FUN_overlay_d_93__021bc360`: clears the queue header, then 1 when `021bd584` accepts the
/// first mon (`021b990c(pokecon, 021b8ff4(main), 0)`) of our client.
pub fn start_queue<E: FlowEnv>(env: &mut E, w: &SvFlowWork, q: &mut [u8]) -> u32 {
    let lead = env.call(0x021B_8FF4, &mut [Arg::V(w.u32_at(4))]);
    let mon = env.call(
        0x021B_990C,
        &mut [Arg::V(w.u32_at(8)), Arg::V(lead), Arg::V(0)],
    );
    q[..8].fill(0);
    u32::from(env.call(0x021B_D584, &mut [Arg::P, Arg::V(mon)]) != 0)
}

/// C: `FUN_overlay_d_93__021bc390`: clears the queue header, runs [`push_client_counts`]; 1 when the
/// queue got a command.
pub fn count_commands<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    s: &[u8],
    q: &mut [u8],
    scratch: &mut [u8],
) -> u32 {
    q[..8].fill(0);
    push_client_counts(env, w, s, q, scratch);
    u32::from(rd32(q) != 0)
}

/// C: `FUN_overlay_d_93__021bc418`: announces a switch (command 0x48 `(client, target)`) for a
/// target other than 0 and 1: gets the two mons of the pair (`021b9c50`), brings the first in
/// (`021c5108`), and for a living second mon runs the entry hooks (`021c5620` for 0x20 / 0x1e under
/// rule `021ef948(2)`, `021d8548`, `021dcfb4`); finally `021bc4cc` records it.
pub fn announce_switch<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    q: &mut [u8],
    scratch: &mut [u8],
    client: u32,
    target: u32,
) {
    if target == 1 || target == 0 {
        return;
    }
    let pc = w.u32_at(8);
    let party = env.call(0x021B_9AEC, &mut [Arg::V(pc), Arg::V(client)]);
    put_cmd(env, q, 0x48, &[client, target], scratch);
    let mut a = [0u8; 4];
    let mut b = [0u8; 4];
    env.call(
        0x021B_9C50,
        &mut [
            Arg::V(party),
            Arg::V(target),
            Arg::Out(&mut a),
            Arg::Out(&mut b),
        ],
    );
    let (mon_a, mon_b) = (rd32(&a), rd32(&b));
    if env.call(0x021D_5B68, &mut [Arg::V(mon_a)]) == 0 {
        env.call(0x021C_5108, &mut [Arg::P, Arg::V(mon_a), Arg::V(1)]);
    }
    if env.call(0x021D_5B68, &mut [Arg::V(mon_b)]) == 0 {
        if env.call(0x021E_F948, &mut [Arg::V(2)]) != 0 {
            for id in [0x20, 0x1E] {
                if env.call(0x021D_62C4, &mut [Arg::V(mon_b), Arg::V(id)]) != 0 {
                    env.call(
                        0x021C_5620,
                        &mut [Arg::P, Arg::V(mon_b), Arg::V(id), Arg::V(0)],
                    );
                }
            }
        }
        env.call(0x021D_8548, &mut [Arg::V(mon_b)]);
        env.call(0x021D_CFB4, &mut [Arg::V(mon_b)]);
    }
    env.call(
        0x021B_C4CC,
        &mut [
            Arg::P,
            Arg::V(target),
            Arg::V(client),
            Arg::V(mon_b),
            Arg::V(pc),
        ],
    );
}

/// Move id of an action word: bits 7.. when its kind nibble is 1, else 0.
fn action_move(act: u32) -> u32 {
    if act & 0xF == 1 {
        (act & 0x7F_FFFF) >> 7
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021bc568`: for each record with a move-use action, asks `021e02dc`
/// (mon, move, `BattleMon_Get(mon, 0xC)`) and on success runs `021c6394` and `021e0394`; bracketed
/// by the queue header word.
pub fn apply_move_hooks<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32) {
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    let mut i = 0;
    if w.u8_at(OFS_REC_COUNT) != 0 {
        loop {
            let rec = rec_ofs(i);
            let mv = action_move(w.u32_at(rec + 4));
            if mv != 0 {
                let mon = w.u32_at(rec);
                let v = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xC)]);
                if env.call(0x021E_02DC, &mut [Arg::V(mon), Arg::V(mv), Arg::V(v)]) != 0 {
                    env.call(0x021C_6394, &mut [Arg::V(waddr), Arg::V(mon), Arg::V(mv)]);
                    env.call(0x021E_0394, &mut [Arg::V(mon), Arg::V(mv)]);
                }
            }
            i += 1;
            if i >= w.u8_at(OFS_REC_COUNT) as usize {
                break;
            }
        }
    }
    set_word(&mut w.mem[OFS_QHEAD..], head);
}

/// C: `FUN_overlay_d_93__021bcfac`: whether the mon uses the item / holds its turn: 0 (no), 2, 3
/// or 4, from the player's remaining count (`021b85a0`) against the mon's value (`021d59b4(mon, 0xF)`)
/// and two `021d78ac` rolls.
pub fn item_use_result<E: FlowEnv>(env: &mut E, w: &SvFlowWork, mon: u32) -> u32 {
    let main = w.u32_at(4);
    if env.call(0x021B_86B8, &mut [Arg::V(main)]) > 1 {
        return 0;
    }
    let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
    let t = env.call(0x021B_8D08, &mut [Arg::V(id)]);
    if t != env.call(0x021B_8FF4, &mut [Arg::V(main)]) {
        return 0;
    }
    let pk = env.call(0x021D_5844, &mut [Arg::V(mon)]);
    let r = env.call(0x021B_871C, &mut [Arg::V(main)]);
    if env.call(0x0201_8480, &mut [Arg::V(pk), Arg::V(r)]) != 0 {
        return 0;
    }
    let lvl = env.call(0x021B_85A0, &mut [Arg::V(main)]);
    if lvl >= 8 {
        return 0;
    }
    let a = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xF)]) & 0xFFFF;
    let b = (10 * (lvl + 1)) & 0xFFFF;
    if a <= b {
        return 0;
    }
    if env.call(0x021D_78AC, &mut [Arg::V(a + b + 1)]) & 0xFFFF < b {
        return 0;
    }
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(2)]) != 0 {
        return 4;
    }
    let roll = env.call(0x021D_78AC, &mut [Arg::V(0x100)]) & 0xFFFF;
    let diff = a - b;
    if (roll as i32) < diff as i32 && event_0e(env, mon) == 0 {
        return 2;
    }
    if (roll.wrapping_sub(diff & 0xFFFF) & 0xFFFF) < diff {
        return 3;
    }
    4
}

/// C: `FUN_overlay_d_93__021bcce0`: executes the action record at work offset `rec` (once: sets
/// the done flag `+0xD`). The action kind is the low nibble of the action word (`+4`): 1 move (hooks
/// once per turn, `021bcfac` result to work + 0x1F78, `021bde98`), 2 item (`021cb730`, then
/// `021bd708`), 3 switch (`021bdc98`, `021bda00`), 4 (`021bd584`), 5 (`021bd3e8`), 6 (`021bc418`
/// in rule 3), 7 (`021c54bc`), 8 (state 4); states 4 / 5 go to work + 0x14. Returns the kind.
pub fn execute_record<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, waddr: u32, rec: usize) -> u32 {
    if w.u8_at(rec + 0xD) != 0 {
        return 0;
    }
    let mon = w.u32_at(rec);
    let act = w.u32_at(rec + 4);
    let kind = act & 0xF;
    let main = w.u32_at(4);
    env.call(0x021C_859C, &mut [Arg::P]);
    w.set_u8_at(rec + 0xD, 1);
    let skip = (w.u32_at(0x14) == 5 && kind != 4)
        || env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0
        || (kind != 6 && {
            let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
            env.call(0x021B_CEEC, &mut [Arg::P, Arg::V(id)]) == 0
        });
    if !skip {
        if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x21)]) != 0
            && (kind != 4 || env.call(0x021B_86E4, &mut [Arg::V(main)]) != 2)
        {
            if env.call(0x021D_69AC, &mut [Arg::V(mon), Arg::V(3)]) != 0 {
                env.call(
                    0x021C_0780,
                    &mut [Arg::P, Arg::V(mon), Arg::V(3), Arg::V(0)],
                );
            }
        } else {
            execute_kind(env, w, waddr, rec, mon, act);
        }
    }
    env.call(0x021C_85B8, &mut [Arg::P]);
    kind
}

fn execute_kind<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    rec: usize,
    mon: u32,
    act: u32,
) {
    let kind = act & 0xF;
    let main = w.u32_at(4);
    event_1_bracketed(env, w, mon, w.u32_at(rec + 4) & 0xF);
    env.call(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(0)]);
    match kind {
        1 => {
            if w.u8_at(0x1FE8) == 0 {
                apply_move_hooks(env, w, waddr);
                w.set_u8_at(0x1FE8, 1);
            }
            let v = item_use_result(env, w, mon);
            w.set_u32_at(0x1F78, v);
            let pri = w.u32_at(rec + 8);
            env.call(
                0x021B_DE98,
                &mut [
                    Arg::P,
                    Arg::V(mon),
                    Arg::In(&act.to_le_bytes()),
                    Arg::V(pri & 0x3F_FFFF),
                ],
            );
        }
        2 => {
            let r = env.call(
                0x021C_B730,
                &mut [
                    Arg::P,
                    Arg::V(mon),
                    Arg::V((act & 0x7F_FFFF) >> 7),
                    Arg::V((act & 0x7FFF_FFFF) >> 23),
                    Arg::V((act & 0x7F) >> 4),
                ],
            );
            if r == 1
                && env.call(
                    0x021B_D708,
                    &mut [Arg::P, Arg::V(mon), Arg::V(1), Arg::V(1)],
                ) != 0
            {
                w.set_u32_at(0x14, 5);
            }
        }
        3 => {
            env.call(
                0x021B_DC98,
                &mut [Arg::P, Arg::V(mon), Arg::V((act & 0x3FF) >> 7)],
            );
            env.call(0x021B_DA00, &mut [Arg::P]);
        }
        4 => {
            if env.call(0x021B_D584, &mut [Arg::P, Arg::V(mon)]) != 0 {
                w.set_u32_at(0x14, 5);
            }
        }
        5 => {
            env.call(0x021B_D3E8, &mut [Arg::P, Arg::V(mon)]);
        }
        6 => {
            if env.call(0x021B_8588, &mut [Arg::V(main)]) == 3 {
                let client = w.u8_at(rec + 0xC) as u32;
                env.call(
                    0x021B_C418,
                    &mut [Arg::P, Arg::V(client), Arg::V((act & 0x7F) >> 4)],
                );
            }
        }
        7 => {
            env.call(0x021C_54BC, &mut [Arg::P, Arg::V(mon)]);
        }
        8 => w.set_u32_at(0x14, 4),
        _ => {}
    }
    if kind == 1 || kind == 2 {
        env.call(0x021D_5FC0, &mut [Arg::V(mon), Arg::V(1)]);
        env.call(0x021C_60EC, &mut [Arg::P, Arg::V(mon), Arg::V(0)]);
    }
    let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
    event_2(env, mon, kind);
    set_word(&mut w.mem[OFS_QHEAD..], head);
}

/// C: `FUN_overlay_d_93__021bc610`: builds the action list at work offset `base` from the clients'
/// action words in `buf` (counts at `+0x30`, words at `client * 12 + j * 4`; `s` = server-side
/// object, used slots only): per word one record (`+0` mon from the slot's party via `021b9bf0`,
/// `+4` the word, `+0xC` client, `+0xD`/`+0xE` cleared), stopping a client's list at a word
/// `021d83e0` rejects; kind 3 uses the index in the word, kind 6 `021b9ab0` of it (and then the
/// final re-sort for the `021c6350` bits is skipped), others a running index. Then every
/// record's priority word is computed (kind 0 gets the constant 0x1002000, kind 8 none) and the
/// list is sorted. Returns the record count.
pub fn build_action_list<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    s: &[u8],
    buf: &[u8],
    base: usize,
) -> u32 {
    let main = w.u32_at(4);
    let mut kind6 = false;
    let mut total = 0u32;
    for client in 0..4u32 {
        let Some(slot) = server_side::slot_checked(s, client) else {
            continue;
        };
        let handle = rd32(&s[slot + 4..]);
        let n = server_side::byte_0x30(buf, client as i32) as u32;
        let mut idx = 0u32;
        let mut k = 0u32;
        if n == 0 {
            continue;
        }
        loop {
            let rec = base + total as usize * REC_SIZE;
            let act_addr = waddr.wrapping_add(rec as u32 + 4);
            let word = server_side::word_at(buf, client as i32, k as i32);
            w.set_u32_at(rec + 4, word);
            if env.call(0x021D_83E0, &mut [Arg::V(act_addr)]) != 0 {
                break;
            }
            let kind = env.call(0x021D_8454, &mut [Arg::V(act_addr)]) & 0xFF;
            let mon;
            if kind == 3 {
                mon = env.call(
                    0x021B_9BF0,
                    &mut [Arg::V(handle), Arg::V((word & 0x7F) >> 4)],
                );
                idx = (idx + 1) & 0xFF;
            } else if kind == 6 {
                idx = env.call(0x021B_9AB0, &mut [Arg::V((word & 0x7F) >> 4)]);
                mon = env.call(0x021B_9BF0, &mut [Arg::V(handle), Arg::V(idx)]);
                kind6 = true;
            } else {
                mon = env.call(0x021B_9BF0, &mut [Arg::V(handle), Arg::V(idx)]);
                idx = (idx + 1) & 0xFF;
            }
            w.set_u32_at(rec, mon);
            w.set_u8_at(rec + 0xC, client as u8);
            w.set_u8_at(rec + 0xD, 0);
            w.set_u8_at(rec + 0xE, 0);
            total = (total + 1) & 0xFF;
            k = (k + 1) & 0xFF;
            if k >= n {
                break;
            }
        }
    }
    let mut j = 0u32;
    while j < total {
        let rec = base + j as usize * REC_SIZE;
        let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
        let mon = w.u32_at(rec);
        let kind = w.u32_at(rec + 4) & 0xF;
        let side = match kind {
            0 => {
                w.set_u32_at(rec + 8, 0x0100_2000);
                j = (j + 1) & 0xFF;
                continue;
            }
            2 => 2,
            3 => 3,
            4 => {
                if env.call(0x021B_86B8, &mut [Arg::V(main)]) == 0 && w.u8_at(rec + 0xC) == 1 {
                    0
                } else {
                    4
                }
            }
            6 => 1,
            8 => {
                j = (j + 1) & 0xFF;
                continue;
            }
            _ => 0,
        };
        let pri_bits = if kind == 1 {
            priority_bits(env, action_move(w.u32_at(rec + 4)), mon)
        } else if kind == 5 || kind == 7 {
            7
        } else {
            0
        };
        let spd = speed_value(env, mon, 1);
        set_word(&mut w.mem[OFS_QHEAD..], head);
        w.set_u32_at(
            rec + 8,
            0x2000 | ((pri_bits & 0x3F) << 16) | ((side & 7) << 22) | (spd & 0x1FFF),
        );
        j = (j + 1) & 0xFF;
    }
    sort_records(env, &mut w.mem[base..], total);
    if !kind6 {
        for i in 0..total as usize {
            let rec = base + i * REC_SIZE;
            let kind = w.u32_at(rec + 4) & 0xF;
            if kind == 1 || kind == 5 {
                let head = advance_fields(&mut w.mem[OFS_QHEAD..]);
                let v = env.call(0x021C_6350, &mut [Arg::V(waddr), Arg::V(w.u32_at(rec))]) & 0xFF;
                let pri = w.u32_at(rec + 8);
                w.set_u32_at(rec + 8, ((v & 7) << 13) | (pri & 0xFFFF_1FFF));
                set_word(&mut w.mem[OFS_QHEAD..], head);
            }
        }
        sort_records(env, &mut w.mem[base..], total);
    }
    total
}

/// C: `FUN_overlay_d_93__021bbfa8`: executes the action records from index `start` on
/// ([`execute_record`]); before a record following a kind-6 one (unless it is kind 6 itself) the
/// priorities of the rest are refreshed (`021bdc2c`, [`refresh_priorities`]). After each record
/// `021c5200` and the party check ([`some_team_unusable`]) can stop the run (state 4 or 3, or the
/// state is already 6 / 1); returns the next index. When the list ran out in state 0, checks
/// `021c446c` and the parties (states 4 / 3), else `021bc0d8` / `021c8534` start a switch round
/// ([`collect_list`], `021c6158`, state 2). Returns the record count then.
pub fn run_actions<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    s: &mut [u8],
    start: u32,
) -> u32 {
    let mut prev = 0u32;
    let mut i = start;
    if i < w.u8_at(OFS_REC_COUNT) as u32 {
        loop {
            let rec = rec_ofs(i as usize);
            let kind = env.call(
                0x021D_8454,
                &mut [Arg::V(waddr.wrapping_add(rec as u32 + 4))],
            );
            if prev == 6 && kind != 6 {
                env.call(0x021B_DC2C, &mut [Arg::P]);
                let n = w.u8_at(OFS_REC_COUNT) as u32;
                refresh_priorities(env, w, waddr, rec, n.wrapping_sub(i));
            }
            prev = execute_record(env, w, waddr, rec);
            let c5200 = env.call(0x021C_5200, &mut [Arg::P]) & 0xFF;
            if some_team_unusable(env, w) & 0xFF != 0 {
                w.set_u32_at(0x14, 4);
                return i + 1;
            }
            let st = w.u32_at(0x14);
            if st == 6 || st == 1 {
                return i + 1;
            }
            if c5200 != 0 {
                w.set_u32_at(0x14, 3);
                return i + 1;
            }
            i += 1;
            if i >= w.u8_at(OFS_REC_COUNT) as u32 {
                break;
            }
        }
    }
    if w.u32_at(0x14) == 0 {
        let c446c = env.call(0x021C_446C, &mut [Arg::P]) & 0xFF;
        if some_team_unusable(env, w) & 0xFF != 0 {
            w.set_u32_at(0x14, 4);
            return w.u8_at(OFS_REC_COUNT) as u32;
        }
        if c446c != 0 {
            w.set_u32_at(0x14, 3);
            return w.u8_at(OFS_REC_COUNT) as u32;
        }
        let r = env.call(0x021B_C0D8, &mut [Arg::P, Arg::V(0)]);
        if env.call(0x021C_8534, &mut [Arg::P]) != 0 || r != 0 {
            collect_list(env, w, s, 0x4CE);
            env.call(0x021C_6158, &mut [Arg::P, Arg::P]);
            w.set_u32_at(0x14, 2);
            return w.u8_at(OFS_REC_COUNT) as u32;
        }
        w.set_u32_at(0x14, 0);
    }
    w.u8_at(OFS_REC_COUNT) as u32
}

/// C: `FUN_overlay_d_93__021bbcd4`: walks the list `021bd2b4` / `021bd34c` hands out and runs
/// `021c6110(mon, 0xC)` for every mon that passes `021d5bdc(mon, 0xC)`.
pub fn run_mon_list<E: FlowEnv>(env: &mut E) {
    env.call(0x021B_D2B4, &mut [Arg::P, Arg::P]);
    let mut out = [0u8; 4];
    while env.call(0x021B_D34C, &mut [Arg::P, Arg::P, Arg::Out(&mut out)]) != 0 {
        let mon = rd32(&out);
        if env.call(0x021D_5BDC, &mut [Arg::V(mon), Arg::V(0xC)]) != 0 {
            env.call(0x021C_6110, &mut [Arg::P, Arg::V(mon), Arg::V(0xC)]);
        }
    }
}

/// C: `FUN_overlay_d_93__021bbc44`: turn start (state 0 handler): clears the state and the queue
/// header; the first time (work + 0x77C clear) resets the helpers, builds the action list
/// (`021bc610`, count to work + 0x782) and sets the flag; then runs the actions
/// (`021bbfa8` from the index at work + 0x783). Returns the state.
pub fn turn_start<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, s: &mut [u8], q: &mut [u8]) -> u32 {
    w.set_u32_at(0x14, 0);
    q[..8].fill(0);
    if w.u8_at(OFS_FLAG_77C) == 0 {
        env.call(0x021C_84E8, &mut [Arg::P]);
        env.call(0x021B_BCC4, &mut [Arg::P]);
        for i in 0..4 {
            w.set_u8_at(0x1FE8 + i, 0);
        }
        env.call(0x021D_6EE8, &mut []);
        server_side::list_clear(s);
        env.call(0x021D_74E4, &mut []);
        run_mon_list(env);
        let n = env.call(0x021B_C610, &mut [Arg::P, Arg::P, Arg::P, Arg::V(6)]);
        w.set_u8_at(OFS_REC_COUNT, n as u8);
        w.set_u8_at(OFS_FLAG_77C, 1);
    }
    let next = env.call(0x021B_BFA8, &mut [Arg::P, Arg::V(w.u8_at(0x783) as u32)]);
    w.set_u8_at(0x783, next as u8);
    w.u32_at(0x14)
}

/// C: `FUN_overlay_d_93__021bbd2c`: like [`turn_start`] for a resumed turn: the first time only
/// resets the list in `s` and the flags (work + 0x77C = 1, + 0x77D = 0) and asks `021bc220`
/// (returns 3 with state 3 when it says so); then runs the actions.
pub fn turn_resume<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, s: &mut [u8], q: &mut [u8]) -> u32 {
    w.set_u32_at(0x14, 0);
    q[..8].fill(0);
    if w.u8_at(OFS_FLAG_77C) == 0 {
        server_side::list_clear(s);
        env.call(0x021D_74E4, &mut []);
        w.set_u8_at(OFS_FLAG_77C + 1, 0);
        w.set_u8_at(OFS_FLAG_77C, 1);
        if env.call(0x021B_C220, &mut [Arg::P, Arg::P]) != 0 {
            w.set_u32_at(0x14, 3);
            return 3;
        }
    }
    let next = env.call(0x021B_BFA8, &mut [Arg::P, Arg::V(w.u8_at(0x783) as u32)]);
    w.set_u8_at(0x783, next as u8);
    w.u32_at(0x14)
}

/// C: `FUN_overlay_d_93__021bbd8c`: the action-list builder / executor state function. The first
/// time (work + 0x77C clear): clears the list in `s`, remembers `021bbf88(work + 0x3E0, 0)`,
/// builds the list (`021bc610`), plays the kind-3 records (switch-ins; living mon: `021bdc98`,
/// fainted one: `021bd9d4`, both mark the record done) and, in rule 3, executes the kind-6 records
/// ([`execute_record`]); sets the flag. Always then `021bda00`, `021c5200`, [`collect_list`] and
/// compares `021bbf98(work + 0x3E0, 0)` with the remembered value: equal runs `021c4614` and
/// returns 0, else 4 when [`some_team_unusable`] and otherwise `021c6158` and 2. The remembered
/// value is uninitialised stack in the original when the flag was already set (0 here).
pub fn build_and_run<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    s: &mut [u8],
    q: &mut [u8],
) -> u32 {
    let mut before = 0u32;
    w.set_u32_at(0x14, 0);
    q[..8].fill(0);
    if w.u8_at(OFS_FLAG_77C) == 0 {
        server_side::list_clear(s);
        env.call(0x021D_74E4, &mut []);
        before = env.call(0x021B_BF88, &mut [Arg::P, Arg::V(0)]);
        let n = env.call(0x021B_C610, &mut [Arg::P, Arg::P, Arg::P, Arg::V(6)]);
        w.set_u8_at(OFS_REC_COUNT, n as u8);
        w.set_u8_at(OFS_FLAG_77C + 7, 0);
        if w.u8_at(OFS_REC_COUNT) != 0 {
            let mut i = 0;
            loop {
                let rec = rec_ofs(i);
                let act = w.u32_at(rec + 4);
                if act & 0xF == 3 && act & 0x400 == 0 {
                    let mon = w.u32_at(rec);
                    if env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0 {
                        env.call(
                            0x021B_DC98,
                            &mut [Arg::P, Arg::V(mon), Arg::V((act & 0x3FF) >> 7)],
                        );
                        w.set_u8_at(rec + 0xD, 1);
                    }
                }
                i += 1;
                if i >= w.u8_at(OFS_REC_COUNT) as usize {
                    break;
                }
            }
        }
        if w.u8_at(OFS_REC_COUNT) != 0 {
            let mut i = 0;
            loop {
                let rec = rec_ofs(i);
                let act = w.u32_at(rec + 4);
                if act & 0xF == 3 && act & 0x400 == 0 {
                    let mon = w.u32_at(rec);
                    if env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0 {
                        let client = w.u8_at(rec + 0xC) as u32;
                        env.call(
                            0x021B_D9D4,
                            &mut [
                                Arg::P,
                                Arg::V(client),
                                Arg::V((act & 0x7F) >> 4),
                                Arg::V((act & 0x3FF) >> 7),
                                Arg::V(1),
                            ],
                        );
                        w.set_u8_at(rec + 0xD, 1);
                    }
                }
                i += 1;
                if i >= w.u8_at(OFS_REC_COUNT) as usize {
                    break;
                }
            }
        }
        if env.call(0x021B_8588, &mut [Arg::V(w.u32_at(4))]) == 3 && w.u8_at(OFS_REC_COUNT) != 0 {
            let mut i = 0;
            loop {
                let rec = rec_ofs(i);
                let act = w.u32_at(rec + 4);
                if act & 0xF == 6 && act & 0x400 == 0 {
                    execute_record(env, w, waddr, rec);
                }
                i += 1;
                if i >= w.u8_at(OFS_REC_COUNT) as usize {
                    break;
                }
            }
        }
        w.set_u8_at(OFS_FLAG_77C, 1);
    }
    env.call(0x021B_DA00, &mut [Arg::P]);
    env.call(0x021C_5200, &mut [Arg::P]);
    collect_list(env, w, s, 0x4CE);
    let after = env.call(0x021B_BF98, &mut [Arg::P, Arg::V(0)]);
    if before == after {
        env.call(0x021C_4614, &mut [Arg::P]);
        return 0;
    }
    if some_team_unusable(env, w) != 0 {
        return 4;
    }
    env.call(0x021C_6158, &mut [Arg::P, Arg::P]);
    2
}

/// Address of the file name string passed to the heap allocator by `021bb968` (literal at 0x021BB9C4).
const WORK_FILE: u32 = 0x021F_3A70;

/// C: `FUN_overlay_d_93__021bb968`: creates the flow work: allocates 0x212C bytes (`02030734`
/// with the heap id, size, 1, file name and line 0x245), stores server `s_addr` (+0), main
/// module (+4), pokecon (+8), queue (+0xC), mode `mode` (+0x18) and the heap id (u16 at +0x780),
/// clears the record count (+0x782), +0x10 and +0x14, takes `0201b70c()` for +0x4A4 and runs the
/// init ([`init_work`], on the image `w`, which is where the allocation lands in the test
/// environment). Returns the work's address (`waddr` is where the test environment puts the image).
#[allow(clippy::too_many_arguments)]
pub fn create<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    waddr: u32,
    s_addr: u32,
    main: u32,
    pokecon: u32,
    queue: u32,
    mode: u32,
    heap: u16,
) -> u32 {
    let base = env.call(
        0x0203_0734,
        &mut [
            Arg::V(heap as u32),
            Arg::V(0x212C),
            Arg::V(1),
            Arg::V(WORK_FILE),
            Arg::V(0x245),
        ],
    );
    let o = base.wrapping_sub(waddr) as usize;
    w.set_u32_at(o, s_addr);
    w.set_u32_at(o + 8, pokecon);
    w.set_u32_at(o + 4, main);
    w.set_u8_at(o + OFS_REC_COUNT, 0);
    w.set_u32_at(o + 0x10, 0);
    w.set_u32_at(o + 0x14, 0);
    w.set_u32_at(o + 0xC, queue);
    w.set_u16_at(o + 0x780, heap);
    w.set_u32_at(o + 0x18, mode);
    let v = env.call(0x0201_B70C, &mut []);
    w.set_u32_at(o + 0x4A4, v);
    init_work(env, w);
    base
}

/// C: `FUN_overlay_d_93__021bb9d8`: initialises a fresh work: resets the event system and field
/// (`021d6eac`, `021ef874(0)`), the helper structs at +0x1C / +0x3E0 / +0x1AB8 (`021bbb08` /
/// `021bbb18` / `021bbb28(.., main, pokecon, 021b8588(main))`), clears four byte/word ranges
/// (+0x7A9 x 0x18, +0x7C1 x 0x18, +0x7D9 x 4, +0x4D4 x 0x2A0), runs `021c7c18`, `021c8588`,
/// `021d8230`, clears the queue header word area at +0x1D74 ([`events::queue_clear`]), runs
/// `021c5474` and `021bbb38`, then sets the state bytes (+0x784 = 6, +0x789 = 0x1F, flag bits of
/// +0x78A cleared, +0x77C/+0x77E/+0x786/+0x787 = 0, words at +0x774/+0x778 and the half at
/// +0x1F74 = 0).
pub fn init_work<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork) {
    env.call(0x021D_6EAC, &mut []);
    env.call(0x021E_F874, &mut [Arg::V(0)]);
    env.call(0x021B_BB08, &mut [Arg::P]);
    env.call(0x021B_BB18, &mut [Arg::P]);
    let main = w.u32_at(4);
    let rule = env.call(0x021B_8588, &mut [Arg::V(main)]);
    env.call(
        0x021B_BB28,
        &mut [Arg::P, Arg::V(main), Arg::V(w.u32_at(8)), Arg::V(rule)],
    );
    w.mem[0x7A9..0x7A9 + 0x18].fill(0);
    w.mem[0x7C1..0x7C1 + 0x18].fill(0);
    w.mem[0x7D9..0x7D9 + 4].fill(0);
    w.mem[0x4D4..0x4D4 + 0x2A0].fill(0);
    env.call(0x021C_7C18, &mut [Arg::P]);
    env.call(0x021C_8588, &mut [Arg::P]);
    env.call(0x021D_8230, &mut [Arg::P]);
    events::queue_clear(&mut w.mem[OFS_QHEAD..]);
    env.call(0x021C_5474, &mut [Arg::P]);
    env.call(0x021B_BB38, &mut []);
    w.set_u8_at(0x786, 0);
    w.set_u8_at(0x787, 0);
    w.set_u8_at(0x78A, w.u8_at(0x78A) & !1);
    w.set_u8_at(0x78A, w.u8_at(0x78A) & !2);
    w.set_u8_at(0x78A, w.u8_at(0x78A) & !4);
    w.set_u8_at(OFS_FLAG_77C, 0);
    w.set_u8_at(0x789, 0x1F);
    w.set_u16_at(0x1F74, 0);
    w.set_u8_at(0x784, 6);
    w.set_u32_at(0x774, 0);
    w.set_u8_at(0x77E, 0);
    w.set_u32_at(0x778, 0);
    w.set_u8_at(0x78A, w.u8_at(0x78A) & !0x40);
}

/// C: `FUN_overlay_d_93__021bbb48`: frees the work: `02049238(work[0x4A4])`, then the allocation.
pub fn destroy<E: FlowEnv>(env: &mut E, w: &SvFlowWork) {
    env.call(0x0204_9238, &mut [Arg::V(w.u32_at(0x4A4))]);
    env.call(0x0203_07B0, &mut [Arg::P]);
}

/// C: `FUN_overlay_d_93__021bbb60`: queues each client's party for the start of the battle:
/// clears the queue header; `1` when `021c3b68(work, b, 0xFF)` (b = the byte at +8 of the record
/// `021b8714(main)` returns, if non-zero) or `021bda00` say something was queued. For every used
/// client slot (in `s`), each living party member `021b9bf0(handle, k)` is announced
/// (`021bd8f0(work, client, k, k)`); in rule 3 (`021b8588`) the first three members also go through
/// `021b8750(main, client, mon)`.
pub fn queue_party_announce<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    waddr: u32,
    s: &[u8],
    q: &mut [u8],
) -> u32 {
    let main = w.u32_at(4);
    q[..8].fill(0);
    let mut r = 0;
    let rec = env.call(0x021B_8714, &mut [Arg::V(main)]);
    let b = w.u8_at(rec.wrapping_sub(waddr) as usize + 8) as u32;
    if b != 0 && env.call(0x021C_3B68, &mut [Arg::P, Arg::V(b), Arg::V(0xFF)]) != 0 {
        r = 1;
    }
    for i in 0..4u32 {
        let Some(slot) = server_side::slot_unchecked(s, i as i32) else {
            continue;
        };
        let handle = rd32(&s[slot + 4..]);
        let count = s[slot + 9] as u32;
        for k in 0..count {
            let mon = env.call(0x021B_9BF0, &mut [Arg::V(handle), Arg::V(k)]);
            if mon != 0 && env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0 {
                env.call(0x021B_D8F0, &mut [Arg::P, Arg::V(i), Arg::V(k), Arg::V(k)]);
            }
        }
        if env.call(0x021B_8588, &mut [Arg::V(main)]) == 3 {
            for k in 0..3u32 {
                let mon = env.call(0x021B_9BF0, &mut [Arg::V(handle), Arg::V(k)]);
                if mon != 0 && env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0 {
                    env.call(0x021B_8750, &mut [Arg::V(main), Arg::V(i), Arg::V(mon)]);
                }
            }
        }
    }
    if env.call(0x021B_DA00, &mut [Arg::P]) != 0 {
        r = 1;
    }
    r
}

#[cfg(test)]
mod tests;
