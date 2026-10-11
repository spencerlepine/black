//! Pure battle-main utilities (asm/battle_server.s): sides and positions.

/// C: `FUN_overlay_d_93__021b9a40`: side of a battle Pokemon ID (0 for 0..12, 1 above).
pub fn poke_side(poke_id: u32) -> u8 {
    (poke_id >= 12) as u8
}

/// C: `FUN_overlay_d_93__021b9a24`: both Pokemon are on the same side.
pub fn same_side(a: u32, b: u32) -> bool {
    poke_side(a) == poke_side(b)
}

/// C: `FUN_overlay_d_93__021b9a5c`: the other side.
pub fn other_side(side: u32) -> u8 {
    (side == 0) as u8
}

/// C: `FUN_overlay_d_93__021b9a4c`: the opposing side of a Pokemon.
pub fn opposing_side(poke_id: u32) -> u8 {
    other_side(poke_side(poke_id) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sides() {
        assert_eq!(poke_side(11), 0);
        assert_eq!(poke_side(12), 1);
        assert!(same_side(0, 5));
        assert!(!same_side(3, 14));
        assert_eq!(opposing_side(13), 0);
        assert_eq!(other_side(7), 0);
    }
}
