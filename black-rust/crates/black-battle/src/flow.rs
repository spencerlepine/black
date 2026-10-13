//! Battle context passed to event handlers (the original's `BTL_SVFLOW_WORK*`).
//!
//! [`Battle`] owns the state handlers read and write directly: the event-variable stack, the
//! battle RNG and the battle Pokemon. Everything else a handler calls in the server-flow layer
//! (asm/battle_server.s, asm/battle_ungrouped.s) goes through [`FlowHost`]: the handler tests stub
//! it (`handlers::testkit::TestHost`, identical to the ROM-side stubs), the production
//! implementation is [`crate::server_flow::host::FlowWorkHost`] (B10-wire), which calls the
//! verified ports (method -> port table in the crate README).

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

/// Server-flow routines handlers call. Doc comments give the behaviour of the verified port the
/// production host ([`crate::server_flow::host::FlowWorkHost`]) calls; names in parentheses are
/// inferred. Methods without a port in this crate are marked "(no port in this crate)".
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
    /// C: `FUN_overlay_d_93__021c7d00(wk, poke_id)`: the Pokemon's battle position (the index the
    /// overlay-95 veneer `7d24(wk + 0x1AB8, poke_id)` finds), 6 when `021b86a0(main, pos)`
    /// rejects it.
    fn poke_pos(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c7f68(wk, code, out)`: the positions `021b8778(main, code)`
    /// expands `code` into -> the IDs of the Pokemon there that exist and are not fainted, written
    /// to `out`; returns how many. Some callers pass the scratch `wk + 0x1FEC`
    /// (`FUN_overlay_d_93__021c8120`), the handler ports a local buffer.
    fn expand_poke_ids(&mut self, code: u32, out: &mut [u8]) -> u32;
    /// C: `FUN_overlay_d_93__021c8288` (-> `021c2ab0`): the current weather.
    fn weather(&mut self) -> u32;
    /// C: `FUN_0201C1E8(move)`: `FUN_0201BD28(move, 2)`, record byte 2, the move's damage
    /// category (1 = physical, 2 = special). ARM9 move data, like the other `FUN_0201xxxx` move
    /// methods: ported in `black_pokemon::waza` (`param2`; the move record is ARC 21 member `move`).
    fn move_category(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201BD28(move, param)`: parameter `param` (0..=31, else 0) of the move record
    /// (`black_pokemon::waza::move_param` / `record_param`, which documents the parameters).
    fn move_param(&mut self, mv: u32, param: u32) -> u32;
    /// C: `FUN_0201BEB8(move, flag)`: bit `flag` of the record's flag word at +0x20 (low byte of
    /// `flag`; 32..=255 give false; `black_pokemon::waza::record_flag`).
    fn move_flag(&mut self, mv: u32, flag: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c82b8(wk, x)`: tail call to `021b911c`, which reads no argument
    /// and returns 0, so this is always false (`x` is unused; `roll_percent_or_forced` never
    /// forces).
    fn fun_021c82b8(&mut self, x: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c83d4(wk)`: thunk to core1 `021bc2b0`: true when one of the two
    /// teams has no client with a usable party (the battle is decided).
    fn fun_021c83d4(&mut self) -> bool;
    /// C: `FUN_0689C944(ability)` via the ARM thunks at 0x021DA2D0 / 0x021DA2E0 and the tail-call
    /// veneer `sub_0689c93c`: the ability cannot be copied (Trace). The ROM body is
    /// `ability != 0x19 && 0689c91c(ability)` (read with rizin, V-0018), so it is provided on top
    /// of [`Self::fun_0689c91c`] and a host cannot answer the two inconsistently.
    fn ability_uncopyable(&mut self, ability: u32) -> bool {
        ability != 0x19 && self.fun_0689c91c(ability)
    }
    /// C: `FUN_overlay_d_93__021c815c(wk)`: tail call `021b8588(main)`, the rule word of the
    /// battle setup (1 double, 2 triple, 3 rotation (names inferred)).
    fn rule(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021b9a6c(a, b)`: `b` is in one of the two position lists of record
    /// `a` of the ROM table 0x021EFD70 (the triple-battle reach of position `a`: the foe positions
    /// in front of it and the positions of its own side next to it, `a` itself included). Not
    /// symmetric in general; `a` must be below 6.
    fn positions_adjacent(&mut self, a: u32, b: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c7d54(wk, poke_id, out)`: the IDs of the existing, non-fainted
    /// Pokemon the core2 iterator (`021bd2b4` / `021bd34c`) visits that are not on `poke_id`'s
    /// side (`021b9a24`; side = ID >= 12), written to `out`; returns how many.
    fn opponent_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32;
    /// C: `FUN_0201C1D8(move)`: `FUN_0201BD28(move, 3)` as a halfword, record byte 3, the base
    /// power (`black_pokemon::waza::param3_u16`).
    fn move_power(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201C1BC(move)`: `FUN_0201BD28(move, 1)`, record byte 1, the move effect class
    /// (9 = one-hit KO; `black_pokemon::waza::param1`).
    fn move_effect_class(&mut self, mv: u32) -> u32;
    /// C: `sub_0201C314(move)`: `FUN_0201BD28(move, 3) != 0`, the move has a base power
    /// (`black_pokemon::waza::param3_nonzero`).
    fn move_fun_0201c314(&mut self, mv: u32) -> bool;
    /// C: `FUN_0201C1C8(move)`: `FUN_0201BD28(move, 0)` as a byte, record byte 0, the move type
    /// (`black_pokemon::waza::param0_u8`).
    fn move_type(&mut self, mv: u32) -> u32;
    /// C: `FUN_0689C9B0(ability)` via the ARM thunk at 0x021DB1C4: Mold Breaker ignores it.
    fn ability_breakable(&mut self, ability: u32) -> bool;
    /// C: `FUN_0689C8EC(move)` via `FUN_021DB550`: the move cannot be redirected.
    fn move_unredirectable(&mut self, mv: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8290(wk, attacker, defender, move)`: core3 `021bfa00` on the two
    /// Pokemon: only in a triple battle (rule 2), and only for a move without move flag 0xB
    /// (`0201BEB8`; move 0 skips the flag test): true when the defender's position is not within
    /// reach ([`Self::positions_adjacent`]) of the attacker's. False otherwise.
    fn fun_021c8290(&mut self, attacker: u32, defender: u32, mv: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c804c(wk, poke_id, &out)`: finds the Pokemon's queued action
    /// record (`wk[0x782]` records of 0x10 bytes from +0x7E0) whose kind (`word & 0xF`) is not 6;
    /// writes its action word to `out` and returns whether found.
    fn find_action(&mut self, poke_id: u32, out: &mut u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8010(wk, poke_id)`: tail call `021b8d8c(main, pokecon,
    /// poke_id)`: the Pokemon's position, or 6 if it is not in battle.
    fn poke_pos_in_battle(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c8234(wk, poke_id)`: core6 `021c6f0c(wk, mon, 1)`: field effect 2
    /// is inactive (`021ef948(2)`) and the event 0x12 query (message 0x2FBC, reply 0x51 defaulting
    /// to `021d5814(mon, 2)` = has type 2, cancelled by reply 0x41) answers non-zero. (The Pokemon
    /// is airborne: Flying type or an event says so, and no Gravity; Arena Trap does not trap
    /// it.)
    fn fun_021c8234(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_0689C8DC(move)` via the ARM thunk at 0x021DB99C.
    fn move_fun_0689c8dc(&mut self, mv: u32) -> bool;
    /// C: `FUN_0689C8CC(move)` via the ARM thunks at 0x021DBE50 / 0x021DC6DC.
    fn move_fun_0689c8cc(&mut self, mv: u32) -> bool;
    /// C: `FUN_0689C980(item)` via the ARM thunk at 0x021E867C.
    fn item_fun_0689c980(&mut self, item: u32) -> bool;
    /// C: `FUN_0689C990(item)` via the ARM thunk at 0x021E868C.
    fn item_fun_0689c990(&mut self, item: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c80ac(wk, poke_id)`: every queued action record (see
    /// [`Self::find_action`]) of a Pokemon other than `poke_id` has its done byte (+0xD) set.
    fn fun_021c80ac(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8168(wk)`: tail call `021b86b8(main)`, the battle type word of
    /// the battle setup; true when it is nonzero (every caller compares with 0).
    fn fun_021c8168(&mut self) -> bool;
    /// C: `FUN_overlay_d_93__021b8ddc(poke_id)`: thunk to `021b8d08`, the client (0..3) of a
    /// Pokemon ID (ID ranges 0-5 -> 0, 12-17 -> 1, 6-11 -> 2, 18-23 -> 3, from 0x021EFCF0; else 0).
    fn fun_021b8ddc(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c8434(wk, poke_id, attacker)`: core2 `021be52c` appends
    /// `(poke_id, attacker, 845c(wk + 0x1AB8, attacker))` to the list behind `wk[0x4B0]` (the
    /// move to bounce back, Magic Bounce; `845c` is an overlay-95 veneer).
    fn fun_021c8434(&mut self, poke_id: u32, attacker: u32);
    /// C: `FUN_020213c0(item)` (Unnerve and Harvest use it as the berry test).
    fn item_fun_020213c0(&mut self, item: u32) -> bool;
    /// C: `FUN_0201C020(move)`: how many of record bytes 0x15..=0x17 are nonzero (the number of
    /// entries `FUN_0201C0BC` reads; `black_pokemon::waza::record_pp_ups_count`).
    fn move_fun_0201c020(&mut self, mv: u32) -> u32;
    /// C: `FUN_0201C0BC(move, index, &out)`: `out` = signed record byte `0x18 + index` if byte
    /// `0x15 + index` is nonzero, else 0 (`black_pokemon::waza::record_signed_if_set`). Returns
    /// `out`; the function's own result (byte `0x15 + index`, V-0022) is not used by handlers.
    fn move_fun_0201c0bc(&mut self, mv: u32, index: u32) -> i32;
    /// C: `sub_021b8c5c(rule, pos)`: the opposing position facing `pos` under `rule` (rule 1:
    /// slot `((pos >> 1) & 0xFF) ^ 1`, rule 2: the mirrored slot, else slot 0 of the other side).
    fn fun_021b8c5c(&mut self, rule: u32, pos: u32) -> u32;
    /// C: `sub_021c8020(wk, pos)`: ID of the Pokemon at `pos` (`021d5440(021b98ac(pokecon,
    /// pos))`).
    fn poke_id_at_pos(&mut self, pos: u32) -> u32;
    /// C: `FUN_021c7ec4(wk)`: the nesting counter `wk[0x774]` is nonzero, i.e. the event runs
    /// inside the simulated type / damage check `021c7dac` / `021c7e14`.
    fn fun_021c7ec4(&mut self) -> bool;
    /// C: `FUN_overlay_d_93__021c7f24(wk, poke_id)`: `021b9bc4(party, n) != 0` for the party of
    /// the Pokemon's client (`021b9af8`) and `n` = 3 in rule 3, else byte +9 of the client record
    /// `021bb85c` returns.
    fn fun_021c7f24(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c818c(wk, poke_id)`: `0201b718(wk[0x4A4], 021d5444(mon), 0, i)` holds for some
    /// `i < 7`.
    fn fun_021c818c(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c830c(wk)`: bit 0 of the flag byte `wk[0x78A]`.
    fn fun_021c830c(&mut self) -> bool;
    /// C: `FUN_overlay_d_93__021c8374(wk, poke_id)`: `x = 021d69ac(mon, 4)` is in 1..=0x18, or
    /// the Pokemon has condition 0x21 (`021d62c4`).
    fn fun_021c8374(&mut self, poke_id: u32) -> bool;
    /// C: `sub_021c8408(wk, poke_id)`: server_side `021bb914(wk[0])` when the battle type
    /// (`021b86b8`) is at most 1 and the Pokemon's client is `021b8ff4(main)`; no result.
    fn fun_021c8408(&mut self, poke_id: u32);
    /// C: `FUN_overlay_d_93__021d805c(item, param)`: item data parameter (tail call through a
    /// function pointer into the ARM9 item data reader, heap ID from a global; no port in this
    /// crate).
    fn item_param(&mut self, item: u32, param: u32) -> u32;
    /// C: `FUN_021D7E34(index)` -> `FUN_0689C9E4(index)`: condition ID `index` of a 6-entry
    /// table (overlay 95), 0 past the end.
    fn sick_list_entry(&mut self, index: u32) -> u32;
    /// C: `FUN_020189D0(pokemon, x)`: Pokemon query returning -1 or a small value (berry
    /// flavour dislike check).
    fn fun_020189d0(&mut self, pp: u32, x: u32) -> i32;
    /// C: `FUN_021c7d34(wk, poke_id)`: tail call into overlay 95 (no port in this crate; the
    /// meaning "battle position, 6 = none" is a guess).
    fn fun_021c7d34(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c7d44(wk)`: tail call into overlay 95 (no port in this crate):
    /// a Pokemon ID, 0x1F = none (guess).
    fn fun_021c7d44(&mut self) -> u32;
    /// C: `FUN_021c7ed8(wk, poke_id)`: `021b997c(main, client)` for the Pokemon's client: outside
    /// comm mode `02007C70(p)` of the client's object `p = main[0x3E0 + 4 client]` when
    /// `02007B50(p)` holds (else 0), in comm mode byte `0x44 + client` of the battle setup. (The
    /// Chatter recording's level: only Chatot's handler `021e12a4` reads it.)
    fn fun_021c7ed8(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c7fcc(wk, poke_id)`: the party of the Pokemon's client
    /// (`021b9af8(pokecon, client)`), which callers read with `021b9b94` (count) and `021b9c00`
    /// (member). Modelled as: writes the member Pokemon IDs (`021d5440`) to `out` and returns how
    /// many.
    fn party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> u32;
    /// C: `FUN_overlay_d_93__021c7fe0(wk, poke_id)`: the party of the partner client
    /// (`021b9010(main, client)`) when `021b8b9c(main, partner)` holds, `None` (a null pointer)
    /// otherwise. Same model as `party_ids`.
    fn partner_party_ids(&mut self, poke_id: u32, out: &mut [u8]) -> Option<u32>;
    /// C: `FUN_overlay_d_93__021c8030(wk, poke_id, move)`: battle_calc `021d80bc(main, pokecon,
    /// mon, move)`, the target position of `move` used by the Pokemon (6 = no target;
    /// `calc_target::fun_021d80bc`).
    fn fun_021c8030(&mut self, poke_id: u32, x: u32) -> u32;
    /// C: `FUN_021c8114(wk)`: the u16 at `wk + 0x1F74` (a move ID, 0 = none: Copycat reads it as
    /// the last move used).
    fn fun_021c8114(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021c8138(wk)`: the current turn number (`wk +0x10`, low half).
    fn fun_021c8138(&mut self) -> u32;
    /// C: `FUN_overlay_d_93__021c814c(wk, poke_id)`: tail call into overlay 95 (no port in this
    /// crate): flow query on a Pokemon.
    fn fun_021c814c(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c8174(wk)`: word +4 of the record `021b8714(main)` returns (`setup + 8`), i.e.
    /// the battle setup word at +0xC (the terrain kind 0-16 Nature Power / Camouflage switch on).
    fn fun_021c8174(&mut self) -> u32;
    /// C: `FUN_021c8180(wk)`: tail call `021b8670(main)`: by rule 0 -> 1, 1 -> 2, 2 -> 3,
    /// else 1 (the positions per side).
    fn fun_021c8180(&mut self) -> u32;
    /// C: `sub_021c81c8(wk, bpp, x)`: thunk to core1 `021bc948`, the speed value of the battle
    /// Pokemon (event 0x13, at most 10000; with `x` != 0 `10000 - v` when event variable 0x4A is
    /// set). `poke_id` is the Pokemon behind the `bpp` pointer.
    fn fun_021c81c8(&mut self, poke_id: u32, x: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c824c(wk, poke_id)`: false when the Pokemon's ability
    /// (`BattleMon_Get` 0x11) is 0x67 or it has condition 0x13, else field effect 7 is inactive.
    /// (It can use its held item: Klutz, Embargo, Magic Room.)
    fn fun_021c824c(&mut self, poke_id: u32) -> bool;
    /// C: `FUN_021c82c4(wk, side, kind)`: tail call into overlay 95 (no port in this crate;
    /// kinds 9 and 10): side condition test.
    fn fun_021c82c4(&mut self, side: u32, kind: u32) -> bool;
    /// C: `FUN_021c8300(wk, x)`: tail call `021b9010(main, x)`: the partner client of client `x`,
    /// `(x + 2) & 3` when that client is present, else 4.
    fn fun_021c8300(&mut self, x: u32) -> u32;
    /// C: unlabeled routine at `0x021C831C(wk, poke_id)`: appends the ID (a byte) to the 6-entry
    /// list at `wk + 0x78B` (count `wk[0x788]`; no-op when full); no result.
    fn fun_021c831c(&mut self, poke_id: u32);
    /// C: `FUN_overlay_d_93__021c83a0(wk, poke_id)`: `max(1, 021d7b10(021d67e8(mon), core4
    /// 021c2af0(wk, mon)))`: the weight times its modifier, at least 1.
    fn fun_021c83a0(&mut self, poke_id: u32) -> u32;
    /// C: `FUN_overlay_d_93__021c83dc(wk, v)`: byte +4 of the per-turn record `wk[0x4A8]` points
    /// to (inside the current move-record block) := `v`; no result.
    fn fun_021c83dc(&mut self, v: u32);
    /// C: `FUN_021c83e8(wk, n)`: when the battle type (`021b86b8`, not the rule) is at most 1,
    /// server_side `021bb908(wk[0], n)` and true; else false.
    fn fun_021c83e8(&mut self, n: u32) -> bool;
    /// C: `FUN_021c846c(wk, poke_id, target, &out)`: core3 `021bfeac` on the two Pokemon: `out`
    /// := 0; when `target` is not fainted, not `021d6d7c` and has none of flag-B bits 3-6
    /// (`021d5c30`): with flag A 7 (`021d5bb0`) puts message 0x20B and sets `out` := 1, else does
    /// the swap bookkeeping and returns true; false otherwise. `out` is a u32 0/1, not a pointer.
    fn fun_021c846c(&mut self, poke_id: u32, target: u32, out: &mut u32) -> bool;
    /// C: `FUN_overlay_d_93__021c8494(wk, poke_id)`: core3 `021bff60(wk, mon, 1)`; no result.
    fn fun_021c8494(&mut self, poke_id: u32);
    /// C: `FUN_overlay_d_93__021c2e4c(wk)` (handlers call the api thunk at 0x021C83CC):
    /// increments the counter `wk[0x778]` (1 after wrapping to 0) and returns it.
    fn fun_021c2e4c(&mut self) -> u32;
    /// C: `sub_06898cf4(side, kind)`: side effect `kind` is active on `side`: the factor word of
    /// entry `(side, kind)` of the side-effect layer array 0x0689E7E0 is nonzero
    /// (`handlers::effects::SideEffects::has_factor`, `handlers.py effect-side-factor`).
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

    /// C: the read of `06898de0` (B7b): the low byte of the layer word of entry `(side, key)` of
    /// the side-effect layer array 0x0689E7E0 (`handlers::effects::SideEffects`), 0 when not
    /// present or `side >= 2` / `key >= 14`. The handler tests seed the array per input (the ROM
    /// side runs the real 06898de0 on it).
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
