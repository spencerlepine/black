# svflow spec: core5 (0x021C4838-0x021C65A8; B10j, written by B10j-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 / B10f-spec / B10f-spec2 / B10i-spec additions). No new grammar.
#
# What this area is: the fifth part of the flow core (btl_server_flow.c): the flow's own small message / command emitters for the
# turn-end passes and the items of the damage tail, plus a few mid-size state functions.
#   4838 / 493c / 4d1c / 4f64 / 5108 / 5200 / 5308 / 5350   the "per-mon pass" family: list walks (veneer iterators), record brackets on the
#                                                           packed queue-header word, flag bytes at work+0x7A9.., the end-of-turn flag pass 5200
#   4a64 / 4b3c / 4bac / 4c24 / 4c50 / 4e38 / 4ea4 / 5480.. message round trips (021d74f8 start, 75ec / 75a8 / 7630 add arguments, 70f0 send,
#                                                           775c read-back, 7540 end) and message / command emitters (cd4b0 / cd64c)
#   5098 / 56a8 / 5798 / 5840 / 59c4 / 5a8c / 5b74 / 5c1c / 5c74 / 5dac.. / 6158 / 6190 / 63d0   value helpers and the multi-word message writers
#   62d8 / 6350 / 6394 / 6458 / 64b4 / 6564                 message round trips whose first argument is the server object (not the work)
# 4838 and 4d1c are the two top-level walks; 5200 is called by both and by 493c. 4a64, 4bac and 4c50 are jump-table functions.
#
# SvFlowWork fields seen here ("Q" = region of the command queue behind work+0x0C):
#   +0x04 main module, +0x08 pokecon (party container), +0x0C command queue buffer (region Q), +0x3E0 per-slot list (only its address
#   is handed to veneers), +0x440 u32 (cleared by 5474), +0x4C0 (address handed to 69b8), +0x782 u8 count of the 16-byte entries at
#   +0x7E0 (6190: word +0 = mon, +4 = packed event record, +0xD / +0xE flag bytes), +0x788 u8 result of 6190, +0x789 / +0x78A flag bytes,
#   +0x7A9..0x7AE one flag byte per mon id (4f64), +0x868 list object (4c50), +0x1AB8 / +0x1AE4 (hem record) helper structs
#   (addresses only), +0x1B08 six 12-byte records (5350, `arg work 0x1b08`), +0x1D74 packed queue-header word (bracketed by cc8d8 / cc95c).
#
# Layout of the blocks (generated from templates, then hand-reviewed): every block carries (a) the stubs of its own callees AND of the
# callees of all its in-area descendants, and (b) the resource lines of the function and its descendants (region Q for the command
# queue, scratch globals, the index and count bounds a descendant needs). Porting any core5 function therefore
# needs no spec edit, whatever order the area is ported in (checked: with every core5 function treated as ported, i.e. the whole
# tree real on the ROM side, 66 / 66 functions run without a fault, a stray write or an unstubbed callee).
#
# Rules (README): callees ported in cmd_queue run as REAL code on both sides: 021cc8d8 021cc95c 021cc918 021cd4b0 021cd4fc 021cd568
# 021cd64c 021ccaa8 (queue; 021ccaa8 is reached through the thunk 021cd63c, which svflow.py allows) and the events word-table getter
# 021cc8b8. Every function that reaches cd4b0 / cd4fc / cd568 / cd64c / ccaa8 declares the region Q block (queue at work+0xC, write count
# below the 3000 cap, scratch words 021f3b44, the fatal 02034714). Callees in core1 / core2 / core3 / core4 / events / server_side /
# api / hem / handex / core6 (everything of the battle server between 021bb968 and 021cc8d8 outside core5) are `force` stubs, so these
# specs stay valid as those areas land and the Rust ports keep calling them through env.call (callers' hashes do not change when
# they get ported; each is verified by its own area). Everything else outside this area, and every core5 function not yet in PORTED,
# is a stub (a stub of a core5 function becomes inactive the moment that address is in a PORTED list). `bx pc` veneers (list below)
# are stubbed at their own address. Stub argument kinds follow the REGISTERS set at each call site and the callee's own reads
# (hidden pass-through of r0..r3 is common: 5480 gets work and the mon from 54c8's own r0 / r1; 8810 takes the hem record only).
# Pointers into the work or into a region that are only handed on are `p`, scalars `v`. Stack words past r3 are declared in call order.
# Stubs whose result is only a small number use `ret=pick:...`; callbacks run in loops (veneer iterators 48f0 / 4900, 4a44, 4cfc / 4d0c,
# 4e18 / 4e28, 69b8) are `seq` + `nullable:3:u32`, so every loop ends.
#
# Argument-shape notes: 5c1c is varargs-style (`push {r0-r3}` prologue, return through `bx r3` after `pop {r3}`): the register arguments
# are the first four words of its argument list. 5798 takes (work, u16 pointer, count, int array, mon-pointer array, u16 array) with the
# last two on the stack; 5840 / 59c4 take (work, count, int array / mon-pointer array, ..., flag on the stack). 5a8c has a u16 stack
# argument first (`ldrh [sp,0x38]`) and a flag after it. 6458 takes the hem record (work+0x1ae4) as its stack argument; 62d8 writes a
# u16 + u8 through its stack argument (`out:3` for the stubs). 4b9c / 4c40 / 54bc are tail-jump wrappers (`bx r3` into 4bac / 4c50 / 5d88
# with swapped registers): their blocks call the tail target through its stub.
#
# Jump tables: 4a64 (11 cases, halfword offsets at 021c4aa8), 4bac (14 cases, 021c4bc4), 4c50 (8 cases, 021c4c6a), 5c74 (21 cases,
# 021c5ca0), 5dac / 5e48 / 5efc / 5f98 (5 cases each, 021c5dc2 / 5e62 / 5f12 / 5fb2) switch with `add pc, r0`; Ghidra splits none of them,
# the case bodies and the literal pools are part of their functions (the specs reach the cases through the selector argument).
#
# Uninitialised stack: none found that changes a result (the emulator zeroes the stack anyway; 493c's 8-byte id array is read by 806c
# beyond its 4 written entries, deterministic zero on both sides).
#
# `--strict-callees` clean. The only ALLOW addition in svflow.py is the thunk 021cd63c (5798's call of the ported push_u8 021ccaa8).
# Dead code: 021c56d0 (`b 021c5704` after the three-case compare of 56a8; r1 is already one of 0x24..0x26 there), 1 insn.
#
# Skipped (not specced): 24 two-byte `bx pc` veneers into unloaded 0x0689xxxx code (list iterators, flow-API getters; each is followed
# by an 8-byte ARM thunk `ldr ip,[pc]; bx ip` that Ghidra names thunk_EXT_FUN_0689xxxx; the thunk halves 021c4a58 021c4b30 021c4e0c
# 021c4e20 021c4e2c 021c507c 021c508c 021c50fc 021c52cc 021c52dc 021c52ec 021c52fc are part of their veneers), and the 8-byte wrapper
# 021c4b8c, which loads a callback pointer (021c4b9d) and jumps straight into unloaded code (0x06898d54: `ldr r3,[pc,4]; bx r3`), so it
# cannot run under the emulator:
#   021c48e0 021c48f0 021c4900 021c4a24 021c4a34 021c4a44 021c4a54 021c4b2c 021c4cec 021c4cfc 021c4d0c 021c4e08 021c4e18 021c4e28
#   021c5078 021c5088 021c50f8 021c52c8 021c52d8 021c52e8 021c52f8 021c54ac 021c5698 021c5bd4
# (their callers stub them at these addresses). 66 real functions are specced. Smoke ports (B10j-spec, in core5.rs): see the PORTED list there.
area core5

