//! `height_ex.c` (overlay 21, `0x021C0598..0x021C06CC`) and the unanchored functions after it in F1 row 21.64
//! (`0x021C06CC..0x021C1024`). HEIGHT_EX (8 bytes, 021C05D4): `+0` record count, `+4` -> records of 0x1C bytes:
//! `+0` tile x, `+4` tile z, `+8` width, `+0xC` depth (tiles), `+0x10` height value, `+0x14` in use, `+0x18` extra;
//! fieldmap (0218DDxx) asks it for the height of a tile inside a rectangle. The rest of the range: GMEVENTs that
//! animate build models (started by script commands 021B2ACC / 021B2B14), a script-callback event (021C06CC), and the
//! boulder (strength) push: the step check 021C0CA4 that field_player 021A2F20 calls and its event 021C0E98.
//!
//! F3a-spec specced all 38 functions (`tools/rom-diff/specs/black-field/height_ex.spec`, header: layouts); this
//! module holds the smoke ports.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_0598,
    0x021C_0688,
    0x021C_068C,
    0x021C_06A8,
    0x021C_06B4,
];

/// Size of a HEIGHT_EX record.
pub const RECORD_SIZE: u32 = 0x1C;

fn record(mem: &Mem, hx: u32, i: u32) -> u32 {
    mem.r32(hx.wrapping_add(4))
        .wrapping_add(i.wrapping_mul(RECORD_SIZE))
}

/// C: `FUN_021C0598` (unlabeled routine at `0x021C0598`): record `rec` is in use (`+0x14 != 0`) and tile `(x, z)`
/// lies inside its rectangle `x0 <= x <= x0 + w - 1`, `z0 <= z <= z0 + d - 1` (signed compares, wrapping sums).
pub fn rect_contains(mem: &Mem, x: u32, z: u32, rec: u32) -> bool {
    if mem.r32(rec.wrapping_add(0x14)) == 0 {
        return false;
    }
    let x0 = mem.r32(rec) as i32;
    let z0 = mem.r32(rec.wrapping_add(4)) as i32;
    let x1 = x0
        .wrapping_add(mem.r32(rec.wrapping_add(8)) as i32)
        .wrapping_sub(1);
    let z1 = z0
        .wrapping_add(mem.r32(rec.wrapping_add(0xC)) as i32)
        .wrapping_sub(1);
    let (x, z) = (x as i32, z as i32);
    x0 <= x && x <= x1 && z0 <= z && z <= z1
}

/// C: `FUN_021C0688` (unlabeled routine at `0x021C0688`): the record count `+0`.
pub fn count(mem: &Mem, hx: u32) -> u32 {
    mem.r32(hx)
}

/// C: `FUN_021C068C` (unlabeled routine at `0x021C068C`): [`rect_contains`] for record `i` (no index check).
pub fn record_contains(mem: &Mem, x: u32, z: u32, hx: u32, i: u32) -> bool {
    rect_contains(mem, x, z, record(mem, hx, i))
}

/// C: `FUN_021C06A8` (unlabeled routine at `0x021C06A8`): the height value `+0x10` of record `i`.
pub fn height(mem: &Mem, i: u32, hx: u32) -> u32 {
    mem.r32(record(mem, hx, i).wrapping_add(0x10))
}

/// C: `FUN_021C06B4` (unlabeled routine at `0x021C06B4`): the extra value `+0x18` of record `i`.
pub fn extra(mem: &Mem, i: u32, hx: u32) -> u32 {
    mem.r32(record(mem, hx, i).wrapping_add(0x18))
}

#[cfg(test)]
mod tests;
