//! Move targeting and small flow helpers: asm/battle_calc.s `0x021D80BC`, `0x021D81F8`,
//! `0x021D8230`-`0x021D82CC`, `0x021D85B0` and `0x021D85C0` (overlay 93).
//!
//! The position and move-data routines these call are not ported; they are the [`TargetEnv`]
//! trait. Names of the position routines are inferred from how the code uses them.

use crate::battle_mon::{value, BattleMon as Bpp, BppEnv};
use crate::calc::{fun_021d80a8, BattleCalc, MathRandContext32};
use crate::hooks::BattleMon;

/// C: `FUN_overlay_d_93__021d80bc` writes the "no target" result 6.
pub const POS_NONE: u32 = 6;
/// C: the `FUN_0201BD28(move, 0x1B)` value for Curse (move 0xAE) is replaced by
/// `fun_021d80a8`.
pub const MOVE_CURSE: u32 = 0xAE;
/// C: `FUN_0201BD28(move, id)` ID read by `FUN_overlay_d_93__021d80bc` (the target class).
pub const WAZA_PARAM_TARGET: u32 = 0x1B;
/// C: `FUN_0201BEB8(move, id)` ID read in triple battles by `FUN_overlay_d_93__021d80bc`.
pub const WAZA_FLAG_TRIPLE: u32 = 0x0B;

/// Undecompiled routines called by targeting. The battle main handle (`r0` of the original
/// calls) is owned by the implementation.
pub trait TargetEnv {
    /// C: `FUN_overlay_d_93__021b8588`: `**main` field at +4; used as the battle rule
    /// (0 single, 2 triple; 1 presumably double).
    fn rule(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021b8d8c(main, arg1, a)`: position lookup; returns 6 for none.
    fn fun_021b8d8c(&mut self, arg1: u32, a: u32) -> u32;
    /// C: `FUN_0201BD28(move, id)`: move data field.
    fn waza_param(&mut self, waza: u32, id: u32) -> u32;
    /// C: `FUN_0201BEB8(move, id)`: move data flag (only the low byte is used).
    fn waza_flag(&mut self, waza: u32, id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021b8c50(main, pos, a)`.
    fn fun_021b8c50(&mut self, pos: u32, a: u32) -> u32;
    /// C: `FUN_overlay_d_93__021b8d38(main, arg1, code, out)`: writes candidate positions and
    /// returns how many.
    fn fun_021b8d38(&mut self, arg1: u32, code: u32, out: &mut [u8; 8]) -> u32;
    /// C: `FUN_overlay_d_93__021b8778(main, code, out)`: writes candidate positions and
    /// returns how many.
    fn fun_021b8778(&mut self, code: u32, out: &mut [u8; 8]) -> u32;
    /// C: `FUN_overlay_d_93__021c82b8(flow, a)`.
    fn fun_021c82b8(&mut self, a: u32) -> u32;
}

/// C: `FUN_overlay_d_93__021d80bc`: resolves the target position of `waza` used by `mon`.
///
/// `arg1` is the second argument of the original (passed through to the position lookups).
/// The buffers written by the callees hold at most 8 entries (the original's stack buffer);
/// `n` returned by a callee must not exceed that, which indexing enforces by panicking.
pub fn fun_021d80bc<E: TargetEnv, M: BattleMon>(
    env: &mut E,
    calc: &mut BattleCalc,
    arg1: u32,
    mon: &M,
    waza: u32,
) -> u32 {
    let rule = env.rule();
    let mut pos = env.fun_021b8d8c(arg1, mon.poke_id() as u32);
    let mut class = env.waza_param(waza, WAZA_PARAM_TARGET);
    if waza == MOVE_CURSE {
        class = fun_021d80a8(mon);
    }
    if rule == 0 {
        if class > 0xD {
            return POS_NONE;
        }
        return match class {
            0 | 3 | 4 | 5 | 9 => env.fun_021b8c50(pos, 0),
            1 | 7 => pos,
            _ => POS_NONE,
        };
    }
    let flag = if rule == 2 {
        env.waza_flag(waza, WAZA_FLAG_TRIPLE) & 0xFF
    } else {
        0
    };
    if class > 9 {
        return POS_NONE;
    }
    match class {
        0 | 3 | 9 => {
            let hi = if flag != 0 { 6 } else { 1 };
            pos = (hi << 8 | pos) & 0xFFFF;
        }
        1 => pos = (3 << 8 | pos) & 0xFFFF,
        2 => {
            pos = if flag != 0 {
                (7 << 8 | pos) & 0xFFFF
            } else {
                (1 << 10 | pos) & 0xFFFF
            }
        }
        7 => return pos,
        _ => return POS_NONE,
    }
    let mut buf = [0u8; 8];
    let n = env.fun_021b8d38(arg1, pos, &mut buf);
    if n != 0 {
        let i = calc.rand(n) as u8;
        return env.fun_021b8d8c(arg1, buf[i as usize] as u32);
    }
    let n = env.fun_021b8778(pos, &mut buf);
    if n != 0 {
        let i = calc.rand(n) as u8;
        return buf[i as usize] as u32;
    }
    POS_NONE
}

/// C: `FUN_overlay_d_93__021d81f8`: [`fun_021d80bc`] without advancing the battle RNG.
///
/// The original copies the 24-byte RNG context at `0x021F6368` into `saved`, runs the lookup,
/// then copies `saved` back; `saved` keeps the pre-call context.
pub fn fun_021d81f8<E: TargetEnv, M: BattleMon>(
    env: &mut E,
    calc: &mut BattleCalc,
    arg1: u32,
    mon: &M,
    waza: u32,
    saved: &mut MathRandContext32,
) -> u32 {
    *saved = calc.rng;
    let r = fun_021d80bc(env, calc, arg1, mon, waza);
    calc.rng = *saved;
    r
}

/// C: `FUN_overlay_d_93__021b902c`: `a == b` or `a == (b + 2) & 3` (a client and its
/// partner). Both compares are on the full register values; only the second uses a byte.
pub fn is_partner_or_self(a: u32, b: u32) -> bool {
    a == b || a == (b.wrapping_add(2) & 3)
}

/// A list of up to four distinct IDs; 4 marks an empty slot.
///
/// C: the 8-byte structure initialised by `FUN_overlay_d_93__021d8230`: `u32` count at +0 and
/// four `u8` slots at +4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdSet {
    count: u32,
    ids: [u8; 4],
}

