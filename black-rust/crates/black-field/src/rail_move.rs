//! `field_rail.c` part 2 (overlay 21, `0x021B88DC..0x021B9B7C`; F3b): the line movement of a RAIL WORK (one step along, back
//! or sideways, continuing onto the next line at the ends), the OBJ / blocked-line / direction helpers, line lengths and
//! widths, and the segment / interpolation vector math.
//!
//! Records: see [`crate::rail`]. Directions are 1..4: back of 1 / 2 / 3 / 4 is 3 / 4 / 1 / 2, "right" ([`dir_right`]) is
//! the next one, "left" ([`dir_left`]) the previous one. A line's half width at a position is interpolated between the
//! values its two end points list for it ([`line_width`]).
//!
//! Callees in [`crate::rail`] (the rail-data lookups, LOCATIONs, the move driver) and the NitroSDK vector / fx functions (D3
//! math hooks) are reached through [`FieldEnv::call`] by address; the position procs of the rail data are called at their
//! table entries' addresses. FX_Mul products (`((a * b) + 0x800) >> 12` on 64 bits) are computed as the original does. The
//! sine table `FX_SinCosTable_` (ARM9 `0x020A1B38`) is read through the memory window. Verified by
//! `tools/check-fn.sh black-field rail_move --per-function` (spec `tools/rom-diff/specs/black-field/rail_move.spec`).

use crate::hooks::{CArg, FieldEnv};
use crate::rail::{abs, div16, rd32, NONE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_88DC,
    0x021B_9164,
    0x021B_919C,
    0x021B_91A8,
    0x021B_91AC,
    0x021B_91C8,
    0x021B_91F8,
    0x021B_9238,
    0x021B_9278,
    0x021B_9280,
    0x021B_9338,
    0x021B_9368,
    0x021B_93D4,
    0x021B_9404,
    0x021B_9434,
    0x021B_9464,
    0x021B_9478,
    0x021B_949C,
    0x021B_94C0,
    0x021B_94D8,
    0x021B_9510,
    0x021B_9530,
    0x021B_95C0,
    0x021B_960C,
    0x021B_963C,
    0x021B_96AC,
    0x021B_9768,
    0x021B_976C,
    0x021B_977C,
    0x021B_97D0,
    0x021B_97FC,
    0x021B_9828,
    0x021B_9A50,
    0x021B_9B24,
];

// field_rail.c part 1 (crate::rail), reached by address
const POINT: u32 = 0x021B_87D8;
const LINE: u32 = 0x021B_87FC;
const CAM_REC: u32 = 0x021B_8820;
const LINE_REC: u32 = 0x021B_8848;
const POS_PROC: u32 = 0x021B_8894;
const POINT_LINE: u32 = 0x021B_8514;
const TRY_LINE: u32 = 0x021B_854C;
const STARTS_AT: u32 = 0x021B_8674;
const ENDS_AT: u32 = 0x021B_868C;
const LINE_BOUNDS: u32 = 0x021B_86A4;
const SET_LOC: u32 = 0x021B_7EE4;
const START_MOVE: u32 = 0x021B_80E0;
const MOVE_FRAME: u32 = 0x021B_810C;
const GET_POS: u32 = 0x021B_8028;
const LOC_VALID: u32 = 0x021B_7DE0;
// NitroSDK (ARM9)
const VEC_ADD: u32 = 0x0207_C9D0;
const VEC_SUB: u32 = 0x0207_CA00;
const VEC_DOT: u32 = 0x0207_CA68;
const VEC_CROSS: u32 = 0x0207_CAE8;
const VEC_MAG: u32 = 0x0207_CBD0;
const VEC_NORMALIZE: u32 = 0x0207_CCA4;
const VEC_MULT_ADD: u32 = 0x0207_CEE4;
const FX_DIV: u32 = 0x0207_C768;
const FX_ACOS_IDX: u32 = 0x0207_D184;
const VEC_SCALE: u32 = 0x0205_20A4;
/// `FX_SinCosTable_` (ARM9 data): `{s16 sin, s16 cos}` per angle index.
pub const SIN_COS_TABLE: u32 = 0x020A_1B38;
/// The default camera record (ROM 0x021D9A0C), the answer of [`cam_of`] for a RAIL WORK of kind 2.
pub const DEFAULT_CAM_REC: u32 = 0x021D_9A0C;
/// Constants the original hands to the empty debug hook `021B9768` (ROM addresses).
const DEBUG_ARGS: [u32; 3] = [0x021D_E2B0, 0x021D_E2B4, 0x021D_E2BC];

/// `FX_Mul(a, b)`: `((a * b) + 0x800) >> 12` on the 64-bit product, low 32 bits.
fn fx_mul(a: i32, b: i32) -> i32 {
    ((i64::from(a) * i64::from(b) + 0x800) >> 12) as i32
}

fn rd_of<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w + 0x24)
}

fn point<E: FieldEnv>(env: &mut E, rd: u32, i: u32) -> u32 {
    env.callv(POINT, &[rd, i])
}

