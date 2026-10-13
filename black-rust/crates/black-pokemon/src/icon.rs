//! Pokemon icon helpers (P4): which ARC 7 member holds a Pokemon's icon graphics, cells, animations and
//! palette (main ARM9 0x0201BA68-0x0201BD28, between `shinka_check.c` and `waza_tool.c`; T1 counted it as
//! `waza_tool.c`, and Ghidra merged 0x0201BC40.. with `FUN_0201BD28`, hence "`FUN_0201BC40` loads ARC 21").
//!
//! ARC 7 (1431 members): member 0 = the palette file ([`palette_member`]), 1..=6 = the cell and animation
//! files for the OBJ VRAM mapping mode ([`cell_member`], [`anim_member`]), 7.. = the icon graphics
//! ([`graphics_member`]): two per species (male / female), then the two eggs, then the alternate forms.
//! The palette number of an icon is a nibble of a 712-byte table (main 0x0209E474, [`palette_index`]); the
//! table comes in as a slice (asset data, D4), the archive and personal data through [`IconHooks`].
//!
//! Verified per function with `tools/check-fn.sh black-pokemon icon --per-function`
//! (`tools/rom-diff/specs/black-pokemon/icon.spec`).

use crate::hooks::PokeToolHooks;
use crate::poke_tool::{box_get_value, decrypt_on_access, reencrypt};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_BA68,
    0x0201_BABC,
    0x0201_BB5C,
    0x0201_BBE8,
    0x0201_BC3C,
    0x0201_BC40,
    0x0201_BC78,
    0x0201_BCB4,
    0x0201_BCEC,
];

/// ARC id of the Pokemon icons.
pub const ICON_ARC: u32 = 7;
/// Address of the icon palette table in the original (main ARM9; 712 bytes, one per icon).
pub const PALETTE_TABLE: u32 = 0x0209_E474;
/// Entries of the palette table: 650 species, 2 eggs, 60 alternate forms.
pub const PALETTE_TABLE_LEN: usize = 712;
/// Species whose egg has its own icon (Manaphy).
pub const MANAPHY: u32 = 490;
/// Icon index of the egg (Manaphy's egg is the next one).
pub const EGG_INDEX: u32 = 0x28A;
/// First graphics member of the alternate forms.
pub const FORM_GRAPHICS_BASE: u32 = 0x51F;
/// Personal-data fields read through [`PokeToolHooks::personal_param`] (`FUN_02019C90(species, 0, field)`).
pub mod personal_field {
    /// Index of the species' first alternate form in the per-form tables.
    pub const FIRST_FORM: u32 = 0x1F;
    /// Number of forms (a form at or above it counts as form 0).
    pub const FORM_COUNT: u32 = 0x20;
    /// Nonzero: the forms have no icons of their own.
    pub const NO_FORM_ICONS: u32 = 0x23;
}

/// Callees of the icon routines beyond the Pokemon accessors.
///
/// The differential test stubs them identically on both sides, so only the arguments are verified.
pub trait IconHooks: PokeToolHooks {
    /// Callee `FUN_020490A8(arc, member)` (`ArcMemberSize`): the size of an archive member (0 = empty).
    fn arc_member_size(&mut self, arc: u32, member: u32) -> u32;
}

/// C: `FUN_0201BC3C`: the palette member of ARC 7 (0).
pub fn palette_member() -> u32 {
    0
}

/// The OBJ mapping bits of a DISPCNT value (`0x00300010`: 1D mode bit 4, boundary bits 20-21) as 0..=3 for
/// 1D 32K / 64K / 128K / anything else.
fn obj_mapping(dispcnt: u32) -> u32 {
    match dispcnt & 0x0030_0010 {
        0x10 => 0,
        0x0010_0010 => 1,
        0x0020_0010 => 2,
        _ => 3,
    }
}

/// C: `FUN_0201BC40` (main engine, `REG_DISPCNT` 0x04000000) and `FUN_0201BC78` (sub engine,
/// `REG_DISPCNT_SUB` 0x04001000): the cell member for the OBJ mapping mode in `dispcnt` (the original reads
/// the register itself): 1D 32K 2, 64K 4, 128K and anything else 6.
pub fn cell_member(dispcnt: u32) -> u32 {
    [2, 4, 6, 6][obj_mapping(dispcnt) as usize]
}

/// C: `FUN_0201BCB4` (main engine) and `FUN_0201BCEC` (sub engine): the animation member for the OBJ
/// mapping mode in `dispcnt`: 1D 32K 1, 64K 3, 128K and anything else 5.
pub fn anim_member(dispcnt: u32) -> u32 {
    [1, 3, 5, 5][obj_mapping(dispcnt) as usize]
}

