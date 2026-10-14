//! `battle_calc.s` 0x021D74C8-0x021D85B0: event variables (global 0x021F3ECC, [`Env::vars`]),
//! the battle calc / RNG (0x021F6360, [`Env::calc`]), action records, id sets and the ability
//! registration. Adapters to the ports.

use super::{val, Bufs, Env};
use crate::battle_mon::BattleMon;
use crate::calc::{self, MathRandContext32};
use crate::calc_action::ActionRecord;
use crate::calc_target::IdSet;
use crate::hooks;
use crate::server_flow::env::{Arg, FlowEnv, Lent};
use crate::sick_cont::SickCont;

/// A battle Pokemon seen through [`hooks::BattleMon`] (the calc ports' view): `BattleMon_Get` with
/// the integration's party-Pokemon environment.
pub struct MonView {
    pub mon: BattleMon,
    pub unp: std::cell::RefCell<super::data::Unp>,
}

impl hooks::BattleMon for MonView {
    fn get(&self, id: u32) -> u32 {
        self.mon.get(id, &mut *self.unp.borrow_mut())
    }
    fn has_type(&self, ty: u32) -> bool {
        self.mon.has_type(ty)
    }
    fn poke_id(&self) -> u8 {
        self.mon.poke_id()
    }
    fn species(&self) -> u16 {
        self.mon.species()
    }
}

/// A pokecon party as [`hooks::BattleParty`] (its members resolved by the caller).
struct Members(Vec<MonView>);

impl hooks::BattleParty for Members {
    type Mon = MonView;
    fn count(&self) -> u32 {
        self.0.len() as u32
    }
    fn member(&self, index: u8) -> &MonView {
        &self.0[index as usize]
    }
}

impl Env {
    /// The [`MonView`] of the battle Pokemon at `addr`.
    pub fn mon_view(&self, addr: u32) -> MonView {
        let mon = self
            .mons
            .get(&addr)
            .unwrap_or_else(|| panic!("B16: no battle Pokemon at {addr:#x} ({})", self.backtrace()))
            .clone();
        MonView {
            mon,
            unp: std::cell::RefCell::new(self.unp()),
        }
    }
}