# # the list walk of the first kind: bracket 0x25ac on the packed queue header word around 4910(0x1f, r2), then per entry another bracket (0x25b6) around 4910(id, r2) + 4f64
# # (the entry's own processing), and 5200 (the end-of-list flag pass), whose result is returned (V-0023: tail bl + epilogue)
fn 021c4838 ret=u32 inputs=300
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c48e0 ret=none args=v
  stub 021c4910 ret=none args=p,v,v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021c48f0 ret=nullable:3:u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c4f64 ret=pick:0/1 args=p,v
  stub 021c4900 seq ret=nullable:3:u32 args=v
  stub 021c5200 ret=pick:0/1 args=p
  stub 021b85ec force ret=pick:0/1 args=v
  stub 021c52c8 ret=pick:0/1/2/3 args=p,v
  stub 021c52d8 ret=pick:0/1 args=p,v,v
  stub 021c52e8 ret=u32 args=p,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b86b8 force ret=pick:0/1/2 args=v
  stub 021bc310 force ret=pick:0/1 args=p
  stub 021b86dc force ret=pick:0/1/2/3 args=v
  stub 021c5308 ret=pick:0/1 args=p,v
  stub 021c52f8 ret=none args=p,v,v
  stub 021b9a40 force ret=pick:0/1/1/2 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9aec force ret=u32 args=v,v
  stub 021cb2d4 force ret=none args=p,v,v,ptr
  stub 021c5350 ret=pick:0/1 args=p,v,p
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d69b8 seq ret=nullable:3:u32 args=v,in:4,p
  stub 021ba1cc force ret=none args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c5078 ret=none args=p,v
  stub 021ba1a8 force ret=pick:0/1 args=v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021c5108 ret=none args=p,v,v
  stub 021d646c ret=none args=v
  stub 021c5098 ret=u32 args=p
  stub 021b92b0 force ret=none args=v,v,v
  stub 021c5088 ret=none args=p,v
  stub 021b902c force ret=pick:0/1 args=v,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021b8640 force ret=pick:0/1/2/3/4/5 args=v
  stub 021b8de4 force ret=u32 args=v,v
  stub 021c50f8 ret=pick:0/1/2/3/31 args=p,v
  stub 021c51d0 ret=none args=p,v
  stub 021bff60 force ret=none args=p,v,v
  stub 021d8510 ret=none args=v
  stub 021dcf24 ret=none args=v
  stub 021d8538 ret=none args=v
  stub 021dcfa4 ret=none args=v
  stub 021e0434 ret=none args=v
  stub 021bd2b4 force ret=none args=ptr,p
  stub 021bd338 force ret=none args=ptr,ptr
  stub 021bd34c force ret=pick:0/1 args=ptr,p,out:4 seq
  stub 021d6264 ret=none args=v,v
  stub 021ef90c ret=none args=v
  stub 021c3fdc force ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021d75ec ret=none args=v,v

