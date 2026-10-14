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

## B16 integration (`src/integration`, test only)

`integration::tests::b16_single_battle` runs one wild battle end to end through the ports: a
level-5 Snivy (player, scripted: move slot 0 every turn) against a level-5 Patrat (the AI), built
from the ROM data (`black-data`, `black-pokemon`), via `server1::proc_init` / `proc_main` (until it
reports the end) / `proc_exit`. It checks that HP never exceeds the maximum (unsigned, so never
negative), that exactly one side ends fainted, and that every command frame the server sends
(adapter message 8) decodes with the ported reader `021cd5e0`. Result (seeded, deterministic, and
since B19 equal to the original code's under Unicorn, see "B19" below): 3 turns / 122 main frames,
Patrat wins (Snivy: Tackle for 5 / 5 / 5; Patrat: Leer, then Tackle for 10 twice), 4 frames / 72
commands. (Before B19 the integration had Snivy win: the foe's Leer targeted the foe itself and two
coherence gaps changed the turn ends.)

**Dispatch.** `Env` is the integration `FlowEnv`: `env.call(addr)` goes to `dispatch::call`, which
routes by address range to the port (`server1`-`3` with `Mem` windows, `server_side`, the flow
areas through `flowcase` = the areas' differential-test `call` tables driven by their spec
blocks, `client1`-`5`, `battle_mon`, `calc` / `calc_target` / `calc_action`, `event_factor` +
the ported handlers, `field`, `sick_cont`, `poke_type`, adapter, all of overlay 96 = the ai1 ports) or
to an explicit host model: heap / GameProc / MI memory (`arm9`), `poke_tool` / personal / waza /
archives (the `black-pokemon` / `black-data` ports), sound, the headless battle view (every start
returns, every "finished?" answers yes, the move menu answers move slot 0), the battle record
builder (`recb`, 0x021EE8B4-0x021EEA2F, not ported), the Pass Power multipliers (inactive), the
overlay-231 IR probe (genuine cartridge), key input (A pressed), the system Mersenne Twister.
An address with neither panics with `B16 gap: no port or host model for callee …`.

**Original code in an interpreter.** Unported Thumb glue runs in a minimal ARMv5T Thumb
interpreter (`thumb.rs`): overlay-95 routines behind the `bx pc` veneers (the side-effect
adder / remover are the ported `handlers::effects` ones), vm.c, btl_setup.c, the held-item and
move-handler factor registration (`OV93_THUMB`) and the turn-end side-effect walks. Overlay 96
runs only as the ai1 ports (B17); `Env::ai_interp` runs it as original Thumb code instead (all but
create / start / chosen move / chosen target) for the cross-check below.

**AI (B17).** The ai1 ports pass every pointer the original passes (`Arg::PAt`: the VM to the vm.c
routines, BattleMon pointers to their accessors, M / pokecon / flow / party to the server and flow
callees, the freed blocks to `020307B0`; hash-neutral, ai1 still `svflow` clean against the
recorded table). In `b16_single_battle` (a wild battle: AI mask 0, no scripts) the opponent's
decisions come from the ports `ai_create` / `ai_start` / `ai_think` (`think_single`, the best-slot
pick) / chosen move / chosen target; the battle is unchanged. `b17_ai_scripts_match_original`
forces the AI client's script mask (`Env::ai_mask`, answered by `021b9fbc`) so the trainer-AI
scripts run (the interpreted vm.c step calls the ported command handlers; 32 distinct handlers) and
checks that 17 masks (every single bit 0-11 and 13, and combinations) give the same HP trace and
command frames as the original overlay-96 code in the interpreter. B18 closed the gaps of bits 0 / 1 /
4 (`021c7e14` -> core4 `021c1e74` -> core6 `021c728c`: a flow case gives a caller-local buffer argument
a temporary heap block, `Src::Local`, so a callee the port passes the pointer on to finds it) and 11
(core2 `021bd708` now lends the work to `021d8244`; `020061E4` play-SE host model; the wild foe flees,
so with a forced mask the battle may end with neither side down). Left: bit 12 (its script, ARC 0xAB
member 12, is 12 bytes and vm.c reads the byte after its end, in the original too: the answer depends
on the neighbouring heap block), bits 14 / 15 (ARC 0xAB has 14 members, no script for them).

**Coherence (lending).** Ports hold objects as Rust images (the flow work, `S`, client objects,
the command queue, the 0x021F3B20 globals). A call lends them (`callw` / `callwr` / `callh` /
`callr` / `call_l` / `Lend::{new, new_ref, wc, c, h, r, hv, wh, whr}`): `Env::dispatch` flushes the
lent images into the object table for the call and reloads the writable ones afterwards; an
object a frame holds without lending it panics on access (`… is held by a port frame and was not
lent`). Read-only holders flush only the bytes they changed since their last flush (a callee's
writes survive; each is reported as a note); the snapshots are keyed by the holder's frame depth
and the image's place in its block and dropped when the frame returns (B17: they were keyed by the
image's host address, so an image allocated where a freed one had been inherited its snapshot and
the outcome could depend on the allocator; suspected cause of the rare `b16_single_battle` failure
under the full parallel suite, "client object … already held by an outer port frame", not proven).
A lent slice that matches two held blocks (a stale holder) panics. Flow cases store back only the bytes a port changed
(arguments overlap, e.g. the 0x021F3B20 arrays) and treat regions / buffers inside the work as
parts of the work. `Arg::{PBuf, PHost, PAt}` carry the pointer values the ports used to drop; since
B18 the stubs check them where the callee's spec declares a buffer / BattleMon (`ptr` stub
arguments, see `tools/rom-diff/svflow_specs/README.md`), elsewhere they see `P`. `FlowEnv::peek` lets a port read memory its test model placed
inside its own image (`S` slot table, adapter reply blocks).