/// The form's index in the per-form tables when the species has icons for its forms (`form` nonzero, below
/// the form count, forms with icons), else `None` (`form` at or above the count counts as 0).
fn form_slot(species: u32, form: u32, h: &mut impl PokeToolHooks) -> Option<u32> {
    let sp = species & 0xFFFF;
    let first = h.personal_param(sp, 0, personal_field::FIRST_FORM);
    let shared = h.personal_param(sp, 0, personal_field::NO_FORM_ICONS);
    let count = h.personal_param(sp, 0, personal_field::FORM_COUNT);
    let form = if form >= count { 0 } else { form };
    (shared == 0 && form != 0).then(|| first.wrapping_add(form))
}

/// C: `FUN_0201BABC(species, form, female, egg)`: the ARC 7 member of the icon graphics. Eggs: `(0x28A +
/// (species == 490)) * 2 + 7`; a form with its own icon: `0x51F + (first form + form - 1) * 2`; else
/// `species * 2 + 7`. `female` 1 adds 1 when the next member is non-empty, 0 and 2 add nothing, any other
/// value is added as it is.
pub fn graphics_member(
    species: u32,
    form: u32,
    female: u32,
    egg: u32,
    h: &mut impl IconHooks,
) -> u32 {
    let mut m = species.wrapping_mul(2).wrapping_add(7);
    if egg != 0 {
        m = (EGG_INDEX + (species == MANAPHY) as u32) * 2 + 7;
    } else if form != 0 {
        if let Some(slot) = form_slot(species, form, h) {
            m = slot
                .wrapping_sub(1)
                .wrapping_mul(2)
                .wrapping_add(FORM_GRAPHICS_BASE);
        }
    }
    let add = match female {
        0 | 2 => 0,
        1 => (h.arc_member_size(ICON_ARC, m.wrapping_add(1)) != 0) as u32,
        v => v,
    };
    m.wrapping_add(add)
}

/// The palette-table index of an icon: eggs `0x28A + (species == 490)`, a form with its own icon
/// `0x28B + first form + form`, else the species.
pub fn palette_index(species: u32, form: u32, egg: u32, h: &mut impl PokeToolHooks) -> u32 {
    if egg != 0 {
        EGG_INDEX + (species == MANAPHY) as u32
    } else if form == 0 {
        species
    } else {
        match form_slot(species, form, h) {
            Some(slot) => slot.wrapping_add(EGG_INDEX + 1),
            None => species,
        }
    }
}

/// C: `FUN_0201BB5C(species, form, high, egg)`: the icon's palette number, a nibble of `table` (the 712-byte
/// table at main 0x0209E474) at [`palette_index`]: the high nibble when `high` is nonzero, else the low one.
pub fn palette_of(
    species: u32,
    form: u32,
    high: u32,
    egg: u32,
    table: &[u8],
    h: &mut impl PokeToolHooks,
) -> u32 {
    let b = table[palette_index(species, form, egg, h) as usize] as u32;
    if high != 0 {
        b >> 4
    } else {
        b & 0xF
    }
}

/// The species (5), form (0x6F), gender (0x6E) and egg flag (0x4C) of a `BoxPokemon`, read inside one
/// decrypt-on-access bracket.
fn icon_fields(mon: &mut [u8], h: &mut impl PokeToolHooks) -> [u32; 4] {
    let enter = decrypt_on_access(mon) as u32;
    let f = [5, 0x6F, 0x6E, 0x4C].map(|id| box_get_value(mon, id, 0, h));
    reencrypt(mon, enter);
    f
}

/// C: `FUN_0201BA68(mon)`: [`graphics_member`] of a `BoxPokemon` (species, form, gender as `female`, egg).
pub fn mon_graphics_member(mon: &mut [u8], h: &mut impl IconHooks) -> u32 {
    let enter = decrypt_on_access(mon) as u32;
    let f = [5, 0x6F, 0x6E, 0x4C].map(|id| box_get_value(mon, id, 0, h));
    let m = graphics_member(f[0], f[1], f[2], f[3], h);
    reencrypt(mon, enter);
    m
}

/// C: `FUN_0201BBE8(mon)`: [`palette_of`] of a `BoxPokemon` (species, form, gender as `high`, egg).
pub fn mon_palette(mon: &mut [u8], table: &[u8], h: &mut impl PokeToolHooks) -> u32 {
    let f = icon_fields(mon, h);
    palette_of(f[0], f[1], f[2], f[3], table, h)
}

#[cfg(test)]
mod tests;
