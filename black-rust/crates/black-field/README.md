# black-field

Field logic: map matrix, collision, player and NPC movement (fldmmdl), encounters, warps/events.

F0-spec (harness job) added the game-data root: four specs under `tools/rom-diff/specs/black-field/`
(`event_work`, `game_data`, `zonedata`, `btl_setup`, 306 functions in all) and smoke ports in
`event_work.rs` / `game_data.rs`, verified with `tools/check-fn.sh black-field <area> --per-function`.
F0a ported `event_work` and `btl_setup` completely (below); F0 ports the rest of the specced functions. Undecompiled callees go in `hooks` as traits (§4 porting pattern).

| C | Rust |
|---|------|
| `FUN_02014388` (flag byte address) | `event_work::flag_slot`, `event_work::flag_addr` |
| `FUN_020142E8` | `event_work::check_flag` |
| `FUN_02014314` | `event_work::set_flag` |
| `FUN_0201433C` | `event_work::clear_flag` |
| `FUN_020143CC` (var address) | `event_work::var_addr` (plus `var_offset` / `var_get` / `var_set` helpers, not in the ROM) |
| `FUN_0201296C` | `game_data::event_work_ptr` |

## Event flags and vars (layout found in the code)

The EVENT WORK object is the record `GAMEDATA + 0x1AC` points to (`FUN_0201296C`); `FUN_020142D0` returns
its size, 0x3EA bytes, and `FUN_020142D8` zeroes it.

| range | content |
|---|---|
| `0x000..0x27C` | vars, one `u16` each: var id `V >= 0x4000` is at `(V - 0x4000) * 2` (`FUN_020143CC`, no range check); 318 vars fit |
| `0x27C..0x3E8` | flags: bit `id & 7` of byte `0x27C + (id >> 3)`; valid ids are `1..=0xB5F` (0x16C bytes = 2912 flags); id 0 and ids with `id >> 3 >= 0x16C` have no byte |
| `0x3E8` | `u8` marker, set to 1 by `FUN_020143F0` (clears the temp flags `0xAA0..=0xB5F` through `FUN_02014368`), read and cleared by `FUN_0201440C` |

Flag ids `0x4000..0x403F` are not in the object: they are the bits of an 8-byte RAM table at `0x02146268`
(index `(id - 0x4000) / 8`, rounding towards zero, valid `0..8`). The original does not check the low end
of that index: an id `>= 0x8000_4000` addresses memory below the table (`event_work::flag_slot` reports it as
`Sys(negative)`; the checked helpers treat it as "no flag", UNVERIFIED, §8).
Flag `0x96D` is read by the battle setup init (`FUN_02013360`, sets setup flag mask 2). The `field_status`
record (0x14 bytes, `FUN_02014428`) is separate from the event work.

## Specs and what each covers

| spec | addresses | functions | content |
|---|---|---|---|
| `event_work` | `0x020142D0..0x02014488` | 28 | flag/var addressing, check/set/clear, range clear, temp flags, `field_status` accessors |
| `game_data` | `0x02011F24..0x02013090` | 159 | game system record (0x34), `game_event.c` task records, GAMEDATA (0x6B0) accessors, PLAYER records (0x44), global handle table |
| `zonedata` | `0x020139B0..0x020142A4` | 84 | zone cache + 0x30-byte zone record accessors, zone id predicates, LOCATION (0x1C) |
| `btl_setup` | `0x02013090..0x02013A00` | 35 | battle setup (0xF0): init/free, flags, builders |

Not specced (UI/hardware glue that only sequences unported subsystems): the boot/main-loop/shutdown functions
of the game system (`0x02011D40..0x02011EEC`, `FUN_02011FCC`, `FUN_02011F90`, `FUN_02011FB8`, `FUN_02012088`),
the function-pointer dispatchers of `game_event.c`, `FUN_020124F4` / `FUN_02012704` (GAMEDATA constructor and
destructor, ~40 subsystem calls each), the camera/screen glue `0x02012C18..0x02012E7C`, and the script/task
callbacks `0x02014488..0x020147B0`.

## F0a: `event_work` (28/28) and `btl_setup` (35/35)

