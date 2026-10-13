# black-script

Event script VM and script commands.

Undecompiled callees go in `hooks` as traits (§4 porting pattern); pointers stay `u32` addresses read and written through
`hooks::Mem` (same model as `black_field::hooks`).

## Areas and checks

| area | original | spec | check | status |
|---|---|---|---|---|
| `vm` | `vm.c`, main ARM9 0x020111EC-0x020113D8 (16 fns) | `tools/rom-diff/specs/black-script/vm.spec` | `tools/check-fn.sh black-script vm` | **16/16 verified, area hash f1bf3db7d540c749** (F7) |
| `script_sys` | `script_sys.c` + `script_work.c`, overlay 10 0x02158804-0x021591FE, 0x02159960-0x02159B48, scrcmd ctx getters 0x0215A5CC-0x0215A5FA (90 fns) | `tools/rom-diff/specs/black-script/script_sys.spec` | `tools/check-fn.sh black-script script_sys` | **90/90 verified, area hash f4bb4f39ea9115b6** (F7) |
| `scrcmd_core` | scrcmd core: overlay 10 0x02159204-0x0215995E, commands 0..48 + 4 helpers (53 fns) | `tools/rom-diff/specs/black-script/scrcmd_core.spec` | `tools/check-fn.sh black-script scrcmd_core` | 53 specced (F8-spec1, 747/747 insns covered), **5 smoke ports verified, area hash 8571c7a4a83df68b** |
| `scrcmd_work` | `scrcmd_work.c`: overlay 10 0x0215A514-0x0215C63A minus the ctx getters and the Heap_Free veneer (130 fns, 86 command ids) | `tools/rom-diff/specs/black-script/scrcmd_work.spec` | `tools/check-fn.sh black-script scrcmd_work` | 130 specced (F8-spec1, 3451/3451 insns covered), **5 smoke ports verified, area hash 935b80ee13c13582** |
| `scrcmd_fld` | overlay-21 field commands 0x021AE948-0x021B0083 (F1 row 21.42, first part; 97 fns, 80 command ids) | `tools/rom-diff/specs/black-script/scrcmd_fld.spec` | `tools/check-fn.sh black-script scrcmd_fld` | 97 specced (F8-spec2, 2360/2360 insns covered), **6 smoke ports verified, area hash 3a3b6fd5c3a80131** |
| `scrcmd_talk` | overlay-21 message / yes-no / menu commands 0x021B0084-0x021B1667 (rows 21.42 tail, 21.43, 21.44 head; 70 fns, 33 ids) | `tools/rom-diff/specs/black-script/scrcmd_talk.spec` | `tools/check-fn.sh black-script scrcmd_talk` | **70/70 verified, area hash f52686b3e04f16d2** (F8b; F8-spec2 spec, 2246/2246 insns covered) |
| `scrcmd_obj` | overlay-21 field-object / camera commands 0x021B1668-0x021B282F (row 21.44; 63 fns, 55 ids) | `tools/rom-diff/specs/black-script/scrcmd_obj.spec` | `tools/check-fn.sh black-script scrcmd_obj` | 63 specced (F8-spec2, 1809/1827: 021b1e80 18 behind an uninitialised-register path), **5 smoke ports verified, area hash 719910d5290037ee** |
| `scrcmd_screen` | overlay-21 screen effects 0x021B2830-0x021B2FD7 (row 21.44 tail, 21.45 `scrcmd_screeneffects.c`, cmd 594; 36 fns, 29 ids) | `tools/rom-diff/specs/black-script/scrcmd_screen.spec` | `tools/check-fn.sh black-script scrcmd_screen` | **36/36 verified, area hash a8af20a1036b361b** (F8b; F8-spec2 spec, 773/773 insns covered) |

