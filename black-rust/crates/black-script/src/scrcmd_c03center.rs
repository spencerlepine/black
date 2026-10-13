//! F8 (fourth batch): overlay-21 `scrcmd_c03center.c`, Thumb code 0x021B5A18-0x021B66B8 (F1-SPLIT row 21.50): the turn tween
//! 307, party / record queries, move tutors (484..487), in-game trades (446 / 447) and name registration (22 command ids, 28
//! functions). Nothing in this file touches the comm / C-Gear state (no X1 networking).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_c03center.spec`. The functions listed in
//! [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_c03center`; the rest are the F8 port
//! job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_5DD0,
    0x021B_5F24,
    0x021B_5FB4,
    0x021B_5FB8,
    0x021B_6124,
];

/// Tutor kinds (the first operand of commands 484..487).
pub const TUTOR_DRACO: u32 = 0;
pub const TUTOR_ULTIMATE: u32 = 1;
pub const TUTOR_PLEDGE: u32 = 2;
/// Move ids the tutors teach: Draco Meteor, the three ultimate moves, the three pledges.
pub const MOVE_DRACO_METEOR: u32 = 0x1B2;
pub const MOVE_FRENZY_PLANT: u32 = 0x152;
pub const MOVE_BLAST_BURN: u32 = 0x133;
pub const MOVE_HYDRO_CANNON: u32 = 0x134;
pub const MOVE_GRASS_PLEDGE: u32 = 0x208;
pub const MOVE_FIRE_PLEDGE: u32 = 0x207;
pub const MOVE_WATER_PLEDGE: u32 = 0x206;
/// Level a tutored mon must reach (`FUN_021B5FB4`: 0xFF, i.e. no level check in effect).
pub const TUTOR_MAX_LEVEL: u8 = 0xFF;

/// Callees of the scrcmd_c03center functions that this area does not port, named by address (meaning per the spec).
pub trait C03Hooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): the address of the next var operand.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next value operand.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215B4AC(ctx, slot, &mon)` (scrcmd_work): the party mon of `slot` into `mon`; 1 when there is one.
    fn fun_0215b4ac(&mut self, ctx: u32, slot: u32, mon: &mut [u8; 4]) -> u32;
    /// C: `FUN_0201891C(mon, k)` (main): a PokeParam type / flag test (UNVERIFIED: has type k).
    fn fun_0201891c(&mut self, mon: u32, k: u32) -> u32;
}

/// C: `FUN_overlay_d_21__021b5dd0`: the value of the first `{u16 key, u16 value}` pair whose key is 0 or not below `v`.
pub fn table_lookup<H: MemEnv>(h: &mut H, table: u32, v: u32) -> u16 {
    let mut p = table;
    loop {
        let key = h.r16(p);
        if key == 0 || v as i32 <= i32::from(key) {
            return h.r16(p + 2);
        }
        p += 4;
    }
}

/// C: `FUN_overlay_d_21__021b5f24`: the move tutor `k` teaches `mon`: Draco Meteor (0), the ultimate move (1) or the pledge (2)
/// of the first matching `FUN_0201891C` test (3 / 4 / 5 resp. 0 / 1 / 2); Blast Burn when none matches or for another `k`.
pub fn tutor_move<H: C03Hooks>(h: &mut H, mon: u32, k: u32) -> u32 {
    match k {
        TUTOR_DRACO => MOVE_DRACO_METEOR,
        TUTOR_ULTIMATE => {
            if h.fun_0201891c(mon, 3) != 0 {
                MOVE_FRENZY_PLANT
            } else if h.fun_0201891c(mon, 4) != 0 {
                MOVE_BLAST_BURN
            } else if h.fun_0201891c(mon, 5) != 0 {
                MOVE_HYDRO_CANNON
            } else {
                MOVE_BLAST_BURN
            }
        }
        TUTOR_PLEDGE => {
            if h.fun_0201891c(mon, 0) != 0 {
                MOVE_GRASS_PLEDGE
            } else if h.fun_0201891c(mon, 1) != 0 {
                MOVE_FIRE_PLEDGE
            } else if h.fun_0201891c(mon, 2) != 0 {
                MOVE_WATER_PLEDGE
            } else {
                MOVE_BLAST_BURN
            }
        }
        _ => MOVE_BLAST_BURN,
    }
}

/// C: `FUN_overlay_d_21__021b5fb4`: the level a tutored mon must have (0xFF for every tutor).
pub fn tutor_max_level(_k: u32) -> u8 {
    TUTOR_MAX_LEVEL
}

/// C: `FUN_overlay_d_21__021b5fb8`: whether tutor `k` can teach `mon`: k 0 test 6; k 1 any of the tests 3 / 4 / 5; k 2 any of
/// 0 / 1 / 2 (in that order, stopping at the first hit); other `k` no.
pub fn tutor_can_learn<H: C03Hooks>(h: &mut H, mon: u32, k: u32) -> bool {
    let tests: &[u32] = match k {
        TUTOR_DRACO => &[6],
        TUTOR_ULTIMATE => &[3, 4, 5],
        TUTOR_PLEDGE => &[0, 1, 2],
        _ => &[],
    };
    tests.iter().any(|&t| h.fun_0201891c(mon, t) != 0)
}

/// C: `sub_021b6124`: command 486: `k`, `slot`, `var`; `*var` = the tutor move of the party mon (0 when the slot is empty); 0.
pub fn cmd_486<H: C03Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let mut mon = [0u8; 4];
    let v = if h.fun_0215b4ac(ctx, slot, &mut mon) == 1 {
        tutor_move(h, u32::from_le_bytes(mon), k)
    } else {
        0
    };
    h.w16(var, v as u16);
    0
}

#[cfg(test)]
mod tests;
