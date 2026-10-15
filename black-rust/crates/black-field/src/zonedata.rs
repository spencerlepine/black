//! `zonedata.c` (`0x020139B0..0x02014128`) and the 0x1C-byte LOCATION helpers right after it (`..0x020142A4`).
//!
//! The zone cache (`0x4C` bytes, the pointer is the global `0x02146264`) holds the handle of the zone
//! header archive, the current zone record (`0x30` bytes at `+4`, re-read from the archive when the
//! requested id differs from the id at `+0x34`) and three small tables. The layout is documented in the
//! header of `tools/rom-diff/specs/black-field/zonedata.spec`; every function is verified per function
//! with `tools/check-fn.sh black-field zonedata` (84 functions).
//!
//! Pointers stay `u32` addresses read and written through [`MemEnv`]; unported callees are the methods
//! of [`ZoneHooks`]. The ROM tables the original reads at fixed addresses are embedded below.
//!
//! UNVERIFIED: every function that reads the zone cache assumes the cache exists (as the game does after
//! `cache_new`). The original dereferences a null cache pointer in `remap` and in the record accessors
//! (`zone_record` itself returns null); this port panics at the `Mem` window instead.

use crate::hooks::{MemEnv, ZoneHooks, FILE_ZONEDATA};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_39B0,
    0x0201_3A48,
    0x0201_3A80,
    0x0201_3A94,
    0x0201_3AD0,
    0x0201_3B10,
    0x0201_3B38,
    0x0201_3B44,
    0x0201_3B50,
    0x0201_3B5C,
    0x0201_3B68,
    0x0201_3B74,
    0x0201_3B90,
    0x0201_3BA8,
    0x0201_3BBC,
    0x0201_3BD0,
    0x0201_3BDC,
    0x0201_3C10,
    0x0201_3C20,
    0x0201_3C30,
    0x0201_3C40,
    0x0201_3C50,
    0x0201_3C60,
    0x0201_3C70,
    0x0201_3C80,
    0x0201_3C9C,
    0x0201_3CC8,
    0x0201_3CE0,
    0x0201_3D00,
    0x0201_3D20,
    0x0201_3D34,
    0x0201_3D5C,
    0x0201_3D74,
    0x0201_3D9C,
    0x0201_3DB8,
    0x0201_3DD0,
    0x0201_3DE0,
    0x0201_3DF0,
    0x0201_3E04,
    0x0201_3E40,
    0x0201_3E4C,
    0x0201_3E6C,
    0x0201_3E7C,
    0x0201_3E90,
    0x0201_3EC0,
    0x0201_3ECC,
    0x0201_3ED8,
    0x0201_3EEC,
    0x0201_3F2C,
    0x0201_3F6C,
    0x0201_3FA8,
    0x0201_3FD8,
    0x0201_3FEC,
    0x0201_3FFC,
    0x0201_400C,
    0x0201_4018,
    0x0201_4024,
    0x0201_4034,
    0x0201_4044,
    0x0201_405C,
    0x0201_4068,
    0x0201_407C,
    0x0201_40B4,
    0x0201_40BC,
    0x0201_40C0,
    0x0201_40CC,
    0x0201_40D8,
    0x0201_40DC,
    0x0201_40E0,
    0x0201_40E4,
    0x0201_40E8,
    0x0201_40F8,
    0x0201_4110,
    0x0201_4134,
    0x0201_416C,
    0x0201_4184,
    0x0201_41A8,
    0x0201_41DC,
    0x0201_41E0,
    0x0201_41F0,
    0x0201_4204,
    0x0201_4264,
    0x0201_4274,
    0x0201_42A4,
];

/// Address of the global that holds the zone cache pointer.
pub const ZONE_CACHE_PTR: u32 = 0x0214_6264;
/// Size of the zone cache.
pub const ZONE_CACHE_SIZE: u32 = 0x4C;
/// Size of one zone record in the header archive.
pub const ZONE_RECORD_SIZE: u32 = 0x30;
/// Zone id the cache holds right after `cache_new`.
pub const ZONE_ID_INITIAL: u32 = 0x1AB;

