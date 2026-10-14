# black-pokemon

Pokémon data and formulas: species/personal data, stats, EXP, natures, learnsets, evolution, PokéParam (poke_tool), moves (waza_tool).

`poke_tool` (P1a/P1b): the Pokémon record layout, block cipher and field-ID get/put, verified per
function against the original machine code (`tools/check-fn.sh black-pokemon poke_tool --per-function`,
spec `tools/rom-diff/specs/black-pokemon/poke_tool.spec`). Undecompiled callees (personal data, growth
table, move data, string buffers, memcpy) are the `hooks::PokeToolHooks` trait; the differential test
stubs them identically on both sides, so their arguments are checked but not their behaviour.

| C | Rust |
|---|------|
| `FUN_02019A50` | `poke_tool::crypt` |
| `FUN_02019C38` | `poke_tool::block_offset` (+ `SHUFFLE`, table at 0x0209E3A4) |
| `FUN_02017DBC` / `FUN_02017DE4` | `poke_tool::decrypt_on_access` / `reencrypt` |
| `FUN_02018D34` / `FUN_02018D8C` | `poke_tool::box_enter` / `box_exit` |
| `FUN_02018D10` / `FUN_02018D6C` | `poke_tool::party_enter` / `party_exit` |
| `FUN_02018E34` / `FUN_02019388` | `poke_tool::box_get` / `box_put` (178 field IDs, `poke_tool::id`) |
| `FUN_02018DB4` / `FUN_02019308` | `poke_tool::party_get` / `party_put` |
| `FUN_02017E88` / `FUN_02017E64` | `poke_tool::box_get_value` / `box_set_value` |
| `FUN_02017E1C` / `FUN_02017E40` | `poke_tool::party_get_value` / `party_set_value` |
| `FUN_02019ABC` | `poke_tool::is_nicknamed` |
| `FUN_02019BBC` | `poke_tool::plate_type` |
| `FUN_02019CE0` | `poke_tool::copy_pokemon` |

Deliberate deviations:

- Records are byte slices (`&mut [u8]`, at least 0x88 bytes for box functions and 0xDC for party
  functions) with the C's offsets; there are no typed structs yet.
- Pointers into the record that the original passes to callees are byte offsets (`mon_ofs`) in the hooks;
  other pointer arguments (string buffers, output arrays) are opaque `u32` handles.
- `FUN_02018DE2` (0x88 status read) is not a function: Ghidra split a case body of `FUN_02018DB4` off.
- Field-ID names in `poke_tool::id` are inferred from the Gen-4 `MON_DATA_*` layout; offsets and numbers
  are ROM-verified.

## P-port: create, personal, growth

Each area is verified per function with `tools/check-fn.sh black-pokemon <area> --per-function` (specs in
`tools/rom-diff/specs/black-pokemon/`). Callees outside the area come in through
`hooks::{CreateHooks, InitHooks, PersonalHooks, GrowthHooks}`; the differential test stubs them
identically on both sides.

| area | recorded (in `PORTED`) | ported, not recorded |
|---|---|---|
| `create` | `FUN_020173F0` `new_pokemon`, `FUN_02017430` `new_pokemon8`, `FUN_02017528` `clear_pokemon`, `FUN_02017550` `clear_box_pokemon`, `FUN_0201761C` `init_core`, `FUN_02017D30` `decrypt_party`, `FUN_02017D70` `encrypt_party`, `FUN_02018328` `next_level_up_move`, `FUN_020185C4` `recalc` `FUN_0201756C` `init_pokemon`, `FUN_02017588` `init_pokemon_full`, `FUN_02017600` `core_init_default`, `FUN_0201747C` `copy_pokemon_new` |
| `personal` | `FUN_0201AE1C` `field`, `FUN_0201AFD4` `form_record_index`, `FUN_0201AD88` `open_archive_once`, `FUN_02019C54` `load_cached_record`, `FUN_02019C7C` `ability_count`, `FUN_02019C90` `cached_field`, `FUN_02019B04` `nature_modifier` (table `NATURE_TABLE` = 25 x 5 signed bytes at 0x0209E324), `FUN_020185A0` `nature_of`, `FUN_0201ADE4` `alloc_record`, `FUN_0201ADA4` `read_record`, `FUN_02018A7C` `recalc_stats` | |
| `growth` | `FUN_02019B94` `load_curve`, `FUN_02019BA4` `exp_for_level`, `FUN_0201850C` `level_from_exp`, `FUN_02018584` `exp_for_species_level` | |

