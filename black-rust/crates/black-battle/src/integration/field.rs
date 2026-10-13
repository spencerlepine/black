//! `btl_field.c` (0x021EF71C-0x021EFD00): [`crate::field::Field`] instances by address. The
//! server's field is the global struct at 0x021F63F4 (the wrappers 021ef874..021ef948 pass it);
//! 021ef958 allocates further instances (the main module's `M + 0x2BC`).

use super::{val, Env};
use crate::event_factor::FactorId;
use crate::field::{Field, FieldHost};
use crate::server_flow::env::{Arg, Lent};
use crate::sick_cont::SickCont;

/// The server's field struct.
pub const FIELD_GLOBAL: u32 = 0x021F_63F4;
/// Address of event factor 0 (the pool at 0x021F4470, 0x3C bytes each).
pub const FACTOR_POOL_ADDR: u32 = 0x021F_4470;

/// The original pointer of a factor.
pub fn factor_addr(f: Option<FactorId>) -> u32 {
    match f {
        Some(f) => FACTOR_POOL_ADDR + f as u32 * crate::event_factor::FACTOR_SIZE as u32,
        None => 0,
    }
}

/// The factor of an original pointer.
pub fn factor_of(addr: u32) -> Option<FactorId> {
    if addr == 0 {
        return None;
    }
    let o = addr - FACTOR_POOL_ADDR;
    let size = crate::event_factor::FACTOR_SIZE as u32;
    assert!(
        o.is_multiple_of(size) && o / size < crate::event_factor::FACTOR_POOL as u32,
        "bad factor pointer {addr:#x}"
    );
    Some((o / size) as FactorId)
}

/// [`FieldHost`] over the environment.
struct Host<'a>(&'a mut Env);

impl FieldHost for Host<'_> {
    /// `06898020(slot, 0)`: registers field effect `slot` (overlay-95 registry 0x0689D650).
    fn effect_create(&mut self, slot: u32) -> u32 {
        super::ov95::field_effect_add(self.0, slot)
    }
    fn effect_destroy(&mut self, factor: u32) {
        let f = factor_of(factor).expect("null factor");
        self.0.events_mut().remove(f);
    }
    fn poke_knows_move(&mut self, poke_id: u32, mv: u32) -> bool {
        let mon = self.0.mon_addr_by_id(poke_id);
        self.0
            .mons
            .get(&mon)
            .expect("battle Pokemon")
            .knows_move(mv)
    }
    fn fun_02045cac(&mut self, _a: u32, _b: u32, _c: u32) {}
    fn fun_0201efa4(&mut self, _a: u32, _i: u32, _v: u32, _m: u32, _x: u32, _y: u32) {}
    fn fun_0201f250(&mut self, _a: u32, _b: u32, _c: u32) {}
}

/// Runs `f` on the field at `addr` (taken out of the table while it runs).
fn with_field<R>(env: &mut Env, addr: u32, f: impl FnOnce(&mut Env, &mut Field) -> R) -> R {
    let mut fld = env
        .fields
        .borrow_mut()
        .remove(&addr)
        .unwrap_or_else(|| panic!("B16: no battle field at {addr:#x} (or it is in use)"));
    let r = f(env, &mut fld);
    env.fields.borrow_mut().insert(addr, fld);
    r
}

pub fn call(env: &mut Env, _lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    // struct-taking entries get the field in r0; the wrappers use the global one
    let (fa, a) = match addr {
        0x021E_F874..=0x021E_F957 => (FIELD_GLOBAL, 0),
        0x021E_F988..=0x021E_FCFF => (val(args, 0), 1),
        _ => (0, 0),
    };
    let arg = |i: usize| val(args, a + i);
    Some(match addr {
        0x021E_F958 => {
            let p = env
                .objs
                .borrow_mut()
                .alloc(crate::field::SIZE as u32, "battle field");
            env.fields.borrow_mut().insert(p, Field::new());
            p
        }
        0x021E_F980 => {
            let p = val(args, 0);
            env.fields
                .borrow_mut()
                .remove(&p)
                .expect("free of an unknown field");
            env.objs.borrow_mut().free(p);
            0
        }
        0x021E_F874 | 0x021E_F988 => {
            let w = arg(0);
            with_field(env, fa, |_, f| f.init(w));
            0
        }
        0x021E_F884 | 0x021E_F9A8 => with_field(env, fa, |_, f| f.weather()),
        0x021E_F894 | 0x021E_F9B0 => with_field(env, fa, |_, f| f.weather_turns()),
        0x021E_F8A4 | 0x021E_F9C0 => {
            let (k, t) = (arg(0), arg(1));
            with_field(env, fa, |_, f| f.set_weather(k, t));
            0
        }
        0x021E_F9C8 => {
            with_field(env, fa, |_, f| f.clear_weather());
            0
        }
        0x021E_F8B8 | 0x021E_F9D0 => with_field(env, fa, |_, f| f.tick_weather()),
        0x021E_F948 | 0x021E_FCE4 => {
            let s = arg(0) as usize;
            with_field(env, fa, |_, f| f.is_active(s))
        }
        // 021ef934(cb, ctx) / 021efc78(field, cb, ctx): ticks the turn-limited effects; the
        // callback (Thumb code) runs for each ended slot. The calls are made after the walk
        // (the field is in use during it): the same order unless a callback changes later slots.
        0x021E_F934 | 0x021E_FC78 => {
            let (cb, ctx) = (arg(0), arg(1));
            let mut ended = Vec::new();
            with_field(env, fa, |e, f| {
                let mut rec = |slot: u32, _ctx: u32| ended.push(slot);
                f.tick_effects(&mut Host(e), Some(&mut rec), ctx)
            });
            if cb != 0 {
                for slot in ended {
                    crate::server_flow::env::FlowEnv::call(
                        env,
                        cb & !1,
                        &mut [Arg::V(slot), Arg::V(ctx)],
                    );
                }
            }
            0
        }
        0x021E_F8C8 | 0x021E_F9F4 => {
            let (slot, cont) = (arg(0) as usize, SickCont(arg(1)));
            let with = if addr == 0x021E_F8C8 {
                true
            } else {
                arg(2) != 0
            };
            with_field(env, fa, |e, f| f.add_effect(&mut Host(e), slot, cont, with)) as u32
        }
        0x021E_F8E8 | 0x021E_FA94 => {
            let slot = arg(0) as usize;
            with_field(env, fa, |e, f| f.remove_effect(&mut Host(e), slot)) as u32
        }
        0x021E_F8F8 | 0x021E_FAD8 => {
            let (slot, id) = (arg(0) as usize, arg(1));
            with_field(env, fa, |_, f| f.add_poke(slot, id)) as u32
        }
        0x021E_F90C | 0x021E_FB34 => {
            let id = arg(0);
            with_field(env, fa, |e, f| f.remove_poke(&mut Host(e), id));
            0
        }
        _ => return None,
    })
}