All 28 `event_work` functions are ported (area hash `10e8720ad2da6d71`): flag range clear, var range clear,
temp flags, the ROM byte table at `0x0209E050` (0x41 bytes embedded), and the `field_status` record
(`event_work::fs_*`, `field_status_new` / `field_status_free`). All 35 specced `btl_setup` functions are
ported as `btl_setup::*` (area hash `02ebd43261225c31`); the table below maps the C names.

Pointers stay `u32` addresses. The ported code reads and writes records through a `hooks::Mem` window
(`hooks::MemEnv`), and the unported callees are `hooks::BtlSetupHooks` / `hooks::HeapHooks` methods named after
their address. Arguments that the original truncates (`ldrh` of a stack argument: the `heap` of the setup
builders; `ldrb`: `d`/`e` of `FUN_02013528`) are truncated the same way.

| C | Rust (`btl_setup::`) |
|---|------|
| `FUN_02013090` / `020130B4` / `020130C4` / `020130DC` | `alloc` / `free` / `init` / `release` |
| `FUN_02013128` / `0201313C` / `02013144` / `02013158` | `copy_to_party` / `party` / `or_flags` / `test_flags` |
| `FUN_02013160` | `read_from_save` |
| `FUN_0201320C` / `02013220` | `init_config_alt` / `init_config` |
| `FUN_02013234` / `02013270` / `02013284` | `new_trainer` / `free_trainer` / `load_trainer` |
| `FUN_0201333C` / `02013360` | `init_party0` / `init_from_game` |
| `FUN_0201347C` / `020134F4` / `02013528` | `setup_with_party` / `setup_kind` / `setup_multi` |
| `FUN_02013610` / `02013644` / `02013678` / `020136AC` | `setup_single_0` .. `setup_single_3` |
| `FUN_020136E0` / `02013734` | `setup_double` / `setup_triple` |
| `FUN_02013790` / `020137C0` / `02013864` / `02013894` | `setup_multi_0` .. `setup_multi_3` |
| `FUN_020137F0` / `02013810` | `setup_multi_b` / `setup_multi_c` |
| `FUN_020138C4` / `02013930` / `0201394C` / `02013980` | `build_all` / `free_all` / `start_battle_ids` / `alloc_block` |

Not ported: `FUN_020135C0` (inside the `btl_setup` range, between `FUN_02013528` and `FUN_02013610`, a variant of
`FUN_0201347C` that ends with `+0 = 4`, `+4 = 0`) is not in the spec.

UNVERIFIED: `setup_multi` copies a local 16-byte config whose last two bytes the original never writes
(uninitialised stack); the port uses 0 and the ROM check agrees. `event_work::table_0209e050` panics past index
`0x40`; `clear_vars` panics when the range leaves the object (the original overwrites what follows).

Deliberate deviations: none yet (`var_get` / `var_set` are range-checked helpers that the original lacks).

## F0b: `game_data` (159/159) and `zonedata` (84/84 ported)

`game_data` (area hash `1dc628979181c9bb`): every spec function is ported in `game_data.rs` (game system record
`sys_*`, EVENT records `ev_*`, GAMEDATA accessors `gd_*`, PLAYER records `pl_*`, the handle table functions).
Records are read through `hooks::MemEnv` (`Mem` now supports several windows) and the unported callees are
`hooks::GameDataHooks` methods named after their address. UNVERIFIED: `pl_set_18_from_table` index past the 17
embedded halfwords returns 0 (the original reads whatever follows the ROM table).

`zonedata` is ported in `zonedata.rs` (`hooks::ZoneHooks`, ROM tables embedded). Its fndiff test is `#[ignore]`d:
`zonedata.spec` line 85 uses `range -1 20` and `black_fndiff::num` cannot parse negative numbers, so the Rust suite
cannot load the spec (BLOCKED: harness). With `-1` replaced by `0` in a scratch copy all 84 functions match; no
hash is recorded yet. The zone functions assume the zone cache exists (the original dereferences null, this port
panics at the `Mem` window).

## F2a-spec: fieldmap, map matrix, fieldmap_func, land_data_patch (specs + smoke ports)