# # message 0x25c3: arg 2 = id of the mon, send to r1 (the server object)
fn 021c4910 ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # per-list damage record: list of up to 6 mon ids in a 4-byte-aligned stack array, 4a54 (record writer) per (mon, entry) pair, then command 0x15 with the packed ids
# # and 5200, whose result is returned (V-0023: tail bl + epilogue)
fn 021c493c ret=u32 inputs=300
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c4a24 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021c4a34 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c4a44 seq ret=nullable:3:u32 args=v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d6064 ret=pick:0/1 args=v,v,out:4,out:4
  stub 021c4a54 ret=none args=v,v,v,v,v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021d806c ret=u32 args=in:8
  stub 021c5200 ret=pick:0/1 args=p
  stub 021b85ec force ret=pick:0/1 args=v
  stub 021c52c8 ret=pick:0/1/2/3 args=p,v
  stub 021c52d8 ret=pick:0/1 args=p,v,v
  stub 021c52e8 ret=u32 args=p,v,v
  stub 021b86b8 force ret=pick:0/1/2 args=v
  stub 021bc310 force ret=pick:0/1 args=p
  stub 021b86dc force ret=pick:0/1/2/3 args=v
  stub 021c5308 ret=pick:0/1 args=p,v
  stub 021c52f8 ret=none args=p,v,v
  stub 021b9a40 force ret=pick:0/1/1/2 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9aec force ret=u32 args=v,v
  stub 021cb2d4 force ret=none args=p,v,v,ptr
  stub 021c5350 ret=pick:0/1 args=p,v,p
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d69b8 seq ret=nullable:3:u32 args=v,in:4,p
  stub 021ba1cc force ret=none args=v,v,v

# # 0x261b bracket: 4b3c (message round trip) gates the effect on 20bc; the effect is selected by r2 (0..10): 5480 with message ids 0x257/0x273/0x258/0x25a..., else the hem path (8810, 4b2c, 20d0)
fn 021c4a64 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 range 0 13
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c4b3c ret=nullable:4:u32 args=p,v,v,v
  stub 021c20bc force ret=pick:0/1 args=p,v,v
  stub 021c5480 ret=none args=p,v,v
  stub 021c8810 force ret=none args=ptr
  stub 021c4b2c ret=none args=p,v,v
  stub 021c20d0 force ret=u32 args=p,v,v,p
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c54ac ret=pick:0/1/2/6 args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x2649 round trip: mon r1 (5440), params r2 / r3, reply slot 0x32
fn 021c4b3c ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # callback trampoline: swaps r0 / r2 and tail-jumps to 4bac
fn 021c4b9c ret=none
  arg u32 any
  arg u32 range 0 16
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c4bac ret=none args=p,v,v

