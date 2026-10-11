//! Battle event factors: asm/battle_pokeparam.s 0x021D6EAC-0x021D74E4 (overlay 93).
//!
//! Abilities, items, moves and field effects register *factors*: a handler table (event ID
//! -> handler) plus a priority. Dispatching an event calls every matching handler in
//! priority order. The original keeps the system in globals at `0x021F3CB0`:
//!
//! | offset | field |
//! |---|---|
//! | +0x00 | `u32` dispatch depth |
//! | +0x04 | head of the priority-sorted doubly linked list |
//! | +0x08 | `u32` factors allocated |
//! | +0x0C | `Factor* free[0x84]` (stack; `free[count..]` are available) |
//!
//! and a pool of 0x84 factors of 0x3C bytes at `0x021F4470`. Pointers become pool indices
//! here.
//!
//! Factor flag word (+0x18):
//!
//! | bits | meaning |
//! |---|---|
//! | 0-15 | dispatch depth when added (0 = none); such factors are skipped until it unwinds |
//! | 16-23 | number of handler table entries |
//! | 24 | handler running |
//! | 25 | disabled for a Pokemon ([`EventSystem::disable_moves_of`]) |
//! | 26 | receives event 0x73 |
//! | 27 | removal requested while running |
//! | 28 | may run again while running (re-entry) |
//! | 29 | alive (linked) |
//! | 30 | disabled ([`EventSystem::disable`]) |

use crate::battle_mon::BattleMon;

/// Pool size.
pub const FACTOR_POOL: usize = 0x84;
/// Size of one factor in the original.
pub const FACTOR_SIZE: usize = 0x3C;
/// `type` argument of [`EventSystem::dispatch`] meaning "any factor type".
pub const TYPE_ANY: u32 = 7;
/// Factor type removed by [`EventSystem::remove_type6`].
pub const TYPE_TEMPORARY: u32 = 6;
/// Factor type 4 (abilities): suppressed by condition 0x10 (presumably Gastro Acid).
pub const TYPE_ABILITY: u32 = 4;
/// Factor type 5 (held items): suppressed by field effect 7 (Magic Room), condition 0x13
/// (Embargo) and flag-A bit 9.
pub const TYPE_ITEM: u32 = 5;
/// Event that only reaches factors with flag bit 26.
pub const EVENT_0X73: u32 = 0x73;

const F_CALLING: u32 = 1 << 24;
const F_DISABLED_FOR_MON: u32 = 1 << 25;
const F_EVENT_0X73: u32 = 1 << 26;
const F_REMOVE_PENDING: u32 = 1 << 27;
const F_REENTRANT: u32 = 1 << 28;
const F_ALIVE: u32 = 1 << 29;
const F_DISABLED: u32 = 1 << 30;

/// Pool index of a factor.
pub type FactorId = u8;

/// C: one 0x3C-byte factor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Factor {
    /// +0x00
    pub prev: Option<FactorId>,
    /// +0x04
    pub next: Option<FactorId>,
    /// +0x08: handler table (opaque handle; entries come from [`EventHost::handler_entry`]).
    pub table: u32,
    /// +0x0C: skip-check callback (0 = none).
    pub skip_check: u32,
    /// +0x10
    pub ftype: u32,
    /// +0x14: `(7 - sub_priority) << 24 | priority`; higher runs first.
    pub priority: u32,
    /// +0x18: see the module docs.
    pub flags: u32,
    /// +0x1C: handler work area.
    pub work: [u32; 7],
    /// +0x38
    pub sub_id: u16,
    /// +0x3A: passed to handlers.
    pub owner: u8,
    /// +0x3B: owning Pokemon ID for types 0, 4 and 5, else 0x1F.
    pub poke_id: u8,
}

