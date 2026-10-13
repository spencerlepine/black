use super::*;
use crate::poke_type::{make_mono_pair, make_pair, types::*};
use crate::testutil::{Fnv, Rng};
use black_core::heap::HostHeap;

/// Gen 5 battle RNG constants.
const LCG_MUL: u64 = 0x5D58_8B65_6C07_8965;
const LCG_ADD: u64 = 0x0026_9EC3;

#[derive(Debug, Clone)]
struct Mon {
    max_hp: u32,
    level: u32,
    form: u32,
    species: u16,
    poke_id: u8,
    t1: u8,
    t2: u8,
}

impl Mon {
    fn random(r: &mut Rng) -> Self {
        Mon {
            max_hp: r.next() & 0xFFFF,
            level: 1 + r.next() % 100,
            form: r.next() % 4,
            species: (r.next() % 650) as u16,
            poke_id: (r.next() % 24) as u8,
            t1: (r.next() % 18) as u8,
            t2: (r.next() % 18) as u8,
        }
    }

    /// Full-width max HP, level, form and species (validation V-0005); mirrors `gen.py WideMon`.
    fn wide(r: &mut Rng) -> Self {
        let max_hp = r.next();
        let sel = r.next();
        Mon {
            max_hp,
            level: if !sel.is_multiple_of(2) {
                r.next()
            } else {
                1 + r.next() % 100
            },
            form: r.next(),
            species: (r.next() & 0xFFFF) as u16,
            poke_id: (r.next() % 24) as u8,
            t1: (r.next() % 18) as u8,
            t2: (r.next() % 18) as u8,
        }
    }

    fn typed(max_hp: u32, t1: u8, t2: u8) -> Self {
        Mon {
            max_hp,
            level: 50,
            form: 0,
            species: 1,
            poke_id: 3,
            t1,
            t2,
        }
    }
}

impl BattleMon for Mon {
    fn get(&self, id: u32) -> u32 {
        match id {
            bpp::MAX_HP => self.max_hp,
            bpp::LEVEL => self.level,
            bpp::FORM => self.form,
            _ => panic!("unexpected BattleMon_Get id {id:#x}"),
        }
    }

    fn has_type(&self, ty: u32) -> bool {
        ty != NONE as u32 && (self.t1 as u32 == ty || self.t2 as u32 == ty)
    }

    fn poke_id(&self) -> u8 {
        self.poke_id
    }

    fn species(&self) -> u16 {
        self.species
    }
}

struct Party(Vec<Mon>);

impl BattleParty for Party {
    type Mon = Mon;
    fn count(&self) -> u32 {
        self.0.len() as u32
    }
    fn member(&self, index: u8) -> &Mon {
        &self.0[index as usize]
    }
}

struct Personal;

impl PersonalData for Personal {
    fn param(&self, species: u16, form: u16, param: u32) -> u32 {
        (species as u32 * 31 + form as u32 * 7 + param) % 256
    }
}

struct Arm9;

impl Arm9Misc for Arm9 {
    fn fun_0202a3fc(&self, value: u32) -> u32 {
        value % 5
    }
}

fn calc_with_seed(x: u64) -> BattleCalc {
    let mut c = BattleCalc::new();
    c.rng = MathRandContext32 {
        x,
        mul: LCG_MUL,
        add: LCG_ADD,
    };
    c
}

// ---- hand-checked ---------------------------------------------------------------------------

#[test]
fn base_damage_examples() {
    // Level 50, 80 power, 120 Atk vs 100 Def: (22 * 80 * 120 / 100) / 50 + 2 = 44.
    assert_eq!(calc_base_damage(80, 120, 50, 100), 44);
    // Level 100, 120 power, 300 vs 200: (42 * 120 * 300 / 200) / 50 + 2 = 153.
    assert_eq!(calc_base_damage(120, 300, 100, 200), 153);
    // Division by zero returns the dividend (_u32_div_f): 22*10*10 = 2200 / 50 + 2.
    assert_eq!(calc_base_damage(10, 10, 50, 0), 46);
}

