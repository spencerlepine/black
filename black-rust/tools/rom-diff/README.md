# rom-diff

Runs original Pokemon Black machine code (ARM9 + overlay 93) under the
[Unicorn](https://www.unicorn-engine.org/) emulator and prints FNV-1a hashes that the
`*_matches_rom_fuzz` tests in `crates/black-battle` must reproduce.

Inputs are read at runtime from the decomp build (`black-rom/build/black.us/main.sbin` and
`OVY_93.sbin`, which match the retail ROM). Override the directory with `BLACK_ROM_BUILD`. Nothing
from the ROM is copied into this repo.

```sh
python3 -m venv /tmp/blk/venv && /tmp/blk/venv/bin/pip install unicorn
/tmp/blk/venv/bin/python -I tools/rom-diff/gen.py            # all suites
/tmp/blk/venv/bin/python -I tools/rom-diff/gen.py event_var  # one suite
```

- `emu.py` maps main RAM, loads ARM9 at `0x02004000` and overlay 93 at `0x021B6100`, and calls a
  Thumb function with r0-r3. `Emu.stub(addr, fn)` replaces an undecompiled callee with a Python
  function. The return honours interworking: Unicorn picks Thumb from bit 0 of the value
  written to PC.
- `gen.py` has one function per Rust fuzz test. Each one copies its Rust test's generator
  (xorshift32 seeding, call order, hashed values) exactly, and its stubs behave like the Rust
  test mocks.
- `evf.py` (event_factor suite) runs small Thumb handler stubs. They ship pre-assembled
  (`ASM_BYTES`), so no ARM toolchain is needed. `Emu.load_thumb` re-assembles `ASM` with
  `arm-none-eabi-as`/`objcopy` only when they are on PATH, and fails if the result differs from
  `ASM_BYTES`. After editing `ASM`, regenerate the bytes (binutils 2.41 `--target=arm-none-eabi`;
  `make all-gas all-binutils` builds it from source in ~3 min).

To verify a newly ported function, add the call to both sides with `"TODO"` as the expected hash
in Rust. Run `gen.py` and check that its hash equals the one the Rust test prints, then paste it in.
Never copy the Rust output on its own.

## Per-function results (Z9)

`handlers.py <module>` prints the module hash exactly as before. Extra modes:

```sh
handlers.py abilities --json            # {"hash": module hash, "funcs": {addr: {"hash", "n_inputs", "seeds"}}}
handlers.py abilities --json --jobs 8   # same, handlers spread over 8 processes; "hash" is null
handlers.py abilities --detail 021d85f4:1   # one input's word stream: [["hostlog", 258], ...]
```

`seeds` holds one hash per input (200 per handler). The module hash folds the same words in the
same order, so equal per-function maps imply equal module hashes.

The Rust side (`handlers/testkit.rs::run_handler_suite`, called by `<module>_match_rom`) writes the
same map when `HANDLER_JSON=<path>` is set, and one input's word stream when
`HANDLER_DETAIL=<hex addr>:<seed>` and `HANDLER_DETAIL_OUT=<path>` are set (only that handler runs).

```sh
tools/check-handlers.sh abilities --per-function   # JOBS=8 by default
```

Compares the two maps (`diffmaps.py maps`). On a mismatch it prints every differing function
(first 20) and its first failing input: the section (`ev`, `mon0`..`mon7`, `null_mon`, `work`,
`skip_check`, `flags`, `rng`, `hostlog`, `envlog`), word index, and +-3 words of context from both
sides (`hostlog` words are stub-call records: `0x1xx` call id, then its arguments). Recording is
unchanged: the hash is written into the test only when the whole module matches. Artifacts go to
`$CARGO_TARGET_DIR/handler-hashes-<module>*.json`, `detail-*.json`.

## Stub skeletons (Z9)

```sh
python3 -I tools/rom-diff/genstub.py <addr> [--set battle] [--tag-base 0x126] [--packet FILE]
```

Reads the work packet (`/tmp/blk/packets/<set>/<addr>.md`, built with `tools/re/packet.py` if
missing) and prints, for each callee that is neither ported nor already stubbed, a Python
`st(addr, lambda ...)` line for `Host.__init__` in `handlers.py` and the matching Rust `FlowHost`
declaration and `TestHost` impl. Callees that are ported, runtime helpers, or `BattleMon`-style
traits outside `flow.rs` run as real code under the emulator and are listed as `# skip`.
Conventions: log `[TAG, scalar args...]`, return `mix(fold(args), addr & 0xFFFF)` (masked to the
return type), arguments past the fourth read from `sp`. Pointer arguments, pointer or 64-bit
returns and guessed prototypes are marked `NEEDS-HAND`. Output is generated: porting workers use it,
the coordinator pastes it into the harness; nobody edits the generated lines.

## Server-flow harness (B10h)

Per-function differential test for the `server_flow` areas, driven by one spec file per area
(`svflow_specs/<area>.spec`, grammar and rules in `svflow_specs/README.md`):

```sh
/tmp/blk/venv/bin/python -I tools/rom-diff/svflow.py cmd_queue --json --jobs 8   # ROM side, PORTED functions
python3 -I tools/rom-diff/genspec.py 021cd4b0 --area cmd_queue                   # draft spec from the packet
tools/check-svflow.sh cmd_queue --per-function      # ROM vs Rust; records hashes only on a full match
tools/check-svflow.sh cmd_queue --fn 021cc95c       # one function, never records
```

`svflow.py` allocates an `SvFlowWork` image (0x212C random bytes) plus the other regions the spec
asks for at fixed addresses, runs the original function under Unicorn with every callee in the spec
that is not yet ported stubbed (logging stubs, `mix()`-derived results; ported callees run as real
code), and hashes the return value, the whole work image, every buffer, declared globals, any stray
write, and the stub log. `crates/black-battle/src/server_flow/testkit.rs` does the same on the Rust
port (`Case`, `TestEnv: FlowEnv`) and `<area>/tests.rs::svflow_match_rom` compares against the
`RECORDED` table written by `check-svflow.sh`. `--detail ADDR:SEED` / `SVFLOW_DETAIL` work as in
handlers.py and `diffmaps.py` reads the same JSON.

## Generic function harness (Z16)

`svflow.py` is the one driver; `server_flow` is just its default area layout. For any other crate
(P poke_tool, S storage, F field, Z15 RNG; main ARM9 or the `field` overlays):

```sh
# 1. spec: tools/rom-diff/specs/<crate>/<area>.spec  (harness job; porters request it, see svflow_specs/README.md)
# 2. port: crates/<crate>/src/<area>.rs with `pub const PORTED: &[u32] = &[0x0201_9A7C, ...];`
# 3. test: crates/<crate>/src/<area>/tests.rs (copy crates/black-core/src/poke_crypt/tests.rs;
#    `black-fndiff` is a dev-dependency) wiring each address to the port in `call`
tools/check-fn.sh <crate> <area> --per-function   # ROM vs Rust; records hashes only on a full match
tools/check-fn.sh <crate> <area> --fn 02019a9c    # one function, never records
```

`check-fn.sh` = `check-svflow.sh` with `--spec`, `--ported-src`, the spec's `set`, and the
`FNDIFF_*` env vars (see its header). The Rust half is `crates/black-fndiff` (spec parser, input
generator, stub model, hash, `run_suite`); `black-battle`'s server_flow testkit is a thin wrapper.
Python and Rust must stay in lock step: a grammar change goes into `svflow.py` and
`black-fndiff` together and is covered by `svflow_specs/selftest.spec`.
P1 added two spec elements for encrypted records (`sum16` generator, `crypt` fix directive; see
`svflow_specs/README.md`), used by `specs/black-pokemon/poke_tool.spec` to build records with a valid
checksum and a really encrypted block.

## Held-item handlers (B5h)

`handlers.py items` / `tools/check-handlers.sh items --per-function` work exactly like `abilities`
(same flags: `--json`, `--jobs`, `--detail`), over `handlers/items.rs::PORTED`. Differences:

- The factor is a type-5 factor owned by the handler's Pokemon (`+0x10` = 5, `+0x38` = item ID,
  `+0x3A`/`+0x3B` = owner). The item ID is, 3 times out of 4, one of the registry items that use
  the handler under test, else any of the 171 registry keys. The registry (0x021F1EA4) and its
  getters are read from the ROM bytes (`item_registry` in `handlers.py`, `ItemRegistry::load` in
  `testkit.rs`; every getter is `movs/str/ldr/bx`, asserted). `handlers.py items --list` prints
  the 229 handlers with their items, events and ported flag.
