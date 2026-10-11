//! Generic differential driver for event handlers; mirrors tools/rom-diff/handlers.py.

use crate::battle_mon::tests::{mix, random_bpp, MockEnv};
use crate::battle_mon::BppEnv;
use crate::calc::MathRandContext32;
use crate::event_factor::EventSystem;
use crate::event_var::EventVarStack;
use crate::flow::{Battle, FlowHost, HandexId};
use crate::testutil::{Fnv, Rng};

/// Every event-variable label the handlers read or write (0 and 1 are reserved).
pub const LABELS: [u16; 57] = [
    2, 3, 4, 5, 6, 12, 13, 14, 16, 17, 18, 20, 21, 22, 24, 25, 26, 27, 29, 30, 31, 32, 33, 34, 36,
    37, 38, 39, 40, 42, 43, 44, 45, 46, 48, 49, 50, 51, 53, 55, 56, 57, 58, 62, 63, 64, 65, 66, 68,
    69, 70, 71, 72, 75, 77, 81, 83,
];
pub const MONS: usize = 8;
const HANDEX_SIZE: usize = 0x80;

/// Mirrors `Host` in tools/rom-diff/handlers.py.
#[derive(Default)]
pub struct TestHost {
    pub env: MockEnv,
    pub log: Vec<u32>,
    pub bufs: Vec<Vec<u8>>,
    pub wseed: u32,
}

impl BppEnv for TestHost {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.env.pp_get(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.env.pp_put(pp, id, value)
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.env.pp_fast_begin(pp)
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.env.pp_fast_end(pp, token)
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.env.pp_set_move(pp, mv, slot)
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.env.pp_fun_02017f00(pp)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.env.pp_status(pp)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.env.move_max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.env.personal(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.env.field_effect_active(id)
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.env.exp_for_level(species, form, level)
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.env.pp_recalc_stats(pp)
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.env.pp_set_status(pp, status)
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.env.pp_set_form(pp, form)
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.env.fun_0202142c(value)
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.env.sick_is_passable(sick)
    }
}

impl FlowHost for TestHost {
    fn handex_push(&mut self, ty: u32, poke_id: u32) -> HandexId {
        self.log.extend([0x100, ty, poke_id]);
        self.bufs.push(vec![0; HANDEX_SIZE]);
        self.bufs.len() - 1
    }
    fn handex(&mut self, id: HandexId) -> &mut [u8] {
        &mut self.bufs[id]
    }
    fn handex_pop(&mut self, id: HandexId) {
        self.log.push(0x101);
        let words: Vec<u32> = self.bufs[id]
            .chunks(4)
            .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
            .collect();
        self.log.extend(words);
    }
    fn handex_push_pop(&mut self, ty: u32, poke_id: u32) {
        self.log.extend([0x102, ty, poke_id]);
    }
    fn poke_pos(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x103, poke_id]);
        mix(poke_id, 0x7D00) % 6
    }
    fn expand_poke_ids(&mut self, code: u32, out: &mut [u8]) -> u32 {
        self.log.extend([0x104, code]);
        let n = mix(code, 0x7F68) % 7;
        for (i, o) in out.iter_mut().enumerate().take(n as usize) {
            *o = (mix(code, i as u32) % MONS as u32) as u8;
        }
        n
    }
    fn weather(&mut self) -> u32 {
        self.log.push(0x105);
        mix(self.wseed, 0x2AB0) % 5
    }
    fn move_category(&mut self, mv: u32) -> u32 {
        self.log.extend([0x106, mv]);
        mix(mv, 0xC1E8) % 3
    }
    fn move_param(&mut self, mv: u32, param: u32) -> u32 {
        self.log.extend([0x109, mv, param]);
        mix(mv, param ^ 0xBD28) % 3
    }
    fn move_flag(&mut self, mv: u32, flag: u32) -> bool {
        self.log.extend([0x10A, mv, flag]);
        mix(mv, flag ^ 0xBEB8) & 1 != 0
    }
    fn fun_021c82b8(&mut self, x: u32) -> bool {
        self.log.extend([0x10B, x]);
        mix(self.wseed, 0x82B8 ^ x) & 1 != 0
    }
    fn fun_021c83d4(&mut self) -> bool {
        self.log.push(0x10C);
        mix(self.wseed, 0x83D4) & 1 != 0
    }
    fn ability_uncopyable(&mut self, ability: u32) -> bool {
        self.log.extend([0x10D, ability]);
        mix(ability, 0xC944).is_multiple_of(4)
    }
    fn rule(&mut self) -> u32 {
        self.log.push(0x10E);
        mix(self.wseed, 0x815C) % 4
    }
    fn positions_adjacent(&mut self, a: u32, b: u32) -> bool {
        self.log.extend([0x10F, a, b]);
        mix(a ^ (b << 8), 0x9A6C) & 1 != 0
    }
    fn opponent_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32 {
        self.log.extend([0x110, poke_id]);
        let n = mix(poke_id, 0x7D54) % 4;
        for (i, o) in out.iter_mut().enumerate().take(n as usize) {
            *o = (mix(poke_id, 0x7D55 + i as u32) % MONS as u32) as u8;
        }
        n
    }
    fn move_power(&mut self, mv: u32) -> u32 {
        self.log.extend([0x111, mv]);
        [0, 1, 1, 40, 60, 90, 120, 250][(mix(mv, 0xC1D8) % 8) as usize]
    }
    fn move_effect_class(&mut self, mv: u32) -> u32 {
        self.log.extend([0x112, mv]);
        mix(mv, 0xC1BC) % 12
    }
    fn move_fun_0201c314(&mut self, mv: u32) -> bool {
        self.log.extend([0x113, mv]);
        mix(mv, 0xC314) & 1 != 0
    }
    fn move_type(&mut self, mv: u32) -> u32 {
        self.log.extend([0x114, mv]);
        mix(mv, 0xC1C8) % 18
    }
    fn find_action(&mut self, poke_id: u32, out: &mut u32) -> bool {
        self.log.extend([0x107, poke_id]);
        *out = mix(poke_id, 0x804C);
        mix(poke_id, 0x804D) & 1 != 0
    }
    fn poke_pos_in_battle(&mut self, poke_id: u32) -> u32 {
        self.log.extend([0x108, poke_id]);
        mix(poke_id, 0x8010) % 7
    }
}