#[test]
fn type_chart_examples() {
    use effectiveness::*;
    assert_eq!(type_effectiveness(FIRE, GRASS), DOUBLE);
    assert_eq!(type_effectiveness(WATER, FIRE), DOUBLE);
    assert_eq!(type_effectiveness(NORMAL, GHOST), IMMUNE);
    assert_eq!(type_effectiveness(GROUND, FLYING), IMMUNE);
    assert_eq!(type_effectiveness(GHOST, STEEL), HALF);
    assert_eq!(type_effectiveness(DARK, STEEL), HALF);
    assert_eq!(type_effectiveness(DRAGON, DRAGON), DOUBLE);
    assert_eq!(type_effectiveness(NONE, DRAGON), NEUTRAL);
    assert_eq!(type_effectiveness(FIRE, NONE), NEUTRAL);
    // Gyarados (Water/Flying) vs Electric; Charizard (Fire/Flying) vs Rock.
    assert_eq!(
        type_effectiveness_vs_pair(ELECTRIC, make_pair(WATER, FLYING)),
        QUADRUPLE
    );
    assert_eq!(
        type_effectiveness_vs_pair(ROCK, make_pair(FIRE, FLYING)),
        QUADRUPLE
    );
    assert_eq!(
        type_effectiveness_vs_pair(FIRE, make_pair(WATER, ROCK)),
        QUARTER
    );
    assert_eq!(
        type_effectiveness_vs_pair(ELECTRIC, make_pair(WATER, GROUND)),
        IMMUNE
    );
    assert_eq!(
        type_effectiveness_vs_pair(FIRE, make_pair(GRASS, WATER)),
        NEUTRAL
    );
    assert_eq!(
        type_effectiveness_vs_pair(FIRE, make_mono_pair(GRASS)),
        DOUBLE
    );
    assert_eq!(apply_type_effectiveness(100, QUADRUPLE as u32), 400);
    assert_eq!(apply_type_effectiveness(101, QUARTER as u32), 25);
    assert_eq!(apply_type_effectiveness(101, 9), 101);
    assert_eq!(effectiveness_class(QUADRUPLE as i32), 2);
    assert_eq!(effectiveness_class(NEUTRAL as i32), 1);
    assert_eq!(effectiveness_class(HALF as i32), 3);
    assert_eq!(effectiveness_class(IMMUNE as i32), 0);
    assert_eq!(effectiveness_class(-1), 3);

    let mut out = [0u8; 17];
    let n = resisting_types(NORMAL, &mut out);
    assert_eq!(&out[..n as usize], &[ROCK, GHOST, STEEL]);
    let n = resisting_types(ELECTRIC, &mut out);
    assert_eq!(&out[..n as usize], &[GROUND, GRASS, ELECTRIC, DRAGON]);
}

#[test]
fn stage_examples() {
    // Index 6 is stage 0; stat stages are (2+n)/2 and 2/(2-n).
    assert_eq!(apply_stat_stage(200, 6), 200);
    assert_eq!(apply_stat_stage(200, 8), 400);
    assert_eq!(apply_stat_stage(200, 12), 800);
    assert_eq!(apply_stat_stage(200, 0), 50);
    assert_eq!(apply_stat_stage(101, 5), 67);
    // The product is truncated to u16 before dividing: 9000 * 8 = 72000 -> 6464 / 2.
    assert_eq!(apply_stat_stage(9000, 12), 3232);
    // Accuracy stages are (3+n)/3 and 3/(3-n), capped at 100.
    assert_eq!(apply_accuracy_stage(70, 6), 70);
    assert_eq!(apply_accuracy_stage(70, 0), 23);
    assert_eq!(apply_accuracy_stage(70, 7), 93);
    assert_eq!(apply_accuracy_stage(90, 8), 100);
}

