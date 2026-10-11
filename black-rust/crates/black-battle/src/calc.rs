//! Battle calculation helpers: asm/battle_calc.s 0x021D7840-0x021D80BC (overlay 93).
//!
//! RNG, damage formula, type effectiveness, stat/accuracy stages, critical hits, multi-hit
//! counts, weather, status durations, EXP yield and prize money.
//!
//! The module state is the global at `0x021F6360`: a `u16` at +0, the work buffer pointer at +4
//! and an SDK `MATHRandContext32` at +8. [`BattleCalc`] owns that state.
//!
//! Integer behaviour follows the machine code: `_u32_div_f` (`FUN_0209C2B0`) is unsigned and
//! returns the dividend when dividing by zero ([`u32_div`]); `_s32_div_f` (`FUN_0209C0A4`) is
//! signed; multiplications wrap.

use black_core::heap::{Heap, HeapId};

use crate::hooks::{bpp, Arm9Misc, BattleMon, BattleParty, PersonalData};
use crate::poke_type::{split_pair, types};
use crate::sick_cont::SickCont;

/// C: `battle_calc_rodata` (0x021F041C), up to the end of the type chart. Copied from
/// asm/battle_calc.s (it matches the built overlay byte for byte). Kept as one block so table
/// reads at any offset behave like the original's.
pub const BATTLE_CALC_RODATA: [u8; 0x181] = [
    0x10, 0x08, 0x04, 0x03, 0x02, 0x00, 0x00, 0x23, 0x46, 0x55, 0x64, 0x00, 0x01, 0x02, 0x04, 0x08,
    0x10, 0x02, 0x04, 0x06, 0x09, 0x0C, 0x10, 0x14, 0x19, 0x1E, 0x73, 0x00, 0x71, 0x08, 0xDB, 0x10,
    0x36, 0x18, 0x6E, 0x21, 0x7D, 0x29, 0xBF, 0x30, 0x86, 0x39, 0xBE, 0x41, 0x02, 0x08, 0x02, 0x07,
    0x02, 0x06, 0x02, 0x05, 0x02, 0x04, 0x02, 0x03, 0x02, 0x02, 0x03, 0x02, 0x04, 0x02, 0x05, 0x02,
    0x06, 0x02, 0x07, 0x02, 0x08, 0x02, 0x06, 0x12, 0x06, 0x10, 0x06, 0x0E, 0x06, 0x0C, 0x06, 0x0A,
    0x06, 0x08, 0x06, 0x06, 0x08, 0x06, 0x0A, 0x06, 0x0C, 0x06, 0x0E, 0x06, 0x10, 0x06, 0x12, 0x06,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x00, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04,
    0x04, 0x08, 0x04, 0x02, 0x02, 0x04, 0x08, 0x02, 0x00, 0x08, 0x04, 0x04, 0x04, 0x04, 0x02, 0x08,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x04, 0x04, 0x02, 0x08, 0x04, 0x02, 0x04, 0x04, 0x08, 0x02, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x02, 0x00, 0x04, 0x04, 0x08, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x08, 0x04, 0x08, 0x02, 0x04, 0x08, 0x08, 0x04, 0x02,
    0x08, 0x04, 0x04, 0x04, 0x04, 0x04, 0x02, 0x08, 0x04, 0x02, 0x04, 0x08, 0x04, 0x02, 0x08, 0x04,
    0x04, 0x04, 0x04, 0x08, 0x04, 0x04, 0x04, 0x02, 0x02, 0x02, 0x04, 0x04, 0x04, 0x02, 0x02, 0x02,
    0x04, 0x08, 0x04, 0x08, 0x04, 0x04, 0x08, 0x00, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x02,
    0x04, 0x04, 0x04, 0x04, 0x08, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x04, 0x04,
    0x02, 0x02, 0x02, 0x04, 0x02, 0x04, 0x08, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x02, 0x08,
    0x04, 0x08, 0x02, 0x02, 0x08, 0x04, 0x04, 0x08, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x08,
    0x04, 0x04, 0x04, 0x08, 0x02, 0x02, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x02, 0x02, 0x08,
    0x08, 0x02, 0x04, 0x02, 0x02, 0x08, 0x02, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x08, 0x04,
    0x00, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x02, 0x02, 0x04, 0x04, 0x02, 0x04, 0x04, 0x08, 0x04,
    0x08, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x00, 0x04, 0x04,
    0x08, 0x04, 0x08, 0x04, 0x04, 0x04, 0x02, 0x02, 0x02, 0x08, 0x04, 0x04, 0x02, 0x08, 0x04, 0x04,
    0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x04,
    0x04, 0x02, 0x04, 0x04, 0x04, 0x04, 0x04, 0x08, 0x02, 0x04, 0x04, 0x04, 0x04, 0x08, 0x04, 0x04,
    0x02,
];