/// ROM table at `0x020A72A8`: `(zone id, replacement)` pairs; pair `k` applies when the byte `cache + 0x48 + k` is set.
const REMAP: [(u16, u16); 2] = [(0x0000, 0x01A8), (0x0126, 0x0127)];
/// ROM table `0x0209E010` (11 halfwords).
const TABLE_E010: [u16; 11] = [
    0x001C, 0x00F9, 0x00D6, 0x00D1, 0x00D3, 0x0042, 0x00F1, 0x00F2, 0x00F3, 0x00F4, 0x0024,
];
/// ROM table `0x0209DFC0` (4 halfwords).
const TABLE_DFC0: [u16; 4] = [0x01A8, 0x0000, 0x0126, 0x0127];
/// ROM table `0x0209DFEC` (9 halfwords).
const TABLE_DFEC: [u16; 9] = [
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x01A8, 0x01A9, 0x01AA,
];
/// ROM table `0x0209DFDC` (8 halfwords).
const TABLE_DFDC: [u16; 8] = [
    0x00F1, 0x00F2, 0x00F3, 0x00F4, 0x00F5, 0x00F6, 0x00F7, 0x00F8,
];
/// ROM table `0x0209DFB8` (4 halfwords).
const TABLE_DFB8: [u16; 4] = [0x00F5, 0x00F6, 0x00F7, 0x00F8];
/// ROM table `0x0209DFD2` (5 halfwords).
const TABLE_DFD2: [u16; 5] = [0x0051, 0x0052, 0x0053, 0x0056, 0x0057];
/// ROM table `0x0209E026` (20 halfwords: `id | value << 11`).
const TABLE_E026: [u16; 20] = [
    0x001C, 0x58F9, 0x081E, 0x48D6, 0x6952, 0x7153, 0x7954, 0x8155, 0x2072, 0x38D1, 0x40D3, 0x1842,
    0x2879, 0x50F1, 0x50F2, 0x50F3, 0x50F4, 0x1024, 0x60FF, 0x3089,
];
/// ROM table `0x0209DFC8` (5 halfwords, same format).
const TABLE_DFC8: [u16; 5] = [0x003E, 0x21A2, 0x1946, 0x10FF, 0x088A];

// ---------------------------------------------------------------------------------------------
// zone cache
// ---------------------------------------------------------------------------------------------

/// C: `FUN_020139B0`: create the zone cache when it does not exist yet.
pub fn cache_new<H: ZoneHooks>(h: &mut H, heap: u32) {
    if h.r32(ZONE_CACHE_PTR) != 0 {
        return;
    }
    let cache = h.heap_alloc(heap, ZONE_CACHE_SIZE, 1, FILE_ZONEDATA, 0x60);
    h.w32(ZONE_CACHE_PTR, cache);
    let handle = h.fun_020490f4(0xC, heap);
    h.w32(cache, handle);
    let mut n = 0;
    let a = h.fun_0204a298(0x51, 0xD, 0, heap, &mut n);
    h.w32(cache + 0x38, a);
    h.w16(cache + 0x40, (n >> 2) as u16);
    let a = h.fun_0204a298(0x52, 1, 0, heap, &mut n);
    h.w32(cache + 0x3C, a);
    h.w8(cache + 0x42, (n >> 2) as u8);
    let mut m = 0;
    let a = h.fun_0204a298(0x95, 0, 0, heap, &mut m);
    h.w32(cache + 0x44, a);
    h.w8(cache + 0x43, (m >> 2) as u8);
    h.w16(cache + 0x34, ZONE_ID_INITIAL as u16);
}

/// C: `FUN_02013A48`: free the zone cache and its tables.
pub fn cache_free<H: ZoneHooks>(h: &mut H) {
    let cache = h.r32(ZONE_CACHE_PTR);
    if cache == 0 {
        return;
    }
    let t = h.r32(cache + 0x44);
    h.heap_free(t);
    let t = h.r32(cache + 0x3C);
    h.heap_free(t);
    let t = h.r32(cache + 0x38);
    h.heap_free(t);
    let handle = h.r32(cache);
    h.fun_02049238(handle);
    h.heap_free(cache);
    h.w32(ZONE_CACHE_PTR, 0);
}

/// C: `FUN_02013A80`: when the cache exists, `cache[0x48 + i] = v` (the remap enable bytes).
pub fn set_remap_flag<E: MemEnv>(env: &mut E, i: u32, v: u8) {
    let cache = env.r32(ZONE_CACHE_PTR);
    if cache != 0 {
        env.w8(cache.wrapping_add(i).wrapping_add(0x48), v);
    }
}

/// C: `FUN_02013A94`: zone id remap: pair `k` of the table `0x020A72A8` replaces its first id by its second
/// when the enable byte `cache[0x48 + k]` is set.
pub fn remap<E: MemEnv>(env: &mut E, id: u32) -> u32 {
    let cache = env.r32(ZONE_CACHE_PTR);
    for (k, (from, to)) in REMAP.iter().enumerate() {
        if env.r8(cache + 0x48 + k as u32) != 0 && id == *from as u32 {
            return *to as u32;
        }
    }
    id
}

