//! Status-condition continuation ("sick cont") bitfields (asm/battle_view_core.s, overlay 93).
//!
//! A [`SickCont`] is a `u32` describing how long a status condition lasts:
//!
//! | bits  | meaning                                                    |
//! |-------|------------------------------------------------------------|
//! | 0-2   | kind: 0 none, 1 permanent, 2 turns, 3 Pokemon-linked, 4 Pokemon + turns |
//! | 3-8   | turns (kinds 2, 4), Pokemon ID (kind 3), counter (kind 1)  |
//! | 9-24  | `u16` parameter (kinds 1, 2)                               |
//! | 9-14  | Pokemon ID (kind 4)                                        |

/// C: `BPP_SICK_CONT`-style packed value.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SickCont(pub u32);

pub const KIND_NONE: u32 = 0;
pub const KIND_PERMANENT: u32 = 1;
pub const KIND_TURN: u32 = 2;
pub const KIND_POKE: u32 = 3;
pub const KIND_POKE_TURN: u32 = 4;

const KIND_MASK: u32 = 0x7;

// The originals build the value in a zeroed stack word and clear each field with a mask
// (0xFFFFFE07, 0xFE0001FF, 0xFFFF81FF) before OR-ing it in; the fields never overlap, so plain
// ORs are equivalent.

fn slot(v: u32) -> u32 {
    (v & 0x3F) << 3
}

fn param(v: u32) -> u32 {
    (v & 0xFFFF) << 9
}

impl SickCont {
    pub const NONE: SickCont = SickCont(0);

    /// C: `FUN_overlay_d_93__021e8940`: lasts `turns` turns.
    pub fn turn(turns: u32) -> Self {
        SickCont(KIND_TURN | slot(turns))
    }

    /// C: `FUN_overlay_d_93__021e896c`: lasts `turns` turns, with a `u16` parameter.
    pub fn turn_param(turns: u32, p: u32) -> Self {
        SickCont(KIND_TURN | slot(turns) | param(p))
    }

    /// C: `FUN_overlay_d_93__021e899c`: lasts while Pokemon `poke_id` is present.
    pub fn poke(poke_id: u32) -> Self {
        SickCont(KIND_POKE | slot(poke_id))
    }

    /// C: `FUN_overlay_d_93__021e89c0`: permanent.
    pub fn permanent() -> Self {
        SickCont(KIND_PERMANENT)
    }

    /// C: `FUN_overlay_d_93__021e89d4`: permanent, with a `u16` parameter.
    pub fn permanent_param(p: u32) -> Self {
        SickCont(KIND_PERMANENT | param(p))
    }

    /// C: `FUN_overlay_d_93__021e89f8`: permanent, with a 6-bit counter and `u16` parameter.
    pub fn permanent_count(count: u32, p: u32) -> Self {
        SickCont(KIND_PERMANENT | slot(count) | param(p))
    }

    /// C: `FUN_overlay_d_93__021e8a28`: linked to `poke_id` and lasting `turns` turns.
    pub fn poke_turn(poke_id: u32, turns: u32) -> Self {
        SickCont(KIND_POKE_TURN | slot(turns) | ((poke_id & 0x3F) << 9))
    }

    /// C: `FUN_overlay_d_93__021e8a58`: permanent with counter 15 (badly poisoned).
    pub fn badly_poisoned() -> Self {
        SickCont(KIND_PERMANENT | slot(0xF))
    }

    /// C: `FUN_overlay_d_93__021e8a78`: kind is permanent and counter is 15.
    pub fn is_badly_poisoned(self) -> bool {
        self.kind() == KIND_PERMANENT && self.slot() == 0xF
    }

    /// C: `FUN_overlay_d_93__021e8918`
    pub fn none() -> Self {
        SickCont::NONE
    }

    /// C: `FUN_overlay_d_93__021e8928`: kind is 0.
    pub fn is_none(self) -> bool {
        self.kind() == KIND_NONE
    }

