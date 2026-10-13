//! The grid MAP CTRL callbacks (`fieldmap_ctrl_grid.c`, ROM table `0x021D88F8`) and `field_player.c` (overlay 21,
//! `0x021A2D88..0x021A36E4`): FIELD_PLAYER, the 0x10-byte front object of the player (`+0` FIELDMAP, `+4` -> PLAYER
//! CORE (`crate::player_core`), `+8` -> PLAYER GRID (`crate::player_grid`, grid maps), `+0xC` -> the no-grid player
//! (F3b, rail maps)); most of its functions pick the grid or the no-grid half by the map kind (`FUN_02188CFC`) or
//! tail-call the core.
//!
//! F3a-spec specced all 72 functions (`tools/rom-diff/specs/black-field/player.spec`, header: records, move forms);
//! this module holds the smoke ports. The move-form table (ROM `0x021D890C`, 2 rows (player sex) x 10 entries
//! `{u16 OBJ code, u16 form}`) is embedded as [`FORM_TABLE`]: form 0 walk (codes 1 / 4), 1 cycle (2 / 5), 2 and 3
//! surf (3 / 6), 4 = the special codes 0x88.. (entries 4..9).

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021A_3190, 0x021A_31A4, 0x021A_31D0, 0x021A_36E0];

/// ROM `0x021D890C`: per player sex (row), 10 entries `(OBJ code, form)`.
pub const FORM_TABLE: [[(u16, u16); 10]; 2] = [
    [
        (0x01, 0),
        (0x02, 1),
        (0x03, 2),
        (0x03, 3),
        (0x88, 4),
        (0x8A, 4),
        (0x8C, 4),
        (0x9E, 4),
        (0xA0, 4),
        (0x9C, 4),
    ],
    [
        (0x04, 0),
        (0x05, 1),
        (0x06, 2),
        (0x06, 3),
        (0x89, 4),
        (0x8B, 4),
        (0x8D, 4),
        (0x9F, 4),
        (0xA1, 4),
        (0x9D, 4),
    ],
];

/// C: `FUN_021A3190` (unlabeled routine at `0x021A3190`): the OBJ code of entry `i` of row `sex` of the form table.
/// The original does not check the indices (it reads past the 0x50-byte table); the port panics outside
/// `sex < 2`, `i < 10` (UNVERIFIED).
pub fn entry_code(sex: u32, i: u32) -> u16 {
    FORM_TABLE[sex as usize][i as usize].0
}

/// C: `FUN_021A31A4` (unlabeled routine at `0x021A31A4`): the OBJ code of the first entry of row `sex` whose form is
/// `form` (the whole register is compared with the u16 form), 1 when there is none.
pub fn form_code(sex: u32, form: u32) -> u16 {
    FORM_TABLE[sex as usize]
        .iter()
        .find(|e| e.1 as u32 == form)
        .map_or(1, |e| e.0)
}

/// C: `FUN_021A31D0` (unlabeled routine at `0x021A31D0`): the entry index (0..9) of OBJ code `code`, searching row 0
/// then row 1 (the whole register is compared with the u16 code); 10 when the code is not a player code.
pub fn code_entry(code: u32) -> u32 {
    for row in &FORM_TABLE {
        if let Some(i) = row.iter().position(|e| e.0 as u32 == code) {
            return i as u32;
        }
    }
    10
}

/// C: `FUN_021A36E0` (unlabeled routine at `0x021A36E0`): FIELD_PLAYER `+8`, the grid player.
pub fn grid_of(mem: &Mem, player: u32) -> u32 {
    mem.r32(player.wrapping_add(8))
}

#[cfg(test)]
mod tests;
