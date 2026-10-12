//! Action records: asm/battle_calc.s `0x021D82D0`-`0x021D8454` (overlay 93), plus the two
//! `.byte`-encoded routines at `0x021D8318` and `0x021D8424` between them.
//!
//! A record is one `u32`: bits 0-3 are a tag, the rest depends on the tag. The roles below are
//! inferred from the field widths and the callers' use (fight, item, switch); the tag numbers
//! are exact.

/// Tag 0: empty record (`FUN_overlay_d_93__021d8434`).
pub const TAG_NONE: u32 = 0;
/// Tag 1: fight - move (bits 7-22), target position (bits 4-6), flag (bit 26).
pub const TAG_FIGHT: u32 = 1;
/// Tag 2: item - item (bits 7-22), position (bits 4-6), argument (bits 23-30).
pub const TAG_ITEM: u32 = 2;
/// Tag 3: switch - two 3-bit values at bits 4-6 and 7-9, flag at bit 10.
pub const TAG_SWITCH: u32 = 3;

const TAG_MASK: u32 = 0xF;

/// The 3-bit position stored at bits 4-6 by every constructor that takes one.
fn pos_bits(pos: u32) -> u32 {
    (pos & 7) << 4
}

/// C: the `u32` record written by `FUN_overlay_d_93__021d82d0` and friends.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ActionRecord(pub u32);

impl ActionRecord {
    /// C: `FUN_overlay_d_93__021d8454`
    pub fn tag(&self) -> u32 {
        self.0 & TAG_MASK
    }

    /// C: `FUN_overlay_d_93__021d8434`: the record becomes 0 (clears all bits).
    pub fn set_none(&mut self) {
        self.0 = (self.0 & !TAG_MASK) & TAG_MASK;
    }

    /// C: `FUN_overlay_d_93__021d82d0`: fight record; replaces the whole word.
    pub fn set_fight(&mut self, waza: u32, pos: u32) {
        self.0 = TAG_FIGHT | pos_bits(pos) | (waza & 0xFFFF) << 7;
    }

    /// C: `FUN_overlay_d_93__021d82fc`: replaces the position of a fight record, unless the
    /// new position is 6 (none) or the record is not a fight.
    pub fn set_fight_pos(&mut self, pos: u32) {
        if self.tag() == TAG_FIGHT && pos != 6 {
            self.0 = (self.0 & !0x70) | pos_bits(pos);
        }
    }

    /// C: unlabeled routine at `0x021D8318`: sets bit 26 of a fight record.
    pub fn set_fight_flag(&mut self) {
        if self.tag() == TAG_FIGHT {
            self.0 |= 1 << 26;
        }
    }

    /// C: `FUN_overlay_d_93__021d832c`: bit 26 of a fight record, else 0.
    pub fn fight_flag(&self) -> u32 {
        if self.tag() == TAG_FIGHT {
            (self.0 >> 26) & 1
        } else {
            0
        }
    }

    /// C: `FUN_overlay_d_93__021d8340`: the move of a fight record, else 0.
    pub fn fight_waza(&self) -> u32 {
        if self.tag() == TAG_FIGHT {
            (self.0 >> 7) & 0xFFFF
        } else {
            0
        }
    }

    /// C: `FUN_overlay_d_93__021d8358`: item record; replaces the whole word.
    pub fn set_item(&mut self, item: u32, pos: u32, arg: u32) {
        self.0 = TAG_ITEM | (item & 0xFFFF) << 7 | pos_bits(pos) | (arg & 0xFF) << 23;
    }

    /// C: `FUN_overlay_d_93__021d8394`: switch record; replaces the whole word (bit 10 clear).
    pub fn set_switch(&mut self, a: u32, b: u32) {
        self.0 = TAG_SWITCH | pos_bits(a) | (b & 7) << 7;
    }

