//! `field_rail_loader.c` (overlay 21, `0x0219EF14..0x0219F030`; F3b): the RAIL LOADER, which loads a rail file (ARC 0x4F)
//! for the no-grid mapper and exposes its RAIL DATA.
//!
//! RAIL LOADER (0x38): `+0` the loaded file, `+4` its size, `+8` the RAIL DATA (0x30): `+8 / +0xA / +0xC / +0xE` u16 counts
//! (points of 0x70 bytes, lines of 0x48, two tables of 0x24), `+0x10 .. +0x1C` pointers to the four tables (file + offset
//! word 2..5), `+0x2C` the word at file + file\[1\]. Cleared rail data: counts 0x14 / 3 / 3 and the default tables ROM
//! `0x021DDDB8` / `0x021DDDA0` / `0x021DDDAC`.
//!
//! F1 put row 21.12 at `0x0219EF44..0x021A0100`; the code after `0x0219F030` is the MMDL billboard-actor draw procs (render,
//! D3), not part of this file. Heap and archive calls go through [`FieldEnv::call`]. Verified by
//! `tools/check-fn.sh black-field rail_loader --per-function` (spec `tools/rom-diff/specs/black-field/rail_loader.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_EF14,
    0x0219_EF44,
    0x0219_EF6C,
    0x0219_EF7C,
    0x0219_EFFC,
    0x0219_F018,
    0x0219_F01C,
];

/// Address of the string `"field_rail_loader.c"`.
pub const FILE_RAIL_LOADER: u32 = 0x021D_DE08;
/// `FUN_0204A298(arc, id, 0, heap, &size)`: load an archive member (allocated), returns its address.
const ARC_LOAD: u32 = 0x0204_A298;
/// The rail archive.
const ARC_RAIL: u32 = 0x4F;
/// Default tables of a cleared RAIL DATA (`+0x18`, `+0x1C`, `+0x20`).
pub const DEFAULT_TABLES: [u32; 3] = [0x021D_DDB8, 0x021D_DDA0, 0x021D_DDAC];

/// C: `FUN_0219EF14(r)`: clear the RAIL DATA `r` (0x30 bytes): counts 0x14 / 3 / 3, the default tables.
pub fn clear<E: FieldEnv>(env: &mut E, r: u32) {
    env.mem().fill(r, 0x30, 0);
    env.w16(r + 8, 0x14);
    env.w16(r + 0xA, 3);
    env.w16(r + 0xC, 3);
    env.w32(r + 0x20, DEFAULT_TABLES[0]);
    env.w32(r + 0x24, DEFAULT_TABLES[1]);
    env.w32(r + 0x28, DEFAULT_TABLES[2]);
}

/// C: `FUN_0219EF44(heap)`: a new loader (0x38 bytes of heap `heap & 0xFFFF`) with its rail data cleared.
pub fn create<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    let ld = env.callv(
        HEAP_ALLOC,
        &[heap & 0xFFFF, 0x38, 1, FILE_RAIL_LOADER, 0xC0],
    );
    clear(env, ld.wrapping_add(8));
    ld
}

/// C: `FUN_0219EF6C(ld)`: unload, then free the loader.
pub fn delete<E: FieldEnv>(env: &mut E, ld: u32) {
    unload(env, ld);
    env.callv(HEAP_FREE, &[ld]);
}

/// C: `FUN_0219EF7C(ld, id, heap)`: load rail file `id` and point the rail data at its four tables (counts = table sizes /
/// 0x70, 0x48, 0x24, 0x24); `+0x34` = the word at file + file\[1\].
pub fn load<E: FieldEnv>(env: &mut E, ld: u32, id: u32, heap: u32) {
    let mut size = [0u8; 4];
    let f = env.call(
        ARC_LOAD,
        &mut [
            CArg::V(ARC_RAIL),
            CArg::V(id),
            CArg::V(0),
            CArg::V(heap & 0xFFFF),
            CArg::OutBuf(&mut size),
        ],
    );
    env.w32(ld, f);
    env.w32(ld + 4, u32::from_le_bytes(size));
    // (word offset, element size) of the four tables, stored at +0x18.. (pointers) and +8.. (counts)
    for (i, el) in [0x70u32, 0x48, 0x24, 0x24].into_iter().enumerate() {
        let i = i as u32;
        let lo = env.r32(f + 8 + 4 * i);
        let hi = env.r32(f + 0xC + 4 * i);
        env.w32(ld + 0x18 + 4 * i, f.wrapping_add(lo));
        env.w16(ld + 8 + 2 * i, (hi.wrapping_sub(lo) / el) as u16);
    }
    let o = env.r32(f + 4);
    let w = env.r32(f.wrapping_add(o));
    env.w32(ld + 0x34, w);
}

/// C: `FUN_0219EFFC(ld)`: free the loaded file (if any; `+0` = 0) and clear the rail data.
pub fn unload<E: FieldEnv>(env: &mut E, ld: u32) {
    let f = env.r32(ld);
    if f != 0 {
        env.callv(HEAP_FREE, &[f]);
        env.w32(ld, 0);
    }
    clear(env, ld + 8);
}

/// C: `FUN_0219F018(ld)`: the rail data (`ld + 8`).
pub fn rail_data(ld: u32) -> u32 {
    ld.wrapping_add(8)
}

/// C: `FUN_0219F01C(a, b, c, d, e, f)`: `FUN_02158768(a, (u16)c, e, (u16)f)` (`b` and `d` are unused).
pub fn call_02158768<E: FieldEnv>(env: &mut E, a: u32, c: u32, e: u32, f: u32) -> u32 {
    env.callv(0x0215_8768, &[a, c & 0xFFFF, e, f & 0xFFFF])
}

#[cfg(test)]
mod tests;
