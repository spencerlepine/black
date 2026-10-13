//! The head of F1 row 10.23 (overlay 10, `0x02161D98..0x0216237C`): Black City / White Forest people data (a LIST
//! of two 10-record lists, called from overlay 20 `field_wfbc_data_local.c`) and a 0x48-byte COMM record whose
//! slot updates are skipped while communication (`FUN_0200FB24`) is active. WFBC is F1 scope X1 (decision
//! pending); the code is ported because it sits in the F6a row and is logic.
//!
//! F6a-spec specced all 40 functions (`tools/rom-diff/specs/black-field/wfbc_data.spec`, whose header documents
//! LIST, REC (0x18 bytes) and COMM); F6a ported them. Callees outside the area (the end-of-string code, the
//! communication state, the own id, the overlay-20 people data) are reached through [`FieldEnv::call`] by address.
//!
//! LIST (0x1E8): `+0 u16` 0, `+2 u16` 2, list A `REC[10]` at `+8`, list B `REC[10]` at `+0xF8`. REC (0x18): `+0 u8`
//! kind (low nibble nonzero = in use; a high nibble needs an end-of-string in the name), `+1 u8` id (< 30), `+2 u8`
//! (<= 100), `+3 u8`, `+4 u16[8]` name. COMM (0x48): `+0 u16` own id, `+2 u16` flags (bit `i < 3` = slot A `i`
//! valid, 8 = `+0x20` valid, 0x10 = `+0x28` valid, bit `i + 5` = slot B `i` valid), `+4 u16`, slot A `i` = 8 bytes at
//! `+8 + 8 i` (`u32` id, `u16` list id, `u16` handler index into [`SLOT_HANDLERS`]), `+0x20` (8), `+0x28` (8: `u32`
//! = `+4`, `u16`, `u16`), slot B `i` = 8 bytes at `+0x30 + 8 i` (`u32 i`, `u16`, `u16`).

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_1D98,
    0x0216_1DC4,
    0x0216_1E4C,
    0x0216_1E60,
    0x0216_1E78,
    0x0216_1E84,
    0x0216_1ECC,
    0x0216_1EEC,
    0x0216_1F00,
    0x0216_1F14,
    0x0216_1F54,
    0x0216_1F68,
    0x0216_1F74,
    0x0216_1F90,
    0x0216_1FEC,
    0x0216_1FF0,
    0x0216_2008,
    0x0216_202C,
    0x0216_2048,
    0x0216_208C,
    0x0216_209C,
    0x0216_20A0,
    0x0216_20B0,
    0x0216_210C,
    0x0216_2120,
    0x0216_2198,
    0x0216_21C0,
    0x0216_21E8,
    0x0216_220C,
    0x0216_2230,
    0x0216_2254,
    0x0216_2278,
    0x0216_2298,
    0x0216_22AC,
    0x0216_22D8,
    0x0216_22F4,
    0x0216_230C,
    0x0216_2330,
    0x0216_2358,
    0x0216_2370,
];

/// Size of the COMM record (`FUN_02161F54` / `FUN_02161F68` clear it).
pub const COMM_SIZE: u32 = 0x48;
/// Size of a REC.
pub const REC_SIZE: u32 = 0x18;
/// Records per list.
pub const LIST_RECS: u32 = 10;
/// Offset of list A in the LIST.
pub const LIST_A: u32 = 8;
/// Offset of list B in the LIST.
pub const LIST_B: u32 = 0xF8;

/// ROM `0x0217144C`: the request kinds of [`comm_request`] (word 3, 0x8000, is the next ROM word; the spec's
/// index reaches it).
pub const REQUEST_KINDS: [u32; 4] = [1, 3, 4, 0x8000];
/// ROM `0x02172990`: the slot-A handlers of [`apply_slots`] ([`handler_0`], [`handler_1`], [`handler_2`]; Thumb).
pub const SLOT_HANDLERS: [u32; 3] = [0x0216_2199, 0x0216_21C1, 0x0216_21E9];

