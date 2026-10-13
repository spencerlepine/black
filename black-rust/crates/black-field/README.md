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
| `fieldmap_event` | 52 | 51 (Z22: +6) | `8a3e9c84cd0ed77c` (Z22; F2a `98b331e9367a69db`) | `0218AAD0` (a) |
| `fieldmap_func` | 13 | 13 | `e8889bfebba46d72` (Z22; F2a `27f7097f727fa8d4`) | |
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
  `0218B53C`. With `in:12` / `in:8` for those arguments the ports would only need `CArg::In` there. **Z22: registered** (spec
  `in:12` / `in:8`, and `outm:2` for the two s16 tiles `02164240` moves; the ports pass `CArg::In` / `CArg::Out`, and their
  callers `02189BD0` / `0218A0E4` / `0218A278` call them as Rust functions).

Where a stub logs an unread register whose value the function itself sets, the port passes that value with a comment (e.g.
`0218AA74`: r3 = the party pointer `0218AA2C`'s epilogue leaves; `0218AB9C`: r3 = the thunk target 0x020071CD; `021881D8`:
`021BC388`'s r2, `021C1AF4`'s r1; `02188758`: `0217F914`'s r1). `02189A38` passes zeros for r1..r3 (its entry registers, 0 in
every harness run; the callee ignores them).

Deviations: none in behaviour. UNVERIFIED (inferred, not checked): the meaning of most callees (named by address), the
attribute script predicates, the fly-point and var-table field names, `ATTRIBUTE_SCRIPTS` roles.

## F4a-spec: fldmmdl (MMDL / MMDLSYS) and the head of event_battle.c (specs + smoke ports)

Two spec areas under `tools/rom-diff/specs/black-field/` (set `field`), generated with closure stubs (a block lists the stubs
of its callees and of the callees of all its in-area descendants and the shared fix lines; with every function of the area
treated as ported all blocks run without a fault, stray write or unstubbed callee), so the area can be ported in any order.

| area | original | range | specced | coverage (plain / closure) | smoke ports (hash) | area hash |
|---|---|---|---|---|---|---|
| `fldmmdl` | `fldmmdl.c` (ov10, F1 row 10.38) | `0x0216CC58..0x0216E62C` | 253 | 2754/2754 / 2754/2754 | `0216D4F0` `count` (b52340c0d3950151), `0216D55C` `status` (079615e140f95902), `0216D5F8` `dir_disp` (283f3ed95c1be88c), `0216D67C` `sys_of` (4250f23b5d2a364a), `0216DD1C` `next_in_use` (d7636c31b57a843c), `0216E4B0` `save_slot` (dde00de11a994a2a) | cf1b5fcbd5cf11de |
| `event_battle_head` | `event_battle.c` head (ov10; counted in F1 row 10.38) | `0x0216E62C..0x0216E954` | 7 | 321/321 | `0216E680` `run_callback` (1125c1d4801c0dd1) | c22d9c951e9751cb |

Coverage = instructions of the function body (Thumb walk from the entry, jump tables decoded) that the spec's inputs execute on
the original, with the in-area callees stubbed (plain) or real (closure).

Function list: the F1 index has 246 entries in the range, two of them fragments (`0216D3A4` tail of `0216D38C`, `0216E688`
tail of `0216E680`). Labels, `bl`/`blx` targets of the whole field set and odd code pointers add 18 entries Ghidra misses
(`0216CCCC`, `0216D38C`, `0216D4F8`, `0216D518`, `0216D524`, `0216D530`, `0216D538`, `0216D5A8`, `0216D5B0`, `0216D5FC`,
`0216D678`, `0216D7C4`, `0216D7CC`, `0216D828`, `0216D860`, `0216D8B0`, `0216DA44`, `0216E3C8`) and two false ones (`0216DE22`
inside `0216DDDC`: a data word in main `0x020A4804` decodes as a `bl` to it; `0216E166` inside `0216E160`: ARM-data hit), so the
range holds 262 functions. 2 are veneers (`0216E3D0` -> `0216E3D8`, `0216E42C` -> event_work `check_flag` `020142E8`; callers
stub the target): 260 specced. Render-only functions left out: none. The draw side is reached only through the draw-proc table
(`+0x90`, FAKE callbacks in the specs) and overlay-21 callees (`fldmmdl_blact` `0219C5FC` / `0219C624` / `0219C630`,
`fldmmdl_g3dobj` `0219D6B8` / `0219D6CC`, `fldmmdl_acmd` `02197C4C` / `02197C78`), all `force` stubs; the glue around them
(`0216CD50` .. `0216CD88`, `0216D3B4`) is specced like the rest.

### Records

* MMDLSYS (0x48, `Heap_AllocDebug(heap, 0x48, 1, "fldmmdl.c", 0xC3)` in `0216CC58`): `+0x00 u32` flags (1 draw side on,
  8, 0x10 task system + ARC 49 open, 0x20 all paused), `+0x04 u16` MMDL count, `+0x06 s16` objects in use, `+0x08 u16` heap,
  `+0x0A u16` heap, `+0x0C u16` task priority (0x80), `+0x10` ARC 49 handle, `+0x14` ARC 48 handle (OBJ code data),
  `+0x18` -> `MMDL[count]`, `+0x1C` task block, `+0x20` task system, `+0x24` draw (blact) system, `+0x28`, `+0x2C` -> save
  positions, `+0x30`..`+0x3C` (create arguments), `+0x40` -> u16, `+0x44` -> OBJ code table (0x310 bytes: u16 count, u16
  code[count] padded to a word, 0x1C-byte OBJ DATA records; ARC 58 member).
* MMDL (0x100): `+0x00 u32` status (1 in use, 2 move proc ready, 8 dir locked, 0x800 alias, 0x2000 rail/3D move, ...),
  `+0x04 u32` move bits (2 draw ready, 8 paused, 0x200/0x400 acmd set/done, ...), `+0x08 u16` OBJ id (0xFF = player),
  `+0x0A u16` zone id, `+0x0C u16` OBJ code (0xA2..0xB1 from event vars 0x4020.., 0xB9 = size from the header), `+0x0E u16`
  move code (move-proc table ROM `0x021D5E04`, 47 entries), `+0x10` event type, `+0x12` event flag, `+0x14` event id (0xFFFF =
  alias), `+0x16` header dir, `+0x18` / `+0x1C` display dir / previous, `+0x1A` / `+0x1E` move dir / previous, `+0x20..+0x24`
  params 0..2, `+0x26` acmd code, `+0x28` acmd seq, `+0x2A`, `+0x2C` / `+0x2E` move limits, `+0x30..+0x34` initial grid x/y/z,
  `+0x36..+0x3A` old grid, `+0x3C..+0x40` current grid (s16), `+0x44` VecFx32 position, `+0x50` / `+0x5C` / `+0x68` VecFx32,
  `+0x74`, `+0x78`, `+0x7C` / `+0x7D` u8 size, `+0x7E..+0x80` s8 offsets, `+0x84` task, `+0x88` -> MMDLSYS, `+0x8C` -> move
  procs (init, move, delete, return), `+0x90` -> draw procs (init, draw, delete, push, pop, get; ROM `0x021D770C[OBJ data byte
  3]`), `+0x94` / `+0xA4` / `+0xB4` / `+0xC4` work blocks (0x10 / 0x10 / 0x10 / 0x20), `+0xE4` OBJ DATA (0x1C). Grid =
  `(fx >> 4) / 4096` (16 units per tile), position = `grid << 16` (+ 0x8000 for x/z: tile centre).
* HEADER (0x24) = the 36-byte NPC record of ARC 125: `+0` OBJ id, `+2` OBJ code, `+4` move code, `+6` event type, `+8` event
  flag, `+0xA` event id (script; 0xFFFF = alias), `+0xC` dir, `+0xE..+0x12` params 0..2, `+0x14` / `+0x16` move limit x / z,
  `+0x18 u32` kind (0 grid, 1 rail), `+0x1C u16` grid x, `+0x1E u16` grid z, `+0x20` fx32 y.
* SAVE POSITIONS (0x168 = 30 VecFx32, -1 = empty): slot of (zone, OBJ id) in ROM `0x02172A70` (`fldmmdl::SAVE_SLOTS`) for
  OBJ codes 0x6D and 0x2003..0x2005.

Cross-check with `nds_rom::map::events::Npc` (A6, 36-byte records): `id` = OBJ id (matches), `unk_02` = OBJ code (sprite),
`unk_04[24]` = move code, event type, event flag, event id (script), dir, params 0..2, move limits x / z, and the u32 kind at
record +0x18; `x` = grid x (matches), `y` is the grid **z** (the horizontal row; the height is separate), `unk_20` = fx32 height
y (`n << 16`; e.g. 0x30000 = 3 tiles up). For kind 1 (rail, 40 of 2257 records in ARC 125) `x`/`y`/`unk_20` are rail
coordinates read by `021642F8` (not decoded). **Fixed in F4a** (below).

### Spec conventions (these two areas)

* The `fldmmdl` arena is the `work` image: MMDLSYS work+0, MMDL[0..7] work+0x100.. (count drawn 0..8), OBJ code table
  work+0x1100, save positions work+0x1480, headers work+0x1600, scratch work+0x1800, FAKE move / draw proc tables work+0x1900 /
  work+0x1920 (callbacks 0x02EE0011.. / 0x02EE0051.., nullable). `0216E400` / `0216E40C` (ROM proc-table lookups) and
  `0216D8AC` are `force` stubs in their callers (the ports call them through the hooks).
* The MMDL iterator `0216DD1C` is stubbed `seq ret=pick:0/1` with `outp:work:0x100:1` (MMDL 0); the out-pointer kind
  `outp:work:OFS:N` is new (F4a-spec, mirrored in `svflow.py` and `black-fndiff`).
* Create paths (`0216CDC4` and its callers) keep MMDL 0 free (`0216CEAC` / `0216CEE4`: all eight) so the first-free search
  never returns null.
* `event_battle_head` blocks are `nowork` with `arg buf 0x100` arenas; every callee is a `force` stub.


## F4a: `fldmmdl` (253/253) and `event_battle_head` (7/7) ported

| area | ported | verified (in `PORTED`) | area hash |
|---|---|---|---|
| `fldmmdl` | 253 | 253 | `1aff530db0eefc54` |
| `event_battle_head` | 7 | 7 | `7fec025df763bb42` |

`fldmmdl.rs` holds every function of `fldmmdl.c` (MMDLSYS create / tasks / delete, MMDL create from a HEADER, the
accessors and flag setters, the "every MMDL in use" searches, position / header / OBJ code changes, the OBJ code table
and the save positions); `event_battle_head.rs` the battle-call GMEVENT proc, its constructors, the callback runner and
the Burmy form update. Records are read and written through the `hooks::Mem` window of a `hooks::FieldEnv`; every
callee outside the area (heap, task system, archives, the event-work var / flag accessors, the rail and grid movement
functions of other overlay-10 files, the overlay-21 `fldmmdl_acmd` / `fldmmdl_blact` / `fldmmdl_g3dobj` draw side, the
proc-table callbacks) is reached through `FieldEnv::call` by address (D3: no draw code is ported here). In-area callees
are Rust calls, except the three the spec keeps behind the seam: `0216E400` (`move_proc_table`) / `0216E40C`
(`draw_proc_table`) from `set_move_procs` / `set_draw_procs`, and `0216D8AC` (`pos_y`) from `find_at_height`. The
callers' stack locals (the HEADER copy of `create`, the HEADER of `create_at`, the position vectors, the 0x44-byte MMDL
list) are `Mem::push_local` windows; the `while (FUN_0216DD1C(sys, &m, &i))` loops iterate on Rust locals
(`next_local`, the same steps as `next_in_use`). ROM tables embedded: `MOVE_PROC_TABLES` (0x021D5E04, 47),
`DRAW_PROC_TABLES` (0x021D770C, 21), `SAVE_SLOTS` (0x02172A70), `event_battle_head::BURMY_FORMS` (0x02172554, 25).

`testenv.rs`: a THUNKS entry is no longer mapped to its target when the function's spec block stubs the thunk itself
(`event_battle_head` stubs `02008DF0`; no F2a spec stubs a THUNKS address, so their runs are unchanged).

UNVERIFIED (behaviour the differential tests cannot reach): `create` with no free MMDL (the original goes on with a null
pointer; the port panics at the window); `create_by_id` with no matching header returns 0 (the original returns an
uninitialised stack word, which is 0 under the Z19 entry state); `move_proc_table` / `draw_proc_table` panic past the
47 / 21 entries (the original reads what follows the ROM table).

Deliberate deviations: none.

`nds_rom::map::events::Npc` now names the HEADER fields: `obj_code` (was `unk_02`), `move_code`, `event_type`,
`event_flag`, `script` (event id), `dir`, `params[3]`, `limit_x` / `limit_z` (s16), `kind` (u32 at +0x18; 0 grid, 1
rail) (these were `unk_04`), `x`, `z` (grid z; was `y`), `height` (fx32 y; was `unk_20`). Byte round trip unchanged
(`nds-rom/tests/map_rom.rs`; `npc_records_are_mmdl_headers`: 2217 grid / 40 rail records, 2131 grid heights are whole
tiles, the others half / quarter tiles).

## F6a-spec: F1 row 10.23 "eventdata_system.c" (zone event data) and its neighbours (specs + smoke ports)

F1 row 10.23 (`0x02161D98..0x02164768`, overlay 10) holds four files; F6a-spec splits it by address into four spec areas under
`tools/rom-diff/specs/black-field/` (set `field`), generated with closure stubs (a block lists the stubs of its callees and of the
callees of all its in-area descendants and the shared fix lines; with every function of the area treated as ported all blocks run
without a fault, stray write or unstubbed callee), so each area can be ported in any order. `svflow.py AREA --all --strict-callees` and
`--order-check` are clean for all four.

| area | contents | range | specced | coverage (plain / closure) | smoke ports (hash) | area hash |
|---|---|---|---|---|---|---|
| `wfbc_data` | Black City / White Forest people LIST + COMM record (callers in ov20 `field_wfbc_data_local.c`, F1 scope X1) | `0x02161D98..0x0216237C` | 40 | 658/658 / 658/658 | `02162254` `slot_a_get` (28b863af3fd4d04e) | bfa64f2c43e3e8cc |
| `eventdata` | `eventdata_system.c`: load, record accessors, lookups by position, warp exits, trigger checks | `0x0216237C..0x02163010` | 56 | 1460/1460 / 1451/1460 | `021625CC` `warp_at` (3fdf6ac27877002c), `02162C54` `warp_exit_dir` (045d3001fed6ce4e), `02162D2C` `warp_contains` (aa222acd171682c8) | a3c5804a426adb7a |
| `mmdl_move` | MMDL grid move procs, height / attribute tracking, attribute field effects, step hit checks, direction tables, rail header glue | `0x02163010..0x021643AC` | 70 | 2129/2129 / 2114/2129 | `021642B4` `dir_between` (96a464d84c2e8a0a) | b006cfa1a82757fc |
| `eventdata_tail` | zone-change hooks, field-menu entry checks / event constructors, two GMEVENTs, hidden-item flags | `0x021643AC..0x02164768` | 19 | 364/364 / 364/364 | `021646E8` `hidden_item_index` (507fe2bc5c2ed984) | bb4944f39966a046 |

