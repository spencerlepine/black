//! `fieldmap_func.c` (overlay 21, `0x021BB890..0x021BBCB8`): the FIELDMAP FUNC system (slots of overlay-loaded
//! field functions run by two task systems) and the 30-entry pass-by ring after it.
//!
//! F2a-spec specced all 13 functions (`tools/rom-diff/specs/black-field/fieldmap_func.spec`, whose header
//! documents the system, slot, function-data and ring layouts); F2a ported them. Records are read through the
//! [`Mem`] window of the environment at their original addresses; heap, task-system, overlay and string
//! callees, and the slot callbacks of a FUNC DATA record, go through [`FieldEnv::call`].
//!
//! SYSTEM (0x18): `+0` fieldmap, `+4` heap id, `+8 s32` slot count, `+0xC` -> SLOT[count], `+0x10` / `+0x14` task
//! systems A (update) / B (draw). SLOT (0x18, free when `+4 == 0`): `+0` -> system, `+4` / `+8` task handles,
//! `+0xC` -> FUNC DATA, `+0x10` work block, `+0x14` overlay id (-1 = none). FUNC DATA: `+0 u32` task priority,
//! `+4 u16` work size, `+8` create, `+0xC` delete, `+0x10` update, `+0x14` draw (each `fn(slot, fieldmap, work)`,
//! nullable).

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_B890,
    0x021B_B90C,
    0x021B_B948,
    0x021B_B954,
    0x021B_B960,
    0x021B_B978,
    0x021B_B990,
    0x021B_BA1C,
    0x021B_BA6C,
    0x021B_BA70,
    0x021B_BBD8,
    0x021B_BC2C,
    0x021B_BC58,
];

/// Address of the string `"fieldmap_func.c"` (the file argument of the allocations).
pub const FILE_FIELDMAP_FUNC: u32 = 0x021D_E2F8;
/// Address of the assert message of `FUN_021BBA70`.
pub const ASSERT_PASSBY: u32 = 0x021D_E308;
/// Size of a SLOT.
pub const SLOT_SIZE: u32 = 0x18;
/// Offset of the pass-by ring in the object handed to `FUN_021BBA70`.
pub const RING_OFS: u32 = 0x30E0;
/// Size of one ring entry.
pub const RING_ENTRY: u32 = 0x24;
/// Entries in the ring.
pub const RING_LEN: u8 = 30;
/// Ring: `u8` index of the oldest entry.
pub const RING_OLDEST: u32 = 0x438;
/// Ring: `u8` index of the next entry to write.
pub const RING_NEXT: u32 = 0x439;
/// Ring: `s32` total number of entries ever written.
pub const RING_COUNT: u32 = 0x43C;
/// Ring: `s32` low mark (at most 29 below the count).
pub const RING_LOW: u32 = 0x440;
/// The empty id of a ring entry (ROM `0x021D9AE0`: six `0xFF` bytes).
pub const EMPTY_ID: [u8; 6] = [0xFF; 6];

/// Task callbacks the slots register (Thumb addresses).
const TASK_UPDATE: u32 = 0x021B_B961;
const TASK_DRAW: u32 = 0x021B_B979;

/// C: `FUN_02030CB0(n)`: size of a task system for `n` tasks.
const TASK_SYS_SIZE: u32 = 0x0203_0CB0;
/// C: `FUN_02030CBC(n, block)`: initialise a task system in `block`, returns it.
const TASK_SYS_INIT: u32 = 0x0203_0CBC;
/// C: `FUN_02030D28(sys)`: run the tasks of a task system.
const TASK_SYS_RUN: u32 = 0x0203_0D28;
/// C: `FUN_02030DA8(sys, callback, arg, priority)`: add a task, returns its handle.
const TASK_ADD: u32 = 0x0203_0DA8;
/// C: `FUN_02030EAC(task)`: delete a task.
const TASK_DEL: u32 = 0x0203_0EAC;
/// C: `FUN_02034AC4(id)`: load an overlay.
const OVERLAY_LOAD: u32 = 0x0203_4AC4;
/// C: `FUN_02034A5C(id)`: unload an overlay.
const OVERLAY_UNLOAD: u32 = 0x0203_4A5C;
/// C: `FUN_02034714(msg, 0, msg)`: assert failure (fatal).
const ASSERT: u32 = 0x0203_4714;

