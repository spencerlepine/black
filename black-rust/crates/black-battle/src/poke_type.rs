//! Packed type pair helpers (asm/battle_view_core.s, overlay 93).
//!
//! A type pair is a `u16`: `type1 << 8 | type2`. A single-type Pokemon stores the same type
//! twice.

pub use black_core::ids::types;

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
    use crate::testutil::{Fnv, Rng};

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

    /// Hashed against the original code run under an emulator (`tools/rom-diff`).
    #[test]
    fn matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut r = Rng::new(0x7E);
        for _ in 0..3000 {
            let a = r.next() as u8;
            let b = (r.next() % 20) as u8;
            let c = (r.next() % 20) as u8;
            h.w(make_pair(a, b) as u32);
            h.w(make_mono_pair(a) as u32);
            let p = make_pair(b, c);
            h.w(pair_type1(p) as u32);
            h.w(pair_type2(p) as u32);
            h.w(pair_has_type(p, a % 20) as u32);
            h.w(pair_is_mono(p) as u32);
        }
        assert_eq!(h.hex(), "525001f51ea0442c");
    }
}