Coverage = instructions of the function body (Thumb walk from the entry, jump tables decoded) that the spec's inputs execute on the
original, with the in-area callees stubbed (plain) or real (closure). The closure gaps are hits that need two random records to agree
(`02162970` facing cases, `021627C8` / `0216282C` / `02162890` / `021628E4` var matches through real record checks, `021638CC` /
`02163AF0` / `02163BAC` overlapping MMDLs) and one path that is unreachable with real callees (`02163868`: `02163F40` never answers -1
when `02163E34` is real, it stores 0 first); plain mode covers all of them.

Function list: 197 entries in the row (bl/blx targets of the whole field set, odd code pointers, the Ghidra index, a Thumb walk from
every entry; the gaps between functions are 2-byte padding only), minus 12 veneers (`ldr r3,=T; bx r3`; callers stub T: `02162024`,
`02162038`, `02162040`, `02162A64`, `02162A6C`, `021626A8`, `021626B0`, `02164454` / `5C` / `64` / `6C`, `02164474`) and 3 false entries
(`02161E82`, `021634A8` inside `02163464`, `02163E72` inside `02163E58`: data words in main `0x020A3E5A..0x020A481C` decode as `bl`): 185
specced. Ghidra misses `02162370`, `0216237C`, `02162458`, `02162478`, `021624B0`, `02162560`, `021625E8`, `021621C0`, `02164240`, `0216458C`,
`021645C0` and lists 9 switch cases / shared tails as `LAB_` functions. `02162478` (trigger of kind 6 at a position) and `02162560` (warp
count) have no caller in the field set. Render-only functions left out: none (the draw side is reached only through overlay-21 field-effect
callees, all stubs).

### Records (as the code reads them; full tables in the spec headers)

* EVENTDATA_SYSTEM (0xAA8, `Heap_AllocDebug(heap, 0xAA8, 1, "eventdata_system.c", 0xAF)` in `0216237C`): `+0x04 / +0x08 / +0x0C`
  ARC 125 / 126 / 57 handles, `+0x10 u16` zone, `+0x12 / +0x14 / +0x16 / +0x18 u16` bg / npc / warp / trigger counts, `+0x1C / +0x20 / +0x24
  / +0x28` the record arrays, `+0x2C..+0x32` copies of the counts, `+0x34` field_encount_st ready, `+0x38` field_encount_st work (`0215E2B8`),
  `+0x120` -> the TAIL after the records (`021623DC` stores `sys + 0x2328`, outside the block, until the load), `+0x124` the ARC 125 member
  (u32 size, u8 counts, records, tail; <= 0x880 + 4 bytes, `021624B0`).
* BG (20): `+0 u16` script, `+2 u16` kind (2 = hidden item: script `8000 + i`, flag `ROM 0x02171534[i] + 900`, present while the flag is
  clear), `+4 u16` facing rule (0: dir 0, 1: dir 3, 2: dir 2, 3: dir 1, 4: any, 5: dir 2 / 3, 6: dir 0 / 1), `+6 u16` position kind (0 grid,
  else rail), grid `+8 u32` tile x, `+0xC u32` tile z, `+0x10 s32` y in `fx32 >> 12` units; rail `+8 u16` line, `+0xA u16` front, `+0xC s16`
  side.
* WARP (20): `+0 u16` destination zone, `+2 u16` destination warp (0x100: LOCATION kind 3, else 2; `0xFFFF` in `+0` or `+2` = disabled),
  `+4 u8` exit direction (1..4 -> 0..3, else 1), `+5 u8` (door / transition kind, `021626E8`), `+6 u16` position kind, grid `+8 / +0xA / +0xC
  s16` x / y / z in `fx32 >> 12` units (16 per tile, door centre `16 t + 8`), `+0xE / +0x10 u16` width / depth in tiles; rail `+8` line,
  `+0xA` front start, `+0xC s16` side start, `+0xE` front length, `+0x10` side length, `+0x12 u16` flip mode.
* TRIGGER (22): `+0 u16` script, `+2 u16` value, `+4 u16` event var (0 = unconditional, else `*var == value`), `+6 u16` kind (1..4 = facing
  0..3, 6 = the `02162890` set, 0 = others), `+8 u16` position kind, grid `+0xA / +0xC u16` tile x / z, `+0xE / +0x10 u16` width / depth,
  `+0x12 s16` y (`fx32 >> 12` units), `+0x14 u16` unread (0 in every ROM record); rail `+0xA` line, `+0xC` front, `+0xE s16` side, `+0x10` /
  `+0x12` lengths.
* RAIL LOCATION (8): `+0 u16` line, `+2 u8` type (1), `+3 u8` key (`0201441C(dir)`), `+4 s16` side, `+6 u16` front; exit code (u16) = width
  `<< 4` | offset | dir `<< 8` (`02162B68` / `02162BCC`, consumed by `02162AA4` / `021625FC`).
* ATTR INFO (0x1C, `021631C0`, on the stack of the `mmdl_move` effect passes): previous / current attribute, their values (`021AB0F0`) and
  flags (`021AB0F8`), `+0x10` -> OBJ DATA, `+0x14` field-effect control, `+0x18` season.
* WFBC LIST (0x1E8) of two `REC[10]` lists (`+8`, `+0xF8`; REC 0x18: kind nibble, id < 30, byte <= 100, `u16 name[8]`) and the COMM record
  (0x48: id, flags, 3 + 1 + 1 + 3 eight-byte slots).

Cross-check with `nds_rom::map::events` (A6; checked against all 428 ARC 125 members with a scratch script; the fields were renamed in
F6a, below):
* `BgEvent`: `script` matches; `unk_02` = kind (0: 351, 1: 127, 2 = hidden item: 184 records, scripts 8000..8176 = items 0..0xB0); `unk_04`
  = `u16` facing rule (4: 325, 0: 235, 6: 71, ...) + `u16` position kind (25 rail records); `unk_08` / `unk_0c` = tile x / z (`u32`; rail:
  line + front / side halfwords); `unk_10` = y in `fx32 >> 12` units (multiples of 16, e.g. 48 = 3 tiles up), not a tile count.
* `Npc`: as F4a-spec found (`y` is grid z, `unk_20` the fx32 height, `unk_04` packs move code .. move limits and the u32 kind).
* `Warp`: `dest_zone` matches; `unk_02` = destination warp index; `unk_04` = `u8` exit direction + `u8` door kind (1..6, 33..42) + `u16`
  position kind (77 rail warps); `unk_08` / `unk_0c` / `unk_10` are not fixed-point words but `s16` pairs: x, y | z, width | depth, flip mode;
  x / y / z are in `fx32 >> 12` units (e.g. x 152 = tile 9 centre), width / depth in tiles (1x1: 500, 3x1: 206, 1x3: 79).
* `Trigger`: `script` matches; `unk_02` = value; `unk_04` = var, kind, position kind (u16 each); `x` = tile x (matches), `y` = tile **z**;
  `unk_0e` = width, depth (tiles), `s16` y (units), `u16` unused (0 everywhere).
* `tail`: the code keeps a pointer to it (`+0x120`, `02162450`, read by script_sys); its entries are not decoded here (F7/F8).

### Spec conventions (these four areas)

