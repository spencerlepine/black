//! The last unlabeled `asm/battle_calc.s` routines: `0x021D7E10`, `0x021D7F88`, `0x021D7FBC`,
//! `0x021D8058`, plus the literal-pool thunks (`0x021D7E34`, `0x021D7E38`, `0x021D805C`).
//!
//! `0x021D7D6C` and `0x021D7E90` live in `handlers::moves` (`fun_021d7d6c`, `fun_021d7e90`) and
//! are verified here against the original. The thunks are one `ldr`/`bx` into ARM9 and stay host
//! stubs: `0x021D7E34` is `FlowHost::sick_list_entry`, `0x021D7E38` (`thunk_EXT_FUN_0689c9e4`)
//! is the same shape, `0x021D805C` is `FlowHost::item_param`.

/// C: `FUN_overlay_d_93__021d7d6c`: whether the sick id is below 6 (signed); same as
/// `handlers::moves::fun_021d7d6c`.
pub fn sick_below_6(sick: i32) -> bool {
    sick < 6
}

/// ARM9 routines used by the shop-style bonus sum of [`fun_021d7f88`].
pub trait ShopBonusEnv {
    /// C: `FUN_0201AA18` (`item`): only the low 8 bits are used.
    fn fun_0201aa18(&mut self, item: u32) -> u32;
    /// C: `FUN_0201AB28` (`item, index`).
    fn fun_0201ab28(&mut self, item: u32, index: u32) -> u32;
    /// C: `FUN_0202A230` (`value, kind`).
    fn fun_0202a230(&mut self, value: u32, kind: u32) -> u32;
    /// C: `sub_02017E1C(handle, id, arg)` (`party_get_value`).
    fn party_get_value(&mut self, handle: u32, id: u32, arg: u32) -> u32;
    /// Reads the word at a pointer the routine dereferences.
    fn read32(&mut self, ptr: u32) -> u32;
}

/// The single-instruction ARM9 thunks (indirect calls through a literal pool). Their targets
/// are not ported; `FlowHost` carries the same two as `sick_list_entry` and `item_param`.
pub trait ArmThunks {
    /// C: `FUN_021D7E34` (`index`) / `FUN_overlay_d_93__021d7e34`: entry `index` of the 0-terminated
    /// sick-condition table (tail call through `DAT_021d7e40`).
    fn fun_021d7e34(&mut self, index: u32) -> u32;
    /// C: `thunk_EXT_FUN_0689c9e4` / `FUN_overlay_d_93__021d7e38`: same shape as `021d7e34`
    /// (target `0x0689C9E4`, an ARM overlay routine).
    fn thunk_ext_fun_0689c9e4(&mut self, index: u32) -> u32;
    /// C: `FUN_overlay_d_93__021d805c` (`item, param`): tail-calls `FUN_02020FE4(item, param,
    /// battle_calc.unk_00)`.
    fn fun_021d805c(&mut self, item: u32, param: u32) -> u32;
}

/// C: `FUN_overlay_d_93__021d7e10` (`bpp`): the first entry of the sick-condition table
/// (`entry(i)`, 0-terminated) for which `has_sick(entry)` holds, or 0.
pub fn first_table_sick(
    mut entry: impl FnMut(u32) -> u32,
    mut has_sick: impl FnMut(u32) -> bool,
) -> u32 {
    let mut i = 0;
    loop {
        let e = entry(i);
        i += 1;
        if e == 0 {
            return 0;
        }
        if has_sick(e) {
            return e;
        }
    }
}

/// C: `FUN_overlay_d_93__021d7fbc` (`ptr, item`): 0 without an item or when `FUN_0201AA18` gives 0
/// (low byte); otherwise `4 * party_get_value(FUN_0201AB28(item, n - 1), 0x9e, 0) *
/// FUN_0202A230(*ptr, 10)`.
pub fn fun_021d7fbc<E: ShopBonusEnv>(env: &mut E, ptr: u32, item: u32) -> u32 {
    if item == 0 {
        return 0;
    }
    let n = env.fun_0201aa18(item) & 0xFF;
    if n == 0 {
        return 0;
    }
    let h = env.fun_0201ab28(item, n - 1);
    let v = env.read32(ptr);
    let m = env.fun_0202a230(v, 10);
    let g = env.party_get_value(h, 0x9E, 0);
    g.wrapping_mul(m).wrapping_shl(2)
}

