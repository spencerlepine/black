# svflow specs

One `<area>.spec` per `server_flow` module (`cmd_queue`, `hem`, ...). The Unicorn driver
(`tools/rom-diff/svflow.py`) and the Rust testkit (`crates/black-battle/src/server_flow/testkit.rs`,
which `include_str!`s the file) read the same text, so a function's inputs, stubs and hash are defined
in exactly one place. **Owned by harness jobs and the coordinator; porting workers must not edit
specs** (they ask for additions in their report, see below). A draft with `NEEDS-HAND` in it is
rejected by both loaders.

## Grammar

`#` starts a comment. Numbers are decimal or `0x` hex (addresses are hex without prefix). One `fn`
block per function, directives indented under it:

```
area cmd_queue
fn 021ccaa8 ret=none [mode=arm] [inputs=200]
  arg u8 any                       # scalar argument, in call order (r0, r1, r2, r3, then the stack)
  arg buf 0xC00                    # pointer to N random bytes (N multiple of 4, <= 0x10000), hashed after
  arg bpp                          # pointer to a BattleMon image (bpp.random_bpp), hashed after
  arg work [OFS]                   # pointer into the SvFlowWork image (0x212C bytes) at OFS
  fix arg0 0 u32 near 3000 12      # after random fill: write a generated value at offset 0 of arg0
  global 021f3b44 0x40             # fixed ROM address the function writes: random fill, hashed after
  env bpp                          # install the BattleMon-API stubs of bpp.py (+ their log is hashed)
  stub 02034714 noreturn ret=none args=v,v,v
```

* `ret=` none | bool | u8 | u16 | u32 | i32 | u64 | `mod:N` | `mask:0xM`. The value is masked like
  this before hashing, so garbage in unused return bits does not matter. `ofs:argK` (B10a-spec):
  the function returns a pointer into pointer argument K (`arg buf`); the ROM side hashes
  `ret - address(argK)` (0 stays 0) and the port returns the offset (0 = null).
* Scalar types: u8 u16 u32 s8 s16 s32 (the register holds the zero/sign-extended value).
* Generators (scalar `arg` and `fix`): `any` (edge-biased: 0, 1, 0xFF, <0x100, <0x10000, full),
  `range LO HI` (inclusive), `near N D` (N-D..N+D, for boundaries), `oneof a b c`, `const V`,
  `mask M`, `addrof (work|argK|globalK) [OFS]` (the fixed address of a region; no random draw),
  `alt GEN || GEN || ...` (B10a-spec: one draw picks an alternative uniformly, then that
  alternative draws; e.g. `alt near 2990 24 || range 0 2900` = near the cap or anywhere below).
  `rel K GEN` / `xrel K GEN` / `relm K MASK GEN` (Z17, scalar `arg` and `fix`; a value derived from
  the earlier scalar argument K, as passed to the function: K plus a `GEN` draw (wrapping), K xor
  a `GEN` draw, `(K & MASK)` plus a `GEN` draw; e.g. a second float operand near, negated, or at
  an exponent offset from the first). `GEN` can be any generator but a bare `alt` inside another
  `alt` is ambiguous (the `||` splits at the outer one). 64-bit values: pass the two words as two
  `u32` args (r0:r1 / r2:r3) and use `ret=u64` (r0 = low word, r1 = high word; `i2d` in
  `specs/black-core/softfloat.spec`); no wider scalar type is needed so far.
* Arguments past r3 are stack words (`vals[4..]` in the port), also for varargs functions whose
  `push {r0-r3}` prologue makes the register and stack arguments one list (`021cd4b0`, `021cd568`,
  `021cd64c`): declare one `arg` per word. The words after the last declared one are 0 in the
  emulator, so the port must be given zeros there too (see `cmd_queue/tests.rs`, `021cd64c`).
  **Entry state (Z19):** every input starts from the same CPU state: `Emu.call` zeroes r0-r12 and
  the flags (N Z C V Q, GE), zero-fills the stack window `0x02EF0000..0x02F01000` (64 KiB below
  sp = `0x02F00000`, 4 KiB from sp up), then writes the declared arguments (r0-r3, stack words
  from sp up). An undeclared register or stack word (a caller's saved r3 pushed in a prologue and
  read back through a short stack buffer, as `021b6848` does; an uninitialised local) therefore
  reads 0 on every input, never the previous input's value; ports' test wiring reads it as
  `c.word(i)` (`black_fndiff::Case::word` / the server_flow testkit `Case::word`, 0 past
  `vals`). `svflow.py AREA --order-check [--all]` proves an area order-independent (reverse order
  after junk in every register / the flags / the stack window); `svflow.py --selftest` tests the
  reset itself.
  A pointer into a `buf` argument can be a plain scalar `range`/`oneof` of the fixed address
  (`0x02E10000 + i * 0x10000 + ofs`); the port then subtracts the base.
* `fix TARGET OFS TYPE GEN [bits=LO:W] [rep=N,STEP]`: TARGET is `work`, `argK` (K = index among all
  arguments) or `globalK` (K-th `global` of the function). Fixes run in order after every region is
  filled; use them to keep indices/counts in range so the original does not fault or loop.
  `bits=LO:W` (B10a-spec) writes only bits LO..LO+W-1 of the TYPE-wide little-endian field at OFS
  (read-modify-write; the rest stays random). `rep=N,STEP` expands to N fixes at OFS, OFS+STEP, ...,
  each with its own draw (plants terminators/records along a buffer). Both come after the generator.