fn point_line<E: FieldEnv>(env: &mut E, w: u32, p: u32, d: u32) -> u32 {
    let rd = env.r32(w + 0x24);
    let blk = env.r32(w + 0x28);
    env.callv(POINT_LINE, &[p, d, rd, blk, 0])
}

fn pos<E: FieldEnv>(env: &mut E, w: u32) -> i32 {
    env.r32(w + 0xC) as i32
}

fn lat<E: FieldEnv>(env: &mut E, w: u32) -> i32 {
    env.r32(w + 0x14) as i32
}

/// The tail shared by every branch of [`line_move`]: check the point, measure the line and try to move onto it.
#[allow(clippy::too_many_arguments)]
fn enter<E: FieldEnv>(
    env: &mut E,
    w: u32,
    p: u32,
    ln: u32,
    d: u32,
    along: impl Fn(i32) -> i32,
    side: i32,
) -> u32 {
    let rd = rd_of(env, w);
    corner_ok(env, p, rd);
    let len = line_len(env, ln);
    let a = along(len);
    let rd = rd_of(env, w);
    let wid = line_width(env, ln, a as u32, len, rd);
    env.callv(
        TRY_LINE,
        &[w, ln, d, a as u32, side as u32, len as u32, wid as u32],
    )
}

fn starts_at<E: FieldEnv>(env: &mut E, w: u32, ln: u32, p: u32) -> u32 {
    let rd = rd_of(env, w);
    env.callv(STARTS_AT, &[rd, ln, p])
}

fn ends_at<E: FieldEnv>(env: &mut E, w: u32, ln: u32, p: u32) -> u32 {
    let rd = rd_of(env, w);
    env.callv(ENDS_AT, &[rd, ln, p])
}

/// C: `FUN_021B88DC(w, dir, step)`: move the RAIL WORK one step in `dir` (along its line, back, or to either side by
/// `step`); past the end of the line (or the side of a crossing) it continues on the line of the end point in that
/// direction (`021B854C`). Returns the direction moved, 0 when blocked.
pub fn line_move<E: FieldEnv>(env: &mut E, w: u32, d: u32, step: u32) -> u32 {
    let ln = env.r32(w + 4);
    let len = env.r32(w + 0x10) as i32;
    let step = step as i32;
    if d == env.r32(ln + 8) {
        let rd = rd_of(env, w);
        let i = env.r32(ln + 4);
        let p = point(env, rd, i);
        let mut past = false;
        let a = point_line(env, w, p, d);
        let dl = dir_left(d);
        let b = point_line(env, w, p, dl);
        let dr = dir_right(d);
        let c = point_line(env, w, p, dr);
        let blk = env.r32(w + 0x28);
        side_step(env, blk, w, step, 0, d);
        if pos(env, w) < len {
            let rd = rd_of(env, w);
            let p0 = pos(env, w);
            let wv = line_width(env, ln, p0 as u32, len, rd);
            if abs(lat(env, w)) <= wv {
                return d;
            }
            past = true;
        }
        if a != 0 && !past {
            let o = pos(env, w).wrapping_sub(len);
            let l = lat(env, w);
            let r = enter(env, w, p, a, d, |_| o, l);
            if r != 0 {
                return r;
            }
        }
        if b != 0 && lat(env, w) < 1 {
            if starts_at(env, w, b, p) == 0 {
                if ends_at(env, w, b, p) != 0 {
                    let l = lat(env, w);
                    let o = pos(env, w).wrapping_sub(len).wrapping_neg();
                    let r = enter(env, w, p, b, d, |n| n.wrapping_add(l), o);
                    if r != 0 {
                        return r;
                    }
                }
            } else {
                let l = lat(env, w).wrapping_neg();
                let o = pos(env, w).wrapping_sub(len);
                let r = enter(env, w, p, b, d, |_| l, o);
                if r != 0 {
                    return r;
                }
            }
        }
        if c != 0 && lat(env, w) >= 0 {
            if starts_at(env, w, c, p) == 0 {
                if ends_at(env, w, c, p) != 0 {
                    let l = lat(env, w);
                    let o = pos(env, w).wrapping_sub(len);
                    let r = enter(env, w, p, c, d, |n| n.wrapping_sub(l), o);
                    if r != 0 {
                        return r;
                    }
                }
            } else {
                let l = lat(env, w);
                let o = pos(env, w).wrapping_sub(len).wrapping_neg();
                let r = enter(env, w, p, c, d, |_| l, o);
                if r != 0 {
                    return r;
                }
            }
        }
        if !past {
            if pos(env, w) <= len {
                return d;
            }
            let v = pos(env, w).wrapping_sub(step);
            env.w32(w + 0xC, v as u32);
            return 0;
        }
        return clamp_lat(env, w, ln, len as u32, d);
    }
    let ld = env.r32(ln + 8);
    if d == dir_back(ld) {
        let rd = rd_of(env, w);
        let i = env.r32(ln);
        let p = point(env, rd, i);
        let mut past = false;
        let dr = dir_right(d);
        let a = point_line(env, w, p, dr);
        let dl = dir_left(d);
        let b = point_line(env, w, p, dl);
        let c = point_line(env, w, p, d);
        let blk = env.r32(w + 0x28);
        side_step(env, blk, w, step.wrapping_neg(), 0, d);
        if pos(env, w) >= 0 {
            let rd = rd_of(env, w);
            let p0 = pos(env, w);
            let wv = line_width(env, ln, p0 as u32, len, rd);
            if abs(lat(env, w)) <= wv {
                return d;
            }
            past = true;
        }
        if c != 0 && !past {
            let o = abs(pos(env, w));
            let l = lat(env, w);
            let r = enter(env, w, p, c, d, |n| n.wrapping_sub(o), l);
            if r != 0 {
                return r;
            }
        }
        if a != 0 && lat(env, w) < 1 {
            if starts_at(env, w, a, p) == 0 {
                if ends_at(env, w, a, p) != 0 {
                    let l = lat(env, w);
                    let o = pos(env, w).wrapping_neg();
                    let r = enter(env, w, p, a, d, |n| n.wrapping_add(l), o);
                    if r != 0 {
                        return r;
                    }
                }
            } else {
                let l = lat(env, w).wrapping_neg();
                let o = pos(env, w);
                let r = enter(env, w, p, a, d, |_| l, o);
                if r != 0 {
                    return r;
                }
            }
        }
        if b != 0 && lat(env, w) >= 0 {
            if starts_at(env, w, b, p) != 0 {
                let l = lat(env, w);
                let o = pos(env, w).wrapping_neg();
                let r = enter(env, w, p, b, d, |_| l, o);
                if r != 0 {
                    return r;
                }
            }
            if ends_at(env, w, b, p) != 0 {
                let l = lat(env, w);
                let o = pos(env, w);
                let r = enter(env, w, p, b, d, |n| n.wrapping_sub(l), o);
                if r != 0 {
                    return r;
                }
            }
        }
        if past {
            return clamp_lat(env, w, ln, len as u32, d);
        }
        let v = pos(env, w).wrapping_add(step);
        env.w32(w + 0xC, v as u32);
        return 0;
    }
    let ld = env.r32(ln + 8);
    if d == dir_right(ld) {
        let blk = env.r32(w + 0x28);
        side_step(env, blk, w, 0, step, d);
        let l = lat(env, w);
        if l > 0 && l.wrapping_rem(16) == 0 {
            let r = sideways(env, w, ln, len, d, false);
            if r != 0 {
                return r;
            }
        }
        let l = lat(env, w);
        if l > 0 && l <= env.r32(w + 0x18) as i32 {
            return clamp_lat(env, w, ln, len as u32, d);
        }
        return d;
    }
    let ld = env.r32(ln + 8);
    if d == dir_left(ld) {
        let blk = env.r32(w + 0x28);
        side_step(env, blk, w, 0, step.wrapping_neg(), d);
        let l = lat(env, w);
        if l < 0 && l.wrapping_neg().wrapping_rem(16) == 0 {
            let r = sideways(env, w, ln, len, d, true);
            if r != 0 {
                return r;
            }
        }
        let l = lat(env, w);
        if l < 0 && l.wrapping_neg() <= env.r32(w + 0x18) as i32 {
            return clamp_lat(env, w, ln, len as u32, d);
        }
        return d;
    }
    0
}

