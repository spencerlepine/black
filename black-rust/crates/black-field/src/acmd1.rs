//! `fldmmdl_acmd.c`, part 1 (overlay 21, `0x02197D3C..0x02197F5C`; F1 row 21.09, F4b job 1): the acmd (animation command)
//! list task (a task that feeds a list of `{u16 code, u16 repeat}` entries, ended by code 0xFE, to one MMDL), the per-direction
//! code lookup, and the acmd driver that runs the step functions of the grid / rail step tables (ROM `0x021D5F58` / `0x021D624C`,
//! 189 codes each; the steps themselves are F4b jobs 2 and the rail file).
//!
//! F4b-spec1 specced all 14 functions (`tools/rom-diff/specs/black-field/acmd1.spec`, header: the acmd table layout, the step
//! contract and the list-task work); F4b-port1 ported them. Every callee outside the area (the fldmmdl.c accessors, the heap /
//! task system, the step functions of the acmd2 areas and the rail file) is reached through [`FieldEnv::call`] by address;
//! in-area callees are Rust calls.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_7D3C,
    0x0219_7DA0,
    0x0219_7DAC,
    0x0219_7DCC,
    0x0219_7DE8,
    0x0219_7DF4,
    0x0219_7E10,
    0x0219_7E28,
    0x0219_7E44,
    0x0219_7E78,
    0x0219_7E7C,
    0x0219_7EB4,
    0x0219_7EDC,
    0x0219_7F20,
];

mod steps;
pub(crate) use steps::{GRID_STEPS, RAIL_STEPS};

/// `FUN_0216D7C4`: the acmd code of an MMDL (`u16 +0x26`, 0xFF = none).
const ACMD_CODE: u32 = 0x0216_D7C4;
/// `FUN_0216D7D4`: the acmd seq of an MMDL (`u16 +0x28`).
const ACMD_SEQ: u32 = 0x0216_D7D4;
/// `FUN_0216D7C0`: acmd code = v.
const SET_ACMD_CODE: u32 = 0x0216_D7C0;
/// `FUN_0216D7C8`: acmd seq = v.
const SET_ACMD_SEQ: u32 = 0x0216_D7C8;
/// `FUN_0216D57C`: move bits & mask.
const MOVE_BITS_TEST: u32 = 0x0216_D57C;
/// `FUN_0216D570`: move bits &= !v.
const MOVE_BITS_CLEAR: u32 = 0x0216_D570;
/// `FUN_0216D55C`: status & mask.
const STATUS_TEST: u32 = 0x0216_D55C;
/// `FUN_0216D67C`: MMDL -> MMDLSYS.
const SYS_OF: u32 = 0x0216_D67C;
/// `FUN_0216D4F8`: MMDLSYS heap id.
const SYS_HEAP: u32 = 0x0216_D4F8;
/// `FUN_0216D4F4`: MMDLSYS task priority.
const SYS_PRIORITY: u32 = 0x0216_D4F4;
/// `FUN_0216D514`: MMDLSYS task system.
const SYS_TASKS: u32 = 0x0216_D514;
/// `FUN_0216D3CC`: 1 when the MMDL can take an acmd.
const ACMD_CAN_SET: u32 = 0x0216_D3CC;
/// `FUN_0216D410`: set an acmd (m, code).
const ACMD_SET: u32 = 0x0216_D410;
/// `FUN_0216D454`: 1 when the MMDL's acmd is done.
const ACMD_DONE_TEST: u32 = 0x0216_D454;
/// `FUN_0216D480`: end the MMDL's acmd if it is done (result unused here).
const ACMD_END: u32 = 0x0216_D480;
/// `FUN_02030DA8`: task add (task system, proc, work, priority).
const TASK_ADD: u32 = 0x0203_0DA8;
/// `FUN_02030EE8`: task -> work.
const TASK_WORK: u32 = 0x0203_0EE8;
/// `FUN_02030EAC`: task delete.
const TASK_DELETE: u32 = 0x0203_0EAC;

