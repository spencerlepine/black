//! Battle context passed to event handlers (the original's `BTL_SVFLOW_WORK*`).
//!
//! [`Battle`] owns the state handlers read and write directly: the event-variable stack, the
//! battle RNG and the battle Pokemon. Everything else a handler calls in the server-flow layer
//! (asm/battle_server.s, asm/battle_ungrouped.s, not yet ported) goes through [`FlowHost`].

use crate::battle_mon::{BattleMon, BppEnv};
use crate::calc::BattleCalc;
use crate::event_factor::{EventSystem, FactorId};
use crate::event_var::EventVarStack;

/// Number of battle Pokemon IDs (4 clients x 6).
pub const POKE_ID_COUNT: usize = 24;
/// "No Pokemon" ID.
pub const POKE_ID_NONE: u32 = 0x1F;

/// Handle to a handler-effect ("HANDEX") work being filled in.
pub type HandexId = usize;

/// Undecompiled server-flow routines handlers call.
pub trait FlowHost: BppEnv {
    /// C: `FUN_overlay_d_93__021c88a4(wk, type, poke_id)`: pushes a handler-effect work of
    /// `ty` and returns it (zeroed) for the handler to fill.
    fn handex_push(&mut self, ty: u32, poke_id: u32) -> HandexId;
    /// The bytes of a pushed work.
    fn handex(&mut self, id: HandexId) -> &mut [u8];
    /// C: `FUN_overlay_d_93__021c88c8(wk, work)`: finishes a work.
    fn handex_pop(&mut self, id: HandexId);
    /// C: `FUN_overlay_d_93__021c88b4(wk, type, poke_id)`: pushes and immediately pops a work
    /// with no parameters.
    fn handex_push_pop(&mut self, ty: u32, poke_id: u32);
    /// C: `FUN_overlay_d_93__021c7d00(wk, poke_id)`: the Pokemon's battle position.
    fn poke_pos(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c7f68(wk, code, out)`: expands a position code into Pokemon
    /// IDs written to `out`; returns how many (`FUN_overlay_d_93__021c8120` supplies `out`).
    fn expand_poke_ids(&mut self, code: u32, out: &mut [u8]) -> u32;
    /// C: `FUN_overlay_d_93__021c8288` (-> `021c2ab0`): the current weather.
    fn weather(&mut self) -> u32;
    /// C: `FUN_0201C1E8(move)`: the move's damage category (1 = physical, 2 = special).
    fn move_category(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201BD28(move, param)`: move data parameter.
    fn move_param(&mut self, mv: u32, param: u32) -> u32;
    /// C: `FUN_0201BEB8(move, flag)`: move data flag.
    fn move_flag(&mut self, mv: u32, flag: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c82b8(wk, x)` (tail call into the battle main module).
    fn fun_021c82b8(&mut self, x: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c83d4(wk)` (-> `021bc2b0`).
    fn fun_021c83d4(&mut self) -> bool;
    /// C: `FUN_0689C944(ability)` via the ARM thunks at 0x021DA2D0 / 0x021DA2E0: the ability
    /// cannot be copied (Trace).
    fn ability_uncopyable(&mut self, ability: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c815c(wk)`: the battle rule (2 = triple).
    fn rule(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021b9a6c(pos_a, pos_b)`: positions are adjacent (triples).
    fn positions_adjacent(&mut self, a: u32, b: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c7d54(wk, poke_id, out)`: IDs of the Pokemon on the other
    /// side (written to `out`); returns how many.
    fn opponent_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32;
    /// C: `FUN_0201C1D8(move)`: base power (`u8`).
    fn move_power(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201C1BC(move)`: move effect class (9 = one-hit KO).
    fn move_effect_class(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201C314(move)`.
    fn move_fun_0201c314(&mut self, mv: u32) -> bool;
    /// C: `FUN_0201C1C8(move)`: move type.
    fn move_type(&mut self, mv: u32) -> u32;
    /// C: `FUN_0689C9B0(ability)` via the ARM thunk at 0x021DB1C4: Mold Breaker ignores it.
    fn ability_breakable(&mut self, ability: u32) -> bool;
    /// C: `FUN_0689C8EC(move)` via `FUN_021DB550`: the move cannot be redirected.
    fn move_unredirectable(&mut self, mv: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8290(wk, attacker, defender, move)`.
    fn fun_021c8290(&mut self, attacker: u32, defender: u32, mv: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c804c(wk, poke_id, &out)`: finds the Pokemon's queued action;
    /// writes it to `out` and returns whether found.
    fn find_action(&mut self, poke_id: u32, out: &mut u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8010(wk, poke_id)`: the Pokemon's position, or 6 if it is not
    /// in battle.
    fn poke_pos_in_battle(&mut self, poke_id: u32) -> u32;
}

/// The battle state handlers operate on.
pub struct Battle<H: FlowHost> {
    pub vars: EventVarStack,
    pub calc: BattleCalc,
    /// Battle Pokemon by ID.
    pub mons: Vec<BattleMon>,
    /// Stands in for the NULL the original's lookup returns for an unknown ID (reads see
    /// zeros; the original would dereference NULL).
    pub null_mon: BattleMon,
    pub host: H,
}

impl<H: FlowHost> Battle<H> {
    pub fn new(host: H) -> Self {
        Battle {
            vars: EventVarStack::new(),
            calc: BattleCalc::new(),
            mons: vec![BattleMon::default(); POKE_ID_COUNT],
            null_mon: BattleMon::default(),
            host,
        }
    }

    /// C: `FUN_overlay_d_93__021c7cf4(wk, poke_id)`: battle Pokemon by ID.
    pub fn mon(&mut self, poke_id: u32) -> &mut BattleMon {
        match self.mons.get_mut(poke_id as usize) {
            Some(m) => m,
            None => &mut self.null_mon,
        }
    }

    /// `BattleMon_Get` on Pokemon `poke_id`.
    pub fn mon_get(&mut self, poke_id: u32, id: u32) -> u32 {
        let mon = match self.mons.get(poke_id as usize) {
            Some(m) => m,
            None => &self.null_mon,
        };
        mon.get(id, &mut self.host)
    }

    /// C: `FUN_overlay_d_93__021d775c`
    pub fn var(&self, label: u16) -> i32 {
        self.vars.get_value(label)
    }
    /// C: `FUN_overlay_d_93__021d76bc`
    pub fn rewrite_var(&mut self, label: u16, v: i32) -> bool {
        self.vars.rewrite_value(label, v)
    }
    /// C: `FUN_overlay_d_93__021d7704`
    pub fn mul_var(&mut self, label: u16, fx: i32) {
        self.vars.mul_value(label, fx)
    }
}

impl<H: FlowHost> Battle<H> {
    /// C: `FUN_overlay_d_93__021d85c0(wk, percent)`: a `percent`% roll, or always true when
    /// `FUN_overlay_d_93__021c82b8(wk, 1)` holds.
    pub fn roll_percent_or_forced(&mut self, percent: u32) -> bool {
        if self.calc.roll_percent(percent) {
            return true;
        }
        self.host.fun_021c82b8(1)
    }

    /// C: `FUN_overlay_d_93__021e85f8(poke_id)`: `poke_id` is one of the event's targets
    /// (var 5 = count, vars 6.. = IDs).
    pub fn is_event_target(&self, poke_id: u32) -> bool {
        let n = self.var(5) as u32;
        let mut i = 0u32;
        while i < n {
            if poke_id == self.var((i + 6) as u16) as u32 {
                return true;
            }
            i += 1;
        }
        false
    }

    /// C: `FUN_overlay_d_93__021d7dc0(sick, mon, &out)` for Pokemon `poke_id`: the
    /// duration for a newly inflicted condition (sleep and confusion roll the RNG).
    pub fn make_sick_cont(&mut self, sick: u32, poke_id: u32) -> crate::sick_cont::SickCont {
        use crate::sick_cont::SickCont;
        if (sick as i32) < 6 {
            return self.calc.make_default_sick_cont(sick);
        }
        match sick {
            6 => SickCont::turn(self.calc.rand_range(2, 5) as u8 as u32),
            7 => SickCont::poke(self.mon(poke_id).poke_id() as u32),
            _ => SickCont::permanent(),
        }
    }

    /// C: `FUN_overlay_d_93__021d7b74` on Pokemon `poke_id`: max HP / `div`, at least 1.
    pub fn max_hp_div_min1(&mut self, poke_id: u32, div: u32) -> u32 {
        match crate::calc::u32_div(self.mon_get(poke_id, crate::battle_mon::value::MAX_HP), div) {
            0 => 1,
            v => v,
        }
    }
}

/// A battle event handler: C `void handler(factor, flow_work, poke_id, work)`.
pub type Handler<H> = fn(&mut EventSystem, FactorId, &mut Battle<H>, u8);

/// A skip-check callback: C `BOOL cb(factor, flow_work, type, event, sub_id, owner)` for the
/// factor `type`/`sub_id`/`owner` about to run; true skips it.
pub type SkipCheck<H> = fn(&EventSystem, FactorId, &mut Battle<H>, u32, u32, u16, u8) -> bool;

/// Byte-level writes into a handler-effect work.
pub trait HandexBytes {
    fn put_u8(&mut self, o: usize, v: u8);
    fn put_u16(&mut self, o: usize, v: u16);
    fn put_u32(&mut self, o: usize, v: u32);
    fn get_u32(&self, o: usize) -> u32;
}

impl HandexBytes for [u8] {
    fn put_u8(&mut self, o: usize, v: u8) {
        self[o] = v;
    }
    fn put_u16(&mut self, o: usize, v: u16) {
        self[o..o + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn put_u32(&mut self, o: usize, v: u32) {
        self[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn get_u32(&self, o: usize) -> u32 {
        u32::from_le_bytes(self[o..o + 4].try_into().unwrap())
    }
}

/// C: `FUN_overlay_d_93__021c8838(param, type, str_id)`: initialises a string parameter
/// (`+0` str ID, `+2` type in bits 0-7, argument count in bits 8-14, flag in bit 15; `+4`
/// up to eight `u32` arguments) at `buf[o..]`.
pub fn str_param_setup(buf: &mut [u8], o: usize, ty: u32, str_id: u32) {
    let h = u16::from_le_bytes([buf[o + 2], buf[o + 3]]);
    let h = (h & !0xFF) | (ty as u8 as u16);
    buf.put_u16(o + 2, h);
    buf.put_u16(o, str_id as u16);
    buf.put_u16(o + 2, h & 0x80FF);
}

/// C: `FUN_overlay_d_93__021c888c(param, value)`: stores an extra value at `+0x20` and sets
/// flag bit 15 (ignored with eight arguments).
pub fn str_param_set_extra(buf: &mut [u8], o: usize, value: u32) {
    let h = u16::from_le_bytes([buf[o + 2], buf[o + 3]]);
    if ((h >> 8) & 0x7F) < 8 {
        buf.put_u32(o + 0x20, value);
        buf.put_u16(o + 2, h | 0x8000);
    }
}

/// C: `FUN_overlay_d_93__021c8860(param, arg)`: appends an argument (ignored past eight).
pub fn str_param_add_arg(buf: &mut [u8], o: usize, arg: u32) {
    let h = u16::from_le_bytes([buf[o + 2], buf[o + 3]]);
    let n = ((h >> 8) & 0x7F) as usize;
    if n < 8 {
        let h = (h & 0x80FF) | ((((n + 1) as u16) & 0x7F) << 8);
        buf.put_u16(o + 2, h);
        buf.put_u32(o + 4 + 4 * n, arg);
    }
}