/// `FUN_02045A98()`: the end-of-string code (0xFFFF).
const EOS: u32 = 0x0204_5A98;
/// `FUN_0200FB24()`: communication active?
const COMM_ACTIVE: u32 = 0x0200_FB24;
/// `FUN_02039338()`: the own id.
const OWN_ID: u32 = 0x0203_9338;
/// `FUN_021867C4(id, x)` (overlay 20, X1): open the people data.
const PEOPLE_OPEN: u32 = 0x0218_67C4;
/// `FUN_02186804(handle)`: close it.
const PEOPLE_CLOSE: u32 = 0x0218_6804;
/// `FUN_0218682C(handle)`: its data.
const PEOPLE_DATA: u32 = 0x0218_682C;
/// `FUN_02186520(p, x, rec)` (overlay 20, X1).
const PEOPLE_STORE: u32 = 0x0218_6520;

/// `1 << n` as Thumb `lsls` by register: the low byte of `n` is the count, 32 or more gives 0.
fn bit(n: u32) -> u32 {
    let s = n & 0xFF;
    if s >= 32 {
        0
    } else {
        1 << s
    }
}

/// C: `FUN_02161D98` (unlabeled routine at `0x02161D98`): initialise a LIST: `+0 = 0`, `+2 = 2` (u16), all 20
/// records cleared ([`rec_clear`], list A and B record `i` in turn).
pub fn list_init(mem: &mut Mem, list: u32) {
    mem.w16(list, 0);
    mem.w16(list.wrapping_add(2), 2);
    for i in 0..LIST_RECS {
        let o = i * REC_SIZE;
        rec_clear(mem, list.wrapping_add(LIST_A + o));
        rec_clear(mem, list.wrapping_add(LIST_B + o));
    }
}

/// C: `FUN_02161DC4` (unlabeled routine at `0x02161DC4`): the first valid record ([`rec_valid`]) with id `id`
/// (`u8 +1`, against the whole register): for each `i < 10`, list A record `i` when `which` is 0 or 2, then list B
/// record `i` when `which - 1 < 2` (unsigned); 0 when none.
pub fn list_find<E: FieldEnv>(env: &mut E, list: u32, id: u32, which: u32) -> u32 {
    for i in 0..LIST_RECS {
        let o = i * REC_SIZE;
        if which == 0 || which == 2 {
            let r = list.wrapping_add(LIST_A + o);
            if rec_valid(env, r) && id == env.r8(r.wrapping_add(1)) as u32 {
                return r;
            }
        }
        if which.wrapping_sub(1) <= 1 {
            let r = list.wrapping_add(LIST_B + o);
            if rec_valid(env, r) && id == env.r8(r.wrapping_add(1)) as u32 {
                return r;
            }
        }
    }
    0
}

/// C: `FUN_02161E4C` (unlabeled routine at `0x02161E4C`): does [`list_find`] find a record?
pub fn list_has<E: FieldEnv>(env: &mut E, list: u32, id: u32, which: u32) -> bool {
    list_find(env, list, id, which) != 0
}

/// C: `FUN_02161E60` (unlabeled routine at `0x02161E60`): compact list A (`which == 0`) or list B (else)
/// ([`recs_compact`]).
pub fn list_compact<E: FieldEnv>(env: &mut E, list: u32, which: u32) {
    let o = if which == 0 { LIST_A } else { LIST_B };
    recs_compact(env, list.wrapping_add(o));
}

/// C: `FUN_02161E78` (unlabeled routine at `0x02161E78`): clear a record (`MI_CpuFill8(rec, 0, 0x18)`).
pub fn rec_clear(mem: &mut Mem, rec: u32) {
    mem.fill(rec, REC_SIZE, 0);
}

