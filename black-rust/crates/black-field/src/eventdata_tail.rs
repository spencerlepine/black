//! The tail of F1 row 10.23 (overlay 10, `0x021643AC..0x02164768`): zone-change hooks (field_encount_st, flag
//! 0x964, the daily hidden-item respawn), the field-menu entry checks and event constructors (ROM tables
//! `0x021714DC` / `0x02171504`, called from `event_shortcut_menu.c`), two GMEVENT procs and the hidden-item flags.
//!
//! F6a-spec specced all 19 functions (`tools/rom-diff/specs/black-field/eventdata_tail.spec`); this module holds
//! the smoke port. HIDDEN ITEMS (ROM `0x02171534`, 0xB1 records of 10 bytes): `u16 +0` = event flag - 900,
//! `u8 +3` nonzero = respawns; a background event of kind 2 with script id `8000 + i` is hidden item `i`.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0216_46E8];

/// First script id of the hidden items.
pub const HIDDEN_ITEM_SCRIPT_BASE: u32 = 8000;
/// Last script id that [`hidden_item_index`] accepts (`0x225F`, 8799).
pub const HIDDEN_ITEM_SCRIPT_LAST: u32 = 0x225F;

/// C: `FUN_021646E8` (unlabeled routine at `0x021646E8`): the hidden-item index of a script id, `id - 8000` for
/// `8000 <= id <= 0x225F` (unsigned), else 0 (the same as item 0).
pub fn hidden_item_index(id: u32) -> u16 {
    if !(HIDDEN_ITEM_SCRIPT_BASE..=HIDDEN_ITEM_SCRIPT_LAST).contains(&id) {
        return 0;
    }
    (id - HIDDEN_ITEM_SCRIPT_BASE) as u16
}

#[cfg(test)]
mod tests;
