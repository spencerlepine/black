//! `field_rail.c` part 1 (overlay 21, `0x021B7884..0x021B88DC`; F3b): the RAIL system of the free-movement maps (bridges,
//! cycling roads, the 3D-camera maps): rail locations, the main RAIL WORK, its per-frame line movement and the table lookups.
//!
//! Records (offsets as the code reads them):
//! * RAIL system R (0x74, [`create`]): `+0` u16 heap, `+2` u16 rail data loaded, `+4` u16 1, `+8` -> OBJ\[n\] (RAIL WORKs of
//!   0x7C), `+0xC` n, `+0x10` the constructor's `x`, `+0x14` the current RAIL WORK (0 = none), `+0x18` "run the camera proc"
//!   flag, `+0x1C` -> the main RAIL WORK, `+0x20` the RAIL DATA (0x30, a copy of the loader's), `+0x50` the blocked-line list
//!   (0x12 u16, 0xFFFF = free).
//! * RAIL DATA (`RD`): `+0` u16 points, `+2` lines, `+4` camera records, `+6` line records, `+8 / +0xA / +0xC` the counts of the
//!   three proc tables, `+0x10` -> points (0x70 each), `+0x14` -> lines (0x48), `+0x18` -> camera records (0x24), `+0x1C` -> line
//!   records (0x24), `+0x20` -> camera procs `fn(R)`, `+0x24` -> position procs `fn(W, &pos)`, `+0x28` -> line-hit procs
//!   `fn(i, R, &hit, &loc, &pos)`, `+0x2C` the unit length. Index 0xFFFF = none; an index past the count gives the first entry
//!   (the camera / line record tables: the default records ROM 0x021D9A0C / 0x021D9A30).
//! * POINT (0x70): `+0..+0xC` four line indices, `+0x10..+0x1C` their directions (1..4), `+0x20..+0x2C` a value per line, `+0x30`
//!   the position (VecFx32), `+0x3C` the camera record index. LINE (0x48): `+0` / `+4` the end points, `+8` the direction, `+0xC`
//!   the line record, `+0x10` the camera record, `+0x14` the length (units).
//! * RAIL WORK W (0x7C): `+0` u16 kind (0 point, 1 line, 2 none), `+2` u16 in use (OBJs), `+4` -> POINT / LINE, `+8` u16 placed,
//!   `+0xC` position along the line, `+0x10` line length, `+0x14` lateral offset, `+0x18` half width, `+0x1C` unit, `+0x20`
//!   direction, `+0x24` -> RD, `+0x28` -> blocked list, `+0x2C` u8 moving, `+0x2D` s8 frame counter, `+0x2E` u16 / `+0x30` u16
//!   move parameters (`+0x30` indexes the step table ROM 0x021DE298), `+0x32` / `+0x33` / `+0x34` u8, `+0x38` the position, `+0x44`
//!   the previous position, `+0x5C` the LOCATION, `+0x64` the previous LOCATION, `+0x74` u8, `+0x78` -> R.
//! * LOCATION (8): `+0` u16 point / line index, `+2` u8 kind (0 point, 1 line), `+3` u8, `+4` s16 x (lateral), `+6` u16 z (along).
//!
//! The rest of the file (the line movement 021B88DC and its helpers, the vector math, the camera / position / line-hit procs) is
//! in other areas: every call to it (and to the NitroSDK vector functions, D3) goes through [`FieldEnv::call`] by address. The
//! proc tables are called through [`FieldEnv::call`] at the table entries' addresses. Verified by
//! `tools/check-fn.sh black-field rail --per-function` (spec `tools/rom-diff/specs/black-field/rail.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_7884,
    0x021B_791C,
    0x021B_7934,
    0x021B_7954,
    0x021B_7970,
    0x021B_79AC,
    0x021B_79C4,
    0x021B_79CC,
    0x021B_79D4,
    0x021B_79E4,
    0x021B_79F0,
    0x021B_7A84,
    0x021B_7C04,
    0x021B_7C38,
    0x021B_7C58,
    0x021B_7C90,
    0x021B_7CD8,
    0x021B_7D38,
    0x021B_7D50,
    0x021B_7D68,
    0x021B_7DC0,
    0x021B_7DE0,
    0x021B_7E60,
    0x021B_7ED4,
    0x021B_7EE4,
    0x021B_8028,
    0x021B_8038,
    0x021B_8040,
    0x021B_8048,
    0x021B_8050,
    0x021B_80D8,
    0x021B_80E0,
    0x021B_810C,
    0x021B_821C,
    0x021B_8220,
    0x021B_8224,
    0x021B_8230,
    0x021B_823C,
    0x021B_8248,
    0x021B_824C,
    0x021B_8250,
    0x021B_82F0,
    0x021B_82F4,
    0x021B_82F8,
    0x021B_82FC,
    0x021B_8300,
    0x021B_8304,
    0x021B_8308,
    0x021B_830C,
    0x021B_8314,
    0x021B_8328,
    0x021B_833C,
    0x021B_834C,
    0x021B_835C,
    0x021B_836C,
    0x021B_8378,
    0x021B_83B0,
    0x021B_83FC,
    0x021B_8484,
    0x021B_8514,
    0x021B_854C,
    0x021B_8658,
    0x021B_8674,
    0x021B_868C,
    0x021B_86A4,
    0x021B_87BC,
    0x021B_87C8,
    0x021B_87D8,
    0x021B_87FC,
    0x021B_8820,
    0x021B_8848,
    0x021B_8870,
    0x021B_8894,
    0x021B_88B8,
];

/// Address of the string `"field_rail.c"`.
pub const FILE_RAIL: u32 = 0x021D_E2A0;
/// "No index" in the rail tables.
pub const NONE: u32 = 0xFFFF;
/// Size of a RAIL WORK.
pub const WORK_SIZE: u32 = 0x7C;

/// ROM `0x021D9A00..0x021D9A54`: the up vector (0, 1.0, 0) and the default camera / line records.
pub const ROM_DEFAULTS: [u32; 21] = [
    0, 0x1000, 0, 0xFFFF, 0, 0, 0, 0, 0, 0, 0, 0, 0xFFFF, 0xFF, 0, 0, 0, 0, 0, 0, 0,
];
/// Address of [`ROM_DEFAULTS`].
pub const ROM_DEFAULTS_ADDR: u32 = 0x021D_9A00;
/// The default camera record (`021B8820` past the count).
pub const DEFAULT_CAM_REC: u32 = 0x021D_9A0C;
/// The default line record (`021B8848` past the count).
pub const DEFAULT_LINE_REC: u32 = 0x021D_9A30;
/// ROM `0x021DE298`: the frame-counter step per move speed (`W + 0x30`).
pub const STEP_TABLE: [u8; 5] = [1, 2, 4, 8, 16];