/// C: `FUN_02013AD0`: address of the zone record of `id` (the cache record at `+4`, re-read when the remapped
/// id differs from the cached one); 0 when there is no cache.
pub fn zone_record<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let cache = h.r32(ZONE_CACHE_PTR);
    if cache == 0 {
        return 0;
    }
    let id = remap(h, id);
    let cache = h.r32(ZONE_CACHE_PTR);
    if id != h.r16(cache + 0x34) as u32 {
        let handle = h.r32(cache);
        h.fun_0204944c(
            handle,
            0,
            id.wrapping_mul(ZONE_RECORD_SIZE),
            ZONE_RECORD_SIZE,
            cache + 4,
        );
        let cache = h.r32(ZONE_CACHE_PTR);
        h.w16(cache + 0x34, id as u16);
    }
    h.r32(ZONE_CACHE_PTR) + 4
}

/// C: `FUN_02013B10`: whether the remapped `id` is one of the `n` halfwords at `list` (`n <= 0` gives false).
pub fn zone_in_list<E: MemEnv>(env: &mut E, id: u32, list: u32, n: i32) -> bool {
    let id = remap(env, id);
    for i in 0..n.max(0) as u32 {
        if id == env.r16(list + 2 * i) as u32 {
            return true;
        }
    }
    false
}

/// The remapped `id` is one of the halfwords of a ROM table.
fn id_in_table<E: MemEnv>(env: &mut E, id: u32, table: &[u16]) -> bool {
    let id = remap(env, id);
    table.iter().any(|&x| x as u32 == id)
}

/// The `(value)` of the first entry of a ROM table of `id | value << 11` whose low 11 bits equal the remapped id.
fn table_value<E: MemEnv>(env: &mut E, id: u32, table: &[u16]) -> Option<u32> {
    let id = remap(env, id);
    table
        .iter()
        .find(|&&e| (e & 0x7FF) as u32 == id)
        .map(|&e| (e >> 11) as u32)
}

// ---------------------------------------------------------------------------------------------
// zone record accessors that are more than one load
// ---------------------------------------------------------------------------------------------

/// C: `FUN_02013BDC`: halfword `k` of the four at `+0xC` of the zone record of `id`, else 0 (`k >= 4`).
pub fn zone_u16_0c_k<H: ZoneHooks>(h: &mut H, id: u32, k: u32) -> u16 {
    let rec = zone_record(h, id);
    if k <= 3 {
        h.r16(rec + 0xC + 2 * k)
    } else {
        0
    }
}

/// The three position words at `+0x24` of the zone record of `id`.
fn zone_pos_raw<H: ZoneHooks>(h: &mut H, id: u32) -> [u32; 3] {
    let rec = zone_record(h, id);
    [h.r32(rec + 0x24), h.r32(rec + 0x28), h.r32(rec + 0x2C)]
}

/// C: `FUN_02013B74`: `out[0..3]` = the position words of the zone record shifted left by 16 (fixed point).
pub fn zone_pos_fixed<H: ZoneHooks>(h: &mut H, id: u32, out: u32) {
    let p = zone_pos_raw(h, id);
    for (k, v) in p.iter().enumerate() {
        h.w32(out + 4 * k as u32, v << 16);
    }
}

/// C: `FUN_02013B90`: `out[0..3]` = the position words of the zone record.
pub fn zone_pos<H: ZoneHooks>(h: &mut H, id: u32, out: u32) {
    let p = zone_pos_raw(h, id);
    for (k, v) in p.iter().enumerate() {
        h.w32(out + 4 * k as u32, *v);
    }
}

/// C: `FUN_02013E4C`: `u16 +0x1A & 0x3FF` of the zone record, but 0x70 when that is 0x70 or 0x71.
pub fn zone_1a_low10<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let rec = zone_record(h, id);
    let v = (h.r16(rec + 0x1A) & 0x3FF) as u32;
    if (v.wrapping_add(0xFF90) & 0xFFFF) <= 1 {
        0x70
    } else {
        v
    }
}

/// C: `FUN_02013E7C`: whether the u16 at `+4` of the zone record is 0.
pub fn zone_u16_04_is_zero<H: ZoneHooks>(h: &mut H, id: u32) -> bool {
    let rec = zone_record(h, id);
    h.r16(rec + 4) == 0
}

/// C: `FUN_02013E90`: the `+4` halfwords of the two zone records are equal, or one is 0 and the other 0xAF.
pub fn zone_same_area<H: ZoneHooks>(h: &mut H, a: u32, b: u32) -> bool {
    let x = zone_u16_04(h, a);
    let y = zone_u16_04(h, b);
    if x == y {
        return true;
    }
    (x == 0 && y == 0xAF) || (x == 0xAF && y == 0)
}