/// C: `FUN_overlay_d_93__021d7f88` (`p`): when `p[0] == 1`, the sum of [`fun_021d7fbc`] for the
/// (pointer `p[0x13]`, item `p[10]`) and (pointer `p[0x15]`, item `p[0xc]`) pairs whose two
/// words are both nonzero.
pub fn fun_021d7f88<E: ShopBonusEnv>(env: &mut E, p: &[u32]) -> u32 {
    let mut sum = 0u32;
    if p[0] == 1 {
        if p[0x13] != 0 && p[10] != 0 {
            sum = fun_021d7fbc(env, p[0x13], p[10]);
        }
        if p[0x15] != 0 && p[0xc] != 0 {
            sum = sum.wrapping_add(fun_021d7fbc(env, p[0x15], p[0xc]));
        }
    }
    sum
}

/// C: `FUN_overlay_d_93__021d8058`: empty function.
pub fn fun_021d8058() {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::moves::{fun_021d7d6c, fun_021d7e90};
    use crate::testutil::{Fnv, Rng};

    fn mix(a: u32, b: u32) -> u32 {
        let x = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
        x ^ (x >> 15)
    }

    struct Env<'a> {
        mem: &'a [(u32, u32)],
        log: Vec<[u32; 4]>,
    }

    impl ShopBonusEnv for Env<'_> {
        fn fun_0201aa18(&mut self, item: u32) -> u32 {
            self.log.push([1, item, 0, 0]);
            mix(item, 0xAA)
        }
        fn fun_0201ab28(&mut self, item: u32, index: u32) -> u32 {
            self.log.push([2, item, index, 0]);
            mix(item.wrapping_add(index), 0xAB)
        }
        fn fun_0202a230(&mut self, value: u32, kind: u32) -> u32 {
            self.log.push([3, value, kind, 0]);
            mix(value, kind) % 1000
        }
        fn party_get_value(&mut self, handle: u32, id: u32, arg: u32) -> u32 {
            self.log.push([4, handle, id, arg]);
            mix(handle, id) % 500
        }
        fn read32(&mut self, ptr: u32) -> u32 {
            self.mem.iter().find(|m| m.0 == ptr).map_or(0, |m| m.1)
        }
    }

    /// Mirrors `tools/rom-diff/calc_leftovers.py leftovers2`.
    #[test]
    fn leftovers2_matches_rom_fuzz() {
        let mut h = Fnv::new();
        let mut r = Rng::new(0xB1B2);
        for _ in 0..3000 {
            let v = r.next();
            let sv = if r.next().is_multiple_of(2) {
                v as i32
            } else {
                (v % 12) as i32 - 3
            };
            assert_eq!(sick_below_6(sv), fun_021d7d6c(sv));
            h.w(sick_below_6(sv) as u32);
            let count = r.next() % 8;
            let list: Vec<u16> = (0..8).map(|_| (r.next() % 12) as u16).collect();
            let value = r.next() % 12;
            h.w(fun_021d7e90(&list, count, value) as u32);

            let s = r.next();
            let s2 = r.next();
            let log = std::cell::RefCell::new(Vec::new());
            let got = first_table_sick(
                |i| {
                    log.borrow_mut().push([10, i, 0, 0]);
                    if mix(i, s).is_multiple_of(4) {
                        0
                    } else {
                        mix(i, s) % 40 + 1
                    }
                },
                |e| {
                    log.borrow_mut().push([11, e, 0, 0]);
                    mix(e, s2) & 1 != 0
                },
            );
            h.w(got);
            for ent in log.into_inner() {
                for val in ent {
                    h.w(val);
                }
            }

            let mut p = [0u32; 24];
            for w in p.iter_mut() {
                *w = r.next() % 5;
            }
            p[0] = r.next() % 3;
            let (pa, pb) = (0x100u32, 0x104u32);
            p[0x13] = if !r.next().is_multiple_of(4) { pa } else { 0 };
            p[0x15] = if !r.next().is_multiple_of(4) { pb } else { 0 };
            let mem = [(pa, r.next()), (pb, r.next())];
            let mut env = Env {
                mem: &mem,
                log: Vec::new(),
            };
            h.w(fun_021d7f88(&mut env, &p));
            for ent in &env.log {
                for val in ent {
                    h.w(*val);
                }
            }
        }
        assert_eq!(h.hex(), "f4fdb368622b0a09");
    }
}