// field_rail.c, other areas (rail_move): reached by address
const OBJ_INIT: u32 = 0x021B_9164;
const OBJ_RESET: u32 = 0x021B_919C;
const OBJ_IN_USE: u32 = 0x021B_91A8;
const W_ACTIVE: u32 = 0x021B_91AC;
const CAM_OF: u32 = 0x021B_91C8;
const POS_OF: u32 = 0x021B_91F8;
const LOC_OF: u32 = 0x021B_9238;
const LINE_LEN: u32 = 0x021B_9278;
const LINE_WIDTH: u32 = 0x021B_9280;
const RESOLVE: u32 = 0x021B_9368;
const DIR_BACK: u32 = 0x021B_93D4;
const DIR_RIGHT: u32 = 0x021B_9404;
const DIR_LEFT: u32 = 0x021B_9434;
const BLK_CLEAR: u32 = 0x021B_9464;
const BLK_ADD: u32 = 0x021B_9478;
const BLK_REMOVE: u32 = 0x021B_949C;
const BLK_HAS: u32 = 0x021B_94C0;
const LINE_IF_OPEN: u32 = 0x021B_9510;
const CAN_ENTER: u32 = 0x021B_95C0;
const CORNER_OK: u32 = 0x021B_963C;
const LINE_MOVE: u32 = 0x021B_88DC;
const KIND_CHANGED: u32 = 0x021B_977C;
const SEG_HIT: u32 = 0x021B_9828;
// NitroSDK vector functions (D3 math hooks)
const VEC_SUB: u32 = 0x0207_CA00;
const VEC_CROSS: u32 = 0x0207_CAE8;
const VEC_NORMALIZE: u32 = 0x0207_CCA4;
const VEC_DISTANCE: u32 = 0x0207_CF3C;

/// A word of the rail data: the ROM default records are read from [`ROM_DEFAULTS`], anything else from memory.
pub(crate) fn rd32<E: FieldEnv>(env: &mut E, a: u32) -> u32 {
    if (ROM_DEFAULTS_ADDR..ROM_DEFAULTS_ADDR + 4 * ROM_DEFAULTS.len() as u32).contains(&a) {
        return ROM_DEFAULTS[((a - ROM_DEFAULTS_ADDR) / 4) as usize];
    }
    env.r32(a)
}

/// `x / 16` rounded towards zero (the original's `(x + ((x >> 3) >>> 28)) >> 4`).
pub fn div16(x: i32) -> i32 {
    x.wrapping_add(((x >> 3) as u32 >> 28) as i32) >> 4
}

/// `|x|` as the original computes it (`negs`; `i32::MIN` stays itself).
pub(crate) fn abs(x: i32) -> i32 {
    if x < 0 {
        x.wrapping_neg()
    } else {
        x
    }
}

/// `memcmp(a, b, 8) != 0` on two 8-byte LOCATIONs (`0203EFC8`, a pure library helper the port inlines).
fn loc_differs(a: &[u8; 8], b: &[u8; 8]) -> bool {
    a != b
}

/// The 8 bytes of the LOCATION at `a`.
fn loc_bytes<E: FieldEnv>(env: &mut E, a: u32) -> [u8; 8] {
    let mut b = [0u8; 8];
    for (i, x) in b.iter_mut().enumerate() {
        *x = env.r8(a + i as u32);
    }
    b
}

fn line_len<E: FieldEnv>(env: &mut E, ln: u32) -> i32 {
    env.callv(LINE_LEN, &[ln]) as i32
}

fn line_width<E: FieldEnv>(env: &mut E, ln: u32, pos: i32, len: i32, rd: u32) -> i32 {
    env.callv(LINE_WIDTH, &[ln, pos as u32, len as u32, rd]) as i32
}

/// C: `FUN_021B7884(heap, n, x)`: a new RAIL system with `n` RAIL WORK OBJs and the main RAIL WORK (`+2` = 1).
pub fn create<E: FieldEnv>(env: &mut E, heap: u32, n: u32, x: u32) -> u32 {
    let r = env.callv(HEAP_ALLOC, &[heap, 0x74, 1, FILE_RAIL, 0x12E]);
    env.w16(r, heap as u16);
    env.w16(r + 2, 0);
    env.w16(r + 4, 1);
    env.w32(r + 0x10, x);
    env.w32(r + 0x14, 0);
    let objs = env.callv(
        HEAP_ALLOC,
        &[heap, n.wrapping_mul(WORK_SIZE), 1, FILE_RAIL, 0x137],
    );
    env.w32(r + 8, objs);
    env.w32(r + 0xC, n);
    let mut i = 0u32;
    while i < n {
        let o = env.r32(r + 8).wrapping_add(WORK_SIZE.wrapping_mul(i));
        env.callv(OBJ_INIT, &[o, r + 0x20, r]);
        i += 1;
    }
    let w = env.callv(HEAP_ALLOC, &[heap, WORK_SIZE, 1, FILE_RAIL, 0x13F]);
    env.w32(r + 0x1C, w);
    env.callv(OBJ_INIT, &[w, r + 0x20, r]);
    let w = env.r32(r + 0x1C);
    env.w16(w + 2, 1);
    r
}

/// C: `FUN_021B791C(r)`: free the main RAIL WORK, the OBJs and the system.
pub fn delete<E: FieldEnv>(env: &mut E, r: u32) {
    let w = env.r32(r + 0x1C);
    env.callv(HEAP_FREE, &[w]);
    let objs = env.r32(r + 8);
    env.callv(HEAP_FREE, &[objs]);
    env.callv(HEAP_FREE, &[r]);
}

/// C: `FUN_021B7934(r, data)`: take the RAIL DATA `data` (copy of 0x30 bytes) and clear the blocked lines.
pub fn set_data<E: FieldEnv>(env: &mut E, r: u32, data: u32) {
    env.w16(r + 2, 1);
    env.callv(BLK_CLEAR, &[r + 0x50]);
    copy_data(env, r + 0x20, data);
}

/// C: `FUN_021B7954(r)`: drop the RAIL DATA (zero it) and clear the blocked lines.
pub fn clear_data<E: FieldEnv>(env: &mut E, r: u32) {
    env.w16(r + 2, 0);
    clear_rd(env, r + 0x20);
    env.callv(BLK_CLEAR, &[r + 0x50]);
}

/// C: `FUN_021B7970(r)`: take the first free OBJ (`+2` = 1); returns it (the original writes `+2` of address 0 when every
/// OBJ is in use).
pub fn obj_alloc<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    let mut found = 0u32;
    let mut i = 0i32;
    while i < env.r32(r + 0xC) as i32 {
        let o = env
            .r32(r + 8)
            .wrapping_add((i as u32).wrapping_mul(WORK_SIZE));
        if env.callv(OBJ_IN_USE, &[o]) == 0 {
            found = env
                .r32(r + 8)
                .wrapping_add((i as u32).wrapping_mul(WORK_SIZE));
            break;
        }
        i += 1;
    }
    env.w16(found.wrapping_add(2), 1);
    found
}

/// C: `FUN_021B79AC(r, obj)`: reset the OBJ and mark it free.
pub fn obj_free<E: FieldEnv>(env: &mut E, r: u32, obj: u32) {
    env.callv(OBJ_INIT, &[obj, r + 0x20, r]);
    env.w16(obj + 2, 0);
}

/// C: `FUN_021B79C4(r, w)`: make `w` the current RAIL WORK and request the camera proc (`+0x18` = 1).
pub fn set_current<E: FieldEnv>(env: &mut E, r: u32, w: u32) {
    env.w32(r + 0x14, w);
    env.w32(r + 0x18, 1);
}

