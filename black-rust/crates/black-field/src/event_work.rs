//! Event flags and vars (`field_status.c` region of the T1 unit `unk_02014128`, `0x02014388..0x02014488`).
//!
//! Layout of the EVENT WORK object (the record `GAMEDATA + 0x1AC` points to; see
//! `tools/rom-diff/specs/black-field/event_work.spec`):
//!
//! | range | content |
//! |---|---|
//! | `0x000..0x27C` | vars, `u16` each; var id `V >= 0x4000` is the halfword at `(V - 0x4000) * 2` |
//! | `0x27C..0x3E8` | flags, bit `id & 7` of byte `0x27C + (id >> 3)`; ids `1..=0xB5F` (2912 flags) |
//! | `0x3E8` | `u8` "temp flags were cleared" marker |
//!
//! Flag ids `0x4000..0x403F` live outside the object, in the 8-byte RAM table at `0x02146268` (system
//! flags that are not saved). Verified per function with `tools/check-fn.sh black-field event_work`.
//!
//! `field_status` (`0x14` bytes, `FUN_02014428` allocates it): `+0` u32, `+4` u8, `+5` u8, `+6` u8, `+8` u16,
//! `+0x11` u8, `+0x12` u16 (cleared by the allocator); the accessors are one load or store each.

use crate::hooks::{HeapHooks, FILE_FIELD_STATUS};
use std::ops::RangeInclusive;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_42D0,
    0x0201_42D8,
    0x0201_42E8,
    0x0201_4314,
    0x0201_433C,
    0x0201_4368,
    0x0201_4388,
    0x0201_43CC,
    0x0201_43D8,
    0x0201_43F0,
    0x0201_440C,
    0x0201_441C,
    0x0201_4428,
    0x0201_4444,
    0x0201_444C,
    0x0201_4450,
    0x0201_4454,
    0x0201_4458,
    0x0201_445C,
    0x0201_4460,
    0x0201_4464,
    0x0201_4468,
    0x0201_446C,
    0x0201_4470,
    0x0201_4474,
    0x0201_4478,
    0x0201_447C,
    0x0201_4484,
];

/// Size of the event work object (`FUN_020142D0` returns it; `FUN_020142D8` zeroes that many bytes).
pub const EVENT_WORK_SIZE: usize = 0x3EA;
/// Offset of the flag block.
pub const FLAGS_OFS: usize = 0x27C;
/// Bytes of flag block: `0x16C * 8 = 2912` flags (ids `0..0xB60`; id 0 is invalid).
pub const FLAG_BYTES: u32 = 0x16C;
/// Offset of the "temp flags cleared" marker byte (`FUN_020143F0` sets it, `FUN_0201440C` takes it).
pub const TEMP_MARKER_OFS: usize = 0x3E8;
/// Temp flag ids cleared by `FUN_020143F0`.
pub const TEMP_FLAGS: RangeInclusive<u32> = 0xAA0..=0xB5F;
/// First var id (`FUN_020143CC`).
pub const VAR_BASE: u32 = 0x4000;
/// Number of var halfwords before the flag block (`0x27C / 2`).
pub const VAR_COUNT: u32 = (FLAGS_OFS / 2) as u32;
/// Address of the 8-byte system flag table (flag ids `0x4000..0x403F`).
pub const SYS_FLAGS_ADDR: u32 = 0x0214_6268;
/// Size of the system flag table.
pub const SYS_FLAGS_SIZE: usize = 8;

/// Where the byte holding a flag lives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlagSlot {
    /// Byte offset in the event work object.
    Work(usize),
    /// Signed byte index from [`SYS_FLAGS_ADDR`]; `0..8` for the real table. The original does not
    /// check the lower end, so a flag id `>= 0x8000_4000` gives a negative index (it addresses memory
    /// below the table).
    Sys(i32),
}

/// C: `FUN_02014388` (flag byte): `None` for id 0 and for ids with no byte, else the slot.
pub fn flag_slot(id: u32) -> Option<FlagSlot> {
    if id == 0 {
        return None;
    }
    if id < 0x4000 {
        let byte = id >> 3;
        if byte >= FLAG_BYTES {
            return None;
        }
        return Some(FlagSlot::Work(FLAGS_OFS + byte as usize));
    }
    // (id - 0x4000) / 8 rounding towards zero, in 32-bit signed arithmetic
    let x = id.wrapping_sub(VAR_BASE) as i32;
    let idx = x.wrapping_add((((x >> 2) as u32) >> 29) as i32) >> 3;
    if idx >= SYS_FLAGS_SIZE as i32 {
        return None;
    }
    Some(FlagSlot::Sys(idx))
}

