//! `rail_attr.c` (overlay 21, `0x021BB088..0x021BB1F4`; F3b): the attribute grids of the rail lines.
//!
//! RAIL ATTR (0x10): `+0` the loaded attribute file (ARC 0x4F member), `+4` its size, `+8` the number of lines, `+0xC` ->
//! line\[count\]. The file is a u32 count followed by one record per line, `{u16 width, u16 length, u32 attr[width * length]}`.
//! A rail LOCATION (8 bytes) is `{u16 line, u8 kind, s16 x, u16 z}`; the attribute at a location of kind 1 is
//! `attr[(x + width / 2) + width * z]` when that cell is inside the grid, else `0xFFFFFFFF`.
//!
//! Heap and archive calls go through [`FieldEnv::call`]. Verified by `tools/check-fn.sh black-field rail_attr
//! --per-function` (spec `tools/rom-diff/specs/black-field/rail_attr.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_B088,
    0x021B_B0A4,
    0x021B_B0B4,
    0x021B_B11C,
    0x021B_B13C,
    0x021B_B188,
    0x021B_B1A0,
    0x021B_B1AC,
    0x021B_B1B8,
    0x021B_B1C4,
    0x021B_B1D0,
    0x021B_B1DC,
    0x021B_B1E8,
];

/// Address of the string `"rail_attr.c"`.
pub const FILE_RAIL_ATTR: u32 = 0x021D_E2C4;
/// `FUN_0204A298(arc, id, 0, heap, &size)`: load an archive member (allocated), returns its address.
const ARC_LOAD: u32 = 0x0204_A298;
/// The answer of [`attr_at`] outside the grid.
pub const NO_ATTR: u32 = 0xFFFF_FFFF;

/// C: `FUN_021BB088(heap)`: allocate a RAIL ATTR (0x10 bytes of heap `heap & 0xFFFF`; not initialised).
pub fn create<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    env.callv(HEAP_ALLOC, &[heap & 0xFFFF, 0x10, 1, FILE_RAIL_ATTR, 0x5C])
}

/// C: `FUN_021BB0A4(ra)`: unload, then free the RAIL ATTR.
pub fn delete<E: FieldEnv>(env: &mut E, ra: u32) {
    unload(env, ra);
    env.callv(HEAP_FREE, &[ra]);
}

/// C: `FUN_021BB0B4(ra, id, heap)`: load attribute file `id` (ARC 0x4F; its size goes to `ra + 4`), allocate the line
/// table and point each entry at its record.
pub fn load<E: FieldEnv>(env: &mut E, ra: u32, id: u32, heap: u32) {
    let f = env.call(
        ARC_LOAD,
        &mut [
            CArg::V(0x4F),
            CArg::V(id),
            CArg::V(0),
            CArg::V(heap & 0xFFFF),
            CArg::Out(ra + 4, 4),
        ],
    );
    env.w32(ra, f);
    let count = env.r32(f);
    env.w32(ra + 8, count);
    let table = env.callv(
        HEAP_ALLOC,
        &[
            heap & 0xFFFF,
            count.wrapping_shl(2),
            1,
            FILE_RAIL_ATTR,
            0x86,
        ],
    );
    env.w32(ra + 0xC, table);
    let mut off = 4u32;
    let mut i = 0u32;
    while i < env.r32(ra + 8) {
        let rec = env.r32(ra).wrapping_add(off);
        let t = env.r32(ra + 0xC);
        env.w32(t.wrapping_add(4 * i), rec);
        let t = env.r32(ra + 0xC);
        let e = env.r32(t.wrapping_add(4 * i));
        let w = u32::from(env.r16(e));
        let h = u32::from(env.r16(e + 2));
        off = off.wrapping_add(w.wrapping_mul(h).wrapping_shl(2).wrapping_add(4));
        i += 1;
    }
}

/// C: `FUN_021BB11C(ra)`: when a file is loaded, free the line table and the file and zero `+0`, `+8`, `+0xC`.
pub fn unload<E: FieldEnv>(env: &mut E, ra: u32) {
    if env.r32(ra) == 0 {
        return;
    }
    let t = env.r32(ra + 0xC);
    env.callv(HEAP_FREE, &[t]);
    let f = env.r32(ra);
    env.callv(HEAP_FREE, &[f]);
    env.w32(ra + 8, 0);
    env.w32(ra + 0xC, 0);
    env.w32(ra, 0);
}

/// C: `FUN_021BB13C(ra, loc)`: the attribute at the rail location `loc` ([`NO_ATTR`] outside the grid or for a location
/// that is not on a line, kind != 1).
pub fn attr_at<E: FieldEnv>(env: &mut E, ra: u32, loc: u32) -> u32 {
    if env.r8(loc + 2) != 1 {
        return NO_ATTR;
    }
    let line = u32::from(env.r16(loc));
    if env.r32(ra + 8) <= line {
        return NO_ATTR;
    }
    let t = env.r32(ra + 0xC);
    let e = env.r32(t.wrapping_add(4 * line));
    let w = i32::from(env.r16(e));
    let x = i32::from(env.r16(loc + 4) as i16).wrapping_add(w >> 1);
    let z = i32::from(env.r16(loc + 6));
    let h = i32::from(env.r16(e + 2));
    if w <= x || h <= z || x < 0 {
        return NO_ATTR;
    }
    let idx = x.wrapping_add(w.wrapping_mul(z)) as u32;
    env.r32(e.wrapping_add(4).wrapping_add(idx.wrapping_shl(2)))
}

/// C: `FUN_021BB188(attr)`: the attribute (low halfword) is 0xA0 or 0xA1.
pub fn is_a0_a1(attr: u32) -> u32 {
    ((attr.wrapping_add(0xFF60) & 0xFFFF) <= 1) as u32
}

/// C: `FUN_021BB1A0(attr)`: attr == 0xA3.
pub fn is_a3(attr: u32) -> u32 {
    (attr == 0xA3) as u32
}

/// C: `FUN_021BB1AC(attr)`: attr == 0xA5.
pub fn is_a5(attr: u32) -> u32 {
    (attr == 0xA5) as u32
}

/// C: `FUN_021BB1B8(attr)`: attr == 0xA4.
pub fn is_a4(attr: u32) -> u32 {
    (attr == 0xA4) as u32
}

/// C: `FUN_021BB1C4(attr)`: attr == 0xA7.
pub fn is_a7(attr: u32) -> u32 {
    (attr == 0xA7) as u32
}

/// C: `FUN_021BB1D0(attr)`: attr == 0xA6.
pub fn is_a6(attr: u32) -> u32 {
    (attr == 0xA6) as u32
}

/// C: `FUN_021BB1DC(attr)`: attr == 0xA8.
pub fn is_a8(attr: u32) -> u32 {
    (attr == 0xA8) as u32
}

/// C: `FUN_021BB1E8(attr)`: attr == 0xB0.
pub fn is_b0(attr: u32) -> u32 {
    (attr == 0xB0) as u32
}

#[cfg(test)]
mod tests;
