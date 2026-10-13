# black-script

Event script VM and script commands.

Undecompiled callees go in `hooks` as traits (§4 porting pattern); pointers stay `u32` addresses read and written through
`hooks::Mem` (same model as `black_field::hooks`).

## Areas and checks

| area | original | spec | check | status |
|---|---|---|---|---|
| `vm` | `vm.c`, main ARM9 0x020111EC-0x020113D8 (16 fns) | `tools/rom-diff/specs/black-script/vm.spec` | `tools/check-fn.sh black-script vm` | **16/16 verified, area hash f1bf3db7d540c749** (F7) |
| `script_sys` | `script_sys.c` + `script_work.c`, overlay 10 0x02158804-0x021591FE, 0x02159960-0x02159B48, scrcmd ctx getters 0x0215A5CC-0x0215A5FA (90 fns) | `tools/rom-diff/specs/black-script/script_sys.spec` | `tools/check-fn.sh black-script script_sys` | **90/90 verified, area hash f4bb4f39ea9115b6** (F7) |
| `scrcmd` | F8 (command handlers), empty | | | |

Every function of both areas is specced with closure stubs (porting order needs no spec edit). The spec headers are the
reference for the layouts; summary:

* **VM** (`vm.c`, 0x30 + 4 * (stack + regs) bytes; scripts use stack 0x100, regs 0x40): +0 u16 stack words, +2 u16 reg
  words, +4 command table, +8 command count, +0xC u8 call depth, +0xD u8 status (0 stopped, 1 running, 2 waiting),
  +0x10 wait callback, +0x14 PC, +0x18 call stack, +0x1C regs (at +0x30), +0x20 ctx, +0x24 pre-dispatch hook, +0x28 hook
  arg, +0x2C script file (script_sys).
* **Dispatch** (`0x02011298`, one call per frame per VM): `cmd = u16 at PC`; `cmd >= count` stops; the hook
  (`0x02158DE8`: per-command permission byte table 0x02170374, bit = ctx mode 0..2, a missing bit is fatal) may stop;
  `table[cmd](vm, ctx)` returns 1 (yield) or 0 (next command). The table is at **0x0217062C** with 609 entries
  (count word 0x021705D8); the 15 words before it (0x021705F0) are a separate null-terminated table of 14 init procs
  (`0x021591B4`, gated by the mask at 0x02172BA0). So `tools/re/F1-SPLIT.md`'s "slot N" is command N - 15 and no command
  id is dispatched outside the table.
* **SYS** (`script_sys.c`, 0x18): heap, live VM count, 3 VM slots, end callback + arg. The script event's work is
  `{SW, SYS}`; its event function is `0x021588DC`.
* **SCRIPT_WORK** (`script_work.c`, 0x14C): magic 0x3643F, script id, MMDL, heap, GAMESYS, GMEVENT, field block,
  wordset + 2 strbufs, script vars 0x8000.. at +0x7C (u16), get/set words +0x13C..+0x144, +0x148 scrcmd work.
  Var ids: `< 0x4000` literal, `0x4000..0x7FFF` event-work var, `0x8000..0xBFFF` script var, `>= 0xC000` literal.
* **scrcmd CTX** (0x24, `scrcmd_work.c`): heap, zone id, script id, mode, mode != 0, SW, and +0x20 -> the scrcmd work
  `{SW, GAMESYS, GAMEDATA, ...}`; getters 0x0215A5F4 / 5DC / 5E4 return its words 0 / 1 / 2.
* **Script ids** -> (ARC 57 file, type, msg) through the 46-entry table 0x021701A8 (ids 2000..10524), zone-local ids
  1..1999 through the zone header.

## Runtime design: traits (F7)

```
black-game (Bevy) ──implements──► ScriptHost            (host.rs: UI requests: pad, message, yes/no, menu, fade)
glue (F12/G2)     ──implements──► ScriptSysHooks         (hooks.rs: game data, event work, field map, text, archives)
                                   └─ VmHooks            (wait callback / pre-dispatch hook function pointers)
                                       ├─ CommandDispatch (handler address -> F8 `scrcmd` port)
                                       └─ HeapHooks       (Heap_AllocDebug / Heap_Free)
```

Data flow per frame: the field's event loop calls the script event function (`script_sys::event_fn`, `0x021588DC`) →
`sys_step` (`0x0215882C`) → `vm::step` (`0x02011298`) for each of the 3 VM slots → per command `CommandDispatch::command`
(through the permission hook `VmHooks::call_hook` → `script_sys::dispatch_hook`). A command that needs the game starts a
`ScriptHost` request, installs a wait callback with `vm::wait` (`0x020113D0`) and yields; later steps poll it through
`VmHooks::call_wait` until it reports done. Nothing blocks; the host never calls into the VM.