#[test]
fn fixed_point_and_percent() {
    assert_eq!(fx_mul(100, 0x1800), 150);
    assert_eq!(fx_mul(3, 0x1555), 4); // 0x3FFF: fraction 0xFFF > 0x800 rounds up
    assert_eq!(fx_mul(1, 0x800), 0); // exactly one half rounds down
    assert_eq!(fx_mul(1, 0x801), 1);
    assert_eq!(fx_mul_min1(1, 0x100), 1);
    assert_eq!(percent_mul(33, 50), 17); // 16.5 rounds up
    assert_eq!(percent_mul(33, 49), 16); // 16.17
    assert_eq!(weather_power_modifier(weather::SUN, FIRE as u32), 0x1800);
    assert_eq!(weather_power_modifier(weather::RAIN, FIRE as u32), 0x800);
    assert_eq!(weather_power_modifier(weather::RAIN, WATER as u32), 0x1800);
    assert_eq!(weather_power_modifier(weather::SUN, WATER as u32), 0x800);
    assert_eq!(weather_power_modifier(weather::SAND, FIRE as u32), FX_ONE);
}

#[test]
fn weather_damage_examples() {
    let hail = weather::HAIL;
    let sand = weather::SAND;
    assert_eq!(
        weather_residual_damage(&Mon::typed(160, FIRE, FIRE), hail),
        10
    );
    assert_eq!(
        weather_residual_damage(&Mon::typed(160, ICE, WATER), hail),
        0
    );
    assert_eq!(
        weather_residual_damage(&Mon::typed(15, FIRE, FIRE), sand),
        1
    );
    assert_eq!(
        weather_residual_damage(&Mon::typed(160, WATER, GROUND), sand),
        0
    );
    assert_eq!(
        weather_residual_damage(&Mon::typed(160, STEEL, STEEL), sand),
        0
    );
    assert_eq!(
        weather_residual_damage(&Mon::typed(160, FIRE, FIRE), weather::RAIN),
        0
    );
    assert_eq!(max_hp_div(&Mon::typed(161, 0, 0), 8), 20);
    assert_eq!(max_hp_div_min1(&Mon::typed(7, 0, 0), 8), 1);
}

#[test]
fn rng_matches_lcg() {
    let mut c = calc_with_seed(0);
    // x1 = 0x269EC3: high word 0.
    assert_eq!(c.rand(0), 0);
    assert_eq!(c.rng.x, LCG_ADD);
    let x2 = LCG_MUL.wrapping_mul(LCG_ADD).wrapping_add(LCG_ADD);
    assert_eq!(c.rand(100), (((x2 >> 32) * 100) >> 32) as u32);
    let ctx = c.rng;
    assert_eq!(MathRandContext32::from_bytes(&ctx.to_bytes()), ctx);
}

#[test]
fn hit_count_distribution() {
    // Over many rolls, 2-5 hits come out at about 35/35/15/15 percent.
    let mut c = calc_with_seed(0x1234_5678_9ABC_DEF0);
    let mut hist = [0u32; 6];
    for _ in 0..10000 {
        hist[c.roll_hit_count(5) as usize] += 1;
    }
    assert_eq!(hist[0] + hist[1], 0);
    for (n, pct) in [(2, 35), (3, 35), (4, 15), (5, 15)] {
        assert!((hist[n] as i32 - pct * 100).abs() < 250, "{hist:?}");
    }
    // Other counts pass through without consuming RNG.
    let before = c.rng;
    assert_eq!(c.roll_hit_count(3), 3);
    assert_eq!(c.rng, before);
}

#[test]
fn sick_cont_examples() {
    let mut c = calc_with_seed(42);
    let mon = Mon::typed(100, 0, 0);
    for _ in 0..50 {
        let s = c.make_default_sick_cont(sick::SLEEP);
        assert_eq!(s.kind(), crate::sick_cont::KIND_TURN);
        assert!((2..=4).contains(&s.slot()));
        let s = c.make_sick_cont(sick::CONFUSION, &mon);
        assert!((2..=5).contains(&s.slot()));
    }
    assert_eq!(c.make_default_sick_cont(sick::BURN), SickCont::permanent());
    assert_eq!(c.make_default_sick_cont(sick::NONE), SickCont::NONE);
    assert_eq!(c.make_sick_cont(sick::ATTRACT, &mon), SickCont::poke(3));
    assert_eq!(c.make_sick_cont(9, &mon), SickCont::permanent());
    let mut out = SickCont(0xDEAD);
    c.make_move_sick_cont(0, &mon, &mut out);
    assert_eq!(out, SickCont(0xDEAD));
    c.make_move_sick_cont(1 | 2 << 4 | 5 << 10, &mon, &mut out);
    assert_eq!(out, SickCont::permanent_count(5, 2));
}