/// C: `FUN_021B79CC(r)`: no current RAIL WORK.
pub fn clear_current<E: FieldEnv>(env: &mut E, r: u32) {
    env.w32(r + 0x14, 0);
}

/// C: `FUN_021B79D4(r, out)`: the position of the current RAIL WORK (if any) to `out`.
pub fn current_pos<E: FieldEnv>(env: &mut E, r: u32, out: u32) {
    let w = env.r32(r + 0x14);
    if w != 0 {
        get_pos(env, w, out);
    }
}

/// C: `FUN_021B79E4(r)`: `r` itself when no rail data is loaded, else the u16 at `+4`.
pub fn frame_state<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    if env.r16(r + 2) == 0 {
        return r;
    }
    u32::from(env.r16(r + 4))
}

/// The camera proc of the camera record of a RAIL WORK's point (kind 0, `+0x3C`) or line (kind 1, `+0x10`), 0 for none.
fn cam_proc_of<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let idx = match env.r16(w) {
        0 => {
            let p = env.r32(w + 4);
            env.r32(p + 0x3C)
        }
        1 => {
            let p = env.r32(w + 4);
            env.r32(p + 0x10)
        }
        _ => return 0,
    };
    if idx == NONE {
        return 0;
    }
    let rd = env.r32(w + 0x24);
    let rec = cam_rec(env, rd, idx);
    let first = rd32(env, rec);
    let rd = env.r32(w + 0x24);
    cam_proc(env, rd, first)
}

/// C: `FUN_021B79F0(r, force)`: one frame: with rail data, a current RAIL WORK that is active and has moved (or `+0x18`
/// set, or `force`), run the camera proc of its camera record with `r`; clears `+0x18`. Returns 1 when it ran.
pub fn frame<E: FieldEnv>(env: &mut E, r: u32, force: u32) -> u32 {
    let w = env.r32(r + 0x14);
    if env.r16(r + 2) == 0 || env.r16(r + 4) == 0 || w == 0 {
        return 0;
    }
    if env.callv(W_ACTIVE, &[w]) == 0 {
        return 0;
    }
    if moved(env, w) == 0 && env.r32(r + 0x18) == 0 && force == 0 {
        return 0;
    }
    let f = cam_proc_of(env, w);
    if f != 0 {
        env.callv(f & !1, &[r]);
    }
    env.w32(r + 0x18, 0);
    1
}

/// C: `FUN_021B7A84(r, a, b, loc, pos)`: the segment `a`..`b` against every open line of the rail data: each line whose
/// line record has a line-hit proc that accepts the hit point gives a LOCATION / position; the one nearest to `a` is
/// written to `loc` / `pos`. Returns 1 when one was found.
pub fn line_hit<E: FieldEnv>(env: &mut E, r: u32, a: u32, b: u32, loc: u32, pos: u32) -> u32 {
    let up = [0u8, 0, 0, 0, 0, 0x10, 0, 0, 0, 0, 0, 0];
    let mut la = [0u8; 12];
    let mut lb = [0u8; 12];
    for i in 0..12 {
        la[i] = env.r8(a + i as u32);
        lb[i] = env.r8(b + i as u32);
    }
    let mut found = 0u32;
    let mut best_d = 0i32;
    let mut best_loc = [0u16; 4];
    let mut best_pos = [0u32; 3];
    let rd = r + 0x20;
    let mut i = 0i32;
    while i < i32::from(env.r16(r + 0x22)) {
        let ln = env.call(
            LINE_IF_OPEN,
            &mut [CArg::V(r + 0x50), CArg::V(rd), CArg::V(i as u32)],
        );
        if ln != 0 {
            let ia = env.r32(ln);
            let pa = point_rec(env, rd, ia);
            let ib = env.r32(ln + 4);
            let pb = point_rec(env, rd, ib);
            let i3 = env.r32(ln + 0xC);
            let t = line_rec_rec(env, rd, i3);
            let mut d = [0u8; 12];
            let mut n = [0u8; 12];
            let mut hit = [0u8; 12];
            env.call(
                VEC_SUB,
                &mut [
                    CArg::In(pa + 0x30, 12),
                    CArg::In(pb + 0x30, 12),
                    CArg::OutBuf(&mut d),
                ],
            );
            env.call(
                VEC_CROSS,
                &mut [CArg::InBuf(&d), CArg::InBuf(&up), CArg::OutBuf(&mut n)],
            );
            let ok = env.call(
                SEG_HIT,
                &mut [
                    CArg::InBuf(&d),
                    CArg::InBuf(&n),
                    CArg::In(pb + 0x30, 12),
                    CArg::InBuf(&la),
                    CArg::InBuf(&lb),
                    CArg::OutBuf(&mut hit),
                ],
            );
            if ok != 0 {
                let t0 = rd32(env, t);
                let f = hit_proc(env, rd, t0);
                if f != 0
                    && env.call(
                        f & !1,
                        &mut [
                            CArg::V(i as u32),
                            CArg::V(r),
                            CArg::InBuf(&hit),
                            CArg::Out(loc, 8),
                            CArg::Out(pos, 12),
                        ],
                    ) != 0
                {
                    let dist =
                        env.call(VEC_DISTANCE, &mut [CArg::InBuf(&la), CArg::In(pos, 12)]) as i32;
                    if found == 0 || dist < best_d {
                        for (k, v) in best_loc.iter_mut().enumerate() {
                            *v = env.r16(loc + 2 * k as u32);
                        }
                        for (k, v) in best_pos.iter_mut().enumerate() {
                            *v = env.r32(pos + 4 * k as u32);
                        }
                        best_d = dist;
                        found = 1;
                    }
                }
            }
        }
        i += 1;
    }
    if found != 0 {
        for (k, v) in best_loc.iter().enumerate() {
            env.w16(loc + 2 * k as u32, *v);
        }
        for (k, v) in best_pos.iter().enumerate() {
            env.w32(pos + 4 * k as u32, *v);
        }
    }
    found
}

/// C: `FUN_021B7C04(r, id, f)`: `f == 0`: block line `id` (unless it is blocked already); else unblock it. Returns the
/// answer of the list call (1 when it was blocked already).
pub fn block_line<E: FieldEnv>(env: &mut E, r: u32, id: u32, f: u32) -> u32 {
    let id = id & 0xFFFF;
    if f == 0 {
        let has = env.callv(BLK_HAS, &[r + 0x50, id]);
        if has != 0 {
            return has;
        }
        return env.callv(BLK_ADD, &[r + 0x50, id]);
    }
    env.callv(BLK_REMOVE, &[r + 0x50, id])
}

/// C: `FUN_021B7C38(r)`: the unit length of the rail data as fx32 (`RD + 0x2C` * 16, `FX_Mul(u << 16, 1.0)` rounded).
pub fn unit_fx<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    let u = i64::from(env.r32(r + 0x4C) as i32);
    ((u << 16).wrapping_add(0x800) >> 12) as u32
}