**Gaps found (see the B16 report for the full list).** High (B18 fixed the ~120 call sites that
passed a bare `P` where the callee's spec argument is a buffer / BattleMon: the harness now rejects
them); ports hold records that live in the
flow work as separate slices (D / E at work + 0xD04 / 0xD18, the damage records, the per-turn
struct) — `SvFlowWork::pull` / `push` sync them in `use_move` / `build_damage` / `build_records`
only. Medium: read-only work holders whose callees write the work (`021bbb60`, `021bf3d8`,
`021bf83c`, `021bf8d4`, `021ca8f8`, …), client callback state words that alias the client object
(`run` merges them), test-shaped data models (`collect_slot_records`, `slot_party`,
`main_step` code 9: the data was assumed inside the holder's image). Low: unported callees
modelled here (record builder, Pass Power, IR probe, `020181F4`, `02018480`, `020185C4` wiring).

## B19 Unicorn differential (`src/integration/b19.rs`, test only)

`integration::b19::b19_battles_match_rom` runs whole battles through the ports (the B16
integration) and compares a trace of each with the trace of the same battle run entirely in the
original code under Unicorn (`tools/rom-diff/battle_diff.py`). The trace (format in the module doc
of `b19.rs`): the hash of the two parties as built, the frame counts of proc_init / proc_main /
proc_exit, after every proc_main frame the HP / max HP of every battle Pokemon of the server's
pokecon and the battle RNG state (`0x021F6368`), the player's moves and party choices, every
command frame the server sent (adapter message 8, with the main frame it was sent in) and its
decoded commands, the number of system Mersenne Twister calls. The test compares the FNV-1a hash
of the Rust trace with `RECORDED` (written from the ROM side), so the gate needs no emulator; it
runs in ~1 s (debug) per battle.

```sh
tools/check-battle-diff.sh             # both sides, line diff on a mismatch (~12 s)
tools/check-battle-diff.sh switch      # one battle
tools/check-battle-diff.sh --record    # write RECORDED (only when every trace matches)
/tmp/blk/venv/bin/python -I tools/rom-diff/battle_diff.py b16 --arm9-funcs   # ROM side only
```

Outputs: `$BLK_OUT/b19/<name>.trace` (ROM) and `<name>.rust.trace` (ports). Diagnostics:
`B19_WATCH=021d78ac,...` logs the ROM side's calls of those addresses (frame, args, return
addresses); `B19_TRACE=K` prints every dispatched call of main frame K on the Rust side.