/// C: `FUN_02161E84` (unlabeled routine at `0x02161E84`): are the record's fields valid: id (`+1`) below 30, `+2` at
/// most 100, and when the high nibble of `+0` is nonzero one of the 8 name halfwords (`+4`) is the end-of-string
/// code (`FUN_02045A98()`, read first)?
pub fn rec_fields_valid<E: FieldEnv>(env: &mut E, rec: u32) -> bool {
    let eos = env.callv(EOS, &[]);
    if env.r8(rec.wrapping_add(1)) >= 30 {
        return false;
    }
    if env.r8(rec.wrapping_add(2)) > 100 {
        return false;
    }
    if env.r8(rec) >> 4 != 0 {
        let mut found = false;
        for i in 0..8 {
            if eos == env.r16(rec.wrapping_add(4 + 2 * i)) as u32 {
                found = true;
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// C: `FUN_02161ECC` (unlabeled routine at `0x02161ECC`): is the record in use (low nibble of `+0` nonzero) and
/// valid ([`rec_fields_valid`])?
pub fn rec_valid<E: FieldEnv>(env: &mut E, rec: u32) -> bool {
    if env.r8(rec) & 0xF == 0 {
        return false;
    }
    rec_fields_valid(env, rec)
}

/// C: `FUN_02161EEC` (unlabeled routine at `0x02161EEC`): 10 halfwords at `p` = 0xFFFF.
pub fn fill_none10(mem: &mut Mem, p: u32) {
    for i in 0..10 {
        mem.w16(p.wrapping_add(2 * i), 0xFFFF);
    }
}

/// C: `FUN_02161F00` (unlabeled routine at `0x02161F00`): the halfword pair at `p` = (0, 10).
pub fn pair_init(mem: &mut Mem, p: u32) {
    mem.fill(p, 4, 0);
    mem.w16(p.wrapping_add(2), 10);
}

/// C: `FUN_02161F14` (unlabeled routine at `0x02161F14`): when `r +2 == 1` (u16): the halfword pair at `out` =
/// `d +0x24`, `d +8` of the people data `d` of `FUN_021867C4(r +9 (u8), (v & 0x7FFF) | 0x8000)` (opened and closed
/// here); else (0, 10).
pub fn people_info<E: FieldEnv>(env: &mut E, out: u32, r: u32, v: u32) {
    if env.r16(r.wrapping_add(2)) != 1 {
        env.w16(out, 0);
        env.w16(out.wrapping_add(2), 10);
        return;
    }
    let id = env.r8(r.wrapping_add(9)) as u32;
    let h = env.callv(PEOPLE_OPEN, &[id, ((v & 0x7FFF) | 0x8000) & 0xFFFF]);
    let d = env.callv(PEOPLE_DATA, &[h]);
    let a = env.r16(d.wrapping_add(0x24));
    env.w16(out, a);
    let b = env.r16(d.wrapping_add(8));
    env.w16(out.wrapping_add(2), b);
    env.callv(PEOPLE_CLOSE, &[h]);
}

/// C: `FUN_02161F54` (unlabeled routine at `0x02161F54`): clear the COMM record; `+0` = the own id
/// (`FUN_02039338()`, u16).
pub fn comm_init<E: FieldEnv>(env: &mut E, comm: u32) {
    env.mem().fill(comm, COMM_SIZE, 0);
    let id = env.callv(OWN_ID, &[]);
    env.w16(comm, id as u16);
}

/// C: `FUN_02161F68` (unlabeled routine at `0x02161F68`): clear the COMM record.
pub fn comm_clear(mem: &mut Mem, comm: u32) {
    mem.fill(comm, COMM_SIZE, 0);
}

/// C: `FUN_02161F74` (unlabeled routine at `0x02161F74`): unless communication is active ([`comm_active`]),
/// [`apply_slots`].
pub fn comm_apply_slots<E: FieldEnv>(env: &mut E, comm: u32, list: u32, mask: u32) {
    if !comm_active(env) {
        apply_slots(env, comm, list, mask);
    }
}

/// C: `FUN_02161F90` (unlabeled routine at `0x02161F90`): move list B's record of `id` ([`list_find`]; the
/// original does not check for null) out to overlay 20: `+2` = byte `d +6` (`r +2 == 0`) or `d +4` of the people
/// data of `FUN_021867C4(id, y)`, `+3 = 7`; `FUN_02186520(r, x, rec)`; then clear it and compact list B.
#[allow(clippy::too_many_arguments)]
pub fn rec_store<E: FieldEnv>(env: &mut E, _a: u32, r: u32, list: u32, x: u32, id: u16, y: u16) {
    let rec = list_find(env, list, id as u32, 1);
    let h = env.callv(PEOPLE_OPEN, &[id as u32, y as u32]);
    let d = env.callv(PEOPLE_DATA, &[h]);
    let b = if env.r16(r.wrapping_add(2)) == 0 {
        env.r16(d.wrapping_add(6))
    } else {
        env.r16(d.wrapping_add(4))
    };
    env.w8(rec.wrapping_add(2), b as u8);
    env.w8(rec.wrapping_add(3), 7);
    env.callv(PEOPLE_CLOSE, &[h]);
    env.call(
        PEOPLE_STORE,
        &mut [CArg::V(r), CArg::V(x), CArg::In(rec, REC_SIZE)],
    );
    rec_clear(env.mem(), rec);
    list_compact(env, list, 1);
}

/// C: `FUN_02161FEC` (unlabeled routine at `0x02161FEC`): `+4 = v` (u16).
pub fn comm_set_4(mem: &mut Mem, comm: u32, v: u32) {
    mem.w16(comm.wrapping_add(4), v as u16);
}

/// C: `FUN_02161FF0` (unlabeled routine at `0x02161FF0`): unless communication is active, [`put_20`].
pub fn comm_put_20<E: FieldEnv>(env: &mut E, comm: u32, rec: u32) {
    if !comm_active(env) {
        put_20(env.mem(), comm, rec);
    }
}

/// C: `FUN_02162008` (unlabeled routine at `0x02162008`): unless communication is active, [`slot_a_set`].
pub fn comm_put_slot_a<E: FieldEnv>(env: &mut E, comm: u32, i: u32, src: u32) {
    if !comm_active(env) {
        slot_a_set(env.mem(), comm, i, src);
    }
}

/// C: `FUN_0216202C` (unlabeled routine at `0x0216202C`): the byte at `b` = the low byte of `u16 +4`; then
/// [`get_28`].
pub fn comm_get_28(mem: &mut Mem, comm: u32, out: u32, b: u32) -> bool {
    let v = mem.r16(comm.wrapping_add(4));
    mem.w8(b, v as u8);
    get_28(mem, comm, out)
}

/// C: `FUN_02162048` (unlabeled routine at `0x02162048`): unless communication is active: [`set_28`]`(comm, v,
/// i)`; while it is: with `t = REQUEST_KINDS[i]`, unless `t == 4`, [`put_20`] of the record (`u32` own id `+0`,
/// `u16 v`, `u16 t`).
pub fn comm_request<E: FieldEnv>(env: &mut E, comm: u32, v: u32, i: u32) {
    if !comm_active(env) {
        set_28(env.mem(), comm, v, i);
        return;
    }
    let t = REQUEST_KINDS[i as usize];
    if t == 4 {
        return;
    }
    let m = env.mem();
    let rec = m.push_local(8);
    let id = m.r16(comm) as u32;
    m.w32(rec, id);
    m.w16(rec + 4, v as u16);
    m.w16(rec + 6, t as u16);
    put_20(m, comm, rec);
    m.pop_local(rec);
}

/// C: `FUN_0216208C` (unlabeled routine at `0x0216208C`): is `+0x20` valid (flag 8)?
pub fn has_20(mem: &Mem, comm: u32) -> bool {
    mem.r16(comm.wrapping_add(2)) & 8 != 0
}

/// C: `FUN_0216209C` (unlabeled routine at `0x0216209C`): `u16 +0x26`.
pub fn get_26(mem: &Mem, comm: u32) -> u32 {
    mem.r16(comm.wrapping_add(0x26)) as u32
}

/// C: `FUN_021620A0` (unlabeled routine at `0x021620A0`): [`clear_28`] then [`clear_20`].
pub fn clear_20_28(mem: &mut Mem, comm: u32) {
    clear_28(mem, comm);
    clear_20(mem, comm);
}

/// C: `FUN_021620B0` (unlabeled routine at `0x021620B0`): compact the 10 records at `recs`: each invalid record
/// `i` ([`rec_valid`]) takes the first valid record after it (copied, then that one cleared).
pub fn recs_compact<E: FieldEnv>(env: &mut E, recs: u32) {
    for i in 0..LIST_RECS {
        let ri = recs.wrapping_add(i * REC_SIZE);
        if rec_valid(env, ri) {
            continue;
        }
        for j in i + 1..LIST_RECS {
            let rj = recs.wrapping_add(j * REC_SIZE);
            if rec_valid(env, rj) {
                env.mem().copy(ri, rj, REC_SIZE);
                rec_clear(env.mem(), rj);
                break;
            }
        }
    }
}

/// C: `FUN_0216210C` (unlabeled routine at `0x0216210C`): is communication active (`FUN_0200FB24() != 0`)?
pub fn comm_active<E: FieldEnv>(env: &mut E) -> bool {
    env.callv(COMM_ACTIVE, &[]) != 0
}

/// Calls the slot-A handler at `f` (an entry of [`SLOT_HANDLERS`]).
fn call_handler<E: FieldEnv>(env: &mut E, f: u32, comm: u32, list: u32, rec: u32, out: u32) -> u32 {
    match f & !1 {
        0x0216_2198 => handler_0(env, comm, list, rec, out),
        0x0216_21C0 => handler_1(env, comm, list, rec, out),
        0x0216_21E8 => handler_2(env, comm, list, rec, out),
        a => env.call(
            a,
            &mut [
                CArg::V(comm),
                CArg::V(list),
                CArg::In(rec, 8),
                CArg::Out(out, 8),
            ],
        ),
    }
}

/// C: `FUN_02162120` (unlabeled routine at `0x02162120`): for the three slots `i`: mask bit `i` clear:
/// [`slot_a_clear`]; else when [`slot_a_get`] copies the slot to a local record, its handler
/// (`SLOT_HANDLERS[rec +6]`) runs on it (`f(comm, list, &rec, &out)`), the slot is cleared, and when the handler
/// answers nonzero [`slot_b_set`]`(comm, i, out +4, out +6)`. UNVERIFIED: a handler index of 3 or more reads past
/// the ROM table in the original; the port panics.
pub fn apply_slots<E: FieldEnv>(env: &mut E, comm: u32, list: u32, mask: u32) {
    for i in 0..3u32 {
        if mask & bit(i) == 0 {
            slot_a_clear(env.mem(), comm, i & 0xFFFF);
            continue;
        }
        let rec = env.mem().push_local(8);
        let out = env.mem().push_local(8);
        if slot_a_get(env.mem(), comm, i & 0xFFFF, rec) {
            let h = env.r16(rec + 6) as usize;
            let r = call_handler(env, SLOT_HANDLERS[h], comm, list, rec, out);
            slot_a_clear(env.mem(), comm, i & 0xFFFF);
            if r != 0 {
                let a = env.r16(out + 4) as u32;
                let b = env.r16(out + 6) as u32;
                slot_b_set(env.mem(), comm, i & 0xFFFF, a, b);
            }
        }
        env.mem().pop_local(out);
        env.mem().pop_local(rec);
    }
}

/// C: `FUN_02162198` (unlabeled routine at `0x02162198`): slot handler 0: `out` = `in` (`u32 +0`, `u16 +4`), `out +6`
/// = 0 when list B has a record of id `in +4` ([`list_find`]) else 1; returns 1.
pub fn handler_0<E: FieldEnv>(env: &mut E, _comm: u32, list: u32, rec: u32, out: u32) -> u32 {
    let a = env.r32(rec);
    let id = env.r16(rec.wrapping_add(4));
    env.w32(out, a);
    env.w16(out.wrapping_add(4), id);
    let f = list_find(env, list, id as u32, 1);
    env.w16(out.wrapping_add(6), if f != 0 { 0 } else { 1 });
    1
}

/// C: `FUN_021621C0` (unlabeled routine at `0x021621C0`): slot handler 1: as [`handler_0`] with `out +6` = 2 (found)
/// or 3; returns 1.
pub fn handler_1<E: FieldEnv>(env: &mut E, _comm: u32, list: u32, rec: u32, out: u32) -> u32 {
    let a = env.r32(rec);
    let id = env.r16(rec.wrapping_add(4));
    env.w32(out, a);
    env.w16(out.wrapping_add(4), id);
    let f = list_find(env, list, id as u32, 1);
    env.w16(out.wrapping_add(6), if f != 0 { 2 } else { 3 });
    1
}

/// C: `FUN_021621E8` (unlabeled routine at `0x021621E8`): slot handler 2: delete list B's record of id `in +4`
/// (cleared, list B compacted); returns 0.
pub fn handler_2<E: FieldEnv>(env: &mut E, _comm: u32, list: u32, rec: u32, _out: u32) -> u32 {
    let id = env.r16(rec.wrapping_add(4)) as u32;
    let f = list_find(env, list, id, 1);
    if f != 0 {
        rec_clear(env.mem(), f);
        list_compact(env, list, 1);
    }
    0
}

/// C: `FUN_0216220C` (unlabeled routine at `0x0216220C`): when the record's `u32 +0` equals the own id (`u16 +0`),
/// copy its 8 bytes to `+0x20` and set flag 8.
pub fn put_20(mem: &mut Mem, comm: u32, rec: u32) {
    if mem.r32(rec) != mem.r16(comm) as u32 {
        return;
    }
    mem.copy(comm.wrapping_add(0x20), rec, 8);
    let f = mem.r16(comm.wrapping_add(2)) | 8;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_02162230` (unlabeled routine at `0x02162230`): copy 8 bytes from `src` to slot A `i` and set its flag bit.
pub fn slot_a_set(mem: &mut Mem, comm: u32, i: u32, src: u32) {
    mem.copy(comm.wrapping_add(8).wrapping_add(i.wrapping_shl(3)), src, 8);
    let f = (mem.r16(comm.wrapping_add(2)) as u32 | bit(i)) as u16;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_02162254` (unlabeled routine at `0x02162254`): when slot A `i` of the COMM record is valid (flag bit
/// `i` of `u16 +2`; a shift of 32 or more is 0, as Thumb `lsls` by register), copies its 8 bytes to `dst` and
/// returns `true`.
pub fn slot_a_get(mem: &mut Mem, comm: u32, i: u32, dst: u32) -> bool {
    if bit(i) & mem.r16(comm.wrapping_add(2)) as u32 == 0 {
        return false;
    }
    mem.copy(dst, comm.wrapping_add(8).wrapping_add(i.wrapping_shl(3)), 8);
    true
}

/// C: `FUN_02162278` (unlabeled routine at `0x02162278`): slot B `i` = (`u32 i`, `u16 a`, `u16 b`) and its flag bit
/// `i + 5` set.
pub fn slot_b_set(mem: &mut Mem, comm: u32, i: u32, a: u32, b: u32) {
    let s = comm.wrapping_add(i.wrapping_shl(3));
    mem.w32(s.wrapping_add(0x30), i);
    mem.w16(s.wrapping_add(0x34), a as u16);
    mem.w16(s.wrapping_add(0x36), b as u16);
    let f = (mem.r16(comm.wrapping_add(2)) as u32 | bit(i.wrapping_add(5))) as u16;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_02162298` (unlabeled routine at `0x02162298`): `+0x2E = b`, `+0x2C = a` (u16), `+0x28` = `u16 +4`
/// (as a word); flag 0x10 set.
pub fn set_28(mem: &mut Mem, comm: u32, a: u32, b: u32) {
    mem.w16(comm.wrapping_add(0x2E), b as u16);
    let v = mem.r16(comm.wrapping_add(4)) as u32;
    mem.w16(comm.wrapping_add(0x2C), a as u16);
    let f = mem.r16(comm.wrapping_add(2)) | 0x10;
    mem.w32(comm.wrapping_add(0x28), v);
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_021622AC` (unlabeled routine at `0x021622AC`): when slot B `i` is valid (flag bit `i + 5`), copies its 8
/// bytes to `dst` and returns `true`.
pub fn slot_b_get(mem: &mut Mem, comm: u32, i: u32, dst: u32) -> bool {
    if mem.r16(comm.wrapping_add(2)) as u32 & bit(i.wrapping_add(5)) == 0 {
        return false;
    }
    mem.copy(
        dst,
        comm.wrapping_add(0x30).wrapping_add(i.wrapping_shl(3)),
        8,
    );
    true
}

/// C: `FUN_021622D8` (unlabeled routine at `0x021622D8`): when `+0x28` is valid (flag 0x10), copies its 8 bytes to
/// `dst` and returns `true`.
pub fn get_28(mem: &mut Mem, comm: u32, dst: u32) -> bool {
    if mem.r16(comm.wrapping_add(2)) & 0x10 == 0 {
        return false;
    }
    mem.copy(dst, comm.wrapping_add(0x28), 8);
    true
}

/// C: `FUN_021622F4` (unlabeled routine at `0x021622F4`): clear `+0x20` (8 bytes) and flag 8.
pub fn clear_20(mem: &mut Mem, comm: u32) {
    mem.fill(comm.wrapping_add(0x20), 8, 0);
    let f = mem.r16(comm.wrapping_add(2)) & !8;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_0216230C` (unlabeled routine at `0x0216230C`): clear slot A `i` and its flag bit.
pub fn slot_a_clear(mem: &mut Mem, comm: u32, i: u32) {
    mem.fill(comm.wrapping_add(8).wrapping_add(i.wrapping_shl(3)), 8, 0);
    let f = (!bit(i) & mem.r16(comm.wrapping_add(2)) as u32) as u16;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_02162330` (unlabeled routine at `0x02162330`): clear slot B `i` and its flag bit `i + 5`.
pub fn slot_b_clear(mem: &mut Mem, comm: u32, i: u32) {
    mem.fill(
        comm.wrapping_add(0x30).wrapping_add(i.wrapping_shl(3)),
        8,
        0,
    );
    let f = (!bit(i.wrapping_add(5)) & mem.r16(comm.wrapping_add(2)) as u32) as u16;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_02162358` (unlabeled routine at `0x02162358`): clear `+0x28` (8 bytes) and flag 0x10.
pub fn clear_28(mem: &mut Mem, comm: u32) {
    mem.fill(comm.wrapping_add(0x28), 8, 0);
    let f = mem.r16(comm.wrapping_add(2)) & !0x10;
    mem.w16(comm.wrapping_add(2), f);
}

/// C: `FUN_02162370` (unlabeled routine at `0x02162370`): `v >= 7` (signed).
pub fn at_least_7(v: u32) -> bool {
    v as i32 >= 7
}

#[cfg(test)]
mod tests;