/// C: `FUN_021B7C58(r, data, line, z, x, out)`: the LOCATION of line `line` at `z` and lateral `x` in the rail data `data`
/// (the main RAIL WORK is set there with `data` in place of the system's rail data).
#[allow(clippy::too_many_arguments)]
pub fn loc_from_line<E: FieldEnv>(
    env: &mut E,
    r: u32,
    data: u32,
    line: u32,
    z: u32,
    x: u32,
    out: u32,
) {
    let w = env.r32(r + 0x1C);
    env.w32(w + 0x24, data);
    env.w16(out, line as u16);
    env.w8(out + 2, 1);
    env.w16(out + 4, 0);
    env.w16(out + 6, z as u16);
    set_loc(env, w, out);
    let half = div16(env.r32(w + 0x18) as i32);
    env.w16(out + 4, (x as i32).wrapping_sub(half) as u16);
    env.w32(w + 0x24, r + 0x20);
}

/// C: `FUN_021B7C90(r, loc, dir, out)`: place the main RAIL WORK at `loc`, move it one step in `dir`; its position to `out`.
/// Returns 0 when the LOCATION did not change, else whether the new one is valid ([`loc_valid`]).
pub fn move_loc<E: FieldEnv>(env: &mut E, r: u32, loc: u32, dir: u32, out: u32) -> u32 {
    let w = env.r32(r + 0x1C);
    set_loc(env, w, loc);
    let w = env.r32(r + 0x1C);
    start_move(env, w, 4, dir);
    let w = env.r32(r + 0x1C);
    move_frame(env, w);
    let w = env.r32(r + 0x1C);
    get_pos(env, w, out);
    let w = env.r32(r + 0x1C);
    let a = loc_bytes(env, loc);
    let b = loc_bytes(env, w + 0x5C);
    if !loc_differs(&a, &b) {
        return 0;
    }
    let w = env.r32(r + 0x1C);
    loc_valid(env, w, w + 0x5C)
}

/// C: `FUN_021B7CD8(r, loc, dir, out)`: [`move_loc`] that writes the new LOCATION to `out`.
pub fn step_loc<E: FieldEnv>(env: &mut E, r: u32, loc: u32, dir: u32, out: u32) -> u32 {
    let old = loc_bytes(env, loc);
    let w = env.r32(r + 0x1C);
    set_loc(env, w, loc);
    let w = env.r32(r + 0x1C);
    start_move(env, w, 4, dir);
    let w = env.r32(r + 0x1C);
    move_frame(env, w);
    let w = env.r32(r + 0x1C);
    get_loc(env, w, out);
    let new = loc_bytes(env, out);
    if !loc_differs(&old, &new) {
        return 0;
    }
    let w = env.r32(r + 0x1C);
    loc_valid(env, w, out)
}

/// C: `FUN_021B7D38(r, loc, out)`: the position of `loc` (main RAIL WORK placed there).
pub fn loc_pos<E: FieldEnv>(env: &mut E, r: u32, loc: u32, out: u32) {
    let w = env.r32(r + 0x1C);
    set_loc(env, w, loc);
    let w = env.r32(r + 0x1C);
    get_pos(env, w, out);
}

/// C: `FUN_021B7D50(r, loc)`: the half width at `loc`, in units (`W + 0x18` / 16).
pub fn loc_width<E: FieldEnv>(env: &mut E, r: u32, loc: u32) -> u32 {
    let w = env.r32(r + 0x1C);
    set_loc(env, w, loc);
    let w = env.r32(r + 0x1C);
    div16(env.r32(w + 0x18) as i32) as u32
}

/// C: `FUN_021B7D68(r, loc)`: place the main RAIL WORK at `loc` and run its camera proc with it as the current one.
pub fn run_loc_proc<E: FieldEnv>(env: &mut E, r: u32, loc: u32) {
    let w = env.r32(r + 0x1C);
    set_loc(env, w, loc);
    let old = env.r32(r + 0x14);
    let w = env.r32(r + 0x1C);
    env.w32(r + 0x14, w);
    let f = cam_proc_of(env, w);
    if f != 0 {
        env.callv(f & !1, &[r]);
    }
    env.w32(r + 0x14, old);
}

/// C: `FUN_021B7DC0(w, out)`: the LOCATION of `w` (`+0x5C`) to `out`.
pub fn get_loc<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    for k in 0..4 {
        let v = env.r16(w + 0x5C + 2 * k);
        env.w16(out + 2 * k, v);
    }
}

/// C: `FUN_021B7DE0(w, loc)`: 1 when `loc` is inside the rail data of `w` (a point index in range; a line index in range
/// with `|x| <= width` and `z <= length`).
pub fn loc_valid<E: FieldEnv>(env: &mut E, w: u32, loc: u32) -> u32 {
    let rd = env.r32(w + 0x24);
    if env.r8(loc + 2) == 0 {
        return (env.r16(loc) < env.r16(rd)) as u32;
    }
    let idx = u32::from(env.r16(loc));
    if idx >= u32::from(env.r16(rd + 2)) {
        return 0;
    }
    let ln = env.r32(rd + 0x14).wrapping_add(idx * 0x48);
    let z = i32::from(env.r16(loc + 6)) << 4;
    let x = i32::from(env.r16(loc + 4) as i16) << 4;
    let len = line_len(env, ln);
    let rd = env.r32(w + 0x24);
    let wid = line_width(env, ln, z, len, rd);
    if wid.wrapping_neg() > x || x > wid {
        return 0;
    }
    if z > len || z < 0 {
        return 0;
    }
    1
}

/// C: `FUN_021B7E60(p, dir, loc, pos)`: the step of the cached LOCATION `p + 0x5C` in `dir` (cached in `p + 0x6C` /
/// `p + 0x50` until `dir` changes; [`step_loc`] / [`loc_pos`] on the system `p + 0x78`); copies them to `loc` / `pos`
/// when non-null. Returns the cached validity byte `p + 0x75`.
pub fn cached_step<E: FieldEnv>(env: &mut E, p: u32, dir: u32, loc: u32, pos: u32) -> u32 {
    if env.r8(p + 0x74) == 0 || dir != u32::from(env.r8(p + 0x76)) {
        let r = env.r32(p + 0x78);
        let v = step_loc(env, r, p + 0x5C, dir, p + 0x6C);
        env.w8(p + 0x75, v as u8);
        let r = env.r32(p + 0x78);
        loc_pos(env, r, p + 0x6C, p + 0x50);
        env.w8(p + 0x74, 1);
        env.w8(p + 0x76, dir as u8);
    }
    if loc != 0 {
        env.mem().copy(loc, p + 0x6C, 8);
    }
    if pos != 0 {
        for k in 0..3 {
            let v = env.r32(p + 0x50 + 4 * k);
            env.w32(pos + 4 * k, v);
        }
    }
    u32::from(env.r8(p + 0x75))
}

/// C: `FUN_021B7ED4(w, out)`: the previous position (`+0x44`) to `out`.
pub fn old_pos<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    for k in 0..3 {
        let v = env.r32(w + 0x44 + 4 * k);
        env.w32(out + 4 * k, v);
    }
}

