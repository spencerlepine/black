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
    /// C: `FUN_0689C944(ability)` via the ARM thunks at 0x021DA2D0 / 0x021DA2E0 and the tail-call
    /// veneer `sub_0689c93c`: the ability cannot be copied (Trace). The ROM body is
    /// `ability != 0x19 && 0689c91c(ability)` (read with rizin, V-0018), so it is provided on top
    /// of [`Self::fun_0689c91c`] and a host cannot answer the two inconsistently.
    fn ability_uncopyable(&mut self, ability: u32) -> bool {
        ability != 0x19 && self.fun_0689c91c(ability)
    }
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
    /// C: `FUN_overlay_d_93__021c8234(wk, poke_id)`: flow query on a Pokemon (Arena Trap
    /// uses it to decide whether a foe is trapped).
    fn fun_021c8234(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_0689C8DC(move)` via the ARM thunk at 0x021DB99C.
    fn move_fun_0689c8dc(&mut self, mv: u32) -> bool;
    /// C: `FUN_0689C8CC(move)` via the ARM thunks at 0x021DBE50 / 0x021DC6DC.
    fn move_fun_0689c8cc(&mut self, mv: u32) -> bool;
    /// C: `FUN_0689C980(item)` via the ARM thunk at 0x021E867C.
    fn item_fun_0689c980(&mut self, item: u32) -> bool;
    /// C: `FUN_0689C990(item)` via the ARM thunk at 0x021E868C.
    fn item_fun_0689c990(&mut self, item: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c80ac(wk, poke_id)`.
    fn fun_021c80ac(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8168(wk)` (-> `021b86b8`).
    fn fun_021c8168(&mut self) -> bool;
    /// C: `FUN_overlay_d_93__021b8ddc(poke_id)`; callers test for 1.
    fn fun_021b8ddc(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c8434(wk, poke_id, attacker)`: bounces the current move
    /// back (Magic Bounce).
    fn fun_021c8434(&mut self, poke_id: u32, attacker: u32);
    /// C: `FUN_020213c0(item)` (Unnerve and Harvest use it as the berry test).
    fn item_fun_020213c0(&mut self, item: u32) -> bool;
    /// C: `FUN_0201C020(move)`: number of entries `FUN_0201C0BC` can return.
    fn move_fun_0201c020(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201C0BC(move, index, &out)`: entry `index` (signed).
    fn move_fun_0201c0bc(&mut self, mv: u32, index: u32) -> i32;
    /// C: `FUN_overlay_d_93__021b8c5c(rule, pos)`: position paired with `pos` (Imposter).
    fn fun_021b8c5c(&mut self, rule: u32, pos: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c8020(wk, pos)`: ID of the Pokemon at `pos`.
    fn poke_id_at_pos(&mut self, pos: u32) -> u32;
    /// C: `FUN_021c7ec4(wk)`: tests a flow-work field for non-zero (read by several item
    /// handlers).
    fn fun_021c7ec4(&mut self) -> bool;
    /// C: `FUN_overlay_d_93__021c7f24(wk, poke_id)`: flow query on a Pokemon (-> `021b9bc4`).
    fn fun_021c7f24(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c818c(wk, poke_id)`: loops over the Pokemon's species data (-> `0201b718`).
    fn fun_021c818c(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c830c(wk)`: low bit of a flow-work byte.
    fn fun_021c830c(&mut self) -> bool;
    /// C: `FUN_overlay_d_93__021c8374(wk, poke_id)`: flow query on a Pokemon (tests condition
    /// 0x21 and `021c8348`).
    fn fun_021c8374(&mut self, poke_id: u32) -> bool;
    /// C: `sub_021c8408(wk, poke_id)` (-> `021bb914`); no result.
    fn fun_021c8408(&mut self, poke_id: u32);
    /// C: `FUN_overlay_d_93__021d805c(item, param)`: item data parameter (tail call through a
    /// function pointer into the ARM9 item data reader, heap ID from a global).
    fn item_param(&mut self, item: u32, param: u32) -> u32;
    /// C: `FUN_021D7E34(index)` -> `FUN_0689C9E4(index)`: condition ID `index` of a 6-entry
    /// table (overlay 95), 0 past the end.
    fn sick_list_entry(&mut self, index: u32) -> u32;
    /// C: `FUN_020189D0(pokemon, x)`: Pokemon query returning -1 or a small value (berry
    /// flavour dislike check).
    fn fun_020189d0(&mut self, pp: u32, x: u32) -> i32;
    /// C: `FUN_021c7d34(wk, poke_id)` (tail call into the battle main module): the Pokemon's
    /// battle position, 6 = none (guess).
    fn fun_021c7d34(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c7d44(wk)` (tail call): a Pokemon ID, 0x1F = none (guess).
    fn fun_021c7d44(&mut self) -> u32;
    /// C: `FUN_021c7ed8(wk, poke_id)`: small count about the Pokemon's position (guess).
    fn fun_021c7ed8(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c7fcc(wk, poke_id)`: the party of the Pokemon's side, a struct
    /// read with `021b9b94` (count) and `021b9c00` (member). Writes the member Pokemon IDs to
    /// `out` and returns how many.
    fn party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32;
    /// C: `FUN_overlay_d_93__021c7fe0(wk, poke_id)`: the second party of the side (partner in
    /// a double battle), `None` when there is none (null pointer). Same layout as `party_ids`.
    fn partner_party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> Option<u32>;
    /// C: `FUN_overlay_d_93__021c8030(wk, poke_id, x)` (-> `021d80bc`, which ignores `poke_id`):
    /// returns 6 or a value written to event variable 0xD (meaning unverified).
    fn fun_021c8030(&mut self, poke_id: u32, x: u32) -> u32;
    /// C: `FUN_021c8114(wk)`: a move ID, 0 = none (guess).
    fn fun_021c8114(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021c8138(wk)`: the current turn number (`wk +0x10`, low half).
    fn fun_021c8138(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021c814c(wk, poke_id)` (tail call): flow query on a Pokemon.
    fn fun_021c814c(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c8174(wk)`: the ground/terrain kind (0-16, switch in Nature Power-like
    /// handlers).
    fn fun_021c8174(&mut self) -> u32;
    /// C: `FUN_021c8180(wk)` (tail call): small count.
    fn fun_021c8180(&mut self) -> u32;
    /// C: `sub_021c81c8(wk, bpp, 0)` (tail call): a stat of a battle Pokemon (speed-like;
    /// used as a divisor). `poke_id` is the Pokemon behind the `bpp` pointer.
    fn fun_021c81c8(&mut self, poke_id: u32, x: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c824c(wk, poke_id)`: flow query on a Pokemon (tests ability
    /// 0x67 and conditions).
    fn fun_021c824c(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c82c4(wk, side, kind)` (tail call; kinds 9 and 10): side condition test.
    fn fun_021c82c4(&mut self, side: u32, kind: u32) -> bool;
    /// C: `FUN_021c8300(wk, x)` (tail call): small value, 4 = none (guess).
    fn fun_021c8300(&mut self, x: u32) -> u32;
    /// C: `FUN_021c831c(wk, poke_id)`: appends a Pokemon ID to a list of at most 6 in the flow
    /// work; no result.
    fn fun_021c831c(&mut self, poke_id: u32);
    /// C: `FUN_overlay_d_93__021c83a0(wk, poke_id)`: weight-like value of a Pokemon (compared
    /// with 1000 and 2000, used as a divisor), at least 1.
    fn fun_021c83a0(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c83dc(wk, v)`: stores a byte flag in the flow work; no result.
    fn fun_021c83dc(&mut self, v: u32);
    /// C: `FUN_021c83e8(wk, n)`: when the battle rule is below 2, forwards `n` through
    /// `021bb908` and returns true.
    fn fun_021c83e8(&mut self, n: u32) -> bool;
    /// C: `FUN_021c846c(wk, poke_id, target, &out)` (-> `021bfeac`): flow query; `out` receives
    /// a pointer or null (modelled as 0/1, only compared with 0).
    fn fun_021c846c(&mut self, poke_id: u32, target: u32, out: &mut u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8494(wk, poke_id)` (-> `021bff60`): no result.
    fn fun_021c8494(&mut self, poke_id: u32);
    /// C: `FUN_overlay_d_93__021c2e4c(wk)`: increments a counter in the flow work (1 after
    /// wrapping to 0) and returns it.
    fn fun_021c2e4c(&mut self) -> u32;
    /// C: `sub_06898cf4(side, kind)`: whether the side has state `kind` (overlay 95 side
    /// table at +kind*0x10, stride 0xE0 per side).
    fn fun_06898cf4(&mut self, side: u32, kind: u32) -> bool;
    /// C: `sub_0689c8ac(move)`: tests `move` against a 6-entry move list.
    fn fun_0689c8ac(&mut self, mv: u32) -> bool;
    /// C: `sub_0689c8bc(move)`: tests `move` against a 10-entry move list.
    fn fun_0689c8bc(&mut self, mv: u32) -> bool;
    /// C: `sub_0689c91c(ability)`: tests an ability against an 8-entry list (or a flag).
    fn fun_0689c91c(&mut self, ability: u32) -> bool;
    /// C: `sub_0689ca1c(move)`: tests `move` against an 11-entry move list (or a flag).
    fn fun_0689ca1c(&mut self, mv: u32) -> bool;
    /// C: `sub_0689ca3c(move)`: tests `move` against a 20-entry move list (or a flag).
    fn fun_0689ca3c(&mut self, mv: u32) -> bool;
    /// C: `sub_0689ca5c(move)`: tests `move` against a 20-entry move list (or a flag).
    fn fun_0689ca5c(&mut self, mv: u32) -> bool;
    /// C: `sub_0689ca7c(move)`: tests `move` against a 6-entry move list.
    fn fun_0689ca7c(&mut self, mv: u32) -> bool;
    /// C: `sub_0689ca8c(&count)`: a table of 0x29 move IDs (`count` = 0x29; excluded from
    /// the random move pick `021d7e44`).
    fn fun_0689ca8c(&mut self) -> Vec<u16>;
    /// C: `sub_0689d064(wk + 0x1c, move, index)`: whether the move-history ring (0x78
    /// entries of `(u16 move, ..., u32 turn)`) entry `index` back matches `move`.
    fn fun_0689d064(&mut self, mv: u32, index: u32) -> bool;
    /// C: `sub_0689d098(wk + 0x1c, move, turn)`: how many ring entries match `move` and
    /// `turn`.
    fn fun_0689d098(&mut self, mv: u32, turn: u32) -> u32;
    /// C: `sub_0689d0c8(wk + 0x1c, turn)`: the move used on `turn` in the ring, 0 = none.
    fn fun_0689d0c8(&mut self, turn: u32) -> u32;
    /// C: `sub_0689d174(wk + 0x3e0, side)`: member count of a side's party record, 0 past 3.
    fn fun_0689d174(&mut self, side: u32) -> u32;
    /// C: `sub_0689d184(wk + 0x3e0, side, index)`: member Pokemon ID of a side's party record,
    /// 0x1F when out of range.
    fn fun_0689d184(&mut self, side: u32, index: u32) -> u32;
    /// The message (string parameter) buffer that event variable 0x3F points to in move
    /// handlers (`FUN_overlay_d_93__021d775c(0x3f)` is a pointer; `021c8838` / `021c8860`
    /// fill it, see `str_param_setup`). At least 0x40 bytes.
    fn msg_param(&mut self) -> &mut [u8];
    /// C: unlabeled routine at `0x021C7F4C(wk)`: test-and-set of bit 3 of the byte at
    /// `wk +0x78A` (a once-per-battle flag): false if it was already set, else sets it and
    /// returns true.
    fn fun_021c7f4c(&mut self) -> bool;

    /// C: Side-effect layer array query (B7b). Returns the layer count for a side effect (0 if not
    /// present). The layer array at 0x0689E7E0 is seeded by the harness before each handler call;
    /// on the ROM side 06898de0 reads from real memory, on the Rust side this method reads from
    /// TestHost's seeded array.
    fn side_effect_layers(&mut self, side: u32, key: u32) -> u32;
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

    /// `BattleMon::is_fainted` on Pokemon `poke_id`.
    pub fn mon_is_fainted(&mut self, poke_id: u32) -> bool {
        let mon = match self.mons.get(poke_id as usize) {
            Some(m) => m,
            None => &self.null_mon,
        };
        mon.is_fainted(&mut self.host)
    }

    /// `BattleMon::is_full_hp` on Pokemon `poke_id`.
    pub fn mon_is_full_hp(&mut self, poke_id: u32) -> bool {
        let mon = match self.mons.get(poke_id as usize) {
            Some(m) => m,
            None => &self.null_mon,
        };
        mon.is_full_hp(&mut self.host)
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

    /// C: `FUN_overlay_d_93__021d7b60` on Pokemon `poke_id`: max HP / `div`.
    pub fn max_hp_div(&mut self, poke_id: u32, div: u32) -> u32 {
        crate::calc::u32_div(self.mon_get(poke_id, crate::battle_mon::value::MAX_HP), div)
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