/// An action record argument (`P` = a slice the caller lent through `PBuf`, or an address).
fn with_record<R>(
    env: &mut Env,
    lent: &mut Lent<'_>,
    args: &mut [Arg<'_>],
    i: usize,
    f: impl FnOnce(&mut ActionRecord) -> R,
) -> R {
    Bufs::new(args).with(env, lent, i, 4, |_, b| {
        let mut r = ActionRecord(u32::from_le_bytes(b[..4].try_into().unwrap()));
        let out = f(&mut r);
        b[..4].copy_from_slice(&r.0.to_le_bytes());
        out
    })
}

fn with_idset<R>(
    env: &mut Env,
    lent: &mut Lent<'_>,
    args: &mut [Arg<'_>],
    f: impl FnOnce(&mut IdSet) -> R,
) -> R {
    Bufs::new(args).with(env, lent, 0, 8, |_, b| {
        let mut s = IdSet::from_bytes(b);
        let out = f(&mut s);
        b[..8].copy_from_slice(&s.to_bytes());
        out
    })
}

pub fn call(env: &mut Env, lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    if addr == 0x021D_80BC || addr == 0x021D_81F8 {
        return target_call(env, lent, addr, args);
    }
    if let Some(r) = registry_call(env, addr, args) {
        return Some(r);
    }
    let v = |i: usize| val(args, i);
    Some(match addr {
        // ---- event variables (EventVarStack) ----
        0x021D_74C8 => {
            crate::event_factor::reset_event_vars(&mut env.vars);
            0
        }
        0x021D_74E4 => {
            crate::event_factor::reset_event_vars_if_used(&mut env.vars);
            0
        }
        0x021D_74F8 => {
            if env.vars.push().is_err() {
                panic!("B16: event-variable stack overflow ({})", env.backtrace());
            }
            0
        }
        0x021D_7540 => {
            env.vars.pop();
            0
        }
        0x021D_75A8 => {
            env.vars.set_value(v(0) as u16, v(1) as i32);
            0
        }
        0x021D_75EC => {
            env.vars.set_const_value(v(0) as u16, v(1) as i32);
            0
        }
        0x021D_7630 => {
            env.vars.set_rewrite_once_value(v(0) as u16, v(1) as i32);
            0
        }
        0x021D_7674 => {
            env.vars
                .set_mul_value(v(0) as u16, v(1) as i32, v(2) as i32, v(3) as i32);
            0
        }
        0x021D_76BC => env.vars.rewrite_value(v(0) as u16, v(1) as i32) as u32,
        0x021D_7704 => {
            env.vars.mul_value(v(0) as u16, v(1) as i32);
            0
        }
        0x021D_775C => env.vars.get_value(v(0) as u16) as u32,
        // ---- calc ----
        0x021D_7840 => {
            let p = match args[0] {
                Arg::P => env.main_module + 0x3F0,
                _ => v(0),
            };
            let mut b = [0u8; 24];
            env.objs.borrow_mut().read(p, &mut b);
            let rng = MathRandContext32::from_bytes(&b);
            let mut heap = std::mem::take(&mut env.calc_heap);
            env.calc
                .init(&rng, &mut heap, black_core::heap::HeapId::Unknown);
            env.calc_heap = heap;
            0
        }
        0x021D_787C => {
            let mut b = [0u8; 24];
            env.objs.borrow_mut().read(v(0), &mut b);
            env.calc.set_rng(&MathRandContext32::from_bytes(&b));
            0
        }
        0x021D_7894 => {
            let mut heap = std::mem::take(&mut env.calc_heap);
            env.calc.free(&mut heap);
            env.calc_heap = heap;
            0
        }
        0x021D_804C => {
            env.calc.set_unk_00(v(0) as u16);
            0
        }
        0x021D_8058 => 0,
        // 021d7f88(setup): prize-money bonus of trainer battles (ShopBonusEnv not modelled: a
        // wild battle (type 0) never reaches it)
        0x021D_7F88 => {
            let p = match args[0] {
                Arg::P => env.setup,
                _ => v(0),
            };
            let words: Vec<u32> = (0..22).map(|i| env.r32(p + 4 * i)).collect();
            crate::calc_leftovers2::fun_021d7f88(&mut NoShop, &words)
        }
        // 021d7ff8(badges, party): prize money the player pays when losing (B19: reached by the
        // B16 battle, which the player loses); the party's members through the server ports
        // `021b9b94` (count) / `021b9c00` (member)
        0x021D_7FF8 => {
            let (badges, party) = (v(0), v(1));
            let n = env.call(0x021B_9B94, &mut [Arg::V(party)]);
            let members: Vec<MonView> = (0..n)
                .map(|i| {
                    let a = env.call(0x021B_9C00, &mut [Arg::V(party), Arg::V(i)]);
                    env.mon_view(a)
                })
                .collect();
            calc::prize_money(badges, &Members(members))
        }
        0x021D_78AC => env.calc.rand(v(0)),
        0x021D_78E4 => calc::apply_stat_stage(v(0), v(1)) as u32,
        0x021D_7908 => calc::apply_accuracy_stage(v(0), v(1)) as u32,
        0x021D_7930 => env.calc.roll_critical(v(0)) as u32,
        0x021D_794C => calc::type_effectiveness(v(0) as u8, v(1) as u8) as u32,
        0x021D_799C => calc::type_effectiveness_vs_pair(v(0) as u8, v(1) as u16) as u32,
        0x021D_79E8 => calc::combine_effectiveness(v(0) as u8, v(1) as u8) as u32,
        0x021D_7A44 => calc::calc_base_damage(v(0), v(1), v(2), v(3)),
        0x021D_7A6C => calc::apply_type_effectiveness(v(0), v(1)),
        0x021D_7AD4 => calc::effectiveness_class(v(0) as i32) as u32,
        0x021D_7AF0 => env.calc.roll_percent(v(0)) as u32,
        0x021D_7B08 => calc::max_i32(v(0) as i32, v(1) as i32) as u32,
        0x021D_7B10 => calc::fx_mul(v(0), v(1)),
        0x021D_7B2C => calc::fx_mul_min1(v(0), v(1)),
        0x021D_7B3C => calc::percent_mul(v(0), v(1)),
        0x021D_7B60 => calc::max_hp_div(&env.mon_view(v(0)), v(1)),
        0x021D_7B74 => calc::max_hp_div_min1(&env.mon_view(v(0)), v(1)),
        0x021D_7B8C => env.calc.rand_range(v(0), v(1)),
        0x021D_7BA4 => env.calc.roll_hit_count(v(0)),
        0x021D_7BD4 => calc::weather_residual_damage(&env.mon_view(v(0)), v(1)),
        0x021D_7C38 => calc::weather_power_modifier(v(0), v(1)),
        0x021D_7D18 => env.calc.make_default_sick_cont(v(0)).0,
        0x021D_7D6C => crate::calc_leftovers2::sick_below_6(v(0) as i32) as u32,
        0x021D_7EF0 => calc::is_not_0_or_3(v(0)) as u32,
        0x021D_7F00 => calc::fun_021d7f00(v(0)),
        0x021D_7F2C => calc::fun_021d7f2c(v(0), v(1)),
        0x021D_806C => {
            let b = Bufs::new(args).get(env, lent, 0, 6);
            calc::pack_5bit_x6(&b[..6].try_into().unwrap())
        }
        0x021D_8090 => {
            let mut out = [0u8; 6];
            calc::unpack_5bit_x6(v(0), &mut out);
            Bufs::new(args).put(env, lent, 1, &out);
            0
        }
        0x021D_80A8 => calc::fun_021d80a8(&env.mon_view(v(0))),
        0x021D_7C70 => {
            let mon = env.mon_view(v(1));
            let mut out = SickCont(0);
            env.calc.make_move_sick_cont(v(0) as u16, &mon, &mut out);
            Bufs::new(args).put(env, lent, 2, &out.0.to_le_bytes());
            0
        }
        0x021D_7DC0 => {
            let mon = env.mon_view(v(1));
            let c = env.calc.make_sick_cont(v(0), &mon);
            Bufs::new(args).put(env, lent, 2, &c.0.to_le_bytes());
            0
        }
        // ---- id sets (8-byte {count, u8[4]}) ----
        0x021D_8230 => {
            let s = IdSet::new();
            Bufs::new(args).put(env, lent, 0, &s.to_bytes());
            0
        }
        0x021D_8244 => {
            let id = v(1) as u8;
            with_idset(env, lent, args, |s| s.add(id));
            0
        }
        0x021D_826C => with_idset(env, lent, args, |s| s.count()),
        0x021D_8270 => {
            let (c, l) = (v(1), v(2));
            with_idset(env, lent, args, |s| s.classify(c, l))
        }
        // ---- action records (u32 word) ----
        0x021D_8454 => with_record(env, lent, args, 0, |r| r.tag()),
        0x021D_8434 => {
            with_record(env, lent, args, 0, |r| r.set_none());
            0
        }
        0x021D_82D0 => {
            let (w, p) = (v(1), v(2));
            with_record(env, lent, args, 0, |r| r.set_fight(w, p));
            0
        }
        0x021D_82FC => {
            let p = v(1);
            with_record(env, lent, args, 0, |r| r.set_fight_pos(p));
            0
        }
        0x021D_8318 => {
            with_record(env, lent, args, 0, |r| r.set_fight_flag());
            0
        }
        0x021D_832C => with_record(env, lent, args, 0, |r| r.fight_flag()),
        0x021D_8340 => with_record(env, lent, args, 0, |r| r.fight_waza()),
        0x021D_8358 => {
            let (i, p, a) = (v(1), v(2), v(3));
            with_record(env, lent, args, 0, |r| r.set_item(i, p, a));
            0
        }
        0x021D_8394 => {
            let (a, b) = (v(1), v(2));
            with_record(env, lent, args, 0, |r| r.set_switch(a, b));
            0
        }
        0x021D_83C4 => {
            with_record(env, lent, args, 0, |r| r.set_switch_flagged());
            0
        }
        0x021D_83E0 => with_record(env, lent, args, 0, |r| r.is_switch_flagged() as u32),
        0x021D_83FC => {
            let x = v(1);
            with_record(env, lent, args, 0, |r| r.set_tag6(x));
            0
        }
        0x021D_8414 => {
            with_record(env, lent, args, 0, |r| r.set_tag4());
            0
        }
        0x021D_8424 => {
            with_record(env, lent, args, 0, |r| r.set_tag5());
            0
        }
        0x021D_8444 => {
            with_record(env, lent, args, 0, |r| r.set_tag7());
            0
        }
        _ => return None,
    })
}

/// [`crate::calc_leftovers2::ShopBonusEnv`] for battles that never reach it.
struct NoShop;

impl crate::calc_leftovers2::ShopBonusEnv for NoShop {
    fn fun_0201aa18(&mut self, _item: u32) -> u32 {
        panic!("B16 gap: 0201AA18 (trainer prize data) is not modelled")
    }
    fn fun_0201ab28(&mut self, _item: u32, _index: u32) -> u32 {
        panic!("B16 gap: 0201AB28 is not modelled")
    }
    fn fun_0202a230(&mut self, _value: u32, _kind: u32) -> u32 {
        panic!("B16 gap: 0202A230 is not modelled")
    }
    fn party_get_value(&mut self, _handle: u32, _id: u32, _arg: u32) -> u32 {
        panic!("B16 gap: 021d7fbc party read is not modelled")
    }
    fn read32(&mut self, _ptr: u32) -> u32 {
        panic!("B16 gap: 021d7fbc read is not modelled")
    }
}

/// The ability registry 0x021F0E74 (0x9E `(ability, getter)` entries) read from the ROM.
pub struct RomAbilities<'a>(pub &'a super::rom::Rom);

impl crate::handlers::registry::AbilityRegistry for RomAbilities<'_> {
    fn getter(&self, ability: u32) -> Option<(u32, u32)> {
        let g = (0..0x9E).find_map(|i| {
            (self.0.r32(0x021F_0E74 + 8 * i) == ability)
                .then(|| self.0.r32(0x021F_0E74 + 8 * i + 4))
        })?;
        if let Some(c) = crate::handlers::registry::custom_getter(ability) {
            return Some(c);
        }
        Some(self.0.getter(g))
    }
}

