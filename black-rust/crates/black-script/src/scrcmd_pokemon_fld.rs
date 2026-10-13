//! F8 (fourth batch): overlay-21 `scrcmd_pokemon_fld.c`, Thumb code 0x021B66B8-0x021B7400 (F1-SPLIT row 21.51): party / box
//! Pokemon commands, calendar names, record tables, the Dream World delivery table (cmd 494; X2 scope) and ARC 208 lookups
//! (14 command ids, 48 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_pokemon_fld.spec`. The functions listed
//! in [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_pokemon_fld`; the rest are the F8 port
//! job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_6AA4,
    0x021B_6AAC,
    0x021B_6AB0,
    0x021B_7364,
    0x021B_7390,
];

/// The u16 table at 0x021D99E8 (overlay 21 ROM) that command 564 indexes (11 entries).
pub const TABLE_564: [u16; 11] = [
    0x75, 0x75, 0x76, 0x85, 0xF8, 0xF9, 0xFA, 0xFB, 0xFC, 0xFD, 0xFE,
];
/// The `month << 8 | day` days at 0x021D99D4 (overlay 21 ROM) that `FUN_021B7390` recognises.
pub const SPECIAL_DAYS: [u16; 5] = [0x0A01, 0x0A0E, 0x0C1E, 0x0201, 0x060C];
/// Records of the 10-byte table that command 460 / 461 walk; the list ends at an id of 0xFFFF (at most 8 records).
pub const RECORD_SIZE: u32 = 10;
pub const RECORD_MAX: u32 = 8;

/// Callees of the scrcmd_pokemon_fld functions that this area does not port, named by address (meaning per the spec).
pub trait PokemonFldHooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): the address of the next var operand.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next value operand.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
}

/// C: `FUN_overlay_d_21__021b6aa4`: record `i` of a 10-byte record table.
pub fn record_at(table: u32, i: u32) -> u32 {
    table.wrapping_add(i.wrapping_mul(RECORD_SIZE))
}

/// C: `FUN_overlay_d_21__021b6aac`: an empty function (`bx lr`; called on each record by [`record_count`]).
pub fn record_nop(_rec: u32) {}

/// C: `FUN_overlay_d_21__021b6ab0`: the index of the first record (of 8) whose u16 id is 0xFFFF; 0 when none is.
pub fn record_count<H: MemEnv>(h: &mut H, table: u32) -> u32 {
    for i in 0..RECORD_MAX {
        let rec = record_at(table, i);
        record_nop(rec);
        if h.r16(rec) == 0xFFFF {
            return i;
        }
    }
    0
}

/// C: `sub_021b7364`: command 564: `v`, `var`; `*var = TABLE_564[v < 0xB ? v : 0]`; 0.
pub fn cmd_564<H: PokemonFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let i = if v < TABLE_564.len() as u32 { v } else { 0 };
    h.w16(var, TABLE_564[i as usize]);
    0
}

/// C: `FUN_overlay_d_21__021b7390`: `(month << 8 | day)` (u16) is one of the [`SPECIAL_DAYS`].
pub fn is_special_day(month: u32, day: u32) -> bool {
    let x = ((month << 8) | day) as u16;
    SPECIAL_DAYS.contains(&x)
}

#[cfg(test)]
mod tests;