/// The side-step crossings of [`line_move`]: at a whole-unit lateral offset, try the lines in direction `d` at both ends of
/// the current line (`left`: the offset is negative and measured as `|lat|`); the direction moved or 0.
fn sideways<E: FieldEnv>(env: &mut E, w: u32, ln: u32, len: i32, d: u32, left: bool) -> u32 {
    let lv = |env: &mut E| {
        let l = lat(env, w);
        if left {
            abs(l)
        } else {
            l
        }
    };
    for end in 0..2u32 {
        let rd = rd_of(env, w);
        let i = env.r32(ln + 4 * end);
        let p = point(env, rd, i);
        let a = point_line(env, w, p, d);
        if a == 0 {
            continue;
        }
        // the offset along `ln` from the end it crosses at (start: `pos`, end: `pos - len`)
        let off = |env: &mut E| {
            let p0 = pos(env, w);
            if end == 0 {
                p0
            } else {
                p0.wrapping_sub(len)
            }
        };
        if starts_at(env, w, a, p) != 0 {
            let l = lv(env);
            let o = off(env);
            let o = if left { o } else { o.wrapping_neg() };
            let r = enter(env, w, p, a, d, |_| l, o);
            if r != 0 {
                return r;
            }
        }
        if ends_at(env, w, a, p) != 0 {
            let l = lv(env);
            let o = off(env);
            let o = if left { o.wrapping_neg() } else { o };
            let r = enter(env, w, p, a, d, |n| n.wrapping_sub(l), o);
            if r != 0 {
                return r;
            }
        }
    }
    0
}