# # message id table (r1 = 0..13 -> id) then 0x5a message with terminator
fn 021c4bac ret=none inputs=300
  arg work
  arg u32 range 0 16
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # callback 0x21c4c41 registration (ef934) + command 0x2f
fn 021c4c24 ret=none
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021ef934 ret=none args=v,v

# # tail-jump to 4c50 with r0 / r1 swapped
fn 021c4c40 ret=none
  arg u32 range 0 10
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c4c50 ret=none args=p,v
  stub 021c43f0 force ret=none args=p,v
  stub 021c4cec ret=none args=v
  stub 021c4cfc ret=nullable:3:u32 args=v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021c195c force ret=none args=p,v,v
  stub 021c4d0c seq ret=nullable:3:u32 args=v

# # message table (r1 = 0..7 -> id) then 0x5a message; command 0x24; case 7 walks the list at work+0x868 and runs 195c on every entry for which 5b7c says yes
fn 021c4c50 ret=none inputs=300
  arg work
  arg u32 range 0 10
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c43f0 force ret=none args=p,v
  stub 021c4cec ret=none args=v
  stub 021c4cfc ret=nullable:3:u32 args=v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021c195c force ret=none args=p,v,v
  stub 021c4d0c seq ret=nullable:3:u32 args=v

# # the second walk: 70e4 with 0x255 when something was done, per entry 4f64
fn 021c4d1c ret=u32 inputs=400
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021ef8b8 ret=pick:0/0/0/1 args=p
  stub 021d59b4 ret=pick:0/1/50/100/200 args=v,v
  stub 021c3be0 force ret=none args=p,v
  stub 021c2ab0 force ret=pick:0/1/2 args=p
  stub 021c4e08 ret=none args=v
  stub 021c4e18 ret=nullable:3:u32 args=v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5bdc ret=pick:0/1 args=v,v
  stub 021d7bd4 ret=u32 args=v,v
  stub 021c4e38 ret=u32 args=p,v,v,v
  stub 021c4ea4 ret=none args=p,v,v,v
  stub 021c4f64 ret=pick:0/1 args=p,v
  stub 021c4e28 seq ret=nullable:3:u32 args=v
  stub 021c70e4 force ret=none args=p,v,v,v,v,v
  stub 021c5200 ret=pick:0/1 args=p
  stub 021b85ec force ret=pick:0/1 args=v
  stub 021c52c8 ret=pick:0/1/2/3 args=p,v
  stub 021c52d8 ret=pick:0/1 args=p,v,v
  stub 021c52e8 ret=u32 args=p,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b86b8 force ret=pick:0/1/2 args=v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021bc310 force ret=pick:0/1 args=p
  stub 021b86dc force ret=pick:0/1/2/3 args=v
  stub 021c5308 ret=pick:0/1 args=p,v
  stub 021c52f8 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b9a40 force ret=pick:0/1/1/2 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9aec force ret=u32 args=v,v
  stub 021cb2d4 force ret=none args=p,v,v,ptr
  stub 021c5350 ret=pick:0/1 args=p,v,p
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d69b8 seq ret=nullable:3:u32 args=v,in:4,p
  stub 021ba1cc force ret=none args=v,v,v
  stub 021c5078 ret=none args=p,v
  stub 021ba1a8 force ret=pick:0/1 args=v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021c5108 ret=none args=p,v,v
  stub 021d646c ret=none args=v
  stub 021c5098 ret=u32 args=p
  stub 021b92b0 force ret=none args=v,v,v
  stub 021c5088 ret=none args=p,v
  stub 021b902c force ret=pick:0/1 args=v,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021b8640 force ret=pick:0/1/2/3/4/5 args=v
  stub 021b8de4 force ret=u32 args=v,v
  stub 021c50f8 ret=pick:0/1/2/3/31 args=p,v
  stub 021c51d0 ret=none args=p,v
  stub 021bff60 force ret=none args=p,v,v
  stub 021d8510 ret=none args=v
  stub 021dcf24 ret=none args=v
  stub 021d8538 ret=none args=v
  stub 021dcfa4 ret=none args=v
  stub 021e0434 ret=none args=v
  stub 021bd2b4 force ret=none args=ptr,p
  stub 021bd338 force ret=none args=ptr,ptr
  stub 021bd34c force ret=pick:0/1 args=ptr,p,out:4 seq
  stub 021d6264 ret=none args=v,v
  stub 021ef90c ret=none args=v
  stub 021c3fdc force ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c8838 force ret=none args=ptr,v,v
  stub 021c8860 force ret=none args=ptr,v
  stub 021c8810 force ret=none args=ptr
  stub 021c20bc force ret=pick:0/1 args=p,v,v
  stub 021c99a0 force ret=none args=p,ptr
  stub 021c8010 force ret=u32 args=p,v
  stub 021c20d0 force ret=u32 args=p,v,v,p
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v

