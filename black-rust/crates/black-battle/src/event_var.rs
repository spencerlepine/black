//! Battle event-variable stack (asm/battle_calc.s 0x021D74F8-0x021D7840, overlay 93).
//!
//! Event handlers (abilities, items, moves) communicate through labelled `i32` values. A caller
//! pushes a frame, sets the values it wants handlers to see or modify, runs the handlers, reads
//! the results back, and pops the frame. The original is one global at `0x021F3ECC`:
//!
//! | offset | field                  |
//! |--------|------------------------|
//! | +0x000 | `u32 sp`               |
//! | +0x004 | `u16 label[0x60]`      |
//! | +0x0C4 | `s32 value[0x60]`      |
//! | +0x244 | `s32 max[0x60]`        |
//! | +0x3C4 | `s32 min[0x60]`        |
//! | +0x544 | `u8 kind[0x60]`        |
//!
//! Label 0 marks an empty slot; label 1 is a frame separator written by [`EventVarStack::push`].
//! A frame's values live in the slots from `sp` up to the next empty slot.

/// Number of slots.
pub const EVENT_VAR_SLOTS: usize = 0x60;
/// Label reserved for frame separators.
pub const LABEL_SEPARATOR: u16 = 1;

/// Value kinds (`kind[]` bytes). Names describe how [`EventVarStack::rewrite_value`] and
/// [`EventVarStack::mul_value`] treat them.
pub mod kind {
    /// Can be rewritten any number of times.
    pub const REWRITABLE: u8 = 0;
    /// Can be rewritten once; becomes [`REWRITTEN`].
    pub const REWRITE_ONCE: u8 = 1;
    /// A [`REWRITE_ONCE`] value that has been rewritten.
    pub const REWRITTEN: u8 = 2;
    /// Only changed by multiplication, clamped to `[min, max]`.
    pub const MULTIPLIER: u8 = 3;
    /// Cannot be rewritten or multiplied.
    pub const CONSTANT: u8 = 4;
}

/// `push` found no room for a separator. The original calls `FUN_02034714` (undecompiled, an
/// assertion/diagnostic routine) and returns; the scan has already advanced `sp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StackOverflow;

/// C: the event-variable global at `0x021F3ECC`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventVarStack {
    pub sp: u32,
    pub label: [u16; EVENT_VAR_SLOTS],
    pub value: [i32; EVENT_VAR_SLOTS],
    pub max: [i32; EVENT_VAR_SLOTS],
    pub min: [i32; EVENT_VAR_SLOTS],
    pub kind: [u8; EVENT_VAR_SLOTS],
}

impl Default for EventVarStack {
    fn default() -> Self {
        Self::new()
    }
}

impl EventVarStack {
    /// Size of the original structure in bytes.
    pub const SIZE: usize = 0x5A4;

    pub fn new() -> Self {
        EventVarStack {
            sp: 0,
            label: [0; EVENT_VAR_SLOTS],
            value: [0; EVENT_VAR_SLOTS],
            max: [0; EVENT_VAR_SLOTS],
            min: [0; EVENT_VAR_SLOTS],
            kind: [0; EVENT_VAR_SLOTS],
        }
    }