/// 0x021F041C: critical-hit denominators by stage (16, 8, 4, 3, 2).
const OFS_CRIT_RATE: usize = 0x00;
/// 0x021F0421: cumulative percentages for 2-5 hit moves (0, 0, 35, 70, 85, 100).
const OFS_HIT_COUNT: usize = 0x05;
/// 0x021F0427: effectiveness code to multiplier in quarters (0, 1, 2, 4, 8, 16).
const OFS_EFFECTIVENESS_QUARTERS: usize = 0x0B;
/// 0x021F042D: prize-money multiplier by badge count (2, 4, 6, 9, 12, 16, 20, 25, 30).
const OFS_PRIZE_MONEY: usize = 0x11;
/// 0x021F0448: stat stage ratios `(num, den)` for stages -6..=+6 (2/8 .. 8/2).
const OFS_STAT_STAGE: usize = 0x2C;
/// 0x021F0462: accuracy stage ratios `(num, den)` for stages -6..=+6 (3/9 .. 9/3, stored x2).
const OFS_ACC_STAGE: usize = 0x46;
/// 0x021F047C: 17x17 type chart `[attack][defend]`: 0 immune, 2 half, 4 neutral, 8 double.
const OFS_TYPE_CHART: usize = 0x60;

/// Effectiveness codes returned by [`type_effectiveness`] and [`combine_effectiveness`].
pub mod effectiveness {
    pub const IMMUNE: u8 = 0;
    pub const QUARTER: u8 = 1;
    pub const HALF: u8 = 2;
    pub const NEUTRAL: u8 = 3;
    pub const DOUBLE: u8 = 4;
    pub const QUADRUPLE: u8 = 5;
}

/// Weather IDs, inferred from [`weather_power_modifier`] and [`weather_residual_damage`].
pub mod weather {
    pub const NONE: u32 = 0;
    pub const SUN: u32 = 1;
    pub const RAIN: u32 = 2;
    pub const HAIL: u32 = 3;
    pub const SAND: u32 = 4;
}

/// Major status / volatile condition IDs, inferred from [`BattleCalc::make_default_sick_cont`]
/// and [`BattleCalc::make_sick_cont`].
pub mod sick {
    pub const NONE: u32 = 0;
    pub const PARALYSIS: u32 = 1;
    pub const SLEEP: u32 = 2;
    pub const FREEZE: u32 = 3;
    pub const BURN: u32 = 4;
    pub const POISON: u32 = 5;
    pub const CONFUSION: u32 = 6;
    pub const ATTRACT: u32 = 7;
}

/// 4.12 fixed-point 1.0.
pub const FX_ONE: u32 = 0x1000;

const CALC_SRC_FILE: &str = "btl_calc.c";
const CALC_SRC_LINE: u32 = 0x50;
const CALC_WORK_SIZE: u32 = 0x460;

fn ro(ofs: usize) -> u8 {
    BATTLE_CALC_RODATA[ofs]
}

/// C: `_u32_div_f` (`FUN_0209C2B0`) quotient. Division by zero returns the dividend.
pub fn u32_div(n: u32, d: u32) -> u32 {
    u32_divmod(n, d).0
}

