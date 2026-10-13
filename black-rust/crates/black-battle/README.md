# black-battle

Safe Rust port of battle logic from overlay 93 (`asm/battle_*.s`; only assembly exists). The
original symbol is in each item's doc comment. Undecompiled callees are traits in `hooks.rs`:
`BattleMon` (`BattleMon_Get`, `HasType`, poke ID, species), `BattleParty`, `PersonalData`
(`FUN_02019C90`) and `Arm9Misc` (`FUN_0202A3FC`). `Heap_AllocDebug`/`Heap_Free` use
`black_core::heap::Heap`.

## Verification

Every function in the tables except the four marked † is hashed against the **original machine
code** run under an emulator (`tools/rom-diff`). The Rust test and the Python harness share the
same input generator, and each stub mirrors its Rust mock. Hand-checked unit tests cover known
game behaviour (type chart, damage examples, stage ratios, hit-count distribution). Mutating the
port (e.g. the rounding boundary in `fx_mul`, or signed vs unsigned division) breaks the hashes.

## battle_calc.s: `calc`

| asm | Rust |
|---|---|
| `FUN_overlay_d_93__021d7840` † | `calc::BattleCalc::init` |
| `FUN_overlay_d_93__021d787c` † | `BattleCalc::set_rng` |
| `FUN_overlay_d_93__021d7894` † | `BattleCalc::free` |
| `FUN_overlay_d_93__021d804c` † | `BattleCalc::set_unk_00` |
| `FUN_overlay_d_93__021d78ac` | `BattleCalc::rand` (`MathRandContext32::next`) |
| `FUN_overlay_d_93__021d7930` | `BattleCalc::roll_critical` |
| `FUN_overlay_d_93__021d7af0` | `BattleCalc::roll_percent` |
| `FUN_overlay_d_93__021d7b8c` | `BattleCalc::rand_range` |
| `FUN_overlay_d_93__021d7ba4` | `BattleCalc::roll_hit_count` |
| `FUN_overlay_d_93__021d7d18` | `BattleCalc::make_default_sick_cont` |
| `FUN_overlay_d_93__021d7dc0` | `BattleCalc::make_sick_cont` |
| `FUN_overlay_d_93__021d7c70` | `BattleCalc::make_move_sick_cont` |
| `FUN_overlay_d_93__021d78e4` | `calc::apply_stat_stage` |
| `FUN_overlay_d_93__021d7908` | `calc::apply_accuracy_stage` |
| `FUN_overlay_d_93__021d794c` | `calc::type_effectiveness` |
| `FUN_overlay_d_93__021d799c` | `calc::type_effectiveness_vs_pair` |
| `FUN_overlay_d_93__021d79e8` | `calc::combine_effectiveness` |
| `CalcBaseDamage` | `calc::calc_base_damage` |
| `ApplyTypeEffectiveness` | `calc::apply_type_effectiveness` |
| `FUN_overlay_d_93__021d7a9c` | `calc::resisting_types` |
| `GetTypeEffectivenessClass` | `calc::effectiveness_class` |
| `FUN_overlay_d_93__021d7b10` / `7b2c` | `calc::fx_mul` / `fx_mul_min1` |
| `FUN_overlay_d_93__021d7b3c` | `calc::percent_mul` |
| `FUN_overlay_d_93__021d7b60` / `7b74` | `calc::max_hp_div` / `max_hp_div_min1` |
| `FUN_overlay_d_93__021d7bd4` | `calc::weather_residual_damage` |
| `FUN_overlay_d_93__021d7c38` | `calc::weather_power_modifier` |
| `FUN_overlay_d_93__021d7eb4` | `calc::exp_yield` |
| `FUN_overlay_d_93__021d7ef0` | `calc::is_not_0_or_3` |
| `FUN_overlay_d_93__021d7f00` / `7f2c` | `calc::fun_021d7f00` / `fun_021d7f2c` |
| `FUN_overlay_d_93__021d7f38` / `7f4c` / `7f60` | `calc::fun_021d7f38` / `fun_021d7f4c` / `fun_021d7f60` |
| `FUN_overlay_d_93__021d7ff8` | `calc::prize_money` |
| `FUN_overlay_d_93__021d806c` / `8090` | `calc::pack_5bit_x6` / `unpack_5bit_x6` |
| `FUN_overlay_d_93__021d80a8` | `calc::fun_021d80a8` |
| `FUN_0209C2B0` (`_u32_div_f`) | `calc::u32_div` / `u32_divmod` |
| `battle_calc_rodata` (0x021F041C, first 0x181 bytes) | `calc::BATTLE_CALC_RODATA` |

