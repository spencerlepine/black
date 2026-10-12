//! Text speed (F9c). Main ARM9 Thumb code: `FUN_02013080` (speed table), `FUN_02013020` /
//! `FUN_02013050` (option setting -> table index), `FUN_0201CDF4` (speed value -> printer pace).
//! Verified per function with `tools/check-fn.sh black-text speed`
//! (spec `tools/rom-diff/specs/black-text/speed.spec`).
//!
//! The printer (`sub_0201CEE0`, render side, UNVERIFIED beyond what is below) keeps two bytes: a
//! delay `d` (`+0x1a`) and characters per tick `k` (`+0x1b`). Each tick it first counts down a wait
//! byte (`+0x1e`); at zero it prints `k` characters and reloads the wait with `d` after each one.
//! A *speed value* `v` becomes a pace in `FUN_0201CDF4`: `v >= 0` is "1 char every `v + 1` ticks",
//! `v < 0` is "`-v` chars per tick". The table at `0x0209DFB0` (5 signed bytes: 3, 1, -2, -4, 6)
//! gives the value per index. Which setting maps to which index (below) is verified; that the game
//! calls the first variant the "text speed" option (slow/mid/fast = index 1, 2, 3) is a reading of
//! the code and UNVERIFIED.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0201_3020, 0x0201_3050, 0x0201_3080, 0x0201_CDF4];

/// The speed values at `0x0209DFB0`.
pub const SPEED_TABLE: [i8; 5] = [3, 1, -2, -4, 6];

/// Frames between characters and characters per tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pace {
    /// Wait ticks after a character (`+0x1a`).
    pub delay: u8,
    /// Characters printed per tick (`+0x1b`).
    pub per_tick: u8,
}

/// C: `FUN_02013080(i)`: the table value for index `i` (clamped to 4), sign-extended.
pub fn speed_value(i: u32) -> i32 {
    SPEED_TABLE[i.min(4) as usize] as i32
}

/// C: `FUN_0201CDF4(v, out)`: `v >= 0` -> `{delay: v, per_tick: 1}`, else `{0, -v}` (low bytes).
pub fn pace(v: i32) -> Pace {
    if v >= 0 {
        Pace {
            delay: v as u8,
            per_tick: 1,
        }
    } else {
        Pace {
            delay: 0,
            per_tick: v.wrapping_neg() as u8,
        }
    }
}

/// C: `FUN_02013020`: the speed value for the option `opt` (the save option read by
/// `FUN_02008374`, 0..2): index `opt + 1`, options above 2 index the table as they are.
pub fn option_speed(opt: u32) -> i32 {
    speed_value(match opt {
        0 => 1,
        1 => 2,
        2 => 3,
        o => o,
    })
}

/// C: `FUN_02013050`: the second option-to-speed mapping: 0 -> index 4, 1 -> 0, 2 -> 1, else as is.
pub fn option_speed_alt(opt: u32) -> i32 {
    speed_value(match opt {
        0 => 4,
        1 => 0,
        2 => 1,
        o => o,
    })
}

/// A typewriter clock following the printer loop. UNVERIFIED model (see the module docs).
#[derive(Debug, Clone, Copy)]
pub struct Pacer {
    pace: Pace,
    wait: u8,
}

impl Pacer {
    pub fn new(pace: Pace) -> Self {
        Pacer { pace, wait: 0 }
    }

    /// One tick: the number of characters to reveal now.
    pub fn tick(&mut self) -> u8 {
        if self.wait > 0 {
            self.wait -= 1;
            return 0;
        }
        self.wait = self.pace.delay;
        self.pace.per_tick
    }
}

#[cfg(test)]
mod tests;