/// C: `_u32_div_f` returning `(quotient, remainder)` (r0, r1). Division by zero returns
/// `(n, 0)`.
pub fn u32_divmod(n: u32, d: u32) -> (u32, u32) {
    match (n.checked_div(d), n.checked_rem(d)) {
        (Some(q), Some(r)) => (q, r),
        _ => (n, 0),
    }
}

/// C: `_s32_div_f` (`FUN_0209C0A4`) quotient, truncating toward zero.
///
/// UNVERIFIED: division by zero (no caller in this module divides by a non-constant).
fn s32_div(n: i32, d: i32) -> i32 {
    n.wrapping_div(d)
}

// ---------------------------------------------------------------------------------------------
// RNG and module state
// ---------------------------------------------------------------------------------------------

/// SDK `MATHRandContext32`: 64-bit LCG `x = x * mul + add`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MathRandContext32 {
    pub x: u64,
    pub mul: u64,
    pub add: u64,
}

impl MathRandContext32 {
    pub const SIZE: usize = 24;

    pub fn from_bytes(b: &[u8; 24]) -> Self {
        let q = |i: usize| u64::from_le_bytes(b[i..i + 8].try_into().unwrap());
        MathRandContext32 {
            x: q(0),
            mul: q(8),
            add: q(16),
        }
    }

    pub fn to_bytes(&self) -> [u8; 24] {
        let mut out = [0u8; 24];
        out[0..8].copy_from_slice(&self.x.to_le_bytes());
        out[8..16].copy_from_slice(&self.mul.to_le_bytes());
        out[16..24].copy_from_slice(&self.add.to_le_bytes());
        out
    }

    /// Advances the state; returns the high word scaled to `[0, max)`, or the raw high word
    /// when `max == 0`.
    pub fn next(&mut self, max: u32) -> u32 {
        self.x = self.mul.wrapping_mul(self.x).wrapping_add(self.add);
        let hi = (self.x >> 32) as u32;
        if max == 0 {
            hi
        } else {
            ((hi as u64 * max as u64) >> 32) as u32
        }
    }
}

/// C: the battle_calc globals at `0x021F6360`.
#[derive(Debug, Default)]
pub struct BattleCalc {
    /// +0x00: `u16` set by [`Self::set_unk_00`], passed to `FUN_02020FE4` by
    /// `FUN_overlay_d_93__021d805c` (not ported).
    pub unk_00: u16,
    /// +0x04: 0x460-byte work buffer.
    pub work: Option<Box<[u8]>>,
    /// +0x08: battle RNG.
    pub rng: MathRandContext32,
}

impl BattleCalc {
    pub fn new() -> Self {
        Self::default()
    }

    /// C: `FUN_overlay_d_93__021d7840`: allocates the work buffer and copies the RNG context.
    ///
    /// Deviation: a previous buffer is dropped (the original overwrites the pointer).
    pub fn init<H: Heap>(&mut self, rng: &MathRandContext32, heap: &mut H, heap_id: HeapId) {
        self.work = heap.alloc_debug(heap_id, CALC_WORK_SIZE, true, CALC_SRC_FILE, CALC_SRC_LINE);
        self.rng = *rng;
    }

    /// C: `FUN_overlay_d_93__021d787c`: replaces the RNG context.
    pub fn set_rng(&mut self, rng: &MathRandContext32) {
        self.rng = *rng;
    }

    /// C: `FUN_overlay_d_93__021d7894`: frees the work buffer if allocated.
    pub fn free<H: Heap>(&mut self, heap: &mut H) {
        if let Some(b) = self.work.take() {
            heap.free(b);
        }
    }

    /// C: `FUN_overlay_d_93__021d804c`
    pub fn set_unk_00(&mut self, v: u16) {
        self.unk_00 = v;
    }

    /// C: `FUN_overlay_d_93__021d78ac`: random value in `[0, range)`, or the raw high word when
    /// `range == 0`.
    pub fn rand(&mut self, range: u32) -> u32 {
        self.rng.next(range)
    }