/// C: `FUN_021B9164(w, rd, r)`: reset a RAIL WORK (kind 2 = none) on the rail data `rd` of the system `r`.
pub fn obj_init<E: FieldEnv>(env: &mut E, w: u32, rd: u32, r: u32) {
    env.w32(w + 0x24, rd);
    let u = env.r32(rd + 0x2C);
    env.w32(w + 0x1C, u);
    env.w16(w, 2);
    env.w8(w + 0x33, 0);
    env.w8(w + 0x32, 0);
    env.w8(w + 0x2C, 0);
    env.w32(w + 0x78, r);
    env.w32(w + 4, 0);
    env.w32(w + 0xC, 0);
    env.w32(w + 0x14, 0);
    env.w32(w + 0x20, 0);
    env.w32(w + 0x28, r.wrapping_add(0x50));
    env.w8(w + 0x74, 0);
    env.w16(w + 8, 0);
}

/// C: `FUN_021B919C(w, rd)`: [`obj_init`] with the RAIL WORK's own system (`w + 0x78`).
pub fn obj_reset<E: FieldEnv>(env: &mut E, w: u32, rd: u32) {
    let r = env.r32(w + 0x78);
    obj_init(env, w, rd, r);
}

/// C: `FUN_021B91A8(o)`: the OBJ is in use (u16 `+2`).
pub fn obj_in_use<E: FieldEnv>(env: &mut E, o: u32) -> u32 {
    u32::from(env.r16(o + 2))
}

/// C: `FUN_021B91AC(w)`: the RAIL WORK is on a point or line and in use.
pub fn active<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    (env.r16(w) < 2 && env.r32(w + 4) != 0 && env.r16(w + 2) != 0) as u32
}

/// C: `FUN_021B91C8(w)`: the camera record of the RAIL WORK's point / line (the ROM default record for kind 2).
pub fn cam_of<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let ofs = match env.r16(w) {
        0 => 0x3C,
        1 => 0x10,
        _ => return DEFAULT_CAM_REC,
    };
    let rd = env.r32(w + 0x24);
    let p = env.r32(w + 4);
    let i = env.r32(p + ofs);
    env.callv(CAM_REC, &[rd, i])
}

/// C: `FUN_021B91F8(w, out)`: the position: a point's own position, on a line the position proc of its line record.
pub fn pos_of<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    match env.r16(w) {
        0 => {
            let p = env.r32(w + 4);
            for k in 0..3 {
                let v = env.r32(p + 0x30 + 4 * k);
                env.w32(out + 4 * k, v);
            }
        }
        1 => {
            let rd = env.r32(w + 0x24);
            let p = env.r32(w + 4);
            let i = env.r32(p + 0xC);
            let t = env.callv(LINE_REC, &[rd, i]);
            let rd = env.r32(w + 0x24);
            let t0 = rd32(env, t);
            let f = env.callv(POS_PROC, &[rd, t0]);
            if f != 0 {
                env.call(f & !1, &mut [CArg::V(w), CArg::Out(out, 12)]);
            }
        }
        _ => {}
    }
}

/// C: `FUN_021B9238(w, out)`: the LOCATION of the RAIL WORK.
pub fn loc_of<E: FieldEnv>(env: &mut E, w: u32, out: u32) {
    let kind = env.r16(w);
    env.w8(out + 2, kind as u8);
    let rd = env.r32(w + 0x24);
    let p = env.r32(w + 4);
    let idx = if kind == 0 {
        point_index(env, rd, p)
    } else {
        line_index(env, rd, p)
    };
    env.w16(out, idx as u16);
    let z = div16(env.r32(w + 0xC) as i32);
    env.w16(out + 6, z as u16);
    let x = div16(env.r32(w + 0x14) as i32);
    env.w16(out + 4, x as u16);
    let d = env.r32(w + 0x20);
    env.w8(out + 3, d as u8);
}

/// C: `FUN_021B9278(ln)`: the line length (`+0x14` units * 16).
pub fn line_len<E: FieldEnv>(env: &mut E, ln: u32) -> i32 {
    env.r32(ln + 0x14).wrapping_shl(4) as i32
}

/// C: `FUN_021B9280(ln, pos, len, rd)`: the half width of the line at `pos`: interpolated between the values the start /
/// end point list for the line (one missing: the other one).
pub fn line_width<E: FieldEnv>(env: &mut E, ln: u32, pos: u32, len: i32, rd: u32) -> i32 {
    let pts = env.r32(rd + 0x10);
    let pa = pts.wrapping_add(env.r32(ln).wrapping_mul(0x70));
    let pb = pts.wrapping_add(env.r32(ln + 4).wrapping_mul(0x70));
    let mut a = -1i32;
    let mut b = -1i32;
    for i in 0..4u32 {
        let li = env.r32(pa + 4 * i);
        if li != NONE && ln == env.r32(rd + 0x14).wrapping_add(li.wrapping_mul(0x48)) {
            a = env.r32(pa + 4 * i + 0x20) as i32;
        }
        let li = env.r32(pb + 4 * i);
        if li != NONE && ln == env.r32(rd + 0x14).wrapping_add(li.wrapping_mul(0x48)) {
            b = env.r32(pb + 4 * i + 0x20) as i32;
        }
    }
    if a < 0 {
        a = b;
    }
    if b < 0 {
        b = a;
    }
    let lim = (len as u32).wrapping_sub(16);
    let p = if pos > lim { lim } else { pos };
    let mut d = b.wrapping_sub(a);
    if lim != 0 {
        d = (p as i32).wrapping_mul(d).wrapping_div(lim as i32);
    }
    d.wrapping_add(a)
}

