//! The production [`FlowHost`] (B10-wire): the event handlers' server-flow calls answered by the
//! verified ports of `btl_server_flow` instead of the handler harness's stubs.
//!
//! The handler differential tests (`handlers::testkit::TestHost`, mirrored by
//! `tools/rom-diff/handlers.py`) stub every [`FlowHost`] method identically on both sides; that is
//! how the handlers are verified and it stays as it is. [`FlowWorkHost`] is the other
//! implementation: it owns the flow work image ([`SvFlowWork`]) and calls
//!
//! - the flow API ports in [`api`] (B10c, `api.spec`, 69/69 verified) for the methods named after
//!   an `021c7cbc..021c8810` address,
//! - the HANDEX manager ports in [`hem`] (B10b) for the push / pop of handler-effect records,
//! - the battle main-module ports in [`server2`] / [`server3`] (B11a/b) for the three pure helpers
//!   handlers call directly (`021b8ddc` -> `021b8d08`, `021b8c5c`, `021b9a6c`),
//! - core4's [`core4::counter_next`] (B10i) behind the api thunk at `0x021C83CC`,
//! - the overlay-95 side-effect layer array [`SideEffects`] (B7b2) for `06898de0` and `06898cf4`
//!   (B10-wire2),
//! - the ARM9 move-data getters of `black_pokemon::waza` (P4, `waza.spec` 31/31) over a
//!   [`MoveData`] (B10-wire2): the move record cache and static record of the original plus the
//!   record source `W` ([`WazaHooks::load_move`], ARC 21).
//!
//! Callees those ports reach outside their own area go through the [`FlowEnv`] `E`, exactly as in
//! the ports' own differential tests (the production `E` is the battle's call dispatcher, B11/B16).
//! Everything with no port in this crate goes to the [`Unported`] hooks `U`, one method each, so the
//! gap stays visible (no `unimplemented!`). The method -> port table is in the crate README
//! ("Production FlowHost").
//!
//! `waddr` is the address of the work in the battle memory map: the api ports compare pointers the
//! callees return against it (see the [`api`] module doc).

use black_pokemon::waza::{self, MoveCache, WazaHooks, RECORD_SIZE};

use super::env::{Arg, FlowEnv};
use super::{api, core4, hem, SvFlowWork};
use crate::battle_mon::BppEnv;
use crate::flow::{FlowHost, HandexId};
use crate::handlers::effects::{SideEffects, SIDE_KEYS};
use crate::server::{server2, server3};

/// Bytes of the stand-in for the null record a failed HANDEX push returns (larger than any record
/// a handler fills; the biggest HANDEX record kind is below 0x80 bytes).
const NULL_RECORD_SIZE: usize = 0x80;