| `scrcmd_fld` | overlay-21 field commands 0x021AE948-0x021B0083 (F1 row 21.42, first part; 97 fns, 80 command ids) | `tools/rom-diff/specs/black-script/scrcmd_fld.spec` | `tools/check-fn.sh black-script scrcmd_fld` | **97/97 verified, area hash c4d23e2d7aaf61b2** (F8c; F8-spec2 spec, 2360/2360 insns covered) |
| `scrcmd_talk` | overlay-21 message / yes-no / menu commands 0x021B0084-0x021B1667 (rows 21.42 tail, 21.43, 21.44 head; 70 fns, 33 ids) | `tools/rom-diff/specs/black-script/scrcmd_talk.spec` | `tools/check-fn.sh black-script scrcmd_talk` | 70 specced (F8-spec2, 2246/2246), **6 smoke ports verified, area hash 03a53d82f880c36f** |
| `scrcmd_obj` | overlay-21 field-object / camera commands 0x021B1668-0x021B282F (row 21.44; 63 fns, 55 ids) | `tools/rom-diff/specs/black-script/scrcmd_obj.spec` | `tools/check-fn.sh black-script scrcmd_obj` | **63/63 verified, area hash c450ae3e75c7410b** (F8c; F8-spec2 spec, 1809/1827: 021b1e80 18 behind an uninitialised-register path) |
| `scrcmd_screen` | overlay-21 screen effects 0x021B2830-0x021B2FD7 (row 21.44 tail, 21.45 `scrcmd_screeneffects.c`, cmd 594; 36 fns, 29 ids) | `tools/rom-diff/specs/black-script/scrcmd_screen.spec` | `tools/check-fn.sh black-script scrcmd_screen` | 36 specced (F8-spec2, 773/773), **5 smoke ports verified, area hash 6e9a0a809f9d93e1** |

| `scrcmd_screen` | overlay-21 screen effects 0x021B2830-0x021B2FD7 (row 21.44 tail, 21.45 `scrcmd_screeneffects.c`, cmd 594; 36 fns, 29 ids) | `tools/rom-diff/specs/black-script/scrcmd_screen.spec` | `tools/check-fn.sh black-script scrcmd_screen` | 36 specced (F8-spec2, 773/773), **5 smoke ports verified, area hash 6e9a0a809f9d93e1** |
| `scrcmd_proc` | `scrcmd_proc.c`: overlay 10 0x0215C63C-0x0215CE97 (F1 row 10.15 minus its tail; 22 fns, 17 command ids) | `tools/rom-diff/specs/black-script/scrcmd_proc.spec` | `tools/check-fn.sh black-script scrcmd_proc` | **22/22 verified, area hash 3ec48c52d531838d** (F8d; F8-spec3 spec, 824/824 insns covered) |
| `scrcmd_sodateya` | `scrcmd_sodateya.c`: overlay 10 0x0215CE98-0x0215D34B (row 10.16 + the tail of 10.15; 18 fns, 15 ids) | `tools/rom-diff/specs/black-script/scrcmd_sodateya.spec` | `tools/check-fn.sh black-script scrcmd_sodateya` | **18/18 verified, area hash 6f0e40c19a57626b** (F8d; F8-spec3 spec, 463/463) |
| `scrcmd_misc10` | overlay 10 rows 10.33 (0x0216A420-0x0216AC3F) and 10.35 (0x0216B504-0x0216B86B) + cmd 359 (0x0215D34C, `scrcmd_musical.c`, X2); 33 fns, 30 ids | `tools/rom-diff/specs/black-script/scrcmd_misc10.spec` | `tools/check-fn.sh black-script scrcmd_misc10` | 33 specced (F8-spec3, 1305/1305); **32/32 in-scope fns verified, area hash 6b303e0a9940ca36** (F8d); cmd 359 (X2) specced only |
| `scrcmd` | F8 index (doc only: which area holds which commands) | | | |

| `scrcmd_core` | scrcmd core: overlay 10 0x02159204-0x0215995E, commands 0..48 + 4 helpers (53 fns) | `tools/rom-diff/specs/black-script/scrcmd_core.spec` | `tools/check-fn.sh black-script scrcmd_core` | **53/53 verified, area hash 3467bfc999c8890b** (F8a; F8-spec1 spec, 747/747 insns covered) |
| `scrcmd_work` | `scrcmd_work.c`: overlay 10 0x0215A514-0x0215C63A minus the ctx getters and the Heap_Free veneer (130 fns, 86 command ids) | `tools/rom-diff/specs/black-script/scrcmd_work.spec` | `tools/check-fn.sh black-script scrcmd_work` | **130/130 verified, area hash d7e9a9ae52796e75** (F8a; F8-spec1 spec, 3451/3451 insns covered) |
| `scrcmd` | F8 index + the command-id -> handler table of the ported commands (`commands`, `handler`, `dispatch`, `call_wait`) | | `cargo test -p black-script scrcmd` (ids unique, every ported id dispatches) | 261 ids (49 core, 86 work, 33 talk, 29 screen, 17 proc, 15 sodateya, 29 misc10, 3 script_sys) |

