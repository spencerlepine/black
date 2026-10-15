//! `field_nogrid_mapper.c` (overlay 21, `0x021BB1F4..0x021BB484`; F3b): the NOGRID MAPPER of a free-movement (rail) map.
//!
//! NOGRID MAPPER (0x18): `+0` the RAIL system (`field_rail`, [`crate::rail`]), `+4` the RAIL LOADER
//! ([`crate::rail_loader`]), `+8` the RAIL ATTR ([`crate::rail_attr`]), `+0xC` the camera area object, `+0x10` the camera
//! area loader (both given to the constructor), `+0x14` the "no rail frame" flag. A map's no-grid record (ARC 0x6D member)
//! is `{u16 rail file, u16 camera area file}` (0xFFFF = none); the attribute file of rail file `r` is ARC 0x4F member
//! `r + count / 2`.
//!
//! Every callee (the other F3b areas included) is reached through [`FieldEnv::call`] by address; the thunks call their
//! targets. Verified by `tools/check-fn.sh black-field nogrid_mapper --per-function` (spec
//! `tools/rom-diff/specs/black-field/nogrid_mapper.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_B1F4,
    0x021B_B240,
    0x021B_B260,
    0x021B_B278,
    0x021B_B2F8,
    0x021B_B348,
    0x021B_B36C,
    0x021B_B388,
    0x021B_B394,
    0x021B_B3A0,
    0x021B_B3AC,
    0x021B_B3B8,
    0x021B_B3C4,
    0x021B_B3D0,
    0x021B_B3DC,
    0x021B_B3E8,
    0x021B_B480,
];

/// Address of the string `"field_nogrid_mapper.c"`.
pub const FILE_NOGRID_MAPPER: u32 = 0x021D_E2D0;

// rail loader (field_rail_loader.c)
const LOADER_CREATE: u32 = 0x0219_EF44;
const LOADER_DELETE: u32 = 0x0219_EF6C;
const LOADER_LOAD: u32 = 0x0219_EF7C;
const LOADER_UNLOAD: u32 = 0x0219_EFFC;
const LOADER_DATA: u32 = 0x0219_F018;
// rail attr (rail_attr.c)
const ATTR_CREATE: u32 = 0x021B_B088;
const ATTR_DELETE: u32 = 0x021B_B0A4;
const ATTR_LOAD: u32 = 0x021B_B0B4;
const ATTR_UNLOAD: u32 = 0x021B_B11C;
const ATTR_AT: u32 = 0x021B_B13C;
// rail system (field_rail.c)
const RAIL_CREATE: u32 = 0x021B_7884;
const RAIL_DELETE: u32 = 0x021B_791C;
const RAIL_SET_DATA: u32 = 0x021B_7934;
const RAIL_CLEAR_DATA: u32 = 0x021B_7954;
const RAIL_FRAME_STATE: u32 = 0x021B_79E4;
const RAIL_FRAME: u32 = 0x021B_79F0;
// camera area (other F items)
const CAM_SET: u32 = 0x021A_B634;
const CAM_CLEAR: u32 = 0x021A_B650;
const CAM_FLAG: u32 = 0x021A_B75C;
const CAMLD_LOAD: u32 = 0x021A_B840;
const CAMLD_UNLOAD: u32 = 0x021A_B868;
const CAMLD_DATA: u32 = 0x021A_B880;
const CAMLD_COUNT: u32 = 0x021A_B884;
const CAMLD_CONST: u32 = 0x021A_B888;
// archives
const ARC_COUNT: u32 = 0x0204_9030;
const ARC_SIZE: u32 = 0x0204_90A8;
const ARC_READ: u32 = 0x0204_8F4C;
/// The no-grid record archive.
const ARC_NOGRID: u32 = 0x6D;
/// "No file" in a no-grid record.
const NONE: u16 = 0xFFFF;

/// The heap id the original uses for temporary buffers: `(heap & 0x7FFF) | 0x8000` (allocate from the top).
fn heap_top(heap: u32) -> u32 {
    (heap & 0x7FFF) | 0x8000
}

/// C: `FUN_021BB1F4(heap, x, cam, camld)`: a new mapper: the RAIL (`heap`, 0x30 objects, `x`), a RAIL LOADER, a RAIL ATTR.
pub fn create<E: FieldEnv>(env: &mut E, heap: u32, x: u32, cam: u32, camld: u32) -> u32 {
    let m = env.callv(
        HEAP_ALLOC,
        &[heap & 0xFFFF, 0x18, 1, FILE_NOGRID_MAPPER, 0x47],
    );
    let rail = env.callv(RAIL_CREATE, &[heap & 0xFFFF, 0x30, x]);
    env.w32(m, rail);
    let ld = env.callv(LOADER_CREATE, &[heap]);
    env.w32(m + 4, ld);
    env.w32(m + 0xC, cam);
    env.w32(m + 0x10, camld);
    let ra = env.callv(ATTR_CREATE, &[heap]);
    env.w32(m + 8, ra);
    m
}