/// [`FlowHost`] methods with no port in this crate (and the [`BppEnv`] party-Pokemon calls the
/// battle Pokemon layer makes). Each method has the meaning documented on the [`FlowHost`] method of
/// the same name; the note says where its code lives.
pub trait Unported: BppEnv {
    // ARM9 item / Pokemon data (not ported in this crate).
    /// `FUN_020213C0`.
    fn item_fun_020213c0(&mut self, item: u32) -> bool;
    /// `021d805c` -> `FUN_02020FE4` (item data; `black-storage::bag` reads the same records).
    fn item_param(&mut self, item: u32, param: u32) -> u32;
    /// `FUN_020189D0`.
    fn fun_020189d0(&mut self, pp: u32, x: u32) -> i32;
    // Overlay 95 tables and the move-history ring (overlay 95 is not ported apart from the
    // effect handlers and the side-effect adder / remover).
    /// `0689C9B0`.
    fn ability_breakable(&mut self, ability: u32) -> bool;
    /// `0689C8EC`.
    fn move_unredirectable(&mut self, mv: u32) -> bool;
    /// `0689C8DC`.
    fn move_fun_0689c8dc(&mut self, mv: u32) -> bool;
    /// `0689C8CC`.
    fn move_fun_0689c8cc(&mut self, mv: u32) -> bool;
    /// `0689C980`.
    fn item_fun_0689c980(&mut self, item: u32) -> bool;
    /// `0689C990`.
    fn item_fun_0689c990(&mut self, item: u32) -> bool;
    /// `021D7E34` -> `0689C9E4`.
    fn sick_list_entry(&mut self, index: u32) -> u32;
    /// `0689c8ac`.
    fn fun_0689c8ac(&mut self, mv: u32) -> bool;
    /// `0689c8bc`.
    fn fun_0689c8bc(&mut self, mv: u32) -> bool;
    /// `0689c91c` (also behind [`FlowHost::ability_uncopyable`]).
    fn fun_0689c91c(&mut self, ability: u32) -> bool;
    /// `0689ca1c`.
    fn fun_0689ca1c(&mut self, mv: u32) -> bool;
    /// `0689ca3c`.
    fn fun_0689ca3c(&mut self, mv: u32) -> bool;
    /// `0689ca5c`.
    fn fun_0689ca5c(&mut self, mv: u32) -> bool;
    /// `0689ca7c`.
    fn fun_0689ca7c(&mut self, mv: u32) -> bool;
    /// `0689ca8c`.
    fn fun_0689ca8c(&mut self) -> Vec<u16>;
    /// `0689d064(work + 0x1C, ..)`.
    fn fun_0689d064(&mut self, mv: u32, index: u32) -> bool;
    /// `0689d098(work + 0x1C, ..)`.
    fn fun_0689d098(&mut self, mv: u32, turn: u32) -> u32;
    /// `0689d0c8(work + 0x1C, ..)`.
    fn fun_0689d0c8(&mut self, turn: u32) -> u32;
    /// `0689d174(work + 0x3E0, ..)`.
    fn fun_0689d174(&mut self, side: u32) -> u32;
    /// `0689d184(work + 0x3E0, ..)`.
    fn fun_0689d184(&mut self, side: u32, index: u32) -> u32;
    // Flow API entries that are tail calls into overlay 95 (not ported, see the `api` module doc).
    /// `021c7d34` (tail call into overlay 95).
    fn fun_021c7d34(&mut self, poke_id: u32) -> u32;
    /// `021c7d44` (tail call into overlay 95).
    fn fun_021c7d44(&mut self) -> u32;
    /// `021c814c` (tail call into overlay 95).
    fn fun_021c814c(&mut self, poke_id: u32) -> bool;
    /// `021c82c4` (tail call into overlay 95).
    fn fun_021c82c4(&mut self, side: u32, kind: u32) -> bool;
    // Memory the flow work image does not hold.
    /// The string-parameter buffer event variable 0x3F points to in move handlers (owned by the
    /// caller of the event, not by the flow work).
    fn msg_param(&mut self) -> &mut [u8];
    /// A byte store the original makes through a pointer that does not point into the flow work
    /// (`fun_021c83dc` before the move-record pointers are set up).
    fn store_u8(&mut self, addr: u32, v: u8);
}

/// The state of the ARM9 move-data getters (`waza_tool.c`) and where their records come from.
///
/// The original keeps a move record cache behind the global 0x02146530 (`FUN_0201C328` builds it;
/// its only caller is the AI, overlay 96) and otherwise loads each record into the static record at
/// 0x02146534 on every call. `source` is the record source: the getters call only its
/// [`WazaHooks::load_move`] (ARC 21 member = move id; the production source copies it from the
/// extracted assets, D4). Its [`WazaHooks::move_param`] is not used: the wrappers' callee
/// `FUN_0201BD28` is run as [`waza::move_param`] over this cache and static record, as in the
/// original (the private adapter `Getters`).
#[derive(Debug, Clone)]
pub struct MoveData<W: WazaHooks> {
    /// The record source (ARC 21 loader).
    pub source: W,
    /// The move record cache (global 0x02146530; `None` = null).
    pub cache: Option<MoveCache>,
    /// The static record (0x02146534).
    pub scratch: [u8; RECORD_SIZE],
}