* `stub ADDR [noreturn] ret=KIND args=K,...`: a callee that is **not** ported. Argument kinds:
  `v` scalar (logged, mixed into the result), `p` pointer (ignored), `in:N` pointer to N bytes the
  callee reads (logged as words), `out:N` pointer to N bytes the callee fills with mix()-derived
  bytes. `noreturn` = fatal routine: the run ends there (`halt` section); on the Rust side the call
  unwinds to the testkit (B10a-spec), so a port that reaches it from a nested callee stops exactly
  where the original does (ports still `return` after the call, which is dead in the test). Stub result: `acc = salt; for v in v-args: acc = mix(acc, v)`;
  return `ret(mix(acc, addr & 0xFFFF))`; out byte j = `mix(acc, (addr & 0xFFFF) + 0x1000 + j) & 0xFF`;
  log entry `[addr, v-args..., in-words...]`. `salt` is drawn per input.
* A stub is **active** only while its address is not in any server_flow module's `PORTED`; once the
  callee is ported, both sides run the real code (ROM / Rust port).

## Generic mode (Z16): any crate, any set

The same files drive logic crates other than `black-battle::server_flow` (`tools/check-fn.sh`,
`crates/black-fndiff`; specs live in `tools/rom-diff/specs/<crate>/<area>.spec`). Extra grammar:

```
area poke_crypt
set field                          # memory image: ARM9 main + the overlays of that tools/re/sets.txt set
fn 02019a9c ret=u16
  nowork                           # no SvFlowWork image (no fill, no draws, not hashed)
  region data 0x100                # any struct: SIZE random bytes (multiple of 4, <= 0x10000), hashed after
  arg region data [OFS]            # pointer to the region (+ OFS); `fix region:data OFS TYPE GEN` and
  arg u32 range 0 0x100            #   `addrof region:data` work like argK/globalK
```

* `set NAME` (file level): `battle` (default; main + overlay 93, as before), `field`, or any set in
  `tools/re/sets.txt`; overlays are loaded at their link addresses from the build's `main.elf`.
* `region NAME SIZE`: at most 14 per function, at `0x02F10000 + k * 0x10000` (k = declaration
  order). Filled after the `global`s, before the `fix` lines. Compared as section `region:NAME`
  after the `argN` buffers. `arg work` is just the legacy region of `SvFlowWork`'s size.