    /// C: `FUN_overlay_d_93__021d83c4`: switch record with bit 10 set and both values 0.
    pub fn set_switch_flagged(&mut self) {
        self.0 = TAG_SWITCH | 1 << 10;
    }

    /// C: `FUN_overlay_d_93__021d83e0`: a switch record with bit 10 set.
    pub fn is_switch_flagged(&self) -> bool {
        self.tag() == TAG_SWITCH && (self.0 >> 10) & 1 == 1
    }

    /// C: `FUN_overlay_d_93__021d83fc`: tag 6 record with a 3-bit value at bits 4-6; replaces
    /// the whole word.
    pub fn set_tag6(&mut self, v: u32) {
        self.0 = 6 | pos_bits(v);
    }

    /// C: `FUN_overlay_d_93__021d8414`: sets the tag to 4, keeping the other bits.
    pub fn set_tag4(&mut self) {
        self.0 = (self.0 & !TAG_MASK) | 4;
    }

    /// C: unlabeled routine at `0x021D8424`: sets the tag to 5, keeping the other bits.
    pub fn set_tag5(&mut self) {
        self.0 = (self.0 & !TAG_MASK) | 5;
    }

    /// C: `FUN_overlay_d_93__021d8444`: sets the tag to 7, keeping the other bits.
    pub fn set_tag7(&mut self) {
        self.0 = (self.0 & !TAG_MASK) | 7;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{Fnv, Rng};

    #[test]
    fn layouts_hand_checked() {
        let mut a = ActionRecord(0xFFFF_FFFF);
        a.set_fight(0x1234, 5);
        assert_eq!(a.0, 1 | 5 << 4 | 0x1234 << 7);
        assert_eq!((a.tag(), a.fight_waza(), a.fight_flag()), (1, 0x1234, 0));
        a.set_fight_flag();
        assert_eq!(a.fight_flag(), 1);
        a.set_fight_pos(6);
        assert_eq!(a.0 >> 4 & 7, 5);
        a.set_fight_pos(2);
        assert_eq!(a.0 >> 4 & 7, 2);
        a.set_item(0xFFFF, 7, 0x1FF);
        assert_eq!(a.0, 2 | 7 << 4 | 0xFFFF << 7 | 0xFF << 23);
        assert_eq!(a.fight_waza(), 0);
        a.set_switch_flagged();
        assert_eq!(a.0, 0x403);
        assert!(a.is_switch_flagged());
        a.set_switch(1, 2);
        assert_eq!(a.0, 3 | 1 << 4 | 2 << 7);
        assert!(!a.is_switch_flagged());
        a.set_tag6(9);
        assert_eq!(a.0, 6 | 1 << 4);
        a.set_none();
        assert_eq!(a.0, 0);
    }

    /// Every record routine against the original (`tools/rom-diff/gen.py action`).
    #[test]
    fn action_matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut r = Rng::new(0xAC71);
        for _ in 0..6000 {
            let mut w = r.next();
            if r.next().is_multiple_of(2) {
                w = w & !0xF | (r.next() % 10);
            }
            let (a, b, c) = (r.next(), r.next(), r.next());
            let mut rec = ActionRecord(w);
            h.w(rec.tag());
            h.w(rec.fight_flag());
            h.w(rec.fight_waza());
            h.w(rec.is_switch_flagged() as u32);
            rec.set_fight_pos(if a % 3 == 0 { 6 } else { b });
            h.w(rec.0);
            rec.set_fight_flag();
            h.w(rec.0);
            let mut ops = [ActionRecord(w); 9];
            ops[0].set_fight(a, b);
            ops[1].set_item(a, b, c);
            ops[2].set_switch(a, b);
            ops[3].set_switch_flagged();
            ops[4].set_tag6(a);
            ops[5].set_tag4();
            ops[6].set_tag5();
            ops[7].set_none();
            ops[8].set_tag7();
            for o in ops {
                h.w(o.0);
            }
        }
        assert_eq!(h.hex(), "7503e2fad672f957");
    }
}