    /// C: `FUN_overlay_d_93__021e8a98`: linked Pokemon ID (kinds 3 and 4), else 0x1F.
    pub fn linked_poke_id(self) -> u32 {
        match self.kind() {
            KIND_POKE => self.slot(),
            KIND_POKE_TURN => (self.0 >> 9) & 0x3F,
            _ => 0x1F,
        }
    }

    /// C: `FUN_overlay_d_93__021e8ac8`: replaces the linked Pokemon ID (kinds 3 and 4).
    pub fn set_linked_poke_id(&mut self, id: u32) {
        match self.kind() {
            KIND_POKE => self.0 = (self.0 & 0xFFFF_FE07) | slot(id),
            KIND_POKE_TURN => self.0 = (self.0 & 0xFFFF_81FF) | ((id & 0x3F) << 9),
            _ => {}
        }
    }

    /// C: `FUN_overlay_d_93__021e8afc`: turn limit (kinds 2 and 4), else 0.
    pub fn turns(self) -> u32 {
        match self.kind() {
            KIND_TURN | KIND_POKE_TURN => self.slot(),
            _ => 0,
        }
    }

    /// C: `FUN_overlay_d_93__021e8b80`: the `u16` parameter (bits 9-24 for kinds 1-3, bits
    /// 15-30 for kind 4), else 0.
    pub fn param_any(self) -> u16 {
        match self.kind() {
            KIND_PERMANENT | KIND_TURN | KIND_POKE => (self.0 >> 9) as u16,
            KIND_POKE_TURN => (self.0 >> 15) as u16,
            _ => 0,
        }
    }

    /// C: unlabeled routine at `0x021E8B28`: sets the parameter of [`Self::param_any`].
    pub fn set_param_any(&mut self, p: u32) {
        match self.kind() {
            KIND_PERMANENT | KIND_POKE | KIND_TURN => {
                self.0 = (self.0 & 0xFE00_01FF) | ((p << 16) >> 7)
            }
            KIND_POKE_TURN => self.0 = (self.0 & 0x8000_7FFF) | ((p << 16) >> 1),
            _ => {}
        }
    }

    /// C: unlabeled routine at `0x021E8BD0`: sets a flag bit (bit 25 for kinds 1-3, bit 31
    /// for kind 4).
    pub fn set_flag(&mut self, f: u32) {
        match self.kind() {
            KIND_PERMANENT | KIND_POKE | KIND_TURN => {
                self.0 = (self.0 & 0xFDFF_FFFF) | ((f << 31) >> 6)
            }
            KIND_POKE_TURN => self.0 = (self.0 & 0x7FFF_FFFF) | (f << 31),
            _ => {}
        }
    }

    /// C: unlabeled routine at `0x021E8C24`: the flag bit of [`Self::set_flag`].
    pub fn flag(self) -> u32 {
        match self.kind() {
            KIND_PERMANENT | KIND_POKE | KIND_TURN => (self.0 >> 25) & 1,
            KIND_POKE_TURN => self.0 >> 31,
            _ => 0,
        }
    }

    /// C: unlabeled routine at `0x021E8C50`: adds to the turn limit if it is below 8.
    pub fn add_turns(&mut self, n: u32) {
        if self.kind() == KIND_TURN && self.slot() < 8 {
            self.0 = (self.0 & 0xFFFF_FE07) | slot(self.slot().wrapping_add(n));
        }
        if self.kind() == KIND_POKE_TURN && self.slot() < 8 {
            self.0 = (self.0 & 0xFFFF_FE07) | slot(self.slot().wrapping_add(n));
        }
    }

    /// C: unlabeled routine at `0x021E8C9C`: sets the turn limit if it is below 8.
    pub fn set_turns(&mut self, n: u32) {
        if self.kind() == KIND_TURN && self.slot() < 8 {
            self.0 = (self.0 & 0xFFFF_FE07) | slot(n);
        }
        if self.kind() == KIND_POKE_TURN && self.slot() < 8 {
            self.0 = (self.0 & 0xFFFF_FE07) | slot(n);
        }
    }

    /// Bits 0-2.
    pub fn kind(self) -> u32 {
        self.0 & KIND_MASK
    }