Five spec areas under `tools/rom-diff/specs/black-field/` (set `field`), 205 functions, every one with a block whose
inputs reach every instruction of the function (ROM-side coverage, see the table) and whose stub list is the closure
of the area (the stubs of the function's callees and of the callees of all its in-area descendants), so the areas
can be ported in any order without spec edits. Each spec header documents the records it uses.

| area | original | range | specced | coverage | smoke ports (hash) | smoke area hash (F2a: below) |
|---|---|---|---|---|---|---|
| `map_matrix` | `map_matrix.c` + `map_replace.c` (ov10) | `0x02159B48..0x0215A07A` | 33 | 569/569 | `02159BC4` `zone_at` (b5615728646bbd48), `02159D68` `block_of` (a09289ab0a748193) | ba958fe99b83be98 |
| `fieldmap` | `fieldmap.c` part 1 (ov21) | `0x02187F00..0x02189BD0` | 101 | 2313/2313 | `02189B04` `is_hold_zone` (8ff8981a934eb2dd) | c7affe5c7675f21b |
| `fieldmap_event` | `fieldmap.c` part 2 (ov21) | `0x02189BD0..0x0218B752` | 52 | 2938/2938 | `0218AE48` `front_position` (dbde03b23eab1ea5) | 754132cfaf4ee687 |
| `fieldmap_func` | `fieldmap_func.c` (ov21) | `0x021BB890..0x021BBCB8` | 13 | 454/462 | `021BBBD8` `ring_take` (9e53e76363c37cdd) | 61a0e7e0c0bfa557 |
| `land_data_patch` | `land_data_patch.c` (ov21) | `0x021C939C..0x021C94BC` | 6 | 131/131 | `021C94AC` `attr_section` (2cd30edea097d968) | 44fb04626e38522e |

Coverage = instructions of the function that the spec's inputs execute on the original (control-flow walk of the
function, jump tables decoded). The 8 instructions `0x021BBAC8..0x021BBAD6` of `021BBA70` are unreachable under the
spec model: the second assert can only be reached after the first one, and the assert `02034714` is a `noreturn`
stub (in the game it returns when no assert hook is installed, and the function then reads through a null pointer).

Function list: the F1 Ghidra index counts 222 functions in these ranges; the real set is 226 entries (Ghidra misses
`02159BF4`, `021880C8`, `02188AD4`, `02188C60`/`C64`/`C70`/`CE0`, `02189108`, `02189140`, `02189758`..`021897BC`, `02189810`,
`021899F0`, `02189A08`, `02189A38`/`A48`/`A60`/`A68`, `0218A0E4`, `0218B32C`, found from bl/blx targets, pointer tables and
calls from other overlays) and lists 18 fragments of other functions (shared tails, switch cases) as functions.
Of the 226: 205 specced, 17 render-only, 4 veneers.

Render-only (G8; graphics / hardware calls without FIELDMAP logic, callers stub them by address): `02188D5C` (2D/3D mode
setup + DISPCNT write), `02188DB0` / `02188DF0` (camera matrix + model draw), `02188E38` (3D swap/flush), `02188E50` (display
register init), `02188EE4` (camera / light / render object creation), `02189084` / `021890AC` (draw dispatch through the
tables 0x021D35D8 / 0x021D35B4), `021890C8` (render object teardown), `02189108` (VBlank task), `02189140` (render callback),
`02189154` (edge marking / BG setup, 0x04000060), `021891C0` (light / fog object), `021897DC` (map draw calls), `02189810`
(main 3D draw, NNS matrix stack), `02189998` / `021899C4` (sub-screen blend). Veneers skipped: `02159BBC` (-> Heap_Free),
`02189B28` (-> `02013B50`), `0218AECC` (-> `021626B8`), `02188DA0` (`0203FC28(0x021D35F4)`).

### Records (offsets in bytes; full tables in the spec headers)

* MAP MATRIX (0x152C, `map_matrix.c` 0x4B): `+0 u16` heap, `+2 u16` default zone, `+4 u32` matrix id (ARC 9), `+8 u16`
  width, `+0xA u16` height (blocks, each <= 30, w*h <= 900), `+0xC u16` layout flag, `+0x10 u32` cells, `+0x14 u16 zone[900]`,
  `+0x71C u32 land[900]` (map container per cell). World position (fx32) -> block: `v / 4096 / 16 / 32` (rounding toward zero).