/// C: `FUN_021B7EE4(w, loc)`: place `w` at the LOCATION `loc`: reset it, find its line (a point uses the first line of
/// its list), set the line state ([`set_line`]), the position, the LOCATION and the previous ones.
///
/// For a point whose first listed line ends at neither end in that point, the original uses its uninitialised stack
/// words as the line values (UNVERIFIED: the port uses 0; the rail data never has such a point).
pub fn set_loc<E: FieldEnv>(env: &mut E, w: u32, loc: u32) {
    let rd = env.r32(w + 0x24);
    env.callv(OBJ_RESET, &[w, rd]);
    let mut ln = 0u32;
    let mut pos = 0i32;
    let mut lat = 0i32;
    let mut dir = 0u32;
    if env.r8(loc + 2) == 0 {
        let rd = env.r32(w + 0x24);
        env.w16(w, 0);
        let idx = u32::from(env.r16(loc));
        let pt = env.r32(rd + 0x10).wrapping_add(idx.wrapping_mul(0x70));
        env.w32(w + 4, pt);
        let v = env.r32(pt + 0x20);
        env.w32(w + 0x18, v);
        for i in 0..4u32 {
            let pt = env.r32(w + 4);
            let li = env.r32(pt + 4 * i);
            if li == NONE || ln != 0 {
                continue;
            }
            let rd = env.r32(w + 0x24);
            let lines = env.r32(rd + 0x14);
            ln = lines.wrapping_add(li.wrapping_mul(0x48));
            let idx = u32::from(env.r16(loc));
            if env.r32(ln) == idx {
                pos = 0;
                lat = 0;
                dir = env.r32(pt + 4 * i + 0x10);
            } else if env.r32(ln + 4) == idx {
                pos = line_len(env, ln);
                lat = 0;
                let pt = env.r32(w + 4);
                dir = env.r32(pt + 4 * i + 0x10);
            }
        }
    } else {
        let rd = env.r32(w + 0x24);
        let idx = u32::from(env.r16(loc));
        ln = env.r32(rd + 0x14).wrapping_add(idx * 0x48);
        let mut z = u32::from(env.r16(loc + 6));
        if z >= env.r32(ln + 0x14) {
            z = z.wrapping_sub(1);
        }
        pos = (z << 4) as i32;
        lat = i32::from(env.r16(loc + 4) as i16) << 4;
        dir = u32::from(env.r8(loc + 3));
    }
    let len = line_len(env, ln);
    let rd = env.r32(w + 0x24);
    let wid = line_width(env, ln, pos, len, rd);
    set_line(
        env, w, ln, dir, pos as u32, lat as u32, len as u32, wid as u32,
    );
    env.w8(w + 0x33, 1);
    env.w8(w + 0x32, dir as u8);
    env.w16(w + 8, 1);
    env.call(POS_OF, &mut [CArg::V(w), CArg::Out(w + 0x38, 12)]);
    for k in 0..4 {
        let v = env.r16(loc + 2 * k);
        env.w16(w + 0x5C + 2 * k, v);
    }
    for k in 0..4 {
        let v = env.r16(w + 0x5C + 2 * k);
        env.w16(w + 0x64 + 2 * k, v);
    }
    for k in 0..3 {
        let v = env.r32(w + 0x38 + 4 * k);
        env.w32(w + 0x44 + 4 * k, v);
    }
}

/// C: `FUN_021B8028(w, out)`: the position (`+0x38`) to `out`.
pub fn get_pos<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    for k in 0..3 {
        let v = env.r32(w + 0x38 + 4 * k);
        env.w32(out + 4 * k, v);
    }
}

/// C: `FUN_021B8038(w)`: the byte `+0x32` (the direction of the last placement).
pub fn dir_byte<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    u32::from(env.r8(w + 0x32))
}

/// C: `FUN_021B8040(w)`: the byte `+0x2C` (moving).
pub fn moving<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    u32::from(env.r8(w + 0x2C))
}

/// C: `FUN_021B8048(w)`: the byte `+0x34` (moved this frame).
pub fn moved<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    u32::from(env.r8(w + 0x34))
}

/// C: `FUN_021B8050(w, out)`: the unit direction of travel (position 16 further along minus the position, normalised; past
/// the line's end through [`RESOLVE`]) as three s16 to `out`.
pub fn dir_vec<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    let lat = env.r32(w + 0x14);
    env.w32(w + 0x14, 0);
    let a = env.mem().push_local(12);
    let bv = env.mem().push_local(12);
    let l = env.mem().push_local(8);
    env.call(POS_OF, &mut [CArg::V(w), CArg::Out(a, 12)]);
    let cur = env.r32(w + 0xC) as i32;
    env.w32(w + 0x14, lat);
    let len = env.r32(w + 0x10) as i32;
    let next = cur.wrapping_add(16);
    if next < len {
        env.w32(w + 0xC, next as u32);
        env.w32(w + 0x14, 0);
        env.call(POS_OF, &mut [CArg::V(w), CArg::Out(bv, 12)]);
        env.w32(w + 0x14, lat);
        env.w32(w + 0xC, cur as u32);
    } else {
        env.call(LOC_OF, &mut [CArg::V(w), CArg::Out(l, 8)]);
        env.w16(l + 4, 0);
        let r = env.r32(w + 0x78);
        let p = env.r32(w + 4);
        let x = env.r32(p + 8);
        env.call(
            RESOLVE,
            &mut [
                CArg::V(r),
                CArg::In(l, 8),
                CArg::V(x),
                CArg::V(next.wrapping_sub(len) as u32),
                CArg::Out(bv, 12),
            ],
        );
    }
    env.call(
        VEC_SUB,
        &mut [CArg::In(bv, 12), CArg::In(a, 12), CArg::Out(bv, 12)],
    );
    env.call(VEC_NORMALIZE, &mut [CArg::In(bv, 12), CArg::Out(bv, 12)]);
    let x = env.r32(bv);
    let y = env.r32(bv + 4);
    let z = env.r32(bv + 8);
    env.mem().pop_local(l);
    env.mem().pop_local(bv);
    env.mem().pop_local(a);
    env.w16(out, x as u16);
    env.w16(out + 2, y as u16);
    env.w16(out + 4, z as u16);
}

/// C: `FUN_021B80D8(w)`: the word `+8` of its POINT / LINE.
pub fn rec_word8<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let p = env.r32(w + 4);
    env.r32(p + 8)
}

/// C: `FUN_021B80E0(w, speed, dir)`: start a move (unless one runs): `+0x2C` = 1, `+0x2E` = dir, `+0x30` = speed, counter 0.
/// Returns 1 when started.
pub fn start_move<E: FieldEnv>(env: &mut E, w: u32, speed: u32, dir: u32) -> u32 {
    if env.r8(w + 0x2C) != 0 {
        return 0;
    }
    env.w8(w + 0x2C, 1);
    env.w16(w + 0x2E, dir as u16);
    env.w16(w + 0x30, speed as u16);
    env.w8(w + 0x2D, 0);
    1
}