    /// C: `FUN_overlay_d_93__021d7930`: rolls a critical hit for `stage` (0-4).
    pub fn roll_critical(&mut self, stage: u32) -> bool {
        let denom = ro(OFS_CRIT_RATE + stage as usize) as u32;
        self.rand(denom) as u8 == 0
    }

    /// C: `FUN_overlay_d_93__021d7af0`: true with probability `percent`/100.
    pub fn roll_percent(&mut self, percent: u32) -> bool {
        self.rand(100) < percent
    }

    /// C: `FUN_overlay_d_93__021d7b8c`: uniform value in `[min(a, b), max(a, b)]`.
    pub fn rand_range(&mut self, a: u32, b: u32) -> u32 {
        let (lo, hi) = if a > b { (b, a) } else { (a, b) };
        lo.wrapping_add(self.rand(hi.wrapping_sub(lo).wrapping_add(1)))
    }

    /// C: `FUN_overlay_d_93__021d7ba4`: for `n == 5` (a 2-5 hit move) rolls 2 (35%), 3 (35%),
    /// 4 (15%) or 5 (15%) hits; other counts are returned unchanged without using the RNG.
    pub fn roll_hit_count(&mut self, n: u32) -> u32 {
        if n != 5 {
            return n;
        }
        let r = self.rand(100) as u8;
        for i in 0..6u8 {
            if r < ro(OFS_HIT_COUNT + i as usize) {
                return i as u32;
            }
        }
        n
    }

    /// C: `FUN_overlay_d_93__021d7d18`: duration for a major status: sleep lasts 2-4 turns
    /// (random), paralysis/freeze/burn/poison are permanent, anything else is none.
    pub fn make_default_sick_cont(&mut self, sick_id: u32) -> SickCont {
        match sick_id {
            sick::SLEEP => SickCont::turn(self.rand_range(2, 4)),
            sick::PARALYSIS | sick::FREEZE | sick::BURN | sick::POISON => SickCont::permanent(),
            _ => SickCont::NONE,
        }
    }

    /// C: `FUN_overlay_d_93__021d7dc0`: duration for any condition: major statuses as
    /// [`Self::make_default_sick_cont`], confusion 2-5 turns, attraction linked to `mon`,
    /// anything else permanent. `sick_id` is compared as signed, like the original.
    pub fn make_sick_cont<M: BattleMon>(&mut self, sick_id: u32, mon: &M) -> SickCont {
        if (sick_id as i32) < sick::CONFUSION as i32 {
            return self.make_default_sick_cont(sick_id);
        }
        match sick_id {
            sick::CONFUSION => SickCont::turn(self.rand_range(2, 5) as u8 as u32),
            sick::ATTRACT => SickCont::poke(mon.poke_id() as u32),
            _ => SickCont::permanent(),
        }
    }