/// `021d8484` / `8510` / `8538` / `8548` / `8564`: ability (de)registration on the event system.
pub fn registry_call(env: &mut Env, addr: u32, args: &[Arg<'_>]) -> Option<u32> {
    use crate::handlers::registry as r;
    let v = |i: usize| match args.get(i) {
        Some(Arg::V(x)) | Some(Arg::PAt(x)) => *x,
        _ => 0,
    };
    if !(0x021D_8484..0x021D_85B0).contains(&addr) {
        return None;
    }
    let rom = env.rom.clone();
    let reg = RomAbilities(&rom);
    let mon = env.mons.get(&v(0)).expect("battle Pokemon").clone();
    let mut unp = env.unp();
    let sys = env.events_mut();
    Some(match addr {
        0x021D_8484 => super::field::factor_addr(r::register_ability(sys, &reg, &mon, &mut unp)),
        0x021D_8510 => {
            r::unregister_ability(sys, &mon);
            0
        }
        0x021D_8538 => {
            r::disable_ability(sys, &mon);
            0
        }
        0x021D_8548 => {
            r::enable_ability(sys, &reg, &mon, &mut unp);
            0
        }
        0x021D_8564 => {
            let b = env.mons.get(&v(1)).expect("battle Pokemon").clone();
            let sys = env.events_mut();
            r::swap_abilities(sys, &reg, &mon, &b, &mut unp);
            0
        }
        _ => return None,
    })
}

/// [`crate::calc_target::TargetEnv`] over the environment: the main-module queries go through
/// `call` with the main module `main` (argument 0 of `021d80bc`).
struct Targets<'a> {
    env: &'a mut Env,
    main: u32,
}