* MAP REPLACE (0x2C, `map_replace.c` 0x77): ARC 10 handle, record count, a 13-byte STATE (season, flags from event vars
  0x4031..0x403A) and the current 16-byte replacement RECORD (matrix id, kind, selector, `u16 value[6]`).
* FIELDMAP (0x154, `fieldmap.c` 0x209): heap, gamesys, gamedata, ~60 subsystem handles, the proc state (`+0xC0..+0xCC`),
  zone ids (`+0xD0` setup, `+0xD8` current), the player LOCATION (`+0xD4`), position (`+0xF0`/`+0xF4`), MAP CTRL (`+0x108`),
  a slot (`+0x118`), the draw mode (`+0x12C`). MAP CTRL = ROM 0x021D88F8 (grid) / 0x021D8AD8 (rail): `kind` + setup / frame /
  teardown functions. MAP SETUP table 0x021D3700 (8 x 0x48, by zone kind `02013B50`). PROC table 0x021D36B8 (6 steps x 2).
* EVENT REQUEST (0x40 on the stack, built by `0218A4E0`): heap, gamesys, gamedata, zone events, zone id, fieldmap, player,
  move state, facing, pending script, key masks, trigger direction, position pointer, FLAGS word (bit layout in the spec).
* FIELDMAP FUNC system (0x18) + SLOT (0x18) + FUNC DATA (ROM, create / delete / update / draw callbacks), the 30 x 0x24
  pass-by ring at `object + 0x30E0`.
* LAND DATA PATCH (4 bytes -> ARC 156 file: attribute section of 0x18-byte cells, building section of 16-byte records).

Cross-check with `nds_rom::map` (A6):
* `matrix`: the header matches (`flags` u32 (the code reads only the low u16), `width`/`height` u16), but for `flags == 1`
  the code (`02159D04`) reads two consecutive PLANES (`land[w*h]`, then `zone[w*h]` stored as u16), not interleaved
  `(primary, secondary)` pairs as `MapMatrix::parse` does: in matrix 0 the planar reading gives runs of consecutive land ids
  (78, 79, 80) with one zone id (355); the interleaved reading pairs neighbouring cells. So `cells` of the two `flags == 1`
  matrices (0 and 175) were wrong in `nds_rom`. **Fixed in F2a:** `MapMatrix::parse` / `to_bytes` now read and write the two
  planes (`cells` = land ids, `cells_b` = zone ids, `MapMatrix::zone_at`); `nds-rom/tests/map_rom.rs`
  `matrix_planes_match_the_game` checks it on the ROM (every overworld block of matrix 0 holds the model `mapXX_YY` of its own
  column and row; Nuvema Town, zone 389, is block (24, 23) on container 0 `map24_23`).
  For `flags == 0` matrices the zone of every block is the load's default zone (the 4th argument of `02159B94`).
* `zone`: zone record `+2` (`unk_02`, 0..273) is the AREA id handed to `0201407C(heap, area, season)` by the FIELDMAP create
  (`02188030`): seasonal areas add the season, ids >= 282 wrap to 0 (282 = ARC 014 map texture sets). The zone id of a block
  comes from the matrix zone plane (`02159BD4`, read by the zone-change check `021893C0`).

### Spec conventions (these five areas)

* The arena is the `work` image (0x212C bytes at 0x02E00000; record zones listed in each header); pointer results of stubs
  are `ptr:work:OFS:1`, function-pointer fields hold FAKE callbacks in 0x02EE0000.. (stubbed there).
* Every out-of-area stub is `force` (overlay addresses collide with battle-overlay ports, e.g. `021BC550`; ARM9 callees are
  ported in other black-field areas); in-area stubs are plain (they deactivate when the callee is ported).
* Event checks: callers that build the EVENT REQUEST on their stack see a stub `0218A4E0` that fills it with masked bytes;
  port `0218A4E0` before or with its consumers (a real consumer with a stubbed builder reads a zero request).