# # message 0x26e8: ids 0x39, 0x41, reply slots 0x32 and 0x41
fn 021c4e38 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # hem record work + 0x1ae4 set up per r2 (3 / 4 / other); when r3 > 0 and 20bc says yes: 99a0, 70e4 with 0x290 and 0x255, 20d0
fn 021c4ea4 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 range 0 6
  arg s32 alt range 0 150 || any
  stub 021d59b4 ret=pick:0/1/50/100/200/300 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c8838 force ret=none args=ptr,v,v
  stub 021c8860 force ret=none args=ptr,v
  stub 021c8810 force ret=none args=ptr
  stub 021c20bc force ret=pick:0/1 args=p,v,v
  stub 021c99a0 force ret=none args=p,ptr
  stub 021c8010 force ret=u32 args=p,v
  stub 021c70e4 force ret=none args=p,v,v,v,v,v
  stub 021c20d0 force ret=u32 args=p,v,v,p

# # the per-mon flag byte at work + 0x7a9 + id
fn 021c4f64 ret=bool inputs=300
  arg work
  arg u32 any
  fix work 0x7a9 u8 alt const 0 || any rep=6,1
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021ba1a8 force ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c5078 ret=none args=p,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021c5108 ret=none args=p,v,v
  stub 021d646c ret=none args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021c5098 ret=u32 args=p
  stub 021b92b0 force ret=none args=v,v,v
  stub 021c5088 ret=none args=p,v
  stub 021b902c force ret=pick:0/1 args=v,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021b8640 force ret=pick:0/1/2/3/4/5 args=v
  stub 021b8de4 force ret=u32 args=v,v
  stub 021c50f8 ret=pick:0/1/2/3/31 args=p,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c51d0 ret=none args=p,v
  stub 021bff60 force ret=none args=p,v,v
  stub 021d8510 ret=none args=v
  stub 021dcf24 ret=none args=v
  stub 021d8538 ret=none args=v
  stub 021dcfa4 ret=none args=v
  stub 021e0434 ret=none args=v
  stub 021bd2b4 force ret=none args=ptr,p
  stub 021bd338 force ret=none args=ptr,ptr
  stub 021bd34c force ret=pick:0/1 args=ptr,p,out:4 seq
  stub 021d6264 ret=none args=v,v
  stub 021ef90c ret=none args=v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021c3fdc force ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # the max of BattleMon field 0xf over the entries of list work + 0x1ab8 (r4 = 0..8641 count inclusive)
fn 021c5098 ret=u32 inputs=300
  arg work
  stub 021d59b4 ret=pick:0/1/2/5/20/50 args=v,v
  stub 021b8640 force ret=pick:0/1/2/3/4/5 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b8de4 force ret=u32 args=v,v
  stub 021b902c force ret=pick:0/1 args=v,v
  stub 021c50f8 ret=pick:0/1/2/3/31 args=p,v
  stub 021b9934 force ret=u32 args=v,v