impl Targets<'_> {
    fn c(&mut self, addr: u32, a: &[u32]) -> u32 {
        let mut args: Vec<Arg<'_>> = a.iter().map(|&x| Arg::V(x)).collect();
        crate::server_flow::env::FlowEnv::call(self.env, addr, &mut args)
    }
}

impl crate::calc_target::TargetEnv for Targets<'_> {
    fn rule(&mut self) -> u32 {
        let m = self.main;
        self.c(0x021B_8588, &[m])
    }
    fn fun_021b8d8c(&mut self, arg1: u32, a: u32) -> u32 {
        let m = self.main;
        self.c(0x021B_8D8C, &[m, arg1, a])
    }
    fn waza_param(&mut self, waza: u32, id: u32) -> u32 {
        self.c(0x0201_BD28, &[waza, id])
    }
    fn waza_flag(&mut self, waza: u32, id: u32) -> u32 {
        self.c(0x0201_BEB8, &[waza, id])
    }
    fn fun_021b8c50(&mut self, pos: u32, a: u32) -> u32 {
        let m = self.main;
        self.c(0x021B_8C50, &[m, pos, a])
    }
    fn fun_021b8d38(&mut self, arg1: u32, code: u32, out: &mut [u8; 8]) -> u32 {
        let m = self.main;
        let mut args = [
            Arg::V(m),
            Arg::V(arg1),
            Arg::V(code),
            Arg::Out(&mut out[..]),
        ];
        crate::server_flow::env::FlowEnv::call(self.env, 0x021B_8D38, &mut args)
    }
    fn fun_021b8778(&mut self, code: u32, out: &mut [u8; 8]) -> u32 {
        let m = self.main;
        let mut args = [Arg::V(m), Arg::V(code), Arg::Out(&mut out[..])];
        crate::server_flow::env::FlowEnv::call(self.env, 0x021B_8778, &mut args)
    }
    fn fun_021c82b8(&mut self, a: u32) -> u32 {
        let w = self.env.waddr;
        self.c(0x021C_82B8, &[w, a])
    }
}

/// `021d80bc(main, arg1, mon, waza)` / `021d81f8(.., &saved)`: the target position of a move.
pub fn target_call(
    env: &mut Env,
    lent: &mut Lent<'_>,
    addr: u32,
    args: &mut [Arg<'_>],
) -> Option<u32> {
    let (main, arg1, mon, waza) = (val(args, 0), val(args, 1), val(args, 2), val(args, 3));
    if addr != 0x021D_80BC && addr != 0x021D_81F8 {
        return None;
    }
    let view = env.mon_view(mon);
    let mut c = std::mem::take(&mut env.calc);
    let r = {
        let mut t = Targets {
            env: &mut *env,
            main,
        };
        if addr == 0x021D_80BC {
            crate::calc_target::fun_021d80bc(&mut t, &mut c, arg1, &view, waza)
        } else {
            let mut saved = MathRandContext32::default();
            let r = crate::calc_target::fun_021d81f8(&mut t, &mut c, arg1, &view, waza, &mut saved);
            Bufs::new(args).put(t.env, lent, 4, &saved.to_bytes());
            r
        }
    };
    env.calc = c;
    Some(r)
}
