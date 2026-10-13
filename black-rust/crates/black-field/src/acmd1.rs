//! `fldmmdl_acmd.c`, part 1 (overlay 21, `0x02197D3C..0x02197F5C`; F1 row 21.09, F4b job 1): the acmd (animation command)
//! list task (a task that feeds a list of `{u16 code, u16 repeat}` entries, ended by code 0xFE, to one MMDL), the per-direction
//! code lookup, and the acmd driver that runs the step functions of the grid / rail step tables (ROM `0x021D5F58` / `0x021D624C`,
//! 189 codes each; the steps themselves are F4b jobs 2 and the rail file).
//!
//! F4b-spec1 specced all 14 functions (`tools/rom-diff/specs/black-field/acmd1.spec`, header: the acmd table layout, the step
//! contract and the list-task work); this module holds the smoke ports. Every callee outside the area (the fldmmdl.c accessors,
//! the heap / task system, the step functions) is reached through [`FieldEnv::call`] by address; in-area callees that are not
//! ported yet too.

use crate::hooks::{FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0219_7DE8, 0x0219_7E44, 0x0219_7E7C, 0x0219_7EDC];

/// `FUN_0216D7C4`: the acmd code of an MMDL (`u16 +0x26`, 0xFF = none).
const ACMD_CODE: u32 = 0x0216_D7C4;
/// `FUN_0216D7C0`: acmd code = v.
const SET_ACMD_CODE: u32 = 0x0216_D7C0;
/// `FUN_0216D7C8`: acmd seq = v.
const SET_ACMD_SEQ: u32 = 0x0216_D7C8;
/// `FUN_0216D57C`: move bits & mask.
const MOVE_BITS_TEST: u32 = 0x0216_D57C;
/// `FUN_0216D570`: move bits &= !v.
const MOVE_BITS_CLEAR: u32 = 0x0216_D570;
/// `FUN_02197EB4` (this area): run the acmd steps of an MMDL.
const ACMD_RUN: u32 = 0x0219_7EB4;

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

/// C: `FUN_02197DE8` (unlabeled routine at `0x02197DE8`): acmd list seq 0: repeat counter (`+8`) = 0, seq (`+0`) = 1;
/// returns 1 (run the next seq now).
pub fn list_start(mem: &mut Mem, w: u32) -> u32 {
    mem.w32(w.wrapping_add(8), 0);
    mem.w32(w, 1);
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

/// C: `FUN_02197EDC` (unlabeled routine at `0x02197EDC`): one frame of an acmd that a movement-type proc drives (it sets the
/// code with `0216D438`): with a code set, run its steps (`02197EB4`) and finish only once it is done (move bit 0x400, else 0);
/// finishing (also with no code, 0xFF) clears bit 0x400, sets code 0xFF and seq 0, and returns 1.
pub fn action<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if env.callv(ACMD_CODE, &[m]) != 0xFF {
        env.callv(ACMD_RUN, &[m]);
        if env.callv(MOVE_BITS_TEST, &[m, ACMD_DONE]) == 0 {
            return 0;
        }
    }
    env.callv(MOVE_BITS_CLEAR, &[m, ACMD_DONE]);
    env.callv(SET_ACMD_CODE, &[m, 0xFF]);
    env.callv(SET_ACMD_SEQ, &[m, 0]);
    1
}

#[cfg(test)]
mod tests;