/// C: `FUN_021BB240(m)`: delete the RAIL ATTR, the RAIL, the RAIL LOADER and the mapper.
pub fn delete<E: FieldEnv>(env: &mut E, m: u32) {
    let ra = env.r32(m + 8);
    env.callv(ATTR_DELETE, &[ra]);
    let rail = env.r32(m);
    env.callv(RAIL_DELETE, &[rail]);
    let ld = env.r32(m + 4);
    env.callv(LOADER_DELETE, &[ld]);
    env.callv(HEAP_FREE, &[m]);
}

/// C: `FUN_021BB260(m, f)`: `+0x14 = (f == 0)`, then the camera area's flag (`021AB75C(cam, f)`).
pub fn set_rail_frame<E: FieldEnv>(env: &mut E, m: u32, f: u32) {
    env.w32(m + 0x14, (f == 0) as u32);
    let cam = env.r32(m + 0xC);
    env.callv(CAM_FLAG, &[cam, f]);
}

/// C: `FUN_021BB278(m, rec, heap)`: unload, then load the rail file `rec[0]` (rail data + attribute file) and the camera
/// area file `rec[1]` (0xFFFF = none).
pub fn load_record<E: FieldEnv>(env: &mut E, m: u32, rec: u32, heap: u32) {
    unload(env, m);
    if env.r16(rec) != NONE {
        let n = env.callv(ARC_COUNT, &[0x4F]);
        let ld = env.r32(m + 4);
        let id = u32::from(env.r16(rec));
        env.callv(LOADER_LOAD, &[ld, id, heap]);
        let ld = env.r32(m + 4);
        let d = env.callv(LOADER_DATA, &[ld]);
        let rail = env.r32(m);
        env.callv(RAIL_SET_DATA, &[rail, d]);
        let id = u32::from(env.r16(rec));
        let ra = env.r32(m + 8);
        env.callv(ATTR_LOAD, &[ra, id.wrapping_add(n >> 1), heap]);
    }
    let cam_id = env.r16(rec + 2);
    if cam_id != NONE {
        let camld = env.r32(m + 0x10);
        env.callv(CAMLD_LOAD, &[camld, 0, u32::from(cam_id), heap & 0xFFFF]);
        let camld = env.r32(m + 0x10);
        let a = env.callv(CAMLD_DATA, &[camld]);
        let camld = env.r32(m + 0x10);
        let b = env.callv(CAMLD_COUNT, &[camld]);
        let c = env.callv(CAMLD_CONST, &[]);
        let cam = env.r32(m + 0xC);
        env.callv(CAM_SET, &[cam, a, b, c]);
    }
}

/// C: `FUN_021BB2F8(m, id, heap)`: read the no-grid record `id` (ARC 0x6D) into a temporary buffer and load it
/// ([`load_record`]).
pub fn load<E: FieldEnv>(env: &mut E, m: u32, id: u32, heap: u32) {
    let size = env.callv(ARC_SIZE, &[ARC_NOGRID, id]);
    let buf = env.callv(
        HEAP_ALLOC,
        &[heap_top(heap), size, 1, FILE_NOGRID_MAPPER, 0xDD],
    );
    env.callv(ARC_READ, &[buf, ARC_NOGRID, id]);
    load_record(env, m, buf, heap);
    env.callv(HEAP_FREE, &[buf]);
}

/// C: `FUN_021BB348(m)`: unload the camera area, the rail data, the RAIL LOADER, the camera area loader and the RAIL ATTR.
pub fn unload<E: FieldEnv>(env: &mut E, m: u32) {
    let cam = env.r32(m + 0xC);
    env.callv(CAM_CLEAR, &[cam]);
    let rail = env.r32(m);
    env.callv(RAIL_CLEAR_DATA, &[rail]);
    let ld = env.r32(m + 4);
    env.callv(LOADER_UNLOAD, &[ld]);
    let camld = env.r32(m + 0x10);
    env.callv(CAMLD_UNLOAD, &[camld]);
    let ra = env.r32(m + 8);
    env.callv(ATTR_UNLOAD, &[ra]);
}

