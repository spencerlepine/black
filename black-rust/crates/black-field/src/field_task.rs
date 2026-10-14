//! `field_task.c` + `field_task_manager.c` (F1 row 21.75, first part; overlay 21, `0x021C6C70..0x021C6F00`; F14).
//!
//! A FIELD TASK (0xC bytes) is a small per-frame job of the field: `+0` state (0 idle, 1 running, 2 done), `+4` its
//! work block (allocated with the task), `+8` its proc `int proc(work)` (1 = finished). The FIELD TASK MANAGER (0xC)
//! holds `+0 u8` slot count, `+4` -> task\[count\], `+8` -> wait\[count\]: a slot's task starts once the task it waits
//! for is done, and the manager drops finished tasks (freeing them). The concrete tasks (camera / player / object
//! animations) are in [`crate::field_tasks`].
//!
//! Heap and the task procs are reached through [`FieldEnv::call`] (a proc pointer is called by its address, Thumb bit
//! cleared); the objects are read and written through the [`Mem`](crate::hooks::Mem) window. Verified by
//! `tools/check-fn.sh black-field field_task --per-function` (spec `tools/rom-diff/specs/black-field/field_task.spec`).

use crate::hooks::{FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_6C70,
    0x021C_6CA8,
    0x021C_6CCC,
    0x021C_6CD0,
    0x021C_6CE0,
    0x021C_6CE4,
    0x021C_6CEC,
    0x021C_6CF4,
    0x021C_6D10,
    0x021C_6D5C,
    0x021C_6D90,
    0x021C_6DE0,
    0x021C_6E20,
    0x021C_6E48,
    0x021C_6E5C,
    0x021C_6E74,
    0x021C_6E94,
    0x021C_6EA8,
    0x021C_6EDC,
];

/// Address of the string `"field_task.c"` (the file argument of the task allocations).
pub const FILE_TASK: u32 = 0x021D_E4D4;
/// Address of the string `"field_task_manager.c"`.
pub const FILE_MANAGER: u32 = 0x021D_E4E4;

/// Task state: created, not started.
pub const IDLE: u32 = 0;
/// Task state: running (its proc is called every frame).
pub const RUNNING: u32 = 1;
/// Task state: finished.
pub const DONE: u32 = 2;

/// Word `i` of the array the pointer at `field` points to (a manager's task\[i\] / wait\[i\]).
fn slot<E: FieldEnv>(env: &mut E, field: u32, i: u32) -> u32 {
    let a = env.r32(field);
    env.r32(a.wrapping_add(4 * i))
}

/// C: `FUN_021C6C70(heap, size, proc)`: a new idle task with a `size`-byte work block and the proc `proc`.
pub fn task_new<E: FieldEnv>(env: &mut E, heap: u32, size: u32, proc_: u32) -> u32 {
    let t = env.callv(HEAP_ALLOC, &[heap, 0xC, 0, FILE_TASK, 0x3B]);
    env.w32(t, IDLE);
    let work = env.callv(HEAP_ALLOC, &[heap, size, 1, FILE_TASK, 0x3D]);
    env.w32(t + 4, work);
    env.w32(t + 8, proc_);
    t
}

/// C: `FUN_021C6CA8(task)`: free the work block, then the task.
pub fn task_free<E: FieldEnv>(env: &mut E, t: u32) {
    let work = env.r32(t + 4);
    env.callv(HEAP_FREE, &[work]);
    env.callv(HEAP_FREE, &[t]);
}

/// C: `FUN_021C6CCC(task)`: the work block.
pub fn task_work<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r32(t + 4)
}

/// C: `FUN_021C6CD0(task)`: 1 when the task is done.
pub fn task_is_done<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    (env.r32(t) == DONE) as u32
}

/// C: `FUN_021C6CE0(task)`: the state.
pub fn task_state<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r32(t)
}

/// C: `FUN_021C6CE4(task)` (also reached through the veneer `021C6CC4`): start the task.
pub fn task_start<E: FieldEnv>(env: &mut E, t: u32) {
    env.w32(t, RUNNING);
}

/// C: `FUN_021C6CEC(task)`: mark the task done.
pub fn task_end<E: FieldEnv>(env: &mut E, t: u32) {
    env.w32(t, DONE);
}

/// C: `FUN_021C6CF4(task)` (also reached through the veneer `021C6CBC`): one frame: a running task calls its proc
/// with its work block; the proc's 1 ends the task.
pub fn task_run<E: FieldEnv>(env: &mut E, t: u32) {
    if env.r32(t) != RUNNING {
        return;
    }
    let work = env.r32(t + 4);
    let proc_ = env.r32(t + 8);
    if env.callv(proc_ & !1, &[work]) == 1 {
        task_end(env, t);
    }
}

/// C: `FUN_021C6D10(count, heap)`: a manager with `count` empty slots.
pub fn mgr_new<E: FieldEnv>(env: &mut E, count: u32, heap: u32) -> u32 {
    let m = env.callv(HEAP_ALLOC, &[heap, 0xC, 0, FILE_MANAGER, 0x34]);
    env.w8(m, count as u8);
    let size = count.wrapping_shl(2);
    let tasks = env.callv(HEAP_ALLOC, &[heap, size, 1, FILE_MANAGER, 0x36]);
    env.w32(m + 4, tasks);
    let waits = env.callv(HEAP_ALLOC, &[heap, size, 1, FILE_MANAGER, 0x37]);
    env.w32(m + 8, waits);
    m
}