* `sum16 OFS SIZE` (P1; a generator, usable in `fix`, `alt` and nowhere else; draws nothing): the
  u16 halfword sum of the fix target's bytes `[OFS, OFS + SIZE)` as they are at that point
  (`fix region:mon 6 u16 sum16 8 0x80` writes the checksum of a Pokemon record's blocks).
* `crypt TARGET OFS SIZE u16@SEEDOFS|u32@SEEDOFS [skipif=u16@OFS&MASK]` (P1; a directive in the
  `fix` list, runs in order, draws nothing): XOR the halfwords of `[OFS, OFS + SIZE)` of the target
  with the poke_tool LCG32 key stream (`state = seed; per halfword: state = state * 0x41C64E6D + 0x6073;
  key = state >> 16`), the seed being the little-endian field at SEEDOFS of the same target.
  Skipped if the `skipif` field `& MASK` is nonzero. With a preceding `sum16` fix this turns random
  bytes into a validly encrypted record (see specs/black-pokemon/poke_tool.spec). Mirrored in
  `svflow.py` (`crypt_fix`) and `black-fndiff` (`run.rs::crypt_fix`).
* `ramp TARGET OFS COUNT TYPE GEN` (P-spec; a directive in the `fix` list, runs in order): COUNT
  consecutive `TYPE` values from OFS, each the previous one (0 before the first, wrapping at the
  type width) plus a `GEN` draw, so a table comes out non-decreasing (`ramp global0 0 101 u32 range 0 20000`
  = an EXP curve). One draw per element. Mirrored in `svflow.py` (`Fix.ramp`) and `black-fndiff`
  (`run.rs`, `spec.rs`).
* `nowork` drops the `work` image for functions that never see an `SvFlowWork`.
* Args are `u8..s32` scalars, `buf N`, `region NAME [OFS]`, `work [OFS]`, `bpp` (black-battle only;
  other crates may add kinds through `black_fndiff::Ext`).
* A function's callees that are ported run as real code only if their address is in the `PORTED` of
  the same area (or of a file in `FNDIFF_PORTED_EXTRA`, or of a `server_flow` module). Stubs of a
  callee ported in another crate are not deactivated automatically.

## B10-specs additions (server_side / events)

All of these are new syntax: no existing spec or hash changes (cmd_queue stays 74d1ed062df4dd20).
They are mirrored in `svflow.py` and `black-fndiff` (`spec.rs`, `stub.rs`; parity vectors in the
`stub.rs` tests, computed with `svflow.py`).

* `stub ADDR ... seq`: the 0-based index of the call (per run, per stub address) is mixed into the
  result before the arguments (`acc = mix(salt, n)`), so a polled callee (`BattleRecordBuilder_ReadNext`,
  a loop condition) answers differently on each call. Without `seq` a stub called twice with the same
  arguments gives the same answer, which turns "loop while the callee says more" into an endless loop.
* Stub argument kind `outm:N:M0/M1/...`: like `out:N` (pointer to N bytes the callee fills) but byte j is
  `(mix(mix(acc, key + 0x1000 + 0x100 * (argpos + 1) + j), 0x5EED) >> 8) & 0xFF & M[j % len(M)]`:
  masked to keep an index, count or size in range (`outm:1:3` = 0..3, `outm:4:0xC/0/0/0` = size word
  0, 4, 8 or 12). `argpos` (position in the stub's argument list) makes two outs of one call differ,
  and the double mix keeps outs of different calls independent (plain `out:` bytes of two calls
  differ by a fixed XOR, so "equal in one means equal in the other"). `out:N` is unchanged.
* `in:N` may use any N (the last word is zero-padded), for pointers to a 2-byte stack pair.
* Stub result kinds (`ret=`): `pick:V1/V2/...` (one of the listed values, chosen by
  `mix(result, 0xC0DE) % count`; a decorrelated replacement for `bool`/`mod:N` when several stubs of one
  function must disagree or hit rare values such as 0xFD/0xFE/0xFF), `ptr:argK:OFS:N` (an address
  inside pointer argument K: `0x02E10000 + K * 0x10000 + OFS + 4 * (mix(result, 0xC0DE) % N)`; `N = 1`
  is a fixed address) and `ptr:work:OFS:N` (the same inside the work image), and `nullable:Z:KIND`
  (0 when `mix(result, 0x1234) % Z == 0`, else `KIND`). A port gets the address from `env.call`
  and turns it into an offset with the fixed base (`addr - 0x02E1_0000 - K * 0x10000`, or
  `addr - vals[K]`). Use them for callees that return a pointer the function then reads or hands to
  real code (a BattleMon, a record, a list).
* Empty `args=` (a callee without arguments) is the same as no `args=` in both parsers.

## B10-spec2 additions

New syntax only (no existing spec text changes meaning). Mirrored in `svflow.py` and `black-fndiff`.

* `stub ADDR force ...`: the stub stays active although `ADDR` is in some `PORTED` list. Only the ROM side needs
  the flag (the Rust port reaches the callee through `env.call`, so it hits the stub anyway; `black-fndiff`
  ignores the token). Use it at a boundary where the Rust port cannot call the real port with the same shape:
  `021cc668` from the server-side state functions (its Rust port takes a `SvFlowWork`, S[0x44] only addresses
  it), the 23 item-effect handlers from `use_item` (`021cb730`; as real code they dereference the null BattleMon
  of slot 6). The forced callee is verified by its own spec; the caller's spec verifies the call site.
* Stub argument kind `outn:N:M0/M1/...`: `outm:N:M..` when the pointer is non-null, a plain `v` argument (value 0,
  logged and mixed into the result) when it is null. For one callee called with both shapes (`021eeb64(handle,
  &size)` from `021bb6a8`/`021bb628`, `021eeb64(handle, 0)` from `021bb0a0`).

How these specs model a function that is not a `SvFlowWork` method (server_side): `nowork`, the object
is `arg buf 0x1000`, and every pointer field the function dereferences is set with
`fix arg0 OFS u32 addrof arg0 0xF00` (a data zone in the same buffer) so no second buffer is needed.
Callees that are thunks (`ldr r3,[pc]; bx r3`) are stubbed at their target address (the thunk runs
as real code and lands on the stub); the spec comments name the thunk. A stub's return kind and
argument count follow the **registers** in the disassembly, not Ghidra's prototype (a "void" callee
whose r0 is passed on or returned is `ret=u32`; a "4-argument" callee that overwrites r2 and r3
takes 2).

## B10f-spec additions (core1)

New syntax only; no existing spec text or hash changes. Mirrored in `svflow.py` and `black-fndiff`.

* `ret=ofs:work`: the function returns a pointer into the work image; the ROM side hashes `ret - 0x02E00000`
  (0 stays 0) and the port returns the work offset (0 = null). Like `ofs:argK` for `arg buf` pointers.
* The server-flow testkit now forwards the generic `region NAME SIZE` blocks to the port: `Case::regions[k]` holds
  region k (declaration order), its address in `Case::vals` is `0x02F10000 + k * 0x10000`. Use a region for an
  object the work only points at (the server-side object S at `work[0]`, the command queue at `work[0xC]`): declare it,
  then `fix work 0 u32 addrof region:S`. Real callees (server_side, cmd_queue) then run on it on both sides.
* Spec style for `core1` (see the header of `core1.spec`): a stub whose result only matters as a small number
  uses `ret=pick:0/1/...` instead of `bool` / `mod:N` (several `bool` stubs of one function are perfectly correlated
  when their addresses have equal low bits: `mix(salt, addr)` has the parity of `salt`). Stubs of one address that
  are called in a loop with unchanging arguments use `seq`. A pointer into the work passed to a stub is `p` (ignored)
  unless the callee's answer must depend on it, then `v` (the fixed address `0x02E00000 + ofs`).
* An old stub in another area's spec (`server_side.spec` stubs `021bb9d8`, `021bc360`, `021bc390`, `021bc55c`, ...)
  becomes inactive as soon as that address is in some `PORTED`; the porter asks the harness job for `force` on it.

## B10f-spec2 additions (harness)

* Callers of a newly ported function: the Rust ports call a ported callee through `env.call` (a stub) unless the port
  calls the Rust function directly. If a port keeps `env.call`, the caller's stub line needs `force` (the ROM side would
  otherwise run the real callee on random pointers: `021bc610`, `021bc220`, `021bbfa8`, `021bc418` from `021bbc44` /
  `021bbd2c` / `021bbd8c` / `021bcce0`, `021bcce0` from `021bd0c0`). Hashes of the callers do not change.