fn alloc<E: FieldEnv>(env: &mut E, heap: u32, size: u32, line: u32) -> u32 {
    env.call(
        HEAP_ALLOC,
        &mut [
            CArg::V(heap),
            CArg::V(size),
            CArg::V(1),
            CArg::V(FILE_FIELDMAP_FUNC),
            CArg::V(line),
        ],
    )
}

/// C: `FUN_021BB890` (unlabeled routine at `0x021BB890`): create the system for `count` slots: the record
/// (line 0x46), the slot array (`count * 0x18`, line 0x4A) and the two task systems (B at `+0x14`, line 0x56,
/// then A at `+0x10`, line 0x59; each block `FUN_02030CB0(count)` bytes, set up by `FUN_02030CBC`); returns it.
pub fn create<E: FieldEnv>(env: &mut E, fieldmap: u32, heap: u32, count: u32) -> u32 {
    let s = alloc(env, heap, 0x18, 0x46);
    env.w32(s, fieldmap);
    env.w32(s + 4, heap);
    env.w32(s + 8, count);
    let slots = alloc(env, heap, count.wrapping_mul(SLOT_SIZE), 0x4A);
    env.w32(s + 0xC, slots);
    let n = env.call(TASK_SYS_SIZE, &mut [CArg::V(count)]);
    let b = alloc(env, heap, n, 0x56);
    env.w32(s + 0x14, b);
    let b = env.call(TASK_SYS_INIT, &mut [CArg::V(count), CArg::V(b)]);
    env.w32(s + 0x14, b);
    let a = alloc(env, heap, n, 0x59);
    env.w32(s + 0x10, a);
    let a = env.call(TASK_SYS_INIT, &mut [CArg::V(count), CArg::V(a)]);
    env.w32(s + 0x10, a);
    s
}

/// C: `FUN_021BB90C` (unlabeled routine at `0x021BB90C`): delete every slot ([`slot_delete`]; count and array
/// re-read each step), then free the slot array, both task systems and the record.
pub fn delete<E: FieldEnv>(env: &mut E, s: u32) {
    let mut i: i32 = 0;
    while i < env.r32(s + 8) as i32 {
        let e = env
            .r32(s + 0xC)
            .wrapping_add((i as u32).wrapping_mul(SLOT_SIZE));
        slot_delete(env, e);
        i += 1;
    }
    for o in [0xC, 0x10, 0x14] {
        let b = env.r32(s + o);
        env.call(HEAP_FREE, &mut [CArg::V(b)]);
    }
    env.call(HEAP_FREE, &mut [CArg::V(s)]);
}

/// C: `FUN_021BB948` (unlabeled routine at `0x021BB948`): run task system A (the slots' update callbacks).
pub fn run_update<E: FieldEnv>(env: &mut E, s: u32) {
    let t = env.r32(s + 0x10);
    env.call(TASK_SYS_RUN, &mut [CArg::V(t)]);
}

/// C: `FUN_021BB954` (unlabeled routine at `0x021BB954`): run task system B (the slots' draw callbacks).
pub fn run_draw<E: FieldEnv>(env: &mut E, s: u32) {
    let t = env.r32(s + 0x14);
    env.call(TASK_SYS_RUN, &mut [CArg::V(t)]);
}

/// Calls the FUNC DATA callback at offset `ofs` of the slot `e` (`fn(slot, fieldmap, work)`), if set.
fn slot_callback<E: FieldEnv>(env: &mut E, e: u32, ofs: u32) {
    let data = env.r32(e + 0xC);
    let f = env.r32(data + ofs);
    if f != 0 {
        let sys = env.r32(e);
        let fieldmap = env.r32(sys);
        let work = env.r32(e + 0x10);
        env.call(f & !1, &mut [CArg::V(e), CArg::V(fieldmap), CArg::V(work)]);
    }
}

