//! The head of `event_battle.c` (overlay 10, `0x0216E62C..0x0216E954`): the battle-call GMEVENT proc and its
//! constructors, plus a callback runner and the Burmy form update. F1 counted these 7 functions in its fldmmdl.c
//! row; they are battle-event glue (F5b owns `event_battle.c` from `0x0216E95C`).
//!
//! F4a-spec specced all 7 (`tools/rom-diff/specs/black-field/event_battle_head.spec`, whose header documents the
//! battle-call work); this module holds the smoke port.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0216_E680];

/// Unported callees of this area.
pub trait EventBattleHooks {
    /// Calls the callback at `f` (a Thumb address) with `f` in r0.
    fn call_callback(&mut self, f: u32);
}

/// C: `FUN_0216E680` (unlabeled routine at `0x0216E680`): runs the callback `f` when it is not null, then
/// returns `!f` (the original's `mvns r0, r4`; the function is referenced from the word at `0x0216EDD8`).
pub fn run_callback(h: &mut impl EventBattleHooks, f: u32) -> u32 {
    if f != 0 {
        h.call_callback(f);
    }
    !f
}

#[cfg(test)]
mod tests;