impl<W: WazaHooks> MoveData<W> {
    /// No cache, a zeroed static record.
    pub fn new(source: W) -> Self {
        MoveData {
            source,
            cache: None,
            scratch: [0; RECORD_SIZE],
        }
    }

    /// The [`WazaHooks`] the wrapper getters (`0201C1BC` / `C1C8` / `C1D8` / `C1E8` / `C314`) get.
    fn getters(&mut self) -> Getters<'_, W> {
        Getters {
            source: &mut self.source,
            cache: &mut self.cache,
            scratch: &mut self.scratch,
        }
    }
}

/// Adapter: the [`WazaHooks`] of the move getters in the battle. `move_param` (callee
/// `FUN_0201BD28`) is [`waza::move_param`] over the [`MoveData`] cache / static record; everything
/// else goes to the record source.
struct Getters<'a, W: WazaHooks> {
    source: &'a mut W,
    cache: &'a mut Option<MoveCache>,
    scratch: &'a mut [u8; RECORD_SIZE],
}

impl<W: WazaHooks> WazaHooks for Getters<'_, W> {
    fn load_move(&mut self, move_id: u32, dst: &mut [u8; RECORD_SIZE]) {
        self.source.load_move(move_id, dst)
    }
    fn move_param(&mut self, move_id: u32, param: u32) -> u32 {
        waza::move_param(
            move_id,
            param,
            self.cache.as_mut(),
            self.scratch,
            self.source,
        )
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

/// The production [`FlowHost`]: the flow work image plus the environment its ports call into.
pub struct FlowWorkHost<E: FlowEnv, U: Unported, W: WazaHooks> {
    /// The `BTL_SVFLOW_WORK` image.
    pub work: SvFlowWork,
    /// Address of [`Self::work`] in the battle memory map.
    pub waddr: u32,
    /// Callees of the ports outside their own areas.
    pub env: E,
    /// Methods without a port in this crate.
    pub unported: U,
    /// The overlay-95 side-effect layer array (0x0689E7E0).
    pub side_effects: SideEffects,
    /// The ARM9 move-data getters' cache, static record and record source.
    pub moves: MoveData<W>,
    /// Stand-in for the null record of a failed HANDEX push (see [`FlowHost::handex`]).
    null_record: [u8; NULL_RECORD_SIZE],
    /// The ROM table `021b9a6c` reads (0x021EFD70).
    pos_lists: server3::Mem,
}

impl<E: FlowEnv, U: Unported, W: WazaHooks> FlowWorkHost<E, U, W> {
    /// `moves` is the move record source (see [`MoveData`]); the host starts without a move
    /// record cache and with an empty side-effect layer array.
    pub fn new(work: SvFlowWork, waddr: u32, env: E, unported: U, moves: W) -> Self {
        FlowWorkHost {
            work,
            waddr,
            env,
            unported,
            side_effects: SideEffects::default(),
            moves: MoveData::new(moves),
            null_record: [0; NULL_RECORD_SIZE],
            pos_lists: server3::Mem::new(),
        }
    }

    /// C: `FUN_overlay_d_93__021c7cf4` ([`api::mon_by_id`]): the BattleMon address of `poke_id`.
    /// Handlers reach battle Pokemon through [`crate::flow::Battle::mon`] (the battle owns them);
    /// this is the same lookup as the flow ports see it.
    pub fn mon_addr(&mut self, poke_id: u32) -> u32 {
        api::mon_by_id(&mut self.env, &self.work, poke_id)
    }

    /// The pokeIDs of the members of `party` (a BattleParty address): `021b9b94(party)` members,
    /// `021d5440(021b9c00(party, i))` each, written to `out` (at most `out.len()`); returns the
    /// member count.
    fn party_member_ids(&mut self, party: u32, out: &mut [u8]) -> u32 {
        let n = self.env.call(0x021B_9B94, &mut [Arg::V(party)]);
        let mut i = 0u32;
        while i < n && (i as usize) < out.len() {
            let mon = self.env.call(0x021B_9C00, &mut [Arg::V(party), Arg::V(i)]);
            out[i as usize] = self.env.call(0x021D_5440, &mut [Arg::V(mon)]) as u8;
            i += 1;
        }
        n
    }
}

impl<E: FlowEnv, U: Unported, W: WazaHooks> BppEnv for FlowWorkHost<E, U, W> {
    fn pp_get(&mut self, pp: u32, id: u32) -> u32 {
        self.unported.pp_get(pp, id)
    }
    fn pp_put(&mut self, pp: u32, id: u32, value: u32) {
        self.unported.pp_put(pp, id, value)
    }
    fn pp_fast_begin(&mut self, pp: u32) -> u32 {
        self.unported.pp_fast_begin(pp)
    }
    fn pp_fast_end(&mut self, pp: u32, token: u32) {
        self.unported.pp_fast_end(pp, token)
    }
    fn pp_set_move(&mut self, pp: u32, mv: u32, slot: u32) {
        self.unported.pp_set_move(pp, mv, slot)
    }
    fn pp_fun_02017f00(&mut self, pp: u32) -> u32 {
        self.unported.pp_fun_02017f00(pp)
    }
    fn pp_status(&mut self, pp: u32) -> u32 {
        self.unported.pp_status(pp)
    }
    fn move_max_pp(&mut self, mv: u32, pp_ups: u32) -> u32 {
        self.unported.move_max_pp(mv, pp_ups)
    }
    fn personal(&mut self, species: u32, form: u32, param: u32) -> u32 {
        self.unported.personal(species, form, param)
    }
    fn field_effect_active(&mut self, id: u32) -> bool {
        self.unported.field_effect_active(id)
    }
    fn exp_for_level(&mut self, species: u32, form: u32, level: u32) -> u32 {
        self.unported.exp_for_level(species, form, level)
    }
    fn pp_recalc_stats(&mut self, pp: u32) {
        self.unported.pp_recalc_stats(pp)
    }
    fn pp_set_status(&mut self, pp: u32, status: u32) {
        self.unported.pp_set_status(pp, status)
    }
    fn pp_set_form(&mut self, pp: u32, form: u32) {
        self.unported.pp_set_form(pp, form)
    }
    fn fun_0202142c(&mut self, value: u32) -> u32 {
        self.unported.fun_0202142c(value)
    }
    fn sick_is_passable(&mut self, sick: u32) -> bool {
        self.unported.sick_is_passable(sick)
    }
}

impl<E: FlowEnv, U: Unported, W: WazaHooks> FlowHost for FlowWorkHost<E, U, W> {
    // ---- HANDEX records (hem, B10b) ----

    /// [`hem::handex_push`]; the id is the record's work offset (0 = the push failed).
    fn handex_push(&mut self, ty: u32, poke_id: u32) -> HandexId {
        hem::handex_push(&mut self.work, ty, poke_id) as HandexId
    }
    /// The record's bytes in the work, to the end of the work. Id 0 (failed push: the original
    /// hands the handler a null pointer and the handler writes through it) gives a stand-in buffer.
    fn handex(&mut self, id: HandexId) -> &mut [u8] {
        if id == 0 {
            return &mut self.null_record;
        }
        &mut self.work.mem[id..]
    }
    /// [`hem::handex_pop`] (the executor `021c8918` is reached through `E`). Panics for id 0: the
    /// original executes the null record and frees relative to address 0.
    fn handex_pop(&mut self, id: HandexId) {
        assert!(
            id != 0,
            "handex_pop: null HANDEX record (the record pool was full)"
        );
        hem::handex_pop(&mut self.env, &mut self.work, self.waddr, id as u32);
    }
    /// [`hem::handex_push_exec`].
    fn handex_push_pop(&mut self, ty: u32, poke_id: u32) {
        hem::handex_push_exec(&mut self.env, &mut self.work, self.waddr, ty, poke_id);
    }

    // ---- flow API (B10c) ----

    fn poke_pos(&mut self, poke_id: u32) -> u32 {
        api::poke_pos(&mut self.env, &self.work, poke_id)
    }
    fn expand_poke_ids(&mut self, code: u32, out: &mut [u8]) -> u32 {
        api::expand_poke_ids(&mut self.env, &self.work, code, out)
    }
    fn weather(&mut self) -> u32 {
        api::weather(&mut self.env)
    }
    fn fun_021c82b8(&mut self, _x: u32) -> bool {
        api::main_911c(&mut self.env) != 0
    }
    fn fun_021c83d4(&mut self) -> bool {
        api::core1_bc2b0(&mut self.env) != 0
    }
    fn rule(&mut self) -> u32 {
        api::rule(&mut self.env, &self.work)
    }
    fn opponent_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32 {
        api::list_foes(&mut self.env, poke_id, out)
    }
    fn fun_021c8290(&mut self, attacker: u32, defender: u32, mv: u32) -> bool {
        api::mon_pair_bfa00(&mut self.env, &self.work, attacker, defender, mv) != 0
    }
    fn find_action(&mut self, poke_id: u32, out: &mut u32) -> bool {
        let mut o = out.to_le_bytes();
        let r = api::find_action(&mut self.env, &self.work, poke_id, &mut o);
        *out = u32::from_le_bytes(o);
        r != 0
    }
    fn poke_pos_in_battle(&mut self, poke_id: u32) -> u32 {
        api::poke_pos_in_battle(&mut self.env, &self.work, poke_id)
    }
    fn fun_021c8234(&mut self, poke_id: u32) -> bool {
        api::mon_query_6f0c(&mut self.env, &self.work, poke_id) != 0
    }
    fn fun_021c80ac(&mut self, poke_id: u32) -> bool {
        api::others_actions_done(&mut self.env, &self.work, poke_id)
    }
    fn fun_021c8168(&mut self) -> bool {
        api::main_86b8(&mut self.env, &self.work) != 0
    }
    fn fun_021c8434(&mut self, poke_id: u32, attacker: u32) {
        api::bounce_move(&mut self.env, poke_id, attacker)
    }
    fn poke_id_at_pos(&mut self, pos: u32) -> u32 {
        api::poke_id_at_pos(&mut self.env, &self.work, pos)
    }
    fn fun_021c7ec4(&mut self) -> bool {
        api::in_type_check(&self.work)
    }
    fn fun_021c7f24(&mut self, poke_id: u32) -> bool {
        api::party_check(&mut self.env, &self.work, self.waddr, poke_id)
    }
    fn fun_021c818c(&mut self, poke_id: u32) -> bool {
        api::species_check(&mut self.env, &self.work, poke_id)
    }
    fn fun_021c830c(&mut self) -> bool {
        api::flag_78a_bit0(&self.work) != 0
    }
    fn fun_021c8374(&mut self, poke_id: u32) -> bool {
        api::mon_check_8374(&mut self.env, &self.work, poke_id)
    }
    fn fun_021c8408(&mut self, poke_id: u32) {
        api::server_bb914(&mut self.env, &self.work, poke_id)
    }
    fn fun_021c7ed8(&mut self, poke_id: u32) -> u32 {
        api::client_value_997c(&mut self.env, &self.work, poke_id)
    }
    fn party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32 {
        let party = api::party_of(&mut self.env, &self.work, poke_id);
        self.party_member_ids(party, out)
    }
    fn partner_party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> Option<u32> {
        match api::partner_party_of(&mut self.env, &self.work, poke_id) {
            0 => None,
            party => Some(self.party_member_ids(party, out)),
        }
    }
    fn fun_021c8030(&mut self, poke_id: u32, x: u32) -> u32 {
        api::mon_query_80bc(&mut self.env, &self.work, poke_id, x)
    }
    fn fun_021c8114(&mut self) -> u32 {
        api::word_1f74(&self.work) as u32
    }
    fn fun_021c8138(&mut self) -> u32 {
        api::turn_count(&self.work) as u32
    }
    fn fun_021c8174(&mut self) -> u32 {
        api::main_8714_word4(&mut self.env, &self.work, self.waddr)
    }
    fn fun_021c8180(&mut self) -> u32 {
        api::main_8670(&mut self.env, &self.work)
    }
    /// Adapter: handlers pass the BattleMon of `poke_id` (from `021c7cf4`); the host looks it up
    /// with [`api::mon_by_id`] and calls [`api::mon_stat`].
    fn fun_021c81c8(&mut self, poke_id: u32, x: u32) -> u32 {
        let mon = api::mon_by_id(&mut self.env, &self.work, poke_id);
        api::mon_stat(&mut self.env, mon, x)
    }
    fn fun_021c824c(&mut self, poke_id: u32) -> bool {
        api::mon_check_824c(&mut self.env, &self.work, poke_id)
    }
    fn fun_021c8300(&mut self, x: u32) -> u32 {
        api::main_9010(&mut self.env, &self.work, x)
    }
    fn fun_021c831c(&mut self, poke_id: u32) {
        api::list6_push(&mut self.work, poke_id)
    }
    fn fun_021c83a0(&mut self, poke_id: u32) -> u32 {
        api::mon_weight(&mut self.env, &self.work, poke_id)
    }
    /// Adapter: [`api::set_turn_rec_byte4`] on the record `work[0x4A8]` points to (inside the
    /// work once the move-record blocks are selected, see [`api::move_records_select`]); a pointer
    /// outside the work goes to [`Unported::store_u8`].
    fn fun_021c83dc(&mut self, v: u32) {
        let p = self.work.u32_at(0x4A8);
        let ofs = p.wrapping_sub(self.waddr) as usize;
        if ofs < SvFlowWork::SIZE - 4 {
            api::set_turn_rec_byte4(&mut self.work.mem[ofs..], v);
        } else {
            self.unported.store_u8(p.wrapping_add(4), v as u8);
        }
    }
    fn fun_021c83e8(&mut self, n: u32) -> bool {
        api::server_bb908(&mut self.env, &self.work, n)
    }
    /// Adapter: [`api::mon_pair_bfeac`] with `out` as an output buffer (the api port passes the
    /// pointer through; core3 `021bfeac` stores 0 or 1 in it).
    fn fun_021c846c(&mut self, poke_id: u32, target: u32, out: &mut u32) -> bool {
        let ma = api::mon_by_id(&mut self.env, &self.work, poke_id);
        let mb = api::mon_by_id(&mut self.env, &self.work, target);
        let mut o = out.to_le_bytes();
        let r = self.env.call(
            0x021B_FEAC,
            &mut [Arg::P, Arg::V(ma), Arg::V(mb), Arg::Out(&mut o)],
        );
        *out = u32::from_le_bytes(o);
        r != 0
    }
    fn fun_021c8494(&mut self, poke_id: u32) {
        api::mon_bff60(&mut self.env, &self.work, poke_id)
    }
    /// Adapter: handlers call the api thunk at 0x021C83CC ([`api::counter_2e4c`]); its target,
    /// core4 `021c2e4c`, is ported, so the host runs [`core4::counter_next`] on the work.
    fn fun_021c2e4c(&mut self) -> u32 {
        core4::counter_next(&mut self.work)
    }
    fn fun_021c7f4c(&mut self) -> bool {
        api::take_flag_78a_bit3(&mut self.work)
    }

    // ---- battle main module (server2 / server3, B11a/b) ----

    /// Thunk to [`server2::mon_id_client`].
    fn fun_021b8ddc(&mut self, poke_id: u32) -> u32 {
        server2::mon_id_client(poke_id)
    }
    fn fun_021b8c5c(&mut self, rule: u32, pos: u32) -> u32 {
        server2::facing_pos(rule, pos)
    }
    /// [`server3::pos_list_contains`]`(a, b)`. Panics for `a >= 6` (the original reads past the
    /// 6-record table).
    fn positions_adjacent(&mut self, a: u32, b: u32) -> bool {
        server3::pos_list_contains(&self.pos_lists, a, b) != 0
    }

    // ---- overlay 95 side-effect layer array (B7b / B7b2) ----

    /// `06898de0`'s read: the low byte of the layer word of entry `(side, key)`, 0 for `side >= 2`
    /// or `key >= 14` (the key check is done by the caller, `side_effect_layer_count`).
    fn side_effect_layers(&mut self, side: u32, key: u32) -> u32 {
        if side >= 2 || key >= SIDE_KEYS {
            return 0;
        }
        self.side_effects.entries[(side * SIDE_KEYS + key) as usize].layers & 0xFF
    }
    /// [`SideEffects::has_factor`] (`06898cf4`). Panics where the entry lies outside the array
    /// (`side * 14 + kind >= 28`; the original reads past it).
    fn fun_06898cf4(&mut self, side: u32, kind: u32) -> bool {
        self.side_effects.has_factor(side, kind)
    }

    // ---- ARM9 move data (black-pokemon::waza, P4; B10-wire2) ----

    /// `FUN_0201C1E8` = [`waza::param2`].
    fn move_category(&mut self, mv: u32) -> u32 {
        waza::param2(mv, &mut self.moves.getters())
    }
    /// `FUN_0201BD28` = [`waza::move_param`].
    fn move_param(&mut self, mv: u32, param: u32) -> u32 {
        let m = &mut self.moves;
        waza::move_param(mv, param, m.cache.as_mut(), &mut m.scratch, &mut m.source)
    }
    /// `FUN_0201BEB8` = [`waza::record_flag`].
    fn move_flag(&mut self, mv: u32, flag: u32) -> bool {
        let m = &mut self.moves;
        waza::record_flag(mv, flag, m.cache.as_mut(), &mut m.scratch, &mut m.source) != 0
    }
    /// `FUN_0201C1D8` = [`waza::param3_u16`].
    fn move_power(&mut self, mv: u32) -> u32 {
        waza::param3_u16(mv, &mut self.moves.getters())
    }
    /// `FUN_0201C1BC` = [`waza::param1`].
    fn move_effect_class(&mut self, mv: u32) -> u32 {
        waza::param1(mv, &mut self.moves.getters())
    }
    /// `sub_0201C314` = [`waza::param3_nonzero`].
    fn move_fun_0201c314(&mut self, mv: u32) -> bool {
        waza::param3_nonzero(mv, &mut self.moves.getters()) != 0
    }
    /// `FUN_0201C1C8` = [`waza::param0_u8`].
    fn move_type(&mut self, mv: u32) -> u32 {
        waza::param0_u8(mv, &mut self.moves.getters())
    }
    /// `FUN_0201C020` = [`waza::record_pp_ups_count`].
    fn move_fun_0201c020(&mut self, mv: u32) -> u32 {
        let m = &mut self.moves;
        waza::record_pp_ups_count(mv, m.cache.as_mut(), &mut m.scratch, &mut m.source)
    }
    /// Adapter: `FUN_0201C0BC(move, index, &out)` = [`waza::record_signed_if_set`]; handlers read
    /// `out` (the returned flag byte is not used by them).
    fn move_fun_0201c0bc(&mut self, mv: u32, index: u32) -> i32 {
        let m = &mut self.moves;
        let mut out = [0u8; 4];
        waza::record_signed_if_set(
            mv,
            index,
            &mut out,
            m.cache.as_mut(),
            &mut m.scratch,
            &mut m.source,
        );
        i32::from_le_bytes(out)
    }

    // ---- no port in this crate ----

    fn item_fun_020213c0(&mut self, item: u32) -> bool {
        self.unported.item_fun_020213c0(item)
    }
    fn item_param(&mut self, item: u32, param: u32) -> u32 {
        self.unported.item_param(item, param)
    }
    fn fun_020189d0(&mut self, pp: u32, x: u32) -> i32 {
        self.unported.fun_020189d0(pp, x)
    }
    fn ability_breakable(&mut self, ability: u32) -> bool {
        self.unported.ability_breakable(ability)
    }
    fn move_unredirectable(&mut self, mv: u32) -> bool {
        self.unported.move_unredirectable(mv)
    }
    fn move_fun_0689c8dc(&mut self, mv: u32) -> bool {
        self.unported.move_fun_0689c8dc(mv)
    }
    fn move_fun_0689c8cc(&mut self, mv: u32) -> bool {
        self.unported.move_fun_0689c8cc(mv)
    }
    fn item_fun_0689c980(&mut self, item: u32) -> bool {
        self.unported.item_fun_0689c980(item)
    }
    fn item_fun_0689c990(&mut self, item: u32) -> bool {
        self.unported.item_fun_0689c990(item)
    }
    fn sick_list_entry(&mut self, index: u32) -> u32 {
        self.unported.sick_list_entry(index)
    }
    fn fun_0689c8ac(&mut self, mv: u32) -> bool {
        self.unported.fun_0689c8ac(mv)
    }
    fn fun_0689c8bc(&mut self, mv: u32) -> bool {
        self.unported.fun_0689c8bc(mv)
    }
    fn fun_0689c91c(&mut self, ability: u32) -> bool {
        self.unported.fun_0689c91c(ability)
    }
    fn fun_0689ca1c(&mut self, mv: u32) -> bool {
        self.unported.fun_0689ca1c(mv)
    }
    fn fun_0689ca3c(&mut self, mv: u32) -> bool {
        self.unported.fun_0689ca3c(mv)
    }
    fn fun_0689ca5c(&mut self, mv: u32) -> bool {
        self.unported.fun_0689ca5c(mv)
    }
    fn fun_0689ca7c(&mut self, mv: u32) -> bool {
        self.unported.fun_0689ca7c(mv)
    }
    fn fun_0689ca8c(&mut self) -> Vec<u16> {
        self.unported.fun_0689ca8c()
    }
    fn fun_0689d064(&mut self, mv: u32, index: u32) -> bool {
        self.unported.fun_0689d064(mv, index)
    }
    fn fun_0689d098(&mut self, mv: u32, turn: u32) -> u32 {
        self.unported.fun_0689d098(mv, turn)
    }
    fn fun_0689d0c8(&mut self, turn: u32) -> u32 {
        self.unported.fun_0689d0c8(turn)
    }
    fn fun_0689d174(&mut self, side: u32) -> u32 {
        self.unported.fun_0689d174(side)
    }
    fn fun_0689d184(&mut self, side: u32, index: u32) -> u32 {
        self.unported.fun_0689d184(side, index)
    }
    fn fun_021c7d34(&mut self, poke_id: u32) -> u32 {
        self.unported.fun_021c7d34(poke_id)
    }
    fn fun_021c7d44(&mut self) -> u32 {
        self.unported.fun_021c7d44()
    }
    fn fun_021c814c(&mut self, poke_id: u32) -> bool {
        self.unported.fun_021c814c(poke_id)
    }
    fn fun_021c82c4(&mut self, side: u32, kind: u32) -> bool {
        self.unported.fun_021c82c4(side, kind)
    }
    fn msg_param(&mut self) -> &mut [u8] {
        self.unported.msg_param()
    }
}

#[cfg(test)]
mod tests;