/// C: `FUN_021C6D5C(mgr)`: remove every slot (freeing its task), then free the arrays and the manager.
pub fn mgr_free<E: FieldEnv>(env: &mut E, m: u32) {
    let mut i = 0u32;
    while i < u32::from(env.r8(m)) {
        mgr_remove(env, m, i & 0xFF);
        i += 1;
    }
    let tasks = env.r32(m + 4);
    env.callv(HEAP_FREE, &[tasks]);
    let waits = env.r32(m + 8);
    env.callv(HEAP_FREE, &[waits]);
    env.callv(HEAP_FREE, &[m]);
}

/// C: `FUN_021C6D90(mgr)`: one frame: run every slot's task, then start the tasks whose wait task is done, then drop
/// the finished tasks.
pub fn mgr_run<E: FieldEnv>(env: &mut E, m: u32) {
    let mut i = 0u32;
    while i < u32::from(env.r8(m)) {
        mgr_run_one(env, m, i & 0xFF);
        i += 1;
    }
    let mut i = 0u32;
    while i < u32::from(env.r8(m)) {
        mgr_start_waiting(env, m, i & 0xFF);
        i += 1;
    }
    let mut i = 0u32;
    while i < u32::from(env.r8(m)) {
        mgr_cleanup(env, m, i & 0xFF);
        i += 1;
    }
}

/// C: `FUN_021C6DE0(mgr, task, wait)`: put `task` in the first free slot, waiting for `wait` (0: start it now); with
/// no free slot the task is freed.
pub fn mgr_add<E: FieldEnv>(env: &mut E, m: u32, task: u32, wait: u32) {
    let count = u32::from(env.r8(m));
    let tasks = env.r32(m + 4);
    for i in 0..count {
        if env.r32(tasks + 4 * i) == 0 {
            mgr_set(env, m, i & 0xFF, task, wait);
            if wait == 0 {
                task_start(env, task);
            }
            return;
        }
    }
    task_free(env, task);
}

/// C: `FUN_021C6E20(mgr)`: the number of occupied slots.
pub fn mgr_active<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let count = u32::from(env.r8(m));
    let tasks = env.r32(m + 4);
    let mut n = 0u32;
    for i in 0..count {
        if env.r32(tasks + 4 * i) != 0 {
            n = (n + 1) & 0xFF;
        }
    }
    n
}

/// C: `FUN_021C6E48(mgr)`: 1 when no slot is occupied.
pub fn mgr_is_empty<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    (mgr_active(env, m) == 0) as u32
}

/// C: `FUN_021C6E5C(mgr, i, task, wait)`: fill slot `i` when it is free.
pub fn mgr_set<E: FieldEnv>(env: &mut E, m: u32, i: u32, task: u32, wait: u32) {
    let tasks = env.r32(m + 4);
    if env.r32(tasks + 4 * i) == 0 {
        env.w32(tasks + 4 * i, task);
        let waits = env.r32(m + 8);
        env.w32(waits + 4 * i, wait);
    }
}

/// C: `FUN_021C6E74(mgr, i)`: free slot `i`'s task and clear the slot.
pub fn mgr_remove<E: FieldEnv>(env: &mut E, m: u32, i: u32) {
    let t = slot(env, m + 4, i);
    if t != 0 {
        task_free(env, t);
        let tasks = env.r32(m + 4);
        env.w32(tasks + 4 * i, 0);
        let waits = env.r32(m + 8);
        env.w32(waits + 4 * i, 0);
    }
}

/// C: `FUN_021C6E94(mgr, i)`: run slot `i`'s task for one frame.
pub fn mgr_run_one<E: FieldEnv>(env: &mut E, m: u32, i: u32) {
    let t = slot(env, m + 4, i);
    if t != 0 {
        task_run(env, t);
    }
}

/// C: `FUN_021C6EA8(mgr, i)`: start slot `i`'s task when it is idle and the task it waits for is done.
pub fn mgr_start_waiting<E: FieldEnv>(env: &mut E, m: u32, i: u32) {
    let t = slot(env, m + 4, i);
    if t == 0 {
        return;
    }
    let w = slot(env, m + 8, i);
    if w == 0 || task_state(env, t) != IDLE {
        return;
    }
    let w = slot(env, m + 8, i);
    if task_state(env, w) == DONE {
        let t = slot(env, m + 4, i);
        task_start(env, t);
    }
}

/// C: `FUN_021C6EDC(mgr, i)`: remove slot `i` when its task is done.
pub fn mgr_cleanup<E: FieldEnv>(env: &mut E, m: u32, i: u32) {
    let t = slot(env, m + 4, i);
    if t != 0 && task_is_done(env, t) == 1 {
        mgr_remove(env, m, i);
    }
}

#[cfg(test)]
mod tests;