**Battles** (`BATTLES`, the same in both files): `b16` (the B16 battle), `trainer_ai` (the same,
AI script mask 7: the trainer-AI scripts run in overlay 96 / vm.c), `status` (Snivy uses Leer twice,
then Tackle), `switch` (Snivy + Tepig: Snivy faints in turn 3, the player answers "yes" and picks
party slot 1, Tepig wins). All four traces are equal.

**Emulated side.** ARM9 main (autoload blocks to ITCM / DTCM / MAINEX / VRAM) and overlays 93 / 94 /
95 / 96 / 231 at their link addresses; boot: the gfl std block (`0203EF5C`) and the system Mersenne
Twister seeded (`0203F034`, the seed of `Env::mt`), the personal-data archive handle (`0201AD88`).
Host hooks only for the system boundary, each the counterpart of the integration's host model of
that address: heap create / alloc (zeroed) / free / destroy, GameProc work, the file system below
the gfl archive code (open an ARC by id, FS_SeekFile / FS_ReadFile / FS_CloseFile over
`black-rom/files`), the save-data block table (`020319A4`), OS_GetTick (`Env::tick`), the RTC seed of
the battle RNG (`0203F258`, `Env::seed`), key input, the overlay loader, sound, the Pokedex and
friendship updates (save / field data, `0200CFA0`, `0201B7A0`), the IR probe, the headless battle view
and overlay 94 (the scripted player: move menu, party list `021e9a78`, yes / no `021eac54`). The
division / square-root coprocessor is emulated; any other I/O or unmapped access fails the run.
Everything else runs as original code — the battle overlays, btl_setup.c, vm.c, and about 170
ARM9 functions (poke_tool, pokeparty, personal, waza, strbuf, the archive code, MI / FX helpers):
where the integration has a host model (`arm9.rs`), the ROM side runs the original.

**Mismatches found and fixed (B19).**
- `events` `021cb174` / `021cb1a4` (`mon_of_main_a` / `_b`): r1 / r2 of the call pass through to
  `021b8c50(main, pos, slot)` / `021b8cb0(main, pos)`; the port dropped them (a foe's single-target
  move, Leer, resolved to the foe itself). The callers (`021ca8f8`, `021caa58`, `021cacf4`) now pass
  the position (`021b8d8c`) and slot (0 / 1 / 2, or `rand(2)`). The events spec declares only r0
  for these stubs: `testkit::UNDECLARED_PASS_THROUGH` drops the extra words for the stub log (hashes
  unchanged); declaring them in `events.spec` (`args=v,v,v` / `v,v`) is open.
- `core2` `021bec58` (`apply_move_category`): the struct `work[0x4A8]` points at lies in the work
  (work + 0xCFC); the port kept its image separate, so `021c091c` / `021bfbd0` read stale positions
  for command 0x30. Now pushed / pulled around the calls (`SvFlowWork::push` / `pull`, no-ops in the
  differential test).
- integration `flowcase` merge: a port that changed a work byte, lent the work, and changed it back
  (the turn-end state `work[0x77E]`) got its intermediate value back from the table: the second
  turn end did not run. `Env::work_sync` (the image the last writable lend handed back) is the merge
  base now.
- `server_side` `021bae60`, `client3` `021d0c78` (`switch_requests`): the adapter reply / request
  list is the channel object's memory, read through `peek` (the ports indexed their own image).
- `client2` / `client3` / `client4`: the party list `021e9a78` and the switch answers (`021d8394`,
  `021d8434`, `021d83c4`) get the pointers the original passes and the client object is lent
  (hash-neutral: `p` stub arguments).
- integration: models / wiring for what the new battles reach (`021d7ff8` prize money through the
  `calc` port, `0200C274` interpreted, `020319A4`, `0201B7A0`, requests 5 / 6 `021d0d38` / `021d0d44`
  / `021d0d50`, the party list and yes / no answers).

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