impl Default for IdSet {
    fn default() -> Self {
        Self::new()
    }
}

impl IdSet {
    /// C: `FUN_overlay_d_93__021d8230`: fills the slots with 4 and clears the count.
    pub fn new() -> Self {
        IdSet {
            count: 0,
            ids: [4; 4],
        }
    }

    /// C: `FUN_overlay_d_93__021d8244`: appends `id` unless a slot already equals it. Slots
    /// hold 4 when empty, so adding 4 never appends.
    ///
    /// Deviation: `id` is a `u8`. The original takes a 32-bit value and stores its low byte, so
    /// a value such as 0x104 is counted without filling a slot and can push the count past 4.
    pub fn add(&mut self, id: u8) {
        for slot in &mut self.ids {
            let cur = *slot;
            if id == cur {
                return;
            }
            if cur == 4 {
                *slot = id;
                self.count += 1;
                return;
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d826c`
    pub fn count(&self) -> u32 {
        self.count
    }

    /// C: `FUN_overlay_d_93__021d8270`: classifies the set against `client` (see
    /// [`is_partner_or_self`]): 4 if no member matches, 3 if all match (or, with `mixed_limit`
    /// at most 1, some match), 2 for a mix with `mixed_limit` above 1.
    pub fn classify(&self, client: u32, mixed_limit: u32) -> u32 {
        let mut any = false;
        let mut other = false;
        for i in 0..self.count as usize {
            if is_partner_or_self(self.ids[i] as u32, client) {
                any = true;
            } else {
                other = true;
            }
        }
        if any && other {
            if mixed_limit > 1 {
                2
            } else {
                3
            }
        } else if any {
            3
        } else {
            4
        }
    }
}

/// C: `FUN_overlay_d_93__021d85b0`: the unstaged Speed stat as a `u16`.
pub fn speed_unstaged<E: BppEnv>(mon: &Bpp, env: &mut E) -> u32 {
    mon.get_unstaged(value::SPEED, env) & 0xFFFF
}

/// C: `FUN_overlay_d_93__021d85c0`: a `percent` roll, or failing that `FUN_overlay_d_93__021c82b8`
/// with 1.
pub fn fun_021d85c0<E: TargetEnv>(env: &mut E, calc: &mut BattleCalc, percent: u32) -> bool {
    calc.roll_percent(percent) || env.fun_021c82b8(1) != 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poke_type::types;
    use crate::testutil::{Fnv, Rng};

    const LCG_MUL: u64 = 0x5D58_8B65_6C07_8965;
    const LCG_ADD: u64 = 0x0026_9EC3;

    fn mix(a: u32, b: u32) -> u32 {
        let x = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
        x ^ (x >> 15)
    }

    /// Mirrors `target()` in tools/rom-diff/gen.py.
    #[derive(Default)]
    struct Env {
        log: Vec<[u32; 4]>,
        rule: u32,
    }

    impl TargetEnv for Env {
        fn rule(&mut self) -> u32 {
            self.log.push([1, 0, 0, 0]);
            self.rule
        }
        fn fun_021b8d8c(&mut self, arg1: u32, a: u32) -> u32 {
            self.log.push([2, arg1, a, 0]);
            mix(arg1, a) % 8
        }
        fn waza_param(&mut self, waza: u32, id: u32) -> u32 {
            self.log.push([3, waza, id, 0]);
            mix(waza, id) % 16
        }
        fn waza_flag(&mut self, waza: u32, id: u32) -> u32 {
            self.log.push([4, waza, id, 0]);
            mix(waza, id)
        }
        fn fun_021b8c50(&mut self, pos: u32, a: u32) -> u32 {
            self.log.push([5, pos, a, 0]);
            mix(pos, a + 5)
        }
        fn fun_021b8d38(&mut self, arg1: u32, code: u32, out: &mut [u8; 8]) -> u32 {
            self.log.push([6, arg1, code, 0]);
            let n = mix(arg1, code) % 5;
            for i in 0..n {
                out[i as usize] = mix(code, i) as u8;
            }
            n
        }
        fn fun_021b8778(&mut self, code: u32, out: &mut [u8; 8]) -> u32 {
            self.log.push([7, code, 0, 0]);
            let n = mix(code, 0x77) % 5;
            for i in 0..n {
                out[i as usize] = mix(code, 0x100 + i) as u8;
            }
            n
        }
        fn fun_021c82b8(&mut self, a: u32) -> u32 {
            self.log.push([8, a, 0, 0]);
            mix(a, 0x82) & 1
        }
    }

    struct Mon {
        poke_id: u8,
        t1: u8,
        t2: u8,
    }

    impl BattleMon for Mon {
        fn get(&self, id: u32) -> u32 {
            panic!("unexpected BattleMon_Get id {id:#x}")
        }
        fn has_type(&self, ty: u32) -> bool {
            ty != types::NONE as u32 && (self.t1 as u32 == ty || self.t2 as u32 == ty)
        }
        fn poke_id(&self) -> u8 {
            self.poke_id
        }
        fn species(&self) -> u16 {
            0
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

    #[test]
    fn single_battle_hand_checked() {
        struct Fixed;
        impl TargetEnv for Fixed {
            fn rule(&mut self) -> u32 {
                0
            }
            fn fun_021b8d8c(&mut self, _: u32, a: u32) -> u32 {
                a + 10
            }
            fn waza_param(&mut self, _: u32, _: u32) -> u32 {
                1
            }
            fn waza_flag(&mut self, _: u32, _: u32) -> u32 {
                0
            }
            fn fun_021b8c50(&mut self, pos: u32, a: u32) -> u32 {
                pos * 100 + a
            }
            fn fun_021b8d38(&mut self, _: u32, _: u32, _: &mut [u8; 8]) -> u32 {
                0
            }
            fn fun_021b8778(&mut self, _: u32, _: &mut [u8; 8]) -> u32 {
                0
            }
            fn fun_021c82b8(&mut self, _: u32) -> u32 {
                0
            }
        }
        let mon = Mon {
            poke_id: 2,
            t1: 0,
            t2: 0,
        };
        let mut c = calc_with_seed(1);
        // Class 1 returns the looked-up position itself.
        assert_eq!(fun_021d80bc(&mut Fixed, &mut c, 0, &mon, 33), 12);
        assert_eq!(c.rng.x, 1);
    }

    #[test]
    fn id_set_hand_checked() {
        let mut s = IdSet::new();
        assert_eq!(s.count(), 0);
        s.add(1);
        s.add(1);
        s.add(2);
        s.add(4);
        assert_eq!(s.count(), 2);
        // Client 1: id 1 matches itself, id 2 does not (partner of 1 is 3).
        assert_eq!(s.classify(1, 0), 3);
        assert_eq!(s.classify(1, 2), 2);
        assert_eq!(s.classify(0, 0), 3);
        assert_eq!(IdSet::new().classify(0, 0), 4);
        assert!(is_partner_or_self(3, 1));
        assert!(!is_partner_or_self(0, 1));
    }

    #[test]
    fn target_matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut env = Env::default();
        for seed in 0..400u32 {
            let mut r = Rng::new(seed);
            let hi = r.next() as u64;
            let lo = r.next() as u64;
            let mut c = calc_with_seed(hi << 32 | lo);
            let mon = Mon {
                poke_id: (r.next() % 24) as u8,
                t1: (r.next() % 18) as u8,
                t2: (r.next() % 18) as u8,
            };
            for _ in 0..20 {
                let op = r.next() % 4;
                let a = r.next();
                let b = r.next();
                env.rule = r.next() % 3;
                let arg1 = a % 8;
                let waza = if b.is_multiple_of(6) {
                    MOVE_CURSE
                } else {
                    (b >> 8) % 300
                };
                let mut saved = MathRandContext32::default();
                let v = match op {
                    0 | 1 => fun_021d80bc(&mut env, &mut c, arg1, &mon, waza),
                    2 => fun_021d85c0(&mut env, &mut c, a % 120) as u32,
                    _ => fun_021d81f8(&mut env, &mut c, arg1, &mon, waza, &mut saved),
                };
                h.w(v);
                h.w(c.rng.x as u32);
                h.w((c.rng.x >> 32) as u32);
                if op == 3 {
                    saved.to_bytes().iter().for_each(|&x| h.b(x));
                }
                for e in env.log.drain(..) {
                    e.iter().for_each(|&x| h.w(x));
                }
            }
        }
        assert_eq!(h.hex(), "05abba97e7169052");
    }

    #[test]
    fn speed_matches_rom_fuzz() {
        use crate::battle_mon::tests::{random_bpp, MockEnv};
        let mut h = Fnv::new();
        let mut env = MockEnv::default();
        for seed in 0..500u32 {
            let mut r = Rng::new(0x5D00 + seed);
            let b = random_bpp(&mut r);
            env.field = r.next() & 1 != 0;
            h.w(speed_unstaged(&b, &mut env));
            for e in env.log.drain(..) {
                e.iter().for_each(|&x| h.w(x));
            }
        }
        assert_eq!(h.hex(), "c5335a29b8522abe");
    }

    #[test]
    fn id_set_matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut r = Rng::new(0x1D5E);
        for _ in 0..3000 {
            let mut s = IdSet::new();
            for _ in 0..(r.next() % 7) {
                let id = r.next();
                s.add(if r.next().is_multiple_of(4) {
                    id as u8
                } else {
                    (id % 5) as u8
                });
            }
            h.w(s.count());
            for id in s.ids {
                h.b(id);
            }
            let client = r.next() % 6;
            let lim = r.next() % 4;
            h.w(s.classify(client, lim));
            h.w(is_partner_or_self(r.next() % 6, client) as u32);
            let (a, b) = (r.next(), r.next());
            h.w(is_partner_or_self(a, b) as u32);
            h.w(is_partner_or_self(a & 3, b) as u32);
        }
        assert_eq!(h.hex(), "25f0f8cc438bde20");
    }
}