/// The acmd list task proc [`list_proc`] (`FUN_02197DCC`, Thumb bit set).
const LIST_PROC: u32 = 0x0219_7DCD;
/// Address of the string `"fldmmdl_acmd.c"` (the file argument of the allocation in [`list_task_new`]).
pub const FILE_FLDMMDL_ACMD: u32 = 0x021D_DBE0;
/// ROM `0x021D4A50`: the six seqs of the acmd list task (Thumb bit set), run by [`list_proc`].
pub const LIST_SEQS: [u32; 6] = [
    0x0219_7DE9,
    0x0219_7DF5,
    0x0219_7E11,
    0x0219_7E29,
    0x0219_7E45,
    0x0219_7E79,
];
/// MMDL status bit 0x2000: the MMDL moves on a rail (the rail step table).
const STATUS_RAIL: u32 = 0x2000;

/// Move bit 0x400: the acmd is done (the last step of every grid list sets it).
const ACMD_DONE: u32 = 0x400;
/// Acmd list entry code that ends the list.
const LIST_END: u16 = 0xFE;

/// ROM `0x021D5D6C`: the 37 acmd kinds that exist per direction (up, down, left, right), in table order (the list of
/// pointers to these 4-word tables is null-terminated; the comment is each table's address).
pub const DIR_CODES: [[u32; 4]; 37] = [
    [0x00, 0x01, 0x02, 0x03], // 021D554C
    [0x0C, 0x0D, 0x0E, 0x0F], // 021D551C
    [0x58, 0x59, 0x5A, 0x5B], // 021D565C
    [0x14, 0x15, 0x16, 0x17], // 021D55EC
    [0x10, 0x11, 0x12, 0x13], // 021D548C
    [0xA7, 0xA8, 0xA9, 0xAA], // 021D558C
    [0xA3, 0xA4, 0xA5, 0xA6], // 021D561C
    [0x04, 0x05, 0x06, 0x07], // 021D55CC
    [0x08, 0x09, 0x0A, 0x0B], // 021D54CC
    [0x18, 0x19, 0x1A, 0x1B], // 021D552C
    [0x1C, 0x1D, 0x1E, 0x1F], // 021D54DC
    [0x20, 0x21, 0x22, 0x23], // 021D556C
    [0x24, 0x25, 0x26, 0x27], // 021D568C
    [0x28, 0x29, 0x2A, 0x2B], // 021D569C
    [0x2C, 0x2D, 0x2E, 0x2F], // 021D557C
    [0x30, 0x31, 0x32, 0x33], // 021D563C
    [0x34, 0x35, 0x36, 0x37], // 021D56BC
    [0x38, 0x39, 0x3A, 0x3B], // 021D564C
    [0x4C, 0x4D, 0x4E, 0x4F], // 021D549C
    [0x50, 0x51, 0x52, 0x53], // 021D54FC
    [0x54, 0x55, 0x56, 0x57], // 021D54AC
    [0x5C, 0x5D, 0x5C, 0x5D], // 021D55BC
    [0x5E, 0x5F, 0x5E, 0x5F], // 021D562C
    [0x60, 0x61, 0x62, 0x63], // 021D54BC
    [0x69, 0x6A, 0x6B, 0x6C], // 021D567C
    [0x6D, 0x6E, 0x6F, 0x70], // 021D55DC
    [0x75, 0x76, 0x77, 0x78], // 021D560C
    [0x71, 0x72, 0x73, 0x74], // 021D54EC
    [0x79, 0x7A, 0x7B, 0x7C], // 021D555C
    [0x7D, 0x7E, 0x7F, 0x80], // 021D550C
    [0x81, 0x82, 0x83, 0x84], // 021D566C
    [0x85, 0x86, 0x87, 0x88], // 021D55FC
    [0x89, 0x8A, 0x8B, 0x8C], // 021D553C
    [0x8D, 0x8E, 0x8F, 0x90], // 021D55AC
    [0x91, 0x92, 0x93, 0x94], // 021D56CC
    [0x95, 0x96, 0x97, 0x98], // 021D559C
    [0x9B, 0x9C, 0x9D, 0x9E], // 021D56AC
];