/// C: `FUN_021B9338(w, ln, len, dir)`: the half width at the current position (`+0x18`); a lateral offset beyond it is
/// clamped. Returns `dir`.
pub fn clamp_lat<E: FieldEnv>(env: &mut E, w: u32, ln: u32, len: u32, dir: u32) -> u32 {
    let p = env.r32(w + 0xC);
    let rd = env.r32(w + 0x24);
    let wv = line_width(env, ln, p, len as i32, rd);
    let l = env.r32(w + 0x14) as i32;
    env.w32(w + 0x18, wv as u32);
    if abs(l) >= wv {
        env.w32(w + 0x14, if l > 0 { wv } else { wv.wrapping_neg() } as u32);
    }
    dir
}

/// C: `FUN_021B9368(r, loc, dir, n, out)`: place the main RAIL WORK at `loc`, then move it 16 frames in `dir`; its
/// position after frame `n` goes to `out`. Returns 0 when the LOCATION did not change, else whether the new one is valid.
pub fn walk<E: FieldEnv>(env: &mut E, r: u32, loc: u32, dir: u32, n: u32, out: u32) -> u32 {
    let w = env.r32(r + 0x1C);
    env.call(SET_LOC, &mut [CArg::V(w), CArg::In(loc, 8)]);
    let w = env.r32(r + 0x1C);
    env.callv(START_MOVE, &[w, 4, dir]);
    let w = env.r32(r + 0x1C);
    env.callv(MOVE_FRAME, &[w]);
    let w = env.r32(r + 0x1C);
    env.callv(START_MOVE, &[w, 0, dir]);
    for i in 0..16u32 {
        if n == i {
            let w = env.r32(r + 0x1C);
            env.call(GET_POS, &mut [CArg::V(w), CArg::Out(out, 12)]);
        }
        let w = env.r32(r + 0x1C);
        env.callv(MOVE_FRAME, &[w]);
    }
    let w = env.r32(r + 0x1C);
    let mut a = [0u8; 8];
    let mut b = [0u8; 8];
    for i in 0..8u32 {
        a[i as usize] = env.r8(loc + i);
        b[i as usize] = env.r8(w + 0x5C + i);
    }
    if a == b {
        return 0;
    }
    let w = env.r32(r + 0x1C);
    env.call(LOC_VALID, &mut [CArg::V(w), CArg::In(w + 0x5C, 8)])
}

/// C: `FUN_021B93D4(d)`: the opposite direction (1 <-> 3, 2 <-> 4; 0 for anything else).
pub fn dir_back(d: u32) -> u32 {
    match d {
        1 => 3,
        2 => 4,
        3 => 1,
        4 => 2,
        _ => 0,
    }
}

/// C: `FUN_021B9404(d)`: the next direction (1 -> 2 -> 3 -> 4 -> 1; 0 for anything else).
pub fn dir_right(d: u32) -> u32 {
    match d {
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 1,
        _ => 0,
    }
}

/// C: `FUN_021B9434(d)`: the previous direction (1 -> 4 -> 3 -> 2 -> 1; 0 for anything else).
pub fn dir_left(d: u32) -> u32 {
    match d {
        1 => 4,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 0,
    }
}

/// Number of entries of the blocked-line list.
const BLK_N: u32 = 18;

/// C: `FUN_021B9464(blk)`: clear the blocked-line list (every entry 0xFFFF).
pub fn blk_clear<E: FieldEnv>(env: &mut E, blk: u32) {
    for i in 0..BLK_N {
        env.w16(blk + 2 * i, 0xFFFF);
    }
}

/// C: `FUN_021B9478(blk, id)`: block line `id` (the first free entry); returns `blk`.
pub fn blk_add<E: FieldEnv>(env: &mut E, blk: u32, id: u32) -> u32 {
    for i in 0..BLK_N {
        if env.r16(blk + 2 * i) == 0xFFFF {
            env.w16(blk + 2 * i, id as u16);
            return blk;
        }
    }
    blk
}

/// C: `FUN_021B949C(blk, id)`: unblock line `id` (the first entry equal to it); returns `blk`.
pub fn blk_remove<E: FieldEnv>(env: &mut E, blk: u32, id: u32) -> u32 {
    for i in 0..BLK_N {
        if id == u32::from(env.r16(blk + 2 * i)) {
            env.w16(blk + 2 * i, 0xFFFF);
            return blk;
        }
    }
    blk
}

/// C: `FUN_021B94C0(blk, id)`: line `id` is blocked.
pub fn blk_has<E: FieldEnv>(env: &mut E, blk: u32, id: u32) -> u32 {
    (0..BLK_N).any(|i| id == u32::from(env.r16(blk + 2 * i))) as u32
}