/// C: `FUN_02014068`: `FUN_02013E40(id) == 0xD6`.
pub fn zone_u16_18_is_d6<H: ZoneHooks>(h: &mut H, id: u32) -> bool {
    zone_u16_18(h, id) == 0xD6
}

// ---------------------------------------------------------------------------------------------
// zone id predicates (with remap)
// ---------------------------------------------------------------------------------------------

/// C: `FUN_02013C70`: the remapped id is one of the 11 halfwords of the table `0x0209E010`.
pub fn zone_in_e010<E: MemEnv>(env: &mut E, id: u32) -> bool {
    id_in_table(env, id, &TABLE_E010)
}

/// C: `FUN_02013DD0`: the remapped id is one of the 4 halfwords of the table `0x0209DFC0`.
pub fn zone_in_dfc0<E: MemEnv>(env: &mut E, id: u32) -> bool {
    id_in_table(env, id, &TABLE_DFC0)
}

/// C: `FUN_02013DE0`: the remapped id is one of the 9 halfwords of the table `0x0209DFEC`.
pub fn zone_in_dfec<E: MemEnv>(env: &mut E, id: u32) -> bool {
    id_in_table(env, id, &TABLE_DFEC)
}

/// C: `FUN_02013FEC`: the remapped id is one of the 8 halfwords of the table `0x0209DFDC`.
pub fn zone_in_dfdc<E: MemEnv>(env: &mut E, id: u32) -> bool {
    id_in_table(env, id, &TABLE_DFDC)
}

/// C: `FUN_02013FFC`: the remapped id is one of the 4 halfwords of the table `0x0209DFB8`.
pub fn zone_in_dfb8<E: MemEnv>(env: &mut E, id: u32) -> bool {
    id_in_table(env, id, &TABLE_DFB8)
}

/// C: `FUN_02014024`: the remapped id is one of the 5 halfwords of the table `0x0209DFD2`.
pub fn zone_in_dfd2<E: MemEnv>(env: &mut E, id: u32) -> bool {
    id_in_table(env, id, &TABLE_DFD2)
}

/// C: `FUN_02013C9C`: value (`>> 11`) of the entry of the table `0x0209E026` whose low 11 bits equal the
/// remapped id, else `0xFFFFFFFF`.
pub fn zone_table_e026<E: MemEnv>(env: &mut E, id: u32) -> u32 {
    table_value(env, id, &TABLE_E026).unwrap_or(0xFFFF_FFFF)
}

/// C: `FUN_02013C80`: `FUN_02013C9C(FUN_02013A94(id)) != -1` (the id is remapped twice).
pub fn zone_has_e026<E: MemEnv>(env: &mut E, id: u32) -> bool {
    let id = remap(env, id);
    zone_table_e026(env, id) != 0xFFFF_FFFF
}

/// C: `FUN_02013FA8`: like `FUN_02013C9C` on the table `0x0209DFC8`, default `0xFFFF`.
pub fn zone_table_dfc8<E: MemEnv>(env: &mut E, id: u32) -> u32 {
    table_value(env, id, &TABLE_DFC8).unwrap_or(0xFFFF)
}

/// C: `FUN_02013CC8`: the remapped id is 422.
pub fn zone_is_422<E: MemEnv>(env: &mut E, id: u32) -> bool {
    remap(env, id) == 0x1A6
}

/// C: `FUN_02013CE0`: the remapped id is 150 or 151 (compared as a halfword sum).
pub fn zone_is_150_151<E: MemEnv>(env: &mut E, id: u32) -> bool {
    (remap(env, id).wrapping_add(0xFF6A) & 0xFFFF) <= 1
}

/// C: `FUN_02013D00`: the remapped id is 75 or 76 (compared as a halfword sum).
pub fn zone_is_75_76<E: MemEnv>(env: &mut E, id: u32) -> bool {
    (remap(env, id).wrapping_add(0xFFB5) & 0xFFFF) <= 1
}

/// C: `FUN_02013D20`: the remapped id is 0x34.
pub fn zone_is_34<E: MemEnv>(env: &mut E, id: u32) -> bool {
    remap(env, id) == 0x34
}

/// C: `FUN_02013D34`: `FUN_02013CC8 || FUN_02013CE0 || FUN_02013D20`.
pub fn zone_is_blocked<E: MemEnv>(env: &mut E, id: u32) -> bool {
    zone_is_422(env, id) || zone_is_150_151(env, id) || zone_is_34(env, id)
}

/// C: `FUN_02013D5C`: the remapped id is 279.
pub fn zone_is_279<E: MemEnv>(env: &mut E, id: u32) -> bool {
    remap(env, id) == 0x117
}

/// C: `FUN_02013D9C`: the remapped id is in 280..=288.
pub fn zone_in_280_288<E: MemEnv>(env: &mut E, id: u32) -> bool {
    (280..=288).contains(&remap(env, id))
}