# # per-mon pass: 51d0, bff60, per-species follow-ups, bd2b4 / bd338 / bd34c loop with command r7, ef90c, 0x23, and when BattleMon field 0x11 == 0x7f: 3fdc
fn 021c5108 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=pick:0/1/127/65663/4096 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c51d0 ret=none args=p,v
  stub 021bff60 force ret=none args=p,v,v
  stub 021d8510 ret=none args=v
  stub 021dcf24 ret=none args=v
  stub 021d8538 ret=none args=v
  stub 021dcfa4 ret=none args=v
  stub 021e0434 ret=none args=v
  stub 021bd2b4 force ret=none args=ptr,p
  stub 021bd338 force ret=none args=ptr,ptr
  stub 021bd34c force ret=pick:0/1 args=ptr,p,out:4 seq
  stub 021d6264 ret=none args=v,v
  stub 021ef90c ret=none args=v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021c3fdc force ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x27df -> 70f0 (0xa3)
fn 021c51d0 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # the end-of-turn flag pass over work + 0x3e0 (52c8 count): per entry 52d8 / 52e8 / 9934 / 86b8 / bc2b0 / bc310 -> command 0x55 and work[0x78a] |= 2; 5308 sets the result
fn 021c5200 ret=bool inputs=300
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b85ec force ret=pick:0/1 args=v
  stub 021c52c8 ret=pick:0/1/2/3 args=p,v
  stub 021c52d8 ret=pick:0/1 args=p,v,v
  stub 021c52e8 ret=u32 args=p,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b86b8 force ret=pick:0/1/2 args=v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021bc310 force ret=pick:0/1 args=p
  stub 021b86dc force ret=pick:0/1/2/3 args=v
  stub 021c5308 ret=pick:0/1 args=p,v
  stub 021c52f8 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b9a40 force ret=pick:0/1/1/2 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9aec force ret=u32 args=v,v
  stub 021cb2d4 force ret=none args=p,v,v,ptr
  stub 021c5350 ret=pick:0/1 args=p,v,p
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d69b8 seq ret=nullable:3:u32 args=v,in:4,p
  stub 021ba1cc force ret=none args=v,v,v

# # per-species gate: 9a40(5440(id)) == 1 -> cb2d4 + 5350
fn 021c5308 ret=bool inputs=300
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b9a40 force ret=pick:0/1/1/2 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9aec force ret=u32 args=v,v
  stub 021cb2d4 force ret=none args=p,v,v,ptr
  stub 021c5350 ret=pick:0/1 args=p,v,p
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d69b8 seq ret=nullable:3:u32 args=v,in:4,p
  stub 021ba1cc force ret=none args=v,v,v

# # six 12-byte records at r2 (= work + 0x1b08): word at +0 (nonzero) -> 0x5a message with 0x2a / 0x2b, command 0x3e with the bytes 5..10, 69b8 loop, command 0x45 and ba1cc
fn 021c5350 ret=bool inputs=400
  arg work
  arg u32 any
  arg work 0x1b08
  fix work 0x1b08 u32 alt const 0 || any rep=6,12
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=pick:0/1/50/99/100/200 args=v,v
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d69b8 seq ret=nullable:3:u32 args=v,in:4,p
  stub 021ba1cc force ret=none args=v,v,v

# # 5bdc says yes: 6110, 62c4(0x1a) -> 5620, command 0x31, bd1d0
fn 021c5418 ret=bool
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5bdc ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c6110 ret=none args=p,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c5620 ret=none args=p,v,v,v
  stub 021bd1d0 force ret=none args=p,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d61f0 ret=none args=v
  stub 021c5698 ret=pick:0/1/2/6 args=p,v
  stub 021d6238 ret=none args=v,v
  stub 021d6008 ret=none args=v,v

# # work + 0x440 = 0
fn 021c5474 ret=none
  arg work

# # 54ac(work + 0x1ab8, id) != 6 -> command 0x4d
fn 021c5480 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c54ac ret=pick:0/1/2/6 args=p,v

# # tail-jump to 5d88(r0, r1, 0x350)
fn 021c54bc ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c5d88 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # 5480 with 0x25b, message 0x15c
fn 021c54c8 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c5480 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c54ac ret=pick:0/1/2/6 args=p,v

# # 62d8(7), 5480 with 0x25c, message with 0x5b / 0x1ed
fn 021c54fc ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d62d8 ret=u32 args=v,v
  stub 021c5480 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c54ac ret=pick:0/1/2/6 args=p,v