/// C: `FUN_021BB960` (unlabeled routine at `0x021BB960`, task A callback): the slot's update callback
/// (FUNC DATA `+0x10`), if any.
pub fn task_update<E: FieldEnv>(env: &mut E, _task: u32, e: u32) {
    slot_callback(env, e, 0x10);
}

/// C: `FUN_021BB978` (unlabeled routine at `0x021BB978`, task B callback): the slot's draw callback
/// (FUNC DATA `+0x14`), if any.
pub fn task_draw<E: FieldEnv>(env: &mut E, _task: u32, e: u32) {
    slot_callback(env, e, 0x14);
}

/// C: `FUN_021BB990` (unlabeled routine at `0x021BB990`): start the field function `data` in the first free
/// slot of `s` (the count is read once): load `overlay` unless it is -1, register the two tasks (priority
/// `*data`), allocate the work block of `u16 data+4` bytes (line 0xC3, heap `(u16) s+4`) when nonzero, call
/// the create callback; returns the slot, or 0 when all are in use.
pub fn start<E: FieldEnv>(env: &mut E, overlay: u32, s: u32, data: u32) -> u32 {
    let n = env.r32(s + 8) as i32;
    let mut e = env.r32(s + 0xC);
    let mut i = 0i32;
    while i < n {
        if env.r32(e + 4) == 0 {
            if overlay != u32::MAX {
                env.call(OVERLAY_LOAD, &mut [CArg::V(overlay)]);
            }
            env.w32(e + 0x14, overlay);
            let (ta, prio) = (env.r32(s + 0x10), env.r32(data));
            let h = env.call(
                TASK_ADD,
                &mut [CArg::V(ta), CArg::V(TASK_UPDATE), CArg::V(e), CArg::V(prio)],
            );
            env.w32(e + 4, h);
            let (tb, prio) = (env.r32(s + 0x14), env.r32(data));
            let h = env.call(
                TASK_ADD,
                &mut [CArg::V(tb), CArg::V(TASK_DRAW), CArg::V(e), CArg::V(prio)],
            );
            let size = env.r16(data + 4) as u32;
            env.w32(e + 8, h);
            env.w32(e, s);
            env.w32(e + 0xC, data);
            if size != 0 {
                let heap = env.r32(s + 4) & 0xFFFF;
                let w = alloc(env, heap, size, 0xC3);
                env.w32(e + 0x10, w);
            }
            let f = env.r32(data + 8);
            if f != 0 {
                let fieldmap = env.r32(s);
                let work = env.r32(e + 0x10);
                env.call(f & !1, &mut [CArg::V(e), CArg::V(fieldmap), CArg::V(work)]);
            }
            return e;
        }
        i += 1;
        e = e.wrapping_add(SLOT_SIZE);
    }
    0
}

/// C: `FUN_021BBA1C` (unlabeled routine at `0x021BBA1C`): stop the field function of slot `e` if it is in use:
/// delete callback, free the work block (when the FUNC DATA has a work size), delete both tasks, unload the
/// overlay (unless -1), clear the slot.
pub fn slot_delete<E: FieldEnv>(env: &mut E, e: u32) {
    if env.r32(e + 4) == 0 {
        return;
    }
    slot_callback(env, e, 0xC);
    let data = env.r32(e + 0xC);
    if env.r16(data + 4) != 0 {
        let w = env.r32(e + 0x10);
        env.call(HEAP_FREE, &mut [CArg::V(w)]);
    }
    for o in [4, 8] {
        let t = env.r32(e + o);
        env.call(TASK_DEL, &mut [CArg::V(t)]);
    }
    let ov = env.r32(e + 0x14);
    if ov != u32::MAX {
        env.call(OVERLAY_UNLOAD, &mut [CArg::V(ov)]);
    }
    env.mem().fill(e, SLOT_SIZE, 0);
}