/// C: `FUN_02013D74`: the remapped id is in 289..=316, or `FUN_02013D9C` of the remapped id.
pub fn zone_in_280_316<E: MemEnv>(env: &mut E, id: u32) -> bool {
    let id = remap(env, id);
    if (289..=316).contains(&id) {
        return true;
    }
    zone_in_280_288(env, id)
}

/// C: `FUN_02013DF0`: the remapped id is 0x4E.
pub fn zone_is_4e<E: MemEnv>(env: &mut E, id: u32) -> bool {
    remap(env, id) == 0x4E
}

/// C: `FUN_02013E04`: none of `FUN_02013CC8`, `FUN_02013CE0`, `FUN_02013D00`, `FUN_02013D5C`, `FUN_02013D74` is true.
pub fn zone_is_plain<E: MemEnv>(env: &mut E, id: u32) -> bool {
    !(zone_is_422(env, id)
        || zone_is_150_151(env, id)
        || zone_is_75_76(env, id)
        || zone_is_279(env, id)
        || zone_in_280_316(env, id))
}

/// The remapped `id` looked up in the table `(u16 id, u16 value)` x `count` at the pointer `cache[ptr_ofs]`.
fn cache_table<E: MemEnv>(
    env: &mut E,
    id: u32,
    ptr_ofs: u32,
    count_ofs: u32,
    wide: bool,
    default: u32,
) -> u32 {
    let id = remap(env, id);
    let cache = env.r32(ZONE_CACHE_PTR);
    let count = if wide {
        env.r16(cache + count_ofs) as u32
    } else {
        env.r8(cache + count_ofs) as u32
    };
    let table = env.r32(cache + ptr_ofs);
    for i in 0..count {
        if id == env.r16(table + 4 * i) as u32 {
            return env.r16(table + 4 * i + 2) as u32;
        }
    }
    default
}

/// C: `FUN_02013EEC`: value of the entry for the remapped id in table A (`cache + 0x38`, count `u16` at `+0x40`),
/// else `0x0FFFFFFF`.
pub fn zone_table_a<E: MemEnv>(env: &mut E, id: u32) -> u32 {
    cache_table(env, id, 0x38, 0x40, true, 0x0FFF_FFFF)
}

/// C: `FUN_02013F2C`: the same for table B (`cache + 0x3C`, count `u8` at `+0x42`), else `0x0FFFFFFF`.
pub fn zone_table_b<E: MemEnv>(env: &mut E, id: u32) -> u32 {
    cache_table(env, id, 0x3C, 0x42, false, 0x0FFF_FFFF)
}

/// C: `FUN_02013F6C`: the same for table C (`cache + 0x44`, count `u8` at `+0x43`), else 0.
pub fn zone_table_c<E: MemEnv>(env: &mut E, id: u32) -> u32 {
    cache_table(env, id, 0x44, 0x43, false, 0)
}

/// C: `FUN_0201407C(heap, kind, season)`: for an id in 2..0xD2 add `c` (mod 0x10000), wrap at 282, then
/// `FUN_02020E08(0xD, a, b * 10, 10)` (archive 0xD load: the 10-byte record; r3 is the multiplier), whose result it returns: the area data the FIELDMAP keeps at
/// `+0xC` (`fieldmap::create`; tail `bl` then `pop`, V-0022 audit).
pub fn load_area_data<H: ZoneHooks>(h: &mut H, a: u32, b: u32, c: u32) -> u32 {
    let mut b = b;
    if is_zone_2_d1(b) {
        b = b.wrapping_add(c & 0xFFFF) & 0xFFFF;
    }
    if b >= 0x11A {
        b = 0;
    }
    h.fun_02020e08(0xD, a, b.wrapping_mul(10), 10)
}

/// C: `FUN_020140B4`: tail call of the heap free routine.
pub fn small_free<H: ZoneHooks>(h: &mut H, p: u32) {
    h.heap_free(p);
}

// ---------------------------------------------------------------------------------------------
// LOCATION (0x1C bytes)
// ---------------------------------------------------------------------------------------------

/// C: `FUN_020140F8`: zero a LOCATION: `+0`, `+4/+6/+8/+0xA` (u16), `+0xC`, `+0x10`, `+0x14`, `+0x18`.
pub fn loc_clear<E: MemEnv>(env: &mut E, loc: u32) {
    env.w32(loc, 0);
    env.w16(loc + 4, 0);
    env.w16(loc + 6, 0);
    env.w16(loc + 8, 0);
    env.w16(loc + 0xA, 0);
    env.w32(loc + 0xC, 0);
    env.w32(loc + 0x10, 0);
    env.w32(loc + 0x14, 0);
    env.w32(loc + 0x18, 0);
}

