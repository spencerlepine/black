//! Undecompiled routines called by the ported code, modelled as traits.
//!
//! Implementations must reproduce the original routine; the ported code never guesses their
//! behaviour. Return types are the raw register values the callers consume.

/// C: `BattleMon_Get` value IDs used by the ported code.
///
/// The IDs are verified against `BattleMon_Get`'s jump table (asm/battle_pokeparam.s); the names
/// are inferred from the field each ID reads and how callers use it.
pub mod bpp {
    /// Reads `u16 @ +0x0E`. Used as max HP (weather residual = value / 16).
    pub const MAX_HP: u32 = 0x0E;
    /// Reads `u8 @ +0x18`. Used as level (EXP yield and prize money formulas).
    pub const LEVEL: u32 = 0x0F;
    /// Reads `u8 @ +0x141`. Passed with the species to the personal-data getter, so the form.
    pub const FORM: u32 = 0x13;
}

/// A battle Pokemon (`BTL_POKEPARAM`, overlay 93). Only the accessors the ported code calls.
pub trait BattleMon {
    /// C: `BattleMon_Get` (0x021D59B4). Returns the raw 32-bit result.
    fn get(&self, id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021d5814`: true if either type equals `ty` (always false for 0x11).
    fn has_type(&self, ty: u32) -> bool;
    /// C: `FUN_overlay_d_93__021d5440` (`ldrb [bpp, #0x19]`), the battle Pokemon ID.
    fn poke_id(&self) -> u8;
    /// C: `FUN_overlay_d_93__021d5444` (`ldrh [bpp, #0x0c]`), the species.
    fn species(&self) -> u16;
}

/// A battle party: the two accessors used by the prize-money calculation.
pub trait BattleParty {
    type Mon: BattleMon;
    /// C: `FUN_overlay_d_93__021b9b94`: number of members.
    fn count(&self) -> u32;
    /// C: `FUN_overlay_d_93__021b9c00`: member at `index`.
    fn member(&self, index: u8) -> &Self::Mon;
}

/// Personal (species) data.
pub trait PersonalData {
    /// C: `FUN_02019C90(species, form, param)`.
    fn param(&self, species: u16, form: u16, param: u32) -> u32;
}

/// ARM9 routines with unknown semantics.
pub trait Arm9Misc {
    /// C: `FUN_0202A3FC`: looks up a 4-bit field (bits 7-10 of a `u16` table at 0x0209FEE4)
    /// for `FUN_0202A3D8(value)`, or 0xC when that index is >= 0x1C.
    fn fun_0202a3fc(&self, value: u32) -> u32;
}