| `scrcmd` | F8 index + the command-id -> handler table of the ported commands (`commands`, `handler`, `dispatch`, `call_wait`) | | `cargo test -p black-script scrcmd` (ids unique, every ported id dispatches) | 273 ids (49 core, 86 work, 80 fld, 55 obj, 3 script_sys) |

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
| `CommandDispatch::command(handler, vm, ctx)` | `table[cmd](vm, ctx)` at 0x02011312 | handler = word of the table at `VM+4` (0x0217062C: `script_sys::CMD_TABLE`, 609 entries); `scrcmd::dispatch` maps the addresses of commands 0..48 and the 86 `scrcmd_work` ids to their ports, `script_sys::call_known` has 49 / 50 / 311 (see "Command dispatch (F8a)") |
| `VmHooks::call_wait(f, vm, ctx)` | `(VM+0x10)(vm, ctx)` at 0x020112B8 | script_sys callbacks 0x02159961 / 0x02159985 / 0x021599A9 = `wait_pad_a` / `wait_pad_b` / `wait_311`; `scrcmd::call_wait`: 0x02159255 (command 3, `scrcmd_core::wait_countdown`), 0x021595A1 (command 28, `wait_call_script`); the menu polls 0x0215A954 / 0x0215A990 are `scrcmd_work::menu_poll` / `menu_poll_x` (called by overlay-21 commands) |
| `VmHooks::call_hook(f, vm, ctx, arg, cmd)` | `(VM+0x24)(vm, ctx, VM+0x28, cmd)` at 0x020112FC | scripts install 0x02158DE9 = `script_sys::dispatch_hook` (arg = GAMESYS) |
| `script_sys::call_known(h, f, args)` | — | helper for implementors: routes the 8 code addresses of this area (waits, commands 49/50/311, the hook, the event fn) to their ports |
| `ScriptSysHooks::fun_0201xxxx` (vm.c) | 0x0201121C .. 0x020113D0 | default to the `vm` ports; the `script_sys` test overrides them with the spec's stubs (the two areas are verified separately) |
| `ScriptSysHooks::call_end_callback` | `(SYS+0x10)(x, SYS+0x14)` at 0x021588BA | installed value 0x0215889D = veneer to main 0x0201235C (unported) |
| `ScriptSysHooks::call_init_proc` | `proc(task, task+0xC)` at 0x021591DC | the 14 procs of `script_sys::INIT_PROCS` (0x021705F0; overlay 10/21 code, F8/F10) |
| `ScriptSysHooks::fun_*` (the rest) | named by address | game data / event work (most ported in `black_field::game_data` / `event_work` / `zonedata`), text (`black_text::wordset` / `strbuf`), ARC loading, field map (ov21), scrcmd_work.c (F8) |
| `ScriptHost::pad_held` | `FUN_020362DC()` | via `ScriptSysHooks::fun_020362dc`; read by commands 49 / 50 (verified) |
| `ScriptHost::message_bank_open` | `FUN_0215A760(ctx, type, msg)` | via `ScriptSysHooks::fun_0215a760`; called by `vm_new` for ctx mode 0 (verified call site) |
| `ScriptHost::menu_start` / `menu_result` | `0x0215A77C` begin / `0x0215A7F0` add item / `0x0215A864` open / `0x0215A954`, `0x0215A990` poll (scrcmd_work.c) | `MenuRequest` follows the builder (F8-spec1); called by overlay-21 menu commands (F8 job 21.42) |
| `ScriptHost::message_*`, `play_se`, `yes_no_*`, `fade_*` | overlay-21 `scrcmd_talk` (talk / balloon / system windows, wait-button command 75, yes / no command 71) and `scrcmd_screen` (fade.c `0x0204E720` / `0x0204E804`, commands 179 / 180 and the field fades 429..440) | request structs and poll semantics from the F8-spec2 call sites (`host.rs` table); the overlay-21 window routines themselves are render side (UNVERIFIED meaning) |

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

## F8 areas (F8-spec1)

* **scrcmd_core** (commands 0..48): VM control (0/1 no-op, 2/29 end, 3 wait N frames, 4/30 call/jump relative, 5 return,
  27 start a parallel script, 28 call a script and wait, 31/32 conditional jump/call), an expression stack on the vm.c call stack
  (8/9 push, 10/11 pop, 12..15 + - * /, 16 push flag, 17 compare op), VM regs (20..22 set, 23/24 compare), var compares (25/26),
  event flags (35 set, 36 clear, 37 get), var arithmetic (18 &=, 19 |=, 38 +=, 39 -=, 40..42 =, 43 *=, 44 /=, 45 %=), 46..48 misc.
* **scrcmd_work** (86 ids): SCRCMD WORK (0x110) / CTX (0x24) constructors and accessors, 8 movement-object slots, the choice-menu
  builder, system / Pokedex / time / party / box queries and setters, give Pokemon / egg. Layouts in the spec header.