/// C: `FUN_02014388` as the original returns it: the address of the flag byte given the address of
/// the event work object, or 0.
pub fn flag_addr(ew_addr: u32, id: u32) -> u32 {
    match flag_slot(id) {
        None => 0,
        Some(FlagSlot::Work(ofs)) => ew_addr.wrapping_add(ofs as u32),
        Some(FlagSlot::Sys(idx)) => SYS_FLAGS_ADDR.wrapping_add(idx as u32),
    }
}

/// The bit of the flag byte (`id & 7`); the original computes it with signed rounding, which is the
/// same for every id below `0x8000_0000`.
fn bit(id: u32) -> u8 {
    (id & 7) as u8
}

fn byte<'a>(work: &'a [u8], sys: &'a [u8], slot: FlagSlot) -> Option<&'a u8> {
    match slot {
        FlagSlot::Work(ofs) => work.get(ofs),
        FlagSlot::Sys(idx) => usize::try_from(idx).ok().and_then(|i| sys.get(i)),
    }
}

fn byte_mut<'a>(work: &'a mut [u8], sys: &'a mut [u8], slot: FlagSlot) -> Option<&'a mut u8> {
    match slot {
        FlagSlot::Work(ofs) => work.get_mut(ofs),
        FlagSlot::Sys(idx) => usize::try_from(idx).ok().and_then(|i| sys.get_mut(i)),
    }
}

/// C: `FUN_020142E8`: is the flag set? `work` is the event work object, `sys` the system flag table
/// (8 bytes). An id without a byte is not set. UNVERIFIED: an id `>= 0x8000_4000` makes the original
/// read memory below the table; here it reads as not set.
pub fn check_flag(work: &[u8], sys: &[u8], id: u32) -> bool {
    flag_slot(id)
        .and_then(|s| byte(work, sys, s))
        .is_some_and(|b| b & (1 << bit(id)) != 0)
}

/// C: `FUN_02014314`: set the flag (no effect when the id has no byte).
pub fn set_flag(work: &mut [u8], sys: &mut [u8], id: u32) {
    if let Some(b) = flag_slot(id).and_then(|s| byte_mut(work, sys, s)) {
        *b |= 1 << bit(id);
    }
}

/// C: `FUN_0201433C`: clear the flag (no effect when the id has no byte).
pub fn clear_flag(work: &mut [u8], sys: &mut [u8], id: u32) {
    if let Some(b) = flag_slot(id).and_then(|s| byte_mut(work, sys, s)) {
        *b &= !(1 << bit(id));
    }
}

/// C: `FUN_020143CC`: address of var `id` in an event work object at `ew_addr` (no range check).
pub fn var_addr(ew_addr: u32, id: u32) -> u32 {
    ew_addr.wrapping_add(id.wrapping_sub(VAR_BASE).wrapping_shl(1))
}

/// Byte offset of var `id` in the event work object, or `None` if it is not one of the
/// [`VAR_COUNT`] vars (the original does not check; this is the safe form for callers).
pub fn var_offset(id: u32) -> Option<usize> {
    let n = id.checked_sub(VAR_BASE)?;
    (n < VAR_COUNT).then_some(n as usize * 2)
}

/// Value of var `id` (composed from [`var_addr`]'s offset; the original reads `*(u16 *)var_addr`).
pub fn var_get(work: &[u8], id: u32) -> Option<u16> {
    let o = var_offset(id)?;
    Some(u16::from_le_bytes([work[o], work[o + 1]]))
}

/// Set var `id`; returns false if `id` is not a var.
pub fn var_set(work: &mut [u8], id: u32, value: u16) -> bool {
    match var_offset(id) {
        Some(o) => {
            work[o..o + 2].copy_from_slice(&value.to_le_bytes());
            true
        }
        None => false,
    }
}

/// C: `FUN_020142D0`: the size of the event work object.
pub fn event_work_size() -> u32 {
    EVENT_WORK_SIZE as u32
}

/// C: `FUN_020142D8`: zero the whole event work object (`MI_CpuFill8(ew, 0, 0x3EA)`).
pub fn event_work_clear(work: &mut [u8]) {
    work[..EVENT_WORK_SIZE].fill(0);
}

/// C: `FUN_02014368`: clear the flags `lo..=hi` (signed compare; each id is truncated to 16 bits).
pub fn clear_flag_range(work: &mut [u8], sys: &mut [u8], lo: i32, hi: i32) {
    let mut id = i64::from(lo);
    while id <= i64::from(hi) {
        clear_flag(work, sys, id as u32 & 0xFFFF);
        id += 1;
    }
}

/// C: `FUN_020143D8`: zero the vars `first..end` (`MI_CpuFill8(var_addr(first), 0, (end - first) * 2)`).
/// UNVERIFIED: a range that leaves the object panics, the original overwrites whatever follows.
pub fn clear_vars(work: &mut [u8], first: u32, end: u32) {
    let o = first.wrapping_sub(VAR_BASE).wrapping_shl(1) as usize;
    let n = end.wrapping_sub(first).wrapping_shl(1) as usize;
    work[o..o + n].fill(0);
}