† Unit-tested only. These are state setters, plus the `Heap` call whose implementation is a
trait.

## battle_calc.s: `event_var`

| asm | Rust |
|---|---|
| global `0x021F3ECC` | `event_var::EventVarStack` |
| `FUN_overlay_d_93__021d74f8` / `7540` | `EventVarStack::push` / `pop` |
| `FUN_overlay_d_93__021d75a8` / `75ec` / `7630` / `7674` | `set_value` / `set_const_value` / `set_rewrite_once_value` / `set_mul_value` |
| `FUN_overlay_d_93__021d76bc` | `rewrite_value` |
| `FUN_overlay_d_93__021d7704` | `mul_value` |
| `FUN_overlay_d_93__021d775c` | `get_value` |
| unlabeled Thumb at `0x021D778C` (data bytes in the .s) | `get_value_if_exists` |
| `FUN_overlay_d_93__021d77c4` / `77e8` / `780c` | `slot_for_set` / `find` / `clamp` |

## battle_view_core.s: `sick_cont`, `poke_type`

| asm | Rust |
|---|---|
| `FUN_overlay_d_93__021e8940` / `896c` | `SickCont::turn` / `turn_param` |
| `FUN_overlay_d_93__021e899c` / `89c0` / `89d4` / `89f8` | `SickCont::poke` / `permanent` / `permanent_param` / `permanent_count` |
| `FUN_overlay_d_93__021e8a28` / `8a58` / `8a78` | `SickCont::poke_turn` / `badly_poisoned` / `is_badly_poisoned` |
| `FUN_overlay_d_93__021e8ce0` / `8cf0` | `poke_type::make_pair` / `make_mono_pair` |
| `FUN_overlay_d_93__021e8cfc` / `8d04` / `8d0c` | `pair_type1` / `pair_type2` / `split_pair` |
| `FUN_overlay_d_93__021e8d24` / `8da8` | `pair_has_type` / `pair_is_mono` |

## handlers::effects (B7: overlay-95 registries)

Decoded by B7h; the full table and evidence are in the module doc of
`src/handlers/effects.rs`. Summary: overlay 95 (linked at 0x06898020) has three registries with
the same getter protocol as abilities/items/moves: field effects 0x0689D650 (8 keys, factor
type 3, owner 0), **position effects** 0x0689D6C8 (5 keys, type 1, owner = battle position;
Wish, Lunar Dance, Healing Wish, Future Sight / Doom Desire, Baton Pass) and **side effects**
0x0689D780 (14 keys with a max layer count, type 2, owner = side; Reflect, Light Screen,
Safeguard, Mist, Tailwind, Lucky Chant, Spikes, Toxic Spikes, Stealth Rock, Wide Guard, Quick
Guard, 3 Pledge combos). The "type-2 adder at 0x06898328" is the Wish handler; the adder is
`06898c10`. 30 distinct handlers (`tools/rom-diff/handlers.py effects --list`), checked with
`tools/check-handlers.sh effects`.

## Production FlowHost (B10-wire): `server_flow::host::FlowWorkHost`