* Both specs list every command id with its handler in the header. Roles are read from the code; names of the main / overlay-21
  callees (PokeParam ids, Pokedex, party) are inferred (UNVERIFIED unless the spec says otherwise).
* F8a ported both areas completely. Callees are trait methods named by address: `scrcmd_core::CoreHooks` (vm.c, the
  scrcmd ctx getters, script_sys, the scrcmd_work helpers command 3 / 28 use, main, overlay 21) and `scrcmd_work::WorkHooks:
  CoreHooks` (the rest; the shared callees come from `CoreHooks`). Callees that need only memory default to their ports
  (vm.c `vm::*`, the `script_sys::ctx_*` getters, `scrcmd_work::set_countdown` / `countdown` / `set_wait_slot` /
  `wait_slot`); each area's test overrides them with the spec's stubs. In-area callees are called as Rust (the ROM side runs
  them as real code). ROM tables the commands index are embedded and compared with the ROM build by the tests:
  `scrcmd_core::COND` 0x021705DC, `scrcmd_work::{MENU_HEADER 0x02171140, TABLE_562 0x02171160, TABLE_563 0x02171170,
  TABLE_281 0x02171180, PARAM_WHITELIST 0x02171182, IV_TABLE 0x0217118E}`, `scrcmd_work::VERSION` (main 0x0209DF08).
* Stack records the original passes to callees are byte arrays (`&mut [u8; N]` for outs, `&[u8; N]` for reads): the
  0x24-byte give-Pokemon record `scrcmd_work::GiveMonRec`, the 0x1C-byte location record of commands 219 / 220, the
  0x20-byte list-menu header, the 2-byte cursor index, the time records of commands 212 / 214 / 312.

### Command dispatch (F8a)

`scrcmd::commands()` is the table of every ported command id -> handler address (261 ids: `scrcmd_core::COMMANDS` 0..48,
`scrcmd_work::COMMANDS` (86 ids), `scrcmd_talk::COMMANDS` (33), `scrcmd_screen::COMMANDS` (29), `scrcmd_proc::COMMANDS` (17),
`scrcmd_sodateya::COMMANDS` (15), `scrcmd_misc10::COMMANDS` (29), `scrcmd::SCRIPT_SYS_COMMANDS` 49 / 50 / 311; each area test
checks its words against the ROM table 0x0217062C). `scrcmd::handler(id)` looks one up. A
game-side `CommandDispatch` implementation is:

`scrcmd::commands()` is the table of every ported command id -> handler address (273 ids: `scrcmd_core::COMMANDS` 0..48,
`scrcmd_work::COMMANDS` (86 ids), `scrcmd_fld::COMMANDS` (80, F8c), `scrcmd_obj::COMMANDS` (55, F8c),
`scrcmd::SCRIPT_SYS_COMMANDS` 49 / 50 / 311; each area test checks its words against the ROM table 0x0217062C). `scrcmd::handler(id)` looks one up. A game-side `CommandDispatch` implementation is:

```rust,ignore
impl CommandDispatch for Game {
    fn command(&mut self, handler: u32, vm: u32, ctx: u32) -> u32 {
        scrcmd::dispatch(self, handler, vm, ctx)                       // core, work, talk, screen ids

        scrcmd::dispatch(self, handler, vm, ctx)                       // core 0..48, work, fld, obj

        scrcmd::dispatch(self, handler, vm, ctx)                       // core, work, talk, screen, proc, sodateya, misc10
            .or_else(|| script_sys::call_known(self, handler, &[vm, ctx])) // 49 / 50 / 311
            .unwrap_or_else(|| todo!("command handler {handler:#x}: unported F8 job"))
    }
}
```

and `VmHooks::call_wait` likewise tries `scrcmd::call_wait` (commands 3 / 28 and the overlay-21 waits) then
`script_sys::call_known`. The overlay-21 commands hand three more kinds of code address to other systems:
`scrcmd::call_event` (the GMEVENT functions of the yes / no and menu events, `0x021B0085` / `0x021B015D`, for the event
system), `scrcmd::call_init_proc` (init procs `0x021B0631` .. `0x021B1605` = `script_sys::INIT_PROCS` slots 1..6 and 9, for
`ScriptSysHooks::call_init_proc`) and `scrcmd::call_task` (the field fade task `0x021B2F19`, added with `FUN_02030DA8`).
Commands of the remaining F8 jobs (scrcmd_fld, scrcmd_obj, `scrcmd_proc.c`, ...) are not in the table yet; `scrcmd_core`
command 46 forwards to the overlay-21 handler `0x021B1BB8` through `CoreHooks::fun_021b1bb8`.