/// C: `FUN_02014110`: kind 0, u16 `+4 = a`, `+6 = b`, `+8 = c`, `+0xA = d`, `+0xC = 0` and the position words `e`, `f`, `g`.
#[allow(clippy::too_many_arguments)]
pub fn loc_set_fixed<E: MemEnv>(
    env: &mut E,
    loc: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    g: u32,
) {
    env.w16(loc + 4, a as u16);
    env.w32(loc, 0);
    env.w16(loc + 6, b as u16);
    env.w16(loc + 8, c as u16);
    env.w16(loc + 0xA, d as u16);
    env.w32(loc + 0xC, 0);
    env.w32(loc + 0x10, e);
    env.w32(loc + 0x14, f);
    env.w32(loc + 0x18, g);
}

/// C: `FUN_02014134`: kind 0, whole-tile position (`+0xC = 1`): u16 `+0x10 = e`, `+0x16 = f`, s16 `+0x14 = g`,
/// `+0x12 = 1`, `+0x13 = FUN_0201441C(FUN_02014274(c))`.
#[allow(clippy::too_many_arguments)]
pub fn loc_set_tiles<H: ZoneHooks>(
    h: &mut H,
    loc: u32,
    id: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
    g: u32,
) {
    h.w16(loc + 4, id as u16);
    h.w16(loc + 6, b as u16);
    h.w16(loc + 8, c as u16);
    h.w32(loc, 0);
    h.w16(loc + 0xA, d as u16);
    h.w32(loc + 0xC, 1);
    h.w8(loc + 0x12, 1);
    h.w16(loc + 0x10, e as u16);
    h.w16(loc + 0x16, f as u16);
    h.w16(loc + 0x14, g as u16);
    let code = dir_index(c);
    let v = h.fun_0201441c(code);
    h.w8(loc + 0x13, v as u8);
}

/// C: `FUN_0201416C`: clear the LOCATION, then u16 `+4 = a`, `+6 = b`, `+0xA = c`.
pub fn loc_set_ids<E: MemEnv>(env: &mut E, loc: u32, a: u32, b: u32, c: u32) {
    loc_clear(env, loc);
    env.w16(loc + 4, a as u16);
    env.w16(loc + 6, b as u16);
    env.w16(loc + 0xA, c as u16);
}

/// C: `FUN_02014184`: kind 1: u16 `+4 = a`, `+6 = 0xFFFF`, `+0xA = 0`, `+0xC = 0`, `+8 = b`, `+0x10 = c`, `+0x14 = d`, `+0x18 = e`.
pub fn loc_set_kind1<E: MemEnv>(env: &mut E, loc: u32, a: u32, b: u32, c: u32, d: u32, e: u32) {
    env.w16(loc + 4, a as u16);
    env.w16(loc + 6, 0xFFFF);
    env.w16(loc + 0xA, 0);
    env.w32(loc + 0xC, 0);
    env.w32(loc, 1);
    env.w32(loc + 0x14, d);
    env.w16(loc + 8, b as u16);
    env.w32(loc + 0x10, c);
    env.w32(loc + 0x18, e);
}

/// C: `FUN_020141A8`: kind 1, whole-tile position: u16 `+4 = a`, `+6 = 0xFFFF`, `+8 = b`, `+0xC = 1`, `+0x12 = 1`,
/// `+0x10 = c`, `+0xA = 0`, `+0x16 = d`, s16 `+0x14 = e`, `+0x13 = FUN_0201441C(FUN_02014274(b))`.
pub fn loc_set_kind1_tiles<H: ZoneHooks>(
    h: &mut H,
    loc: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
) {
    h.w16(loc + 4, a as u16);
    h.w16(loc + 6, 0xFFFF);
    h.w32(loc, 1);
    h.w16(loc + 8, b as u16);
    h.w32(loc + 0xC, 1);
    h.w8(loc + 0x12, 1);
    h.w16(loc + 0x10, c as u16);
    h.w16(loc + 0xA, 0);
    h.w16(loc + 0x16, d as u16);
    h.w16(loc + 0x14, e as u16);
    let code = dir_index(b);
    let v = h.fun_0201441c(code);
    h.w8(loc + 0x13, v as u8);
}

/// C: `FUN_020141E0`: `out[0..3]` = the position words at `+0x10`, `+0x14`, `+0x18`.
pub fn loc_get_pos<E: MemEnv>(env: &mut E, loc: u32, out: u32) {
    let x = env.r32(loc + 0x10);
    let y = env.r32(loc + 0x14);
    let z = env.r32(loc + 0x18);
    env.w32(out + 4, y);
    env.w32(out, x);
    env.w32(out + 8, z);
}

