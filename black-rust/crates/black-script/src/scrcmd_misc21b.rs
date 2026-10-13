//! F8 (fifth batch): script commands in overlay-20 / overlay-21 files that are not scrcmd files: the WFBC (Black City / White
//! Forest) commands 450..457 at the end of overlay-20 `field_wfbc_data_local.c` (+ the counter accessors they use), the
//! field-skill commands 197..200 after `sodateya.c`, `fld_wipe_3dobj.c` 415..417 / 576, `pleasure_boat.c` 421..424 / 426 / 428
//! (+ the boat-work helpers) and `fld_trade.c` 443 (39 functions, 23 command ids; the WFBC talk dialog 0x021871F8 is UI flow,
//! left out).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc21b.spec`. The functions listed in
//! [`PORTED`] are ported here (smoke ports) and checked with `tools/check-fn.sh black-script scrcmd_misc21b`; the rest are the
//! F8 port job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_67A8,
    0x0218_67B4,
    0x0218_69B0,
    0x0218_69BC,
    0x021C_8738,
    0x021C_87D4,
    0x021C_8810,
];

/// WFBC counter b is raised in steps of 10 while it is below this cap (`FUN_021869BC`).
pub const WFBC_COUNT_B_CAP: u16 = 10000;
/// Records of the pleasure-boat work (0x1C bytes each from W + 0x24).
pub const BOAT_RECORDS: u32 = 15;
/// Size of a pleasure-boat record.
pub const BOAT_RECORD_SIZE: u32 = 0x1C;

/// C: `FUN_overlay_d_20__021867a8`: the high nibble of the record's byte 0.
pub fn wfbc_rec_hi4<H: MemEnv>(h: &mut H, rec: u32) -> u8 {
    h.r8(rec) >> 4
}

/// C: `FUN_overlay_d_20__021867b4`: the record's byte 2 is at most 0x14.
pub fn wfbc_rec_near<H: MemEnv>(h: &mut H, rec: u32) -> bool {
    h.r8(rec + 2) <= 0x14
}

/// C: `FUN_overlay_d_20__021869b0`: counter a += 1 (u16, wrapping).
pub fn wfbc_count_a_inc<H: MemEnv>(h: &mut H, c: u32) {
    let n = h.r16(c);
    h.w16(c, n.wrapping_add(1));
}

/// C: `FUN_overlay_d_20__021869bc`: counter b += 10 while it is below 10000.
pub fn wfbc_count_b_add10<H: MemEnv>(h: &mut H, c: u32) {
    let n = h.r16(c + 2);
    if n < WFBC_COUNT_B_CAP {
        h.w16(c + 2, n + 10);
    }
}

/// C: `FUN_overlay_d_21__021c8738`: a pleasure-boat query: k 0 -> byte W+0x1B0, 1..3 -> bytes W+0x1B1..0x1B3, 4 -> u16 W+6,
/// 5 -> the number of the 15 records whose first word is nonzero, other k -> 0.
pub fn boat_query<H: MemEnv>(h: &mut H, w: u32, k: u32) -> u32 {
    match k {
        0 => u32::from(h.r8(w + 0x1B0)),
        1 => u32::from(h.r8(w + 0x1B1)),
        2 => u32::from(h.r8(w + 0x1B2)),
        3 => u32::from(h.r8(w + 0x1B3)),
        4 => u32::from(h.r16(w + 6)),
        5 => (0..BOAT_RECORDS)
            .filter(|&i| h.r32(w + 0x24 + BOAT_RECORD_SIZE * i) != 0)
            .count() as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021c87d4`: the first word of boat record `i` = `v`.
pub fn boat_set<H: MemEnv>(h: &mut H, w: u32, i: u32, v: u32) {
    h.w32(
        w.wrapping_add(BOAT_RECORD_SIZE.wrapping_mul(i))
            .wrapping_add(0x24),
        v,
    );
}

/// C: `FUN_overlay_d_21__021c8810`: the boat flag word W+0x1B4 = 1.
pub fn boat_flag<H: MemEnv>(h: &mut H, w: u32) {
    h.w32(w + 0x1B4, 1);
}

#[cfg(test)]
mod tests;