and `VmHooks::call_wait` likewise tries `scrcmd::call_wait` then `script_sys::call_known`. Commands of the remaining F8 jobs
(overlay-21 message / yes-no / menu / fade files, `scrcmd_proc.c`, ...) are not in the table yet; `scrcmd_core` command 46
forwards to the overlay-21 handler `0x021B1BB8` through `CoreHooks::fun_021b1bb8` (ported: `scrcmd_obj::cmd_46_body`;
command 47's `CoreHooks::fun_021b1be8` is `scrcmd_obj::release_objects`), and `ScriptSysHooks::call_init_proc` routes the
init procs of mask bits 10 / 12 / 13 to `scrcmd_fld::init_proc`.

## F8 areas (F8-spec2, overlay 21)

* **scrcmd_fld** (0x021AE948..): trainer-eye / trainer-battle starts (127..148), field events and sounds (152..172, 584..607),
  the WORDSET commands 76..99 / 538 / 539, name / string queries 181..189, warps / map changes 190..196 / 602, 3 init procs.
* **scrcmd_talk** (0x021B0084..): yes / no (71), choice menus (173..178), the talk window (52..55, 63, 75), balloons over field
  objects (60..62, 72, 73, 135), system / info windows (51, 56..59, 64..70, 74, 579), 6 init procs. F1 row 21.43 ("event_trainer_eye.c")
  is not trainer-eye code: it is the balloon placement (camera projection + the ROM table 0x021D939C) of these commands.
* **scrcmd_obj** (0x021B1668..): movements (100 / 101), field objects (102..126, 577, 583, 588, 603), the talk lock (`0x021B1B08`,
  entered from the scrcmd_core thunk at 0x021598E4), and 384..410 / 608: wrappers of an overlay at 0x021F3xxx (outside the field set).
* **scrcmd_screen** (0x021B2830..): fades (179 / 180, 429..440 / 433), field 3D objects (292..302), child events 293 / 303..306 / 436, 594.
* The specs list every function with its command id(s) and a one-line role; every stub outside the area is `force` (overlay-21
  addresses collide with battle `PORTED` entries). The smoke ports call their callees through `FldHooks` / `TalkHooks` /
  `ObjHooks` / `ScreenHooks` (one method per callee address); globals 0x021EE71C (talk-lock flags) and 0x021EE720 (command 298) are
  absolute addresses in the `Mem` window.

## F8b: scrcmd_talk and scrcmd_screen ported

* Both areas are ported completely (70 + 36 functions, 33 + 29 command ids) and dispatched by `scrcmd`. `TalkHooks: CoreHooks`
  and `ScreenHooks: CoreHooks`: vm.c, the operand readers, the ctx getters, the countdown and the child-event call come from
  `CoreHooks`; the methods of the overlay-10 routines ported in this crate (`script_sys::sw_*` / `proc_mask_*` / `ctx_*`,
  the `scrcmd_work` accessors of SCRCMD WORK +0xD4..+0x105) default to those ports. Some main / overlay-21 callees
  (`FUN_02045CAC`, `FUN_0201F250`, `FUN_0201210C`, `FUN_02158F58`, the menu builder `0x0215A77C` ..) are methods of
  `WorkHooks` too: a type implementing both traits implements them twice (kept apart so the parallel F8 jobs do not edit each
  other's traits).
* In-area callees are called as Rust (the ROM side runs them as real code once they are `PORTED`); e.g. command 429's
  `scrcmd_screen::field_fade_start` (`0x021B2EBC`) is no longer a hook.
* Stack records passed to callees are byte arrays: camera vectors / matrices of the balloon placement (`&[u8; 12]`,
  `&[u8; 0x30]`), the object positions (`FUN_0216D88C`), the 3D-object grid position of commands 294..302, the size bytes of
  command 58. `scrcmd_talk::balloon_side` (`0x021B0A40`) takes the target position as `&[u8; 12]` (its callers pass a stack
  record).
* ROM data embedded and compared with the build by the tests: `scrcmd_talk::UP` (overlay-21 0x021D9390, fx32 (0, 1, 0)) and
  `scrcmd_talk::OFFSETS` (0x021D939C, 2 x 4 balloon offsets).
* ScriptHost: the window / fade hooks are where a game-side implementation issues the `ScriptHost` requests (mapping in
  `host.rs`; `MessageWindow` gained the info board, sized info window and the windows of commands 67 / 579 and 69 from their
  call sites).

### F8c port (scrcmd_fld / scrcmd_obj)

* Both areas are ported completely: `FldHooks: CoreHooks` / `ObjHooks: CoreHooks` (the vm.c operand readers, ctx getters,
  `0215897C` and the GMEVENT / GAMEDATA getters are shared with the scrcmd core; one method per other callee address).
  fldmmdl (F4a) and the overlay at 0x021F3xxx (commands 384..410 / 608) stay hooks by address, as the specs stub them.
* Stack records the original passes are byte arrays: VecFx32 positions `[u8; 12]` (warps 190..196 / 602, command 193's
  player / event-data positions, the object positions of 123 / 583), the 4-byte sound record of the command 171 event, the
  8-byte records of commands 119 / 124 / 125. ROM tables embedded and compared with the build by the tests:
  `scrcmd_obj::{DIRS_119 0x021D93FC, RECORDS_119 0x021D9404..0x021D941C, VEC_583 0x021D9424}`.
* The WORDSET thunks (commands 77 / 79 / 87 / 88 / 97 / 98: `ldr r2, =fn; ldr r3, =body; bx r3`) are the shared bodies
  `wordset_body_77` / `_87` / `_97` with the routine address; the `blx` goes through `FldHooks::call_wordset_fn(f, ..)`.
* Spec-comment errors found: 021af4e8 (command 171's event) passes `(a, b, ..)` to `FUN_02006984` in both modes (the header says
  `(b, a, ..)` for mode != 0); 021b1c2c (command 112): the first var gets the OBJ id, the second the found flag (the header says
  "found, id"). Command 603 (0x021B1DEC) is a byte-identical copy of command 118. Command 193 passes its second position record
  uninitialised when `FUN_0216282C` finds nothing (the port passes zeros, the Z19 test state).

## F8 areas (F8-spec3, overlay 10)

* **File boundaries** (from the `__FILE__` strings the commands pass to `Heap_AllocDebug`: 0x021728F0 "scrcmd_proc.c", 0x02172900
  "scrcmd_sodateya.c", 0x02172914 "scrcmd_musical.c"): `scrcmd_proc.c` = 0x0215C63C-0x0215CE97, `scrcmd_sodateya.c` =
  0x0215CE98-0x0215D34B (commands 235..237 sit in F1's row 10.15), `scrcmd_musical.c` starts at 0x0215D34C (cmd 359; F1 says
  0x0215D3EE). Cmd 359 is specced in `scrcmd_misc10` for completeness (X2, not to be ported).
* **scrcmd_proc**: the app-proc request `proc_call_wait` (0x0215C63C): proc work {param, cbwork, cb} in the SW + 0x3C slot,
  `FUN_020120B4(GAMESYS, ovl, proc table, param)` starts the proc, the VM waits on 0x0215C69D until `FUN_020120F0(GAMESYS)` is 0,
  then `cb(W)` copies the result into script vars (callbacks 0x0215C700 / 0x0215C7C4 / 0x0215C870). Commands 334 / 337 / 339 /
  341 / 370 use it directly, 336 / 489 through the app-proc events `FUN_02014840` / `FUN_02014808`; 383..406 call overlay 37
  with overlay 0x25 loaded around the call. Which app each overlay id is (0x46, 0x47, 0x88, 0xAD, 0xAE, 0xDD..0xE0) is not
  identified (G2 / U).
* **scrcmd_sodateya**: daycare object `FUN_02188D50(field map)` (overlay-21 `sodateya.c`, F13); slot -> PokeParam
  `FUN_021C34F8`; 247 runs the party-select event 0x0215D2A4 / 0x0215D1E4; 593 = `gender_mark` (0x0215D198).
* **scrcmd_misc10**: 506..520 call overlay 21 0x021D0Axx..0x021D1028 / overlay-10 events of `fld_btl_inst_tool.c` on
  `*FUN_02012F50(GAMEDATA)`; 549 / 550 move a field object (GMEVENTs 0x0216A838 straight, 0x0216A8F0 a curved path from
  FX_SinCosTable); 0x0216AAE4 = the seasonal forms of Deerling / Sawsbuck in the party; 528..543 save blocks 0x34 / 0x3F.
  Roles of the overlay-21 / save-block callees are UNVERIFIED (named by address).
* **ScriptHost**: no `host.rs` change. The app-proc start / poll (`FUN_020120B4` / `FUN_020120F0`) and the app-proc events
  (`FUN_02014808` / `FUN_02014840`: gamesys, field map, ovl, proc table, param, result callback, callback work) are the
  candidates for an app request (`ScriptHost::app_start(ovl, proc table, param)` / `app_running()`); the port job decides
  once a caller needs it.
* Smoke ports call their callees through `ProcHooks` / `SodateyaHooks` / `Misc10Hooks` (one method per callee address).

## F8d: scrcmd_proc, scrcmd_sodateya and scrcmd_misc10 ported

* All three areas are ported (22 + 18 + 32 functions; 17 + 15 + 29 command ids) and dispatched by `scrcmd`; cmd 359 is not
  (X2). `ProcHooks` / `SodateyaHooks` / `Misc10Hooks` extend `CoreHooks` (vm.c, the operand readers, the ctx getters,
  `FUN_020120F4` / `FUN_020122C0` / `FUN_020124E8` and the child-event call `FUN_0215897C` come from there); `ProcHooks`
  defaults `FUN_02158F50` / `FUN_02158FB4` / `FUN_02158FB8` to the `script_sys` ports. In-area callees are called as Rust
  (`proc_call_wait`, `party_select_event`, `gender_mark`).
* Code addresses handed to other systems: the app-proc wait `scrcmd_proc::PROC_WAIT` (in `scrcmd::call_wait`), the GMEVENT
  functions `scrcmd_sodateya::EVENT_247` and `scrcmd_misc10::EVENT_549` / `EVENT_550` (in `scrcmd::call_event`) and the
  result callbacks `scrcmd_proc::CB_334` / `CB_336` / `CB_489` (`scrcmd::call_callback`; the proc wait reaches them through
  `ProcHooks::call_proc_callback`, the app events of main through their own callback call).
* Heap destinations a callee fills (`FUN_02175648(gd, W + 0x2C)` 16 bytes, `FUN_0216D88C(mmdl, W + pos)` 12 bytes) are
  passed as addresses; stack records stay byte arrays (`FUN_020124AC`'s 8 / 16-byte record, command 399's 8 var addresses,
  command 559's out word `k`).
* ROM data: `scrcmd_misc10::{COUNTS_532 0x0217219C (32 B incl. 2 zero bytes after the 30 counts), ROWS_532 0x021721BC (30 x 17)}`;
  `FX_SinCosTable_` (main 0x020A1B38) is computed (`scrcmd_misc10::fx_sin` / `fx_cos`, checked against all 4096 ROM entries,
  same formula as `black_field::fieldmap::fx_cos`). Command 550's curved path inlines `_u32_div_f` / `_ll_mul` (svflow `ALLOW`)
  and the compiler's `% 0x10000` angle reduction (`step_index`).
* **App-proc requests (proposed mapping, not a `ScriptHost` method yet):** `FUN_020120B4(GAMESYS, ovl, proc table, param)`
  -> `ScriptHost::app_start(AppRequest { ovl, proc_table, param })`, `FUN_020120F0(GAMESYS) != 0` -> `ScriptHost::app_running()`
  (the proc wait `0x0215C69C` polls it, then runs the result callback). The app events `FUN_02014808` / `FUN_02014840(GAMESYS,
  field map, ovl, proc table, param, cb, cbwork)` (commands 336 / 489, and musical 359) are main-side GMEVENTs that leave the
  field, run the same kind of app, return and call `cb(cbwork)`: they map to the same pair once the event code (main, G2 /
  F10) is ported; until then `ProcHooks::fun_02014808` / `fun_02014840` are the seam. The `param` / result layouts are the
  commands' (doc of each `cmd_*`); which app each overlay id is (0x46, 0x47, 0x88, 0xAD, 0xAE, 0xDD..0xE0) is not identified
  (UNVERIFIED; G2 / U), so `host.rs` gets no method until a caller (G2) needs it.
* Mutation spot check: 5/6 killed; survivor `cmd_532` `best < s` -> `best <= s` (the `0200e4f8` / `0200e4b0` stubs answer
  mixed u32 values, so ties never happen: needs small `pick:` answers, harness queue).

## Notes for porters

* A `vm` function ported in `vm.rs` does not deactivate its stubs in `script_sys.spec` (stub deactivation is per area):
  `script_sys` ports keep calling vm.c through their hooks (`ScriptSysHooks::fun_02011330` etc.).
* `MIi_CpuFill32` (0x02082A7C), `MIi_CpuCopy16` (0x02082A60) and `MI_CpuFill8` (0x02082BCC) run as real code on the ROM
  side (svflow `ALLOW`): the ports inline them.
* `hooks::Mem` duplicates `black_field::hooks::Mem` (this crate does not depend on `black-field`).
* `0x02159B38` (`ctx_zone_id`) takes the ctx in r0, unlike the `(vm, ctx)` command helpers next to it.

## Deliberate deviations / UNVERIFIED

* scrcmd_core / scrcmd_work: command roles and callee names are read from the code; names of main / overlay-21 callees
  (PokeParam ids, Pokedex, party, options) are inferred (UNVERIFIED unless the doc says otherwise). ROM tables indexed by a
  script operand (`COND`, `TABLE_562` / `563` / `281`) panic past their end where the original reads the following bytes.
* `scrcmd_work::cmd_289` (0x0215B994): the jump table gives v 1 and 2 the same case (`FUN_020079A0`), v 3 the difference and
  v 5 the free places, v 4 = 0; the spec header comment lists them shifted by one from case 2 (the port follows the code).
* `scrcmd_work::cmd_270` / `cmd_291` read eight value operands (the spec header says nine).
* `scrcmd_work::cmd_222` (0x0215AEE8): the first operand is the mode, the second the species (the spec header names them the
  other way round).

* `script_sys::wait_311` (0x021599A8): for a status `FUN_02012DD0` > 3 the original returns its caller's r4 (never set on
  that path; in the game the VM step's caller's r4, e.g. the SYS slot index). The port returns
  `WAIT_311_STALE_R4` = 0, the value the differential test enters with.
* `scrcmd_talk::balloon_project` (0x021B0808): the column index divides by 12 through `0x0209C2B0` (inlined; the svflow
  `ALLOW` list runs it as real code); the jump-table fragments of 0x021B0084 / 0x021B015C / 0x021B03E4 / 0x021B0960 /
  0x021B09EC are `match`es. Command 58's `y + h > 0x18` edge is untested (the shared u16 operand stub rarely draws a y near
  0x18; mutation survivor).
* `scrcmd_obj::object_of` (0x021B1668): for id 0xF1 the original returns its caller's r6 (never set; command 100: the movement
  offset); the port takes it as `stale_r6`, and the test never generates 0xF1. The same kind of uninitialised-register path:
  0x021AE948 (r5 for a value other than 0 / 1) and 0x021B1E80 (r7 when no direction passes): not exercised (specs bound them).