/// C: `FUN_021BBA6C` (unlabeled routine at `0x021BBA6C`): the slot's work block (`e + 0x10`).
pub fn slot_work(mem: &Mem, e: u32) -> u32 {
    mem.r32(e + 0x10)
}

/// C: `FUN_021BBA70` (unlabeled routine at `0x021BBA70`): record a pass-by in the ring at `obj + 0x30E0`, either
/// from a received record `rec` (skipped when [`ring_is_new`] says the id at `rec + 0x64` was already seen with
/// the key at `rec + 0x24`) or, when `rec` is null, the player's own entry at `pos` (`u16[4]`) with `flag`.
/// Fatal (assert) when both `rec` and `pos` are null. Then the count goes up and the low mark follows it
/// (`count - low` stays below 30; an empty ring moves the low mark too).
pub fn ring_add<E: FieldEnv>(env: &mut E, obj: u32, rec: u32, pos: u32, flag: u32) {
    let ring = obj.wrapping_add(RING_OFS);
    if rec == 0 && pos == 0 {
        env.call(
            ASSERT,
            &mut [CArg::V(ASSERT_PASSBY), CArg::V(0), CArg::V(ASSERT_PASSBY)],
        );
    }
    if rec != 0 {
        let key = env.r16(rec + 0x24) as u32;
        if !ring_is_new(env, ring, rec + 0x64, key) {
            return;
        }
    }
    let e = ring_take(env.mem(), ring);
    if rec == 0 {
        let o = env.r32(obj);
        let me = env.r32(o);
        // C: `FUN_020084F8(x, dst, 8)` (thunk to `FUN_0201F41C`): copy the 8-character name of `x` to `dst`
        env.call(
            0x0200_84F8,
            &mut [CArg::V(me), CArg::Out(e, 16), CArg::V(8)],
        );
        if pos == 0 && flag == 0 {
            env.call(
                ASSERT,
                &mut [CArg::V(ASSERT_PASSBY), CArg::V(0), CArg::V(ASSERT_PASSBY)],
            );
        }
        for k in 0..4 {
            let v = env.r16(pos + 2 * k);
            env.w16(e + 0x10 + 2 * k, v);
        }
        env.w8(e + 0x21, flag as u8);
        env.w16(e + 0x18, 0);
        // C: `FUN_02087C6C(dst)`: copy the player's own 6-byte id to `dst`
        env.call(0x0208_7C6C, &mut [CArg::Out(e + 0x1A, 6)]);
        // C: `FUN_02008550(x)`: byte `x + 0x1D`
        let b = env.call(0x0200_8550, &mut [CArg::V(me)]);
        env.w8(e + 0x20, b as u8);
        env.w8(e + 0x22, 0);
    } else {
        // C: `FUN_0201F41C(src, dst, 8)`: copy an 8-character string
        env.call(
            0x0201_F41C,
            &mut [CArg::V(rec + 0xC), CArg::Out(e, 16), CArg::V(8)],
        );
        for k in 0..4 {
            let v = env.r16(rec + 0x1C + 2 * k);
            env.w16(e + 0x10 + 2 * k, v);
        }
        let b7 = env.r8(rec + 7);
        env.w8(e + 0x21, b7 & 7);
        let key = env.r16(rec + 0x24);
        env.w16(e + 0x18, key);
        env.mem().copy(e + 0x1A, rec + 0x64, 6);
        let b7 = env.r8(rec + 7);
        env.w8(e + 0x20, (b7 >> 3) & 1);
        let o = env.r32(obj);
        let gs = env.r32(o + 0xC);
        // C: `FUN_020120F4(gamesys)`: the GAMEDATA
        let gd = env.call(0x0201_20F4, &mut [CArg::V(gs)]);
        // C: `FUN_0201283C(gd)`, `FUN_02012EBC(gd)` (GAMEDATA accessors)
        let x = env.call(0x0201_283C, &mut [CArg::V(gd)]);
        let save = env.call(0x0201_2EBC, &mut [CArg::V(gd)]);
        // C: `FUN_0200ED48(save)` (thunk: `FUN_020071CC(save, 0x44)`, save block 0x44)
        let blk = env.call(0x0200_ED48, &mut [CArg::V(save)]);
        // C: `FUN_0217F894(x, rec)`, `FUN_0200EDC4(blk, u32 rec+0x5C)`: 1 = known (meaning UNVERIFIED)
        let kind = if env.call(0x0217_F894, &mut [CArg::V(x), CArg::V(rec)]) == 1 {
            1
        } else {
            let id = env.r32(rec + 0x5C);
            if env.call(0x0200_EDC4, &mut [CArg::V(blk), CArg::V(id)]) == 1 {
                2
            } else {
                0
            }
        };
        env.w8(e + 0x22, kind);
    }
    let (count, low) = (ring + RING_COUNT, ring + RING_LOW);
    if env.r32(count) == env.r32(low) {
        let v = env.r32(low).wrapping_add(1);
        env.w32(low, v);
    }
    let c = env.r32(count).wrapping_add(1);
    env.w32(count, c);
    if (c.wrapping_sub(env.r32(low)) as i32) >= 30 {
        env.w32(low, c.wrapping_sub(29));
    }
}

