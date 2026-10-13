use super::*;
use crate::testutil::{Fnv, Rng};

pub(crate) fn wide(a: u32, b: u32) -> u32 {
    [b % 64, b % 0x400, b % 0x20000, b][((a >> 28) & 3) as usize]
}

pub(crate) fn mix(a: u32, b: u32) -> u32 {
    let x = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
    x ^ (x >> 15)
}

/// Mirrors `Env` in tools/rom-diff/bpp.py.
#[derive(Default)]
pub(crate) struct MockEnv {
    pub log: Vec<[u32; 4]>,
    pub field: bool,
}

impl BppEnv for MockEnv {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.log.push([1, pp, id, 0]);
        mix(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.log.push([2, pp, id, value]);
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.log.push([3, pp, 0, 0]);
        mix(pp, 0xFB) & 1
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.log.push([4, pp, token, 0]);
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.log.push([5, pp, mv, slot]);
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.log.push([6, pp, 0, 0]);
        mix(pp, 0xF00)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.log.push([7, pp, 0, 0]);
        mix(pp, 0xF90) % 7
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.log.push([8, mv, pp_ups, 0]);
        mix(mv, pp_ups) & 0x3F
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.log.push([9, species, form, param]);
        mix(species ^ (form << 16), param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.log.push([10, id, 0, 0]);
        self.field
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.log.push([11, species, form, level]);
        mix(species ^ (form << 16), level) & 0x1FFFF
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.log.push([12, pp, 0, 0]);
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.log.push([13, pp, status, 0]);
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.log.push([14, pp, form, 0]);
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.log.push([15, value, 0, 0]);
        value % 3
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.log.push([16, sick, 0, 0]);
        mix(sick, 0x99) & 1 != 0
    }
}

pub(crate) fn random_bpp(r: &mut Rng) -> BattleMon {
    let mut b = BattleMon::default();
    for i in 0..BPP_SIZE / 4 {
        b.set_u32(4 * i, r.next());
    }
    b.raw[0x140] %= 5;
    b.raw[0x1EF] %= 3;
    for i in 0..3 {
        b.raw[0x1EC + i] %= 7;
    }
    b.raw[0xD0] %= 25;
    b
}

pub(crate) fn hash_state(h: &mut Fnv, b: &BattleMon, env: &mut MockEnv) {
    b.raw.iter().for_each(|&x| h.b(x));
    for e in env.log.drain(..) {
        e.iter().for_each(|&v| h.w(v));
    }
}

fn op_a(b: &mut BattleMon, env: &mut MockEnv, op: u32, a: u32, x: u32, d: u32) -> Vec<u32> {
    let s = a % 4;
    match op {
        0 => {
            b.move_clear_used(s);
            vec![]
        }
        1 => {
            b.move_replace(
                s,
                if x >> 28 == 0 { 0 } else { x & 0xFFFF },
                d & 0xFF,
                (a >> 8) & 1 != 0,
                env,
            );
            vec![]
        }
        2 => vec![b.move_load_from_party(s, x, env) as u32],
        3 => vec![b.moves_load(x, (a >> 8) & 1 != 0, env)],
        4 => {
            b.moves_store(env);
            vec![]
        }
        5 => {
            b.moves_reload(env);
            vec![]
        }
        6 => {
            b.moves_restore();
            vec![]
        }
        7 => vec![
            b.move_count() as u32,
            b.orig_move_count() as u32,
            b.used_move_count() as u32,
            b.distinct_moves_used() as u32,
        ],
        8 => vec![
            b.move_id(s) as u32,
            b.orig_move_id(s) as u32,
            b.move_used_flag(s) as u32,
        ],
        9 => {
            let mut dst = BattleMon::default();
            b.copy_moves_to(&mut dst);
            dst.raw.iter().map(|&v| v as u32).collect()
        }
        10 => {
            let (m, pp, max) = b.move_info(s);
            vec![m as u32, pp as u32, max as u32]
        }
        11 => vec![
            b.move_pp_missing(s) as u32,
            b.orig_move_pp_missing(s) as u32,
            b.move_pp(a % 6) as u32,
        ],
        12 => {
            let mv = if x & 1 != 0 {
                b.move_id(s) as u32
            } else {
                x & 0xFFFF
            };
            vec![
                b.move_pp_of(mv) as u32,
                b.knows_move(mv) as u32,
                b.move_slot_of(mv) as u32,
            ]
        }
        13 => vec![b.move_pp_full(s, x & 1 != 0) as u32],
        14 => {
            b.move_spend_pp(s, wide(a, x), env);
            vec![]
        }
        15 => {
            b.move_spend_orig_pp(s, wide(a, x), env);
            vec![]
        }
        16 => vec![b.move_restore_pp(s, wide(a, x)) as u32],
        17 => vec![b.move_restore_orig_pp(s, wide(a, x)) as u32],
        18 => {
            b.move_mark_used(s);
            vec![]
        }
        19 => {
            b.reset_switch_state();
            vec![]
        }
        20 => {
            b.clear_157();
            vec![]
        }
        21 => {
            b.reset_stages();
            vec![]
        }
        22 => vec![
            b.has_negative_stage() as u32,
            b.clear_negative_stages() as u32,
            b.has_negative_stage() as u32,
        ],
        23 => {
            let st = 1 + a % 7;
            let delta = (x % 65) as i32 - 32;
            vec![
                b.can_change_stage(st, delta) as u32,
                b.stage_headroom(st) as u32,
            ]
        }
        24 => vec![b.raise_stage(a % 10, wide(a, x))],
        25 => vec![b.lower_stage(a % 10, wide(a, x))],
        26 => {
            b.set_stage(a % 10, wide(a, x));
            vec![]
        }
        27 => vec![
            b.crit_stage() as u32,
            b.add_crit_boost((x % 201) as i32 - 100) as u32,
        ],
        28 => {
            b.damage(wide(a, x));
            vec![]
        }
        29 => {
            b.heal(wide(a, x));
            vec![]
        }
        30 => {
            b.faint();
            vec![]
        }
        31 => {
            let r = vec![b.held_item() as u32, b.u16_146() as u32];
            b.set_held_item(x as u16);
            r
        }
        32 => {
            let n = x % 16;
            vec![b.flag_a(n) as u32, b.flag_b(n) as u32]
        }
        33 => {
            let n = x % 16;
            match a % 3 {
                0 => b.set_flag_a(n),
                1 => b.set_flag_b(n),
                _ => b.clear_flag_b(n),
            }
            vec![]
        }
        34 => vec![b.first_flag_b_3546(), b.any_flag_b_3546() as u32],
        35 => {
            let r = vec![b.get_pokemon(), b.visible_mon()];
            if a & 1 != 0 {
                b.set_illusion_mon(x);
            } else {
                b.clear_illusion_mon();
            }
            r
        }
        _ => unreachable!(),
    }
}

/// Moves, stages, HP, flags and Illusion: random states and operations, hashed against the
/// original code (`tools/rom-diff/bpp.py suite_a`).
#[test]
fn suite_a_matches_rom() {
    let mut h = Fnv::new();
    let mut env = MockEnv::default();
    for seed in 0..150u32 {
        let mut r = Rng::new(0xB0 + seed);
        let mut b = random_bpp(&mut r);
        env.field = r.next() & 1 != 0;
        for _ in 0..40 {
            let op = r.next() % 36;
            let (a, x, d) = (r.next(), r.next(), r.next());
            for v in op_a(&mut b, &mut env, op, a, x, d) {
                h.w(v);
            }
            hash_state(&mut h, &b, &mut env);
        }
    }
    assert_eq!(h.hex(), "7d4b099bf7737bd6");
}

#[test]
fn stages_hand_checked() {
    let mut b = BattleMon::default();
    b.reset_stages();
    assert_eq!(b.raise_stage(1, 2), 2);
    assert_eq!(b.stage(0), 8);
    assert_eq!(b.raise_stage(1, 9), 4); // clamped at +6 (12)
    assert_eq!(b.raise_stage(1, 1), 0);
    assert_eq!(b.lower_stage(5, 20), 6); // speed down to -6 (0)
    assert!(b.has_negative_stage());
    assert!(!b.can_change_stage(5, -1));
    assert!(b.clear_negative_stages());
    assert_eq!(b.stage(4), STAGE_NEUTRAL);
}

#[test]
fn hp_hand_checked() {
    let mut b = BattleMon::default();
    b.set_u16(0x0E, 100);
    b.set_u16(0x10, 40);
    b.damage(50);
    assert_eq!(b.hp(), 0);
    b.heal(250);
    assert_eq!(b.hp(), 100);
}

const LCG_MUL: u64 = 0x5D58_8B65_6C07_8965;
const LCG_ADD: u64 = 0x0026_9EC3;

fn op_b(
    b: &mut BattleMon,
    b2: &mut BattleMon,
    env: &mut MockEnv,
    op: u32,
    a: u32,
    x: u32,
    d: u32,
) -> Vec<u32> {
    match op {
        0 => {
            if a & 3 == 0 {
                // V-0012 hint: grounded (sick 0x18) with Flying in either slot (or both), which
                // random op sequences almost never combine.
                b.set_sick(0x18, SickCont(d | 1));
                b.set_type_pair(
                    [0x0211, 0x1102, 0x0202, ((x % 18) << 8 | 2) as u16][((a >> 2) & 3) as usize],
                );
            }
            let (t1, t2) = b.types();
            vec![
                t1 as u32,
                t2 as u32,
                b.type_pair() as u32,
                b.has_type(x % 32) as u32,
            ]
        }
        1 => {
            let pair = match (x >> 16) & 3 {
                0 => 0x1111,
                1 => 0x1102,
                _ => x as u16,
            };
            b.set_type_pair(pair);
            b.set_ability(if (d >> 16) & 3 == 0 { 0x30 } else { d as u16 });
            vec![]
        }
        2 => vec![
            b.get(a % 23, env),
            b.get_unstaged(x % 14, env),
            b.get_for_crit(6 + d % 8, env),
        ],
        3 => {
            let mut out = [0u16; 7];
            b.read_party_stats(&mut out, env);
            out.iter()
                .flat_map(|v| v.to_le_bytes())
                .map(|v| v as u32)
                .collect()
        }
        4 => {
            b.set_stat(7 + a % 7, x as u16, env);
            vec![]
        }
        5 => {
            if a & 3 == 1 {
                b.set_u16(0x10, b.u16_at(0x0E));
            } else if a & 3 == 2 {
                b.set_u16(0x10, 0);
            }
            vec![
                b.is_full_hp(env) as u32,
                b.is_fainted(env) as u32,
                b.is_alive(env) as u32,
            ]
        }
        6 => {
            b.set_u16(0x0E, ((x & 0xFFFF) >> ((a >> 20) & 15)) as u16);
            let hp = if (a >> 28) & 7 == 0 {
                0
            } else {
                (a & 0xFFFF) >> ((a >> 24) & 15)
            };
            b.set_u16(0x10, hp as u16);
            vec![b.hp_percent_fx() as u32]
        }
        7 => {
            let s = a % 36;
            vec![
                b.has_sick(s) as u32,
                b.get_sick_cont(s).0,
                b.sick_turns(s) as u32,
                b.sick_param(s) as u32,
                b.major_status(),
            ]
        }
        8 => {
            b.set_sick(a % 36, SickCont(x));
            vec![]
        }
        9 => {
            if a & 1 != 0 {
                b.cure_major_status();
            } else {
                b.cure_sick(x % 36);
            }
            vec![]
        }
        10 => {
            b.cure_sicks_linked_to(x % 40);
            vec![]
        }
        11 => {
            if (a >> 8) & 3 == 0 {
                // V-0012 hint: a fresh kind-1 cont with a count-up limit in bits 3-8 (turns 0 < limit)
                b.set_sick(
                    a % 36,
                    SickCont((d & !0x1FF) | (((x >> 2) % 63 + 1) << 3) | 1),
                );
            }
            let mut prev = SickCont(0xCCCC_CCCC);
            let mut ended = 0xCCCC_CCCCu32;
            let r = b.tick_sick(
                a % 36,
                if x & 1 != 0 { Some(&mut prev) } else { None },
                if x & 2 != 0 { Some(&mut ended) } else { None },
            );
            vec![r as u32, prev.0, ended]
        }
        12 => {
            if a & 3 == 0 {
                b.set_ability(0x30); // V-0012 hint: Early Bird doubles the sleep countdown
            }
            vec![b.tick_sleep() as u32, b.tick_confusion() as u32]
        }
        13 => vec![b.sick_damage(a % 12, env)],
        14 => {
            b.clear_sicks(a & 1 != 0);
            vec![]
        }
        15 => {
            b.reload_from_party(x, a & 1 != 0, (a >> 1) & 1 != 0, env);
            vec![]
        }
        16 => {
            b.reload_types_and_stats(x, a & 1 != 0, env);
            vec![]
        }
        17 => {
            b.undo_transform(env);
            vec![]
        }
        18 => {
            match a % 3 {
                0 => b.reset_on_switch_out(env),
                1 => b.reset_on_battle_end(env),
                _ => b.reset_on_replace(env),
            }
            vec![]
        }
        19 => {
            b.copy_passable_state(b2, env);
            b2.raw.iter().map(|&v| v as u32).collect()
        }
        20 => {
            if x & 1 != 0 {
                b.set_u16(0x0C, 0x1EC);
            }
            b.change_form((a % 32) as u8, env);
            vec![]
        }
        21 => {
            b.refresh_from_party(env);
            vec![]
        }
        22 => {
            b.store_to_party(a & 1 != 0, env);
            vec![]
        }
        23 => {
            b.set_pokemon(x, env);
            b.set_pp_0x98(d, env);
            b.recalc_party_stats(env);
            vec![]
        }
        24 => {
            if x & 1 != 0 {
                b2.set_u16(0x1F2, 0);
            }
            if x & 2 != 0 {
                b2.raw[0x1B] &= 0xDF;
                b.raw[0x1B] &= 0xDF;
            }
            let t = b2.clone();
            vec![b.transform_into(&t, env) as u32]
        }
        25 => {
            let mv = if (x >> 16) & 1 != 0 {
                b.last_move()
            } else {
                ((a >> 8) % 4) as u16
            };
            b.record_move(
                (a & 3) as u8,
                x & 1 != 0,
                ((x >> 8) & 0x1F) as u8,
                mv,
                d as u16,
            );
            vec![]
        }
        26 => {
            let r = vec![
                b.move_streak() as u32,
                b.last_move() as u32,
                b.last_move_type() as u32,
                b.last_move_orig() as u32,
                b.last_move_target() as u32,
                b.flag_1b_bit7() as u32,
                b.weight() as u32,
            ];
            b.set_weight((x % 3) as u16);
            r
        }
        27 => {
            b.set_148(x as u16);
            vec![]
        }
        28 => {
            b.on_turn_start();
            vec![]
        }
        29 => {
            b.clear_flag_a(x % 16);
            vec![]
        }
        30 => {
            let r = vec![b.u14() as u32];
            if a & 1 != 0 {
                b.set_u14(x as u16);
            } else {
                b.clear_u14();
            }
            r
        }
        31 => {
            if a & 1 != 0 {
                b.clear_history();
            } else {
                b.advance_history();
            }
            vec![]
        }
        32 => {
            b.push_history(&[a as u16, (a >> 16) as u16, x as u16, (x >> 16) as u16]);
            vec![]
        }
        33 => {
            let rec = b.history_record(a % 4, x % 8);
            let mut v = vec![b.history_count(a % 4) as u32, rec.is_some() as u32];
            let rec = rec.unwrap_or_default();
            v.extend(rec.iter().flat_map(|r| r.to_le_bytes()).map(|x| x as u32));
            v
        }
        34 => {
            b.set_u8_157(a % 5, x as u8);
            vec![b.u8_157(d % 5) as u32]
        }
        35 => {
            b.set_1f6((a % 0x20) as u8, x as u16);
            let g = b.get_1f6();
            let (p, q) = g.unwrap_or((0, 0));
            vec![g.is_some() as u32, p as u32, q as u32, b.has_1f6() as u32]
        }
        36 => {
            b.clear_1f6();
            vec![b.has_1f6() as u32]
        }
        37 => {
            if a & 1 != 0 {
                b.make_substitute(x as u16);
            } else {
                b.set_substitute_hp(0);
            }
            vec![b.has_substitute() as u32, b.substitute_hp() as u32]
        }
        38 => {
            let mut dmg = x as u16;
            let r = b.hit_substitute(&mut dmg);
            vec![r as u32, dmg as u32]
        }
        39 => {
            if a & 1 != 0 {
                b.clear_faced();
            }
            b.add_faced((x % 30) as u8);
            vec![b.faced_count() as u32, b.faced(d % 30) as u32]
        }
        40 => {
            b.set_u32(0x08, wide(d, a));
            b.set_u8(0x18, (a >> 16) as u8);
            let mut exp = wide(a, x);
            let mut info = LevelUpInfo::default();
            let r = b.gain_exp(&mut exp, Some(&mut info), env);
            let mut v = vec![r as u32, exp];
            v.extend(info.to_bytes().iter().map(|&x| x as u32));
            v.push(b.exp_for_max_level(env));
            v
        }
        41 => {
            let mut calc = crate::calc::BattleCalc::new();
            calc.rng = crate::calc::MathRandContext32 {
                x: (a as u64) << 32 | x as u64,
                mul: LCG_MUL,
                add: LCG_ADD,
            };
            *b = BattleMon::new(x, (a & 0xFF) as u8, &mut calc, env);
            vec![1, calc.rng.x as u32, (calc.rng.x >> 32) as u32]
        }
        _ => unreachable!(),
    }
}

/// Types, BattleMon_Get, conditions, switching, Transform, history, substitute, EXP and the
/// constructor, hashed against the original code (`tools/rom-diff/bpp.py suite_b`).
#[test]
fn suite_b_matches_rom() {
    let mut h = Fnv::new();
    let mut env = MockEnv::default();
    for seed in 0..200u32 {
        let mut r = Rng::new(0xB000 + seed);
        let mut b = random_bpp(&mut r);
        let mut b2 = random_bpp(&mut r);
        env.field = r.next() & 1 != 0;
        for _ in 0..30 {
            let op = r.next() % 42;
            let (a, x, d) = (r.next(), r.next(), r.next());
            for v in op_b(&mut b, &mut b2, &mut env, op, a, x, d) {
                h.w(v);
            }
            hash_state(&mut h, &b, &mut env);
        }
    }
    assert_eq!(h.hex(), "542d98380ee76f0d");
}

/// `set_tagged_payload` against the original (`tools/rom-diff/bpp.py tagged`).
#[test]
fn tagged_payload_matches_rom() {
    let mut h = Fnv::new();
    let mut r = Rng::new(0x4D64);
    for _ in 0..300u32 {
        let mut buf = vec![0u8; 0x48];
        let len = r.next() % 0x40;
        buf[0..4].copy_from_slice(&(len << 8 | (r.next() & 0xFF)).to_le_bytes());
        let src: Vec<u8> = (0..0x40).map(|_| r.next() as u8).collect();
        let tag = r.next() as u8;
        set_tagged_payload(&mut buf, &src, tag);
        buf.iter().for_each(|&b| h.b(b));
    }
    assert_eq!(h.hex(), "037e8b18d36622a0");
}

/// B6-last: `021d5c44` returns `i32::MAX` for 0/0 (NaN through the soft-float conversion; checked
/// against the ROM for every hp/max-HP pair in a grid), and saturates for hp > 0 with max HP 0.
#[test]
fn hp_percent_fx_edge_cases() {
    let mut b = BattleMon::default();
    b.set_u16(0x0E, 0);
    b.set_u16(0x10, 0);
    assert_eq!(b.hp_percent_fx(), i32::MAX);
    b.set_u16(0x10, 5);
    assert_eq!(b.hp_percent_fx(), i32::MAX);
    b.set_u16(0x0E, 200);
    b.set_u16(0x10, 100);
    assert_eq!(b.hp_percent_fx(), 50 * 4096);
}