* `scrcmd_obj::object_of` (0x021B1668): for id 0xF1 the original returns its caller's r6 (never set; its only caller, command
  100, has the movement offset there); the port takes it as `stale_r6` and `cmd_100` passes the offset (verified: the command
  100 block generates 0xF1 and runs 0x021B1668 as real code). The same kind of uninitialised-register path: `scrcmd_fld::cmd_128`
  (0x021AE948, r5 for a value other than 0 / 1) and `scrcmd_obj::cmd_119` (0x021B1E80, r7 when no direction passes) take
  `stale_r5` / `stale_r7`; their `dispatch` passes 0 (the Z19 entry value); not exercised (the specs bound the paths away).
* `script_lookup` (0x02158CD0): an id inside a range's `lo` but past its `hi` calls the assertion hook `FUN_02034714`; in
  retail it returns, and the port (like the original) then uses the entry anyway. The test models the hook as noreturn,
  so that continuation is not covered.
* `dispatch_hook` / `run_init_procs` index the embedded tables; ids past them (the original reads the following bytes)
  panic.
* `fatal` (0x02158DBC) loops on `FUN_020053EC` forever, as the original.
* `scrcmd_proc::cmd_356` (0x0215CC4C) stores its caller's r6 / r4 into the event record when an operand misses its cases
  (unset registers): the port takes them as `stale_r4` / `stale_r6`; `scrcmd_misc10::cmd_532` (0x0216B720) stores an unset
  stack word when no row wins: the port takes it as `stale`. `dispatch` passes 0 for all three (the Z19 entry state the test
  runs with), so a game running a script that hits these paths gets 0 where the original gets whatever its caller left.
* `scrcmd_misc10::event_549` divides by the frame count with Rust's `wrapping_div` (`_s32_div_f` inlined): a 0-frame move
  panics where the original's division-by-zero result is unknown (§8; the spec bounds frames to 1..0x40).
  `scrcmd_misc10::cmd_532` indexes the embedded tables with the script operand `c` (panics for `c` > 31; the original reads
  the following bytes).
* `scrcmd_misc10` command 359 (0x0215D34C, `scrcmd_musical.c`) is out of scope (X2): specced, not ported, not dispatched
  (`scrcmd::dispatch` returns `None` for it, like any unported command).
