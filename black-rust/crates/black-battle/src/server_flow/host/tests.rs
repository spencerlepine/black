//! B10-wire: every wired [`FlowHost`] method of [`FlowWorkHost`] returns what the port it is
//! wired to returns, makes the same callee calls (addresses, scalar arguments, input bytes, in
//! order) and leaves the same work image, on random work images. The ports themselves are verified
//! against the original code by their own suites (`check-svflow.sh api` / `hem`, server2/3, core4,
//! `handlers.py effect-sides` / `effect-side-factor`, `check-fn.sh black-pokemon waza`); these tests check the wiring (right port, argument order,
//! conversions, adapters).

use super::*;
use crate::battle_mon::tests::{mix, MockEnv};
use crate::handlers::effects::SideEffectEntry;
use crate::testutil::Rng;

/// Address of the work image in the tests.
const WADDR: u32 = 0x0220_0000;
/// Logged in place of a pointer argument (`Arg::P` and `Arg::Out` alike).
const PTR: u32 = 0xA0A0_A0A0;
const SEEDS: u32 = 96;

/// A deterministic callee environment: logs every call and answers `mix(..) % 4` (so iterator
/// loops end), fills output buffers, and returns fixed answers for the callees whose result the
/// ports dereference inside the work.
#[derive(Clone, Default)]
struct RecEnv {
    log: Vec<u32>,
    seq: u32,
    salt: u32,
    fixed: Vec<(u32, u32)>,
}