- The 7 work words are 0 (50%), 0-3 (25%) or 32-bit noise (25%): handlers test "already used"
  flags in `work[0]`, which noise would almost never leave at 0. The extra draws come after the
  ability draws, so the `abilities` stream and hash are untouched.
- Items need ROM data at run time (registry), so `items_match_rom` is skipped without
  `BLACK_ROM_BUILD`; the per-function flow is unchanged.
- New stubs (tags 0x126-0x12E): flow queries `021c7ec4`, `021c7f24`, `021c818c`, `021c830c`,
  `021c8374`, `021c8408`, item data parameter `021d805c(item, param)` (byte), condition table
  `0689c9e4(index)` (0 from index 6), Pokemon query `020189d0(pokemon, x)` (-1 or 0-4).
  Everything else item code reaches (BattleMon accessors, event variables, HANDEX string
  params, soft-float helpers `0209bdbc`/`0209be04`/`0209c494`/`0209c95c`, the pure helper
  `021d7e10`, and the item code's own helpers 0x021DCFD0-0x021E02C0) runs as real code, so a
  port must reproduce it (port the helper, do not stub it).
- To check one handler while porting: add it to `items.rs`, then
  `tools/check-handlers.sh items --per-function` (it reports the failing function, input and
  diverging words); the other ports stay in the module hash.

## Move handlers (B6h)

`handlers.py moves` / `tools/check-handlers.sh moves --per-function` work exactly like `items`
(flags `--json`, `--jobs`, `--detail`, `--list`), over `handlers/moves.rs::PORTED`.

- The factor is a type-0 factor owned by the handler's Pokemon (`+0x10` = 0, `+0x38` = move ID,
  `+0x3A`/`+0x3B` = owner). The move ID is, 3 times out of 4, one of the registry moves that use
  the handler under test, else any of the 258 registry keys. The registry (0x021F3030:
  `(u16 move ID, pad, getter)`, stride 8) and its getters are read from the ROM bytes
  (`move_registry` in `handlers.py`, `MoveRegistry::load` in `testkit.rs`). The getters of
  moves 87, 100, 217 and 542 have the same `movs/str/ldr/bx` shape as the others (their `ldr`
  literal is just not a pool-aligned address: tables 0x021F2CC0, 0x021F2CD8 and 0x021F2D08 in the
  gap between pools B and C), so one decoder reads all 258. `handlers.py moves --list` prints the
  337 distinct handlers (0x021E0468-0x021E84FC) with their moves, events and ported flag.