    /// Little-endian image of the original structure.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(&self.sp.to_le_bytes());
        self.label
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
        self.value
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
        self.max
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
        self.min
            .iter()
            .for_each(|v| out.extend_from_slice(&v.to_le_bytes()));
        out.extend_from_slice(&self.kind);
        debug_assert_eq!(out.len(), Self::SIZE);
        out
    }

    /// C: `FUN_overlay_d_93__021d74f8`: skips past the current frame's values and writes a
    /// separator.
    pub fn push(&mut self) -> Result<(), StackOverflow> {
        let mut i = self.sp;
        while i < EVENT_VAR_SLOTS as u32 && self.label[i as usize] != 0 {
            self.sp += 1;
            i = self.sp;
        }
        if i >= EVENT_VAR_SLOTS as u32 - 1 {
            return Err(StackOverflow);
        }
        self.sp += 1;
        self.label[i as usize] = LABEL_SEPARATOR;
        Ok(())
    }

    /// C: `FUN_overlay_d_93__021d7540`: clears the current frame and its separator, then moves
    /// `sp` to just after the previous separator (or to 0).
    pub fn pop(&mut self) {
        if self.sp == 0 {
            return;
        }
        let mut i = self.sp as u16;
        while (i as usize) < EVENT_VAR_SLOTS && self.label[i as usize] != 0 {
            self.label[i as usize] = 0;
            i = i.wrapping_add(1);
        }
        self.sp = self.sp.wrapping_sub(1);
        self.label[self.sp as usize] = 0;
        while self.sp != 0 {
            let j = self.sp - 1;
            self.sp = j;
            if self.label[j as usize] == LABEL_SEPARATOR {
                self.sp = j + 1;
                return;
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d77c4`: slot holding `label` in the current frame, else the
    /// first empty slot. Returns 0 when the scan runs off the end.
    pub fn slot_for_set(&self, label: u16) -> usize {
        let mut i = self.sp as usize;
        while i < EVENT_VAR_SLOTS {
            let l = self.label[i];
            if l == 0 || l == label {
                break;
            }
            i += 1;
        }
        if i >= EVENT_VAR_SLOTS {
            0
        } else {
            i
        }
    }

    /// C: `FUN_overlay_d_93__021d77e8`: slot holding `label` in the current frame, or -1.
    pub fn find(&self, label: u16) -> i32 {
        let mut i = self.sp as usize;
        while i < EVENT_VAR_SLOTS && self.label[i] != 0 {
            if self.label[i] == label {
                return i as i32;
            }
            i += 1;
        }
        -1
    }

    fn set(&mut self, label: u16, value: i32, min: i32, max: i32, kind: u8) {
        let i = self.slot_for_set(label);
        self.label[i] = label;
        self.value[i] = value;
        self.min[i] = min;
        self.max[i] = max;
        self.kind[i] = kind;
    }

    /// C: `FUN_overlay_d_93__021d75a8`: sets a [`kind::REWRITABLE`] value.
    pub fn set_value(&mut self, label: u16, value: i32) {
        self.set(label, value, 0, 0, kind::REWRITABLE);
    }

    /// C: `FUN_overlay_d_93__021d75ec`: sets a [`kind::CONSTANT`] value.
    pub fn set_const_value(&mut self, label: u16, value: i32) {
        self.set(label, value, 0, 0, kind::CONSTANT);
    }

    /// C: `FUN_overlay_d_93__021d7630`: sets a [`kind::REWRITE_ONCE`] value.
    pub fn set_rewrite_once_value(&mut self, label: u16, value: i32) {
        self.set(label, value, 0, 0, kind::REWRITE_ONCE);
    }

    /// C: `FUN_overlay_d_93__021d7674`: sets a [`kind::MULTIPLIER`] value (usually 4.12 fixed
    /// point) with clamp bounds. `min == max == 0` disables clamping.
    pub fn set_mul_value(&mut self, label: u16, value: i32, min: i32, max: i32) {
        self.set(label, value, min, max, kind::MULTIPLIER);
    }

    /// C: `FUN_overlay_d_93__021d76bc`: rewrites a [`kind::REWRITABLE`] or
    /// [`kind::REWRITE_ONCE`] value. Returns whether it was written.
    pub fn rewrite_value(&mut self, label: u16, value: i32) -> bool {
        let i = self.find(label);
        if i < 0 {
            return false;
        }
        let i = i as usize;
        if self.kind[i] > kind::REWRITE_ONCE {
            return false;
        }
        self.label[i] = label;
        self.value[i] = value;
        if self.kind[i] == kind::REWRITE_ONCE {
            self.kind[i] = kind::REWRITTEN;
        }
        true
    }

    /// C: `FUN_overlay_d_93__021d7704`: multiplies a [`kind::MULTIPLIER`] value by the 4.12
    /// fixed-point `mul` (rounding half up), then clamps.
    pub fn mul_value(&mut self, label: u16, mul: i32) {
        let i = self.find(label);
        if i < 0 {
            return;
        }
        let i = i as usize;
        if self.kind[i] != kind::MULTIPLIER {
            return;
        }
        let product = (self.value[i] as i64).wrapping_mul(mul as i64);
        let v = (product.wrapping_add(0x800) >> 12) as i32;
        self.value[i] = self.clamp(i, v);
    }

    /// C: `FUN_overlay_d_93__021d780c`: clamps `value` to slot `i`'s bounds unless both are 0.
    pub fn clamp(&self, i: usize, value: i32) -> i32 {
        let max = self.max[i];
        let min = self.min[i];
        let mut v = value;
        if max != 0 || min != 0 {
            if min > v {
                v = min;
            }
            if max < v {
                v = max;
            }
        }
        v
    }

    /// C: `FUN_overlay_d_93__021d775c`: value of `label` in the current frame, or 0.
    pub fn get_value(&self, label: u16) -> i32 {
        let mut i = self.sp as usize;
        while i < EVENT_VAR_SLOTS {
            let l = self.label[i];
            if l == label {
                return self.value[i];
            }
            if l == 0 {
                break;
            }
            i += 1;
        }
        0
    }

    /// C: unnamed Thumb routine at `0x021D778C` (stored as data in battle_calc.s and not
    /// referenced in overlay 93): like [`Self::get_value`] but reports whether `label` exists.
    pub fn get_value_if_exists(&self, label: u16) -> Option<i32> {
        let mut i = self.sp as usize;
        while i < EVENT_VAR_SLOTS {
            let l = self.label[i];
            if l == label {
                return Some(self.value[i]);
            }
            if l == 0 {
                break;
            }
            i += 1;
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::{Fnv, Rng};

    #[test]
    fn frames_nest_and_unwind() {
        let mut s = EventVarStack::new();
        s.push().unwrap();
        assert_eq!((s.sp, s.label[0]), (1, LABEL_SEPARATOR));
        s.set_value(10, 100);
        s.set_mul_value(11, 0x1000, 1, 200);
        s.push().unwrap();
        // Outer values are skipped; the new separator lands after them.
        assert_eq!((s.sp, s.label[3]), (4, LABEL_SEPARATOR));
        assert_eq!(s.get_value(10), 0);
        s.set_value(10, 7);
        assert_eq!(s.get_value(10), 7);
        s.pop();
        assert_eq!(s.sp, 1);
        assert_eq!(s.get_value(10), 100);
        assert_eq!(&s.label[..5], &[1, 10, 11, 0, 0]);
        s.pop();
        assert_eq!(s.sp, 0);
        assert!(s.label.iter().all(|&l| l == 0));
    }

    #[test]
    fn rewrite_rules() {
        let mut s = EventVarStack::new();
        s.push().unwrap();
        s.set_value(2, 1);
        s.set_rewrite_once_value(3, 1);
        s.set_const_value(4, 1);
        s.set_mul_value(5, 1, 0, 0);
        assert!(s.rewrite_value(2, 9) && s.rewrite_value(2, 10));
        assert!(s.rewrite_value(3, 9));
        assert!(!s.rewrite_value(3, 10));
        assert!(!s.rewrite_value(4, 9));
        assert!(!s.rewrite_value(5, 9));
        assert!(!s.rewrite_value(6, 9));
        assert_eq!(
            [2, 3, 4, 5].map(|l| s.get_value(l)),
            [10, 9, 1, 1],
            "values after rewrites"
        );
        assert_eq!(s.kind[s.find(3) as usize], kind::REWRITTEN);
    }

    #[test]
    fn mul_rounds_and_clamps() {
        let mut s = EventVarStack::new();
        s.push().unwrap();
        // 100 * 1.5 = 150; 150 * 1.5 = 225 -> clamped to 200.
        s.set_mul_value(20, 100, 1, 200);
        s.mul_value(20, 0x1800);
        assert_eq!(s.get_value(20), 150);
        s.mul_value(20, 0x1800);
        assert_eq!(s.get_value(20), 200);
        // Rounds half up: 1 * 0x800 = 0x800 -> (0x800 + 0x800) >> 12 = 1.
        s.set_mul_value(21, 1, 0, 0);
        s.mul_value(21, 0x800);
        assert_eq!(s.get_value(21), 1);
        s.mul_value(21, 0x7FF);
        assert_eq!(s.get_value(21), 0);
        // Not a multiplier: untouched.
        s.set_value(22, 10);
        s.mul_value(22, 0x2000);
        assert_eq!(s.get_value(22), 10);
    }

    #[test]
    fn overflow_reports_and_advances() {
        let mut s = EventVarStack::new();
        s.label[..0x5F].fill(9);
        assert_eq!(s.push(), Err(StackOverflow));
        assert_eq!(s.sp, 0x5F);
        assert_eq!(s.label[0x5F], 0);
    }

    #[test]
    fn get_value_if_exists() {
        let mut s = EventVarStack::new();
        s.push().unwrap();
        s.set_value(30, -4);
        assert_eq!(s.get_value_if_exists(30), Some(-4));
        assert_eq!(s.get_value_if_exists(31), None);
    }

    /// Labels drawn from a small set so operations collide; hashed against the original code
    /// run under an emulator (see `PORTING-STATUS.md`, "Verification").
    #[test]
    fn matches_rom_fuzz() {
        let mut h = Fnv::new();
        for seed in 0..300u32 {
            let mut rng = Rng::new(seed);
            let mut s = EventVarStack::new();
            for _ in 0..60 {
                let op = rng.next() % 12;
                let label = (2 + rng.next() % 6) as u16;
                let v = rng.next() as i32 >> (rng.next() % 32);
                match op {
                    0 | 1 => h.w(s.push().is_ok() as u32),
                    2 => s.pop(),
                    3 => s.set_value(label, v),
                    4 => s.set_const_value(label, v),
                    5 => s.set_rewrite_once_value(label, v),
                    6 => {
                        let a = (rng.next() % 300) as i32;
                        let b = (rng.next() % 300) as i32;
                        s.set_mul_value(label, v, a, b)
                    }
                    7 => h.w(s.rewrite_value(label, v) as u32),
                    8 => s.mul_value(label, (rng.next() % 0x4000) as i32),
                    9 => h.w(s.get_value(label) as u32),
                    10 => h.w(s.find(label) as u32),
                    _ => {
                        let r = s.get_value_if_exists(label);
                        h.w(r.is_some() as u32);
                        h.w(r.unwrap_or(0) as u32);
                    }
                }
            }
            s.to_bytes().iter().for_each(|&b| h.b(b));
        }
        assert_eq!(h.hex(), "3345d98e60c66013");
    }
}