/// C: `FUN_021B94D8(blk, ln, rd)`: the line `ln` is blocked.
pub fn blk_has_line<E: FieldEnv>(env: &mut E, blk: u32, ln: u32, rd: u32) -> u32 {
    let mut idx = 0u32;
    let mut i = 0i32;
    if env.r16(rd + 2) > 0 {
        let lines = env.r32(rd + 0x14);
        while i < i32::from(env.r16(rd + 2)) {
            if ln == lines.wrapping_add((i as u32).wrapping_mul(0x48)) {
                idx = i as u32;
            }
            i += 1;
        }
    }
    blk_has(env, blk, idx & 0xFFFF)
}

/// C: `FUN_021B9510(blk, rd, i)`: line `i` unless it is blocked (0).
pub fn line_if_open<E: FieldEnv>(env: &mut E, blk: u32, rd: u32, i: u32) -> u32 {
    if blk_has(env, blk, i & 0xFFFF) != 0 {
        return 0;
    }
    env.callv(LINE, &[rd, i])
}

/// C: `FUN_021B9530(blk, w, da, dl, dir)`: move `da` along and `dl` sideways; on a blocked line the position stops at the
/// width of the crossing at the start point and (without a line in `dir` there) the lateral offset at the start width.
pub fn side_step<E: FieldEnv>(env: &mut E, blk: u32, w: u32, da: i32, dl: i32, dir: u32) {
    let ln = env.r32(w + 4);
    let rd = env.r32(w + 0x24);
    if blk_has_line(env, blk, ln, rd) == 0 {
        let p = env.r32(w + 0xC) as i32;
        env.w32(w + 0xC, p.wrapping_add(da) as u32);
        let l = env.r32(w + 0x14) as i32;
        env.w32(w + 0x14, l.wrapping_add(dl) as u32);
        return;
    }
    let rd = env.r32(w + 0x24);
    let ln = env.r32(w + 4);
    let i = env.r32(ln);
    let p = point(env, rd, i);
    let rd = env.r32(w + 0x24);
    let nl = env.callv(POINT_LINE, &[p, dir, rd, blk, 0]);
    let mut side = [0u8; 4];
    let mut start = [0u8; 4];
    let rd = env.r32(w + 0x24);
    let ln = env.r32(w + 4);
    let unit = env.r32(w + 0x1C);
    env.call(
        LINE_BOUNDS,
        &mut [
            CArg::V(rd),
            CArg::V(ln),
            CArg::OutBuf(&mut side),
            CArg::OutBuf(&mut start),
            CArg::V(blk),
            CArg::V(unit),
        ],
    );
    let side = i32::from_le_bytes(side);
    let start = i32::from_le_bytes(start);
    let p = (env.r32(w + 0xC) as i32).wrapping_add(da);
    env.w32(w + 0xC, p as u32);
    let l = (env.r32(w + 0x14) as i32).wrapping_add(dl);
    env.w32(w + 0x14, l as u32);
    if p > side {
        env.w32(w + 0xC, side as u32);
    }
    if nl == 0 {
        let l = env.r32(w + 0x14) as i32;
        if abs(l) > start {
            env.w32(
                w + 0x14,
                if l > 0 { start } else { start.wrapping_neg() } as u32,
            );
        }
    }
}

/// C: `FUN_021B95C0(blk, ln, pos, lat, rd, unit)`: the place is allowed (an open line always; a blocked one within the
/// crossing widths at its start).
pub fn can_enter<E: FieldEnv>(
    env: &mut E,
    blk: u32,
    ln: u32,
    pos: i32,
    lat: i32,
    rd: u32,
    unit: u32,
) -> u32 {
    if blk_has_line(env, blk, ln, rd) == 0 {
        return 1;
    }
    let mut side = [0u8; 4];
    let mut start = [0u8; 4];
    env.call(
        LINE_BOUNDS,
        &mut [
            CArg::V(rd),
            CArg::V(ln),
            CArg::OutBuf(&mut side),
            CArg::OutBuf(&mut start),
            CArg::V(blk),
            CArg::V(unit),
        ],
    );
    if pos > i32::from_le_bytes(side) {
        return 0;
    }
    if abs(lat) > i32::from_le_bytes(start) {
        return 0;
    }
    1
}

/// C: `FUN_021B960C(pt, ln, rd)`: the direction in which the point `pt` lists the line `ln` (0 when it does not).
pub fn dir_of_line<E: FieldEnv>(env: &mut E, pt: u32, ln: u32, rd: u32) -> u32 {
    for i in 0..4u32 {
        let li = env.r32(pt + 4 * i);
        if env.callv(LINE, &[rd, li]) == ln {
            return env.r32(pt + 4 * i + 0x10);
        }
    }
    0
}

/// C: `FUN_021B963C(ln, rd)`: both end points list the line, the start one in the line's direction and the end one in
/// the opposite one.
pub fn corner_ok_line<E: FieldEnv>(env: &mut E, ln: u32, rd: u32) -> u32 {
    if env.r32(ln) == NONE || env.r32(ln + 4) == NONE {
        return 0;
    }
    let i = env.r32(ln);
    let pa = point(env, rd, i);
    let i = env.r32(ln + 4);
    let pb = point(env, rd, i);
    let da = dir_of_line(env, pa, ln, rd);
    if da == 0 || da != env.r32(ln + 8) {
        return 0;
    }
    let db = dir_of_line(env, pb, ln, rd);
    if db == 0 {
        return 0;
    }
    (db == dir_back(env.r32(ln + 8))) as u32
}