/// Callbacks into code that is not part of the event system.
pub trait EventHost {
    /// Entry `i` of handler table `table`: `(event, handler)`.
    fn handler_entry(&self, table: u32, i: u32) -> (u32, u32);
    /// Calls a handler: C `handler(factor, ctx, factor->owner, factor->work)`.
    fn call_handler(
        &mut self,
        sys: &mut EventSystem,
        handler: u32,
        factor: FactorId,
        ctx: u32,
        owner: u8,
    );
    /// Calls a skip-check callback: C `cb(factor, ctx, type, event, sub_id, owner)`, where
    /// `type`/`sub_id`/`owner` describe the factor about to run. Non-zero skips it.
    #[allow(clippy::too_many_arguments)]
    fn call_skip_check(
        &mut self,
        sys: &mut EventSystem,
        cb: u32,
        factor: FactorId,
        ctx: u32,
        ftype: u32,
        event: u32,
        sub_id: u16,
        owner: u8,
    ) -> bool;
    /// C: `FUN_overlay_d_93__021c7cf4(ctx, poke_id)`: the battle Pokemon, if any.
    fn get_mon(&mut self, ctx: u32, poke_id: u32) -> Option<&BattleMon>;
    /// C: `FUN_overlay_d_93__021ef948(id)`: whether a field effect is active.
    fn field_effect_active(&mut self, id: u32) -> bool;
    /// C: `FUN_overlay_d_93__021e03e8(factor, factor->work)`.
    fn fun_021e03e8(&mut self, sys: &EventSystem, factor: FactorId) -> bool;
}

/// C: the event-factor globals at `0x021F3CB0` and the pool at `0x021F4470`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSystem {
    pub depth: u32,
    pub head: Option<FactorId>,
    pub count: u32,
    pub free: [FactorId; FACTOR_POOL],
    pub pool: [Factor; FACTOR_POOL],
}

impl Default for EventSystem {
    fn default() -> Self {
        let mut s = EventSystem {
            depth: 0,
            head: None,
            count: 0,
            free: [0; FACTOR_POOL],
            pool: [Factor::default(); FACTOR_POOL],
        };
        for i in 0..FACTOR_POOL {
            s.free[i] = i as FactorId;
        }
        s
    }
}