/// C: `FUN_021BBBD8` (unlabeled routine at `0x021BBBD8`): take the next ring slot: when the ring has entries
/// (`count > 0`) and `next == oldest`, `oldest` moves on (wrapping at 30); `next` moves on the same way; returns
/// the address of the old `next` slot (`ring + slot * 0x24`). The indices are not range-checked.
pub fn ring_take(mem: &mut Mem, ring: u32) -> u32 {
    let (oldest, next) = (ring + RING_OLDEST, ring + RING_NEXT);
    if (mem.r32(ring + RING_COUNT) as i32) > 0 && mem.r8(next) == mem.r8(oldest) {
        let o = mem.r8(oldest).wrapping_add(1);
        mem.w8(oldest, if o >= RING_LEN { 0 } else { o });
    }
    let slot = mem.r8(next);
    let n = slot.wrapping_add(1);
    mem.w8(next, if n >= RING_LEN { 0 } else { n });
    ring.wrapping_add(slot as u32 * RING_ENTRY)
}

/// C: `FUN_021BBC2C` (unlabeled routine at `0x021BBC2C`): entry `i % 30` (unsigned) of the ring, or 0 when its
/// id is the empty id (six `0xFF`).
pub fn ring_entry(mem: &Mem, ring: u32, i: u32) -> u32 {
    let e = ring.wrapping_add((i % 30) * RING_ENTRY);
    if (0..6).all(|k| mem.r8(e + 0x1A + k) == EMPTY_ID[k as usize]) {
        0
    } else {
        e
    }
}

/// C: `FUN_021BBC58` (unlabeled routine at `0x021BBC58`): false when one of the last 30 entries (from the
/// count down, stopping at a negative index) has the 6-byte id at `id` and the key `key` (an entry with the id
/// and another key gives true at once); true otherwise.
pub fn ring_is_new<E: FieldEnv>(env: &mut E, ring: u32, id: u32, key: u32) -> bool {
    let count = ring + RING_COUNT;
    let mut i = env.r32(count) as i32;
    if i <= i.wrapping_sub(30) {
        return true;
    }
    loop {
        if i < 0 {
            return true;
        }
        let e = ring_entry(env.mem(), ring, i as u32);
        if e != 0 && (0..6).all(|k| env.r8(e + 0x1A + k) == env.r8(id + k)) {
            return env.r16(e + 0x18) as u32 != key;
        }
        i = i.wrapping_sub(1);
        if i <= (env.r32(count) as i32).wrapping_sub(30) {
            return true;
        }
    }
}

#[cfg(test)]
mod tests;