/// C: `FUN_021B96AC(pt, rd)`: the point's line list is consistent: a direction for every line and none for an empty slot,
/// every line touches the point, and a line and the one listed in the opposite direction have the same line direction.
pub fn corner_ok<E: FieldEnv>(env: &mut E, pt: u32, rd: u32) -> u32 {
    for i in 0..4u32 {
        let dv = env.r32(pt + 4 * i + 0x10);
        let li = env.r32(pt + 4 * i);
        if (dv == 0) != (li == NONE) {
            return 0;
        }
        if li == NONE {
            continue;
        }
        let ln = env.callv(LINE, &[rd, li]);
        let a0 = env.r32(ln);
        let a = point(env, rd, a0);
        let b0 = env.r32(ln + 4);
        let b = point(env, rd, b0);
        if a != pt && b != pt {
            return 0;
        }
        if dv == 0 {
            continue;
        }
        for j in i + 1..4 {
            let dj = env.r32(pt + 4 * j + 0x10);
            if dv == dir_back(dj) {
                let lj = env.r32(pt + 4 * j);
                let o = env.callv(LINE, &[rd, lj]);
                if env.r32(ln + 8) != env.r32(o + 8) {
                    return 0;
                }
            }
        }
    }
    1
}

/// C: `FUN_021B9768(..)`: empty (a debug draw hook).
pub fn debug_hook() {}

/// C: `FUN_021B976C(a, p, b)`: [`debug_hook`] for the position of the point `p` (when non-null).
pub fn debug_point(p: u32) {
    if p != 0 {
        debug_hook();
    }
}

/// C: `FUN_021B977C(w)`: the debug hook for the RAIL WORK's point, or both end points of its line.
pub fn debug_work<E: FieldEnv>(env: &mut E, w: u32) {
    let _ = DEBUG_ARGS;
    match env.r16(w) {
        0 => {
            let p = env.r32(w + 4);
            debug_point(p);
        }
        1 => {
            let rd = env.r32(w + 0x24);
            let l = env.r32(w + 4);
            let i = env.r32(l);
            let a = point(env, rd, i);
            let rd = env.r32(w + 0x24);
            let l = env.r32(w + 4);
            let i = env.r32(l + 4);
            let b = point(env, rd, i);
            debug_point(a);
            debug_point(b);
        }
        _ => {}
    }
}

/// C: `FUN_021B97D0(rd, p)`: the index of the point `p` (0 when not found).
pub fn point_index<E: FieldEnv>(env: &mut E, rd: u32, p: u32) -> u32 {
    let n = i32::from(env.r16(rd));
    let pts = env.r32(rd + 0x10);
    for i in 0..n.max(0) {
        if p == pts.wrapping_add((i as u32).wrapping_mul(0x70)) {
            return i as u32;
        }
    }
    0
}

/// C: `FUN_021B97FC(rd, ln)`: the index of the line `ln` (0 when not found).
pub fn line_index<E: FieldEnv>(env: &mut E, rd: u32, ln: u32) -> u32 {
    let mut i = 0i32;
    if env.r16(rd + 2) > 0 {
        let lines = env.r32(rd + 0x14);
        while i < i32::from(env.r16(rd + 2)) {
            if ln == lines.wrapping_add((i as u32).wrapping_mul(0x48)) {
                return i as u32;
            }
            i += 1;
        }
    }
    0
}

fn word(b: &[u8; 12], k: usize) -> i32 {
    i32::from_le_bytes(b[4 * k..4 * k + 4].try_into().unwrap())
}

