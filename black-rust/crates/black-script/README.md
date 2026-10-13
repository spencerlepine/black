# black-script

Event script VM and script commands.

Undecompiled callees go in `hooks` as traits (§4 porting pattern); pointers stay `u32` addresses read and written through
`hooks::Mem` (same model as `black_field::hooks`).

## Areas and checks

| area | original | spec | check |
|---|---|---|---|
| `vm` | `vm.c`, main ARM9 0x020111EC-0x020113D8 (16 fns) | `tools/rom-diff/specs/black-script/vm.spec` | `tools/check-fn.sh black-script vm` |
| `script_sys` | `script_sys.c` + `script_work.c`, overlay 10 0x02158804-0x021591FE, 0x02159960-0x02159B48, scrcmd ctx getters 0x0215A5CC-0x0215A5FA (90 fns) | `tools/rom-diff/specs/black-script/script_sys.spec` | `tools/check-fn.sh black-script script_sys` |
| `scrcmd` | F8 (command handlers), empty | | |

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

## Symbols

| C | Rust |
|---|------|
| `FUN_02011330` | `vm::read_u16` |
| `FUN_0201137C` | `vm::push` |
| `FUN_overlay_d_10__02158fd0` | `script_sys::sw_var_addr` |
| `FUN_overlay_d_10__02158fe0` | `script_sys::var_addr` |
| `FUN_overlay_d_10__0215901c` | `script_sys::var_value` |
| `FUN_overlay_d_10__02159198` | `script_sys::proc_mask_test` |
| `FUN_overlay_d_10__02159ae8` | `script_sys::cmd_var_addr` |
| `FUN_overlay_d_10__0215a5f4` | `script_sys::ctx_script_work` |

## Notes for porters

* A `vm` function ported in `vm.rs` does not deactivate its stubs in `script_sys.spec` (stub deactivation is per area):
  `script_sys` ports keep calling vm.c through their hooks (`ScriptSysHooks::fun_02011330` etc.).
* `MIi_CpuFill32` (0x02082A7C) and `MIi_CpuCopy16` (0x02082A60) run as real code on the ROM side (svflow `ALLOW`): inline them.
* `hooks::Mem` duplicates `black_field::hooks::Mem` (this crate does not depend on `black-field`).

Deliberate deviations: none yet.