Handlers reach the server flow through the `flow::FlowHost` trait. The handler differential tests
keep their stub host (`handlers::testkit::TestHost`, the same stubs as `tools/rom-diff/handlers.py`;
that is how the handlers are verified, and it is unchanged). `FlowWorkHost<E: FlowEnv, U: Unported,
W: WazaHooks>` is the production implementation: it owns the `SvFlowWork`
image (`work`, at address `waddr`), the overlay-95 side-effect layer array (`side_effects`), the ARM9
move-data state (`moves: MoveData<W>`: the move record cache of global 0x02146530, the static record
0x02146534 and the record source `W`) and calls the verified ports. Callees those ports
reach outside their own area go through `E` (the battle's call dispatcher), as in the ports' own
differential tests. Methods with no port in this crate call the `Unported` hook trait `U` (one method
each, plus `BppEnv`), so the gap stays visible; nothing is `unimplemented!`. Unit tests
(`server_flow/host/tests.rs`) check each wired method against its port on random work images: same
answer, same callee calls in order, same work bytes.

87 trait methods: **51 mapped**, **7 adapted**, **28 unported**, 1 default body (`ability_uncopyable`
= `ability != 0x19 && fun_0689c91c(ability)`). B10-wire2 added the 9 move-data methods and
`fun_06898cf4`.

Move data (B10-wire2): this crate depends on `black-pokemon` (logic crate, no cycle) and calls the
`black_pokemon::waza` ports (P4, `waza.spec` 31/31). The record source is the generic `W:
black_pokemon::waza::WazaHooks` given to `FlowWorkHost::new` (no global): the getters call only its
`load_move` (ARC 21 member = move id; the production source copies the extracted record, D4). The
wrappers' callee `FUN_0201BD28` is not `W::move_param` but `waza::move_param` over the host's cache /
static record (adapter `Getters`), as the original's wrappers tail-call it. There is no cache unless
the integrator sets `moves.cache` (the original builds one only for the AI, `0201C328` from overlay 96).

| FlowHost method | port |
|---|---|
| `handex_push` / `handex` / `handex_pop` / `handex_push_pop` | `hem::handex_push` (88a4; the id is the record's work offset) / the record bytes / `hem::handex_pop` (88c8) / `hem::handex_push_exec` (88b4) |
| `poke_pos` | `api::poke_pos` (7d00) |
| `expand_poke_ids` | `api::expand_poke_ids` (7f68) |
| `weather` | `api::weather` (8288) |
| `fun_021c82b8` | `api::main_911c` (always 0) |
| `fun_021c83d4` | `api::core1_bc2b0` (83d4) |
| `rule` | `api::rule` (815c) |
| `opponent_ids` | `api::list_foes` (7d54) |
| `fun_021c8290` | `api::mon_pair_bfa00` |
| `find_action` | `api::find_action` (804c; `out` as 4 bytes) |
| `poke_pos_in_battle` | `api::poke_pos_in_battle` (8010) |
| `fun_021c8234` | `api::mon_query_6f0c` |
| `fun_021c80ac` | `api::others_actions_done` |
| `fun_021c8168` | `api::main_86b8` |
| `fun_021c8434` | `api::bounce_move` |
| `poke_id_at_pos` | `api::poke_id_at_pos` (8020) |
| `fun_021c7ec4` / `fun_021c7f24` / `fun_021c818c` / `fun_021c830c` / `fun_021c8374` / `fun_021c8408` | `api::in_type_check` / `party_check` / `species_check` / `flag_78a_bit0` / `mon_check_8374` / `server_bb914` |
| `fun_021c7ed8` / `fun_021c8030` / `fun_021c8114` / `fun_021c8138` / `fun_021c8174` / `fun_021c8180` | `api::client_value_997c` / `mon_query_80bc` / `word_1f74` / `turn_count` / `main_8714_word4` / `main_8670` |
| `fun_021c824c` / `fun_021c8300` / `fun_021c831c` / `fun_021c83a0` / `fun_021c83e8` / `fun_021c8494` / `fun_021c7f4c` | `api::mon_check_824c` / `main_9010` / `list6_push` / `mon_weight` / `server_bb908` / `mon_bff60` / `take_flag_78a_bit3` |
| `fun_021b8ddc` | `server2::mon_id_client` (thunk target 021b8d08) |
| `fun_021b8c5c` | `server2::facing_pos` |
| `positions_adjacent` | `server3::pos_list_contains` (021b9a6c; ROM table 0x021EFD70) |
| `side_effect_layers` | `SideEffects` entry layers, low byte (06898de0's read, B7b) |
| `fun_06898cf4` | `SideEffects::has_factor` (06898cf4: factor word != 0; `handlers.py effect-side-factor` **337dea218d0ba4d8**, mirrored by `effect_side_factor_match_rom`) |
| `move_param` / `move_flag` / `move_fun_0201c020` | `waza::move_param` (0201BD28) / `record_flag` (0201BEB8) / `record_pp_ups_count` (0201C020) |
| `move_category` / `move_effect_class` / `move_type` / `move_power` / `move_fun_0201c314` | `waza::param2` (0201C1E8) / `param1` (C1BC) / `param0_u8` (C1C8) / `param3_u16` (C1D8) / `param3_nonzero` (C314) |
| adapter `move_fun_0201c0bc` | `waza::record_signed_if_set` (0201C0BC); returns the `out` word (handlers ignore the flag byte the function returns, V-0022) |
| adapter `party_ids` / `partner_party_ids` | `api::party_of` / `partner_party_of`, then `021b9b94` / `021b9c00` / `021d5440` through `E` (the original returns the party pointer; handlers read it) |
| adapter `fun_021c81c8` | `api::mon_by_id` + `api::mon_stat` (handlers pass the BattleMon pointer) |
| adapter `fun_021c83dc` | `api::set_turn_rec_byte4` on the record `work[0x4A8]` points to; a pointer outside the work goes to `Unported::store_u8` |
| adapter `fun_021c846c` | `api::mon_pair_bfeac` with `out` as an output buffer (the api port passes it through) |
| adapter `fun_021c2e4c` | `core4::counter_next` (handlers call the api thunk 0x021C83CC, whose target is ported) |

`Battle::mon` is not a FlowHost method (the battle owns its `BattleMon`s); `FlowWorkHost::mon_addr`
is the same lookup as the flow ports see it (`api::mon_by_id`, 7cf4). The integrating dispatcher `E`
must map `021b9934(pokecon, id)` to the BattleMon of `Battle::mons[id]`.

Unported (`Unported` trait): ARM9 item / Pokemon data `item_fun_020213c0`,
`item_param`, `fun_020189d0`; overlay 95 `ability_breakable`, `move_unredirectable`, `move_fun_0689c8dc`,
`move_fun_0689c8cc`, `item_fun_0689c980`, `item_fun_0689c990`, `sick_list_entry`, `fun_0689c8ac`, `fun_0689c8bc`, `fun_0689c91c`, `fun_0689ca1c`, `fun_0689ca3c`, `fun_0689ca5c`,
`fun_0689ca7c`, `fun_0689ca8c`, `fun_0689d064`, `fun_0689d098`, `fun_0689d0c8`, `fun_0689d174`,
`fun_0689d184`; api tail calls into overlay 95 `fun_021c7d34`, `fun_021c7d44`, `fun_021c814c`,
`fun_021c82c4`; and `msg_param` (the event's string buffer, not in the work).

Deviations: a failed HANDEX push (id 0; the original hands the handler a null pointer) gives a
stand-in buffer for `handex(0)`, and `handex_pop(0)` panics (the original executes the null record
and frees relative to address 0). `positions_adjacent(a, _)` panics for `a >= 6` (the original reads
past the 6-record table); likewise `fun_06898cf4(side, kind)` for `side * 14 + kind >= 28` (the
original reads past the layer array). The handlers' `wk + 0x1FEC` scratch (`021c8120`) is a local buffer in the
handler ports, so the host does not leave the expanded IDs there.

## Deviations

- Function and argument types are narrowed to what callers pass (`u8` types, `u16` type pairs
  and move condition params). Table reads index `BATTLE_CALC_RODATA`, so any index inside the
  table reads the same bytes as the original; reads past the end panic instead of reading
  neighbouring memory.
- `EventVarStack::push` returns `Err(StackOverflow)` where the original calls `FUN_02034714`
  (undecompiled diagnostic) and returns. The state change is identical.
- `BattleCalc::init` drops a previously allocated work buffer. The original overwrites the
  pointer, leaking it.
- `SickCont` constructors use plain ORs. The original's per-field clear masks act on a zeroed
  word with non-overlapping fields, so the results are identical (hash-verified).
- Not ported: `FUN_overlay_d_93__021d7e10` (calls into the ARM overlay at `0x0689C9E4`),
  `021d7f88`/`021d7fbc` (four undecompiled ARM9 item/save calls), `021d805c` (tail call to
  `FUN_02020FE4`), `021d8058` (empty), and two unreferenced data-encoded duplicates of
  `SickCont::turn`/`poke` at `0x021D7D78`.

## Inferred names (UNVERIFIED semantics, verified behaviour)

`bpp::{MAX_HP, LEVEL, FORM}`, the `weather` and `sick` ID constants, and personal param 9 as the
"base EXP yield" are named from usage. The code behaves identically whatever they mean.
