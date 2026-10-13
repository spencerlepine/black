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


## F6a: `wfbc_data` (40/40), `eventdata` (56/56), `mmdl_move` (70/70), `eventdata_tail` (18/19) ported

| area | ported | verified (in `PORTED`) | area hash | unverified ports |
|---|---|---|---|---|
| `wfbc_data` | 40 | 40 | `721154b28c4c24fa` | |
| `eventdata` | 56 | 56 | `e2fedff5ec7feede` | |
| `mmdl_move` | 70 | 70 | `854080fe99c7c86c` | |
| `eventdata_tail` | 19 | 18 | `366498360545b869` | `02164708` `hidden_item_flag` (below) |

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
verified through its `force` stub.

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
attribute; a `seq` on that stub would separate them).

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