* Arenas: `wfbc_data` / `eventdata` / `mmdl_move` use the `work` image with fixed zones (spec headers); `eventdata_tail` is `nowork` with `arg
  buf` arenas (the GMEVENT work of `020124E8` is the block's argument 2).
* Every out-of-area stub is `force` (overlay addresses collide with battle `PORTED` entries; fldmmdl / ARM9 callees are ported in other
  black-field areas); the only call between the four areas (`02162970` -> `02164708`) is a forced stub too.
* Position records and positions are drawn small and correlated (tile 0 / 1, units 0..0x20) so the containment checks hit; the event-work
  var stub `020143CC` answers inside work+0xC00, whose values are drawn like the trigger values.
* Indirect calls through ROM tables (`02162120`: 0x02172990 [slot +6]; `02163D04`: 0x021714A0 / 0x02171490 [dir]; `021644C4`: 0x021714DC
  [i]; `021644F4`: 0x02171504 [i]) land on in-area stubs (plain) or force stubs at the table targets; their indices are bounded (`dir` 0..3,
  `i` 0..9 / 0..5, the slot index 0..2) because larger values read other ROM words as code.
* `02161F90` writes through the `02161DC4` result without a null check: its block plants a valid list-B record for each id 0..3.
* `svflow.py` `ALLOW` += `02082C60` (memset, used by `021631C0`).


## F6a: `wfbc_data` (40/40), `eventdata` (56/56), `mmdl_move` (70/70), `eventdata_tail` (18/19; 19/19 after Z22) ported

| area | ported | verified (in `PORTED`) | area hash | unverified ports |
|---|---|---|---|---|
| `wfbc_data` | 40 | 40 | `721154b28c4c24fa` | |
| `eventdata` | 56 | 56 | `2690176c4070cea5` (Z22; F6a `e2fedff5ec7feede`) | |
| `mmdl_move` | 70 | 70 | `21f1f6ebcf39f7b0` (Z22; F6a `854080fe99c7c86c`) | |
| `eventdata_tail` | 19 | 19 (Z22: +`02164708`) | `1278de1cfc023004` (Z22; F6a `366498360545b869`) | |

`--order-check` and `--strict-callees` are clean for all four. Every callee outside an area is reached through
`hooks::FieldEnv::call` by address (heap, archives, zonedata, field_encount_st, the LOCATION constructors, event-work
vars / flags, `FX_Mul`, the fldmmdl getters (F4a ports, `force` stubs in these specs), the map queries, the overlay-21
attribute predicates and field effects, `fldmmdl_acmd`, the rail system, GMEVENT, overlay-20 WFBC people data);
callees inside an area are Rust calls, including the in-area entries of the ROM function tables (`02162120`'s
`SLOT_HANDLERS`, `021644C4`'s `MENU_CHECKS`, `021644F4`'s `MENU_EVENTS`; out-of-area entries go through the seam).
The only call between the four areas (`02162970` -> `02164708`) stays behind the seam. Stack structs of the original
(ATTR INFO, positions, rectangles, LOCATION parts, the COMM request record, the slot records) are `Mem::push_local`
windows. ROM data embedded: `eventdata_tail::HIDDEN_ITEMS` (0x02171534, 0xB1 records; only `+0` and `+3` are read
here), `MENU_CHECKS` (0x021714DC), `MENU_EVENTS` (0x02171504), `wfbc_data::REQUEST_KINDS` (0x0217144C, plus the next ROM
word that the spec's index 3 reads), `SLOT_HANDLERS` (0x02172990), `mmdl_move::DIR_ROM` (0x02171458..0x021714F0: the
direction tables, read by address so that indices past a table read the next one, as the original does).

Unverified port (doc `UNVERIFIED port of`; not in `PORTED`): `02164708` (`hidden_item_flag`): `021646E8` accepts
script ids up to 8799 but the hidden-item table has 0xB1 records, so ids 8177..8799 make the original read past the
table (overlay 10 data, then overlay 11 code at 0x02172BC0..) and the spec's inputs (`range 7990 8200 || range
0x2250 0x2270`) reach them; the port panics there. Spec fix needed: draw `id` from `alt range 7990 8176 || range
0x2260 0x2270 || range 0 7999` (or bound the in-area answer of `021646E8` to `0..0xB0`). Its caller `02162970` is
verified through its `force` stub. **Z22: registered**; the spec draws `alt range 7990 8176 || range 0x2260 0x2270 || range 0
7999 || range 0x2260 0xFFFFFFFF` (the hidden-item scripts in the ROM are exactly 8000..=8176, `map_rom.rs`).

UNVERIFIED (behaviour the differential tests cannot reach): `attr_blocked` (`02163D04`) with an empty footprint reads
an uninitialised stack word as the first cell's attribute (0 under Z19 in every harness path; the port uses 0);
`apply_slots` with a handler index >= 3, `menu_check` with `i >= 10`, `menu_event` with `i >= 6` and `DIR_ROM` reads
outside 0x02171458..0x021714F0 panic (the original reads the following ROM); `rec_store` (`02161F90`) writes through
a null record when list B has no record of the id (the original does too; the port panics at the window).

Mutation spot check (`tools/check-fn.sh black-field AREA --fn`, each reverted): caught 5 of 10: `02162AA4` (`o > half`
-> `>=`), `021620B0` (no `break`), `02162B68` (`/ 16` -> `>> 4`), `02162278` (flag bit `i + 5` -> `i + 4`),
`02164508` (`b == 1` -> `b == 2`). Survivors: `021638CC` height threshold 0x14000 -> 0x13000 (the spec's heights are
multiples of 0x8000: no input between), `021645E0` wait bound 30 -> 31 and `02162970` facing rule 5 `<= 1` -> `<= 2`
(input gaps: `work[1]` is random 32-bit; the rule-5 records rarely hit with real containment checks), `02163A40`
`>` -> `>=` (equivalent: equal-width spans give the same answer), `021640B0` swapped attribute values (harness: the
`in:` words of the `0218DAEC` stub are logged but not mixed into its answer, so both map queries return the same
attribute; a `seq` on that stub would separate them). **Z22:** all of these but the equivalent `02163A40` are caught now
(`in:` words feed the stub answer; input biases for `021638CC`, `021645E0`, `02162970`; see `tools/rom-diff/svflow_specs/README.md`
"Z22 additions").

Deliberate deviations: none in behaviour.

`nds_rom::map::events` now names what the code reads: `BgEvent { script, kind (2 = hidden item), facing (rule 0..6),
pos_kind, x, z (u32 tiles; rail: line / front, side), y (i32, fx32 >> 12 units) }`; `Warp { dest_zone, dest_warp
(0x100 = LOCATION kind 3, 0xFFFF disabled), exit_dir (1..4 -> 0..3), door, pos_kind, x, y, z (i16 units, 16 per
tile), width, depth (tiles), flip (rail) }`; `Trigger { script, value, var, kind, pos_kind, x, z (was `y`), width,
depth, y (i16 units), unused }`. Byte round trip unchanged (`nds-rom/tests/map_rom.rs`); the new
`bg_warp_trigger_records_match_eventdata_system` checks all 428 members: bg kinds 0 / 1 / 2 = 351 / 127 / 184, the
hidden items' scripts are exactly 8000..=8176, facing rules <= 6, 25 rail bg records; warp exit codes <= 4, 77 rail
warps, all 809 grid warps centred on a tile (`x & 15 == 8`, `z & 15 == 8`), 6 disabled, sizes 1x1 476 / 3x1 195 / 1x3
70; trigger kinds 0..4 and 6, `unused` 0 everywhere, 10 rail triggers.

## F3a-spec: the grid player (F1 rows 21.20 / 21.21 / 21.22 / 21.23 / 21.37 / 21.64 / 21.65; specs + smoke ports)

Six spec areas under `tools/rom-diff/specs/black-field/` (set `field`), generated with closure stubs (a block lists the stubs of its
callees and of the callees of all its in-area descendants and the shared fix lines; with every function of the area treated as ported
all blocks run without a fault, stray write or unstubbed callee), so each area can be ported in any order. `svflow.py AREA --all
--strict-callees` and `--order-check` are clean for all six. Every out-of-area stub is `force`, also between the six areas.

| area | original | range | specced | coverage (plain / closure) | smoke ports (hash) | area hash |
|---|---|---|---|---|---|---|
| `player` | `fieldmap_ctrl_grid.c` + `field_player.c` (FIELD_PLAYER, grid / no-grid dispatch) | `0x021A2D88..0x021A36E4` | 72 | 917/917 / 917/917 | `021A3190` `entry_code` (5a417f2cf8abdbd6), `021A31A4` `form_code` (c7e81effe976cd55), `021A31D0` `code_entry` (dd5b97749a5b3fb8), `021A36E0` `grid_of` (f93cdd2b02d44263) | 7a77d4f37d4b2fe1 |
| `player_core` | `field_player_core.c` (PLAYER CORE: MMDL, form, requests, pad -> dir) | `0x021A36E4..0x021A4140` | 64 | 1085/1085 / 1085/1085 | `021A3C24` `remember_keys` (2b8a55792ed82023), `021A3C3C` `key_dir_x` (b1666604ca238329), `021A3C54` `key_dir_z` (8483be8f63ca7b5a), `021A3C94` `first_request` (32fc474f22c818fb), `021A3CB0` `set_requests` (58ef84c83e497712) | b6110fa7fe6d5c7f |
| `player_grid` | `field_player_grid.c` (PLAYER GRID: move values, acmds, footstep SE, ice / slide) | `0x021A4140..0x021A5880` | 97 | 2444/2445 / 2427/2445 | `021A5304` `set_flags` (5178a9ea306cc7e6), `021A530C` `clear_flags` (53f85f95155c0c9a), `021A5318` `has_flags` (1c2a9a6f8ff3d57f), `021A5768` `set_1c` (68f72573ed797dcc), `021A5778` `state_is_3` (fb7ae193d4545242) | 1560d3655b5196fb |
| `namipoke` | `fldeff_namipoke.c` (surf Pokemon effect state + position logic) | `0x021ACA38..0x021AD758` | 18 | 943/943 / 941/943 | `021ACBB8` `set_mode` (a69a84dd05f40bfe), `021ACBE4` `follow_speed` (d931040f589761aa), `021ACC04` `ahead_along` (adf6c2eab41c6a30), `021ACCAC` `draw_none` (15e96184e19bea96) | 23d1b071964a9d4b |
| `height_ex` | `height_ex.c` + unanchored GMEVENTs (build-model events, boulder push) | `0x021C0598..0x021C1024` | 38 | 1140/1140 / 1140/1140 | `021C0598` `rect_contains` (cd3ede0886dfd55c), `021C0688` `count` (0d725dd5e1bf09d1), `021C068C` `record_contains` (b84e81969871669e), `021C06A8` `height` (78dc3b05ac38d1ab), `021C06B4` `extra` (be1a0871baa7f7f0) | 358aff190a331d7a |
| `player_event` | `field_player_grid_event.c` (surf start / end tasks) | `0x021C1024..0x021C1A08` | 17 | 1067/1067 / 1067/1067 | `021C1094` `surf_start_done` (c18ea68039a0a196), `021C10A0` `surf_start_delete` (fd8ac52e091b8203), `021C12C4` `surf_end_done` (e99d310f2b862f16), `021C12D0` `surf_end_delete` (567e9a5406aa55b1), `021C12E4` `set_ride_offset` (58f3e36eaae0db5c) | cf794bd97b65a59c |

Coverage = instructions of the function body (Thumb walk from the entry, jump tables decoded) that the spec's inputs execute on the
original, with the in-area callees stubbed (plain) or real (closure): 7596 / 7597 plain, 7577 / 7597 closure. The one plain miss is `021A4E4E` in `021A4DC0`, a
`b` that no input can reach (the acmd base is 0xC / 0x10 / 0x58 whenever r7 is 1). The closure gaps are hits that need two real callees
to agree (a pick of one stub that a real in-area callee now computes).

Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry: 330 entries
(021C0598 sits just before the F1 cut of row 21.64 but only height_ex.c calls it; 021C0628 / 021C06C0 have no caller), minus 5 false
ones (`021A3494`, `021A53AC`, `021AD38C`, `021C0BB8`: ARM code / data in main decodes as `bl`; `021A3562`: a literal), 3 veneers
(`021A3810` / `021A41A8` -> Heap_Free, `021A575C` -> `021A550C`; callers stub the target), the 7 `fldeff_encount.c` heads `021AD758` ..
`021AD7A0` (create / delete pairs of the encounter-effect table 0x021D903C, U2) and the 9 render-only functions of `fldeff_namipoke.c` =
306 specced. Ghidra misses `021A3E48`, `021A3EE0`, `021A408C`, `021ACC3C`, `021AD738`, `021C0628`, `021C06C0`, `021C10B4`, `021C1318`,
`021C161C` (ROM-table targets). Thin wrappers (`ldr r0, [r0, #k]; ldr r3, =T; bx r3`) are real functions and are specced.

Render-only functions left out (D3, U2; callers stub them by address): `021ACA70` / `021AD410` (load the surf / splash 3D models),
`021ACB0C` / `021AD530` (free them), `021ACCB0` / `021AD274` (draw the surf Pokemon), `021ACFD8` / `021AD6A4` (free the animations),
`021AD738` (splash draw position).

### Records (as the code reads them; full tables in the spec headers)

* FIELD_PLAYER (0x10, `field_player.c` 0x4C): `+0` FIELDMAP, `+4` -> CORE, `+8` -> GRID (grid maps), `+0xC` -> the no-grid player
  (F3b). The map kind (`02188CFC`: 0 grid, 1 rail) picks the half; most functions are wrappers over the core.
* MAP CTRL grid work (0x10, `fieldmap_ctrl_grid.c` 0x68; the grid entry of the MAP CTRL table 0x021D88F8): `+0 u16` heap, `+4`
  FIELDMAP, `+0xC` hold flag. Frame `021A2DF0`: pad trigger `020362DC`, pad held `0203630C`, running allowed = event flag 0x963
  (running shoes) -> `021A41B0` (grid frame).
* PLAYER CORE (0x34, `field_player_core.c` 0x97): `+4` GAMESYS, `+8` FIELDMAP, `+0xC` PLAYER record (`+0x1C` saved OBJ code, `+0x40`
  move form), `+0x10` move state, `+0x14` move value, `+0x18` sex (form-table row), `+0x1C` the player MMDL (OBJ id 0xFF; created from
  the HEADER template ROM 0x021D89A0), `+0x20` / `+0x22` last pad x / z dir, `+0x24` request bits, `+0x28` surf effect task, `+0x2C` /
  `+0x2E` counters, `+0x30 s16` request step, `+0x32` request done.
* MOVE FORMS (PLAYER record `+0x40`): 0 walk, 1 cycle, 2 surf, 3 surf variant (request 8). FORM TABLE ROM 0x021D890C (2 rows x 10
  `{OBJ code, form}`; walk 1 / 4, cycle 2 / 5, surf 3 / 6, special codes 0x88..0xA1) = `player::FORM_TABLE`.
* REQUESTS (`+0x24` bit i, lowest first): procs ROM 0x021D89C4[i][step]: 0 walk, 1 cycle, 2 surf, 3 the saved form's code, 4 / 5 / 6
  table entries 4..6, 7 entry 9, 8 surf variant, 9 entry 8. Every proc answers 0, advances `+0x30` or sets `+0x32`.
* PAD -> DIRECTION (`021A3BB4`): x 0x20 left (2) / 0x10 right (3), z 0x40 up (0) / 0x80 down (1); with both axes pressed the newly
  pressed one wins unless the MMDL already moves along one of them. B (held bit 1) runs when the running shoes flag is set.
* PLAYER GRID (0x20, `field_player_grid.c` 0x17D): `+0` move state (0 stand, 1 walk / ride / swim, 2 turn, 3 bump, 5 slide, 7 fall),
  `+4` move value, `+8` flags (1 forced move, 8 ice spin, 0x10 sliding, 0x20 running), `+0xC` -> CORE, `+0x10` FIELDMAP, `+0x14` /
  `+0x16` ice-spin counter (60 frames), `+0x18` / `+0x1A` footstep SE (0x546 none) and its frame counter, `+0x1C` one-shot flag.
* INPUT (0x14, on the stack of `021A41B0`): pad trigger, pad held, `u16` dir (9 none), flags (bit 0 running allowed), slow mode.
* MOVE VALUE -> ACTION (`021A4444` by form -> `021A4CA8`): walk 0 none, 1 stand, 2 walk / run (acmd bases 0xC / 0x10 / 0x14 / 0x58,
  slopes 0xA7 / 0x4C / 0xA3), 3 turn (0x24), 4 bump (0x1C + SE 0x55A unless a warp is ahead, `021A5810`), 5 jump (0x38 + SE 0x55E),
  7 slide (0x34), 9 fall; cycle 0..5 (ride 0x14 / 0x10, dust 0x9B); surf 0..4. `02197E7C(dir, base)` -> `0216D410(MMDL, acmd)`.
  Footstep SE: flag tables 0x021D8A00 / 0x021D8A0C, predicate tables 0x021D89F4 / 0x021D8A2C (spec header).
* HEIGHT_EX (8, `height_ex.c` 0x5A): count, -> records of 0x1C (`+0` x, `+4` z, `+8` width, `+0xC` depth, `+0x10` height, `+0x14` in
  use, `+0x18` extra), read by fieldmap 0218DDxx. Boulder push event work (0x24) and the build-model event works: height_ex header.
* SURF tasks (`field_player_grid_event.c`): start work 0x18 (0x1E6), end work 0x4C (0x2B3), layouts in the player_event header; the surf
  effect task work and the splash task work in the namipoke header.

### Spec conventions (these six areas)

* Arena = the `work` image (fixed zones, `fam.py`-style list in each header); the shared fix lines keep the FIELD_PLAYER / CORE / GRID
  pointers linked and the state fields in range.
* Pointers to the caller's stack are always `in:N` / `out:N` / `outm` / `outv` stub arguments (no stack address reaches a hash); stubs
  called in a loop whose answer must change are `seq` (polled callees, the OBJ code while a change is pending, per-tile attributes).
* New stub argument kind `outv:N:V1/V2/..` (F3a-spec, mirrored in `svflow.py` and `black-fndiff`, vectors in both test suites): an
  out-parameter that the caller compares with a constant (`020508EC`'s 0x3FFF in `021A3A3C`).
* The ports call the out-of-area callees through `FieldEnv::call` by address (D3: no draw code is ported).
* Mutation check of the smoke ports: 6/6 caught (one per area; the first `namipoke` mutation, `min(0x3FFFF)`, was equivalent and the
  first `height_ex` one exposed a weak rectangle bias in `021C0598`, now fixed).

## F3a: the grid player ported (304 of 306 verified)

| area | ported | verified (in `PORTED`) | area hash | unverified ports |
|---|---|---|---|---|
| `player` | 72 | 72 | `cfc4cc44e7fa9732` | |
| `player_core` | 64 | 64 | `d8693533c4c894f4` | |
| `player_grid` | 97 | 97 | `fb2d9e2ee4aa1297` | |
| `namipoke` | 18 | 18 | `5f2f50bb7702f7eb` | |
| `height_ex` | 38 | 36 | `948bf8d388d1c30b` | `021C09B0`, `021C0BE8` (below) |
| `player_event` | 17 | 17 | `5471f01d992c9f52` | |

The area hashes differ from the F3a-spec ones because the in-area stubs switch off once their callees are in `PORTED`
(the ROM side then runs the real callees). `--order-check` is clean for all six. Every callee outside an area (the other
five F3a areas, the no-grid player and rail system of F3b, fldmmdl, mmdl_move, fieldmap, eventdata, effect_encount,
fldmmdl_acmd, the PLAYER record of the GAMEDATA, heap, tasks, SE, camera shake, the namipoke render functions) is reached
through `hooks::FieldEnv::call` by address (D3: no draw code is ported); callees inside an area are Rust calls, also the
in-area targets of the ROM tables. The Heap_Free veneers `021A3810` / `021A41A8` and the veneer `021A575C` -> `021A550C`
are called at their targets (the specs stub the targets). Stack locals the original hands to its callees (positions,
rail locations, grid triples, the 0x14-byte grid INPUT record, the HEADER copy of the core constructor) are local byte
arrays (`CArg::InBuf` / `OutBuf`) or `Mem::push_local` windows.

* `player` (`fieldmap_ctrl_grid.c` + `field_player.c`): the MAP CTRL grid callbacks, FIELD_PLAYER create / delete /
  save, the grid / no-grid dispatch wrappers and the tails into the core, the front-tile / front-position / direction
  vector helpers (rail offsets `(d * u + 0x800) >> 12` on 64-bit products as the original). `021A30C8` / `021A3128`
  return their caller's r4 / r6 for a map kind other than 0 / 1: the ports take that register as an explicit `stale`
  argument (0 in the tests; the map-kind stub answers 0 / 1 only).
* `player_core` (`field_player_core.c`): the CORE constructor (finds or creates the player MMDL from the HEADER template
  ROM 0x021D89A0, `HEADER_TEMPLATE`), move state / form / OBJ-code changes, pad -> direction, and the request state
  machine (request lists ROM 0x021D89C4 `REQUEST_LISTS` and the proc block 0x021D895C..0x021D89A0 `REQUEST_PROCS`,
  dispatched to the Rust procs).
* `player_grid` (`field_player_grid.c`): the frame (INPUT record), move values by form (walk, ice, cycle, surf),
  actions by form (walk / run / turn / bump / jump / slide / fall, ride with dust, swim), hit checks, footstep SE (flag
  tables 0x021D8A00 / 0x021D8A0C, predicate tables 0x021D89F4 / 0x021D8A2C), forced-move tiles (0x021D89EC, dispatched to
  the in-area ports) and the ice turn order 0x021D8A24.
* `namipoke` (`fldeff_namipoke.c` logic): RESOURCE blocks, the surf and splash tasks, mode / hide setters, init procs and
  the grid / rail update procs (bob height 0x1000..0x4000, draw offset, the follower trailing the MMDL); splash offsets
  ROM 0x021D91F8 (`SPLASH_OFFSETS`).
* `height_ex` (`height_ex.c` + GMEVENTs): HEIGHT_EX create / set / delete / lookups, the script-callback GMEVENT, the two
  build-model animation GMEVENTs (model offsets ROM 0x021DA098 / 0x021DA0E0, `MODEL_OFFSETS`) and the boulder push
  (step check `021C0CA4`, footprint, start, event).
* `player_event` (`field_player_grid_event.c`): the surf start task and the surf end task (step procs ROM 0x021DA128,
  `SURF_END_STEPS`, Rust calls).

Unverified ports (doc `UNVERIFIED port of`; not in `PORTED`, their callers `021C0844` / `021C0AC0` reach them through
their stubs and are verified): `021C09B0` / `021C0BE8` pass `ROM 0x021DA098 + 12 * i` (resp. 0x021DA0E0) to
`MTX_MultVec33` (`0207C9D0`, reads 0x24 bytes) with `i` = work `+0x15`, which the code only advances while it is below
the count `+0x14` (at most 6 in the game). The spec draws both bytes `alt range 0 6 || any`, so `i` reaches ~255 and the
original reads far past the 6-entry tables (the port embeds 0xA8 bytes and panics there). Spec fix: `fix work 0xC15 u8
range 0 5` and `fix work 0xC14 u8 range 0 6` in both blocks.

UNVERIFIED (behaviour the differential tests cannot reach; the port panics where the original reads past a table):
`player_core::run_request` with a request / step outside the ROM proc block; `021A5630` with a display dir >= 4 (4-entry
stack copy of 0x021D8A24); `021A55A8` with a forced-move index >= 2; `namipoke` `021AD5E0` with a kind > 4;
`player_event` `021C1988` with a step outside 0..5; `player::entry_code` outside the form table.

Spec comments that disagree with the code (the ports follow the code; harness job): `021A5408` jump table (state 0 -> 1,
1 -> 0, 2..3 -> cancel the acmd and 1, > 3 -> 1); `021A3D08` equal-code path returns 1, not the getter's r0;
`021ACDC4` / `021ACFF8` bob clamp is 0x1000..0x4000 (not 0x400..), the rail follower's "still" test is `<= 40` (grid
`< 40`), and the mode-2 rail proc still adds `bob - 0x1000` to y; `021C0780` step 3 passes the `02012138(gsys)` result,
not 1; `021C14F8` / `021C16AC` shake the camera with 0xFFFE0000; `021C1318` writes the tile centre to `+0x24` and then
overwrites it; `021C1988` runs `021C16AC` when `+0 == 0` (the table loop otherwise); `021C09B0` uses the ROM row as the
matrix and work `+8` as the vector (the `MTX_Rot` result is unused).

Mutation spot check (`tools/check-fn.sh black-field AREA --fn`, each reverted): 11 of 15 caught, at least one per area:
`player` 2/3 (`021A3504` form bound, `021A3524` form 2 -> 3; survivor: the rail rounding 0x800 -> 0x400 in `021A3364`,
because the unit-length stub `021B7C38` answers only multiples of 0x1000), `player_core` 2/2, `player_grid` 2/2,
`height_ex` 2/2, `player_event` 1/1, `namipoke` 2/5 (survivors: the bob lower bound `>` -> `>=` and the rail speed
`<= 40` -> `< 40` need exact values the inputs never hit; the grid clamp `>` -> `>=` is equivalent).

Deliberate deviations: none in behaviour.

## F5a-spec: wild encounters (field_encount_st, enc_pokeset, field_encount, effect_encount, map_attr; specs + smoke ports)

Five spec areas under `tools/rom-diff/specs/black-field/` (set `field`), generated with closure stubs (a block lists the stubs of its
callees and of the callees of all its in-area descendants; with every function of the area treated as ported all blocks run without a
fault, stray write or unstubbed callee), so each area can be ported in any order. `svflow.py AREA --all --strict-callees` and
`--order-check` are clean for all five. File bounds come from the `__FILE__` strings passed to `Heap_AllocDebug` (0215E31C
"field_encount_st.c", 021A9ED8 / 021AA490 / 021AA548 "field_encount.c", 021AA77C "effect_encount.c") and, where a file allocates
nothing, from the call graph.

| area | original | range | specced | coverage (plain / closure) | smoke ports (hash) | smoke area hash |
|---|---|---|---|---|---|---|
| `encount_st` | `field_encount_st.c` (ov10, F1 row 10.18) | `0x0215E2B8..0x0215E440` | 10 | 153/153 / 153/153 | `0215E358` `move_roamer` (bd830468a80bd244), `0215E3E4` `swarm_zone` (6da815577882823b), `0215E404` `any_roamer_active` (b4bb3a5e30a49624), `0215E428` `push_location` (42fa3505f8b7c075) | b181a5118f9b7c9f |
| `enc_pokeset` | enc_pokeset.c (ov21; F1 counted it in row 21.28 "field_menu.c") | `0x021A9350..0x021A9ED8` | 25 | 1327/1327 / 1327/1327 | `021A94B0` `rate_modifier` (faa9a0fa40414d9b), `021A996C` `rejected` (07e1c7f19f7738d0), `021A9DC8` `roll100` (ec3cc29e686f3cd0), `021A9DE0` `land_slot` (2af26d6b476b6055), `021A9E64` `water_slot` (74773d933136a7bb), `021A9E98` `fishing_slot` (cce9ca220a45f360) | 8c4b543b7265043a |
| `encount` | `field_encount.c` (ov21, F1 row 21.29) | `0x021A9ED8..0x021AA77C` | 19 | 925/925 / 925/925 | `021AA3E0` `rate_of` (e58485b1e4fc408c), `021AA3FC` `rate_check` (4a0ea365af4f3bc3), `021AA69C` `weather_class` (be901561c94fb25f), `021AA6E0` `rate_roll` (6cf5cf1e9c1b7cf8), `021AA778` `steps` (9e0936cf35143d00) | 171a2e9e134c10b0 |
| `effect_encount` | `effect_encount.c` (ov21, F1 row 21.30, first part) | `0x021AA77C..0x021AB0D4` | 29 | 1030/1030 / 1030/1030 | `021AA7DC` `set_params` (101957db2c123694), `021AAB48` `item` (c1a269747cc5d424), `021AABC4` `step_due` (2de542984a587430), `021AABEC` `reset_counter` (9ae9caf275a47fbc), `021AB070` `shadow_item` (43816e9c3e1eac63) | 515de880ccb606c9 |
| `map_attr` | MAPATTR accessors (ov21, F1 row 21.30, tail; no `__FILE__`) | `0x021AB0D4..0x021AB5F4` | 68 | 601/601 / 601/601 | `021AB0F0` `value` (04e931146b9319e1), `021AB100` `valid` (6e923909b2e34ca5), `021AB250` `is_long_grass_a` (09bdf3432f2f4591), `021AB270` `is_long_grass_b` (f0601593713d4759), `021AB29C` `is_long_grass` (6f65057c43ebd492), `021AB504` `effect_kind` (7aa0038a043177e6) | 58a7861b87714527 |

Coverage = instructions of the function body (Thumb walk from the entry, jump tables decoded) that the spec's inputs execute on the
original, with the in-area callees stubbed (plain) or real (closure). Area hashes of the whole specs (`svflow.py AREA --all`): encount_st
0e78a61f591671cc, enc_pokeset 5f76f704a63123fd, encount 8abddf9e19162aab, effect_encount ba71840eb21c733c, map_attr d42b07bcc5b35293.

Function lists (bl/blx targets of the whole field set, odd code pointers, the Ghidra index, a Thumb walk from every entry; the gaps between
functions are 2-byte padding only): 153 entries, minus 2 veneers (`0215E334` -> Heap_Free, `021AA8F8` -> `021AAF54`; callers stub the
targets) and 3 false entries (`021A939C`, `021A9C56`, `021AB394`: data words in main 0x020A3EE2 / 0x020A4790 / 0x020A3EE6 decode as bl)
= 151 specced. Ghidra misses `021A9810` (swarm species, no caller in the field set), `021A9E64`, `021A9E98`, `021AB1A0`, `021AB1AC`,
`021AB2F8`, `021AB358`, `021AB428`, `021AB458`, `021AB3F8` (no caller) and splits the slot-pick functions into `LAB_` pieces.

F1 corrections: row 21.28 ("field_menu.c", anchor 0x021A84A0) ends at 0x021A934C; 0x021A9350..0x021A9ED8 is enc_pokeset (all its entry points
are called by field_encount.c). Row 21.30 is effect_encount.c (to 0x021AB0D4) + the map attribute accessors. Row 21.40 is
fldeff_fishing.c only up to 0x021AE0CC; 0x021AE0CC..0x021AE44E is the head of event_trainer_eye.c (trainer sight checks over the MMDLs,
GMEVENT 021AE430; F4c).

Render-only functions left out (U2, D3): all of `fldeff_fishing.c` (`0x021ADEDC..0x021AE0CC`: 021ADEDC / 021ADF00 effect create /
delete (table 0x021D9000), 021ADF14 / 021ADF68 G3D resource / animation set-up and teardown (0204xxxx NNS calls), 021ADF94 rod model
position by direction (soft float into a VecFx32), 021AE01C animation start, 021AE030 / 021AE04C / 021AE050 / 021AE090 the effect procs of
table 0x021D92EC). Nothing there decides an encounter: the fishing encounter is field_encount `021AA180` (types 5 / 6, rate without the
walk counter, the lead's Suction Cups / Sticky Hold doubling it). The shaking-grass / dust / ripple objects (fldeff_encount.c, F1 row
21.38) are reached only through the force stubs `021ADAAC` / `021ADB4C` / `021ADB5C` / `021AC154`.

### The encounter roll (from the code)

* ENCOUNT TABLE (one ARC 126 member, `black_data::encounters`; loaded by `0215E2B8` into EVENTDATA_SYSTEM + 0x38): `rates[t]` (u8,
  t = 0..6) is the rate of slot group `t`, `t` = encounter type: 0 grass (12 slots at +0x08), 1 dark grass (12 at +0x38, double battles),
  2 shaking grass (12 at +0x68), 3 surf (5 at +0x98), 4 rippling water (5 at +0xAC), 5 fishing (5 at +0xC0), 6 fishing on a ripple (5 at
  +0xD4). `rates[7]` is the zone's effect-encounter kind 0..2 (bit 7 set in RAM = loaded). Members of 0xE8 bytes serve every season, of
  0x3A0 bytes hold four seasonal tables (table `season`); other sizes are rejected.
* Type of a step (`021AA35C`): the attribute word under the player (map_attr: flags bit 2 = wild encounters, bit 1 = water): no
  encounter attribute -> none (unless kind 1, the effect encounter); water -> 3 (4 for kind 1); land -> 1 when the value is long grass
  (6 / 7 / 9 / 0x22), else 0 (2 for kind 1). Attribute values 8 / 9 add 10 to the rate.
* Rate check (kind 0 walking only, `021A9F40`): rate' = `021A94B0`: x2 for Illuminate / Arena Trap / No Guard, / 2 for Stench / White
  Smoke / Quick Feet (Sand Veil in sandstorm, Snow Cloak in hail), 2/3 for Cleanse Tag / Pure Incense (of the unmodified rate), at most
  100, then the pass-power modifier `02010FB0`; the walk counter (`021AA728`): the first step after a reset forces rate 1; encounter
  when `021AA6E0()` (= `020056EC(0xFFFF) / 0x290`, 0..99) `<= rate'`. Fishing (`021AA180`): Suction Cups / Sticky Hold double the rate.
* RNG: the global 64-bit LCG of `black_core::rng` (`MathRandContext32`; `FUN_020056EC` = `next`, `FUN_02005728` = `next_scaled`; both
  ported in black-core). Percent rolls are `next(0xFFFF) / 656`, per-mille rolls `next_scaled(1000)`.
* Roamers (`021A957C`): an active roamer (ENC SAVE record) in the current zone that passes the level filter replaces the roll half of
  the time (`02005728(1000) >= 500`). Swarms (`021A99E4`): flag 0x960 set, the zone of the swarm record ROM 0x021D8DE8[ENC SAVE +0x30]
  (17 records: zone, species, form, levels 15..55), grass (type 0): 40 % (`02005728(1000) <= 400`) the swarm species.
* Slot pick (`021A9C58`): Magnet Pull / Static (half the time) prefer a slot of a steel / electric species; else ROM 0x021D8DD8[mode]:
  land 20/20/10/10/10/10/5/5/4/4/1/1 %, water 60/30/5/4/1 %, fishing 40/40/15/4/1 %, uniform (swarm). Level (`021A9B00`) = min + roll %
  (max - min + 1); form 0x1F = random form; Pressure / Hustle / Vital Spirit (half the time) raise it (`021A9BE8`: to the highest max level of
  the species in the group, or by 5 when that is at least 5 higher and the group's levels vary).
* Level filter (`021A996C`): repel (ENC SAVE +0x31 steps) rejects levels below the lead's; Intimidate / Keen Eye (half the time) reject
  levels 5 or more below; not for kinds 1 / 2. Dark grass gives a double (flag 0x20) with two able Pokemon and a 40 % roll; with one
  accepted Pokemon it becomes a single.
* The Pokemon (`021A96B8`): POKEPARAM from species / level / trainer id / PID (`021A9D30`: Cute Charm 67 % opposite gender, the rare check
  of `mon +6`), held item rolled from the personal items with ROM 0x021D8DCC odds (Compound Eyes), Synchronize copies the lead's nature.
* Effect encounters (effect_encount): every step with no active effect counts up to the kind's interval (20 steps, or 5 for kind 2), then
  10 % (15 %) places one candidate tile around the player (a 11 x 11 or half window of the 32 x 32 block); stepping on it starts a kind-1
  encounter, or gives a dust-cloud / bird-shadow item.

**A7g rate <-> group mapping: verified.** The mapping in `black_data::encounters` (rate `i` <-> slot group `i` in file order) is what the
code does (`021AA3E0` reads `table[type]`, `021A9A5C` copies group `type`); the ROM agrees (in all 148 tables a rate is nonzero exactly
when its group has species). The 8th byte is not a rate but the effect-encounter kind (0: 115 tables, 1: 31, 2: 2); the black-data doc
now says so (doc only, no format change).

### Records (full tables in the spec headers)

* FIELD ENCOUNT (0x14, `021A9ED8`): FIELDMAP, GAMESYS, GAMEDATA, -> ENCOUNT TABLE, -> EFFECT ENCOUNT.
* EFP (0x2C, `021A9350`, on the caller's stack): GAMESYS, ENC SAVE, type, kind, slot-pick mode, slot / Pokemon counts, the lead's species /
  item / egg / ability / gender / nature / level, trainer id, MYSTATUS, FLAGS (ability and repel bits, header of enc_pokeset.spec).
* MON (0x14 per Pokemon): species, item, level, form, rare mode, special, gender request.
* ENCOUNT WORK (0x20, `02012A38(GAMEDATA)`): walk counter (grid position at the reset, moved, steps), effect step counter, effect state
  (zone, x / y / z, height, kind, state bits).
* EFFECT ENCOUNT (0x5C0, `021AA77C`): effect control, candidate count, 121 candidates of 0xC, effect object, interval, probability, item.
* ENC SAVE (save block 0x38): locations, two roamer records, swarm index (+0x30), repel steps (+0x31).
* FIELD SITUATION (0x10, `021AA304`): battle background, ground kind (`021AB580`, ROM 0x021D8EA0), weather class, season, zone, RTC hour /
  minute.

### Spec conventions (these five areas)

* The FIELD ENCOUNT arena is `arg buf 0x800` (object +0, ENCOUNT TABLE +0x20, EFFECT ENCOUNT +0x200, ENCOUNT WORK +0x7C0); blocks that
  iterate MMDLs (`021AAEC0` and its callers) also use the `work` image (the `0216DD1C` iterator's MMDL pointers).
* Locals handed to callees (EFP, MON, slot copy, field situation, search window) are `in:` / `out:` / `outm:` stub arguments; the in-area
  stub of `021A9350` writes the EFP (`outm:0x2C`, Pokemon count 0 / 1).
* RNG stubs `020056EC` / `02005728` answer below their argument (`lt:0`); in-area percent rolls pick their callers' thresholds.
* `021A9604` (WFBC) reads the caller's r5 as its weight column for types other than 0 / 1 / 3 / 0xFF (0 under the Z19 entry state; kept in
  the inputs; WFBC passes only 0 / 1 / 3 / 0xFF). `021A9A5C` with a negative type copies 0 bytes from the caller's r4.
* `021AACD4` runs the ARM9 soft-float helpers (`0209BDBC` / `0209C95C` / `0209CB8C` / `0209C494`, ALLOW) on both sides: its port must use
  `black_core::softfloat`.
## F4c-spec: event_trainer_eye.c (trainer line-of-sight spotting; spec + smoke ports)

One spec area under `tools/rom-diff/specs/black-field/` (set `field`), generated with closure stubs (a block lists the stubs of its
callees and of the callees of all its in-area descendants; with every function of the area treated as ported all blocks run without a
fault, stray write or unstubbed callee), so F4c can port it in any order. `svflow.py trainer_eye --all --strict-callees` and
`--order-check` are clean (all-function spec hash `8275afc1452b46ab`).

| area | original | range | specced | coverage (plain / closure) | smoke ports (hash) | area hash |
|---|---|---|---|---|---|---|
| `trainer_eye` | `event_trainer_eye.c` (ov21; F1 row 21.40 tail + row 21.41) | `0x021AE0CC..0x021AE948` | 38 | 941/941 / 936/941 | `021AE348` `line_test` (3cc557c88f633dca), `021AE360` `line_up` (a37f5802b499ec4f), `021AE394` `line_down` (46c6d1fbc0f967f0), `021AE3C8` `line_left` (a5e9f4aa51c90424), `021AE3FC` `line_right` (a14f402dedbbaa9d), `021AE5D4` `state_wait_30` (d1dc52683dc56dab) | 7eaf18973a1bd90a |

Coverage = instructions of the function body (Thumb walk from the entry, jump tables decoded) that the spec's inputs execute on the
original, with the in-area callees stubbed (plain) or real (closure). The closure gap is `021AE0CC`'s "two separate trainers see
the player at once" path (`0x021AE120..0x021AE12A`), which needs two independent random trainers to pass the whole real search; plain
mode covers it. Mutation spot check on the smoke ports (each reverted): 4/4 caught (`>=` -> `>` on the near boundary of up / left,
`<=` -> `<` on down / right, the 30-frame wait `>= 30` -> `> 30`, the wrong cross axis in `line_left`).

**File bounds.** The only `__FILE__` strings referenced from `0x021ADEDC..0x021AE948` are "fldeff_fishing.c" (`021ADEDC`,
`Heap_AllocDebug(.., 0x18, 1, file, 96)`) and "event_trainer_eye.c" (`021AE454`, line 506). `021ADEDC..0x021AE0C6` are the fishing
field effects (F5a-spec: render); from `0x021AE0CC` the code works on MMDLs, trainer ids and script records, and line 506 needs ~500
source lines before `021AE454`, so the file starts at `0x021AE0CC` (F1 put `0x021AE0CC..0x021AE44E` into row 21.40 fldeff_fishing.c)
and ends at `0x021AE948` (scrcmd_fld, F8-spec2). F1 row 21.43 (`0x021B0808..`, also labelled event_trainer_eye.c) is scrcmd_talk's
balloon-window placement (F8-spec2), so the file has 38 functions, not F1's 55.

Function list: 38 entries (bl/blx targets of the whole field set, odd code pointers, the ROM tables `0x021D9338` (4 line tests) /
`0x021D9348` (18 task states), the Ghidra index, a Thumb walk from every entry; the gaps are padding and literal pools). Ghidra misses
`021AE394`, `021AE3C8`, `021AE3FC`, `021AE4E0`, `021AE5A8`, `021AE650` and lists 20 `LAB_` fragments (shared returns / branch targets,
header of the spec). No veneers. Render-only functions left out: none; the "!" balloon (`fldeff_gyoe` `021BB528` / `021BB5C8`, field
effect delete `021AC154`) and the face / walk animations (`fldmmdl_acmd` codes from `02197E7C`, set with `fldmmdl::acmd_set`) are
callees behind the seam (D3).

Callers outside the file: `021AE0CC` from `fieldmap_event` `02189C18` (the field event check), `021AE198` from `fldmmdl_acmd`
`0219AE38` (a trainer MMDL re-checking its line), `021AE884` from `fldmmdl_acmd` `0219BBC6`.., `021AE454` / `021AE430` from the trainer
script commands `021AE948` / `021AE994` (scrcmd_fld: one TASK per EYE HIT slot, then the GMEVENT that runs both).

### Sight algorithm (as the code does it)

* **Who is a trainer** (`021AE79C`): `fldmmdl::kind_434(event_type)` (MMDL `+0x10`): 1 for event types 1, 4..8 and 11..13 (a trainer
  looking along its display direction), 2 for event type 2 (a trainer that looks in all four directions, tried 0, 1, 2, 3 in turn);
  anything else is skipped. **Sight range** = `(s16)` header param 0 (MMDL `+0x20`, `fldmmdl::param(m, 0)`). **Direction** = display
  dir (`+0x18`). **Trainer id** (`021AE884`) = `0215A460(event id)` (MMDL `+0x14`, the script id). **Already fought** (`021AE890`) =
  `0215A4D8(event work, trainer id)` (script_sys: the trainer's flag), such trainers are skipped.
* **Line test** (`021AE348` -> ROM `0x021D9338[dir]`): grid positions `s16[3]` x / y / z (`fldmmdl::grid`, MMDL `+0x3C..+0x40`) of the
  trainer `a` and the player `b`. Up: `a.x == b.x`, `a.y == b.y`, `a.z - range <= b.z < a.z`, distance `a.z - b.z`; down: `a.z < b.z <=
  a.z + range`; left / right: the same on x with `a.z == b.z`, `a.y == b.y` (so a different height never sees). Otherwise -1. 32-bit
  wrapping arithmetic, signed compares; range 0 never sees.
* **Path check** (`021AE7A8(m, dir, dist, enc)`; dist 0 = clear): from the tile in front of the trainer (`dir_dx` / `dir_dz`), each of
  the `dist - 1` tiles in between must be free (`mmdl_move::step_hits_here(m, x, y, z, dir) & !1 == 0`) and not the active
  effect-encounter tile (`021AA900(enc, &pos)`: shaking grass etc. blocks the view); the player's tile must hit exactly "an MMDL"
  (`& !1 == 4`) and not be the effect-encounter tile. Any failure = blocked.
* **Search** (`021AE1FC(fm, exclude, &rec)`): the first MMDL in use (`fldmmdl` iterator `0216DD1C`, MMDLSYS order) that is not the
  player's MMDL (`021A3074(FIELD_PLAYER)`), not `exclude`, sees the player and is not defeated -> EYE HIT record.
* **Event check** (`021AE0CC(fm)` -> GMEVENT or 0): no hit -> 0. `type = 0215A4C8(trainer id)` (the trainer's battle type),
  `n = 0218B6AC(gamesys)` (party members able to battle). Type 0 (single): with `n >= 2` a second search that excludes the first trainer;
  found -> both records, kind 2 (two trainers at once = a double battle); else one record, kind 0. Type 1 (a double-battle trainer pair):
  needs `n >= 2`; the partner is the first other MMDL of trainer kind 1 / 2 with the same trainer id (`021AE8D4`), recorded with the
  first trainer's distance and direction, kind 1. Types 2 / 3 (triple / rotation): need `n >= 3`, one record, kind 0. Other types, or
  too few members: 0. The event is the trainer script `02158768(gamesys, 0xE20, m, 1)` (`021AE930`); the records go to its script work
  with `0215A438(event, slot 0 / 1, &rec)`.
* EYE HIT record (0x18): `+0` MMDL, `+4` distance, `+8` dir, `+0xC` script id, `+0x10` trainer id, `+0x14` kind (0 one trainer, 1
  double-battle pair, 2 two separate trainers).

### Approach TASK (0x38, `021AE454`; ROM state table `0x021D9348`, `021AE72C` runs states while they answer 1)

`+0` state, `+4` done, `+8..+0x1F` the EYE HIT record, `+0x20` 0, `+0x24` slot index (0 / 1), `+0x28` frame counter, `+0x2C` the "!"
effect, `+0x30` fieldmap, `+0x34` FIELD_PLAYER. States: 0 unpause the trainer if its move bit 0x10 is set; 1 wait while move bit 0x10 is set, then show `dir` and pause; 2 wait for the player's acmd
(move codes 0x33..0x36 go to 7); 3/4 face acmd; 5/6 "!" balloon and its end; 7/8 acmd 0x65 (the other move codes' reaction); 9 wait
30 frames; 10..12 walk `distance - 1` tiles (acmd walk, `distance <= 1` stops); 13 wait 8 frames; 14 the player turns to the trainer
(`dir_between` of the grid x / z) unless already facing it, only for slot 0 or kind 2 (the partner of a pair does not turn him again);
15 wait; 16 end the acmd, move code 0 (the trainer stops wandering); 17 done. The GMEVENT `021AE754` (work = the two TASKs, made by
`021AE430`) steps both and frees each when done; it ends when both are gone.

### Spec conventions (this area)

* MMDLs are values only (no field is read directly): the iterator `0216DD1C` answers `work+0x100..+0x10C` (`outp:work:0x100:4`), the
  player's MMDL `021A3074` the same set, so "not the player / not excluded / not itself" hit; blocks without the iterator are `nowork`.
* Grid positions from `0216D828` are `outm:6:1/0/0/0/1/0` (x, z 0..1, y 0) so the real line tests of the closure runs see the player;
  the line-test blocks themselves take two `arg buf 8` positions with cross axes mostly 0 and the line axis small (plus `near 0x7FFF`
  and `any`), range `0..7`, -1, `0x7FFFFFFF`, `0x80000000` or any, 1000 inputs.
* Table indices are bounded: dir 0..3 (`021AE198`, `021AE348`, `dir_disp` answers 0..3), task state 0..17.
* The task states are `seq` stubs in `021AE72C` / `021AE754` (the loop runs while they answer 1); `Heap_AllocDebug` answers into an
  extra `arg buf 0x40` of `021AE454` (5th argument, never read), `020124E8` into an extra `arg buf 0x10` of `021AE430`.
* Every out-of-area stub is `force`; no grammar change.

Deliberate deviations (smoke ports): none. UNVERIFIED: `line_test` panics for `dir > 3` (the original calls through the ROM words after
the table).

## F4c: `trainer_eye` (38/38) ported

| area | ported | verified (in `PORTED`) | area hash | unverified ports |
|---|---|---|---|---|
| `trainer_eye` | 38 | 38 | `9d1854b2d2feadff` | |

`--order-check` is clean; ported from the raw `OVY_21.sbin` disassembly (rizin `-a arm -b 16`). Every callee outside the
area (fldmmdl getters and acmd calls, mmdl_move `step_hits_here` / `dir_dx` / `dir_dz` / `dir_between`, script_sys trainer
ids / battle type / defeated flag / EYE HIT store / script event, fieldmap glue, `field_player` MMDL, the effect-encount
tile test, GMEVENT, heap, and the D3 seam: `fldeff_gyoe` "!" balloon `021BB528` / `021BB5C8`, field effect delete
`021AC154`, acmd codes `02197E7C`) is reached through `hooks::FieldEnv::call` by address; callees inside the area are Rust
calls, including both ROM tables (`0x021D9338` = `LINE_TESTS`, `0x021D9348` = `run_state`'s match). The original's stack
locals that ported callees read or write (EYE HIT records, grid positions, the iterator's MMDL / index words, the found
direction) are `Mem::push_local` windows; the path check's position (seen only by stubs) is a Rust buffer.

| symbol | Rust item |
|---|---|
| `021AE0CC` | `check` (field event check) |
| `021AE198` / `021AE1FC` / `021AE288` | `sees_dir`, `search`, `sees` |
| `021AE348` / `360` / `394` / `3C8` / `3FC` | `line_test`, `line_up` / `line_down` / `line_left` / `line_right` |
| `021AE430` / `021AE454` | `approach_event_new`, `task_new` |
| `021AE498`..`021AE724` (states 0..17) | `state_unpause`, `state_face`, `state_wait_player`, `state_face_acmd`, `state_face_done`, `state_balloon`, `state_balloon_done`, `state_acmd_65`, `state_acmd_65_done`, `state_wait_30`, `state_walk_check`, `state_walk`, `state_walk_done`, `state_wait_8`, `state_player_turn`, `state_player_turn_done`, `state_end`, `state_done` |
| `021AE72C` / `021AE754` | `task_step`, `approach_event` |
| `021AE79C` / `7A8` / `884` / `890` / `8B4` / `8D4` / `930` | `trainer_kind`, `path_blocked`, `trainer_id`, `defeated`, `record_fill`, `partner`, `script_event` |

Mutation spot check (each reverted, `check-fn.sh --fn`): 6/6 caught (path loop `<` -> `<=`, triple/rotation `n >= 3` -> `>= 2`,
trainer-kind window `> 1` -> `> 2`, player-turn kind `!= 2` -> `!= 1`, partner without the "not itself" skip, the 8-frame
wait `< 8` -> `<= 8`).

Deliberate deviations: none. UNVERIFIED: `line_test` panics for `dir > 3` and `run_state` for `state > 17` (the original
calls through the ROM words after the tables); the game only passes display directions 0..3 and states it set itself.

## F4b-spec1: fldmmdl_acmd.c part 1 (acmd list task, acmd driver) and the MMDL draw glue before it (specs + smoke ports)

Two spec areas under `tools/rom-diff/specs/black-field/` (set `field`), with closure stubs (every function of the area treated as
ported runs without a fault, stray write or unstubbed callee); `svflow.py AREA --all --strict-callees` and `--order-check` are clean.

| area | original | range | specced | coverage (plain / closure) | smoke ports (hash) | smoke area hash |
|---|---|---|---|---|---|---|
| `acmd1` | `fldmmdl_acmd.c` head (ov21, F1 row 21.09, F4b job 1) | `0x02197D3C..0x02197F5C` | 14 | 222/222 / 222/222 | `02197DE8` `list_start` (444601f6d35221ae), `02197E44` `list_next` (0affd666e073898a), `02197E7C` `code_for_dir` (8374968a8a72d04f), `02197EDC` `action` (653e1c5505c96161) | 3361e06226a8ce4d |
| `mmdl_draw` | MMDL draw glue (ov21; F1 counted it in row 21.08 "ircbattlemenu.c"; no `__FILE__`) | `0x02197C4C..0x02197D3C` | 4 | 101/101 / 101/101 | `02197C9C` `anim_held` (f07f62817440e8e9), `02197CCC` `draw_pos` (e5e4d12e291b6fa4) | 6b6e3923f8e3dcbc |

Area hashes of the whole specs (`svflow.py AREA --all`): acmd1 80622af7be2ef983, mmdl_draw 472ce4ccbd163df4. No render-only functions in
either range; nothing skipped (no veneers or fragments: the Thumb walk from every entry covers both ranges up to 2-byte padding and
literal pools).

### File bounds (for F4b jobs 2 / 3)

* F1 row 21.08 (`ircbattlemenu.c`) really ends at `0x02197C4A`; `0x02197C4C..0x02197D3A` is MMDL code (`mmdl_draw`; its callers are
  fldmmdl.c `0216CD54` / `0216D3C6`, fldmmdl_blact / g3dobj, field_player, fldeff_namipoke).
* The only `__FILE__` literal in row 21.09 is "fldmmdl_acmd.c" (0x021DDBE0, `Heap_AllocDebug` in `02197D3C`, line 277). Its .data is
  0x021DDBD4..0x021DDBEF (a 3-pointer table read by `02198624` / `02199030` / `02199AE8`, then the string); the next `__FILE__` is
  "fldmmdl_blact.c" (0x021DDD70, first used at `0219C5C8`). No other file name can be placed inside the row, so the job split follows
  the code structure:
  * job 2: `0x02197F5C..0x0219A098`, 225 functions = the acmd steps (all 212 distinct targets of the grid step lists, `02197F5C`
    .. `0219A07C`) plus their helpers (e.g. `02197F6C` face-direction helper, `02197FC0` acmd-work set-up helper);
  * job 3: `0x0219A098..0x0219C4AC`, 187 functions = movement-type (move code) procs: the targets of the move-proc records
    0x021D56E0..0x021D5D64 (via the 47-entry table 0x021D5E04 that fldmmdl.c reads) and of the sub-tables 0x021D4D04..0x021D53B4, with
    their helpers; they call `02197EDC` / `02197E7C` of `acmd1`;
  * `0x0219C4AC` / `0x0219C52C` (called only from fldmmdl_blact `0219CBC2` / fldmmdl_g3dobj `0219D402`; table 0x021D75C4, shared with
    `0219D444`) are blact draw glue (render, U2), most likely the head of fldmmdl_blact.c: leave them to U2.
* The rail step lists (0x021D8104.., steps `0219DC18..0219E2A0`) are in F1 row 21.11 ("fldmmdl_g3dobj.c"), which therefore also holds
  the MMDL rail acmd code (logic, F3b/F4b follow-up).

### ACMD (animation command) layout

* MMDL `+0x26` u16 acmd code (0xFF = none), `+0x28` u16 seq (index into the code's step list), move bits `+4`: 0x200 set / 0x400 done,
  acmd work `+0xB4` (0x10). `fldmmdl::acmd_set` (`0216D410`) starts a code from outside (bit 0x200); a movement-type proc changes it
  with `fldmmdl::acmd_change` (`0216D438`) and calls `02197EDC` every frame until it returns 1.
* Step tables: ROM 0x021D5F58[code] (grid) and 0x021D624C[code] (rail, when MMDL status bit 0x2000 is set), 189 codes each (0..0xBC;
  neither index is checked by `02197F20`). Each entry points to a list of Thumb step functions; grid lists (0x021D4160..0x021D4A50) are
  `[step]` or `[init, update]` followed by `02197F5C` (sets move bit 0x400 and returns 0). Rail lists (0x021D8104..) have two entries (seq 0 / 1; the second is one of a few shared steps, e.g. `0219DC18`; not decoded further), except codes 0x3C..0x42, 0x45..0x4B, 0x9A, 0xA2, whose rail entry is the grid list.
* Step contract: `step(m) -> 0 | 1`: 0 = done for this frame, 1 = run the next step now (`02197EB4` loops while a step returns
  nonzero, reading code / seq afresh each time); a step moves on by `fldmmdl::inc_acmd_seq` (`0216D7CC`) itself and keeps its state in
  the acmd work (init steps clear it with `0216D6BC(m, size)`). r1 / r2 hold table words at the call; no step reads them.
* Kinds of 4 (per direction up / down / left / right): ROM 0x021D5D6C, a null-terminated list of 37 pointers to 4-word tables
  (`acmd1::DIR_CODES`); `02197E7C(dir, code)` maps a code to the same kind's code for `dir` (unknown codes unchanged).
* Grid step lists by code (from the ROM; `init` / `update` = list entries 0 / 1, `step` = a one-step list):
  `0x00..0x03`: step 2197F90/2197F9C/2197FA8/2197FB4
  `0x04..0x17`: init 2198060/2198074/2198088/219809C/21980B0/21980C4/21980DC/21980F0/2198104/219811C/2198134/2198148/219815C/2198170/2198184/2198198/21981AC/21981C4/21981DC/21981F0, update 219800C
  `0x18..0x2B`: init 2198380/2198390/21983A0/21983B0/21983C0/21983D0/21983E0/21983F0/2198400/2198410/2198420/2198430/2198440/2198450/2198460/2198470/2198480/2198490/21984A0/21984B0, update 219834C
  `0x2C..0x3B`: init 219862C/2198650/2198674/2198694/21986B8/21986D8/21986F8/2198718/2198738/2198758/219877C/21987A0/21987C0/21987E0/2198804/2198824, update 2198550
  `0x3C..0x42`: init 2198998/21989A4/21989B0/21989BC/21989C8/21989D4/21989E0, update 2198978
  `0x43..0x43`: init 21989EC, update 2198A10
  `0x44..0x44`: init 2198A58, update 2198A8C
  `0x45..0x4A`: step 2198AD4/2198AE8/2198AFC/2198B10/2198B24/2198B38
  `0x4B..0x4B`: init 2198BA8, update 2198B80
  `0x4C..0x4F`: init 2198CA4/2198CB4/2198CC4/2198CD4, update 2198D24
  `0x50..0x53`: init 2198D3C/2198D4C/2198D5C/2198D6C, update 2198D7C
  `0x54..0x5B`: init 2198208/219821C/2198234/2198248/2198260/2198278/2198290/21982A4, update 219800C
  `0x5C..0x5F`: init 2198848/2198868/2198888/21988AC, update 2198550
  `0x60..0x63`: init 2198D94/2198DA4/2198DB4/2198DC4, update 2198DD4
  `0x64..0x64`: init 2198E44, update 2198EC8
  `0x65..0x65`: init 2198F68, update 2198FB0
  `0x66..0x66`: init 2199070, update 2199050
  `0x67..0x67`: init 2198BB8, update 2198B80
  `0x68..0x68`: init 2199088, update 2199050
  `0x69..0x74`: init 2199234/2199268/2199298/21992CC/21992FC/219932C/2199360/2199394/21993C8/21993FC/2199430/2199460, update 21991F0
  `0x75..0x78`: init 21988D0/21988F4/2198918/219893C, update 2198550
  `0x79..0x84`: init 2199618/219964C/2199680/21996B4/21996E8/219971C/2199750/2199784/21997B8/21997EC/2199820/2199854, update 21991F0
  `0x85..0x90`: init 2199AEC/2199B10/2199B38/2199B60/2199B88/2199BAC/2199BD8/2199C00/2199C28/2199C50/2199C7C/2199CA4, update 2199970
  `0x91..0x98`: init 2199490/21994C0/21994F0/2199520/2199550/2199584/21995B8/21995E8, update 21991F0
  `0x99..0x99`: init 2198BC8, update 2198B80
  `0x9A..0x9A`: init 21990B8, update 21990D4
  `0x9B..0x9E`: init 21982BC/21982D4/21982EC/2198300, update 219800C
  `0x9F..0xA1`: init 2198BD8/2198BE8/2198BF8, update 2198B80
  `0xA2..0xA2`: init 21990A0, update 2199050
  `0xA3..0xA6`: init 2198CE4/2198CF4/2198D04/2198D14, update 2198D24
  `0xA7..0xAA`: init 2198DEC/2198DFC/2198E0C/2198E1C, update 2198E2C
  `0xAB..0xAB`: init 2199CC8, update 2199CE4
  `0xAC..0xAC`: step 2199D00
  `0xAD..0xAD`: init 2199D14, update 2199D4C
  `0xAE..0xAE`: init 2199D8C, update 2199DB4
  `0xAF..0xAF`: step 2199EAC
  `0xB0..0xB0`: init 2199EC0, update 2199ED4
  `0xB1..0xB1`: init 219A014, update 219A058
  `0xB2..0xB3`: init 219A028/219A03C, update 219A07C
  `0xB4..0xB4`: step 2199F30
  `0xB5..0xB5`: init 2198E44, update 2198F20
  `0xB6..0xB6`: init 2198E60, update 2198EC8
  `0xB7..0xB7`: init 2198E60, update 2198F20
  `0xB8..0xBA`: init 2198E98/2198EB0/2198E7C, update 2198EC8
  `0xBB..0xBB`: init 2198E7C, update 2198F20
  `0xBC..0xBC`: init 2199F50, update 2199F94

### ACMD LIST task (`02197D3C` start / `02197DA0` end flag / `02197DAC` delete; callers: scrcmd `021B16CA`, ov10 `02161878` / `02161B5C`)

Work (0x14, `Heap_AllocDebug(0x8000 | heap, 0x14, 1, "fldmmdl_acmd.c", 277)`, task priority = MMDLSYS priority - 2): `+0` seq (0..5,
ROM 0x021D4A50: `02197DE8` start entry, `02197DF4` wait until the MMDL can take an acmd, `02197E10` set it, `02197E28` wait until done,
`02197E44` repeat / next entry / end, `02197E78` ended), `+4` end flag, `+8` repeat counter, `+0xC` MMDL, `+0x10` -> current entry
`{u16 code, u16 repeat count}` (list ends with code 0xFE). The task proc `02197DCC` runs seqs while they return 1.

### Spec conventions (these two areas)

* Every callee is outside the areas except the acmd-list seq procs and `02197EB4` / `02197F20`; out-of-area stubs are `force`.
* The acmd code / seq getters (`0216D7C4` / `0216D7D4`) and the in-area dispatch stubs are `seq` (loops); codes are drawn from 8 kinds
  (0, 1, 4, 8, 0x18, 0x3C, 0x60, 0xBC) with seq 0..1, and the 22 step functions those reach in both tables are `force seq` stubs.
* The list-task work is an `arg buf 0x40` with a 6-entry list at +0x20; `02197D3C`'s allocation lands in a trailing `arg buf` (r2).

Mutation spot check (`check-fn.sh --fn`, reverted): 2 of 2 caught (`list_next` `<` -> `<=`, `anim_held` acmd test inverted).


## F5a: `encount_st` (10/10), `enc_pokeset` (25/25), `encount` (15/19), `effect_encount` (29/29), `map_attr` (68/68) ported

| area | ported | verified (in `PORTED`) | area hash | unverified ports |
|---|---|---|---|---|
| `encount_st` | 10 | 10 | `0e78a61f591671cc` | |
| `enc_pokeset` | 25 | 25 | `fc4dbf578e169a8c` | |
| `encount` | 19 | 15 | `75a53e7247a857c0` | `021A9F40` `step_check`, `021AA0A0` `fixed_battle`, `021AA180` `fishing`, `021AA248` `wfbc_step` (below) |
| `effect_encount` | 29 | 29 | `bae064b190095912` | |
| `map_attr` | 68 | 68 | `36aa2102c19d726f` | |

`--order-check` is clean for all five. Callees outside an area (also those in the other F5a areas, which the specs stub with
`force`) go through `hooks::FieldEnv::call` by address; in-area callees are Rust calls. The stack structs of the originals (EFP 0x2C,
MON pairs 0x28, slot groups 0x30, FIELD SITUATION 0x10, RTC time, VecFx32 positions, the search window, the attribute query output,
the MMDL iterator's `m` / `i`) are `Mem::push_local` windows passed as `CArg::In` / `CArg::Out` (the iterator index as `CArg::P`).
ROM data embedded: `map_attr::GROUND_KINDS` (0x021D8EA0), `enc_pokeset::SWARM_DATA` (0x021D8DE8, read by its ROM address because
`021A99A8` returns record addresses), `SLOT_PICK` (0x021D8DD8, dispatched to the Rust slot functions; a mode >= 4 panics where the
original jumps through the swarm table), `ITEM_ODDS` (0x021D8DCC), `effect_encount::WINDOW_OFFSETS` (0x021D8E76), `DUST_RARE`
(0x021D8E8A), `KIND_PARAMS` (0x021D8E70), `encount_st::SWARM_ZONES`. `021AACD4` computes the tile centres with
`black_core::softfloat` (`i2f`, `fadd` / `fsub` 0.5, `f2i`), as the original does through the ARM9 helpers. `_u32_div_f`
(`0209C2B0`) is `enc_pokeset::u32_divmod` (division by zero returns the dividend, as in black-battle; no input divides by 0).
`021AA490` / `021AA548` allocate `FUN_020173EC()` = 0xDC bytes (that library function returns the constant).

Found in the code (the spec comments differ):
* `021A9508` stores Pokemon `i` at `mons + 0x14 * accepted`: a Pokemon the level filter rejects is overwritten by the next one.
* `021A96B8` calls `021A9CAC(pp, per, eyes, dark)` with the Compound Eyes flag (0x200) third and `type == 1` fourth; the odds row is
  `eyes + 2 * dark` (rows 50/5/0, 60/20/0, 50/5/1, 60/20/5).
* `021AB5AC`: 0 when `a` is blocked; else 1 when `b` is water and not blocked, or when `b`'s value is 0x41 / 0x44 (also on land).
* `021AB140`, `021AB3B0`, `021AB458`, `021AB474` test ranges with `(v - k) & 0xFFFF <= 1` (low half only); the other predicates
  compare the whole register.
* `021A9810` returns the first halfword of the swarm record, which is its zone (not the species).
* `021AA0A0` returns the battle event of `0216E7C8` (the spec has `ret=none`).

Unverified ports (doc `UNVERIFIED port of`; not in `PORTED`; their callers' specs keep their stubs): `021A9F40`, `021AA0A0`,
`021AA180`, `021AA248` build the EFP on their own stack and reach `FUN_021A96B8` / `FUN_021A9780` (through `021AA490` /
`021AA548`), whose stubs log the EFP pointer as a value (`v`), so the hash holds the emulator's stack address. Spec fix needed
(encount.spec, blocks 021A9F40 / 021AA0A0 / 021AA180 / 021AA248 / 021AA490 / 021AA548): `stub 021a96b8 ... args=v,in:0x2C,in:0x14`,
`stub 021a9780 ... args=v,in:0x2C,v`, and `stub 0201347c ... args=v,v,v,in:14,v,v` (the original never writes the FIELD SITUATION's
padding `+0xE..0x10` and hands stale stack bytes to `0201347C`; seen in `021AA248`). With that fix and `CArg::In(efp, 0x2C)` /
`CArg::In(sit, 14)` in `setup_wild` / `setup_roamer` (tried in the worktree and reverted) all six functions match.

UNVERIFIED (behaviour the differential tests cannot reach): `021A9B68` with more than 12 typed slots (the original's 12-byte local
overflows); `021A957C` picks a candidate by `02005728(n)` (always below `n` in the game); the slot-pick mode >= 4 and the window
offsets / dust table past their ROM tables panic.

Mutation spot check (`tools/check-fn.sh black-field AREA --fn`, each reverted): caught 6 of 11: `021A9508` (`accepted` -> `i`),
`021AA728` (`< 0xA000` -> `<=`), `021AB3B0` (`<= 1` -> `<= 2`), `0215E2B8` (one-table season not forced to 0), `021AACD4` (height
from `out +8`), `021A9CAC` (`<` -> `<=`). Equivalent: `021A9BE8` (`>` -> `>=`: equal values give the same level), `021AACD4`
(`t << 4 > 0` -> `>= 0`: tile 0 gives 0x8000 either way). Input gaps: `021AABF4` `x1 >= 32` -> `> 32` (the gx / gz picks never give a
tile 27..31 with offset 5: add 0x1B / 0x3B), `021A9828` Cute Charm `< 67` -> `< 66` (with `021A9DC8` ported its in-area threshold
picks no longer apply; the raw `020056EC` answer hits roll 66 too rarely), `021AA42C` `>` -> `>=` (the `02010FB0` stub answers any
32-bit word against a roll below 100: `lt:101` or picks near the roll would separate them).

Harness defect found: `tools/check-fn.sh black-field encount` runs `cargo test encount::tests::fndiff_match_rom`, a substring of
`effect_encount::tests::fndiff_match_rom`, so both suites write `FNDIFF_JSON` and the last one wins (MISSING / MISMATCH depending on
the thread timing). Workaround: `RUST_TEST_THREADS=1` (libtest then runs `effect_encount` first and `encount` last). Fix: pass
`-- --exact` (or a per-area JSON path) in check-fn.sh.

Deliberate deviations: none in behaviour.
## F4b-spec2: fldmmdl_acmd.c part 2, the acmd steps (specs + smoke ports)

Five spec areas under `tools/rom-diff/specs/black-field/` (set `field`, generated by `/tmp/blk/f4bs2/gen2.py` from `specs_def.py`), with
closure stubs (every function of the area treated as ported runs without a fault, stray write or unstubbed callee); `svflow.py AREA --all
--strict-callees` and `--order-check` are clean for all five. The cuts fall between helper families, so no area calls another (0 crossing
calls).

| area | range | specced | coverage (plain / closure) | smoke ports (hash) | smoke area hash | whole-spec hash |
|---|---|---|---|---|---|---|
| `acmd2a` | `0x02197F5C..0x02198318` | 40 | 402/402 / 402/402 | `02197F5C` `end_step` (7c35198a5c82f60a), `02197F6C` `face` (9bb657d6daad12f0), `02197F90` `face_up` (6b6a22ff650d4cbd), `02197FC0` `walk_start` (d032acac0c5cc728), `0219800C` `walk_update` (b242a680dcfb4d3b) | 0892bb3b45f29d22 | bcc60321bf6a9402 |
| `acmd2b` | `0x02198318..0x02198960` | 49 | 709/709 / 709/709 | `02198318` `stay_start` (c8a9f52d9932846b), `0219834C` `stay_update` (95e7b40431641801), `02198380` `stay_up_32` (007eb8597db33785), `021984C0` `jump_start` (35b2d5fc63789f24), `0219852C` `jump_start_se` (6a96f6d1dd051f71), `02198550` `jump_update` (03e9fc650c6433f2) | a96c64e3694cdfdd | d34712f358a4518f |
| `acmd2c` | `0x02198960..0x02198E44` | 53 | 514/514 / 514/514 | `02198960` `wait_start` (e8a48d22613cdf5e), `02198978` `wait_update` (abb75e924af358bf), `021989BC` `wait_8` (fdb14095036c0c0e), `02198A8C` `fall_update` (68e00bfadd6b5ca9), `02198C4C` `table_walk_update` (03f6a843e2bb51db), `02198D7C` `table_walk_update_3` (7c8ab46d67c9fe1a) | 9c3e8f7b9d7edf56 | 043b6902047776da |
| `acmd2d` | `0x02198E44..0x02199888` | 50 | 1093/1093 / 1093/1093 | `02198E44` `anim10_start` (5ef0dcb370eb7cb1), `02198E98` `anim10_start_se` (da3ed0b93de2f16a), `02198EC8` `anim_wait` (b15f888878391784), `021990D4` `bounce_update` (584bdae6b4c22a0c), `0219912C` `vec_move_start` (e2a8d05fa2545b69) | 0204e048d3dad8d0 | 34b3ca32cf833e68 |
| `acmd2e` | `0x02199888..0x0219A098` | 32 | 864/864 / 864/864 | `02199888` `arc_start` (f5ae3163c89ad91c), `02199970` `arc_update` (b4481a123a24c74d), `02199AEC` `arc_85` (bc80c2c4f3a66879), `02199CE4` `count_22` (bf95d98817c9511c), `02199D14` `slow_left_start` (306df02baf5e3ece), `02199F94` `model_wait` (d65cfc5a6b68ee2c) | a55be673dbeabd03 | fdeb9e759d258d48 |

Coverage = instructions of the function body (Thumb walk from the entry, the jump table of `02199DB4` decoded) that the spec's inputs
execute on the original. Function list: the 212 distinct targets of the grid step lists (`0x021D5F58[code]`, 189 codes) plus 12 helpers
(`02197F6C`, `02197FC0`, `02198318`, `021984C0`, `0219852C`, `02198960`, `02198B4C`, `02198C08`, `02198C4C`, `02199034`, `0219912C`,
`02199888`) = 224; `021983DA` (in the F1 index) is not a function (a data word in main `0x020A3EC2` decodes as a `bl` into `021983D0`).
No veneers, fragments or render-only functions; the Thumb walk from the 224 entries covers the range up to padding and literal pools.

**Boundary with job 3 (checked):** `0x0219A098` is right. The last step `0219A07C` ends at `0x0219A096`; `0219A098` is the target of the
thunks `0219A0CC..0219A0FC` that the move-proc records `0x021D5D44` / `0x021D5D08` point at; no step list points at or past `0x0219A098`.
But the code-0x65 steps `02198F68` / `02198FB0` call three move-proc work accessors inside job 3 (`0219BF48` read +4, `0219BF3C` write
+4, `0219BF54` set byte +2; all through `0216D69C`, the movement-type work of the MMDL), so job 3's `0219BF00` family owns an effect
handle that the acmd steps start and delete (force stubs here; job 3 must keep their addresses). The 3D-model waits call
`0219FC68` / `0219FCC0` / `0219FE20` / `0219FE64` (F1 row 21.11, fldmmdl_g3dobj; force stubs).

### Steps by acmd kind (grid lists; `init` = entry 0, `update` = entry 1, entry 2 = `02197F5C`)

| codes | kind | init | update | work (acmd work = MMDL +0xB4) |
|---|---|---|---|---|
| 0x00..0x03 | face up / down / left / right | `02197F90..02197FB4` -> `02197F6C(m, dir)` (one-step list) | - | - |
| 0x04..0x17 | walk one tile, 32 / 16 / 8 / 4 / 2 frames (speed 0x800..0x8000, anim 1..5) | `02198060..021981F0` -> `02197FC0(m, dir, speed, frames, anim)` | `0219800C` | +0 u16 anim, +2 s16 frames, +4 dir, +8 speed |
| 0x54..0x57 / 0x58..0x5B / 0x9B..0x9E | walk: 1 frame 0x10000 anim 0 / 4 frames anim 10 / 16 frames anim 5 | `02198208..02198300` -> `02197FC0` | `0219800C` | as above |
| 0x18..0x2B | walk in place, 32..2 frames | `02198380..021984B0` -> `02198318(m, dir, frames, anim)` | `0219834C` | +0 u16 anim, +2 s16 frames |
| 0x2C..0x33 | jump in place 16 / 8 frames (low table, SE 0x55E) | `0219862C..02198718` -> `0219852C` -> `021984C0` | `02198550` | +0 speed, +4 distance, +8 u16 phase step, +0xA u16 phase, +0xC s8 dir, +0xD s8 frames, +0xE anim, +0xF s8 table |
| 0x34..0x3B / 0x5C..0x5F | jump 1 / 2 tiles, left / right 1 / 3 tiles (high table, SE 0x55E) | `02198738..021988AC` -> `0219852C` | `02198550` | as above |
| 0x75..0x78 | jump 3 tiles, 24 frames, no SE | `021988D0..0219893C` -> `021984C0` | `02198550` | as above |
| 0x3C..0x42 | wait 1 / 2 / 4 / 8 / 15 / 16 / 32 frames (also the rail lists) | `02198998..021989E0` -> `02198960(m, n)` | `02198978` | +0 frames left |
| 0x43 / 0x44 | rise / fall (draw offset y 0 -> 0x140000 / back, 0x10000 per frame; fall SE 0x676) | `021989EC` / `02198A58` | `02198A10` / `02198A8C` | +0 height, +4 speed |
| 0x45..0x4A | status 4 on / off (vanish? name guessed), 8 on / off (dir lock), 0x10 on / off (animation held, cf. `02197C9C`) | `02198AD4..02198B38` (one-step lists, also rail) | - | - |
| 0x4B, 0x67, 0x99, 0x9F..0xA1 | balloon mark over the MMDL, kind 0 / 0 / 0 / 1 / 2 / 3 | `02198BA8..02198BF8` -> `02198B4C(m, kind)` (return 0) | `02198B80` | +0 kind, +4 effect handle (`021BB528`) |
| 0x4C..0x4F / 0xA3..0xA6 / 0x50..0x53 / 0x60..0x63 / 0xA7..0xAA | walk one tile with a per-frame speed table, 6 / 6 / 3 / 7 / 12 frames | `02198CA4..02198E1C` -> `02198C08(m, dir, frames, anim)` | `02198D24` / `02198D7C` / `02198DD4` / `02198E2C` -> `02198C4C(m, table)` | +0 u16 dir, +2 anim, +4 s16 frames, +6 s16 index |
| 0x64, 0xB5..0xBB | draw animation 10 / 11 / 12 (0xB8 / 0xB9 with SE 0x7B7), wait for the blact actor | `02198E44` / `02198E60` / `02198E7C` / `02198E98` / `02198EB0` (return 0) | `02198EC8` (anim 0 after) / `02198F20` | - |
| 0x65 | hop with the movement type's effect handle (high table, step 2 per frame) | `02198F68` (return 0) | `02198FB0` | +0 index |
| 0x66 / 0x68 / 0xA2 | draw animation 1 / 2 / 0 for 0x28 / 0x28 / 0xC frames | `02199070` / `02199088` / `021990A0` -> `02199034(m, anim, frames)` (return 0) | `02199050` | +0 u16 counter, +2 u16 frames |
| 0x9A | bounce (ROM 0x021D4C1C, 20 frames) | `021990B8` | `021990D4` | +0 frame |
| 0x69..0x74 / 0x79..0x84 / 0x91..0x98 | vector move (VecFx32 per frame, separate display / move dir), 8 / 4 / 4 / 2 frames | `02199234..021995E8` -> `0219912C(m, &vec, disp, move, frames, anim)` | `021991F0` | +0 frames, +4 VecFx32 |
| 0x85..0x90 | arc move (±0x2000 per frame on x / y / z, low-table arc on another axis), 8 frames | `02199AEC..02199CA4` -> `02199888` | `02199970` | +0 u8 frames, +1 arc axis, +2 axis, +3 neg, +4 dist, +8 distance, +0xC phase, +0xE step |
| 0xAB / 0xAC / 0xAD / 0xAE | count 22 frames / anim 10 / slow step left / model-anim rise | `02199CC8` / `02199D00` / `02199D14` / `02199D8C` | `02199CE4` / - / `02199D4C` / `02199DB4` | see the spec comments |
| 0xAF / 0xB0 / 0xB1..0xB3 / 0xB4 / 0xBC | anim 10 / anim 11 + face down after the actor / 3D model animations / draw offset 0 / model anim with SEs | `02199EAC` / `02199EC0` / `0219A014` / `0219A028` / `0219A03C` / `02199F30` / `02199F50` | - / `02199ED4` / `0219A058` / `0219A07C` / - / `02199F94` | see the spec comments |

Most init steps return 1 (the update runs in the same frame); these return 0: the balloon inits, `02198E44` / `E60` / `E7C` / `E98` /
`EB0`, `02198F68`, `02199070` / `088` / `0A0`, `02199CC8`, `02199EC0`, `0219A014` / `028` / `03C`. Updates return 1 only where they move
on to the next entry in the same frame (wait, table walk, rise / fall, balloon, vector / arc moves, model waits); the walk, walk-in-place,
bounce, timed-animation (`02199050`) and counter (`02199CE4`) updates always return 0.

### Notes for the porter

* Every init step gets its work through `0216D6BC(m, size)` (the real one clears `size` bytes; the spec stub does not, so the fields an
  init leaves to the clear stay random in the test: `021984C0` +4 / +0xA, `02198C08` +6, `02199888` +8 / +0xC, `021989EC` +0, `02199034`
  +0). A port must call `0216D6BC` (or `fldmmdl::acmd_work_init`) and write exactly the fields the original writes.
* `02198E44` / `E60` / `E7C`, `02198F68` and `021990B8` / `02199CC8` call `0216D6BC(m, 4)` only for the clear; `02198EC8` / `02198F20`
  call `0216D6D4` and ignore it. Keep the calls (they are in the stub log).
* `0219852C` passes its stack arguments on as `(u16)anim`, `(s16)kind` (sign-extended), `(u16)step` and SE 0x55E.
* Byte / halfword reads: `02199888` reads frames / anim / axis / arc axis / neg as bytes of their stack words, `0219912C` reads anim as a
  byte; `021984C0` stores dir / frames / anim / kind as bytes (anim and kind via `ldrh` / `ldrsh` of the stack word first).
* `02199970` steps the grid by +1 when dist is 0 (no zero test; `02199888` tests dist != 0 first). Its y step is 2 * sign.
* Signed division `0x1000 / frames` in `02199888` is `0209C0A4` (real code on the ROM side, `ALLOW`); frames 0 is not tested.
* `02198C4C` gets a ROM speed table address in r1 (`acmd2c::SPEED_TABLES`); the table index is unchecked.
* The heights / speeds / bounce / vector constants are ROM data: `acmd2b::JUMP_HEIGHTS` (`0x021DDBD4`: the 3-pointer table of
  fldmmdl_acmd.c's .data = the jump-height tables), `acmd2c::SPEED_TABLES`, `acmd2d::BOUNCE`; the vector-move inits copy a ROM VecFx32 to
  their stack and pass its address (stub `in:12`).
* Draw offset = `0216D8C0` (MMDL VecFx32 +0x50): jumps / rises / bounces / arcs only change the drawn height; the grid and the position
  move separately (`02163E7C` one tile at the start or per whole tile of distance, `02163FC0` / `02163F8C` per frame).

### Traps

* Ghidra marks `020061E4` (play SE) noreturn: the pseudo-C of `02198550`, `021984C0`, `02198E98` / `02198EB0`, `02199DB4`, `02199F94`
  drops everything after the SE call; `02198550` really returns 1 after SE 0x67B, `02198E98` / `02198EB0` return their callee's result.
  It also hides `02163FC0`'s third argument (speed, r2) in `02198550` and the extra arguments of `0219852C`. Use the disassembly.
* Unchecked indexes the specs bound: jump-height table (+0xF) 0..2, `02198FB0` +0 0..15, `021990D4` frame 0..0x18 (a negative frame would
  index the stack below the copied table), `02198C4C` index within its table. The ports panic outside those (UNVERIFIED).
* The `0216D7A8` draw actor is compared with 0xFFFF as a whole word, then passed on as a u16 (the specs answer 0x10005 too).

Mutation spot check (`check-fn.sh --fn`, reverted; `/tmp/blk/f4bs2/mut.sh`): 9 of 10 caught (`walk_update` `>` -> `>=`, `walk_start`
frames <-> anim, `jump_update` distance `>=` -> `>`, `table_walk_update` `<` -> `<=`, `bounce_update` `min(9)` -> `min(8)`,
`vec_move_start` sign test, `arc_update` neg test, `model_wait` frame 3 -> 2, `arc_start` y step); the survivor (`jump_start_se` kind
sign extension) is equivalent while `021984C0` runs as real code (it stores only the low byte).


## F4b-spec3: fldmmdl_acmd.c part 3, the movement-type (move code) procs (specs + smoke ports)

Four spec areas under `tools/rom-diff/specs/black-field/` (set `field`, generated by `/tmp/blk/f4bs3/gen3.py` from `defs3.py` /
`headers3.py`), with closure stubs (every function of the area treated as ported runs without a fault, stray write or unstubbed callee;
the closure includes the targets of the ROM state tables and of the init thunks); `svflow.py AREA --all --strict-callees` and
`--order-check` are clean for all four. The cuts minimise crossing calls: the only ones are mvproc3a -> the mvproc3b helpers `0219B0C4` /
`0219B0F0` / `0219B104` / `0219B200` (10 call sites, force stubs).

| area | range | specced | coverage (plain / closure) | smoke ports (hash) | smoke area hash | whole-spec hash |
|---|---|---|---|---|---|---|
| `mvproc3a` | `0x0219A098..0x0219A87C` | 43 | 820/820 / 820/820 | `0219A098` `look_init` (bd984c8e3c605e68), `0219A0CC` `look_init_all` (58807c955155a546), `0219A1E8` `next_dir` (95cf090568566ca4), `0219A528` `face_init` (837dc0cb022442d4), `0219A54C` `face_move` (cc1e36dc00095413), `0219A7C4` `spin_wait` (ee4fa05f545e9254) | 07d60e5e5b2318b4 | d2d09618ab475bd2 |
| `mvproc3b` | `0x0219A87C..0x0219B41C` | 55 | 1240/1241 / 1234/1241 | `0219AA38` `route_init` (792ba237f200f9f5), `0219AA68` `route_init_15` (85ff9595735eaa1b), `0219B334` `walks_fast_near` (250c416edf81a8b6), `0219B358` `spins_while_walking` (6a6d15862cfe4603), `0219B374` `spin_order` (293133a3389b5f3e), `0219B404` `spin_unlock` (02aa2a3ce47774e1) | 929baef9b3a0cd61 | 7e7dea912cd0644b |
| `mvproc3c` | `0x0219B41C..0x0219BDC4` | 38 | 1013/1013 / 1013/1013 | `0219B41C` `ev_busy` (7acf7715805cabc8), `0219B494` `moved` (6a1c25e7763334ed), `0219B4C8` `standing` (b05e8b0879482c0b), `0219B508` `ev_steps_init` (2569610849461695), `0219B9B0` `follow_start` (5ee982f09b1bd801), `0219BA2C` `follower_base` (0fa04383e09d4c8e) | c87eddfab6ec2283 | 519c2a0f18ea794b |
| `mvproc3d` | `0x0219BDC4..0x0219C4AC` | 45 | 718/718 / 718/718 | `0219BE60` `effect_delete` (d897500eb05dd0d1), `0219BF3C` `set_effect_handle` (9cde8f866ae12dcc), `0219BF48` `effect_handle` (458f7e2be3817fed), `0219BF54` `effect_done` (fc5c248f9db15331), `0219BF60` `mirror_init` (f2189c17855573e3), `0219C0D4` `wall_init` (fafca15e1a9642ba) | af7673205c64d591 | 11c678cb632302fe |

Coverage = instructions of the function body (Thumb walk from the entry, jump tables decoded) that the spec's inputs execute on the
original. The one plain miss is dead code: `0219B074`'s `b` at `0x0219B0B6` needs an acmd code above 0xA4, not above 0xA5 and not equal
to 0xA5. Closure misses (in-area callees real): `0219B13C` `0x0219B18C` (the player vanishing between two `0216DEE0` calls with the same
argument) and `0219B200` `0x0219B21C..0x0219B220` / `0x0219B2C0` (no real dir set has one entry; the plain run covers them).

### Real range and boundaries (checked)

* Start `0x0219A098` (F4b-spec2's end; the target of the thunks `0219A0CC..0219A144`). End `0x0219C4AC`: the last proc helper `0219C338`
  ends at `0x0219C4A2` + its literal pool `0x0219C4A4..0x0219C4AB`. `0x0219C4AC` / `0x0219C52C` are blact draw glue (render, U2: they call
  only 02046EFC.. / 0204F044.. / 02050xxx draw routines and are called only from fldmmdl_blact `0219CBC2` / fldmmdl_g3dobj `0219D402`);
  F1 row 21.09 (`..0x0219C552`) therefore ends 0xA8 bytes early as logic, and `fldmmdl_blact.c` most likely starts at `0x0219C4AC`.
* Function list: 184 entries in `0x0219A098..0x0219C554` (bl / blx targets of the whole field set, odd code pointers, the Ghidra index,
  Thumb-walked) minus the two render functions, the veneer `0219B414` (`ldr r3, =0219B441; bx r3`; mmdl_move `02163010` calls it and
  stubs it) and five false entries (data words in main `0x020A3EC6` / `0x020A3ECA` / `0x020A47AC` / `0x0205DCB8` / `0x02088514` decode as
  `bl` into `0219A3D2` (padding + literal after `0219A2B8`), `0219C3CC` (inside `0219C338`), `0219ACC4` (inside `0219AB8C`), `0219A4B8` (a
  case of `0219A3D8`'s jump table), `0219BD14` (inside `0219BCE4`)) = **181 specced**. The 34 `ldr r3, =T; movs r1, #k; [movs r2, #j;]
  bx r3` init thunks are specced (they pass constants); their target is an in-area stub of their block.
* **Move-proc table: ROM `0x021D5E04` has 0x55 entries, not 47.** Codes 0..0x2E as F4a found, entry 0x2F is null, then 0x30..0x54
  (follow / partner / effect / mirror / wall types of mvproc3c / 3d, code 0x43 = random walk mode 2, 0x44 -> `021944BC..` in F1 row 21.07
  `field_comm_actor.c` (X1), 0x45..0x54 -> rail procs `0219E828..0219E930` in row 21.11). `fldmmdl::MOVE_PROC_TABLES` (47 entries)
  panics for codes 0x2F..0x54 (`0216E400` checks nothing): a porting follow-up for fldmmdl (the table is ROM data; extend it to 0x55,
  entry 0x2F = 0).
* Records `{u32 id, init, move, delete, return}` (0x14 bytes) at `0x021D56DC..0x021D5D6C`; the id word is 0..3 for codes 0..0x2E
  (meaning unknown; also 3 for 0x30..0x32 and 0x43) and the code itself for the others from 0x33 on. Codes 0 / 1 use the fldmmdl.c no-op procs `0216E464..0216E470`; the return proc
  of every code below 0x32 is `0216E470` (no-op), delete is `0219A24C` / `0216E46C`.
* Callers from outside: fldmmdl.c through the records only; mmdl_move `02163010` -> `0219B414` -> `0219B440` (event-type init) and
  `02163074` -> `0219B41C` (event-type busy); scrcmd_obj `021B1AD6` / `021B1B38` -> `0219BBA4` (partner search); acmd2d `02198F68` /
  `02198FB0` -> `0219BF3C` / `0219BF48` / `0219BF54` (the code-0x65 steps use the effect handle of the codes 0x33..0x36 in the move work
  +4 / +2; its owner is `021D046C`, an overlay-21 field effect added by `0219BEB4`). All of these callers already `force`-stub them.
  Calls into mmdl_move (`0x02163010..0x021643AC`, ported): `02163E34`, `02163F10`, `021639C0`, `02164168`, `021641E8`, `021641F8`,
  `021642A4`, `021642B4` (force stubs).
* Not in this range, for a later job: the rail acmd steps `0x0219DC18..0x0219E2A0` and the rail move procs `0x0219E828..0x0219E930`
  (codes 0x45..0x54) in F1 row 21.11 (`fldmmdl_g3dobj.c`, marked render), both logic.

### Movement types by move code (record procs; `w` = move work MMDL +0x94)

| codes | type | init | move (states) | work |
|---|---|---|---|---|
| 2, 6..0xD, 0x2D, 0x2E | look around in direction set 0..0xA (2 = all, 6 = up/left, ... 0x2E = left/right) | `0219A098(m, set)` via thunks | `0219A150`; delete `0219A24C` | +2 s16 wait (0x10..0x40), +4 set |
| 3, 4, 5, 0x43 | random walk in set 0xB / 0xC / 0xD (0x43: mode 2) | `0219A290(m, 0xC, set, mode)` | `0219A2B8` (5 states; `0219A4CC` / `0219A3D8` move-limit box for mode 1, unused) | +0 s16 state, +2 wait, +4 mode, +8 acmd base, +0xC set |
| 0xE..0x11 | always face up / down / left / right | `0219A528(m, dir)` | `0219A54C` | +0 dir, +4 done |
| 0x12, 0x13, 0x31 | spin on the spot (turn order by k) | `0219A5A4(m, k)` | `0219A5E0` (0x021D4DE8: `A604` / `A638` / `A660` / `A6CC`), `0219A75C` (0x021D4DB8: `A780` / `A7A8` / `A7C4` / `A7DC`) | +0 u8 order, +2 state, +4 counter |
| 0x14 | walk back and forth from the header dir | `0219A87C` | `0219A89C` (0x021D4D60: `A8C0` / `A900` / `AA08`) | +0 state, +1 reversed, +2 sight counter, +4 spin-look |
| 0x15..0x24 | walk a 4-leg route (`0219B120` ids 0xE..0x1D) | `0219AA38(m, 2, axis, route)` | `0219AB68` (0x021D4D04: `AB8C` / `ACE8`) | +0 state, +1 leg, +2 turn, +3 axis, +4 route, +5 sight counter, +8 spin-look |
| 0x25..0x2C | reversible route (ids 0x1E..0x25; pairs 0x25/0x2B, 0x26/0x2C, 0x27/0x2A, 0x28/0x29) | `0219AD18` | `0219ADC8` (0x021D4D78: `AE10` trainer sight / `AE68` reverse / `AEC8` / `B030`) | as above, +5 direction, +6 sight counter |
| 0x30 | follow the player | `0219B8BC` | `0219B8E8` (0x021D5374: `B924` / `B968`); delete `0219B920` | +0 state, +1 have pos, +2 / +4 player x / z, +6 0xFF |
| 0x32 | walk behind the partner trainer (same zone + trainer id) | `0219BAE0` | `0219BB08` (0x021D538C: `BB48` / `BB84`); delete `0219BB38`, return `0219BB3C` | as 0x30, +8 partner MMDL |
| 0x33..0x36 | with a field effect (kind 0 / 1 / 1 / 0), draw offset y -0x20000, move bit 0x8000 | `0219BDC4(m, k)` | `0219BE3C` (0x021D537C: `BED0` / `BF00`); delete `0219BE60`, return `0219BE70` | +0 state, +1 kind, +2 no-effect flag, +4 effect handle |
| 0x37..0x3E | mirror the player's facing (k 0 / 1; the dir argument is ignored) | `0219BF60(m, dir, k)` | `0219BFDC` (0x021D53A0: `C000` / `C034` / `C04C` / `C080` / `C0AC`) | +0 state, +1 copied dir (9 = none), +2 k |
| 0x3F..0x42 | walk along a wall (left / right hand; mode 2 may turn back) | `0219C0D4(m, mode, hand)` | `0219C118` (0x021D5394: `C13C` / `C150` -> `0219C338` / `C164`) | +0 state, +4 mode, +8 hand |

Event-type procs (by MMDL event type, mvproc3c; sub work MMDL +0xA4): init `0219B440` / busy `0219B45C` / spin `0219B478` through the
21-entry tables 0x021D5278 / 0x021D52CC / 0x021D5320: types 4 / 0xE -> `0219B508` / `0219B520` / `0219B59C`, types 5 / 6 / 0xF / 0x10 /
0x14 -> `0219B6B0` / `0219B6E0` / `0219B75C` (look around after header param 1 steps, 2-row / 4-dir turn orders 0x021D5248 / 0x021D5258),
all others `0219B4FC` / `0219B500` / `0219B504` (nothing / 0 / 0). `0219B41C` (busy && spin) keeps mmdl_move from running the move proc.

### Notes for the porter

* Every init gets its work through `0216D684(m, size)` / `0216D6A0(m, size)` (the real ones clear; the stubs do not, so fields an init
  leaves to the clear stay random in the tests). A port must call them and write exactly the fields the original writes.
* The state procs are reached through ROM tables by an unchecked state byte (`u8` or `s8`); a port embeds the tables (address lists)
  and dispatches with `env.call(table[state])` for unported states. The dispatch loop re-reads the state after every call.
* Direction sets and routes are ROM tables behind `0219B120(id)` (pairs at 0x021D50F8; ids 0..0xD and 0x26 end in 9, 0xE..0x25 are 4-dir
  routes without a terminator). Routes are indexed by a leg byte that can reach 4 (`0219AB8C` / `0219AEC8` read the next ROM words then):
  a port embeds the whole block 0x021D4D0C..0x021D4F8C. `0219B0D8` loops until the terminator with no bound.
* Unchecked indexes the specs bound: dir 0..3 for the turn-order copies (`0219A6CC` / `0219A7DC` read the stack past the copy
  otherwise), `0219B390` row (p[1]) 0..1, `0219B59C` sub+5 / sub+6 0..1, `0219B75C` sub+5 0..1, the wall tables of `0219C180` /
  `0219C20C` (dir 0..3, hand 0..1), event type 0..0x14 for the three event-type tables, state bytes to their table size.
* `0219A3D8` leaves the rectangle unwritten for move codes 0..5 and above 0xD; `0219A4CC` then compares uninitialised stack. It is only
  reachable with random-walk mode 1, which no record sets (the spec draws codes 6..0xD for it; UNVERIFIED outside).
* `0219BF60` ignores its dir argument (r1 is overwritten by the work size before use) and sets display dir 0.
* `0219AE10` passes the display dir to `02197E7C` without masking; `0219AE68` writes `w+1` after `0216E0E8` (change move code), which
  re-inits the procs and may clear the work first.
* `0219C080` reads the player's move dir without a null check (`0216D608(0)` when there is no player).
* `0219BA2C` returns the follower's own acmd code (mapped 0x58..0x5B -> 0x10..0x13); the player lookup it makes is discarded.
* Pad bits: `0219B060` / `0219B8A8` test bit 2 of the ARM9 trigger bits `0203630C`, `0219B0C4` bit 2 of the held bits `020362DC`.

### Traps

* rizin shows the literal `0x0219BF61` of the thunks `0219BF7C..` as a Thumb-2 `itttt vs` and conditionalises the following
  instructions (`ldrvs`, `pushvs`, `blvs` in `0219BF88..0219BFE0`): they are plain Thumb-1 (`ldr` / `movs` / `bx`, `push` / `bl`).
* `0219B074`'s jump table (`subs r1, 0x58; bmi; add pc`) is not the usual `cmp; bhi` shape; all its cases go to `return 1`.
* Polled in-area stubs that one function calls twice with the same arguments need `seq` (`0219C338` calls `0219C314` up to three times
  with the same `(m, &dir, hand)`; without `seq` the third answer always equals the second).

Mutation spot check (`check-fn.sh --fn`, reverted; `/tmp/blk/f4bs3/mut.sh`): 10 of 10 caught (`next_dir` wrap index 1 -> 0, `spin_wait`
`<` -> `<=`, `face_move` dir mask 0xFFFF -> 0xFF, `walks_fast_near` `<= 4` -> `< 4`, `spin_order` 0x11 -> 0x12, `spins_while_walking`
`<= 1` -> `<= 2`, `follower_base` `0..=3` -> `0..=2`, `moved` old z -> old x, `mirror_init` 9 -> 8, `effect_delete` `!= 0` -> `!= 1`).