/// C: `FUN_02197D3C` (unlabeled routine at `0x02197D3C`): start an acmd list on the MMDL `m`: the work (0x14) is
/// `Heap_AllocDebug(0x8000 | (heap & 0x7FFF), 0x14, 1, "fldmmdl_acmd.c", 277)` (heap of the MMDLSYS; bit 15 = from the back),
/// the task is added to the MMDLSYS task system with [`list_proc`] at the MMDLSYS priority - 2; work `+0xC` = `m`, `+0x10` =
/// `list` (the rest is left as allocated: seq 0 clears what it uses). Returns the task.
pub fn list_task_new<E: FieldEnv>(env: &mut E, m: u32, list: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    let heap = env.callv(SYS_HEAP, &[sys]);
    let heap = (0x8000 | (heap & 0x7FFF)) & 0xFFFF;
    let w = env.callv(HEAP_ALLOC, &[heap, 0x14, 1, FILE_FLDMMDL_ACMD, 277]);
    let sys2 = env.callv(SYS_OF, &[m]);
    let prio = env.callv(SYS_PRIORITY, &[sys2]).wrapping_sub(2);
    let tasks = env.callv(SYS_TASKS, &[sys]);
    let task = env.callv(TASK_ADD, &[tasks, LIST_PROC, w, prio]);
    env.w32(w.wrapping_add(0xC), m);
    env.w32(w.wrapping_add(0x10), list);
    task
}

/// C: `FUN_02197DA0` (unlabeled routine at `0x02197DA0`): the end flag (work `+4`) of an acmd list task: 1 once the list's 0xFE
/// terminator was reached.
pub fn list_ended<E: FieldEnv>(env: &mut E, task: u32) -> u32 {
    let w = env.callv(TASK_WORK, &[task]);
    env.r32(w.wrapping_add(4))
}

/// C: `FUN_02197DAC` (unlabeled routine at `0x02197DAC`): delete an acmd list task: end the MMDL's acmd if it is done
/// (`0216D480`), free the work, delete the task.
pub fn list_delete<E: FieldEnv>(env: &mut E, task: u32) {
    let w = env.callv(TASK_WORK, &[task]);
    let m = env.r32(w.wrapping_add(0xC));
    env.callv(ACMD_END, &[m]);
    env.callv(HEAP_FREE, &[w]);
    env.callv(TASK_DELETE, &[task]);
}

/// C: `FUN_02197DCC` (unlabeled routine at `0x02197DCC`): the acmd list task proc: run the seq ([`LIST_SEQS`]`[work+0]`, read
/// afresh each time) while it returns 1. UNVERIFIED: a seq above 5 panics (the original calls through the ROM word after the
/// table).
pub fn list_proc<E: FieldEnv>(env: &mut E, _task: u32, w: u32) {
    loop {
        let r = match env.r32(w) {
            0 => list_start(env.mem(), w),
            1 => list_wait_ready(env, w),
            2 => list_set(env, w),
            3 => list_wait_done(env, w),
            4 => list_next(env.mem(), w),
            5 => list_stop(w),
            s => panic!("acmd list seq {s} is past the ROM table"),
        };
        if r != 1 {
            return;
        }
    }
}

/// C: `FUN_02197DE8` (unlabeled routine at `0x02197DE8`): acmd list seq 0: repeat counter (`+8`) = 0, seq (`+0`) = 1;
/// returns 1 (run the next seq now).
pub fn list_start(mem: &mut Mem, w: u32) -> u32 {
    mem.w32(w.wrapping_add(8), 0);
    mem.w32(w, 1);
    1
}

/// C: `FUN_02197DF4` (unlabeled routine at `0x02197DF4`): acmd list seq 1: 0 while the MMDL cannot take an acmd (`0216D3CC`);
/// else seq 2, returns 1.
pub fn list_wait_ready<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let m = env.r32(w.wrapping_add(0xC));
    if env.callv(ACMD_CAN_SET, &[m]) == 0 {
        return 0;
    }
    env.w32(w, 2);
    1
}

/// C: `FUN_02197E10` (unlabeled routine at `0x02197E10`): acmd list seq 2: set the entry's acmd (`0216D410(m, u16 code)`); seq 3;
/// returns 0 (wait a frame).
pub fn list_set<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let e = env.r32(w.wrapping_add(0x10));
    let m = env.r32(w.wrapping_add(0xC));
    let code = u32::from(env.r16(e));
    env.callv(ACMD_SET, &[m, code]);
    env.w32(w, 3);
    0
}

/// C: `FUN_02197E28` (unlabeled routine at `0x02197E28`): acmd list seq 3: 0 until the MMDL's acmd is done (`0216D454`); then
/// seq 4, returns 1.
pub fn list_wait_done<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let m = env.r32(w.wrapping_add(0xC));
    if env.callv(ACMD_DONE_TEST, &[m]) == 0 {
        return 0;
    }
    env.w32(w, 4);
    1
}