# # command 0x36, 5b74(-r2, 1), message 0x50
fn 021c5540 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5b74 ret=none args=p,v,v,v
  stub 021d5fa4 ret=none args=v,v
  stub 021c604c ret=none args=p,v,v
  stub 021c5bd4 ret=pick:0/1/2/6 args=p,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5f90 ret=none args=v,v

# # commands 0x10 / 0x35, message selected by r2 (2 / 3 / other) when r3 != 0
fn 021c557c ret=none
  arg work
  arg u32 any
  arg u32 alt range 0 4 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d61f0 ret=none args=v

# # command 0x59
fn 021c5604 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # (region 0 = OUT, the u32 that r3 points at, or r3 = 0; region 1 = Q) r2 == 0: nothing; r3 != 0: *r3 = 6314(mon, r2); r2 < 6: commands 0x10 / 0x35 (unless 5698 says 6) else 6238 and command 0x11
fn 021c5620 ret=none inputs=300
  region OUT 4
  arg work
  arg u32 any
  arg u32 alt const 0 || range 1 12 || any
  arg u32 alt const 0 || addrof region:OUT
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d6314 ret=u32 args=v,v
  stub 021d61f0 ret=none args=v
  stub 021c5698 ret=pick:0/1/2/6 args=p,v
  stub 021d6238 ret=none args=v,v

# # event-variable decode: *r1 selects (0, 0x24, 0x25, 0x26, other) the result via 62a8 / 62c4 / 7e10
fn 021c56a8 ret=u32 inputs=300
  arg u32 any
  arg buf 0x10
  fix arg1 0 u32 alt oneof 0 0x24 0x25 0x26 || range 0 0x30 || any
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d7e10 ret=pick:0/1 args=v

# # message 0x47
fn 021c5718 ret=none
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # message 0xd5 for mon r1
fn 021c572c ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # message 0xd2 for mon r1
fn 021c5750 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # message 0xd8 for mon r1
fn 021c5774 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # per entry (count r2, words r3 compared with 3, mon pointers stack0, u16 values stack1): 604c, then command 0x33 with the verdict (1 none above / 2 / 3), then ccaa8 per entry
fn 021c5798 ret=none inputs=300
  arg work
  arg buf 0x10
  arg u32 range 0 7
  arg buf 0x20
  arg buf 0x20
  arg buf 0x20
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c604c ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5f90 ret=none args=v,v

# # words r2[0..r1) compared with 3: > 3 / < 3 counts, then (flag stack0) per-class 0x5b / 0x5a messages with up to 3 mon ids from r3
fn 021c5840 ret=none inputs=400
  arg work
  arg u32 range 0 7
  arg buf 0x20
  arg buf 0x20
  arg u32 alt const 0 || any
  fix arg2 0 u32 alt range 0 6 || any rep=8,4
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # per entry with flag byte set (r3[i]): message 0x5b / 0x180 or 0x5a / 0x51
fn 021c59c4 ret=none inputs=300
  arg work
  arg u32 range 0 7
  arg buf 0x20
  arg buf 0x20
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # command 0x57
fn 021c5a1c ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # command 0x58
fn 021c5a34 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # message 0x99 (r3 > 0) or 0xae
fn 021c5a4c ret=none
  arg work
  arg u32 any
  arg u32 any
  arg s32 alt range 0 3 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # r3 > 0: 5dc0, commands 9 / 0x37; stack1 != 0: message 0x1b / 0x3aa; r3 <= 0: 5e38, commands 0xa / 0x38, message 0x5a
fn 021c5a8c ret=none inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg s32 alt near 0 3 || any
  arg u16 alt const 0 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5dc0 ret=u32 args=v,v,v
  stub 021d5e38 ret=u32 args=v,v,v

# # r2 > 0: 5fa4 + command 2; r2 < 0: 604c(-r2); r3 != 0 and 5bd4 != 6: command 0x41
fn 021c5b74 ret=none inputs=300
  arg work
  arg u32 any
  arg s32 alt const 0 || near 0 4 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5fa4 ret=none args=v,v
  stub 021c604c ret=none args=p,v,v
  stub 021c5bd4 ret=pick:0/1/2/6 args=p,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5f90 ret=none args=v,v

# # BattleMon_Get(mon, 0xd), 5fb8, commands 3 / 0x43
fn 021c5be4 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5fb8 ret=none args=v

