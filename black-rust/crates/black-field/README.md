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