/// C: `FUN_020141F0`: `+0xC = 0`; `+0x10..+0x18` = the three words at `src`.
pub fn loc_set_pos<E: MemEnv>(env: &mut E, loc: u32, src: u32) {
    env.w32(loc + 0xC, 0);
    env.mem().copy(loc + 0x10, src, 12);
}

/// C: `FUN_02014204`: LOCATION of a zone: kind 1, `+8 = 2`, `+6 = 0xFFFF`; in whole tiles from `FUN_02013B90` for
/// the zones of `FUN_02013C70`, else in fixed point from `FUN_02013B74`.
pub fn loc_set_zone<H: ZoneHooks>(h: &mut H, loc: u32, id: u32) {
    loc_clear(h, loc);
    h.w16(loc + 6, 0xFFFF);
    h.w16(loc + 8, 2);
    h.w32(loc, 1);
    h.w16(loc + 4, id as u16);
    h.w16(loc + 0xA, 0);
    if zone_in_e010(h, id) {
        let p = zone_pos_raw(h, id);
        h.w32(loc + 0xC, 1);
        h.w8(loc + 0x12, 1);
        h.w16(loc + 0x10, p[0] as u16);
        h.w16(loc + 0x16, p[1] as u16);
        h.w16(loc + 0x14, p[2] as u16);
    } else {
        let p = zone_pos_raw(h, id);
        h.w32(loc + 0xC, 0);
        for (k, v) in p.iter().enumerate() {
            h.w32(loc + 0x10 + 4 * k as u32, v << 16);
        }
    }
}

/// C: `FUN_02014264`: `FUN_02014204(loc, 0x187)`.
pub fn loc_set_default_zone<H: ZoneHooks>(h: &mut H, loc: u32) {
    loc_set_zone(h, loc, 0x187);
}

/// C: `FUN_02014274`: direction code to index: 1 -> 0, 2 -> 1, 3 -> 2, 4 -> 3, anything else -> 1.
pub fn dir_index(code: u32) -> u32 {
    match code {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 1,
    }
}

/// C: `FUN_020142A4`: index to direction code: 0 -> 1, 1 -> 2, 2 -> 3, 3 -> 4, anything else -> 2.
pub fn dir_code(index: u32) -> u32 {
    match index {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        _ => 2,
    }
}

// ---------------------------------------------------------------------------------------------
// accessors
// ---------------------------------------------------------------------------------------------

/// C: `FUN_02013B38`: u16 at +0x2 of the zone record of `id`.
pub fn zone_u16_02<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x2)
}

/// C: `FUN_02013B44`: u16 at +0x4 of the zone record of `id`.
pub fn zone_u16_04<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x4)
}

/// C: `FUN_02013B5C`: u16 at +0x6 of the zone record of `id`.
pub fn zone_u16_06<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x6)
}

/// C: `FUN_02013B68`: u16 at +0xa of the zone record of `id`.
pub fn zone_u16_0a<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0xa)
}

/// C: `FUN_02013BD0`: u16 at +0x20 of the zone record of `id`.
pub fn zone_u16_20<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x20)
}

/// C: `FUN_02013E40`: u16 at +0x18 of the zone record of `id`.
pub fn zone_u16_18<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x18)
}

/// C: `FUN_02013EC0`: u16 at +0x14 of the zone record of `id`.
pub fn zone_u16_14<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x14)
}

/// C: `FUN_02013ECC`: u16 at +0x16 of the zone record of `id`.
pub fn zone_u16_16<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x16)
}

/// C: `FUN_02013B50`: u8 at +0x0 of the zone record of `id`.
pub fn zone_u8_00<H: ZoneHooks>(h: &mut H, id: u32) -> u8 {
    let rec = zone_record(h, id);
    h.r8(rec)
}

/// C: `FUN_0201405C`: u8 at +0x1 of the zone record of `id`.
pub fn zone_u8_01<H: ZoneHooks>(h: &mut H, id: u32) -> u8 {
    let rec = zone_record(h, id);
    h.r8(rec + 0x1)
}

/// C: `FUN_02013BA8`: bits 6..8 of the u16 at +0x1C of the zone record of `id`.
pub fn zone_1c_bits_6_8<H: ZoneHooks>(h: &mut H, id: u32) -> u8 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1C) >> 6) & 7) as u8
}

/// C: `FUN_02013BBC`: bits 9..15 of the u16 at +0x1C of the zone record of `id`.
pub fn zone_1c_bits_9_15<H: ZoneHooks>(h: &mut H, id: u32) -> u8 {
    let rec = zone_record(h, id);
    (h.r16(rec + 0x1C) >> 9) as u8
}