- Work words and draw order are the same as items (the extra draws come after the ability
  draws, so the `abilities` and `items` streams and hashes are untouched). Move-only differences
  (`moves=True` in `handlers.py`, `Some(MoveRegistry)` in `testkit.rs`):
  - the move stubs below are installed only in move mode (the ability and item runs keep running
    the real code at those addresses);
  - event variable 0x3F is the fixed value `STRP` (0x02EB0000): in the original it is a pointer
    to the message (string parameter) buffer that `021c8838`/`021c8860` fill (6 handlers use
    it). `FlowHost::msg_param()` is that buffer (0x40 bytes, zeroed per input); a new last hash
    section `strparam` covers it;
  - the global 0x021F6364 (scratch pointer used by the random move pick `021d7e44`) points to
    `SCRATCH`, which is never hashed: a port uses a local buffer.
- New stubs, tags 0x12F-0x154, all log `[tag, args...]` and return `mix()` values:
  flow queries `021c7d34`, `021c7d44`, `021c7ed8`, `021c8030`, `021c8114`, `021c8138`,
  `021c814c`, `021c8174` (ground kind 0-16), `021c8180`, `021c81c8` (takes a `bpp` pointer: logged
  and passed as the Pokemon ID), `021c824c`, `021c82c4`, `021c8300`, `021c831c`, `021c83a0`
  (weight-like), `021c83dc`, `021c83e8`, `021c846c` (out value), `021c8494`, `021c2e4c`, `021c7f4c`
  (test-and-set of a flow-work flag; `021c2ab0` is the same `weather` stub as the thunk
  `021c8288`); the party structs `021c7fcc`/`021c7fe0` (`party_ids`/`partner_party_ids`: a
  struct of mon pointers with the count at +0x18, built in `PARTY`, read by the real
  `021b9b94`/`021b9c00`; the second one is sometimes null; like `expand`/`opponents` the member
  IDs go to an out buffer on the Rust side); overlay-95 queries `06898cf4`, `0689c8ac`,
  `0689c8bc`, `0689c91c`, `0689c93c`, `0689ca1c`, `0689ca3c`, `0689ca5c`, `0689ca7c`, `0689ca8c`
  (a table of 0x29 move IDs plus count, `fun_0689ca8c() -> Vec<u16>`), and the history/party
  record queries `0689d064`, `0689d098`, `0689d0c8`, `0689d174`, `0689d184` (their first
  argument is a pointer into the flow work, `wk + 0x1c` or `wk + 0x3e0`, dropped on the Rust
  side; `021c812c`/`021c8130` are the pure address computations and run as real code).
  Everything else (BattleMon accessors, event variables, HANDEX string parameters, the move
  code's own helpers 0x021E02DC-0x021E8D50, the pure helpers listed in the B6h report) runs as
  real code, so a port must reproduce it.
- `handlers.py movestubs` is a self-test of the stubs above: it calls each through the emulator
  with random arguments and hashes result, log and out buffers; `moves/tests.rs`
  `move_stubs_match_python` computes the same hash from the `TestHost` impls (no ROM needed).
- Coverage: with 200 random inputs per handler about 76% of the handler instructions run.
  Handlers gated on one specific event variable value (for example `021e1354`: var 0x1D == 8)
  are reached rarely; to check such a handler thoroughly a port job must request harness hints
  (a harness job adds them), not narrow the inputs itself.

## Soft-float runtime (Z17)

`specs/black-core/softfloat.spec` covers the ARM9 compiler-runtime float helpers (`0209BDBC`,
`0209BE04`, `0209C494`, `0209C95C`, `0209CB8C`, `0209B750`, `0209B790`; ARM mode, `mode=arm`):
`tools/check-fn.sh black-core softfloat --per-function`. Operands are drawn from special-value
lists (zeros, infinities, NaNs, denormals, exponent boundaries, rounding ties) mixed with random
bit patterns; the second operand of `fadd`/`fsub` is derived from the first with the `rel`/`xrel`/
`relm` generators (svflow_specs/README.md). The `item_percent_fixed` helper `021dd008` is checked
through the item handlers that call it (`tools/check-handlers.sh items --per-function`).

## Overlay-95 effect handlers (B7h)

`handlers.py effects` / `tools/check-handlers.sh effects --per-function` work like `moves`, for the
30 handlers of the three overlay-95 registries (field effects 0x0689D650, position effects
0x0689D6C8, side effects 0x0689D780; decoded in the module doc of `handlers/effects.rs`).
- Overlay 95 is loaded at 0x06898020 next to overlay 93 (`effects=True` in `handlers.py`; other
  modes do not load it, so their results are unchanged). Its `bx pc` veneers reach overlay 93,
  where the existing move-mode stubs apply (`Host(..., effects=True)` installs `move_stubs`), so
  only one stub is new: `06898de0` (tag 0x155, the side-effect layer count read from the
  0x0689E7E0 BSS array; meaning guessed). `handlers.py effectstubs` is its self-test, mirrored by
  `effect_stubs_match_python` in `handlers/effects/tests.rs`.
- Registry decode: `fx_registry` in `handlers.py` (`EffectRegistry::load` in `testkit.rs`): three
  tables of `(key, getter[, max layers])`; every getter is `movs/str/ldr/bx` (asserted).
  `effects --list` prints `address [ported] kind keys=.. events=.. [max=..]` per handler.
- Factor: type 3 / 1 / 2 by kind, sub ID = a key using the handler (3 of 4 draws, else any key
  of the kind), owner (`+0x3A`, the handler's `poke_id` argument) = 0 / `me % 6` / `me & 1`,
  `+0x3B` = 0x1F. Same work-word distribution as moves. Hints use the shared `HINTS` table
  (keys are 0x0689xxxx, so no clash with overlay-93 handlers); one stub salt swap
  (`FX_ALT_BD28`: `0201bd28` answers 4/5 for handler `068993b0`; the usual `% 3` can never).
- `06898e54` (shared tail-called helper of side keys 0 and 1) is not a handler; `coverage.py
  effects --all` measures it from its callers' executed instructions. Coverage: 30 handlers +
  the helper, 100% of the reachable instructions with the hints (without hints 6 were below 70%).

## Instruction coverage (Z18)

`coverage.py abilities|items|moves|effects [--all] [--jobs N] [--threshold 70] [--only ADDR,..] [--json]
[--json-out FILE]` reports, per handler, the % of its instructions the harness's 200 inputs
execute (Unicorn code hook via `handlers.run(cov=...)`, which does not change any hash). The
denominator is the Thumb instructions statically reachable from the entry (branches and fall-through
only; calls are not followed, literal pools never reached); `missed` lists the unexecuted ranges,
which is where a hint is needed. `--all` covers every registry handler (not only PORTED), so
unported ones are included. `tools/check-handlers.sh MODULE --coverage` prints the table for PORTED
after the hash check (informational). The 337-move run takes about 1.5 min on 8 cores.
- Hints: `MOVE_HINTS` in `handlers.py` (mirrored by `move_hint` in `handlers/testkit.rs`) force
  event variables and mon bytes on 3 of 4 seeds for the handlers listed there, without extra RNG
  draws; change both sides together. Item handler `021dfe28` keeps its older inline hint.
