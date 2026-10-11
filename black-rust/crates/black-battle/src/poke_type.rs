//! Packed type pair helpers (asm/battle_view_core.s, overlay 93).
//!
//! A type pair is a `u16`: `type1 << 8 | type2`. A single-type Pokemon stores the same type
//! twice.

/// Type IDs (Gen 5 order), as indexed by the type chart in `battle_calc.s`.
pub mod types {
    pub const NORMAL: u8 = 0;
    pub const FIGHTING: u8 = 1;
    pub const FLYING: u8 = 2;
    pub const POISON: u8 = 3;
    pub const GROUND: u8 = 4;
    pub const ROCK: u8 = 5;
    pub const BUG: u8 = 6;
    pub const GHOST: u8 = 7;
    pub const STEEL: u8 = 8;
    pub const FIRE: u8 = 9;
    pub const WATER: u8 = 10;
    pub const GRASS: u8 = 11;
    pub const ELECTRIC: u8 = 12;
    pub const PSYCHIC: u8 = 13;
    pub const ICE: u8 = 14;
    pub const DRAGON: u8 = 15;
    pub const DARK: u8 = 16;
    /// Number of real types (rows/columns of the type chart).
    pub const COUNT: u8 = 17;
    /// "No type" (`0x11`): always neutral and never matched by `has_type`.
    pub const NONE: u8 = 17;
}

/// C: `FUN_overlay_d_93__021e8ce0`
pub fn make_pair(type1: u8, type2: u8) -> u16 {
    ((type1 as u16) << 8) | type2 as u16
}

/// C: `FUN_overlay_d_93__021e8cf0` (both slots set to `ty`).
pub fn make_mono_pair(ty: u8) -> u16 {
    make_pair(ty, ty)
}

/// C: `FUN_overlay_d_93__021e8cfc`
pub fn pair_type1(pair: u16) -> u8 {
    (pair >> 8) as u8
}

/// C: `FUN_overlay_d_93__021e8d04`
pub fn pair_type2(pair: u16) -> u8 {
    pair as u8
}

/// C: `FUN_overlay_d_93__021e8d0c`. Returns `(type1, type2)`.
pub fn split_pair(pair: u16) -> (u8, u8) {
    (pair_type1(pair), pair_type2(pair))
}

/// C: `FUN_overlay_d_93__021e8d24`
pub fn pair_has_type(pair: u16, ty: u8) -> bool {
    pair_type1(pair) == ty || pair_type2(pair) == ty
}

/// C: `FUN_overlay_d_93__021e8da8`: both slots hold the same type.
pub fn pair_is_mono(pair: u16) -> bool {
    pair_type1(pair) == pair_type2(pair)
}

#[cfg(test)]
mod tests {
    use super::types::*;
    use super::*;

    #[test]
    fn pair_roundtrip() {
        let p = make_pair(WATER, GROUND);
        assert_eq!(p, 0x0A04);
        assert_eq!(split_pair(p), (WATER, GROUND));
        assert!(pair_has_type(p, GROUND));
        assert!(!pair_has_type(p, FIRE));
        assert!(!pair_is_mono(p));
        assert!(pair_is_mono(make_mono_pair(FIRE)));
        assert_eq!(make_mono_pair(DARK), 0x1010);
    }
}