    /// Bits 3-8.
    pub fn slot(self) -> u32 {
        (self.0 >> 3) & 0x3F
    }

    /// Bits 9-24.
    pub fn param(self) -> u32 {
        (self.0 >> 9) & 0xFFFF
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{Fnv, Rng};

    #[test]
    fn hand_checked() {
        assert_eq!(SickCont::turn(5).0, 2 | 5 << 3);
        assert_eq!(SickCont::turn(0x41).0, 2 | 1 << 3);
        assert_eq!(
            SickCont::turn_param(3, 0x1_2345).0,
            2 | 3 << 3 | 0x2345 << 9
        );
        assert_eq!(SickCont::poke(7).0, 3 | 7 << 3);
        assert_eq!(SickCont::permanent().0, 1);
        assert_eq!(SickCont::permanent_param(0xBEEF).0, 1 | 0xBEEF << 9);
        assert_eq!(SickCont::permanent_count(4, 2).0, 1 | 4 << 3 | 2 << 9);
        assert_eq!(SickCont::poke_turn(5, 3).0, 4 | 3 << 3 | 5 << 9);
        assert_eq!(SickCont::badly_poisoned().0, 0x79);
        assert!(SickCont::badly_poisoned().is_badly_poisoned());
        assert!(!SickCont::permanent().is_badly_poisoned());
        assert!(!SickCont::turn(15).is_badly_poisoned());
    }

    /// Hash of every constructor over random inputs, matching the original machine code run
    /// under an emulator (see `docs/migration/NOTES.md`, "Verification methods").
    #[test]
    fn matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut rng = Rng::new(0x5C);
        for _ in 0..2000 {
            let a = rng.next();
            let b = rng.next();
            h.w(SickCont::turn(a).0);
            h.w(SickCont::turn_param(a, b).0);
            h.w(SickCont::poke(a).0);
            h.w(SickCont::permanent().0);
            h.w(SickCont::permanent_param(a).0);
            h.w(SickCont::permanent_count(a, b).0);
            h.w(SickCont::poke_turn(a, b).0);
            h.w(SickCont::badly_poisoned().0);
            h.w(SickCont(a).is_badly_poisoned() as u32);
            h.w(SickCont(a & 0x7F).is_badly_poisoned() as u32);
        }
        assert_eq!(h.hex(), "2dbe3d01dd5611d2");
    }

    /// V-0014: every accessor and in-place setter on kinds 0-4 (biased), 5-7 and random words;
    /// hashed against the original (`tools/rom-diff/gen.py sick_cont_wide`).
    #[test]
    fn wide_matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut rng = Rng::new(0x5D);
        for i in 0..60000u32 {
            let a = rng.next();
            let b = rng.next();
            let k = if i % 7 != 0 { i % 5 } else { rng.next() & 7 };
            let mut v = (a & !7) | k;
            if i % 11 == 0 {
                v = a;
            }
            h.w(SickCont::none().0);
            h.w(SickCont(v).is_none() as u32);
            h.w(SickCont(v).linked_poke_id());
            h.w(SickCont(v).turns());
            h.w(SickCont(v).param_any() as u32);
            h.w(SickCont(v).flag());
            for f in 0..5 {
                for arg in [b, b & 0xF] {
                    let mut s = SickCont(v);
                    match f {
                        0 => s.set_linked_poke_id(arg),
                        1 => s.set_param_any(arg),
                        2 => s.set_flag(arg),
                        3 => s.add_turns(arg),
                        _ => s.set_turns(arg),
                    }
                    h.w(s.0);
                }
            }
            h.w(SickCont::turn(a).0);
            h.w(SickCont::turn_param(a, b).0);
            h.w(SickCont::poke(a).0);
            h.w(SickCont::permanent_param(a).0);
            h.w(SickCont::permanent_count(a, b).0);
            h.w(SickCont::poke_turn(a, b).0);
            h.w(SickCont(v).is_badly_poisoned() as u32);
            h.w(SickCont(0x79 | (a & !0xFF)).is_badly_poisoned() as u32);
        }
        assert_eq!(h.hex(), "bb51b5141039826f");
    }
}