/// C: `FUN_02013C10`: bits 0..5 of the u16 at +0x1C of the zone record of `id`.
pub fn zone_1c_bits_0_5<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x1C) & 0x3F
}

/// C: `FUN_02013C20`: bit 10 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bit10<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1E) >> 10) & 1) as u32
}

/// C: `FUN_02013C30`: bit 14 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bit14<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1E) >> 14) & 1) as u32
}

/// C: `FUN_02013C40`: bit 13 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bit13<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1E) >> 13) & 1) as u32
}

/// C: `FUN_02013C50`: bit 12 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bit12<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1E) >> 12) & 1) as u32
}

/// C: `FUN_02013C60`: bit 15 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bit15<H: ZoneHooks>(h: &mut H, id: u32) -> u32 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1E) >> 15) & 1) as u32
}

/// C: `FUN_02013ED8`: bits 5..9 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bits_5_9<H: ZoneHooks>(h: &mut H, id: u32) -> u8 {
    let rec = zone_record(h, id);
    ((h.r16(rec + 0x1E) >> 5) & 0x1F) as u8
}

/// C: `FUN_02013FD8`: bits 0..4 of the u16 at +0x1E of the zone record of `id`.
pub fn zone_1e_bits_0_4<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x1E) & 0x1F
}

/// C: `FUN_02013E6C`: bits 10..15 of the u16 at +0x1A of the zone record of `id`.
pub fn zone_1a_bits_10_15<H: ZoneHooks>(h: &mut H, id: u32) -> u16 {
    let rec = zone_record(h, id);
    h.r16(rec + 0x1A) >> 10
}

/// C: `FUN_020140BC`: u16 at +2 of the small record `p`.
pub fn small_u16_02<E: MemEnv>(env: &mut E, p: u32) -> u16 {
    env.r16(p + 2)
}

/// C: `FUN_020140D8`: u8 at +6 of the small record `p`.
pub fn small_u8_06<E: MemEnv>(env: &mut E, p: u32) -> u8 {
    env.r8(p + 6)
}

/// C: `FUN_020140DC`: u8 at +7 of the small record `p`.
pub fn small_u8_07<E: MemEnv>(env: &mut E, p: u32) -> u8 {
    env.r8(p + 7)
}

/// C: `FUN_020140E0`: u8 at +8 of the small record `p`.
pub fn small_u8_08<E: MemEnv>(env: &mut E, p: u32) -> u8 {
    env.r8(p + 8)
}

/// C: `FUN_020140E4`: u8 at +9 of the small record `p`.
pub fn small_u8_09<E: MemEnv>(env: &mut E, p: u32) -> u8 {
    env.r8(p + 9)
}

/// C: `FUN_020140C0`: u8 at +4 of the small record `p`; 0xFF becomes 0xFFFFFFFF.
pub fn small_u8_04<E: MemEnv>(env: &mut E, p: u32) -> u32 {
    match env.r8(p + 4) {
        0xFF => 0xFFFF_FFFF,
        v => v as u32,
    }
}

/// C: `FUN_020140CC`: u8 at +5 of the small record `p`; 0xFF becomes 0xFFFFFFFF.
pub fn small_u8_05<E: MemEnv>(env: &mut E, p: u32) -> u32 {
    match env.r8(p + 5) {
        0xFF => 0xFFFF_FFFF,
        v => v as u32,
    }
}

/// C: `FUN_020141DC`: u32 at +0xC of the LOCATION `loc` (1 = the position is in whole tiles).
pub fn loc_get_0c<E: MemEnv>(env: &mut E, loc: u32) -> u32 {
    env.r32(loc + 0xC)
}

/// C: `FUN_02014044`: id is 378 or 374 (no remap).
pub fn is_zone_374_or_378(id: u32) -> bool {
    id == 0x17A || id == 0x176
}

/// C: `FUN_0201400C`: id is 0xF6 (no remap).
pub fn is_zone_f6(id: u32) -> bool {
    id == 0xF6
}

/// C: `FUN_02014018`: id is 0xF7 (no remap).
pub fn is_zone_f7(id: u32) -> bool {
    id == 0xF7
}

/// C: `FUN_02014034`: 0x4C <= id <= 0x69 (no remap).
pub fn is_zone_4c_69(id: u32) -> bool {
    (0x4C..=0x69).contains(&id)
}

/// C: `FUN_020140E8`: 2 <= id < 0xD2 (no remap).
pub fn is_zone_2_d1(id: u32) -> bool {
    (2..0xD2).contains(&id)
}

/// C: `FUN_02013DB8`: 289 <= id <= 316 (no remap).
pub fn is_zone_289_316(id: u32) -> bool {
    (289..=316).contains(&id)
}

#[cfg(test)]
mod tests;
