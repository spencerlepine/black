//! F8 (fifth batch): overlay-10 script commands of F1 rows 10.27 ("symbol_map.c", 0x021664CC-0x02166F9C: the command handlers
//! and their helpers only) and 10.34 (0x0216AC40-0x0216B504, an unanchored scrcmd file: the Entree Forest symbol encounters):
//! pleasure-boat queries 425 / 427, the 20-record save block 477 / 478 / 480, 493, weather 310 and 554..558 / 560 (35 functions,
//! 13 command ids).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc10b.spec`. The functions listed in
//! [`PORTED`] are ported here (smoke ports) and checked with `tools/check-fn.sh black-script scrcmd_misc10b`; the rest are the
//! F8 port job's. The overlay-18 / overlay-11 intrude (Entralink) callees of row 10.34 stay hooks (X1).

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_6658,
    0x0216_670C,
    0x0216_6714,
    0x0216_6824,
    0x0216_6834,
    0x0216_B2E4,
];

/// One boat-time unit (s16 W+4 counts in these; `FUN_02166714` / `FUN_021C86E8`).
pub const BOAT_TIME_UNIT: i32 = 0x546;

/// C: `FUN_overlay_d_10__02166658`: the boat record word `W + 0x14 + 0x1C * i`, or 10 without a boat work.
pub fn boat_slot_word<H: MemEnv>(h: &mut H, w: u32, i: u32) -> u32 {
    if w == 0 {
        return 10;
    }
    h.r32(w.wrapping_add(i.wrapping_mul(0x1C)).wrapping_add(0x14))
}

/// C: `FUN_overlay_d_10__0216670c`: the boat time, s16 W+4 sign-extended.
pub fn boat_time<H: MemEnv>(h: &mut H, w: u32) -> u32 {
    h.r16(w + 4) as i16 as i32 as u32
}

/// C: `FUN_overlay_d_10__02166714`: set the boat time to `t` (u16 W+4); the unit count `(s16 W+4) / 0x546` (signed division,
/// `_s32_div_f` inlined) replaces W[8] when W[8] is not above it (signed).
pub fn boat_time_set<H: MemEnv>(h: &mut H, w: u32, t: u32) {
    h.w16(w + 4, t as u16);
    let q = i32::from(h.r16(w + 4) as i16) / BOAT_TIME_UNIT;
    if h.r32(w + 8) as i32 <= q {
        h.w32(w + 8, q as u32);
    }
}

/// C: `FUN_overlay_d_10__02166824`: byte `3 + i` of the record when `i < R[1]`, else 0xB.
pub fn rec20_byte<H: MemEnv>(h: &mut H, r: u32, i: u32) -> u8 {
    if i < u32::from(h.r8(r + 1)) {
        h.r8(r.wrapping_add(i).wrapping_add(3))
    } else {
        0xB
    }
}

/// C: `FUN_overlay_d_10__02166834`: field `k` of the record: 0 -> byte 1, 1 -> byte 0, 2 -> byte 2, else 0.
pub fn rec20_field<H: MemEnv>(h: &mut H, r: u32, k: u32) -> u8 {
    match k {
        0 => h.r8(r + 1),
        1 => h.r8(r),
        2 => h.r8(r + 2),
        _ => 0,
    }
}

/// C: `FUN_overlay_d_10__0216b2e4`: copy the position `src` (VecFx32, 12 bytes) to `dst` and move it to the forest edge of
/// warp kind `k`: 0 -> z = 0x188000, 1 -> z = 0x98000, 2 -> x = 0x1A8000, 3 -> x = 0x58000, 4..9 and others -> unchanged.
pub fn symbol_warp_pos<H: MemEnv>(h: &mut H, src: u32, k: u32, dst: u32) {
    for o in [0, 4, 8] {
        let v = h.r32(src + o);
        h.w32(dst + o, v);
    }
    match k {
        0 => h.w32(dst + 8, 0x62 << 14),
        1 => h.w32(dst + 8, 0x26 << 14),
        2 => h.w32(dst, 0x6A << 14),
        3 => h.w32(dst, 0x16 << 14),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