#[test]
fn prize_money_and_exp() {
    let mons: Vec<Mon> = [12, 40, 7]
        .iter()
        .map(|&l| Mon {
            level: l,
            ..Mon::typed(1, 0, 0)
        })
        .collect();
    // Highest level 40, 3 badges (x9): 40 * 4 * 9.
    assert_eq!(prize_money(3, &Party(mons)), 1440);
    // Empty party uses level 1; badges clamp to 8 (x30).
    assert_eq!(prize_money(20, &Party(vec![])), 120);
    let mon = Mon {
        species: 10,
        form: 1,
        level: 30,
        ..Mon::typed(1, 0, 0)
    };
    // personal = (310 + 7 + 9) % 256 = 70; 70 * 30 / 5.
    assert_eq!(exp_yield(&mon, &Personal), 420);
}

#[test]
fn work_buffer_lifecycle() {
    let mut heap = HostHeap::default();
    let mut c = BattleCalc::new();
    let ctx = MathRandContext32 {
        x: 1,
        mul: LCG_MUL,
        add: LCG_ADD,
    };
    c.init(&ctx, &mut heap, HeapId::Unknown);
    assert_eq!(c.work.as_ref().map(|b| b.len()), Some(0x460));
    assert_eq!(heap.last_site, Some(("btl_calc.c", 0x50)));
    assert_eq!(c.rng, ctx);
    c.free(&mut heap);
    assert!(c.work.is_none());
    assert_eq!(heap.live_blocks, 0);
    c.free(&mut heap);
}

#[test]
fn pack_roundtrip() {
    let v = [31, 0, 17, 5, 30, 1];
    let p = pack_5bit_x6(&v);
    let mut o = [0u8; 6];
    unpack_5bit_x6(p, &mut o);
    assert_eq!(o, v);
}

// ---- differential: hashes from the original machine code under an emulator -----------------

fn trunc_mod8(v: u32) -> i32 {
    (v as i32) % 8
}

#[test]
fn pure_matches_rom_fuzz() {
    let mut h = Fnv::new();
    let mut r = Rng::new(0xCA1C);
    for _ in 0..5000 {
        let a = r.next();
        let b = r.next();
        let c = r.next();
        let d = r.next();
        let stage = r.next() % 13;
        h.w(apply_stat_stage(a & 0xFFFF, stage) as u32);
        h.w(apply_stat_stage(a, stage) as u32);
        h.w(apply_accuracy_stage(b & 0xFF, stage) as u32);
        h.w(apply_accuracy_stage(b, stage) as u32);
        let t1 = (c % 18) as u8;
        let t2 = ((c >> 8) % 18) as u8;
        let t3 = ((c >> 16) % 18) as u8;
        h.w(type_effectiveness(t1, t2) as u32);
        h.w(type_effectiveness_vs_pair(t1, make_pair(t2, t3)) as u32);
        h.w(combine_effectiveness((d % 6) as u8, ((d >> 8) % 6) as u8) as u32);
        let mut out = [0xFFu8; 17];
        let n = resisting_types(((d >> 16) % 17) as u8, &mut out);
        h.w(n as u32);
        out.iter().for_each(|&x| h.b(x));
        h.w(calc_base_damage(a % 251, b % 1000, c % 101, d % 1000));
        h.w(calc_base_damage(a, b, c, d));
        h.w(calc_base_damage(a % 251, b % 1000, c % 101, 0));
        h.w(apply_type_effectiveness(a, b % 8));
        h.w(effectiveness_class(trunc_mod8(c)) as u32);
        h.w(fx_mul(a, b));
        h.w(fx_mul(a & 0xFFFF, b & 0x3FFF));
        h.w(fx_mul_min1(a & 0xFF, b & 0xFFF));
        h.w(percent_mul(a, b));
        h.w(percent_mul(a & 0xFFFF, b % 200));
        h.w(weather_power_modifier(a % 6, b % 18));
        h.w(is_not_0_or_3(a % 5) as u32);
        h.w(fun_021d7f00(b % 6));
        h.w(fun_021d7f2c(c % 5, d));
        let six = a.to_le_bytes();
        let six = [six[0], six[1], six[2], six[3], b as u8, (b >> 8) as u8];
        h.w(pack_5bit_x6(&six));
        let mut o6 = [0u8; 6];
        unpack_5bit_x6(c, &mut o6);
        o6.iter().for_each(|&x| h.b(x));
    }
    assert_eq!(h.hex(), "acc20326d79f2526");
}