* `svflow.py` prints `svflow: warning: F calls C ...` on stderr when the ROM run of F enters a callee C that has no active
  stub, is not in any `PORTED` list, is not F and is not on the allowlist (pure library helpers: unsigned divide, MI_CpuFill8,
  MI_CpuCopy8, bulk copy, the thunk 021b8ddc; `SVFLOW_ALLOW=hex,hex` adds more). Such a callee runs as real code on the ROM
  side while the Rust port panics ("has no stub"), or the port inlines it. `--strict-callees` makes the run exit 4. Detection:
  the first instruction after a jump with `lr` = end of the previous instruction.
* B10-spec3 (warning check only, no hash effect): a Thumb thunk `ldr r3,[pc]; [movs rN,#k;] bx r3` whose target is stubbed / ported
  is not reported; the `env bpp` mock API counts as stubbed; a callee in any crate's `PORTED` is accepted; real-by-design callees
  are listed with reasons in `ALLOW` / `ALLOW_BPP` in `svflow.py` (soft float, 64-bit helpers, create/wordset helpers, BattleMon
  accessors under `env bpp`). `check-fn.sh` / `check-svflow.sh` run `cargo fmt --all` after recording.
* The stray-write scan survives a dirty address that is not mapped (the run records a `fault` section).

## P-spec2 additions (evolution / waza)

New syntax only; no existing spec text or hash changes. Mirrored in `svflow.py` and `black-fndiff`
(`spec.rs`); the parity is checked by the `evolution` area (a scalar argument that depends on another).

* Generator `ifeq K V THEN || ELSE` (scalar `arg` and `fix`): the earlier argument K (as passed) equal to V draws
  THEN, otherwise ELSE; only the chosen side draws. THEN must not contain `||`; ELSE may be an `alt` (it takes the rest
  of the line). Use it for an argument whose kind depends on another: `evolution.spec` passes the trade partner
  mon (`addrof region:partner`) as `param` when the mode is 1, and a location or item id otherwise.
* `addrof` now also works in a scalar `arg` generator on the ROM side (it already did in the Rust port), e.g.
  `arg u32 alt const 0 || addrof region:out` = a pointer that may be null.
* `tools/rom-diff/fncov.py AREA --spec S --fn ADDR [--case NAME=ADDR]... [--win ADDR]... [--reset ADDR]...
  [--label NAME=LO-HI]...`: instruction coverage of one spec function on the ROM side, with per-case entry and match
  counts for a jump table (see its docstring). Informational; it changes no hash.
* On macOS `svflow.py --jobs N` with `--spec` does not work (multiprocessing starts fresh interpreters that lose
  `--spec`); use `JOBS=1`.
## B10i-spec additions (core4)

No new syntax. Spec-writing conventions that `core4.spec` follows (and that make porting need no spec edits):

* Every block lists the stubs of its callees **and of the callees of all its in-area descendants**, and the resource lines
  (command-queue region Q, the P / B work pointers, scratch globals, input bounds a descendant needs) of the whole sub-tree.
  The "stub the callees of a newly ported function in its callers' specs" rule (RUST-REWRITE-TASK §6, B10-specs) therefore
  does not apply to core4 (`force` on callees ported in other areas is still a harness-job edit); to
  check a spec this way, treat every function of the area as ported (add its addresses to the `ported` set) and run the ROM
  side: no fault, no stray write and no unstubbed-callee warning means the spec holds for any porting order.
* An argument that any descendant dereferences is a buffer in every ancestor.
* Two `global`s must not overlap (the ROM has one memory, the Rust side one vector per global): when the put_cmd scratch
  (`021f3b44 0x40`) and another range overlap, one `global` covers both and the port slices it.
* `svflow.py` `ALLOW` (pure helpers that run as real code on both sides and that ports inline): added `0209c054` (64-bit
  multiply); the ARM9 random source `0203f0a8` is a stub.
* `fix`/`range` take non-negative numbers only in the Rust parser (`black-fndiff`): write a signed edge as `alt range 0 N || any`
  for an `s32` argument (`any` already produces negative values).

## B10j-spec additions (core5)

No new syntax. `core5.spec` follows the `core4.spec` conventions (closure stubs: every block lists the stubs and resource lines of its in-area
descendants, so porting in any order needs no spec edit; callees of the whole battle server outside the area are `force` stubs). Harness change:

* `svflow.py` `ALLOW`: the thunk `021cd63c` (`ldr r3,[pc]; bx r3` to the ported cmd_queue `push_u8` `021ccaa8`; core5 `5798` calls the thunk) runs as
  real code on both sides. No other area's area-level result changes.