/// C: `FUN_020143F0`: clear the temp flags (`0xAA0..=0xB5F`) and set the marker byte.
pub fn clear_temp_flags(work: &mut [u8], sys: &mut [u8]) {
    clear_flag_range(work, sys, 0xAA0, 0xB5F);
    work[TEMP_MARKER_OFS] = 1;
}

/// C: `FUN_0201440C`: read and clear the marker byte.
pub fn take_temp_marker(work: &mut [u8]) -> u8 {
    std::mem::take(&mut work[TEMP_MARKER_OFS])
}

/// The first bytes of the ROM table at `0x0209E050` that `FUN_0201441C` indexes. Only the first four
/// bytes look like a table (`1, 3, 4, 2`); the rest is whatever follows it in the ROM.
const TABLE_0209E050: [u8; 0x41] = [
    0x01, 0x03, 0x04, 0x02, 0xFF, 0xFF, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00,
    0x09, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00,
    0x00,
];

/// C: `FUN_0201441C`: byte `i` of the ROM table at `0x0209E050`. UNVERIFIED: only `i <= 0x40` is embedded
/// (panics beyond; the original reads on into the ROM).
pub fn table_0209e050(i: u32) -> u8 {
    TABLE_0209E050[i as usize]
}

/// Size of the `field_status` record.
pub const FIELD_STATUS_SIZE: u32 = 0x14;

/// C: `FUN_02014428`: allocate a `field_status` record (`"field_status.c"`, line 0x33) and clear its
/// `u16` at `+0x12`; returns the block address.
pub fn field_status_new<E: HeapHooks>(env: &mut E, heap: u32) -> u32 {
    let b = env.heap_alloc(heap, FIELD_STATUS_SIZE, 1, FILE_FIELD_STATUS, 0x33);
    env.mem().w16(b.wrapping_add(0x12), 0);
    b
}

/// C: `FUN_02014444`: free a `field_status` record.
pub fn field_status_free<E: HeapHooks>(env: &mut E, fs: u32) {
    env.heap_free(fs);
}

fn w32(fs: &mut [u8], o: usize, v: u32) {
    fs[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn w16(fs: &mut [u8], o: usize, v: u16) {
    fs[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

fn r16(fs: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([fs[o], fs[o + 1]])
}

/// C: `FUN_0201444C`: the `u32` at `+0`.
pub fn fs_get_0(fs: &[u8]) -> u32 {
    u32::from_le_bytes(fs[0..4].try_into().unwrap())
}

/// C: `FUN_02014450`: set the `u32` at `+0`.
pub fn fs_set_0(fs: &mut [u8], v: u32) {
    w32(fs, 0, v);
}

/// C: `FUN_02014454`: set the byte at `+4`.
pub fn fs_set_4(fs: &mut [u8], v: u8) {
    fs[4] = v;
}

/// C: `FUN_02014458`: the byte at `+4`.
pub fn fs_get_4(fs: &[u8]) -> u8 {
    fs[4]
}

/// C: `FUN_0201445C`: set the byte at `+5`.
pub fn fs_set_5(fs: &mut [u8], v: u8) {
    fs[5] = v;
}

/// C: `FUN_02014460`: the byte at `+5`.
pub fn fs_get_5(fs: &[u8]) -> u8 {
    fs[5]
}

/// C: `FUN_02014464`: set the byte at `+6`.
pub fn fs_set_6(fs: &mut [u8], v: u8) {
    fs[6] = v;
}

/// C: `FUN_02014468`: the byte at `+6`.
pub fn fs_get_6(fs: &[u8]) -> u8 {
    fs[6]
}

/// C: `FUN_0201446C`: set the `u16` at `+8`.
pub fn fs_set_8(fs: &mut [u8], v: u16) {
    w16(fs, 8, v);
}

/// C: `FUN_02014470`: the `u16` at `+8`.
pub fn fs_get_8(fs: &[u8]) -> u16 {
    r16(fs, 8)
}

/// C: `FUN_02014474`: set the `u16` at `+0x12`.
pub fn fs_set_12(fs: &mut [u8], v: u16) {
    w16(fs, 0x12, v);
}

/// C: `FUN_02014478`: the `u16` at `+0x12`.
pub fn fs_get_12(fs: &[u8]) -> u16 {
    r16(fs, 0x12)
}

/// C: `FUN_0201447C`: set the byte at `+0x11`, returns the old value.
pub fn fs_swap_11(fs: &mut [u8], v: u8) -> u8 {
    std::mem::replace(&mut fs[0x11], v)
}

/// C: `FUN_02014484`: the byte at `+0x11`.
pub fn fs_get_11(fs: &[u8]) -> u8 {
    fs[0x11]
}

#[cfg(test)]
mod tests;