* `global 04000204 4` (EXMEMCNT) in the blocks whose code toggles it; `tools/rom-diff/emu.py` maps the I/O page for
  non-battle sets (F2a-spec harness change). `svflow.py` `ALLOW` gained the MI 16/32-bit clear/copy helpers and `memcmp`.


## F2a: the five areas ported (192 of 205 verified)

| area | ported | verified (in `PORTED`) | area hash | unverified ports (reason below) |
|---|---|---|---|---|
| `map_matrix` | 33 | 33 | `49c1368306aaed86` | |
| `fieldmap` | 101 | 95 | `776501833139141f` | `021880C8`, `021885E8`, `021888C8`, `02188990`, `02188AD4`, `021895E8` (a) |
| `fieldmap_event` | 52 | 45 | `98b331e9367a69db` | `0218AAD0` (a); `0218AED4`, `0218AF90`, `0218B0CC`, `0218B13C`, `0218B4B0`, `0218B53C` (b) |
| `fieldmap_func` | 13 | 13 | `27f7097f727fa8d4` | |
| `land_data_patch` | 6 | 6 | `adfc5aaf9bee54a4` | |

Every callee outside an area is reached through `hooks::FieldEnv::call(addr, args)` (`hooks::CArg`: values, pointers into
the memory window, a caller's local passed as bytes, and `X` for a trailing argument the spec's stub does not list); in-area
callees that are ported are called as Rust functions. The draw side (the 17 render-only functions, the G2/G3/GX calls) is
only ever an address behind `FieldEnv::call` (D3). Stack structs of the original that callees read through a pointer are
`Mem::push_local` windows. ROM tables are embedded (`map_matrix::VAR_TABLE_DATA` / `FLY_TABLE_DATA`,
`fieldmap::MAP_SETUP_DATA` / `CTRL_KIND` / `PROC_TABLE`, `fieldmap_event::ATTRIBUTE_SCRIPTS`, `fieldmap_func::EMPTY_ID`);
`FX_SinCosTable_` is computed (`fieldmap::fx_cos`, equal to all 4096 ROM entries). The test environment is
`src/testenv.rs` (stubs answer `FieldEnv::call`; thunks such as `02008854` -> `020071CC` map to their stubbed target).

Unverified ports (doc `UNVERIFIED port of` on each; not in `PORTED`, so their callers' specs keep their stubs and the callers
are verified):
* (a) the spec's stub of a callee logs registers the callee does not read, and those registers are the function's own entry
  registers (the caller's leftovers, or in the standalone run whatever the previous input left in the emulator), so no port
  reproduces the hash: `02188d5c`, `02189084`, `021890c8` `args=v,v,v,v`, `02188e38 args=p,v,v,v` (render functions that read
  only r0), `021c1c04` / `021c1ad0 args=v,v,v,v`, `0218f134 args=v,p,p,v`, `02010ff4 args=v,p,v,v` (one-argument callees).
* (b) the function hands the address of one of its stack locals to a callee whose stub logs that pointer as a value (`v`), so
  the hash holds the emulator's stack address: `02162564` / `021626A8` (via `02162B68`) from `0218AED4` / `0218AF90`,
  `02162BCC` (via `021626B0`) from `0218B2E8` called by `0218B0CC` / `0218B13C`, `02164240` from `0218B4B0`, `0219E570` from
  `0218B53C`. With `in:12` / `in:8` for those arguments the ports would only need `CArg::In` there.

Where a stub logs an unread register whose value the function itself sets, the port passes that value with a comment (e.g.
`0218AA74`: r3 = the party pointer `0218AA2C`'s epilogue leaves; `0218AB9C`: r3 = the thunk target 0x020071CD; `021881D8`:
`021BC388`'s r2, `021C1AF4`'s r1; `02188758`: `0217F914`'s r1). `02189A38` passes zeros for r1..r3 (its entry registers, 0 in
every harness run; the callee ignores them).

Deviations: none in behaviour. UNVERIFIED (inferred, not checked): the meaning of most callees (named by address), the
attribute script predicates, the fly-point and var-table field names, `ATTRIBUTE_SCRIPTS` roles.