Wrappers whose ported callee is stubbed in the spec carry the `force` flag on that stub line (the ROM side
keeps the stub). `fn 02018a7c` is specced with natures limited to 0..=24 (a stored nature is always below 25),
since the table lookup would otherwise read unrelated ROM data past the 125-byte table. Private
helpers inlined from unlisted callees: `FUN_02017F08` (`create::gender_of`), `FUN_02017ED8`
(`create::is_shiny`), `FUN_02018438` (`create::level_of`), `FUN_0201808C`/`FUN_020181FC` (private).

Data and callees outside the crate come in through `hooks::{CreateHooks, PersonalHooks, GrowthHooks}`
(heap allocation, the personal-data archive reader, the ARC 17 growth tables), so the crate has no
dependency on `nds-rom` or `black-data` (D6).

## P-spec2: waza (move data), evolution

Same pattern, new modules `waza.rs` and `evolution.rs`; their hook traits (`WazaHooks`, `EvolutionHooks`) live
in the modules, not in `hooks.rs`. The Pokemon getters used by the evolution routines run as real code
(`poke_tool`); the evolution specs use really encrypted records (the `poke_tool.spec` recipe).

| area | ported (smoke) | specified only |
|---|---|---|
| `waza` | `FUN_0201BD28` `waza::move_param` (record cache), `FUN_0201BDB4` `record_param` (32-parameter getter), `FUN_0201BF44` `flag_test`, `FUN_0201C158` `max_pp`, `FUN_0201ADD0` `read_learnset` | `0201BEB8`, `0201BF58`, `0201C020`, `0201C0BC`, `0201C1F4`, `0201C284`, wrappers `0201C1BC`/`C1C8`/`C1D8`/`C1E8`/`C314`, `0201C184`, `0201C3C0`, `0201C408`, PP/slot helpers `0201B920`, `B970`, `B9F0`, `BA34`, learning helpers `02017FB0`, `02018094`, `020180FC`, `020181A0`, `020181FC`, `02018240`, `020182A8` (not specified: `0201C328`, the cache constructor) |
| `evolution` | `FUN_0201B2B0` `evolution::evolution_check` (all 29 methods, modes 0-3), `FUN_0201B764` `load_record`, `FUN_0201B774` `knows_move` | `0201B70C`, `0201B718`, friendship `0201B7A0`, `B7B4`, `B7EC`, `B818`, `B850` |

Deliberate deviations: the evolution record block and the move-record cache are Rust values handed back by the
hooks / owned by the caller instead of heap addresses; `move_param` takes the cache as `Option<&mut MoveCache>`
(the global at 0x02146530) and the static record (0x02146534) as `scratch`. `FUN_0201B3B8` / `FUN_0201B406`
that Ghidra lists are the record-loop head and a case label inside `FUN_0201B2B0`, not functions.
`FUN_0201BF58` returns the caller's `r3` low halfword when record byte 0xB is 0 (an uninitialised read in the
original; specified, not ported).


## P4: waza remainder, icon

`waza` gained the cache constructor `FUN_0201C328` (`waza::cache_create`; the allocator is the new
`WazaHooks::alloc`, specced with a `seqptr` bump-allocator stub) and is complete for `waza_tool.c`
(0x0201BD28-0x0201C41C, 31 functions). `FUN_0201ADD0` (`read_learnset`) now fills a slice through
`WazaHooks::load_learnset(index, dst)`, so a production hook can copy the ARC 18 member from the assets (D4).