/// C: `FUN_021B810C(w)`: one frame of a move: on a line the line movement ([`LINE_MOVE`]) gives the new direction (0 =
/// blocked); the frame counter advances by the step of the speed (ROM 0x021DE298); at 16 or when blocked the move ends:
/// the previous LOCATION / position are updated and the position along the line snaps to a multiple of 16.
pub fn move_frame<E: FieldEnv>(env: &mut E, w: u32) {
    let r = env.r32(w + 0x78);
    if env.r16(r + 2) == 0 || env.r16(w + 8) == 0 {
        return;
    }
    if env.callv(W_ACTIVE, &[w]) == 0 {
        return;
    }
    env.w8(w + 0x34, 0);
    if env.r8(w + 0x2C) == 0 {
        return;
    }
    let speed = env.r16(w + 0x30);
    let kind = env.r16(w);
    let step = STEP_TABLE[speed as usize] as i8;
    let c = (env.r8(w + 0x2D) as i8).wrapping_add(step);
    let d = u32::from(env.r16(w + 0x2E));
    env.w8(w + 0x2D, c as u8);
    let mut nd = 0u32;
    if kind == 1 {
        nd = env.call(
            LINE_MOVE,
            &mut [CArg::Out(w, 2), CArg::V(d), CArg::V(u32::from(step as u8))],
        );
    }
    if nd != 0 {
        env.w8(w + 0x32, nd as u8);
        let speed = env.r16(w + 0x30);
        env.w8(w + 0x33, speed as u8);
        env.w8(w + 0x34, 1);
        env.call(POS_OF, &mut [CArg::V(w), CArg::Out(w + 0x38, 12)]);
    }
    if kind != env.r16(w) {
        env.callv(KIND_CHANGED, &[w]);
    }
    if (env.r8(w + 0x2D) as i8) < 16 && nd != 0 {
        return;
    }
    env.w8(w + 0x2C, 0);
    env.w8(w + 0x74, 0);
    for k in 0..4 {
        let v = env.r16(w + 0x5C + 2 * k);
        env.w16(w + 0x64 + 2 * k, v);
    }
    env.call(LOC_OF, &mut [CArg::V(w), CArg::Out(w + 0x5C, 8)]);
    for k in 0..3 {
        let v = env.r32(w + 0x38 + 4 * k);
        env.w32(w + 0x44 + 4 * k, v);
    }
    let p = env.r32(w + 0xC) as i32;
    let rem = p.wrapping_rem(16);
    if rem == 0 {
        return;
    }
    if rem > 8 {
        env.w32(w + 0xC, p.wrapping_add(16 - rem) as u32);
    } else {
        env.w32(w + 0xC, p.wrapping_sub(rem) as u32);
    }
}

/// C: `FUN_021B821C(x)`: the word `+0x10`.
pub fn word10<E: FieldEnv>(env: &mut E, x: u32) -> u32 {
    env.r32(x + 0x10)
}

/// C: `FUN_021B8220(x)`: the word `+0x14`.
pub fn word14<E: FieldEnv>(env: &mut E, x: u32) -> u32 {
    env.r32(x + 0x14)
}

/// C: `FUN_021B8224(r, i)`: point `i` of the system's rail data ([`point_rec`] on `r + 0x20`).
pub fn sys_point<E: FieldEnv>(env: &mut E, r: u32, i: u32) -> u32 {
    point_rec(env, r + 0x20, i)
}

/// C: `FUN_021B8230(r, i)`: line `i` of the system's rail data.
pub fn sys_line<E: FieldEnv>(env: &mut E, r: u32, i: u32) -> u32 {
    line_rec(env, r + 0x20, i)
}

/// C: `FUN_021B823C(r, i)`: line record `i` of the system's rail data.
pub fn sys_line_rec<E: FieldEnv>(env: &mut E, r: u32, i: u32) -> u32 {
    line_rec_rec(env, r + 0x20, i)
}

/// C: `FUN_021B8248(r)`: the unit length of the system's rail data (`+0x4C`).
pub fn sys_unit<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    env.r32(r + 0x4C)
}

/// C: `FUN_021B824C(r)`: the main RAIL WORK (`+0x1C`).
pub fn main_work<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    env.r32(r + 0x1C)
}

/// C: `FUN_021B8250(r, loc, out)`: `loc` clamped to its line (z below the length, `|x|` within the width) to `out`.
pub fn clamp_loc<E: FieldEnv>(env: &mut E, r: u32, loc: u32, out: u32) {
    let l: [u16; 4] = [
        env.r16(loc),
        env.r16(loc + 2),
        env.r16(loc + 4),
        env.r16(loc + 6),
    ];
    let lines = env.r32(r + 0x34);
    for (k, v) in l.iter().enumerate() {
        env.w16(out + 2 * k as u32, *v);
    }
    let ln = lines.wrapping_add(u32::from(l[0]) * 0x48);
    let z = i32::from(l[3]) << 4;
    let x = i32::from(l[2] as i16) << 4;
    let len = line_len(env, ln);
    let wid = line_width(env, ln, z, len, r + 0x20);
    if z >= len {
        env.w16(out + 6, div16(len.wrapping_sub(1)) as u16);
    }
    if abs(x) > wid {
        if x > 0 {
            env.w16(out + 4, div16(wid) as u16);
        } else {
            env.w16(out + 4, div16(wid.wrapping_neg()) as u16);
        }
    }
}

/// C: `FUN_021B82F0(w)`: the kind (u16 `+0`).
pub fn kind<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    u32::from(env.r16(w))
}

/// C: `FUN_021B82F4(w)`: the POINT / LINE (`+4`).
pub fn rec<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 4)
}

/// C: `FUN_021B82F8(w)`: the position along the line (`+0xC`).
pub fn along<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 0xC)
}

/// C: `FUN_021B82FC(w)`: the line length (`+0x10`).
pub fn length<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 0x10)
}

/// C: `FUN_021B8300(w)`: the lateral offset (`+0x14`).
pub fn lateral<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 0x14)
}

/// C: `FUN_021B8304(w)`: the half width (`+0x18`).
pub fn width<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 0x18)
}

/// C: `FUN_021B8308(w)`: the unit (`+0x1C`).
pub fn unit<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 0x1C)
}

/// C: `FUN_021B830C(w)`: thunk to `021B91C8(w)` (the camera record of the RAIL WORK).
pub fn cam_of<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.callv(CAM_OF, &[w])
}

/// C: `FUN_021B8314(w, out)`: the position without the lateral offset.
pub fn pos_no_lat<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    let lat = env.r32(w + 0x14);
    env.w32(w + 0x14, 0);
    env.call(POS_OF, &mut [CArg::V(w), CArg::Out(out, 12)]);
    env.w32(w + 0x14, lat);
}

/// C: `FUN_021B8328(w, along, out)`: the position at `along` on the current line.
pub fn pos_at<E: FieldEnv>(env: &mut E, w: u32, along: u32, out: u32) {
    let old = env.r32(w + 0xC);
    env.w32(w + 0xC, along);
    env.call(POS_OF, &mut [CArg::V(w), CArg::Out(out, 12)]);
    env.w32(w + 0xC, old);
}

/// C: `FUN_021B833C(w)`: the start point of its line.
pub fn line_start<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let rd = env.r32(w + 0x24);
    let p = env.r32(w + 4);
    let i = env.r32(p);
    point_rec(env, rd, i)
}