# # varargs: (work, mon, n, extra): 6034, command 0xf, n < 6: command 0x35 (n == 5 and 8a78(extra) -> 6)
fn 021c5c1c ret=none inputs=300
  arg work
  arg u32 any
  arg u32 alt range 0 8 || any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d6034 ret=none args=v,v,v
  stub 021e8a78 ret=pick:0/1 args=v

# # sub-id table (r3 = 0..0x14) for mode r2 == 1, mode 2: message 0xd2, other: 0x47; all set work[0x78a] |= 0x10
fn 021c5c74 ret=none inputs=400
  arg work
  arg u32 any
  arg u32 oneof 0 1 1 1 1 2 2 3
  arg u32 range 0 0x17
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # message with terminator
fn 021c5d88 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # message 0x5a with 0..4 words from r3
fn 021c5dac ret=none inputs=300
  arg work
  arg u32 any
  arg u32 range 0 7
  arg buf 0x20
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # message 0x5c with 0..4 words from stack0
fn 021c5e48 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 range 0 7
  arg buf 0x20
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # message 0x5b with 0..4 words from r3
fn 021c5efc ret=none inputs=300
  arg work
  arg u32 any
  arg u32 range 0 7
  arg buf 0x20
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # message 0x5d with 0..4 words from stack0
fn 021c5f98 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 range 0 7
  arg buf 0x20
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # 911c(work[4], 4) == 0: 5f90(mon, id) + command 1
fn 021c604c ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5f90 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # r4 flag stack0 != 0: 56a0 + command 8 else 5674 + command 7
fn 021c6080 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d56a0 ret=none args=v,v,v
  stub 021d5674 ret=none args=v,v,v

# # command 0x42
fn 021c60d4 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # 5fe4 + command 0x19
fn 021c60ec ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5fe4 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # 6008 + command 0x1a
fn 021c6110 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d6008 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # 5fc0 + command 0x1b
fn 021c6134 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # byte list at r1: [count, ids...]: per id 9aec + command 0x2a + 9d10
fn 021c6158 ret=none inputs=300
  arg work
  arg buf 0x10
  fix arg1 0 u8 range 0 6
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9d10 force ret=none args=v,v,v

# # entries of 16 bytes at work + 0x7e0 (count work[0x782]): mark (+0x7ed) the ones that are not r1 and carry species 0xe4, then message 0x2c18; clear them
fn 021c6190 ret=u8 inputs=400
  arg work
  arg u32 any
  fix work 0x782 u8 range 0 7
  fix work 0x7e4 u32 alt const 0x7201 || any rep=8,16
  fix work 0x7ed u8 alt const 0 || any rep=8,16
  fix work 0x7ee u8 alt const 0 || const 1 || any rep=8,16
  stub 021d59b4 ret=u32 args=v,v
  stub 021e02dc ret=pick:0/1 args=v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021e03d8 ret=none args=v

# # message 0x2c3f: server r0, mon r1, params r3; reply words stored through stack0 (u16 + u8); true when the 0x41 slot is zero
fn 021c62d8 ret=bool
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x2c5d: mon r1, reply slot 0x11
fn 021c6350 ret=u8
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x2c72 with params r2
fn 021c6394 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # base damage 7a44(0x28, ...) scaled by (100 - 78ac(0x10)) / 100 (>= 1), 1678 record, 5540, 16f8; returns the damage
fn 021c63d0 ret=u16 inputs=300
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d7a44 ret=pick:1/20/100/500 args=v,v,v,v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021c1678 force ret=pick:0/0/1/2/3/4 args=p,v,v,v,out:2
  stub 021c5540 ret=none args=p,v,v
  stub 021c16f8 force ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5b74 ret=none args=p,v,v,v
  stub 021d5fa4 ret=none args=v,v
  stub 021c604c ret=none args=p,v,v
  stub 021c5bd4 ret=pick:0/1/2/6 args=p,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5f90 ret=none args=v,v

# # message 0x2cb0: params r3 and stack0 (hem record work + 0x1ae4), reply slot 0x51
fn 021c6458 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg work 0x1ae4
  stub 021c8810 force ret=none args=ptr
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x2cc8: mon r2, species r1; writes the 0x14-byte record at r3
fn 021c64b4 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x14
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 021d7630 ret=none args=v,v
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021d75a8 ret=none args=v,v
  stub 0201c1e8 ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12/13/14/15/16/17 args=v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x2cf4
fn 021c6564 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