impl FlowEnv for RecEnv {
    fn call(&mut self, addr: u32, args: &mut [Arg<'_>]) -> u32 {
        self.seq = self.seq.wrapping_add(1);
        let mut h = mix(addr ^ self.salt, self.seq);
        self.log.push(addr);
        for a in args.iter_mut() {
            match a {
                Arg::V(v) => {
                    self.log.push(*v);
                    h = mix(h, *v);
                }
                Arg::P | Arg::PBuf(_) | Arg::PHost(_) | Arg::PAt(_) => self.log.push(PTR),
                Arg::In(b) => {
                    self.log.push(PTR);
                    self.log.extend(b.iter().map(|&x| x as u32));
                }
                Arg::Out(b) => {
                    self.log.push(PTR);
                    for (i, o) in b.iter_mut().enumerate() {
                        *o = mix(h, i as u32) as u8;
                    }
                }
            }
        }
        if let Some(&(_, r)) = self.fixed.iter().find(|f| f.0 == addr) {
            return r;
        }
        h % 4
    }
}

/// Logs the unported hooks it is asked for (by name) and answers with small values.
#[derive(Default)]
struct RecUnported {
    bpp: MockEnv,
    calls: Vec<&'static str>,
    stores: Vec<(u32, u8)>,
    msg: Vec<u8>,
}

impl BppEnv for RecUnported {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.bpp.pp_get(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.bpp.pp_put(pp, id, value)
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.bpp.pp_fast_begin(pp)
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.bpp.pp_fast_end(pp, token)
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.bpp.pp_set_move(pp, mv, slot)
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.bpp.pp_fun_02017f00(pp)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.bpp.pp_status(pp)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.bpp.move_max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.bpp.personal(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.bpp.field_effect_active(id)
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.bpp.exp_for_level(species, form, level)
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.bpp.pp_recalc_stats(pp)
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.bpp.pp_set_status(pp, status)
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.bpp.pp_set_form(pp, form)
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.bpp.fun_0202142c(value)
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.bpp.sick_is_passable(sick)
    }
}

macro_rules! rec {
    ($self:ident, $name:literal, $v:expr) => {{
        $self.calls.push($name);
        $v
    }};
}

impl Unported for RecUnported {
    fn item_fun_020213c0(&mut self, _: u32) -> bool {
        rec!(self, "item_fun_020213c0", true)
    }
    fn item_param(&mut self, item: u32, param: u32) -> u32 {
        rec!(self, "item_param", item + param)
    }
    fn fun_020189d0(&mut self, _: u32, _: u32) -> i32 {
        rec!(self, "fun_020189d0", -1)
    }
    fn ability_breakable(&mut self, _: u32) -> bool {
        rec!(self, "ability_breakable", true)
    }
    fn move_unredirectable(&mut self, _: u32) -> bool {
        rec!(self, "move_unredirectable", true)
    }
    fn move_fun_0689c8dc(&mut self, _: u32) -> bool {
        rec!(self, "move_fun_0689c8dc", true)
    }
    fn move_fun_0689c8cc(&mut self, _: u32) -> bool {
        rec!(self, "move_fun_0689c8cc", true)
    }
    fn item_fun_0689c980(&mut self, _: u32) -> bool {
        rec!(self, "item_fun_0689c980", true)
    }
    fn item_fun_0689c990(&mut self, _: u32) -> bool {
        rec!(self, "item_fun_0689c990", true)
    }
    fn sick_list_entry(&mut self, index: u32) -> u32 {
        rec!(self, "sick_list_entry", index + 6)
    }
    fn fun_0689c8ac(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689c8ac", true)
    }
    fn fun_0689c8bc(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689c8bc", true)
    }
    fn fun_0689c91c(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689c91c", true)
    }
    fn fun_0689ca1c(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689ca1c", true)
    }
    fn fun_0689ca3c(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689ca3c", true)
    }
    fn fun_0689ca5c(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689ca5c", true)
    }
    fn fun_0689ca7c(&mut self, _: u32) -> bool {
        rec!(self, "fun_0689ca7c", true)
    }
    fn fun_0689ca8c(&mut self) -> Vec<u16> {
        rec!(self, "fun_0689ca8c", vec![7, 8])
    }
    fn fun_0689d064(&mut self, _: u32, _: u32) -> bool {
        rec!(self, "fun_0689d064", true)
    }
    fn fun_0689d098(&mut self, mv: u32, turn: u32) -> u32 {
        rec!(self, "fun_0689d098", mv + turn)
    }
    fn fun_0689d0c8(&mut self, turn: u32) -> u32 {
        rec!(self, "fun_0689d0c8", turn + 7)
    }
    fn fun_0689d174(&mut self, side: u32) -> u32 {
        rec!(self, "fun_0689d174", side + 8)
    }
    fn fun_0689d184(&mut self, side: u32, index: u32) -> u32 {
        rec!(self, "fun_0689d184", side + index)
    }
    fn fun_021c7d34(&mut self, poke_id: u32) -> u32 {
        rec!(self, "fun_021c7d34", poke_id + 9)
    }
    fn fun_021c7d44(&mut self) -> u32 {
        rec!(self, "fun_021c7d44", 0x1F)
    }
    fn fun_021c814c(&mut self, _: u32) -> bool {
        rec!(self, "fun_021c814c", true)
    }
    fn fun_021c82c4(&mut self, _: u32, _: u32) -> bool {
        rec!(self, "fun_021c82c4", true)
    }
    fn msg_param(&mut self) -> &mut [u8] {
        self.calls.push("msg_param");
        self.msg.resize(0x40, 0);
        &mut self.msg
    }
    fn store_u8(&mut self, addr: u32, v: u8) {
        self.calls.push("store_u8");
        self.stores.push((addr, v));
    }
}

/// A move record source: logs every call, loads a record of bytes `mix(salt ^ move, i)`.
#[derive(Clone, Default)]
struct RecWaza {
    log: Vec<u32>,
    salt: u32,
}

impl RecWaza {
    fn record(&self, mv: u32) -> [u8; RECORD_SIZE] {
        let mut r = [0u8; RECORD_SIZE];
        for (i, b) in r.iter_mut().enumerate() {
            *b = mix(self.salt ^ mv, i as u32) as u8;
        }
        // Byte 4 = 101 and byte 0xE = 6 (special-cased by `record_param`) now and then.
        if mix(mv, 0x65).is_multiple_of(4) {
            r[4] = 101;
            r[0xE] = 6;
        }
        r
    }
}

impl WazaHooks for RecWaza {
    fn load_move(&mut self, move_id: u32, dst: &mut [u8; RECORD_SIZE]) {
        self.log.extend([0x15, move_id]);
        *dst = self.record(move_id);
    }
    fn move_param(&mut self, move_id: u32, param: u32) -> u32 {
        // Not reached by the host: the getters' `0201BD28` runs over the host's cache.
        self.log.extend([0xBD28, move_id, param]);
        0xDEAD
    }
    fn form_record_index(&mut self, species: u32, form: u32) -> u32 {
        self.log.extend([0xAFD4, species, form]);
        0
    }
    fn load_learnset(&mut self, index: u32, _: &mut [u8]) {
        self.log.extend([0x12, index]);
    }
    fn learnset_read(&mut self, species: u32, form: u32, dst: u32, _: &mut [u8]) {
        self.log.extend([0xADD0, species, form, dst]);
    }
    fn free(&mut self, block: u32) {
        self.log.extend([0x07B0, block]);
    }
    fn alloc(&mut self, heap: u32, size: u32, line: u32) -> u32 {
        self.log.extend([0x0734, heap, size, line]);
        0
    }
}

type Host = FlowWorkHost<RecEnv, RecUnported, RecWaza>;

/// A host over a random work image, plus the same environment and work for running the port.
fn setup(seed: u32) -> (Host, RecEnv, SvFlowWork, [u32; 3]) {
    let mut r = Rng::new(seed ^ 0xB10E);
    let mut w = SvFlowWork::new();
    for b in w.mem.iter_mut() {
        *b = r.next() as u8;
    }
    // Pointers the ports dereference inside the work: the per-turn record behind +0x4A8
    // (`move_records_select` block 0), and the records 021b8714 / 021bb85c answer.
    w.set_u32_at(0x4A8, WADDR + 0x874 + 0x1F8);
    let env = RecEnv {
        salt: seed,
        fixed: vec![(0x021B_8714, WADDR + 0x100), (0x021B_B85C, WADDR + 0x200)],
        ..RecEnv::default()
    };
    let args = [r.next() % 0x20, r.next() % 0x20, r.next() % 0x230];
    let waza = RecWaza {
        salt: seed,
        ..RecWaza::default()
    };
    let host = FlowWorkHost::new(w.clone(), WADDR, env.clone(), RecUnported::default(), waza);
    (host, env, w, args)
}

/// `wired!(test, |host, args| host call, |env, work, args| port call)`: for every seed, the host
/// method and the port give the same answer, callee log and work image.
macro_rules! wired {
    ($name:ident, |$h:ident, $a:ident| $host:expr, |$e:ident, $w:ident, $b:ident| $port:expr) => {
        #[test]
        fn $name() {
            for seed in 0..SEEDS {
                let (mut host, mut env, mut work, args) = setup(seed);
                let got = {
                    let $h = &mut host;
                    let $a = args;
                    $host
                };
                let want = {
                    let $e = &mut env;
                    let $w = &mut work;
                    let $b = args;
                    $port
                };
                assert_eq!(got, want, "seed {seed}: result");
                assert_eq!(host.env.log, env.log, "seed {seed}: callee log");
                assert!(host.work.mem[..] == work.mem[..], "seed {seed}: work image");
                assert!(
                    host.unported.calls.is_empty(),
                    "seed {seed}: unported hook called"
                );
            }
        }
    };
}

// ---- flow API (B10c) ----

wired!(poke_pos, |h, a| h.poke_pos(a[0]), |e, w, a| api::poke_pos(
    e, w, a[0]
));
wired!(
    expand_poke_ids,
    |h, a| {
        let mut out = [0u8; 8];
        let n = h.expand_poke_ids(a[2], &mut out);
        (n, out)
    },
    |e, w, a| {
        let mut out = [0u8; 8];
        let n = api::expand_poke_ids(e, w, a[2], &mut out);
        (n, out)
    }
);
wired!(weather, |h, _a| h.weather(), |e, _w, _a| api::weather(e));
wired!(
    fun_021c82b8,
    |h, a| h.fun_021c82b8(a[0]),
    |e, _w, _a| api::main_911c(e) != 0
);
wired!(
    fun_021c83d4,
    |h, _a| h.fun_021c83d4(),
    |e, _w, _a| api::core1_bc2b0(e) != 0
);
wired!(rule, |h, _a| h.rule(), |e, w, _a| api::rule(e, w));
wired!(
    opponent_ids,
    |h, a| {
        let mut out = [0u8; 64];
        let n = h.opponent_ids(a[0], &mut out);
        (n, out)
    },
    |e, _w, a| {
        let mut out = [0u8; 64];
        let n = api::list_foes(e, a[0], &mut out);
        (n, out)
    }
);
wired!(
    fun_021c8290,
    |h, a| h.fun_021c8290(a[0], a[1], a[2]),
    |e, w, a| api::mon_pair_bfa00(e, w, a[0], a[1], a[2]) != 0
);
wired!(
    find_action,
    |h, a| {
        let mut out = 0x5555_5555u32;
        let r = h.find_action(a[0], &mut out);
        (r, out)
    },
    |e, w, a| {
        let mut out = 0x5555_5555u32.to_le_bytes();
        let r = api::find_action(e, w, a[0], &mut out);
        (r != 0, u32::from_le_bytes(out))
    }
);
wired!(
    poke_pos_in_battle,
    |h, a| h.poke_pos_in_battle(a[0]),
    |e, w, a| api::poke_pos_in_battle(e, w, a[0])
);
wired!(
    fun_021c8234,
    |h, a| h.fun_021c8234(a[0]),
    |e, w, a| api::mon_query_6f0c(e, w, a[0]) != 0
);
wired!(fun_021c80ac, |h, a| h.fun_021c80ac(a[0]), |e, w, a| {
    api::others_actions_done(e, w, a[0])
});
wired!(
    fun_021c8168,
    |h, _a| h.fun_021c8168(),
    |e, w, _a| api::main_86b8(e, w) != 0
);
wired!(
    fun_021c8434,
    |h, a| h.fun_021c8434(a[0], a[1]),
    |e, _w, a| api::bounce_move(e, a[0], a[1])
);
wired!(poke_id_at_pos, |h, a| h.poke_id_at_pos(a[0]), |e, w, a| {
    api::poke_id_at_pos(e, w, a[0])
});
wired!(fun_021c7ec4, |h, _a| h.fun_021c7ec4(), |_e, w, _a| {
    api::in_type_check(w)
});
wired!(fun_021c7f24, |h, a| h.fun_021c7f24(a[0]), |e, w, a| {
    api::party_check(e, w, WADDR, a[0])
});
wired!(fun_021c818c, |h, a| h.fun_021c818c(a[0]), |e, w, a| {
    api::species_check(e, w, a[0])
});
wired!(
    fun_021c830c,
    |h, _a| h.fun_021c830c(),
    |_e, w, _a| api::flag_78a_bit0(w) != 0
);
wired!(fun_021c8374, |h, a| h.fun_021c8374(a[0]), |e, w, a| {
    api::mon_check_8374(e, w, a[0])
});
wired!(fun_021c8408, |h, a| h.fun_021c8408(a[0]), |e, w, a| {
    api::server_bb914(e, w, a[0])
});
wired!(fun_021c7ed8, |h, a| h.fun_021c7ed8(a[0]), |e, w, a| {
    api::client_value_997c(e, w, a[0])
});
wired!(
    fun_021c8030,
    |h, a| h.fun_021c8030(a[0], a[2]),
    |e, w, a| api::mon_query_80bc(e, w, a[0], a[2])
);
wired!(
    fun_021c8114,
    |h, _a| h.fun_021c8114(),
    |_e, w, _a| api::word_1f74(w) as u32
);
wired!(
    fun_021c8138,
    |h, _a| h.fun_021c8138(),
    |_e, w, _a| api::turn_count(w) as u32
);
wired!(fun_021c8174, |h, _a| h.fun_021c8174(), |e, w, _a| {
    api::main_8714_word4(e, w, WADDR)
});
wired!(fun_021c8180, |h, _a| h.fun_021c8180(), |e, w, _a| {
    api::main_8670(e, w)
});
wired!(fun_021c824c, |h, a| h.fun_021c824c(a[0]), |e, w, a| {
    api::mon_check_824c(e, w, a[0])
});
wired!(fun_021c8300, |h, a| h.fun_021c8300(a[0]), |e, w, a| {
    api::main_9010(e, w, a[0])
});
wired!(fun_021c831c, |h, a| h.fun_021c831c(a[2]), |_e, w, a| {
    api::list6_push(w, a[2])
});
wired!(fun_021c83a0, |h, a| h.fun_021c83a0(a[0]), |e, w, a| {
    api::mon_weight(e, w, a[0])
});
wired!(fun_021c83e8, |h, a| h.fun_021c83e8(a[2]), |e, w, a| {
    api::server_bb908(e, w, a[2])
});
wired!(fun_021c8494, |h, a| h.fun_021c8494(a[0]), |e, w, a| {
    api::mon_bff60(e, w, a[0])
});
wired!(fun_021c7f4c, |h, _a| h.fun_021c7f4c(), |_e, w, _a| {
    api::take_flag_78a_bit3(w)
});
wired!(mon_addr, |h, a| h.mon_addr(a[0]), |e, w, a| api::mon_by_id(
    e, w, a[0]
));

// ---- adapters ----

wired!(
    fun_021c81c8,
    |h, a| h.fun_021c81c8(a[0], a[1]),
    |e, w, a| {
        let mon = api::mon_by_id(e, w, a[0]);
        api::mon_stat(e, mon, a[1])
    }
);
wired!(fun_021c83dc, |h, a| h.fun_021c83dc(a[2]), |_e, w, a| {
    let ofs = (WADDR + 0x874 + 0x1F8 - WADDR) as usize;
    api::set_turn_rec_byte4(&mut w.mem[ofs..], a[2])
});
// `fun_021c846c`: same callee sequence and answer as the api port (which passes `out` through);
// `out` is what the callee wrote.
wired!(
    fun_021c846c,
    |h, a| {
        let mut out = 0x5555_5555u32;
        let r = h.fun_021c846c(a[0], a[1], &mut out);
        (r, out)
    },
    |e, w, a| {
        // Replays the call on a copy of the environment to learn what the callee writes.
        let mut probe = e.clone();
        let r = api::mon_pair_bfeac(e, w, a[0], a[1]) != 0;
        let ma = api::mon_by_id(&mut probe, w, a[0]);
        let mb = api::mon_by_id(&mut probe, w, a[1]);
        let mut o = [0u8; 4];
        probe.call(
            0x021B_FEAC,
            &mut [Arg::P, Arg::V(ma), Arg::V(mb), Arg::Out(&mut o)],
        );
        assert_eq!(probe.log, e.log);
        (r, u32::from_le_bytes(o))
    }
);
wired!(fun_021c2e4c, |h, _a| h.fun_021c2e4c(), |_e, w, _a| {
    core4::counter_next(w)
});
wired!(
    party_ids,
    |h, a| {
        let mut out = [0u8; 8];
        let n = h.party_ids(a[0], &mut out);
        (n, out)
    },
    |e, w, a| {
        let party = api::party_of(e, w, a[0]);
        party_members(e, party)
    }
);
wired!(
    partner_party_ids,
    |h, a| {
        let mut out = [0u8; 8];
        let n = h.partner_party_ids(a[0], &mut out);
        (n, out)
    },
    |e, w, a| match api::partner_party_of(e, w, a[0]) {
        0 => (None, [0u8; 8]),
        party => {
            let (n, out) = party_members(e, party);
            (Some(n), out)
        }
    }
);

/// The member pokeIDs of a BattleParty as the original reads them (`021b9b94` count,
/// `021d5440(021b9c00(party, i))`).
fn party_members(e: &mut RecEnv, party: u32) -> (u32, [u8; 8]) {
    let mut out = [0u8; 8];
    let n = e.call(0x021B_9B94, &mut [Arg::V(party)]);
    for i in 0..n {
        let mon = e.call(0x021B_9C00, &mut [Arg::V(party), Arg::V(i)]);
        out[i as usize] = e.call(0x021D_5440, &mut [Arg::V(mon)]) as u8;
    }
    (n, out)
}

// ---- HANDEX records (hem) ----

/// A HANDEX manager with `used` bytes already allocated.
fn with_manager(w: &mut SvFlowWork, used: u32) {
    w.set_u32_at(hem::HANDEX_MGR, (used & 0x1FF) << 10);
}

#[test]
fn handex_push_and_pop() {
    for seed in 0..SEEDS {
        let (mut host, mut env, mut work, args) = setup(seed);
        let used = (args[2] % 0x40) & !3;
        with_manager(&mut host.work, used);
        with_manager(&mut work, used);
        let ty = args[0] % 0x3C;
        let id = host.handex_push(ty, args[1]);
        let rec = hem::handex_push(&mut work, ty, args[1]);
        assert_eq!(id as u32, rec, "seed {seed}");
        assert!(host.work.mem[..] == work.mem[..], "seed {seed}: push");
        if id == 0 {
            // Unknown kind: the handler gets the stand-in null record.
            assert_eq!(host.handex(0).len(), NULL_RECORD_SIZE);
            continue;
        }
        // The handler fills the record through `handex`, then pops it.
        host.handex(id)[4..8].copy_from_slice(&seed.to_le_bytes());
        work.mem[rec as usize + 4..rec as usize + 8].copy_from_slice(&seed.to_le_bytes());
        host.handex_pop(id);
        hem::handex_pop(&mut env, &mut work, WADDR, rec);
        assert_eq!(host.env.log, env.log, "seed {seed}: pop log");
        assert_eq!(env.log[0], 0x021C_8918, "seed {seed}: executor");
        assert!(host.work.mem[..] == work.mem[..], "seed {seed}: pop");
    }
}

#[test]
fn handex_push_pop() {
    for seed in 0..SEEDS {
        let (mut host, mut env, mut work, args) = setup(seed);
        with_manager(&mut host.work, 0);
        with_manager(&mut work, 0);
        // Kinds that exist (0x0D is not in the size table).
        let ty = [0u32, 1, 4, 0xB, 0x24, 0x3B][(args[0] % 6) as usize];
        host.handex_push_pop(ty, args[1]);
        hem::handex_push_exec(&mut env, &mut work, WADDR, ty, args[1]);
        assert_eq!(host.env.log, env.log, "seed {seed}");
        assert!(host.work.mem[..] == work.mem[..], "seed {seed}");
    }
}

#[test]
#[should_panic(expected = "null HANDEX record")]
fn handex_pop_of_a_failed_push_panics() {
    let (mut host, ..) = setup(0);
    let id = host.handex_push(0x0D, 0);
    assert_eq!(id, 0);
    host.handex_pop(id);
}

// ---- battle main module (server2 / server3) ----

#[test]
fn fun_021b8ddc() {
    let (mut host, ..) = setup(0);
    for id in 0..0x40 {
        assert_eq!(host.fun_021b8ddc(id), server2::mon_id_client(id));
    }
    // Clients by first id {0, 12, 6, 18}.
    assert_eq!(
        [0, 5, 6, 11, 12, 17, 18, 23, 24].map(|id| host.fun_021b8ddc(id)),
        [0, 0, 2, 2, 1, 1, 3, 3, 0]
    );
    assert!(host.env.log.is_empty());
}

#[test]
fn fun_021b8c5c() {
    let (mut host, ..) = setup(0);
    for rule in 0..6 {
        for pos in 0..8 {
            assert_eq!(host.fun_021b8c5c(rule, pos), server2::facing_pos(rule, pos));
        }
    }
    assert!(host.env.log.is_empty());
}

#[test]
fn positions_adjacent() {
    let (mut host, ..) = setup(0);
    let mem = server3::Mem::new();
    for a in 0..6 {
        for b in 0..8 {
            let want = server3::pos_list_contains(&mem, a, b) != 0;
            assert_eq!(host.positions_adjacent(a, b), want, "{a} {b}");
        }
    }
    // Position 0 (a side-0 edge in a triple): foes 5 and 3, its own side 0 and 2.
    let reach: Vec<u32> = (0..6).filter(|&b| host.positions_adjacent(0, b)).collect();
    assert_eq!(reach, [0, 2, 3, 5]);
    // The centre reaches everything.
    assert!((0..6).all(|b| host.positions_adjacent(2, b)));
    assert!(host.env.log.is_empty());
}

// ---- side-effect layers (B7b2) ----

#[test]
fn side_effect_layers() {
    let (mut host, ..) = setup(0);
    for (i, e) in host.side_effects.entries.iter_mut().enumerate() {
        e.layers = 0x100 * i as u32 + i as u32;
    }
    for side in 0..3 {
        for key in 0..16 {
            let want = if side < 2 && key < 14 {
                (side * 14 + key) & 0xFF
            } else {
                0
            };
            assert_eq!(host.side_effect_layers(side, key), want, "{side} {key}");
        }
    }
}

#[test]
fn fun_06898cf4() {
    for seed in 0..SEEDS {
        let (mut host, ..) = setup(seed);
        let mut r = Rng::new(seed ^ 0x8CF4);
        for e in host.side_effects.entries.iter_mut() {
            let f = r.next();
            *e = SideEffectEntry {
                factor: (!f.is_multiple_of(3)).then_some(((f >> 8) % 0x84) as u8),
                cont: r.next(),
                w8: r.next(),
                layers: r.next() % 5,
            };
        }
        let before = host.side_effects;
        for side in 0..2 {
            for kind in 0..14 {
                let want = before.entries[(side * 14 + kind) as usize].factor.is_some();
                assert_eq!(
                    host.fun_06898cf4(side, kind),
                    want,
                    "seed {seed}: {side} {kind}"
                );
                assert_eq!(host.fun_06898cf4(side, kind), before.has_factor(side, kind));
            }
        }
        // Unchecked like the original: side 0 kind 14 + k is side 1 kind k.
        for k in 0..14 {
            assert_eq!(host.fun_06898cf4(0, 14 + k), host.fun_06898cf4(1, k));
        }
        assert!(
            host.side_effects == before,
            "seed {seed}: the query writes nothing"
        );
        assert!(host.env.log.is_empty() && host.unported.calls.is_empty());
    }
}

#[test]
#[should_panic(expected = "06898cf4: side 2 key 0 is outside the layer array")]
fn fun_06898cf4_outside_the_array_panics() {
    let (mut host, ..) = setup(0);
    host.fun_06898cf4(2, 0);
}

// ---- ARM9 move data (black-pokemon::waza) ----

/// The port side of the move-data tests: the waza routines over their own record source, cache
/// and static record, with `0201BD28` as the wrappers' callee (what the original's wrappers
/// tail-call).
struct RefWaza {
    source: RecWaza,
    cache: Option<MoveCache>,
    scratch: [u8; RECORD_SIZE],
}

impl RefWaza {
    fn bd28(&mut self, mv: u32, param: u32) -> u32 {
        waza::move_param(
            mv,
            param,
            self.cache.as_mut(),
            &mut self.scratch,
            &mut self.source,
        )
    }
}

/// `RefWaza` as the wrappers' hooks: `move_param` is `0201BD28` over the same state.
impl WazaHooks for RefWaza {
    fn load_move(&mut self, move_id: u32, dst: &mut [u8; RECORD_SIZE]) {
        self.source.load_move(move_id, dst)
    }
    fn move_param(&mut self, move_id: u32, param: u32) -> u32 {
        self.bd28(move_id, param)
    }
    fn form_record_index(&mut self, species: u32, form: u32) -> u32 {
        self.source.form_record_index(species, form)
    }
    fn load_learnset(&mut self, index: u32, dst: &mut [u8]) {
        self.source.load_learnset(index, dst)
    }
    fn learnset_read(&mut self, species: u32, form: u32, dst: u32, buf: &mut [u8]) {
        self.source.learnset_read(species, form, dst, buf)
    }
    fn free(&mut self, block: u32) {
        self.source.free(block)
    }
    fn alloc(&mut self, heap: u32, size: u32, line: u32) -> u32 {
        self.source.alloc(heap, size, line)
    }
}

/// Moves of one test run: drawn from a small pool so a cache gets hits, misses and reuse of its
/// last slot.
fn waza_case(seed: u32) -> (Vec<(u32, u32)>, Option<MoveCache>) {
    let mut r = Rng::new(seed ^ 0x0201_BD28);
    let calls = (0..24)
        .map(|_| (1 + r.next() % 9, r.next()))
        .collect::<Vec<_>>();
    // Without a cache (the battle's case: 0201C328 is called by the AI only) for half the seeds,
    // else a cache of 1..4 slots, some already holding moves (with stale records).
    let cache = if seed.is_multiple_of(2) {
        None
    } else {
        let n = 1 + (r.next() % 4) as usize;
        let mut c = MoveCache::new(n);
        for i in 0..n {
            if r.next().is_multiple_of(2) {
                c.ids[i] = (1 + r.next() % 9) as u16;
                for b in c.bufs[i].iter_mut() {
                    *b = r.next() as u8;
                }
            }
        }
        Some(c)
    };
    (calls, cache)
}

/// `waza_wired!(test, |host, mv, x| host call, |port, mv, x| port call)`: for every seed, a
/// sequence of calls on the host and on the waza port over the same record source and cache give
/// the same answers, record loads (in order), cache and static record. `x` is a random word for
/// the second argument. The record-level answer is checked separately by `move_data_semantics`.
macro_rules! waza_wired {
    ($name:ident, |$h:ident, $mv:ident, $x:ident| $host:expr, |$p:ident, $pm:ident, $px:ident| $port:expr) => {
        #[test]
        fn $name() {
            for seed in 0..SEEDS {
                let (mut host, ..) = setup(seed);
                let (calls, cache) = waza_case(seed);
                host.moves.cache = cache.clone();
                let mut port = RefWaza {
                    source: host.moves.source.clone(),
                    cache,
                    scratch: [0; RECORD_SIZE],
                };
                for &(mv, x) in &calls {
                    let got = {
                        let $h = &mut host;
                        let ($mv, $x) = (mv, x);
                        $host
                    };
                    let want = {
                        let $p = &mut port;
                        let ($pm, $px) = (mv, x);
                        $port
                    };
                    assert_eq!(got, want, "seed {seed}: move {mv} x {x:#x}");
                }
                assert_eq!(host.moves.source.log, port.source.log, "seed {seed}: loads");
                assert!(!host.moves.source.log.is_empty(), "seed {seed}: no load");
                assert_eq!(host.moves.cache, port.cache, "seed {seed}: cache");
                assert_eq!(
                    host.moves.scratch, port.scratch,
                    "seed {seed}: static record"
                );
                assert!(host.env.log.is_empty() && host.unported.calls.is_empty());
            }
        }
    };
}

waza_wired!(
    move_category,
    |h, mv, _x| h.move_category(mv),
    |p, mv, _x| waza::param2(mv, p)
);
waza_wired!(
    move_param,
    |h, mv, x| h.move_param(mv, x % 34),
    |p, mv, x| p.bd28(mv, x % 34)
);
waza_wired!(
    move_flag,
    |h, mv, x| h.move_flag(mv, x % 40),
    |p, mv, x| waza::record_flag(mv, x % 40, p.cache.as_mut(), &mut p.scratch, &mut p.source) != 0
);
waza_wired!(move_power, |h, mv, _x| h.move_power(mv), |p, mv, _x| {
    waza::param3_u16(mv, p)
});
waza_wired!(
    move_effect_class,
    |h, mv, _x| h.move_effect_class(mv),
    |p, mv, _x| waza::param1(mv, p)
);
waza_wired!(
    move_fun_0201c314,
    |h, mv, _x| h.move_fun_0201c314(mv),
    |p, mv, _x| waza::param3_nonzero(mv, p) != 0
);
waza_wired!(move_type, |h, mv, _x| h.move_type(mv), |p, mv, _x| {
    waza::param0_u8(mv, p)
});
waza_wired!(
    move_fun_0201c020,
    |h, mv, _x| h.move_fun_0201c020(mv),
    |p, mv, _x| waza::record_pp_ups_count(mv, p.cache.as_mut(), &mut p.scratch, &mut p.source)
);
waza_wired!(
    move_fun_0201c0bc,
    |h, mv, x| h.move_fun_0201c0bc(mv, x % 3),
    |p, mv, x| {
        let mut out = [0x55u8; 4];
        waza::record_signed_if_set(
            mv,
            x % 3,
            &mut out,
            p.cache.as_mut(),
            &mut p.scratch,
            &mut p.source,
        );
        i32::from_le_bytes(out)
    }
);

/// Without a cache every getter loads the move's record into the static record once and answers
/// from its bytes (`waza::record_param` parameters 0 / 1 / 2 / 3, flag word 0x20, bytes
/// 0x15..0x1A), the record source's `move_param` is never called.
#[test]
fn move_data_semantics() {
    for seed in 0..SEEDS {
        let (mut host, ..) = setup(seed);
        let mv = 1 + seed % 700;
        let rec = host.moves.source.record(mv);
        let p = |i| waza::record_param(&rec, i);
        assert_eq!(host.move_type(mv), p(0) & 0xFF);
        assert_eq!(host.move_effect_class(mv), p(1));
        assert_eq!(host.move_category(mv), p(2));
        assert_eq!(host.move_power(mv), p(3) & 0xFFFF);
        assert_eq!(host.move_fun_0201c314(mv), p(3) != 0);
        for prm in 0..34 {
            assert_eq!(host.move_param(mv, prm), p(prm));
        }
        let flags = u32::from_le_bytes(rec[0x20..0x24].try_into().unwrap());
        for bit in 0..32 {
            assert_eq!(host.move_flag(mv, bit), flags >> bit & 1 != 0);
        }
        let n = (0x15..0x18).filter(|&o| rec[o] != 0).count() as u32;
        assert_eq!(host.move_fun_0201c020(mv), n);
        for i in 0..3 {
            let want = if rec[0x15 + i] != 0 {
                rec[0x18 + i] as i8 as i32
            } else {
                0
            };
            assert_eq!(host.move_fun_0201c0bc(mv, i as u32), want);
        }
        let calls = 5 + 34 + 32 + 1 + 3;
        assert_eq!(host.moves.source.log, [0x15, mv].repeat(calls));
        assert_eq!(host.moves.scratch, rec);
    }
}

// ---- unported hooks ----

#[test]
fn unported_methods_reach_the_hooks() {
    let (mut host, ..) = setup(0);
    assert!(host.item_fun_020213c0(0));
    assert_eq!(host.item_param(2, 3), 5);
    assert_eq!(host.fun_020189d0(0, 0), -1);
    assert!(host.ability_breakable(0));
    assert!(host.move_unredirectable(0));
    assert!(host.move_fun_0689c8dc(0));
    assert!(host.move_fun_0689c8cc(0));
    assert!(host.item_fun_0689c980(0));
    assert!(host.item_fun_0689c990(0));
    assert_eq!(host.sick_list_entry(1), 7);
    assert!(host.fun_0689c8ac(0));
    assert!(host.fun_0689c8bc(0));
    assert!(host.fun_0689c91c(0));
    assert!(host.fun_0689ca1c(0));
    assert!(host.fun_0689ca3c(0));
    assert!(host.fun_0689ca5c(0));
    assert!(host.fun_0689ca7c(0));
    assert_eq!(host.fun_0689ca8c(), [7, 8]);
    assert!(host.fun_0689d064(0, 0));
    assert_eq!(host.fun_0689d098(1, 2), 3);
    assert_eq!(host.fun_0689d0c8(1), 8);
    assert_eq!(host.fun_0689d174(1), 9);
    assert_eq!(host.fun_0689d184(1, 2), 3);
    assert_eq!(host.fun_021c7d34(1), 10);
    assert_eq!(host.fun_021c7d44(), 0x1F);
    assert!(host.fun_021c814c(0));
    assert!(host.fun_021c82c4(0, 0));
    assert_eq!(host.msg_param().len(), 0x40);
    // The default body: ability 0x19 never reaches the hook.
    assert!(!host.ability_uncopyable(0x19));
    assert!(host.ability_uncopyable(1));
    assert_eq!(host.unported.calls.len(), 29);
    assert_eq!(host.unported.calls[28], "fun_0689c91c");
    assert!(host.env.log.is_empty());
    assert!(host.moves.source.log.is_empty());
    // BppEnv goes to the hooks too.
    host.pp_put(1, 2, 3);
    assert_eq!(host.unported.bpp.log, [[2, 1, 2, 3]]);
}

#[test]
fn fun_021c83dc_outside_the_work() {
    let (mut host, ..) = setup(0);
    host.work.set_u32_at(0x4A8, 0);
    let before = host.work.mem.clone();
    host.fun_021c83dc(0x1_23);
    assert!(host.work.mem[..] == before[..]);
    assert_eq!(host.unported.stores, [(4, 0x23)]);
}