/// C: `FUN_021B834C(w)`: the end point of its line.
pub fn line_end<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let rd = env.r32(w + 0x24);
    let p = env.r32(w + 4);
    let i = env.r32(p + 4);
    point_rec(env, rd, i)
}

/// C: `FUN_021B835C(w)`: the line record of its line.
pub fn line_record<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let rd = env.r32(w + 0x24);
    let p = env.r32(w + 4);
    let i = env.r32(p + 0xC);
    line_rec_rec(env, rd, i)
}

/// C: `FUN_021B836C(w, pt)`: the camera record of the point `pt` (`+0x3C`).
pub fn point_cam<E: FieldEnv>(env: &mut E, w: u32, pt: u32) -> u32 {
    let rd = env.r32(w + 0x24);
    let i = env.r32(pt + 0x3C);
    cam_rec(env, rd, i)
}

/// C: `FUN_021B8378(x)`: `x / 16` rounded to the nearest (ties away from zero).
pub fn round16(x: u32) -> u32 {
    let x = x as i32;
    let a = abs(x) as u32;
    if a & 0xF >= 8 {
        if x >= 0 {
            return div16(x).wrapping_add(1) as u32;
        }
        return div16(x).wrapping_sub(1) as u32;
    }
    div16(x) as u32
}

/// C: `FUN_021B83B0(a, b, r)`: `a` and `b` (VecFx32) are within `2 * r` on the ground (whole units) and less than 8.0 apart
/// in height.
pub fn near<E: FieldEnv>(env: &mut E, a: u32, b: u32, r: u32) -> u32 {
    let dx = (env.r32(a) as i32 >> 12).wrapping_sub(env.r32(b) as i32 >> 12);
    let dz = (env.r32(a + 8) as i32 >> 12).wrapping_sub(env.r32(b + 8) as i32 >> 12);
    let sq = dx.wrapping_mul(dx).wrapping_add(dz.wrapping_mul(dz));
    let dy = abs((env.r32(a + 4) as i32).wrapping_sub(env.r32(b + 4) as i32));
    let rr = (r as i32 >> 12).wrapping_shl(1);
    (dy < 0x8000 && sq < rr.wrapping_mul(rr)) as u32
}

/// C: `FUN_021B83FC(w, idx, z, x)`: the LOCATION values of `w` as numbers: its line index, `z` (along / 16) and `x`
/// (lateral / 16 + the larger of the widths at both ends / 16).
pub fn loc_numbers<E: FieldEnv>(env: &mut E, w: u32, idx: u32, z: u32, x: u32) {
    let ln = env.r32(w + 4);
    let len = env.r32(w + 0x10);
    let rd = env.r32(w + 0x24);
    let mut m = env.callv(LINE_WIDTH, &[ln, 0, len, rd]) as i32;
    let len = env.r32(w + 0x10);
    let ln = env.r32(w + 4);
    let rd = env.r32(w + 0x24);
    let e = env.callv(LINE_WIDTH, &[ln, len, len, rd]) as i32;
    if m < e {
        m = e;
    }
    let rd = env.r32(w + 0x24);
    let mut found = 0u32;
    let mut i = 0i32;
    if env.r16(rd + 2) > 0 {
        while i < i32::from(env.r16(rd + 2)) {
            let lines = env.r32(rd + 0x14);
            if env.r32(w + 4) == lines.wrapping_add((i as u32).wrapping_mul(0x48)) {
                found = i as u32;
            }
            i += 1;
        }
    }
    env.w16(idx, found as u16);
    let a = div16(env.r32(w + 0xC) as i32);
    env.w16(z, a as u16);
    let l = div16(env.r32(w + 0x14) as i32);
    env.w16(x, l.wrapping_add(div16(m)) as u16);
}

/// C: `FUN_021B8484(w, idx, z, x)`: place `w` on line `idx` at `z` with lateral `x` (units; the LOCATION kind is kept).
pub fn set_loc_xz<E: FieldEnv>(env: &mut E, w: u32, idx: u32, z: u32, x: u32) {
    let l = env.mem().push_local(8);
    get_loc(env, w, l);
    env.w16(l, idx as u16);
    env.w16(l + 4, 0);
    env.w16(l + 6, z as u16);
    set_loc(env, w, l);
    let half = div16(env.r32(w + 0x18) as i32);
    env.w16(l + 4, (x as i32).wrapping_sub(half) as u16);
    env.call(POS_OF, &mut [CArg::V(w), CArg::Out(w + 0x38, 12)]);
    for k in 0..4 {
        let v = env.r16(l + 2 * k);
        env.w16(w + 0x5C + 2 * k, v);
        env.w16(w + 0x64 + 2 * k, v);
    }
    for k in 0..3 {
        let v = env.r32(w + 0x38 + 4 * k);
        env.w32(w + 0x44 + 4 * k, v);
    }
    env.mem().pop_local(l);
}

/// C: `FUN_021B8514(pt, dir, rd, blk, open)`: the line of the point `pt` that leaves in direction `dir` (0 when none; with
/// `open` only a line that is not blocked).
pub fn point_line<E: FieldEnv>(
    env: &mut E,
    pt: u32,
    dir: u32,
    rd: u32,
    blk: u32,
    open: u32,
) -> u32 {
    for i in 0..4u32 {
        if dir == env.r32(pt + 4 * i + 0x10) {
            let li = env.r32(pt + 4 * i);
            if open != 0 {
                return env.callv(LINE_IF_OPEN, &[blk, rd, li]);
            }
            return line_rec(env, rd, li);
        }
    }
    0
}

/// C: `FUN_021B854C(w, ln, dir, pos, lat, len, wid)`: move `w` onto line `ln` at `pos` / `lat` (continuing on the next
/// line past either end); returns `dir`, or 0 when the place is not allowed.
#[allow(clippy::too_many_arguments)]
pub fn try_line<E: FieldEnv>(
    env: &mut E,
    w: u32,
    ln: u32,
    dir: u32,
    pos: i32,
    lat: i32,
    len: i32,
    wid: i32,
) -> u32 {
    let (mut ln, mut pos, mut len, mut wid) = (ln, pos, len, wid);
    let blk = env.r32(w + 0x28);
    let rd = env.r32(w + 0x24);
    let unit = env.r32(w + 0x1C);
    if env.callv(CAN_ENTER, &[blk, ln, pos as u32, lat as u32, rd, unit]) == 0 {
        return 0;
    }
    if abs(lat) > wid {
        return 0;
    }
    if !(pos < len && pos >= 0) {
        let rd = env.r32(w + 0x24);
        let (p, d) = if pos >= len {
            let i = env.r32(ln + 4);
            let p = point_rec(env, rd, i);
            (p, env.r32(ln + 8))
        } else {
            let i = env.r32(ln);
            let p = point_rec(env, rd, i);
            let d0 = env.r32(ln + 8);
            (p, env.callv(DIR_BACK, &[d0]))
        };
        let rd = env.r32(w + 0x24);
        let blk = env.r32(w + 0x28);
        let nl = point_line(env, p, d, rd, blk, 1);
        if nl == 0 {
            if pos > len {
                return 0;
            }
        } else {
            ln = nl;
            if pos >= len {
                pos = pos.wrapping_sub(len);
                len = line_len(env, nl);
                let rd = env.r32(w + 0x24);
                wid = line_width(env, nl, pos, len, rd);
            } else if pos < 0 {
                len = line_len(env, nl);
                pos = pos.wrapping_add(len);
                let rd = env.r32(w + 0x24);
                wid = line_width(env, nl, pos, len, rd);
            }
            if pos >= len {
                return 0;
            }
            if abs(lat) > wid {
                return 0;
            }
        }
    }
    set_line(
        env, w, ln, dir, pos as u32, lat as u32, len as u32, wid as u32,
    );
    let rd = env.r32(w + 0x24);
    env.callv(CORNER_OK, &[ln, rd]);
    dir
}