impl EventSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn factor(&self, f: FactorId) -> &Factor {
        &self.pool[f as usize]
    }

    pub fn factor_mut(&mut self, f: FactorId) -> &mut Factor {
        &mut self.pool[f as usize]
    }

    /// C: `FUN_overlay_d_93__021d6eac`: resets the pool and list, and (like the original)
    /// the event-variable stack.
    pub fn init(&mut self, vars: &mut crate::event_var::EventVarStack) {
        for i in 0..FACTOR_POOL {
            self.pool[i] = Factor::default();
            self.free[i] = i as FactorId;
        }
        self.head = None;
        self.count = 0;
        self.depth = 0;
        reset_event_vars(vars);
    }

    /// C: `FUN_overlay_d_93__021d7474`: takes a factor from the pool.
    fn alloc(&mut self) -> Option<FactorId> {
        if self.count == FACTOR_POOL as u32 {
            return None;
        }
        let f = self.free[self.count as usize];
        self.count += 1;
        Some(f)
    }

    /// C: `FUN_overlay_d_93__021d7494`: clears a factor and returns it to the pool (no-op
    /// when nothing is allocated).
    fn release(&mut self, f: FactorId) {
        if self.count == 0 {
            return;
        }
        self.pool[f as usize] = Factor::default();
        self.count -= 1;
        self.free[self.count as usize] = f;
    }

    /// C: `FUN_overlay_d_93__021d6f0c`: registers a factor (inserted after existing factors
    /// of equal priority). Returns `None` when the pool is exhausted.
    #[allow(clippy::too_many_arguments)]
    pub fn add(
        &mut self,
        ftype: u32,
        sub_id: u16,
        sub_priority: u32,
        priority: u32,
        owner: u8,
        table: u32,
        handler_count: u16,
    ) -> Option<FactorId> {
        let id = self.alloc()?;
        let depth = self.depth;
        let f = &mut self.pool[id as usize];
        f.priority = priority | (7u32.wrapping_sub(sub_priority) << 24);
        f.ftype = ftype;
        f.prev = None;
        f.next = None;
        f.table = table;
        let mut flags = (f.flags & 0xFF00_FFFF) | (((handler_count as u32) << 24) >> 8);
        f.sub_id = sub_id;
        flags &= !(F_CALLING | F_DISABLED_FOR_MON | F_DISABLED);
        f.skip_check = 0;
        f.owner = owner;
        flags &= !(F_EVENT_0X73 | F_REMOVE_PENDING | F_REENTRANT);
        flags = (flags & 0xFFFF_0000) | (depth & 0xFFFF);
        f.flags = flags | F_ALIVE;
        f.poke_id = if ftype == 0 || ftype.wrapping_sub(4) <= 1 {
            owner
        } else {
            0x1F
        };
        f.work = [0; 7];
        self.link(id);
        Some(id)
    }

    fn link(&mut self, id: FactorId) {
        let pri = self.pool[id as usize].priority;
        let Some(head) = self.head else {
            self.head = Some(id);
            return;
        };
        if pri > self.pool[head as usize].priority {
            self.pool[head as usize].prev = Some(id);
            self.pool[id as usize].next = self.head;
            self.pool[id as usize].prev = None;
            self.head = Some(id);
            return;
        }
        let mut last = head;
        let mut cur = self.pool[head as usize].next;
        while let Some(c) = cur {
            if pri > self.pool[c as usize].priority {
                let p = self.pool[c as usize].prev;
                self.pool[id as usize].next = Some(c);
                self.pool[id as usize].prev = p;
                let p = p.expect("non-head node has a predecessor");
                self.pool[p as usize].next = Some(id);
                self.pool[c as usize].prev = Some(id);
                return;
            }
            last = c;
            cur = self.pool[c as usize].next;
        }
        self.pool[last as usize].next = Some(id);
        self.pool[id as usize].prev = Some(last);
    }

    /// C: `FUN_overlay_d_93__021d7040`: unlinks and frees a factor; while its handler runs,
    /// only marks it for removal.
    pub fn remove(&mut self, id: FactorId) {
        let f = self.pool[id as usize];
        if f.flags & F_ALIVE == 0 {
            return;
        }
        if f.flags & F_CALLING != 0 {
            self.pool[id as usize].flags |= F_REMOVE_PENDING;
            return;
        }
        if self.head == Some(id) {
            self.head = f.next;
        }
        if let Some(p) = f.prev {
            self.pool[p as usize].next = f.next;
        }
        if let Some(n) = f.next {
            self.pool[n as usize].prev = f.prev;
        }
        self.release(id);
    }

    fn iter_ids(&self) -> Vec<FactorId> {
        let mut v = Vec::new();
        let mut cur = self.head;
        while let Some(c) = cur {
            v.push(c);
            cur = self.pool[c as usize].next;
        }
        v
    }

    /// C: `FUN_overlay_d_93__021d701c`: removes every factor of type 6.
    pub fn remove_type6(&mut self) {
        let mut cur = self.head;
        while let Some(c) = cur {
            let next = self.pool[c as usize].next;
            if self.pool[c as usize].ftype == TYPE_TEMPORARY {
                self.remove(c);
            }
            cur = next;
        }
    }

    /// C: `FUN_overlay_d_93__021d6ee8`: clears bit 25 on every factor.
    pub fn enable_all_moves(&mut self) {
        for c in self.iter_ids() {
            self.pool[c as usize].flags &= !F_DISABLED_FOR_MON;
        }
    }

    /// C: `FUN_overlay_d_93__021d70e4`: dispatches to all factor types, no skip checks.
    pub fn dispatch_all<H: EventHost>(&mut self, host: &mut H, ctx: u32, event: u32) {
        self.dispatch(host, ctx, event, TYPE_ANY, false);
    }

    /// C: `FUN_overlay_d_93__021d70f0`: dispatches to all factor types with skip checks.
    pub fn dispatch_all_checked<H: EventHost>(&mut self, host: &mut H, ctx: u32, event: u32) {
        self.dispatch(host, ctx, event, TYPE_ANY, true);
    }

    /// C: `FUN_overlay_d_93__021d70fc`: dispatches to one factor type with skip checks.
    pub fn dispatch_type<H: EventHost>(&mut self, host: &mut H, ctx: u32, event: u32, ftype: u32) {
        self.dispatch(host, ctx, event, ftype, true);
    }

    /// C: `FUN_overlay_d_93__021d7108`: runs [`Self::dispatch_level`] one level deeper; when
    /// the outermost dispatch returns, clears every factor's "added at depth" stamp.
    pub fn dispatch<H: EventHost>(
        &mut self,
        host: &mut H,
        ctx: u32,
        event: u32,
        ftype: u32,
        check: bool,
    ) {
        self.depth = self.depth.wrapping_add(1);
        self.dispatch_level(host, ctx, event, ftype, check);
        self.depth = self.depth.wrapping_sub(1);
        if self.depth == 0 {
            for c in self.iter_ids() {
                self.pool[c as usize].flags &= 0xFFFF_0000;
            }
        }
    }

    fn alive(&self, id: FactorId) -> bool {
        self.pool[id as usize].flags & F_ALIVE != 0
    }

    /// C: `FUN_overlay_d_93__021d713c`: the dispatch loop.
    fn dispatch_level<H: EventHost>(
        &mut self,
        host: &mut H,
        ctx: u32,
        event: u32,
        ftype: u32,
        check: bool,
    ) {
        let mut cur = self.head;
        while let Some(id) = cur {
            let mut next = self.pool[id as usize].next;
            'factor: {
                let f = self.pool[id as usize];
                if f.flags & F_CALLING != 0 && f.flags & F_REENTRANT == 0 {
                    break 'factor;
                }
                if f.flags & (F_DISABLED_FOR_MON | F_DISABLED) != 0 {
                    break 'factor;
                }
                if ftype != TYPE_ANY && f.ftype != ftype {
                    break 'factor;
                }
                let stamp = f.flags & 0xFFFF;
                if stamp != 0 && stamp >= self.depth {
                    break 'factor;
                }
                if f.flags & F_ALIVE == 0 {
                    break 'factor;
                }
                if event == EVENT_0X73 && f.flags & F_EVENT_0X73 == 0 {
                    break 'factor;
                }
                let n = (f.flags >> 16) & 0xFF;
                for i in 0..n {
                    let (ev, handler) = host.handler_entry(f.table, i);
                    if ev != event {
                        continue;
                    }
                    if check && self.should_skip(host, ctx, id, event) {
                        break 'factor;
                    }
                    self.pool[id as usize].flags |= F_CALLING;
                    let owner = self.pool[id as usize].owner;
                    host.call_handler(self, handler, id, ctx, owner);
                    let fl = self.pool[id as usize].flags;
                    self.pool[id as usize].flags = if fl & F_REENTRANT != 0 {
                        fl & !F_REENTRANT
                    } else {
                        fl & !F_CALLING
                    };
                    if self.pool[id as usize].flags & F_REMOVE_PENDING != 0 {
                        if let Some(n) = next {
                            if !self.alive(n) {
                                next = self.pool[id as usize].next;
                            }
                        }
                        self.remove(id);
                    }
                    break 'factor;
                }
            }
            let Some(n) = next else {
                return;
            };
            cur = if self.alive(n) {
                Some(n)
            } else {
                self.pool[id as usize].next
            };
        }
    }

    /// C: `FUN_overlay_d_93__021d7240`: whether factor `id` must be skipped for `event`.
    fn should_skip<H: EventHost>(
        &mut self,
        host: &mut H,
        ctx: u32,
        id: FactorId,
        event: u32,
    ) -> bool {
        let f = self.pool[id as usize];
        let (mon_exists, sick_10, sick_13, flag9) = if f.poke_id != 0x1F {
            match host.get_mon(ctx, f.poke_id as u32) {
                Some(m) => (true, m.has_sick(0x10), m.has_sick(0x13), m.flag_a(9)),
                None => (false, false, false, false),
            }
        } else {
            (false, false, false, false)
        };
        if f.ftype == TYPE_ABILITY && mon_exists && sick_10 {
            return true;
        }
        if f.ftype == TYPE_ITEM {
            if host.field_effect_active(7) {
                return true;
            }
            if mon_exists && (sick_13 || flag9) {
                return true;
            }
        }
        let mut cur = self.head;
        while let Some(g) = cur {
            let gf = self.pool[g as usize];
            if gf.skip_check != 0 {
                let suppressed = gf.ftype == TYPE_ABILITY
                    && host
                        .get_mon(ctx, gf.poke_id as u32)
                        .is_some_and(|m| m.has_sick(0x10));
                if !suppressed
                    && gf.flags & F_DISABLED == 0
                    && host.call_skip_check(
                        self,
                        gf.skip_check,
                        g,
                        ctx,
                        f.ftype,
                        event,
                        f.sub_id,
                        f.owner,
                    )
                {
                    return true;
                }
            }
            cur = self.pool[g as usize].next;
        }
        false
    }

    /// C: `FUN_overlay_d_93__021d7320` / `021d7324`
    pub fn set_skip_check(&mut self, id: FactorId, cb: u32) {
        self.pool[id as usize].skip_check = cb;
    }
    pub fn clear_skip_check(&mut self, id: FactorId) {
        self.pool[id as usize].skip_check = 0;
    }

    /// C: `FUN_overlay_d_93__021d732c`: first factor of `ftype` with `owner` not pending
    /// removal.
    pub fn find(&self, ftype: u32, owner: u32) -> Option<FactorId> {
        self.iter_ids().into_iter().find(|&c| {
            let f = &self.pool[c as usize];
            f.ftype == ftype && f.owner as u32 == owner && f.flags & F_REMOVE_PENDING == 0
        })
    }

    /// C: `FUN_overlay_d_93__021d7360`: next factor after `id` with the same type and owner.
    pub fn find_next(&self, id: FactorId) -> Option<FactorId> {
        let f = self.pool[id as usize];
        let mut cur = f.next;
        while let Some(c) = cur {
            let g = &self.pool[c as usize];
            if g.ftype == f.ftype && g.owner == f.owner && g.flags & F_REMOVE_PENDING == 0 {
                return Some(c);
            }
            cur = g.next;
        }
        None
    }

    /// C: `FUN_overlay_d_93__021d7394`: sets bit 25 on type-0 factors of `poke_id` for which
    /// `FUN_overlay_d_93__021e03e8` holds.
    pub fn disable_moves_of<H: EventHost>(&mut self, host: &mut H, poke_id: u32) {
        let mut cur = self.head;
        while let Some(c) = cur {
            let f = self.pool[c as usize];
            if f.ftype == 0 && f.poke_id as u32 == poke_id && host.fun_021e03e8(self, c) {
                self.pool[c as usize].flags |= F_DISABLED_FOR_MON;
            }
            cur = self.pool[c as usize].next;
        }
    }

    /// C: `FUN_overlay_d_93__021d73d4`
    pub fn enable_moves_of(&mut self, poke_id: u32) {
        for c in self.iter_ids() {
            let f = &mut self.pool[c as usize];
            if f.ftype == 0 && f.poke_id as u32 == poke_id {
                f.flags &= !F_DISABLED_FOR_MON;
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d7404`: sets bit 30 on factors of `poke_id` and `ftype`.
    pub fn disable(&mut self, poke_id: u32, ftype: u32) {
        for c in self.iter_ids() {
            let f = &mut self.pool[c as usize];
            if f.poke_id as u32 == poke_id && f.ftype == ftype {
                f.flags |= F_DISABLED;
            }
        }
    }

    /// C: `FUN_overlay_d_93__021d7438`: clears bit 30 on those factors; returns whether any
    /// matched.
    pub fn enable(&mut self, poke_id: u32, ftype: u32) -> bool {
        let mut any = false;
        for c in self.iter_ids() {
            let f = &mut self.pool[c as usize];
            if f.poke_id as u32 == poke_id && f.ftype == ftype {
                f.flags &= !F_DISABLED;
                any = true;
            }
        }
        any
    }

    /// C: `FUN_overlay_d_93__021d70a8`
    pub fn sub_id(&self, id: FactorId) -> u16 {
        self.pool[id as usize].sub_id
    }
    /// C: `FUN_overlay_d_93__021d70ac`
    pub fn poke_id(&self, id: FactorId) -> u8 {
        self.pool[id as usize].poke_id
    }
    /// C: `FUN_overlay_d_93__021d70b4`
    pub fn work(&self, id: FactorId, i: u32) -> u32 {
        self.pool[id as usize].work[i as usize]
    }
    /// C: `FUN_overlay_d_93__021d70dc`
    pub fn set_work(&mut self, id: FactorId, i: u32, v: u32) {
        self.pool[id as usize].work[i as usize] = v;
    }
    /// C: `FUN_overlay_d_93__021d70bc`
    pub fn enable_event_0x73(&mut self, id: FactorId) {
        self.pool[id as usize].flags |= F_EVENT_0X73;
    }
    /// C: `FUN_overlay_d_93__021d70c8`: lets a running handler's factor run again.
    pub fn allow_reentry(&mut self, id: FactorId) {
        let f = &mut self.pool[id as usize];
        if f.flags & F_CALLING != 0 {
            f.flags |= F_REENTRANT;
        }
    }
    /// C: unlabeled routine at `0x021D7090`
    pub fn make_temporary(&mut self, id: FactorId) {
        self.pool[id as usize].ftype = TYPE_TEMPORARY;
    }
    /// C: unlabeled routine at `0x021D7098`
    pub fn is_temporary(&self, id: FactorId) -> bool {
        self.pool[id as usize].ftype == TYPE_TEMPORARY
    }
}

/// C: `FUN_overlay_d_93__021d74c8`: clears the event-variable stack.
pub fn reset_event_vars(vars: &mut crate::event_var::EventVarStack) {
    vars.sp = 0;
    vars.label = [0; crate::event_var::EVENT_VAR_SLOTS];
}

/// C: `FUN_overlay_d_93__021d74e4`: clears the event-variable stack if anything is pushed.
pub fn reset_event_vars_if_used(vars: &mut crate::event_var::EventVarStack) {
    if vars.sp != 0 {
        reset_event_vars(vars);
    }
}

#[cfg(test)]
mod tests;