/// C: `FUN_021B9828(d, n, p, a, b, out)`: the segment `a`..`b` crosses the vertical plane through `p` spanned by `d` and
/// `n` (normal = normalised `d` x `n`) at a fraction `t` of its length with `1.0 < t <= |b - a|`: the crossing point to
/// `out`, returns 1.
pub fn seg_hit<E: FieldEnv>(env: &mut E, d: u32, n: u32, p: u32, a: u32, b: u32, out: u32) -> u32 {
    let mut s = [0u8; 12];
    env.call(
        VEC_SUB,
        &mut [CArg::In(b, 12), CArg::In(a, 12), CArg::OutBuf(&mut s)],
    );
    let seg_len = env.call(VEC_MAG, &mut [CArg::InBuf(&s)]) as i32;
    let s0 = s;
    env.call(VEC_NORMALIZE, &mut [CArg::InBuf(&s0), CArg::OutBuf(&mut s)]);
    let mut dn = [0u8; 12];
    env.call(VEC_NORMALIZE, &mut [CArg::In(d, 12), CArg::OutBuf(&mut dn)]);
    let mut nn = [0u8; 12];
    env.call(VEC_NORMALIZE, &mut [CArg::In(n, 12), CArg::OutBuf(&mut nn)]);
    let mut c = [0u8; 12];
    env.call(
        VEC_CROSS,
        &mut [CArg::InBuf(&dn), CArg::InBuf(&nn), CArg::OutBuf(&mut c)],
    );
    let c0 = c;
    env.call(VEC_NORMALIZE, &mut [CArg::InBuf(&c0), CArg::OutBuf(&mut c)]);
    let (cx, cy, cz) = (word(&c, 0), word(&c, 1), word(&c, 2));
    let pv = [
        env.r32(p) as i32,
        env.r32(p + 4) as i32,
        env.r32(p + 8) as i32,
    ];
    let av = [
        env.r32(a) as i32,
        env.r32(a + 4) as i32,
        env.r32(a + 8) as i32,
    ];
    let sum_p = fx_mul(cz, pv[2]).wrapping_add(fx_mul(cx, pv[0]).wrapping_add(fx_mul(cy, pv[1])));
    let sum_a = fx_mul(cz, av[2]).wrapping_add(fx_mul(cx, av[0]).wrapping_add(fx_mul(cy, av[1])));
    let num = sum_p.wrapping_sub(sum_a);
    let (sx, sy, sz) = (word(&s, 0), word(&s, 1), word(&s, 2));
    let den = fx_mul(cz, sz).wrapping_add(fx_mul(cx, sx).wrapping_add(fx_mul(cy, sy)));
    if den == 0 {
        return 0;
    }
    let t = env.callv(FX_DIV, &[num as u32, den as u32]) as i32;
    if t > seg_len {
        return 0;
    }
    if t <= 0x1000 {
        return 0;
    }
    let z = av[2].wrapping_add(fx_mul(sz, t));
    let y = av[1].wrapping_add(fx_mul(sy, t));
    let x = av[0].wrapping_add(fx_mul(sx, t));
    env.w32(out, x as u32);
    env.w32(out + 4, y as u32);
    env.w32(out + 8, z as u32);
    1
}

/// `FX_SinCosTable_` sine at `idx`.
fn sin_at<E: FieldEnv>(env: &mut E, idx: i32) -> u32 {
    u32::from(env.r16(SIN_COS_TABLE.wrapping_add((idx as u32).wrapping_shl(2))))
}

/// C: `FUN_021B9A50(out, a, b, t)`: the spherical interpolation of the directions `a` and `b` at `t` (fx32 0..1.0),
/// normalised, to `out` (`b` normalised when they are parallel).
pub fn slerp<E: FieldEnv>(env: &mut E, out: u32, a: u32, b: u32, t: u32) {
    let mut an = [0u8; 12];
    let mut bn = [0u8; 12];
    env.call(VEC_NORMALIZE, &mut [CArg::In(a, 12), CArg::OutBuf(&mut an)]);
    env.call(VEC_NORMALIZE, &mut [CArg::In(b, 12), CArg::OutBuf(&mut bn)]);
    let dot = env.call(VEC_DOT, &mut [CArg::InBuf(&an), CArg::InBuf(&bn)]);
    let ang = env.callv(FX_ACOS_IDX, &[dot]) as i32;
    let sin_a = sin_at(env, ang >> 4);
    if sin_a == 0 {
        for k in 0..3 {
            env.w32(out + 4 * k as u32, word(&bn, k) as u32);
        }
        return;
    }
    let ang12 = ang.wrapping_shl(12);
    let i1 = (fx_mul(ang12, 0x1000i32.wrapping_sub(t as i32))) >> 16;
    let s1 = sin_at(env, i1);
    let k1 = env.callv(FX_DIV, &[s1, sin_a]);
    let a0 = an;
    env.call(
        VEC_SCALE,
        &mut [CArg::InBuf(&a0), CArg::V(k1), CArg::OutBuf(&mut an)],
    );
    let i2 = fx_mul(ang12, t as i32) >> 16;
    let s2 = sin_at(env, i2);
    let k2 = env.callv(FX_DIV, &[s2, sin_a]);
    let b0 = bn;
    env.call(
        VEC_SCALE,
        &mut [CArg::InBuf(&b0), CArg::V(k2), CArg::OutBuf(&mut bn)],
    );
    env.call(
        VEC_ADD,
        &mut [CArg::InBuf(&an), CArg::InBuf(&bn), CArg::Out(out, 12)],
    );
    env.call(VEC_NORMALIZE, &mut [CArg::In(out, 12), CArg::Out(out, 12)]);
}

/// C: `FUN_021B9B24(out, a, b, t)`: [`slerp`] scaled to the linearly interpolated length of `a` and `b`.
pub fn interp<E: FieldEnv>(env: &mut E, out: u32, a: u32, b: u32, t: u32) {
    let zero = [0u8; 12];
    slerp(env, out, a, b, t);
    let ma = env.call(VEC_MAG, &mut [CArg::In(a, 12)]) as i32;
    let mb = env.call(VEC_MAG, &mut [CArg::In(b, 12)]) as i32;
    let m = ma.wrapping_add(fx_mul(mb.wrapping_sub(ma), t as i32));
    env.call(
        VEC_MULT_ADD,
        &mut [
            CArg::V(m as u32),
            CArg::In(out, 12),
            CArg::InBuf(&zero),
            CArg::Out(out, 12),
        ],
    );
}

#[cfg(test)]
mod tests;
