use super::*;
use crate::battle_mon::tests::{mix, random_bpp};
use crate::event_var::EventVarStack;
use crate::testutil::{Fnv, Rng};

const STUBS: u32 = 0x02F1_0000;
const CB1: u32 = 0x02F1_8000;
const CB2: u32 = 0x02F1_8010;
const NONE: u32 = 0xFF;

fn h(i: u32) -> u32 {
    STUBS + 0x40 * i
}
const LOG: u32 = 0;
const REMOVE: u32 = 1;
const NEST: u32 = 2;
const ADD: u32 = 3;
const WORK: u32 = 4;
const REENTER: u32 = 5;
const E73: u32 = 6;
const NEST_REENTER: u32 = 7;

const TABLE_DEFS: [[(u32, u32); 4]; 4] = [
    [(0, LOG), (1, NEST), (2, WORK), (0x73, LOG)],
    [(0, WORK), (2, REMOVE), (1, ADD), (3, LOG)],
    [(1, NEST_REENTER), (0, E73), (2, LOG), (0x73, WORK)],
    [(2, ADD), (0, REMOVE), (1, REENTER), (3, WORK)],
];

fn table_addr(i: u32) -> u32 {
    0x02F2_0000 + 0x40 * i
}

/// Mirrors `Ctx` in tools/rom-diff/evf.py.
struct Host {
    log: Vec<[u32; 6]>,
    field: bool,
    mons: Vec<BattleMon>,
}

fn idx(f: Option<FactorId>) -> u32 {
    f.map_or(NONE, |v| v as u32)
}

impl EventHost for Host {
    fn handler_entry(&self, table: u32, i: u32) -> (u32, u32) {
        let t = ((table - table_addr(0)) / 0x40) as usize;
        match TABLE_DEFS[t].get(i as usize) {
            Some(&(ev, hid)) => (ev, h(hid) | 1),
            None => (0, 0),
        }
    }

    fn call_handler(
        &mut self,
        sys: &mut EventSystem,
        handler: u32,
        factor: FactorId,
        ctx: u32,
        owner: u8,
    ) {
        let hid = (handler & !1) - STUBS;
        self.log.push([
            1,
            handler & !1,
            factor as u32,
            ctx,
            owner as u32,
            factor as u32,
        ]);
        match hid / 0x40 {
            LOG => {}
            REMOVE => sys.remove(factor),
            NEST => sys.dispatch_all_checked(self, ctx, 2),
            ADD => {
                sys.add(0, 0x55, 1, 0x80, 3, table_addr(1), 2);
            }
            WORK => sys.factor_mut(factor).work[0] = sys.factor(factor).work[0].wrapping_add(1),
            REENTER => sys.allow_reentry(factor),
            E73 => sys.enable_event_0x73(factor),
            NEST_REENTER => {
                sys.allow_reentry(factor);
                sys.dispatch_all(self, ctx, 2);
            }
            _ => unreachable!(),
        }
    }

    fn call_skip_check(
        &mut self,
        _sys: &mut EventSystem,
        cb: u32,
        factor: FactorId,
        ctx: u32,
        ftype: u32,
        event: u32,
        sub_id: u16,
        owner: u8,
    ) -> bool {
        let cb = cb & !1;
        self.log.push([5, cb, factor as u32, ctx, ftype, event]);
        self.log.push([6, sub_id as u32, owner as u32, 0, 0, 0]);
        if cb == CB2 {
            return false;
        }
        mix(
            factor as u32 ^ (ftype << 8),
            event ^ ((sub_id as u32) << 8) ^ ((owner as u32) << 24),
        ) & 1
            != 0
    }

    fn get_mon(&mut self, ctx: u32, poke_id: u32) -> Option<&BattleMon> {
        self.log.push([2, ctx, poke_id, 0, 0, 0]);
        self.mons.get(poke_id as usize)
    }

    fn field_effect_active(&mut self, id: u32) -> bool {
        self.log.push([3, id, 0, 0, 0, 0]);
        self.field
    }

    fn fun_021e03e8(&mut self, _sys: &EventSystem, factor: FactorId) -> bool {
        self.log.push([4, factor as u32, factor as u32, 0, 0, 0]);
        mix(factor as u32, 0x3E8) & 1 != 0
    }
}

fn hash_state(h: &mut Fnv, s: &EventSystem, vars: &EventVarStack, host: &mut Host) {
    for f in &s.pool {
        h.w(idx(f.prev));
        h.w(idx(f.next));
        h.w(f.table);
        h.w(f.skip_check);
        h.w(f.ftype);
        h.w(f.priority);
        h.w(f.flags);
        f.work.iter().for_each(|&w| h.w(w));
        h.w(f.sub_id as u32);
        h.b(f.owner);
        h.b(f.poke_id);
    }
    h.w(s.depth);
    h.w(idx(s.head));
    h.w(s.count);
    s.free.iter().for_each(|&f| h.w(f as u32));
    vars.to_bytes().iter().for_each(|&b| h.b(b));
    for e in host.log.drain(..) {
        e.iter().for_each(|&v| h.w(v));
    }
}