fn value_for(r: &mut Rng, me: u32) -> u32 {
    match r.next() % 8 {
        0..=2 => me,
        3 | 4 => r.next() % MONS as u32,
        5 => r.next() % 0x200,
        6 => 0x800 + r.next() % 0x1800,
        _ => r.next(),
    }
}

fn hash_battle(h: &mut Fnv, b: &mut Battle<TestHost>, sys: &EventSystem) {
    for chunk in b.vars.to_bytes().chunks(4) {
        let mut w = [0u8; 4];
        w[..chunk.len()].copy_from_slice(chunk);
        h.w(u32::from_le_bytes(w));
    }
    for m in b.mons.iter().chain(std::iter::once(&b.null_mon)) {
        for c in m.raw.chunks(4) {
            h.w(u32::from_le_bytes(c.try_into().unwrap()));
        }
    }
    let f = sys.factor(0);
    f.work.iter().for_each(|&w| h.w(w));
    h.w(f.skip_check);
    h.w(f.flags);
    h.w(b.calc.rng.x as u32);
    h.w((b.calc.rng.x >> 32) as u32);
    for v in b.host.log.drain(..) {
        h.w(v);
    }
    for e in b.host.env.log.drain(..) {
        e.iter().for_each(|&v| h.w(v));
    }
    b.host.bufs.clear();
}

/// Runs every handler in `addrs` over random states; returns the combined hash.
pub fn run_handler_suite(
    addrs: &[u32],
    get: impl Fn(u32) -> crate::flow::Handler<TestHost>,
) -> String {
    let mut h = Fnv::new();
    for &addr in addrs {
        let f = get(addr);
        for seed in 0..200u32 {
            let mut r = Rng::new(addr ^ seed.wrapping_mul(0x9E37));
            let mut b = Battle::new(TestHost::default());
            b.mons = (0..MONS).map(|_| random_bpp(&mut r)).collect();
            b.host.env.field = r.next() & 1 != 0;
            b.host.wseed = r.next();
            let me = r.next() % MONS as u32;
            let mut vars = EventVarStack::new();
            vars.sp = 1;
            vars.label[0] = 1;
            for (i, &lab) in LABELS.iter().enumerate() {
                let s = i + 1;
                vars.label[s] = lab;
                let v = value_for(&mut r, me);
                // Label 5 is a target count used as a loop bound.
                vars.value[s] = if lab == 5 { v % 8 } else { v } as i32;
                vars.kind[s] = [0, 1, 2, 3, 3, 3, 4, 0][(r.next() % 8) as usize];
                if vars.kind[s] == 3 {
                    vars.value[s] = (0x100 + r.next() % 0x10000) as i32;
                    if r.next() & 1 != 0 {
                        vars.min[s] = (r.next() % 0x40) as i32;
                        vars.max[s] = (r.next() % 0x10000) as i32;
                    }
                }
            }
            b.vars = vars;
            let hi = r.next() as u64;
            let lo = r.next() as u64;
            b.calc.rng = MathRandContext32 {
                x: hi << 32 | lo,
                mul: 0x5D58_8B65_6C07_8965,
                add: 0x0026_9EC3,
            };
            let mut sys = EventSystem::new();
            let fac = sys.factor_mut(0);
            for w in fac.work.iter_mut() {
                *w = r.next();
            }
            f(&mut sys, 0, &mut b, me as u8);
            hash_battle(&mut h, &mut b, &sys);
        }
    }
    h.hex()
}