#[test]
fn rng_matches_rom_fuzz() {
    let mut h = Fnv::new();
    for seed in 0..400u32 {
        let mut r = Rng::new(seed);
        let hi = r.next() as u64;
        let lo = r.next() as u64;
        let mut c = calc_with_seed(hi << 32 | lo);
        let mon = Mon::random(&mut r);
        for _ in 0..20 {
            let op = r.next() % 9;
            let a = r.next();
            let b = r.next();
            let v = match op {
                0 => c.rand(a % 300),
                1 => c.roll_critical(a % 6) as u32,
                2 => c.roll_percent(a % 120) as u32,
                3 => c.rand_range(a % 50, b % 50),
                4 => c.roll_hit_count(a % 7),
                5 => c.make_default_sick_cont(a % 9).0,
                6 => c.make_sick_cont((a % 12).wrapping_sub(2), &mon).0,
                7 => {
                    let mut o = SickCont(b);
                    c.make_move_sick_cont(a as u16, &mon, &mut o);
                    o.0
                }
                _ => c.rand(a),
            };
            h.w(v);
            h.w(c.rng.x as u32);
            h.w((c.rng.x >> 32) as u32);
        }
    }
    assert_eq!(h.hex(), "d080ff788291d0a8");
}

#[test]
fn hooks_match_rom_fuzz() {
    let mut h = Fnv::new();
    for seed in 0..2000u32 {
        let mut r = Rng::new(seed);
        let mon = Mon::random(&mut r);
        let d = r.next() % 40;
        h.w(max_hp_div(&mon, d));
        h.w(max_hp_div_min1(&mon, d));
        h.w(weather_residual_damage(&mon, r.next() % 6));
        h.w(exp_yield(&mon, &Personal));
        h.w(fun_021d80a8(&mon));
        let n = r.next() % 7;
        let party = Party((0..n).map(|_| Mon::random(&mut r)).collect());
        h.w(prize_money(r.next() % 12, &party));
        let v = r.next() % 100;
        h.w(fun_021d7f38(&Arm9, v) as u32);
        h.w(fun_021d7f4c(&Arm9, v) as u32);
        h.w(fun_021d7f60(&Arm9, v) as u32);
    }
    assert_eq!(h.hex(), "c0fc8425b1d2fa14");
}

/// Widened [`pure_matches_rom_fuzz`] (validation V-0005): full-range codes for the match-based
/// helpers (table-indexed ones stay in range), divisor 0, full-width damage and multipliers.
#[test]
fn pure_wide_matches_rom_fuzz() {
    let mut h = Fnv::new();
    let mut r = Rng::new(0xCA1D);
    for _ in 0..20000 {
        let a = r.next();
        let b = r.next();
        let c = r.next();
        let d = r.next();
        h.w(calc_base_damage(a, b, c, 0));
        h.w(calc_base_damage(a % 251, b % 1000, c % 101, d));
        h.w(apply_type_effectiveness(a, b));
        h.w(apply_type_effectiveness(a, b % 16));
        h.w(effectiveness_class(c as i32) as u32);
        h.w(effectiveness_class((c % 9) as i32 - 4) as u32);
        h.w(fx_mul_min1(a, b));
        h.w(weather_power_modifier(a, b));
        h.w(weather_power_modifier(a % 8, b % 20));
        h.w(is_not_0_or_3(a) as u32);
        h.w(fun_021d7f00(b));
        h.w(fun_021d7f2c(c, d));
        h.w(fun_021d7f2c(c % 5, d));
    }
    assert_eq!(h.hex(), "5d03a40cdcc1ff18");
}