* A scalar `arg` can be a pointer to a region or null: `arg u32 alt const 0 || addrof region:OUT` (5620's out pointer; the region is declared before the
  arguments so its index is the first one).
* Spot checks used for the area: all core5 functions treated as ported (the whole in-area tree real on the ROM side) runs 66 / 66 functions without a
  fault, a stray write or an unstubbed-callee warning; `svflow.py core5 --all --strict-callees` is clean.

## B12-spec additions (adapter)

New syntax only; no existing spec text or hash changes. Mirrored in `svflow.py` and `black-fndiff` (`stub.rs`, with a parity
vector in its tests).

* Stub argument kind `outp:K:OFS:N`: a pointer to a 4-byte out word that the stub fills with an ADDRESS inside pointer argument K:
  `0x02E10000 + K * 0x10000 + OFS + 4 * (mix(mix(acc, key + 0x1000 + 0x100 * (argpos + 1)), 0xC0DE) % N)` (`key` = stub address & 0xFFFF,
  `argpos` the position in the stub's argument list, as for `outm`). Use it for a callee that returns a pointer through an out
  parameter which the function then dereferences (`021d4c6c` in `adapter.spec`; `outm` can only AND bytes, so it cannot make a valid
  address). The port gets an `Arg::Out(&mut [u8; 4])`, reads the word back and turns the address into an offset with the fixed base.
* `svflow.py` `ALLOW` gained the thunks `021ba1a8` (`ldr r0,[r0]; bx 02013158`) and `021d4c58` (`bx 021d4cec`): they run as real code
  and land on the stub of their target.
* Spec-writing conventions of `adapter.spec` (see its header): the object a function works on is the arena `arg buf 0x1000`; message
  strings are an extra `arg buf 0x100` whose index is the last declared argument; two globals that never overlap
  (`021f6384 0x14` for the adapter table, `021f6398 0x60` for the message context).

## B10-spec4 additions

New syntax only (no existing spec text changes meaning). Mirrored in `svflow.py` and `black-fndiff` (`stub.rs`).

* Stub argument kind `v16`: a scalar of which only the low halfword is logged and mixed in. For a callee that ignores the high half
  while the caller leaves stale stack data there (`021d7c70` called from `021c236c`, which stores a u16 into the word below `sp` and
  loads the whole word; the callee reads `(u16)p1`, `p1 & 0xf`, `(p1 & 0x3ff) >> 4`, so the high half cannot change its behaviour).
* Stub result kind `lt:K`: `mix(...) % argK`, 0 when `argK` is 0 (argument K must be a scalar): a `rand_range(n)` answer strictly
  below `n` (`021d78ac` in the `021c39f4` / `021c3810` / `021c376c` blocks, where the answer indexes an 8-byte stack array of which only
  the first `count` entries are written).

## B13-spec1 additions (client1..client5, btl_client.c)

No new syntax; no existing spec text or hash changes. Harness change (small, backward compatible): areas named `clientN` live in
`crates/black-battle/src/client/clientN.rs` (+ `clientN/tests.rs`, test path `client::clientN::tests::svflow_match_rom`) instead of
`server_flow/`: `check-svflow.sh` picks the directory from the name, `svflow.py` scans both directories for `PORTED` lists (a client
function that is ported deactivates the stubs of its address in every spec, like a `server_flow` one).

* All client areas are `nowork`; the client object (0x1B8 bytes) is the first `arg buf 0x1C0`. Same closure-stub convention as core4/core5:
  a block lists the stubs of its callees and of the callees of its in-area descendants (checked by running every function with the whole
  area treated as ported: no fault, no stray write, no unstubbed callee).
* A stub argument list is the registers the caller sets before the `bl` (mon first for the `021d5xxx` view calls); ignored results are `ret=none`.
* An `s32` argument that must hit a signed byte: `arg s32 alt range 0 5 || oneof 0xFFFFFFFF 0x80` (`021cfbbc`).
* A stack struct passed to a stub whose last bytes the caller never writes: `in:7` (zero-padded last word, `021d3cc8`).
* Only stub addresses of other areas' smoke ports need `force` (here: adapter `021eebdc` in `021cd984`).

## B10bc-spec additions (hem / api / handex)

No new syntax; `svflow.py` and `black-fndiff` are unchanged. Conventions of `hem.spec`, `api.spec` and `handex.spec` (headers):

* Closure stubs as in core4: with every function of the area treated as ported, all blocks run without a fault, a stray write or an
  unstubbed callee, so the areas can be ported in any order without spec edits. Callees in other server-flow areas (including between
  hem, api and handex) are `force` stubs; cmd_queue callees run as real code (queue region Q block); thunks `021b8ddc` / `021b98a4` run
  and land on the stubs of their targets.
* A HANDEX proc is specced as `(work, buf 0x80 record[, v])`; the executor `021c8918` sees each proc as `ret=pick:0/1 args=p,v[,v]`
  and carries the closure of all 59 procs (its block holds the record bounds the procs need). Its case table is in the comment on its
  block.
* Caller stubs of the smoke ports got `force` where they were plain: `021c8838` / `021c8860` (core4), `021c88a4` (events),
  `021c88e4` (core2, core3, core4, events). Hash-neutral (the callers' ports reach them through `env.call`).

## B11-spec additions (server1..server3, battle_server.s)

No new syntax; no existing spec text or hash changes. Harness change (small, backward compatible, like B13-spec1): areas named
`serverN` live in `crates/black-battle/src/server/serverN.rs` (+ `serverN/tests.rs`, test path `server::serverN::tests::svflow_match_rom`):
`check-svflow.sh` picks the directory from the name, `svflow.py` scans it for `PORTED` lists too.

* All server areas are `nowork`. The main module M (0x474 bytes) is `arg buf 0x1000` with its BATTLE_SETUP (`*M`) at arena offset 0x480
  (`fix arg0 0 u32 addrof arg0 0x480`); further pointed-to objects sit in higher zones of the same buffer (each spec header lists them).
  The `btl_server.c` object S is `arg buf 0x1000` with zones at 0xD00+ (as in `server_side.spec`).
* Function-pointer fields that a function calls (`blx rN`) are fixed to FAKE callee addresses in the unused mapped page
  0x02EE0000-0x02EEFFFF (Thumb: odd values, e.g. `fix arg0 0x450 u32 alt const 0 || const 0x02EE0011`) and stubbed there
  (`stub 02ee0010 ...`); the emulator hooks the stub address, a port calls `env.call(ptr & !1, ..)`.
* Same closure-stub convention as core4/core5/client: a block lists the stubs of its callees and of the callees of its in-area descendants;
  checked by running every function with the whole area treated as ported (no fault, no stray write, no unstubbed callee). Callees in another
  `serverN` area or another svflow area are `force` stubs.
* A pointer that a stub returns into a block the function never receives as an argument (021b9a6c reads the record 021b99e0 returns) is
  modelled with an extra `arg buf` the function ignores, as the `ptr:argK` target; the port treats that argument as test-only data.

## F7-spec additions (black-script vm / script_sys)

No new syntax; no existing spec text or hash changes. Harness change (warning check only): `svflow.py` `ALLOW` gained `02082a60`
(MIi_CpuCopy16) and `02082a7c` (MIi_CpuFill32), pure ARM helpers like `MI_CpuFill8` / `MI_CpuCopy8`: real code on the ROM side,
inlined by the ports. Conventions of `specs/black-script/{vm,script_sys}.spec` (headers): closure stubs as in core4 (checked with
every area function treated as ported: no fault, stray write or unstubbed callee); objects are `arg buf` arenas with the pointed-to
objects in zones; a function that only allocates gets a trailing `arg buf` arena it never reads as the `ptr:argK` target (stack word
or unused register); one `02030734` stub serves every allocation of a block (`ptr:argK:OFS:N`, blocks may overlap; deterministic);
fake callbacks at 0x02EE0011 / 0x02EE0021 / 0x02EE0031; a vm.c function ported in `vm` keeps its stubs in `script_sys` active.

## B13-spec2 additions (client1..client5 complete)

No new syntax; no existing spec text or hash changes. Harness change: `svflow.py` `ALLOW` += `02082a90` (MI_CpuCopy32, a pure word copy;
client4 `021d1a74` copies the adapter data into the command queue with it).

* Every block of the B13-spec2 sections follows the closure-stub convention inside its area (checked with the whole area treated as
  ported: no fault, budget exhaustion, stray write or unstubbed callee). A callee in **another client area** is a `force` stub (ported or
  not), so porting order across areas needs no spec edit; the port calls it through `env.call`. A client smoke port called from another area
  without a stub line runs as real code (`021cdee8` from client4 `021d1c40`, its stubs are listed).
* Indirect calls (`blx rN` through a function-pointer field: `c+0x28` step, `c+0xC8` request handler, `c+0xD0` pending callback, `c+0x17C`
  playback handler) go to `force` stubs at the target addresses; the field is fixed with `oneof` to those addresses. A port dispatches the
  pointer through `env.call(ptr)`.
* Out-pointer data that the function dereferences (adapter `021eec20` data, record pointers of `021ee6b8`) lives in an extra `arg buf`
  that the function never reads as a register (its r1 / r2 / r3 is overwritten first); the stub answers `outp:K:0:1` / `ptr:argK:0:N` into
  it. Such an extra argument is part of the port's test wiring only.
* Loops whose bound comes from the input are bounded by `fix` lines where the original would loop forever or overrun a stack array (u8
  counters against a u32 bound: `021d09e0`, `021d215c`, `021d39a8`); the comment says why.
* Function list: `battle_client.s` labels + the `sub_` seeds of `mkelf.py` + every odd pointer / `bl` target into the range, minus the
  `LAB_` pieces of `.byte` functions, plus a static reachability scan of the range for code no known start reaches (found `021d3d84`,
  called from overlay 95). The ten Thumb-to-ARM veneers (`bx pc` + `ldr ip; bx ip` to 0x0689xxxx) are not specced: callers stub them at
  the Thumb entry.

## What is compared (per input, in this order)

`halt` (0, or 1 + stub address) , `ret`, `work` (all 0x212C bytes), `argN` for each buf/bpp,
`globalK`, `stray` (any byte written outside work/args/globals/stack that differs from the
original image; the Rust side has none, so an undeclared write is a mismatch: add a `global`),
`stublog` (count + words), `env` (only with `env bpp`). Hash = 64-bit FNV-1a over the 32-bit words
(`h = (h ^ w) * 0x100000001b3`), started at the FNV offset basis. Per input, per function (fold of
its input hashes), per area (fold of function addresses and input hashes). The ROM run faulting
(bad memory access, instruction budget) records a `fault` section the Rust side cannot produce:
tighten the inputs with `fix`.

Fixed addresses (identical in the emulator and in `Case.vals`): work 0x02E00000; pointer
argument i 0x02E10000 + i * 0x10000; stack 0x02EF0000-0x02F01000.

## Asking for a spec (porting workers)

Put this in the report, do not edit the spec: the function address, the area, which arguments are
pointers to what (offsets read/written, valid index ranges), callees you needed that have no stub
(address, prototype guess, whether they write through a pointer or never return), and the
command you ran. The harness job generates a draft with

    python3 -I tools/rom-diff/genspec.py <addr>... --area <area>

and resolves every `NEEDS-HAND` against the disassembly.

## P4 additions (waza / icon)

New syntax only; no existing spec text or hash changes (black-pokemon poke_tool / create / personal / growth / evolution hashes
re-verified; waza's 30 earlier functions keep their hashes and the area hash of those 30 is still 4f85bade6b03f326). Mirrored in
`svflow.py` and `black-fndiff` (`stub.rs`, parity vector `seqptr_hands_out_consecutive_blocks`).

* Stub result kind `seqptr:argK:OFS:STEP`: the n-th call (0-based, per run, per stub address, with or without `seq`) returns
  `0x02E10000 + K * 0x10000 + OFS + n * STEP`, independent of the arguments: a bump allocator whose blocks never overlap (an
  allocator stub for a constructor that makes several allocations and links them, `FUN_0201C328` in `waza.spec`; the arena is an
  `arg buf` the function itself ignores). `ptr:argK` cannot do this (its addresses are random, so blocks alias).
* A `global` outside the mapped RAM (0x02000000-0x02FFFFFF), e.g. the I/O register `REG_DISPCNT` 0x04000000, gets its 4 KiB pages
  mapped (zero-filled) by the ROM side; it is filled, fixed and hashed like any global (`icon.spec`). The Rust side needs nothing.
* `svflow.py` `ALLOW`: `02018438` (level from EXP, black-pokemon `create::level_of`), which `02017fb0` (waza) runs as real code by
  design; its callee `0201850c` stays a stub. This removes the waza unstubbed-callee warning; hash-neutral.

## F2a-spec additions (black-field map_matrix / fieldmap / fieldmap_event / fieldmap_func / land_data_patch)

No new syntax; no existing spec text or hash changes. Harness changes (small, additive):

* `emu.py`: for non-battle sets (`Emu([overlays])`) the I/O page 0x04000000-0x04001FFF is mapped (zero-filled), so field code
  that toggles a register (EXMEMCNT 0x04000204 around card reads) runs; a spec owns and hashes the register with
  `global 04000204 4` (an undeclared write is a stray). Battle runs are unchanged; inputs that used to fault on such a write
  were never recordable.
* `svflow.py` `ALLOW`: the ARM9 library helpers `02082a48` MIi_CpuClear16, `02082a60` MIi_CpuCopy16, `02082a7c` MIi_CpuClear32,
  `02082a90` MIi_CpuCopy32 and `0203efc8` memcmp run as real code (the ports inline them; they write / read only their arguments).

Conventions of these specs (headers): the arena is the `work` image (records at fixed offsets, pointer results of stubs are
`ptr:work:OFS:1`); every out-of-area stub carries `force` (overlay addresses collide with battle-overlay `PORTED` entries, e.g.
`021bc550`, and ARM9 callees are ported in other black-field areas); a stub argument list is positional, so a register the
callee does not read before one it does is `p`; closure stubs as in core4 (checked with every function of the area treated as
ported: no fault, stray write or unstubbed callee; `svflow.py AREA --all --strict-callees` clean).

## F8-spec1 additions (black-script scrcmd_core / scrcmd_work)

No new syntax; `svflow.py` and `black-fndiff` are unchanged; no existing spec text or hash changes. Conventions of
`specs/black-script/{scrcmd_core,scrcmd_work}.spec` (headers):

* A command handler `fn(vm, ctx)` gets two arenas: the VM (`arg buf 0x800`: regs at 0x40, script bytes at 0x500, the compare
  byte +0xE in 0..2) and the scrcmd ctx (`arg buf 0x800`: CTX at 0, SCRCMD WORK at 0x100, zones at 0x300 / 0x400, vars at
  0x600..0x7FF where the var-address stub `02159ae8` points, `ptr:arg1:0x600:0x80`). Helpers that take the ctx in r0 get the ctx
  arena as arg0.
* Script-operand readers (vm.c `02011330` / `0201134c` / `02011398`, script_sys `02159ae8` / `02159b10`) are `seq` stubs, so the
  operands of one command differ; operands that index a ROM table or pick a switch case are bounded with `pick:`.
* Closure stubs as in core4 (every function of the area treated as ported: no fault, stray write or unstubbed callee). The two areas
  call each other only through plain stubs (each is verified on its own; check-fn deactivates only the area's own `PORTED`).
* Function list: command-table targets + `bl` targets from the whole field set + odd code pointers + labels, walked as Thumb from
  every entry (only padding and literal pools stay uncovered). Mid-function labels that Ghidra / mkelf turned into functions are
  listed as fragments in the headers, not specced.

## B14-spec additions (ai1, tr_ai.c / overlay 96)

New grammar (mirrored in `svflow.py` and `black-fndiff`, with a parity vector in `stub.rs`): stub result kind
`ret=u64`. The callee returns a 64-bit value in `r0:r1`: `r0 = mix(acc, key)` (as a `u32` stub) and
`r1 = mix(r0, 0x6464)`. The ROM side writes both registers (`emu.py` accepts a tuple stub result); the port
calls [`black_fndiff::StubEnv::call64`] and gets `r1 << 32 | r0`. Used for `020870D0` (OS tick) in `ai1.spec`.

Also new (no hash change to existing specs): the file-level `set battle+96` loads the historical battle image
(ARM9 main + overlay 93) plus the listed overlays at their link addresses, so a spec can run code in a
co-resident overlay such as 96 without a `tools/re/sets.txt` entry (`set_overlays` in `svflow.py`).

`ai1` is `crates/black-battle/src/ai/ai1.rs` (`check-svflow.sh` / `svflow.py` pick the `ai/` directory from the
`ai[0-9]` name, like `clientN` / `serverN`). All of overlay 96 is one `nowork` area (the three dispatchers are
hubs, so any split only adds crossing calls). Closure stubs as in core4/B11 (every block lists the stubs of its
callees and of all its in-area descendants); callees of ov96 all live in ARM9 main or other battle areas and
are `force` stubs (several are already ported in black-battle, so the stub must stay active to keep the ROM
side deterministic). `svflow.py ai1 --all --strict-callees` is clean.

## F8-spec2 additions (black-script scrcmd_fld / scrcmd_talk / scrcmd_obj / scrcmd_screen)

No new syntax; `svflow.py` and `black-fndiff` are unchanged; no existing spec text or hash changes. Conventions of
`specs/black-script/scrcmd_{fld,talk,obj,screen}.spec` (headers), on top of the F8-spec1 ones:

* Every stub outside the block's area carries `force` (overlay-21 addresses share the range of battle `PORTED` entries; the four
  areas and the overlay-10 ones call each other only through forced stubs, so each area is verified on its own).
* A GMEVENT function `(event, seq, work)` is `arg u32, arg buf 0x10 (seq word, bounded with alt range), arg buf 0x40 (work)`; an
  init proc of the table 0x021705F0 `(task, task + 0xC)` is `arg buf 0x100, arg buf 0x10`.
* Code paths that read an uninitialised callee-saved register (the emulator leaves r4..r7 from the previous input, which a port
  cannot reproduce) are kept out of the inputs (0x021AE948 value 0 / 1 only, 0x021B1668 never 0xF1, 0x021B1E80's stub always
  answers nonzero); the header says so and the coverage report counts the lost instructions.
* Overlay-21 BSS words the commands write (0x021EE71C, 0x021EE720) are `global`s; the ports address them absolutely.

## F4a-spec additions (black-field fldmmdl / event_battle_head)

New syntax only; no existing spec text or hash changes. Mirrored in `svflow.py` (`make_stub`) and `black-fndiff` (`stub.rs`, parity
vector `outp_work_writes_an_address_inside_the_work_image`, computed with `svflow.py`).

* Stub argument kind `outp:work:OFS:N`: like `outp:K:OFS:N`, but the address is inside the work image
  (`0x02E00000 + OFS + 4 * (mix(mix(acc, key + 0x1000 + 0x100 * (argpos + 1)), 0xC0DE) % N)`). For an iterator whose out
  parameter is a pointer to an object inside the work (`0216DD1C(sys, &mmdl, &index)`, stubbed
  `seq ret=pick:0/1 args=v,outp:work:0x100:1,p` in `fldmmdl.spec`).

Conventions of `specs/black-field/{fldmmdl,event_battle_head}.spec` (headers): closure stubs as in core4 (checked with every
function of the area treated as ported: no fault, stray write or unstubbed callee); in-area getters whose answer callers compare
(counts, OBJ id / code, move code, flag tests) are stubbed with `pick:` over the interesting values; ROM-table lookups that hand out
function tables (`0216E400` / `0216E40C`) are `force` stubs returning FAKE tables in the work image; out-of-area callees are
`force` stubs.

## F6a-spec additions (black-field wfbc_data / eventdata / mmdl_move / eventdata_tail)

No new syntax; `black-fndiff` is unchanged; no existing spec text or hash changes. Harness change (warning check only, hash-neutral):
`svflow.py` `ALLOW` += `02082c60` (memset(dst, v, n), a pure byte fill like `MI_CpuFill8`; `mmdl_move` `021631c0` clears its ATTR INFO
with it, the ports inline it).

Conventions of `specs/black-field/{wfbc_data,eventdata,mmdl_move,eventdata_tail}.spec` (headers): closure stubs as in core4 (checked with
every function of the area treated as ported: no fault, stray write or unstubbed callee; `--strict-callees` and `--order-check` clean);
every out-of-area stub is `force`, also between the four areas; records whose fields a lookup compares with a position are drawn small
and correlated with the position (several `alt` alternatives of one small range weight it); indirect calls through ROM tables land on
in-area stubs or force stubs at the table targets, with the table index bounded to the table.