| trait / method | original | notes |
|---|---|---|
| `HeapHooks::heap_alloc` / `heap_free` | `Heap_AllocDebug` 0x02030734 / `Heap_Free` 0x020307B0 (+ veneers 0x02011264, 0x02158824, 0x0215A554) | same as `black_field::hooks::HeapHooks` |
| `CommandDispatch::command(handler, vm, ctx)` | `table[cmd](vm, ctx)` at 0x02011312 | handler = word of the table at `VM+4` (0x0217062C: `script_sys::CMD_TABLE`, 609 entries); F8 maps addresses to ports; 49 / 50 / 311 are `script_sys::cmd_wait_pad_a` / `cmd_wait_pad_b` / `cmd_311` |
| `VmHooks::call_wait(f, vm, ctx)` | `(VM+0x10)(vm, ctx)` at 0x020112B8 | script_sys callbacks 0x02159961 / 0x02159985 / 0x021599A9 = `wait_pad_a` / `wait_pad_b` / `wait_311`; the rest are F8's |
| `VmHooks::call_hook(f, vm, ctx, arg, cmd)` | `(VM+0x24)(vm, ctx, VM+0x28, cmd)` at 0x020112FC | scripts install 0x02158DE9 = `script_sys::dispatch_hook` (arg = GAMESYS) |
| `script_sys::call_known(h, f, args)` | — | helper for implementors: routes the 8 code addresses of this area (waits, commands 49/50/311, the hook, the event fn) to their ports |
| `ScriptSysHooks::fun_0201xxxx` (vm.c) | 0x0201121C .. 0x020113D0 | default to the `vm` ports; the `script_sys` test overrides them with the spec's stubs (the two areas are verified separately) |
| `ScriptSysHooks::call_end_callback` | `(SYS+0x10)(x, SYS+0x14)` at 0x021588BA | installed value 0x0215889D = veneer to main 0x0201235C (unported) |
| `ScriptSysHooks::call_init_proc` | `proc(task, task+0xC)` at 0x021591DC | the 14 procs of `script_sys::INIT_PROCS` (0x021705F0; overlay 10/21 code, F8/F10) |
| `ScriptSysHooks::fun_*` (the rest) | named by address | game data / event work (most ported in `black_field::game_data` / `event_work` / `zonedata`), text (`black_text::wordset` / `strbuf`), ARC loading, field map (ov21), scrcmd_work.c (F8) |
| `ScriptHost::pad_held` | `FUN_020362DC()` | via `ScriptSysHooks::fun_020362dc`; read by commands 49 / 50 (verified) |
| `ScriptHost::message_bank_open` | `FUN_0215A760(ctx, type, msg)` | via `ScriptSysHooks::fun_0215a760`; called by `vm_new` for ctx mode 0 (verified call site) |
| `ScriptHost::message_*`, `yes_no_*`, `menu_*`, `fade_*` | F8 command handlers (not located yet) | planned interface (request + poll, see `host.rs`); F8 fixes the request fields |

## Symbols

Full table (generated from the `/// C:` comments): `SYMBOLS.md`. vm.c: `vm::{reset, create, init, start, stop, step, ctx,
set_hook, read_u16, read_u32, push, pop, jump, call, ret, wait}` (0x020111EC .. 0x020113D0, in address order).
script_sys.c / script_work.c: `sys_*` (SYS), `start_event` / `event_fn` / `sw_start_script` (script events),
`run_sync*` / `run_special*` / `run_conditional` / `find_special` / `find_conditional` (synchronous and zone scripts),
`vm_new` / `vm_free` / `script_lookup` / `load_script_file` / `dispatch_hook` / `fatal` / `install_hook` (VM setup),
`sw_*` (SCRIPT_WORK), `var_addr` / `var_value` / `set_var` / `save_vars` / `restore_vars` (vars), `proc_mask_*` /
`run_init_procs` (init procs), commands 49 / 50 / 311 and their waits, `cmd_var_addr` / `cmd_var_value` (operand
helpers 0x02159AE8 / 0x02159B10), `ctx_*` (scrcmd ctx getters 0x0215A5CC .. 0x0215A5F4).

ROM tables embedded in `script_sys/tables.rs` (copied from overlay 10 data): `CMD_PERMISSIONS` (0x02170374, 609 B),
`SCRIPT_RANGES` (0x021701A8, 46 × 10 B), `INIT_PROCS` (0x021705F0, 15 words). Constants: `CMD_TABLE` 0x0217062C,
`CMD_COUNT` 609 (the word at 0x021705D8).

## Notes for porters

* A `vm` function ported in `vm.rs` does not deactivate its stubs in `script_sys.spec` (stub deactivation is per area):
  `script_sys` ports keep calling vm.c through their hooks (`ScriptSysHooks::fun_02011330` etc.).
* `MIi_CpuFill32` (0x02082A7C), `MIi_CpuCopy16` (0x02082A60) and `MI_CpuFill8` (0x02082BCC) run as real code on the ROM
  side (svflow `ALLOW`): the ports inline them.
* `hooks::Mem` duplicates `black_field::hooks::Mem` (this crate does not depend on `black-field`).
* `0x02159B38` (`ctx_zone_id`) takes the ctx in r0, unlike the `(vm, ctx)` command helpers next to it.

## Deliberate deviations / UNVERIFIED

* `script_sys::wait_311` (0x021599A8): for a status `FUN_02012DD0` > 3 the original returns its caller's r4 (never set on
  that path; in the game the VM step's caller's r4, e.g. the SYS slot index). The port returns
  `WAIT_311_STALE_R4` = 0, the value the differential test enters with.
* `script_lookup` (0x02158CD0): an id inside a range's `lo` but past its `hi` calls the assertion hook `FUN_02034714`; in
  retail it returns, and the port (like the original) then uses the entry anyway. The test models the hook as noreturn,
  so that continuation is not covered.
* `dispatch_hook` / `run_init_procs` index the embedded tables; ids past them (the original reads the following bytes)
  panic.
* `fatal` (0x02158DBC) loops on `FUN_020053EC` forever, as the original.
