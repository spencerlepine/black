//! WebAssembly exports for the battle-calc playground in `web/`.
//!
//! Plain `extern "C"` functions over integers, so the page loads the module with
//! `WebAssembly.instantiateStreaming` and needs no bindings generator. Each export calls one
//! ported function; nothing here combines them into a damage pipeline the game doesn't have yet.
//! Out-of-range type IDs are clamped to "no type" so bad input can't trap.

use std::cell::RefCell;

use black_battle::calc::{self, BattleCalc, MathRandContext32};
use black_battle::poke_type::{make_pair, types};

/// Gen 5 battle RNG constants.
const LCG_MUL: u64 = 0x5D58_8B65_6C07_8965;
const LCG_ADD: u64 = 0x0026_9EC3;

thread_local! {
    static CALC: RefCell<BattleCalc> = RefCell::new(BattleCalc::new());
}

fn ty(v: u32) -> u8 {
    if v > types::NONE as u32 {
        types::NONE
    } else {
        v as u8
    }
}

/// Stage -6..=+6 to table index 0..=12.
fn stage_index(stage: i32) -> u32 {
    (stage.clamp(-6, 6) + 6) as u32
}

/// Seeds the battle RNG (`x = hi << 32 | lo`).
#[no_mangle]
pub extern "C" fn seed_rng(hi: u32, lo: u32) {
    let ctx = MathRandContext32 {
        x: (hi as u64) << 32 | lo as u64,
        mul: LCG_MUL,
        add: LCG_ADD,
    };
    CALC.with(|c| c.borrow_mut().set_rng(&ctx));
}

/// Effectiveness code (0 immune .. 5 quadruple) of `attack` against types `t1`/`t2`.
#[no_mangle]
pub extern "C" fn type_effectiveness(attack: u32, t1: u32, t2: u32) -> u32 {
    calc::type_effectiveness_vs_pair(ty(attack), make_pair(ty(t1), ty(t2))) as u32
}

/// `stat` after a stat stage of -6..=+6.
#[no_mangle]
pub extern "C" fn apply_stat_stage(stat: u32, stage: i32) -> u32 {
    calc::apply_stat_stage(stat, stage_index(stage)) as u32
}

/// `accuracy` after an accuracy stage of -6..=+6 (capped at 100).
#[no_mangle]
pub extern "C" fn apply_accuracy_stage(accuracy: u32, stage: i32) -> u32 {
    calc::apply_accuracy_stage(accuracy, stage_index(stage)) as u32
}

/// `CalcBaseDamage`.
#[no_mangle]
pub extern "C" fn base_damage(power: u32, attack: u32, level: u32, defense: u32) -> u32 {
    calc::calc_base_damage(power, attack, level, defense)
}

/// `ApplyTypeEffectiveness`.
#[no_mangle]
pub extern "C" fn apply_type_effectiveness(damage: u32, eff: u32) -> u32 {
    calc::apply_type_effectiveness(damage, eff)
}

/// 4.12 fixed-point weather power modifier (0x1000 = 1.0).
#[no_mangle]
pub extern "C" fn weather_power_modifier(weather: u32, move_type: u32) -> u32 {
    calc::weather_power_modifier(weather, move_type)
}

/// One critical-hit roll at `stage` (0-4). Returns 1 on a crit.
#[no_mangle]
pub extern "C" fn roll_critical(stage: u32) -> u32 {
    CALC.with(|c| c.borrow_mut().roll_critical(stage.min(4)) as u32)
}

/// One 2-5 hit move roll. Returns the hit count.
#[no_mangle]
pub extern "C" fn roll_hit_count() -> u32 {
    CALC.with(|c| c.borrow_mut().roll_hit_count(5))
}

/// One `percent`% roll (accuracy check style). Returns 1 on success.
#[no_mangle]
pub extern "C" fn roll_percent(percent: u32) -> u32 {
    CALC.with(|c| c.borrow_mut().roll_percent(percent) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exports_wrap_the_port() {
        assert_eq!(type_effectiveness(12, 10, 2), 5);
        assert_eq!(type_effectiveness(99, 10, 2), 3);
        assert_eq!(apply_stat_stage(200, 2), 400);
        assert_eq!(apply_stat_stage(200, -9), 50);
        assert_eq!(apply_accuracy_stage(70, -6), 23);
        assert_eq!(base_damage(80, 120, 50, 100), 44);
        assert_eq!(apply_type_effectiveness(44, 4), 88);
        assert_eq!(weather_power_modifier(1, 9), 0x1800);
        seed_rng(0, 0);
        let hits = roll_hit_count();
        assert!((2..=5).contains(&hits));
        assert!(roll_critical(99) <= 1);
        assert!(roll_percent(100) == 1);
    }
}
