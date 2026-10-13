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
| `scrcmd_talk` | overlay-21 message / yes-no / menu commands 0x021B0084-0x021B1667 (rows 21.42 tail, 21.43, 21.44 head; 70 fns, 33 ids) | `tools/rom-diff/specs/black-script/scrcmd_talk.spec` | `tools/check-fn.sh black-script scrcmd_talk` | 70 specced (F8-spec2, 2246/2246), **6 smoke ports verified, area hash 03a53d82f880c36f** |
| `scrcmd_obj` | overlay-21 field-object / camera commands 0x021B1668-0x021B282F (row 21.44; 63 fns, 55 ids) | `tools/rom-diff/specs/black-script/scrcmd_obj.spec` | `tools/check-fn.sh black-script scrcmd_obj` | 63 specced (F8-spec2, 1809/1827: 021b1e80 18 behind an uninitialised-register path), **5 smoke ports verified, area hash 719910d5290037ee** |
| `scrcmd_screen` | overlay-21 screen effects 0x021B2830-0x021B2FD7 (row 21.44 tail, 21.45 `scrcmd_screeneffects.c`, cmd 594; 36 fns, 29 ids) | `tools/rom-diff/specs/black-script/scrcmd_screen.spec` | `tools/check-fn.sh black-script scrcmd_screen` | 36 specced (F8-spec2, 773/773), **5 smoke ports verified, area hash 6e9a0a809f9d93e1** |
| `scrcmd` | F8 index (doc only: which area holds which commands) | | | |

| `scrcmd_core` | scrcmd core: overlay 10 0x02159204-0x0215995E, commands 0..48 + 4 helpers (53 fns) | `tools/rom-diff/specs/black-script/scrcmd_core.spec` | `tools/check-fn.sh black-script scrcmd_core` | **53/53 verified, area hash 3467bfc999c8890b** (F8a; F8-spec1 spec, 747/747 insns covered) |
| `scrcmd_work` | `scrcmd_work.c`: overlay 10 0x0215A514-0x0215C63A minus the ctx getters and the Heap_Free veneer (130 fns, 86 command ids) | `tools/rom-diff/specs/black-script/scrcmd_work.spec` | `tools/check-fn.sh black-script scrcmd_work` | **130/130 verified, area hash d7e9a9ae52796e75** (F8a; F8-spec1 spec, 3451/3451 insns covered) |
| `scrcmd` | F8 index + the command-id -> handler table of the ported commands (`commands`, `handler`, `dispatch`, `call_wait`) | | `cargo test -p black-script scrcmd` (ids unique, every ported id dispatches) | 138 ids (49 core, 86 work, 3 script_sys) |

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

`scrcmd::commands()` is the table of every ported command id -> handler address (138 ids: `scrcmd_core::COMMANDS` 0..48,
`scrcmd_work::COMMANDS` (86 ids), `scrcmd::SCRIPT_SYS_COMMANDS` 49 / 50 / 311; each area test checks its words against the ROM
table 0x0217062C). `scrcmd::handler(id)` looks one up. A game-side `CommandDispatch` implementation is:

```rust,ignore
impl CommandDispatch for Game {
    fn command(&mut self, handler: u32, vm: u32, ctx: u32) -> u32 {
        scrcmd::dispatch(self, handler, vm, ctx)                       // commands 0..48 + scrcmd_work ids
            .or_else(|| script_sys::call_known(self, handler, &[vm, ctx])) // 49 / 50 / 311
            .unwrap_or_else(|| todo!("command handler {handler:#x}: unported F8 job"))
    }
}
```

and `VmHooks::call_wait` likewise tries `scrcmd::call_wait` then `script_sys::call_known`. Commands of the remaining F8 jobs
(overlay-21 message / yes-no / menu / fade files, `scrcmd_proc.c`, ...) are not in the table yet; `scrcmd_core` command 46
forwards to the overlay-21 handler `0x021B1BB8` through `CoreHooks::fun_021b1bb8`.

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
* `scrcmd_obj::object_of` (0x021B1668): for id 0xF1 the original returns its caller's r6 (never set; command 100: the movement
  offset); the port takes it as `stale_r6`, and the test never generates 0xF1. The same kind of uninitialised-register path:
  0x021AE948 (r5 for a value other than 0 / 1) and 0x021B1E80 (r7 when no direction passes): not exercised (specs bound them).
* `script_lookup` (0x02158CD0): an id inside a range's `lo` but past its `hi` calls the assertion hook `FUN_02034714`; in
  retail it returns, and the port (like the original) then uses the entry anyway. The test models the hook as noreturn,
  so that continuation is not covered.
* `dispatch_hook` / `run_init_procs` index the embedded tables; ids past them (the original reads the following bytes)
  panic.
* `fatal` (0x02158DBC) loops on `FUN_020053EC` forever, as the original.
