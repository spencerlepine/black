//! The event-factor system (`battle_calc.s` 0x021D6EAC-0x021D74C7, [`EventSystem`] in
//! [`Env::events`]) and the handler dispatch: `021d70e4` / `021d70f0` / `021d70fc` run the ported
//! ability / item / move / field-effect handlers on a [`Battle`] whose host is the production
//! [`FlowWorkHost`] over the flow work (taken from the object table for the dispatch), with the
//! environment as its `FlowEnv` and [`Unp`] as its `Unported`.
//!
//! Nested dispatch (a handler's host call reaching `021d70f0` again) runs on a snapshot of the
//! event system that is merged back when the handler returns (the outer loop holds the system);
//! a handler that also changed the system itself around such a call is a reported conflict.

use super::data::Unp;
use super::field::{factor_addr, factor_of};
use super::{val, Env};
use crate::battle_mon::BattleMon;
use crate::event_factor::{EventHost, EventSystem, FactorId};
use crate::flow::{Battle, Handler, SkipCheck};
use crate::handlers;
use crate::server_flow::env::{Arg, Lent};
use crate::server_flow::host::FlowWorkHost;
use crate::server_flow::SvFlowWork;

type Host<'a> = FlowWorkHost<&'a mut Env, Unp>;

/// The handler of a table entry.
fn handler<'a>(addr: u32) -> Option<Handler<Host<'a>>> {
    handlers::abilities::handler::<Host<'a>>(addr)
        .or_else(|| handlers::items::handler::<Host<'a>>(addr))
        .or_else(|| handlers::moves::handler::<Host<'a>>(addr))
        .or_else(|| handlers::effects::handler::<Host<'a>>(addr))
}

/// The skip-check callbacks the handlers install.
fn skip_check<'a>(addr: u32) -> Option<SkipCheck<Host<'a>>> {
    Some(match addr & !1 {
        0x021D_AEC4 => handlers::abilities::skip_021daec4::<Host<'a>>,
        0x021D_B1A8 => handlers::abilities::skip_021db1a8::<Host<'a>>,
        0x021D_B650 => handlers::abilities::skip_021db650::<Host<'a>>,
        _ => return None,
    })
}

/// The dispatch's [`EventHost`].
struct Disp<'a> {
    battle: Battle<Host<'a>>,
}

impl<'a> EventHost for Disp<'a> {
    fn handler_entry(&self, table: u32, i: u32) -> (u32, u32) {
        let rom = &self.battle.host.env.rom;
        (rom.r32(table + 8 * i), rom.r32(table + 8 * i + 4))
    }

    fn call_handler(
        &mut self,
        sys: &mut EventSystem,
        h: u32,
        factor: FactorId,
        _ctx: u32,
        owner: u8,
    ) {
        let Some(f) = handler::<'a>(h & !1) else {
            panic!(
                "B16 gap: event handler {:#010x} is not ported ({})",
                h & !1,
                self.battle.host.env.backtrace()
            );
        };
        // a nested dispatch inside the handler works on this snapshot (see the module doc)
        let before = sys.clone();
        let saved = self.battle.host.env.events_shadow.replace(sys.clone());
        self.battle.host.env.handler_stack.push(h & !1);
        f(sys, factor, &mut self.battle, owner);
        self.battle.host.env.handler_stack.pop();
        let shadow = std::mem::replace(&mut self.battle.host.env.events_shadow, saved).unwrap();
        if shadow != before {
            assert!(
                *sys == before,
                "B16 gap: handler {h:#x} changed the event system around a nested dispatch"
            );
            *sys = shadow;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn call_skip_check(
        &mut self,
        sys: &mut EventSystem,
        cb: u32,
        factor: FactorId,
        _ctx: u32,
        ftype: u32,
        event: u32,
        sub_id: u16,
        owner: u8,
    ) -> bool {
        let Some(f) = skip_check::<'a>(cb) else {
            panic!("B16 gap: skip-check callback {cb:#010x} is not ported");
        };
        f(sys, factor, &mut self.battle, ftype, event, sub_id, owner)
    }

    fn get_mon(&mut self, _ctx: u32, poke_id: u32) -> Option<&BattleMon> {
        let a = self.battle.host.env.mon_addr_by_id(poke_id);
        self.battle.host.env.mons.get(&a)
    }

    fn field_effect_active(&mut self, id: u32) -> bool {
        self.battle.host.env.fields.borrow()[&super::field::FIELD_GLOBAL].is_active(id as usize)
            != 0
    }

    fn fun_021e03e8(&mut self, _sys: &EventSystem, _factor: FactorId) -> bool {
        panic!("B16 gap: 021e03e8 (event_factor callee) is not wired")
    }
}

/// Runs a dispatch on the event system (the real one, or a nested dispatch's snapshot).
fn dispatch(env: &mut Env, event: u32, ftype: Option<u32>, check: bool) {
    let (mut sys, shadow) = match env.events.take() {
        Some(s) => (s, false),
        None => (
            env.events_shadow
                .take()
                .unwrap_or_else(|| panic!("B16: no event system ({})", env.backtrace())),
            true,
        ),
    };
    let wa = env.waddr;
    let v = env.objs.borrow_mut().take(wa);
    let work = SvFlowWork {
        mem: v.into_boxed_slice().try_into().unwrap(),
    };
    let side = std::mem::take(&mut env.side_effects);
    let unp = env.unp();
    let mut host = FlowWorkHost::new(work, wa, &mut *env, unp);
    host.side_effects = side;
    let mut d = Disp {
        battle: Battle::new(host),
    };
    match (ftype, check) {
        (None, false) => sys.dispatch_all(&mut d, wa, event),
        (None, true) => sys.dispatch_all_checked(&mut d, wa, event),
        (Some(t), _) => sys.dispatch_type(&mut d, wa, event, t),
    }
    let Disp { battle } = d;
    let host = battle.host;
    let side = host.side_effects;
    let work = host.work;
    let env = host.env;
    env.side_effects = side;
    let v: Box<[u8]> = work.mem;
    env.objs.borrow_mut().put(wa, v.into_vec());
    if shadow {
        env.events_shadow = Some(sys);
    } else {
        env.events = Some(sys);
    }
}

pub fn call(env: &mut Env, _lent: &mut Lent<'_>, addr: u32, args: &mut [Arg<'_>]) -> Option<u32> {
    let v = |i: usize| match args.get(i) {
        Some(Arg::V(x)) | Some(Arg::PAt(x)) => *x,
        _ => 0,
    };
    let _ = val;
    Some(match addr {
        0x021D_6EAC => {
            let mut sys = env.events.take().expect("event system");
            sys.init(&mut env.vars);
            env.events = Some(sys);
            0
        }
        0x021D_6EE8 => {
            env.events_mut().enable_all_moves();
            0
        }
        0x021D_6F0C => {
            // (type, sub id, sub priority, priority, owner, table, count)
            let f =
                env.events_mut()
                    .add(v(0), v(1) as u16, v(2), v(3), v(4) as u8, v(5), v(6) as u16);
            factor_addr(f)
        }
        0x021D_7040 => {
            let f = factor_of(v(0)).expect("null factor");
            env.events_mut().remove(f);
            0
        }
        0x021D_701C => {
            env.events_mut().remove_type6();
            0
        }
        0x021D_70E4 => {
            dispatch(env, v(1), None, false);
            0
        }
        0x021D_70F0 => {
            dispatch(env, v(1), None, true);
            0
        }
        0x021D_70FC => {
            dispatch(env, v(1), Some(v(2)), true);
            0
        }
        0x021D_732C => factor_addr(env.events_mut().find(v(0), v(1))),
        0x021D_7360 => {
            let f = factor_of(v(0)).expect("null factor");
            factor_addr(env.events_mut().find_next(f))
        }
        0x021D_70A8 => env.events_mut().sub_id(factor_of(v(0)).unwrap()) as u32,
        0x021D_70AC => env.events_mut().poke_id(factor_of(v(0)).unwrap()) as u32,
        0x021D_70B4 => env.events_mut().work(factor_of(v(0)).unwrap(), v(1)),
        0x021D_70DC => {
            env.events_mut()
                .set_work(factor_of(v(0)).unwrap(), v(1), v(2));
            0
        }
        0x021D_7404 => {
            env.events_mut().disable(v(0), v(1));
            0
        }
        0x021D_7438 => env.events_mut().enable(v(0), v(1)) as u32,
        0x021D_74BC | 0x021D_7474 | 0x021D_7494 => {
            panic!("B16 gap: event factor internal {addr:#x} called directly")
        }
        _ => return None,
    })
}