/// Widened [`rng_matches_rom_fuzz`] (validation V-0005): full-range `rand` / `rand_range`
/// (including the 0..=u32::MAX span, where `hi - lo + 1` wraps to 0 and the raw high word is
/// used), sick ids, hit counts and percents.
#[test]
fn rng_wide_matches_rom_fuzz() {
    let mut h = Fnv::new();
    let mut full_spans = 0;
    for seed in 0..2000u32 {
        let mut r = Rng::new(seed);
        let hi = r.next() as u64;
        let lo = r.next() as u64;
        let mut c = calc_with_seed(hi << 32 | lo);
        let mon = Mon::wide(&mut r);
        for _ in 0..40 {
            let op = r.next() % 10;
            let a = r.next();
            let b = r.next();
            let v = match op {
                0 => c.rand(if !b.is_multiple_of(4) { a } else { 0 }),
                1 => c.rand_range(a, b),
                2 => {
                    let lo = if a.is_multiple_of(3) { 0 } else { a };
                    let hi = u32::MAX - b % 2;
                    full_spans += (lo == 0 && hi == u32::MAX) as u32;
                    if b & 2 != 0 {
                        c.rand_range(hi, lo)
                    } else {
                        c.rand_range(lo, hi)
                    }
                }
                3 => c.roll_hit_count(a),
                4 => c.make_default_sick_cont(a).0,
                5 => c.make_sick_cont(a, &mon).0,
                6 => {
                    let mut o = SickCont(b);
                    c.make_move_sick_cont(a as u16, &mon, &mut o);
                    o.0
                }
                7 => c.roll_percent(a) as u32,
                8 => c.roll_critical(a % 5) as u32,
                _ => c.make_default_sick_cont(a % 9).0,
            };
            h.w(v);
            h.w(c.rng.x as u32);
            h.w((c.rng.x >> 32) as u32);
        }
    }
    assert!(full_spans > 0);
    assert_eq!(h.hex(), "e4e27e20aaef739c");
}

/// Widened [`hooks_match_rom_fuzz`] (validation V-0005): full-u32 max HP (the `& 0xFFFF` of
/// max_hp / 16, max_hp >= 2^31), level (u32 wrap of max_level * 4 * mult), form (`& 0xFFFF`),
/// divisor (including 0), weather and badges (unsigned clamp).
#[test]
fn hooks_wide_match_rom_fuzz() {
    let mut h = Fnv::new();
    for seed in 0..5000u32 {
        let mut r = Rng::new(seed);
        let mon = Mon::wide(&mut r);
        let x = r.next();
        let d = if x.is_multiple_of(3) { x % 40 } else { x };
        h.w(max_hp_div(&mon, d));
        h.w(max_hp_div_min1(&mon, d));
        h.w(weather_residual_damage(&mon, r.next()));
        h.w(weather_residual_damage(&mon, r.next() % 6));
        h.w(exp_yield(&mon, &Personal));
        h.w(fun_021d80a8(&mon));
        let n = r.next() % 7;
        let party = Party((0..n).map(|_| Mon::wide(&mut r)).collect());
        h.w(prize_money(r.next(), &party));
        let v = r.next();
        h.w(fun_021d7f38(&Arm9, v) as u32);
        h.w(fun_021d7f4c(&Arm9, v) as u32);
        h.w(fun_021d7f60(&Arm9, v) as u32);
    }
    assert_eq!(h.hex(), "44c219e197d73914");
}