fn op(
    s: &mut EventSystem,
    vars: &mut EventVarStack,
    host: &mut Host,
    o: u32,
    a: u32,
    b: u32,
    d: u32,
) -> Vec<u32> {
    let slot = |v: u32| (v % 0x84) as FactorId;
    let ev = if a & 0x100 != 0 { 0x73 } else { a % 4 };
    match o {
        0 => {
            s.init(vars);
            vec![]
        }
        1..=3 => {
            let r = s.add(
                a % 8,
                b as u16,
                (b >> 16) % 8,
                d & 0xFF_FFFF,
                ((a >> 8) % 6) as u8,
                table_addr((a >> 16) % 4),
                ((a >> 24) % 5) as u16,
            );
            vec![idx(r)]
        }
        4 => {
            s.remove(slot(a));
            vec![]
        }
        5 => {
            s.remove_type6();
            vec![]
        }
        6 => {
            s.enable_all_moves();
            vec![]
        }
        7 => {
            s.dispatch_all(host, b, ev);
            vec![]
        }
        8 => {
            s.dispatch_all_checked(host, b, ev);
            vec![]
        }
        9 => {
            s.dispatch_type(host, b, ev, d % 8);
            vec![]
        }
        10 => {
            s.set_skip_check(slot(a), if b & 1 != 0 { CB1 | 1 } else { CB2 | 1 });
            vec![]
        }
        11 => {
            s.clear_skip_check(slot(a));
            vec![]
        }
        12 => vec![idx(s.find(a % 8, b % 6)), idx(s.find_next(slot(d)))],
        13 => {
            s.disable_moves_of(host, b % 6);
            vec![]
        }
        14 => {
            s.enable_moves_of(b % 6);
            vec![]
        }
        15 => {
            s.disable(b % 6, a % 8);
            vec![]
        }
        16 => vec![s.enable(b % 6, a % 8) as u32],
        17 => {
            let f = slot(a);
            let r = vec![
                s.sub_id(f) as u32,
                s.poke_id(f) as u32,
                s.work(f, d % 7),
                s.is_temporary(f) as u32,
            ];
            match b % 5 {
                0 => s.set_work(f, d % 7, b),
                1 => s.enable_event_0x73(f),
                2 => s.allow_reentry(f),
                3 => s.make_temporary(f),
                _ => {}
            }
            r
        }
        18 => {
            if a & 1 != 0 {
                reset_event_vars_if_used(vars);
            } else {
                reset_event_vars(vars);
            }
            vec![]
        }
        _ => unreachable!(),
    }
}

/// Factor registration, removal, priority order and re-entrant dispatch, hashed against the
/// original code with real (assembled) handlers (`tools/rom-diff/evf.py`).
#[test]
fn matches_rom_fuzz() {
    let mut h = Fnv::new();
    for seed in 0..120u32 {
        let mut r = Rng::new(0xE7 + seed);
        let mons = (0..4).map(|_| random_bpp(&mut r)).collect();
        let mut host = Host {
            log: vec![],
            field: r.next() & 1 != 0,
            mons,
        };
        let mut vars = EventVarStack::new();
        vars.sp = r.next() % 3;
        let mut s = EventSystem::new();
        s.init(&mut vars);
        host.log.clear();
        for _ in 0..60 {
            let o = r.next() % 19;
            let (a, b, d) = (r.next(), r.next(), r.next());
            for v in op(&mut s, &mut vars, &mut host, o, a, b, d) {
                h.w(v);
            }
            hash_state(&mut h, &s, &vars, &mut host);
        }
    }
    assert_eq!(h.hex(), "4d5afe18b01f6b8a");
}

#[test]
fn priority_order_and_removal() {
    let mut vars = EventVarStack::new();
    let mut s = EventSystem::new();
    s.init(&mut vars);
    let a = s.add(4, 1, 0, 10, 0, 0, 0).unwrap();
    let b = s.add(4, 2, 0, 30, 0, 0, 0).unwrap();
    let c = s.add(4, 3, 0, 20, 0, 0, 0).unwrap();
    let d = s.add(4, 4, 0, 20, 0, 0, 0).unwrap();
    // Higher priority first; equal priorities keep insertion order.
    assert_eq!(s.iter_ids(), vec![b, c, d, a]);
    // Sub-priority dominates: (7 - sub) << 24.
    let e = s.add(4, 5, 6, 1000, 0, 0, 0).unwrap();
    assert_eq!(*s.iter_ids().last().unwrap(), e);
    s.remove(c);
    assert_eq!(s.iter_ids(), vec![b, d, a, e]);
    assert_eq!(s.count, 4);
}