`icon` (new, `icon.rs`): the Pokemon icon helpers at 0x0201BA68-0x0201BD28 (between `shinka_check.c` and
`waza_tool.c`; T1 had them as waza_tool, and Ghidra's "`FUN_0201BC40`" was a merge with `FUN_0201BD28`).

| C | Rust |
|---|------|
| `FUN_0201BC3C` | `icon::palette_member` (ARC 7 member 0) |
| `FUN_0201BC40` / `FUN_0201BC78` | `icon::cell_member` (main / sub DISPCNT) |
| `FUN_0201BCB4` / `FUN_0201BCEC` | `icon::anim_member` (main / sub DISPCNT) |
| `FUN_0201BABC` | `icon::graphics_member` |
| `FUN_0201BB5C` | `icon::palette_of` (+ `palette_index`) |
| `FUN_0201BA68` / `FUN_0201BBE8` | `icon::mon_graphics_member` / `mon_palette` |

Deliberate deviations: the DISPCNT helpers take the register value as an argument (the original reads
0x04000000 / 0x04001000 itself); the 712-byte palette table (main 0x0209E474) is a slice argument (asset data,
not embedded).

## P6: PID, gender, shininess, ability slot (`pid.rs`)

Eleven `poke_tool.c` functions (0x02017C50-0x02018790, 0x0201AF6C), new `PidHooks` trait (personal record/field,
ability count, RNG; supertrait `PokeToolHooks`). The nature is a stored field (id 0x70, `personal::nature_of`;
`rand(25)` in `create::init_core`), not derived from the PID, so nothing new is needed for it.

| C | Rust |
|---|------|
| `FUN_02017ED8` | `pid::is_shiny` (re-export of `create::is_shiny`) |
| `FUN_0201AF6C` | `pid::is_fixed_gender` (ratio 0 / 0xFE / 0xFF) |
| `FUN_02017F50` | `pid::gender_from_ratio` |
| `FUN_02017EB4` / `FUN_02017F08` | `pid::is_shiny_mon` / `pid::gender_of` |
| `FUN_020185F8` | `pid::gender_threshold` (low PID byte for a ratio) |
| `FUN_0201862C` | `pid::pid_without_rng` |
| `FUN_020186C8` | `pid::random_pid` |
| `FUN_02017C50` / `FUN_02017CC8` | `pid::update_ability_slot` / `pid::update_gender` |
| `FUN_020179B8` | `pid::set_form` |

`FUN_02017F50` is called from the wrappers through `PokeToolHooks::gender` (as `box_put` id 0x6E does). Verified
by `tools/check-fn.sh black-pokemon pid` (11/11, da143d1b836cda45).

## P3: exp

`exp` (new, `exp.rs`): `FUN_0201854C` `exp::exp_at_level` (EXP a record needs for its current level; species/form/level
read with the box getter, then `FUN_02018584`), hash 1a566a4812a0b332. Growth lookups are `growth`, level storage
is `create::recalc`, level-up learnset scan is `create::next_level_up_move`. The battle EXP formula is not in this
crate: `black-battle` `calc::exp_yield` (0x021D7EB4), `server_flow::events::exp_scale` (0x021CB55C),
`exp_distribute` (0x021CB2D4), `BattleMon::gain_exp` (0x021D69B8), `exp_for_max_level` (0x021D6B6C).

## P7: egg generation (`egg.rs`)

The Pokemon-side logic of the day care: overlay 21, `0x021C38F4-0x021C4570`, 39 functions (the driver
`FUN_021C38F4` and the steps it runs on a 0x8C-byte work struct, `egg::Egg`, plus the helpers
`FUN_021C42F4` add-move, `FUN_021C447C` TM compatibility, `FUN_021C4504` baby form, `FUN_021C452C` ability slot).
Verified per function in the `field` set (`tools/check-fn.sh black-pokemon egg --per-function`, spec
`tools/rom-diff/specs/black-pokemon/egg.spec`); every callee outside the area (Pokemon accessors, RNG,
personal records, archive readers, message and string routines, item helpers) is `egg::EggHooks` and a stub.

Steps, in driver order: parent order (male first, Ditto second), baby species (ARC 20), Nidoran /
Volbeat / Illumise coin flips, Manaphy -> Phione, incense babies, forms (Shellos, Burmy, Basculin), nature
(Everstone), ability slot (hidden-ability roll), egg cycles (personal field 0x15, stored as friendship),
Power-item IVs then random IVs up to three, OT id, PID (Masuda: up to five re-rolls), moves (level-1
learnset, egg moves from ARC 123 = the first parent's, moves both parents know, TM moves, Volt Tackle),
then `build_mon` (`FUN_021C4340`) creates the egg Pokemon and sets the egg flag (id 0x4C).

Deviations: the parents and the work struct are `[u32; 2]` / `[u32; 35]` (the work struct is a stack local
of the driver); Pokemon are `u32` handles behind the hooks. `EggHooks::learnset_end` is 0xFFFF in the game;
the differential test moves it because a random stub buffer never ends. `FUN_021C3898` (the day care
struct reader that calls the driver) and the rest of `sodateya.c` are F13. Hatching (the step counter
and the hatch scene) is in no decompiled overlay set (main, 10, 11, 18, 20, 21, battle set): not found.

## F13: day care (`daycare.rs`)

`sodateya.c` (overlay 21, `0x021C31D4-0x021C38F4`, 29 functions) plus the block getter `FUN_0200BDB0` (main ARM9).
Same crate as the egg generator (P7): the day care is Pokemon logic (EXP, moves, compatibility, egg roll) over a
small save block, not field state; the field only owns the object pointer (`FUN_02188D50(field map)`) and calls
`daycare::step` from its step hook (`hatch::egg_step`'s `daycare_step` hook). Verified per function in the
`field` set: `tools/check-fn.sh black-pokemon daycare --per-function` (spec
`tools/rom-diff/specs/black-pokemon/daycare.spec`), **30/30, area hash a72e525e966a7232**; coverage 100% of the
instructions of every function (nops / literal pools aside), `--strict-callees` and `--order-check` clean.

The save block (block 50, 0x1CC bytes: two 0xE4-byte slots `+0` occupied / `+4` party Pokemon / `+0xE0` EXP
gained, `+0x1C8` egg flag) and its accessors `0x0200BC10-0x0200BDA0` are `black_storage::daycare` (S5b).
black-storage depends on this crate, so the overlay code reaches the block through `DaycareHooks::save_*`
(one per accessor, named after the `black_storage::daycare::Daycare` methods: `save_mon` = `mon` / `FUN_0200BC2C`,
`save_put` = `put`, `save_take` = `take`, `save_count`, `save_occupied`, `save_exp`, `save_swap`, `save_add_exp`,
`save_egg` / `save_set_egg` / `save_clear_egg`); the game's hook impl calls those methods. The object
(`FUN_021C31D4`, 0x10 bytes) is `daycare::DaycareObj`: `+0` u16 heap, `+4` field map, `+8` the save block handle,
`+0xC` u16 egg-roll counter.

| C | Rust |
|---|------|
| `FUN_0200BDB0` | `save_block` (`FUN_020071CC(save data, 0x32)`) |
| `FUN_021C31D4` / `31FC` | `create` / `delete` |
| `FUN_021C3204` / `3248` / `3250` | `deposit` / `restore_pp` (veneer to `0201B9F0`) / `reset_form` (Shaymin -> form 0) |
| `FUN_021C3270` / `32BC` / `36B0` | `withdraw` / `register_dex` / `compact` |
| `FUN_021C3530` / `360C` / `3670` | `apply_exp` (day care EXP, capped at level 100, level-up moves) / `learn_move` / `knows_move` |
| `FUN_021C3378` / `385C` | `step` (the field step hook's day care step: 1 EXP per step) / `egg_roll` (every 16 steps: `rand(1600) < compatibility`) |
| `FUN_021C34C4` / `36EC` / `37F8` | `compatibility` / `compat_pair` (egg groups, Ditto, genders) / `compat_score` (20 / 50 / 70) |
| `FUN_021C32F4` / `335C` / `3898` | `take_egg` / `refuse_egg` / `make_egg` (calls `egg::generate`, place 0xEA62) |
| `FUN_021C33A4` / `33E0` | `set_egg_var` / `clear_egg_var` (event var 0x4139: egg waiting and party not full) |
| `FUN_021C3400` / `340C` / `34F8` | `egg_ready` / `count` / `slot_mon` |
| `FUN_021C341C` / `3490` / `34B4` | `level_now` / `levels_gained` / `fee` ((levels + 1) * 100) |
| `FUN_021C3510` / `38DC` | `sat_add` / `party_changed` |

Deviations: Pokemon, parties, the save block, the field map and game data are `u32` handles. `create` returns the
allocator's handle with the object value. `DaycareHooks::set_work_var` is `*FUN_020143CC(ev, id) = v`. `apply_exp`
stops at the end of its 26-pair learnset buffer where the original would read on (a real learnset always ends inside
it). UNVERIFIED roles (named by address in the hooks): `FUN_0216556C` (overlay 10, called on the egg handover),
`FUN_02007C34(FUN_02012F98(gd), party)` after a deposit.

Mutation spot check (`check-fn.sh --fn`, reverted): 17 of 17 caught (`compat_score` 70 -> 50, egg roll `<=` -> `<`,
learn `<` -> `<=`, EXP `min` -> `max`, saturating -> wrapping add, `push_move` without the u16 mask, `set_egg_var`
`<` -> `<=`, one egg-group pair dropped, `take_egg` free before the party add, `withdraw` compact before the free,
`level_now` empty answer 1 -> 0, `deposit` count `< 2` -> `< 1`, `deposit` into the party index instead of the count,
Shaymin species masked, step adds 2, compact swap arguments swapped, egg parents swapped). The Shaymin mask first
survived (spec gap: the species picks now include 0x101EC / 0x11EC).