/// C: `FUN_021BB36C(m)`: the RAIL frame state, then (unless `+0x14`) the RAIL frame (`021B79F0(rail, 0)`).
pub fn frame<E: FieldEnv>(env: &mut E, m: u32) {
    let rail = env.r32(m);
    env.callv(RAIL_FRAME_STATE, &[rail]);
    if env.r32(m + 0x14) == 0 {
        let rail = env.r32(m);
        env.callv(RAIL_FRAME, &[rail, 0]);
    }
}

/// C: `FUN_021BB388(m)`: `021B79F0(m->rail, 1)` (a forced RAIL frame).
pub fn frame_forced<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let rail = env.r32(m);
    env.callv(RAIL_FRAME, &[rail, 1])
}

/// C: `FUN_021BB394(m)`: `021B7970(m->rail)`.
pub fn rail_7970<E: FieldEnv>(env: &mut E, m: u32) {
    let rail = env.r32(m);
    env.callv(0x021B_7970, &[rail]);
}

/// C: `FUN_021BB3A0(m, obj)`: `021B79AC(m->rail, obj)`.
pub fn rail_79ac<E: FieldEnv>(env: &mut E, m: u32, obj: u32) {
    let rail = env.r32(m);
    env.callv(0x021B_79AC, &[rail, obj]);
}

/// C: `FUN_021BB3AC(m, x)`: `021B79C4(m->rail, x)`.
pub fn rail_79c4<E: FieldEnv>(env: &mut E, m: u32, x: u32) {
    let rail = env.r32(m);
    env.callv(0x021B_79C4, &[rail, x]);
}

/// C: `FUN_021BB3B8(m)`: `021B79CC(m->rail)`.
pub fn rail_79cc<E: FieldEnv>(env: &mut E, m: u32) {
    let rail = env.r32(m);
    env.callv(0x021B_79CC, &[rail]);
}

/// C: `FUN_021BB3C4(m, loc)`: the attribute at a rail location (`021BB13C(m->attr, loc)`).
pub fn attr_at<E: FieldEnv>(env: &mut E, m: u32, loc: u32) -> u32 {
    let ra = env.r32(m + 8);
    env.call(ATTR_AT, &mut [CArg::V(ra), CArg::In(loc, 8)])
}

/// C: `FUN_021BB3D0(m, loc)`: `021B7D68(m->rail, loc)`.
pub fn rail_7d68<E: FieldEnv>(env: &mut E, m: u32, loc: u32) {
    let rail = env.r32(m);
    env.call(0x021B_7D68, &mut [CArg::V(rail), CArg::In(loc, 8)]);
}

/// C: `FUN_021BB3DC(m, id, f)`: `021B7C04(m->rail, id, f)`.
pub fn rail_7c04<E: FieldEnv>(env: &mut E, m: u32, id: u32, f: u32) -> u32 {
    let rail = env.r32(m);
    env.callv(0x021B_7C04, &[rail, id, f])
}

/// C: `FUN_021BB3E8(m, zone, line, z, x, loc, heap)`: load the rail file of `zone` (its no-grid record, ARC 0x6D member
/// `02013C9C(zone)`) into a temporary RAIL LOADER and let the RAIL compute the location `loc` on `line` (`021B7C58`).
#[allow(clippy::too_many_arguments)]
pub fn location_in_zone<E: FieldEnv>(
    env: &mut E,
    m: u32,
    zone: u32,
    line: u32,
    z: u32,
    x: u32,
    loc: u32,
    heap: u32,
) {
    let h = heap_top(heap);
    let id = env.callv(0x0201_3C9C, &[zone & 0xFFFF]);
    let size = env.callv(ARC_SIZE, &[ARC_NOGRID, id]);
    let buf = env.callv(HEAP_ALLOC, &[h, size, 1, FILE_NOGRID_MAPPER, 0x1C9]);
    let id = env.callv(0x0201_3C9C, &[zone & 0xFFFF]);
    env.callv(ARC_READ, &[buf, ARC_NOGRID, id]);
    let ld = env.callv(LOADER_CREATE, &[h]);
    let rail_id = u32::from(env.r16(buf));
    env.callv(LOADER_LOAD, &[ld, rail_id, h]);
    let d = env.callv(LOADER_DATA, &[ld]);
    let rail = env.r32(m);
    env.call(
        0x021B_7C58,
        &mut [
            CArg::V(rail),
            CArg::V(d),
            CArg::V(line),
            CArg::V(z),
            CArg::V(x & 0xFFFF),
            CArg::Out(loc, 8),
        ],
    );
    env.callv(LOADER_DELETE, &[ld]);
    env.callv(HEAP_FREE, &[buf]);
}

/// C: `FUN_021BB480(m)`: the RAIL system.
pub fn rail<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.r32(m)
}

#[cfg(test)]
mod tests;
