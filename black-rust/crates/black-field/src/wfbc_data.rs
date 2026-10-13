//! The head of F1 row 10.23 (overlay 10, `0x02161D98..0x0216237C`): Black City / White Forest people data (a LIST
//! of two 10-record lists, called from overlay 20 `field_wfbc_data_local.c`) and a 0x48-byte COMM record whose
//! slot updates are skipped while communication (`FUN_0200FB24`) is active. WFBC is F1 scope X1 (decision
//! pending); the code is specced because it sits in the F6a row.
//!
//! F6a-spec specced all 40 functions (`tools/rom-diff/specs/black-field/wfbc_data.spec`, whose header documents
//! LIST, REC (0x18 bytes) and COMM); this module holds the smoke port. COMM: `+2 u16` flags (bit `i < 3` = slot
//! A `i` valid), slot A `i` = 8 bytes at `+8 + 8 i`.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0216_2254];

/// Size of the COMM record (`FUN_02161F54` / `FUN_02161F68` clear it).
pub const COMM_SIZE: u32 = 0x48;

/// C: `FUN_02162254` (unlabeled routine at `0x02162254`): when slot A `i` of the COMM record is valid (flag bit
/// `i` of `u16 +2`; a shift of 32 or more is 0, as Thumb `lsls` by register), copies its 8 bytes to `dst` and
/// returns `true`.
pub fn slot_a_get(mem: &mut Mem, comm: u32, i: u32, dst: u32) -> bool {
    let sh = i & 0xFF;
    let bit = if sh >= 32 { 0 } else { 1u32 << sh };
    if bit & mem.r16(comm.wrapping_add(2)) as u32 == 0 {
        return false;
    }
    mem.copy(dst, comm.wrapping_add(8).wrapping_add(i.wrapping_shl(3)), 8);
    true
}

#[cfg(test)]
mod tests;