/// C: `FUN_02197E44` (unlabeled routine at `0x02197E44`): acmd list seq 4: count the run; below the entry's repeat count
/// (signed compare) run the same code again (seq 1); else step to the next `{u16 code, u16 count}` entry: seq 0 for a code,
/// or, for the 0xFE terminator, end flag (`+4`) = 1 and seq 5 (returns 0).
pub fn list_next(mem: &mut Mem, w: u32) -> u32 {
    let n = mem.r32(w.wrapping_add(8)).wrapping_add(1);
    mem.w32(w.wrapping_add(8), n);
    let e = mem.r32(w.wrapping_add(0x10));
    if (n as i32) < i32::from(mem.r16(e.wrapping_add(2))) {
        mem.w32(w, 1);
        return 1;
    }
    let e = e.wrapping_add(4);
    mem.w32(w.wrapping_add(0x10), e);
    if mem.r16(e) != LIST_END {
        mem.w32(w, 0);
        return 1;
    }
    mem.w32(w.wrapping_add(4), 1);
    mem.w32(w, 5);
    0
}

/// C: `FUN_02197E78` (unlabeled routine at `0x02197E78`): acmd list seq 5 (ended): returns 0 (the task stays until
/// [`list_delete`]).
pub fn list_stop(_w: u32) -> u32 {
    0
}

/// C: `FUN_02197E7C` (unlabeled routine at `0x02197E7C`): the acmd code of the same kind as `code` for direction `dir`
/// (0 up, 1 down, 2 left, 3 right): the first [`DIR_CODES`] table that holds `code` gives its entry `dir` (as a u16); a code
/// of no table comes back unchanged. UNVERIFIED: `dir > 3` panics (the original reads the ROM words after the table).
pub fn code_for_dir(dir: u32, code: u32) -> u32 {
    for t in DIR_CODES.iter() {
        if t.contains(&code) {
            return t[dir as usize] & 0xFFFF;
        }
    }
    code
}

/// C: `FUN_02197EB4` (unlabeled routine at `0x02197EB4`): run the MMDL's acmd: while the code (`0216D7C4`) is not 0xFF, run the
/// step [`step`]`(m, code, seq)` (seq from `0216D7D4`, both read afresh each time); stop when a step returns 0.
pub fn run<E: FieldEnv>(env: &mut E, m: u32) {
    loop {
        let code = env.callv(ACMD_CODE, &[m]);
        if code == 0xFF {
            return;
        }
        let seq = env.callv(ACMD_SEQ, &[m]);
        if step(env, m, code, seq) == 0 {
            return;
        }
    }
}

/// C: `FUN_02197EDC` (unlabeled routine at `0x02197EDC`): one frame of an acmd that a movement-type proc drives (it sets the
/// code with `0216D438`): with a code set, [`run`] its steps and finish only once it is done (move bit 0x400, else 0);
/// finishing (also with no code, 0xFF) clears bit 0x400, sets code 0xFF and seq 0, and returns 1.
pub fn action<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if env.callv(ACMD_CODE, &[m]) != 0xFF {
        run(env, m);
        if env.callv(MOVE_BITS_TEST, &[m, ACMD_DONE]) == 0 {
            return 0;
        }
    }
    env.callv(MOVE_BITS_CLEAR, &[m, ACMD_DONE]);
    env.callv(SET_ACMD_CODE, &[m, 0xFF]);
    env.callv(SET_ACMD_SEQ, &[m, 0]);
    1
}

/// C: `FUN_02197F20` (unlabeled routine at `0x02197F20`): one acmd step: the step function `seq` of the code's list in
/// [`RAIL_STEPS`] (MMDL status bit 0x2000) or [`GRID_STEPS`], called with the MMDL (the original leaves the step address and the
/// list address in r1 / r2; no step reads them); returns its result (0 = stop for this frame, 1 = run the next step now). The
/// steps are reached through [`FieldEnv::call`] by address. UNVERIFIED: a code above 0xBC or a seq past its list panics (the
/// original reads the ROM words that follow).
pub fn step<E: FieldEnv>(env: &mut E, m: u32, code: u32, seq: u32) -> u32 {
    let table = if env.callv(STATUS_TEST, &[m, STATUS_RAIL]) != 0 {
        &RAIL_STEPS
    } else {
        &GRID_STEPS
    };
    let f = table[code as usize].1[seq as usize];
    env.call(f & !1, &mut [CArg::V(m)])
}

#[cfg(test)]
mod tests;
