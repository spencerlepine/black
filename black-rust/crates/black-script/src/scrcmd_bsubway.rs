//! F8 (fifth batch): overlay-21 `bsubway_scr.c`, Thumb code 0x021C9934-0x021CBFB0 (F1-SPLIT row 21.84): the Battle Subway work
//! W (0x7EC bytes) and commands 343..346 (58 functions; command 346 dispatches ~170 sub-commands).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_bsubway.spec`. The functions listed in
//! [`PORTED`] are ported here (smoke ports) and checked with `tools/check-fn.sh black-script scrcmd_bsubway`; the rest are the
//! F8 port job's. The Wi-Fi / wireless multi parts (comm_player.c, comm_entry_menu.c, the net layer) stay hooks (X1).

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_A234,
    0x021C_A238,
    0x021C_A23C,
    0x021C_A250,
    0x021C_A258,
    0x021C_A270,
    0x021C_BEFC,
];

/// Offset of the u16 win streak in the work W.
pub const W_STREAK: u32 = 0xE;

/// C: `FUN_overlay_d_21__021ca234`: the win streak (u16 W+0xE).
pub fn streak<H: MemEnv>(h: &mut H, w: u32) -> u16 {
    h.r16(w + W_STREAK)
}

/// C: `FUN_overlay_d_21__021ca238`: set the win streak.
pub fn set_streak<H: MemEnv>(h: &mut H, w: u32, n: u16) {
    h.w16(w + W_STREAK, n);
}

/// C: `FUN_overlay_d_21__021ca23c`: add 1 to the win streak unless it is 0xFFFF.
pub fn inc_streak<H: MemEnv>(h: &mut H, w: u32) {
    let n = h.r16(w + W_STREAK);
    if n < 0xFFFF {
        h.w16(w + W_STREAK, n + 1);
    }
}

/// C: `FUN_overlay_d_21__021ca250`: clear the win streak.
pub fn zero_streak<H: MemEnv>(h: &mut H, w: u32) {
    h.w16(w + W_STREAK, 0);
}

/// C: `FUN_overlay_d_21__021ca258`: streak = min(7 * rounds + n, 0xFFFF) (32-bit wrapping arithmetic, unsigned compare).
pub fn streak_from<H: MemEnv>(h: &mut H, w: u32, rounds: u32, n: u32) {
    let v = rounds.wrapping_mul(7).wrapping_add(n);
    h.w16(w + W_STREAK, v.min(0xFFFF) as u16);
}

/// C: `FUN_overlay_d_21__021ca270`: party size of a subway mode (jump table: modes 0 / 4 / 5 -> 3, 1 / 6 -> 4, 2 / 3 / 7 / 8 -> 2;
/// other values 0).
pub fn party_size(mode: u32) -> u8 {
    match mode {
        0 | 4 | 5 => 3,
        1 | 6 => 4,
        2 | 3 | 7 | 8 => 2,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021cbefc`: the opponents' IV value for a trainer id (3, 6, .. 21 in steps of 20 ids from 100, else 31).
pub fn trainer_iv(id: u32) -> u32 {
    match id {
        0..=99 => 3,
        100..=119 => 6,
        120..=139 => 9,
        140..=159 => 12,
        160..=179 => 15,
        180..=199 => 18,
        200..=219 => 21,
        _ => 31,
    }
}

#[cfg(test)]
mod tests;