    /// C: `FUN_overlay_d_93__021d7c70`: duration from a move's packed `u16` condition
    /// parameter (bits 0-3 kind, 4-9 minimum, 10-15 maximum). Kinds 0 and >4 leave `out`
    /// untouched.
    ///
    /// | kind | result                                                   |
    /// |------|----------------------------------------------------------|
    /// | 1    | [`SickCont::permanent_count`]`(max, min)`                |
    /// | 2    | [`SickCont::turn`]`(rand_range(min, max))`               |
    /// | 3    | [`SickCont::poke`]`(poke_id)`                            |
    /// | 4    | [`SickCont::poke_turn`]`(poke_id, rand_range(min, max))` |
    pub fn make_move_sick_cont<M: BattleMon>(&mut self, param: u16, mon: &M, out: &mut SickCont) {
        let kind = param & 0xF;
        let lo = ((param >> 4) & 0x3F) as u32;
        let hi = ((param >> 10) & 0x3F) as u32;
        match kind {
            1 => *out = SickCont::permanent_count(hi, lo),
            2 => *out = SickCont::turn(self.rand_range(lo, hi) as u8 as u32),
            3 => *out = SickCont::poke(mon.poke_id() as u32),
            4 => {
                let id = mon.poke_id() as u32;
                *out = SickCont::poke_turn(id, self.rand_range(lo, hi) as u8 as u32);
            }
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Pure calculations
// ---------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_93__021d78e4`: applies stat stage index `stage` (0-12 for -6..=+6) to
/// `stat`. The product is truncated to `u16` before dividing, like the original.
pub fn apply_stat_stage(stat: u32, stage: u32) -> u16 {
    let i = OFS_STAT_STAGE + 2 * stage as usize;
    let num = ro(i) as u32;
    let den = ro(i + 1) as u32;
    u32_div(stat.wrapping_mul(num) & 0xFFFF, den) as u16
}

/// C: `FUN_overlay_d_93__021d7908`: applies accuracy stage index `stage` (0-12) to `accuracy`,
/// capped at 100.
pub fn apply_accuracy_stage(accuracy: u32, stage: u32) -> u8 {
    let i = OFS_ACC_STAGE + 2 * stage as usize;
    let num = ro(i) as i32;
    let den = ro(i + 1) as i32;
    let v = s32_div(num.wrapping_mul(accuracy as i32), den) as u32;
    if v > 100 {
        100
    } else {
        v as u8
    }
}

/// C: `FUN_overlay_d_93__021d794c`: effectiveness code of `attack` against one `defend` type.
/// [`types::NONE`] on either side is neutral.
pub fn type_effectiveness(attack: u8, defend: u8) -> u8 {
    if attack == types::NONE || defend == types::NONE {
        return effectiveness::NEUTRAL;
    }
    let v = ro(OFS_TYPE_CHART + attack as usize * types::COUNT as usize + defend as usize);
    match v {
        0 => effectiveness::IMMUNE,
        2 => effectiveness::HALF,
        4 => effectiveness::NEUTRAL,
        8 => effectiveness::DOUBLE,
        _ => effectiveness::IMMUNE,
    }
}

/// C: `FUN_overlay_d_93__021d79e8`: combines two effectiveness codes.
pub fn combine_effectiveness(a: u8, b: u8) -> u8 {
    let p = ro(OFS_EFFECTIVENESS_QUARTERS + a as usize) as i32
        * ro(OFS_EFFECTIVENESS_QUARTERS + b as usize) as i32;
    match p / 4 {
        0 => effectiveness::IMMUNE,
        1 => effectiveness::QUARTER,
        2 => effectiveness::HALF,
        4 => effectiveness::NEUTRAL,
        8 => effectiveness::DOUBLE,
        16 => effectiveness::QUADRUPLE,
        _ => effectiveness::IMMUNE,
    }
}

/// C: `FUN_overlay_d_93__021d799c`: effectiveness of `attack` against a packed type pair.
pub fn type_effectiveness_vs_pair(attack: u8, pair: u16) -> u8 {
    let (t1, t2) = split_pair(pair);
    if t1 == t2 {
        return type_effectiveness(attack, t1);
    }
    combine_effectiveness(
        type_effectiveness(attack, t1),
        type_effectiveness(attack, t2),
    )
}

/// C: `CalcBaseDamage` (0x021D7A44):
/// `((2 * level / 5 + 2) * power * attack / defense) / 50 + 2`, unsigned and wrapping.
pub fn calc_base_damage(power: u32, attack: u32, level: u32, defense: u32) -> u32 {
    let level_factor = u32_div(level.wrapping_mul(2), 5).wrapping_add(2);
    let n = power.wrapping_mul(attack).wrapping_mul(level_factor);
    u32_div(u32_div(n, defense), 50).wrapping_add(2)
}

/// C: `ApplyTypeEffectiveness` (0x021D7A6C): scales `damage` by an effectiveness code; codes
/// above 5 leave it unchanged.
pub fn apply_type_effectiveness(damage: u32, eff: u32) -> u32 {
    match eff {
        0 => 0,
        1 => damage >> 2,
        2 => damage >> 1,
        4 => damage << 1,
        5 => damage << 2,
        _ => damage,
    }
}

/// C: `FUN_overlay_d_93__021d7a9c`: writes the types that resist or are immune to `attack` into
/// `out` (in type order) and returns how many.
pub fn resisting_types(attack: u8, out: &mut [u8; types::COUNT as usize]) -> u8 {
    let row = OFS_TYPE_CHART + attack as usize * types::COUNT as usize;
    let mut n = 0u8;
    for t in 0..types::COUNT {
        let v = ro(row + t as usize);
        if v == 0 || v == 2 {
            out[n as usize] = t;
            n += 1;
        }
    }
    n
}

/// C: `GetTypeEffectivenessClass` (0x021D7AD4): 0 for immune, 1 for neutral, 2 for super
/// effective, 3 for resisted (signed compare).
pub fn effectiveness_class(eff: i32) -> u8 {
    if eff > 3 {
        2
    } else if eff == 3 {
        1
    } else if eff != 0 {
        3
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021d7b10`: 4.12 fixed-point multiply, rounding fractions above one
/// half up.
pub fn fx_mul(value: u32, fx: u32) -> u32 {
    let p = fx.wrapping_mul(value);
    let q = p >> 12;
    if p & 0xFFF > 0x800 {
        q.wrapping_add(1)
    } else {
        q
    }
}

/// C: `FUN_overlay_d_93__021d7b2c`: [`fx_mul`], but never 0.
pub fn fx_mul_min1(value: u32, fx: u32) -> u32 {
    match fx_mul(value, fx) {
        0 => 1,
        v => v,
    }
}

/// C: `FUN_overlay_d_93__021d7b3c`: `value * percent / 100`, rounding half up.
pub fn percent_mul(value: u32, percent: u32) -> u32 {
    let (q, r) = u32_divmod(value.wrapping_mul(percent), 100);
    if r >= 50 {
        q.wrapping_add(1)
    } else {
        q
    }
}

/// C: `FUN_overlay_d_93__021d7b60`: max HP / `div`.
pub fn max_hp_div<M: BattleMon>(mon: &M, div: u32) -> u32 {
    u32_div(mon.get(bpp::MAX_HP), div)
}

/// C: `FUN_overlay_d_93__021d7b74`: max HP / `div`, but never 0.
pub fn max_hp_div_min1<M: BattleMon>(mon: &M, div: u32) -> u32 {
    match max_hp_div(mon, div) {
        0 => 1,
        v => v,
    }
}

/// C: `FUN_overlay_d_93__021d7bd4`: end-of-turn weather damage: 1/16 of max HP (at least 1)
/// in hail unless Ice type, or in sandstorm unless Rock, Steel or Ground type.
pub fn weather_residual_damage<M: BattleMon>(mon: &M, weather: u32) -> u32 {
    match weather {
        weather::HAIL => {
            if mon.has_type(types::ICE as u32) {
                return 0;
            }
        }
        weather::SAND => {
            if mon.has_type(types::ROCK as u32)
                || mon.has_type(types::STEEL as u32)
                || mon.has_type(types::GROUND as u32)
            {
                return 0;
            }
        }
        _ => return 0,
    }
    let d = (mon.get(bpp::MAX_HP) as i32 / 16) as u32 & 0xFFFF;
    if d == 0 {
        1
    } else {
        d
    }
}

/// C: `FUN_overlay_d_93__021d7c38`: 4.12 power modifier for Fire/Water moves in sun or rain.
pub fn weather_power_modifier(weather: u32, move_type: u32) -> u32 {
    const FIRE: u32 = types::FIRE as u32;
    const WATER: u32 = types::WATER as u32;
    match (weather, move_type) {
        (weather::SUN, FIRE) | (weather::RAIN, WATER) => 0x1800,
        (weather::SUN, WATER) | (weather::RAIN, FIRE) => 0x800,
        _ => FX_ONE,
    }
}

/// C: `FUN_overlay_d_93__021d7eb4`: `personal(species, form, 9) * level / 5`. Param 9 is
/// presumably the base EXP yield (UNVERIFIED: `FUN_02019C90` is not ported).
pub fn exp_yield<M: BattleMon, P: PersonalData>(mon: &M, personal: &P) -> u32 {
    let species = mon.species();
    let form = mon.get(bpp::FORM) as u16;
    let level = mon.get(bpp::LEVEL) as u16;
    let base = personal.param(species, form, 9);
    u32_div(base.wrapping_mul(level as u32), 5)
}

/// C: `FUN_overlay_d_93__021d7ef0`: `v` is neither 0 nor 3.
pub fn is_not_0_or_3(v: u32) -> bool {
    v != 0 && v != 3
}

/// C: `FUN_overlay_d_93__021d7f00`: maps 0 -> 1, 1 -> 2, 2 -> 3, anything else -> 1.
pub fn fun_021d7f00(v: u32) -> u32 {
    match v {
        1 => 2,
        2 => 3,
        _ => 1,
    }
}

/// C: `FUN_overlay_d_93__021d7f2c`: 3 if `a == 3`, else `b`.
pub fn fun_021d7f2c(a: u32, b: u32) -> u32 {
    if a == 3 {
        3
    } else {
        b
    }
}

/// C: `FUN_overlay_d_93__021d7f38`: `FUN_0202A3FC(v) == 2`.
pub fn fun_021d7f38<A: Arm9Misc>(arm9: &A, v: u32) -> bool {
    arm9.fun_0202a3fc(v) == 2
}

/// C: `FUN_overlay_d_93__021d7f4c`: `FUN_0202A3FC(v) == 3`.
pub fn fun_021d7f4c<A: Arm9Misc>(arm9: &A, v: u32) -> bool {
    arm9.fun_0202a3fc(v) == 3
}

/// C: `FUN_overlay_d_93__021d7f60`: either of the above, or `v == 0x59`.
pub fn fun_021d7f60<A: Arm9Misc>(arm9: &A, v: u32) -> bool {
    fun_021d7f38(arm9, v) || fun_021d7f4c(arm9, v) || v == 0x59
}

/// C: `FUN_overlay_d_93__021d7ff8`: prize money for losing: highest party level (at least 1)
/// x 4 x a multiplier for `badges` (clamped to 8).
pub fn prize_money<P: BattleParty>(badges: u32, party: &P) -> u32 {
    let count = party.count();
    let mut max_level = 1u32;
    let mut i = 0u32;
    while i < count {
        let level = party.member(i as u8).get(bpp::LEVEL);
        if level > max_level {
            max_level = level;
        }
        i += 1;
    }
    let badges = if badges >= 9 { 8 } else { badges };
    (max_level << 2).wrapping_mul(ro(OFS_PRIZE_MONEY + badges as usize) as u32)
}

/// C: `FUN_overlay_d_93__021d806c`: packs six 5-bit values (bits `5*i`).
pub fn pack_5bit_x6(v: &[u8; 6]) -> u32 {
    v.iter()
        .enumerate()
        .fold(0, |acc, (i, &x)| acc | ((x as u32 & 0x1F) << (5 * i)))
}

/// C: `FUN_overlay_d_93__021d8090`: inverse of [`pack_5bit_x6`].
pub fn unpack_5bit_x6(packed: u32, out: &mut [u8; 6]) {
    let mut p = packed;
    for o in out.iter_mut() {
        *o = (p & 0x1F) as u8;
        p >>= 5;
    }
}

/// C: `FUN_overlay_d_93__021d80a8`: 0 if `mon` is Ghost type, else 7.
pub fn fun_021d80a8<M: BattleMon>(mon: &M) -> u32 {
    if mon.has_type(types::GHOST as u32) {
        0
    } else {
        7
    }
}

#[cfg(test)]
mod tests;