/// C: `FUN_021B8658(w, ln, dir, pos, lat, len, wid)`: `w` is on line `ln` (kind 1) with these values.
#[allow(clippy::too_many_arguments)]
pub fn set_line<E: FieldEnv>(
    env: &mut E,
    w: u32,
    ln: u32,
    dir: u32,
    pos: u32,
    lat: u32,
    len: u32,
    wid: u32,
) {
    env.w32(w + 4, ln);
    env.w32(w + 0x14, lat);
    env.w16(w, 1);
    env.w32(w + 0x10, len);
    env.w32(w + 0x20, dir);
    env.w32(w + 0xC, pos);
    env.w32(w + 0x18, wid);
}

/// C: `FUN_021B8674(rd, ln, p)`: the line starts at the point `p`.
pub fn starts_at<E: FieldEnv>(env: &mut E, rd: u32, ln: u32, p: u32) -> u32 {
    let i = env.r32(ln);
    (point_rec(env, rd, i) == p) as u32
}

/// C: `FUN_021B868C(rd, ln, p)`: the line ends at the point `p`.
pub fn ends_at<E: FieldEnv>(env: &mut E, rd: u32, ln: u32, p: u32) -> u32 {
    let i = env.r32(ln + 4);
    (point_rec(env, rd, i) == p) as u32
}

/// C: `FUN_021B86A4(rd, ln, side, start, blk, unit)`: the widths around the start point of `ln`: `*start` = the larger of
/// the width of `ln` at its start and the width of the line going back at its end; `*side` = the larger (positive) width of
/// the lines to the left / right at the point's side.
pub fn line_bounds<E: FieldEnv>(
    env: &mut E,
    rd: u32,
    ln: u32,
    side: u32,
    start: u32,
    blk: u32,
    unit: u32,
) {
    let _ = unit;
    let mut s = 0i32;
    let i = env.r32(ln);
    let p = point_rec(env, rd, i);
    let d = env.r32(ln + 8);
    let db = env.callv(DIR_BACK, &[d]);
    let back = point_line(env, p, db, rd, blk, 1);
    let d = env.r32(ln + 8);
    let dl = env.callv(DIR_LEFT, &[d]);
    let left = point_line(env, p, dl, rd, blk, 1);
    let d = env.r32(ln + 8);
    let dr = env.callv(DIR_RIGHT, &[d]);
    let right = point_line(env, p, dr, rd, blk, 1);
    let len = line_len(env, ln);
    let mut st = line_width(env, ln, 0, len, rd);
    if back != 0 {
        let l = line_len(env, back);
        let v = line_width(env, back, l, l, rd);
        if v > st {
            st = v;
        }
    }
    if left != 0 {
        let l = line_len(env, left);
        let v = if starts_at(env, rd, left, p) != 0 {
            line_width(env, left, 0, l, rd)
        } else {
            line_width(env, left, l, l, rd)
        };
        if v > 0 {
            s = v;
        }
    }
    if right != 0 {
        let l = line_len(env, right);
        let v = if starts_at(env, rd, right, p) != 0 {
            line_width(env, right, 0, l, rd)
        } else {
            line_width(env, right, l, l, rd)
        };
        if v > s {
            s = v;
        }
    }
    env.w32(side, s as u32);
    env.w32(start, st as u32);
}

/// C: `FUN_021B87BC(rd)`: zero the RAIL DATA (0x30 bytes, `MI_CpuFill8`).
pub fn clear_rd<E: FieldEnv>(env: &mut E, rd: u32) {
    env.mem().fill(rd, 0x30, 0);
}

/// C: `FUN_021B87C8(dst, src)`: copy a RAIL DATA (0x30 bytes, `MI_CpuCopy8`).
pub fn copy_data<E: FieldEnv>(env: &mut E, dst: u32, src: u32) {
    env.mem().copy(dst, src, 0x30);
}

/// C: `FUN_021B87D8(rd, i)`: point `i` (0 for 0xFFFF, the first point past the count).
pub fn point_rec<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    if i < u32::from(env.r16(rd)) {
        return env.r32(rd + 0x10).wrapping_add(i.wrapping_mul(0x70));
    }
    if i == NONE {
        return 0;
    }
    env.r32(rd + 0x10)
}

/// C: `FUN_021B87FC(rd, i)`: line `i` (0 for 0xFFFF, the first line past the count).
pub fn line_rec<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    if i < u32::from(env.r16(rd + 2)) {
        return env.r32(rd + 0x14).wrapping_add(i.wrapping_mul(0x48));
    }
    if i == NONE {
        return 0;
    }
    env.r32(rd + 0x14)
}

/// C: `FUN_021B8820(rd, i)`: camera record `i` (0 for 0xFFFF, the ROM default record past the count).
pub fn cam_rec<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    if i < u32::from(env.r16(rd + 4)) {
        return env.r32(rd + 0x18).wrapping_add(i.wrapping_mul(0x24));
    }
    if i == NONE {
        return 0;
    }
    DEFAULT_CAM_REC
}

/// C: `FUN_021B8848(rd, i)`: line record `i` (0 for 0xFFFF, the ROM default record past the count).
pub fn line_rec_rec<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    if i < u32::from(env.r16(rd + 6)) {
        return env.r32(rd + 0x1C).wrapping_add(i.wrapping_mul(0x24));
    }
    if i == NONE {
        return 0;
    }
    DEFAULT_LINE_REC
}

/// Entry `i` of the proc table at `rd + tab` with the count at `rd + cnt` (0 for 0xFFFF, entry 0 past the count).
fn proc_entry<E: FieldEnv>(env: &mut E, rd: u32, cnt: u32, tab: u32, i: u32) -> u32 {
    if i < u32::from(env.r16(rd + cnt)) {
        let t = env.r32(rd + tab);
        return env.r32(t.wrapping_add(i.wrapping_mul(4)));
    }
    if i == NONE {
        return 0;
    }
    let t = env.r32(rd + tab);
    env.r32(t)
}

/// C: `FUN_021B8870(rd, i)`: camera proc `i`.
pub fn cam_proc<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    proc_entry(env, rd, 8, 0x20, i)
}

/// C: `FUN_021B8894(rd, i)`: position proc `i`.
pub fn pos_proc<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    proc_entry(env, rd, 0xA, 0x24, i)
}

/// C: `FUN_021B88B8(rd, i)`: line-hit proc `i`.
pub fn hit_proc<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    proc_entry(env, rd, 0xC, 0x28, i)
}

#[cfg(test)]
mod tests;
