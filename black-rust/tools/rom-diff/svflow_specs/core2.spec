# svflow spec: core2 (0x021BD1D0-0x021BF324; B10g, written by B10g-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 / B10f-spec additions).
#
# What this area is: the second part of the flow core (btl_server_flow.c): the "message builders" that wrap one
# server->client command (d74f8 start / d75ec,d75a8,d7630 add argument / d70f0,d70fc send / d775c read back / d7540
# end), the queue wrappers that bracket a builder with cc8d8/cc95c on the queue header word (work+0x1D74), the party /
# switch helpers (bd200..bdd7c), and the two big routines bde98 (use a move: the whole attack sequence) and bec58
# (apply the effect of one move category).
# SvFlowWork fields seen here ("P" = the work image, `arg work`):
#   +0x04 main module, +0x08 pokecon (party container), +0x0C command queue buffer (pointer), +0x10 u32 (u16 used)
#   +0x1C helper struct (address only), +0x4A8 / +0x4AC / +0x4B0 pointers to small per-turn structs (+0x4B8 helper struct)
#   +0x786 u8 counter (capped 0x1E), +0x78A u8 flag bits (bit0 loop guard, bit2, bit4), +0x7C1 u8[6] per mon id flags,
#   +0x7D9 u8[4] per client counters, +0x850 / +0x854 / +0x860 / +0x864 pointers passed on to callees, +0x1A68 / +0x1AB8
#   helper structs (address only), +0x1AB0 / +0x1AB4 pointers to 20-byte records (copied / patched by bde98),
#   +0x1D74 packed queue header word (cmd_queue 021cc8d8 / 021cc95c), +0x1F74 u16, +0x1F88 helper struct
# Mon ids are small numbers 0..5 (021d5440 = BattleMon -> id); clients are 0..3.
#
# Rules (README): callees ported in cmd_queue / server_side / events / core1 run as REAL code on both sides:
# 021cc8d8 021cc95c 021cd4b0 021cd4fc 021cd64c (cmd_queue), 021bb85c 021bb878 (server_side), 021cc8a0 (events),
# 021bca08 021bcb68 (core1). Everything else outside this area, and every core2 function not yet in PORTED, is a stub
# (a stub of a core2 function becomes inactive the moment that address is in a PORTED list). `bx pc` veneers are
# stubbed at their own address; thunks (`ldr ip,[pc]; bx ip`: 021b8ddc -> 021b8d08, 0201c1bc, 0201beb8) are stubbed at
# the address the caller branches to. Stub argument kinds follow the REGISTERS set at each call site, not Ghidra's
# prototypes (hidden pass-through of r0..r3 is common here).
# The region S (server-side object, work[0]) and Q (command queue buffer, work[0xC]) blocks are copied from core1.spec.
#
# Skipped (not specced): veneers `bx pc` into unloaded 0x0689xxxx code: 021bd264 021bd274 021bd284 021bd294 021bd2a4
# 021bd4d4 021bd89c 021bd9c4 021bdb28 021bdb38 021bdb48 021bdb58 021bdb68 021bdb78 021bde08 021be338 021be348 021be358
# 021be368 021be378 021be388 021be398 021beb04 021bebb8 021bebc8 021befec 021beffc 021bf00c 021bf01c 021bf02c 021bf03c
# 021bf0a4 021bf0b4 021bf0c4 021bf0d4 021bf138 021bf148 021bf158 021bf204 021bf214 021bf224 021bf234 (42 entries, all
# `bx pc` + 2-byte filler; their callers stub them). Mid-function entries: 021be8d4 (inside 021be8c4), 021be92c (inside
# 021be924) are Ghidra splits, not functions. Jump tables are inside 021bec58 (3 tables at 021beca8 / 021bed56 / 021beeaa).
# Smoke ports (B10g-spec, in core2.rs): 021bec08 021bec2c (a ported callee: 021bec2c's 021bec08 stub goes inactive and its
# callee stubs below take over) 021bdc78 (real cmd_queue put_cmd on region Q + global 021f3b44) 021be8c4 (real
# cmd_queue advance_fields / set_word on the header word at work+0x1D74).
# Every function that calls 021cd4b0 declares `global 021f3b44 0x40` (its scratch words), every one that calls 021cd4b0 /
# 021cd64c / 021cd4fc declares the region Q block. Stack-pointer out arguments of stubs (c62d8, be7e4, beb14) use outm.
# Porters: other areas' specs stub these core2 functions by name (see the B10g-spec report): core1.spec 021bd2b4
# 021bd34c 021bd3e8 021bd584 021bd708 021bd8f0 021bd9d4 021bda00 021bdc2c 021bdc98 021bde48 021bde98; events.spec 021bdbb8.
# A port of one of them needs `force` on that stub line (harness job).
area core2

# ---- tiny leaves ------------------------------------------------------------------------------------------------

# the action record of the mon with id r1 (core1 021bca08, real) gets its done flag (+0xD) set
fn 021bd1d0 ret=none
  arg work
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# reset a 6-byte record {u16 +0, u8 +2 = 6, u8 +3 = 6, u8 +4 = 0, u8 +5 &= ~3}
fn 021bd1e0 ret=none
  arg buf 0x10

# list push: {u8 count; u8 a[6] at +1; u8 b[6] at +7; u8 c[6] at +0xD}; ignored when count >= 6
fn 021be52c ret=none
  arg buf 0x20
  arg u8 any
  arg u8 any
  arg u8 any
  fix arg0 0 u8 range 0 8

# list clear: count = 0, a[0..6] = 0x1F
fn 021be518 ret=none
  arg buf 0x10

# arg1 = main module (its +4 is passed on); battle type 3 sets +3
fn 021bd338 ret=none
  arg buf 0x10
  arg buf 0x10
  stub 021b8588 force ret=pick:0/1/2/3 args=v

# 1 when the client of mon r1 is our client
fn 021bec08 ret=u32
  arg work
  arg u32 any
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v

# for ability-like ids 0x96 / 0xa5 of a mon on our side, tell the client (021ba1b4 with the main module)
fn 021bec2c ret=none
  arg work
  arg u32 any
  arg u32 oneof 0x96 0xa5 0x96 0xa5 0 7 0x95
  stub 021bec08 ret=pick:0/1 args=p,v
  stub 021ba1b4 force ret=none args=v,v
  # callees of 021bec08 (real once 021bec08 is in PORTED): same stubs as in its own block
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v

# ---- message builders (one server->client command each) ----------------------------------------------------------------
# start (74f8) / args (75ec 75a8 7630) / send (70f0 70fc) / read back (775c) / end (7540)

fn 021bdbec ret=none
  arg work
  stub 021d74f8 ret=none args=v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bdc0c ret=none
  arg work
  stub 021d74f8 ret=none args=v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bdc58 ret=none
  arg work
  stub 021d74f8 ret=none args=v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bd554 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bdb88 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bde18 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bdbb8 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70fc ret=none args=p,v,v
  stub 021d7540 ret=none args=v

fn 021be8f4 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bf2ec ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be95c ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# returns the 021d775c read-back of id 0x51
fn 021bd8ac ret=u32
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# arg2 = {u16 +0 = id, u8 +6}
fn 021be6a8 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be9d0 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# like 021be6a8, reads the 021d775c answer back into arg2 +6
fn 021be620 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u8 args=v
  stub 021d7540 ret=none args=v

fn 021bea14 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be878 ret=u32
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# arg1 = {u16 +0}
fn 021bf168 ret=u32
  arg work
  arg buf 0x10
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# nothing happens for a fainted mon
fn 021bd6b8 ret=u32
  arg work
  arg u32 any
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

fn 021bea60 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bd850 ret=u32
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021bd89c ret=none args=p,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

fn 021beb14 ret=u32 inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  stub 021bebb8 ret=pick:0/1/2/3/4/5 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bebc8 ret=u32 args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/0x1f/0x1f/2 args=v
  stub 021d7540 ret=none args=v

# ---- queue wrappers: header word cc8d8 / builder / cc95c (cmd_queue, real) ---------------------------------------------

fn 021be8c4 ret=none
  arg work
  arg u32 any
  stub 021be8f4 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be924 ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021be95c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bdc2c ret=none
  arg work
  stub 021bdc58 ret=none args=p
  stub 021d74f8 ret=none args=v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bd518 ret=none
  arg work
  arg u32 any
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bd554 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be66c ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021be6a8 ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be994 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021be9d0 ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bf244 ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021bf2ec ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bf27c ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021bf2ec ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bf2b4 ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021bf2ec ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# ---- iterators over a client's party (S = server-side object, real 021bb85c / 021bb878) -----------------------------------

# first alive mon (client, index) of the slots into out struct {u8 client; u8 index; u8 done=0; u8 mode=1}
fn 021bd2b4 ret=none
  arg buf 0x10
  arg work
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  stub 021b9c00 force ret=nullable:3:u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v

# iterator step: arg0 = {u8 client; u8 index; u8 done; u8 mode}; stores the next mon in *arg2; 1 = found or finished
fn 021bd34c ret=u32
  arg buf 0x10
  arg work
  arg buf 0x10
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1f u8 range 0 3
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix arg0 0 u8 const 0
  fix arg0 1 u8 range 0 4
  fix arg0 2 u8 alt const 0 || const 0 || const 0 || const 1
  fix arg0 3 u8 alt const 0 || const 1
  stub 021b9bf0 force ret=nullable:3:u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v

# ---- party / switch bookkeeping ------------------------------------------------------------------------------------------

# out = {+2: veneer 021bd274 answer, +3: 6 or veneer 021bd2a4 answer, +5 bits 0,1 cleared}
fn 021bd200 ret=none
  arg buf 0x10
  arg work
  arg u32 any
  arg u32 any
  stub 021bd264 ret=pick:0/1/2 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bd274 ret=u8 args=v,v
  stub 021bd284 ret=pick:0/1 args=v
  stub 021bd294 ret=u32 args=v,v
  stub 021bd2a4 ret=u8 args=v,v

# mon at party index 0 or 2: swap/announce bookkeeping
fn 021bd3e8 ret=none
  arg work
  arg u32 any
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9c98 force ret=pick:0/1/2/3 args=v,v
  stub 021bd43c force ret=none args=p,v,v,v,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021bd4e4 ret=none args=p,v,v,v
  stub 021b9c00 force ret=nullable:3:u32 args=v,v
  stub 021bd518 ret=none args=p,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bd554 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# swap party slots a, b of `client` and put cmd 0x44 (unless r4 != 0)
fn 021bd43c ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021b9aec force ret=u32 args=v,v
  stub 021b8bec force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021bd4d4 ret=none args=v,v,v
  stub 021b9c10 force ret=none args=v,v,v
  stub 021b9bf0 force ret=nullable:3:u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5fc0 ret=none args=v,v

fn 021bd4e4 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9c00 force ret=nullable:3:u32 args=v,v
  stub 021bd518 ret=none args=p,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bd554 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# put cmd 0x3d (r1, r2, r3, u8 r4) after 021bd8f0
fn 021bd9d4 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u8 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021bd8f0 ret=none args=p,v,v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9d10 force ret=none args=v,v,v
  stub 021b9c10 force ret=none args=v,v,v
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8750 force ret=none args=v,v,v
  stub 021d8484 ret=none args=v
  stub 021dcf5c ret=none args=v
  stub 021d63ec ret=none args=v,v
  stub 021d6580 ret=none args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b6994 force ret=pick:0/1/2/3/4/5/6 args=v,v
  stub 021b8bec force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021bd9c4 ret=none args=v,v,v,v

# put cmd 0x3b (client of mon, mon id)
fn 021bdc78 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v

# put cmd 0x30 (4 words)
fn 021bebd8 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v

# cmd 0x2a, swap in party slot r2 <-> r3, bookkeeping of the new front mon, cmd 0x12
fn 021bd8f0 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9d10 force ret=none args=v,v,v
  stub 021b9c10 force ret=none args=v,v,v
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8750 force ret=none args=v,v,v
  stub 021d8484 ret=none args=v
  stub 021dcf5c ret=none args=v
  stub 021d63ec ret=none args=v,v
  stub 021d6580 ret=none args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b6994 force ret=pick:0/1/2/3/4/5/6 args=v,v
  stub 021b8bec force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021bd9c4 ret=none args=v,v,v,v

# per-client send loop: the mons of the 021bd34c iteration, header words, 021bdb* builders
fn 021bda00 ret=u32 inputs=300
  arg work
  fix work 0x7c1 u8 alt const 0 || any rep=6,1
  stub 021bdb28 ret=none args=v
  stub 021bd2b4 force ret=none args=ptr,p
  stub 021bd34c force ret=pick:0/1 args=ptr,p,out:4 seq
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bdb38 ret=none args=v,v
  stub 021bdb48 ret=none args=v,v
  stub 021bdbec ret=none args=p
  stub 021c88e4 force ret=pick:0/1 args=p
  stub 021bdb58 ret=none args=v
  stub 021bdb68 ret=nullable:2:u32 args=v seq
  stub 021bdb88 ret=none args=p,v
  stub 021bdb78 ret=nullable:2:u32 args=v seq
  stub 021bdc0c ret=none args=p
  stub 021d74f8 ret=none args=v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021d75ec ret=none args=v,v

# ---- fainting / reaction messages -------------------------------------------------------------------------------------

# cmd 0x3c, builder 021bde18, cmd 0x20 ...
fn 021bdd7c ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021bd1d0 ret=none args=p,v
  stub 021bde18 ret=none args=p,v
  stub 021c5108 force ret=none args=p,v,v
  stub 021d64f0 ret=none args=v
  stub 021bde08 ret=none args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bdc98 ret=none
  arg work
  arg u32 any
  arg u32 any
  fix work 0x7d9 u8 alt const 255 || any rep=4,1
  stub 021bdc78 force ret=none args=p,v
  stub 021bdd00 ret=pick:0/1 args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021c6190 force ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021bd0c0 force ret=none args=p,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bdd7c force ret=none args=p,v,v

# mon r1 (flag r2 == 0): up to n repeats of 021bd0c0 while the mon lives; then, if it lives, 021bdd7c
fn 021bdd00 ret=u32
  arg work
  arg u32 any
  arg u32 alt const 0 || const 0 || any
  stub 021c6190 force ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bd0c0 force ret=none args=p,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bdd7c force ret=none args=p,v,v

fn 021bd584 ret=u32
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  fix work 0x786 u8 range 0 40
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021bd60c ret=pick:0/0/1 args=p,v,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021b86e4 force ret=pick:0/1/2 args=v
  stub 021bd6b8 ret=pick:0/1/5 args=p,v
  stub 021b9920 force ret=nullable:2:u32 args=v,v,v
  stub 021d58a8 ret=u16 args=v,v
  stub 0209c0a4 ret=pick:0/0x100/0x400/0x1000/0x40000 args=v,v
  stub 021d78ac ret=pick:0/10/100/200/255 args=v
  stub 021bd708 force ret=pick:0/1 args=p,v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

fn 021bd60c ret=u32
  arg work
  arg u32 any
  arg u32 alt const 0 || const 1
  fix work 0x786 u8 range 0 12
  stub 021b86e4 force ret=pick:0/1/2 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021bd6b8 ret=pick:0/1/5 args=p,v
  stub 021b9920 force ret=nullable:2:u32 args=v,v,v
  stub 021d58a8 ret=u16 args=v,v
  stub 0209c0a4 ret=pick:0/0x100/0x400/0x1000/0x40000 args=v,v
  stub 021d78ac ret=pick:0/10/100/200/255 args=v
  stub 021bd708 force ret=pick:0/1 args=p,v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

fn 021bd708 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b86e4 force ret=pick:0/1/2 args=v
  stub 021b86c0 force ret=pick:0/1 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b902c force ret=pick:0/1 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bd850 ret=pick:0/1/5 args=p,v
  stub 021bd8ac ret=pick:0/1/5 args=p,v
  stub 021b86b8 force ret=pick:0/1 args=v
  stub 021d8244 ret=none args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021bd89c ret=none args=p,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021d75a8 ret=none args=v,v

# ---- move-effect helpers --------------------------------------------------------------------------------------------------

fn 021be3a8 ret=u32
  arg work
  arg u32 any
  arg buf 0x10
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021ef948 ret=pick:0/1 args=v
  stub 021c0450 force ret=none args=p,v,v,v

# arg0 = mon, arg1 -> word whose bits 7..22 are a move id (0xA5 = no-op)
fn 021bde48 ret=u32
  arg u32 any
  arg buf 0x10
  fix arg1 0 u32 oneof 0xa5 0xa6 0xa7 0 0x17 bits=7:16
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=pick:0/0xa5/0xa6/0xa7/0xa8/0x17 args=v
  stub 021d55a0 ret=pick:0/1 args=v,v

# the per-turn lists at [work+0x4B0] ({u8 n; u8 ids[6]; u8 b[6]; u8 c[6]}) and the 20-byte record at [work+0x1AB0]
fn 021be400 ret=none inputs=300
  arg work
  arg u32 any
  region L 0x20
  fix work 0x4b0 u32 addrof region:L
  fix region:L 0 u8 range 0 6
  region Z 0x40
  fix work 0x1ab0 u32 addrof region:Z
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021bea60 ret=none args=p,v,v,v
  stub 021c64b4 force ret=none args=v,v,v,v
  stub 021ca7f4 force ret=none args=p,v,v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d7394 ret=none args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021e02dc ret=none args=v,v,v
  stub 021bec58 force ret=none args=p,v,v,v
  stub 021e03d8 ret=none args=v,v
  stub 021d73d4 ret=none args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021be550 ret=none
  arg work
  arg u32 any
  arg u32 any
  region L 0x20
  fix work 0x4ac u32 addrof region:L
  fix region:L 0 u8 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021bea14 ret=none args=p,v,v,v
  stub 021c64b4 force ret=none args=v,v,v,v
  stub 021ca7f4 force ret=none args=p,v,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021c0520 force ret=none args=p,v,v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021e02dc ret=none args=v,v,v
  stub 021bec58 force ret=none args=p,v,v,v
  stub 021e03d8 ret=none args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# arg4 = out record {u8 +0 = 1, +1, +7}; locals b9/b8 (0x1f = none) come from 021beb14
fn 021beab4 ret=u32 inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021beb14 ret=pick:0/1 args=p,v,v,v,outm:1:0x1f,outm:1:0x1f
  stub 021beb04 ret=u8 args=v
  stub 021bebb8 ret=pick:0/1/2/3/4/5 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bebc8 ret=u32 args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/0x1f/0x1f/2 args=v
  stub 021d7540 ret=none args=v

# id r2 in the 3-entry table at 0x021EFDB8 (0x207 0x206 0x208): pick the lowest-index record and put cmd 0x5b
fn 021be6ec ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 oneof 0x207 0x206 0x208 0x207 0x206 0x208 0x100 0
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  fix work 0x782 u8 range 0 6
  stub 021d5bb0 ret=pick:0/0/1 args=v,v
  stub 021d6e74 ret=pick:0/0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bcb14 ret=nullable:3:ptr:work:0x7e0:24 args=p,v,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021d6e40 ret=none args=v,v,v
  stub 021bd148 ret=none args=p,v
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021d8340 ret=pick:0/1/2/3/4/5 args=v

# arg4 -> flag byte; r3 is a move-word passed to 021be878
fn 021be7e4 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021be878 ret=pick:0/1/7 args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021be8c4 ret=none args=p,v
  stub 021c5718 force ret=none args=p
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021be8f4 ret=none args=p,v

# ---- bf04c / bf0e4 / bf1b4: loops over the targets of a veneer iterator -----------------------------------------------------

fn 021bf04c ret=none
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  stub 021bf0a4 ret=none args=v
  stub 021bf0b4 ret=nullable:2:u32 args=v
  stub 021bfa00 force ret=pick:0/1 args=p,v,v,v
  stub 021bf0c4 ret=none args=v,v
  stub 021c572c force ret=none args=p,v
  stub 021bf0d4 ret=nullable:3:u32 args=v seq

fn 021bf0e4 ret=none inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 alt const 0 || any
  fix arg3 0xc u32 alt const 7 || any
  stub 021bf138 ret=pick:0/1/1 args=v
  stub 021bf148 ret=nullable:3:u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021bf158 ret=none args=v,v

fn 021bf1b4 ret=none
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  stub 021bf204 ret=none args=v
  stub 021bf214 ret=nullable:2:u32 args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021bf224 ret=none args=v,v
  stub 021bf234 ret=nullable:3:u32 args=v seq

# ---- the two big routines -----------------------------------------------------------------------------------------------

# apply the effect of move/ability r2 (category = 0201c1bc(r2), r3 = a target word) of mon r1; returns 1 when the
# effect is handled. [work+0x4A8] points at a small flag struct {+5: bit0 "started", bit1}; cd4fc puts a placeholder
# cmd 0x30 at [work+0xC]; switch tables: 021beca8 (021bfc08 answer 1..3), 021bed56 (category 0..9), 021beeaa (category 0..13).
fn 021bec58 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  fix region:P 5 u8 any
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bec2c ret=none args=p,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021bfc08 force ret=pick:0/1/2/3/4 args=p,v,v,v
  stub 021befec ret=none args=v
  stub 021bd200 ret=none args=v,p,v,v
  stub 021c65a8 force ret=pick:0/1/2 args=p,v,v,v
  stub 021bfbd0 force ret=none args=p,v,v,v
  stub 021beffc ret=none args=v
  stub 021bf04c force ret=none args=p,v,v,v
  stub 021bf0e4 force ret=none args=p,v,v,v,v
  stub 021bf00c ret=pick:0/1 args=v
  stub 021c5718 force ret=none args=p
  stub 021bf83c force ret=none args=p,v,v,v
  stub 021bf168 force ret=pick:0/1 args=p,v,v
  stub 021bf3d8 force ret=none args=p,v,v,v,v
  stub 021bf4ac force ret=none args=p,v,v,v,v
  stub 021bf8d4 force ret=none args=p,v,v,v
  stub 021bf01c ret=pick:0/1 args=v
  stub 021bf1b4 force ret=none args=p,v,v,v,v
  stub 021bf02c ret=pick:0/1 args=v
  stub 021bf27c ret=none args=p,v,v
  stub 021c091c force ret=none args=p,v,v,v,v,v
  stub 021c2dc4 force ret=none args=p,v,v,v
  stub 021c2488 force ret=none args=p,v,v,v
  stub 021c3184 force ret=none args=p,v,v,v
  stub 021c3438 force ret=none args=p,v,v,v
  stub 021c376c force ret=none args=p,v,v,v
  stub 021c3274 force ret=none args=p,v,v,v
  stub 021c3aa8 force ret=none args=p,v,v
  stub 021c3d08 force ret=none args=p,v,v,v
  stub 021bf03c ret=none args=v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c60ec force ret=none args=p,v,v
  stub 021bf244 ret=none args=p,v,v
  stub 021bf2b4 ret=none args=p,v,v
  stub 021c4f64 force ret=none args=p,v
  stub 021bec08 ret=pick:0/1 args=p,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021bd264 ret=pick:0/1/2 args=v
  stub 021bd274 ret=u8 args=v,v
  stub 021bd284 ret=pick:0/1 args=v
  stub 021bd294 ret=u32 args=v,v
  stub 021bd2a4 ret=u8 args=v,v
  stub 021bf2ec ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# use a move: mon r1, arg2 -> move word (bits 7..22 = move id, bits 4..6 = target slot), r3 = a word passed to 021e02dc.
# Pointers in the work: [0x4A8] [0x4B0] flag structs (address only), [0x850] [0x854] [0x864] (address only),
# [0x1AB0] D and [0x1AB4] E = 20-byte records (E is copied over D, D+2 / D+6 / D+0x10 are touched).
fn 021bde98 ret=none inputs=400
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  fix arg2 0 u32 oneof 0xa5 0x10 0x206 0 1 0x17 bits=7:16
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  region D 0x40
  fix work 0x1ab0 u32 addrof region:D
  region E 0x40
  fix work 0x1ab4 u32 addrof region:E
  stub 021d5c08 ret=pick:0/1/2/0x10/0x10 args=v
  stub 021bd1e0 force ret=none args=v
  stub 021be518 force ret=none args=v
  stub 021bde48 ret=pick:0/0/0x17/0x45 args=v,v
  stub 021d80bc ret=u32 args=v,v,v,v
  stub 021d82d0 ret=none args=v,v,v
  stub 021d5778 ret=pick:0/1/2/3/4 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021e02dc ret=none args=v,v,v
  stub 021be924 ret=none args=p,v,v
  stub 021c001c force ret=pick:0/0/1 args=p,v,v,v
  stub 021c62d8 force ret=pick:0/1 args=v,v,v,v,outm:3:0xa5/0/7
  stub 021c5604 force ret=none args=p,v,v
  stub 021c64b4 force ret=none args=v,v,v,v
  stub 021ca7f4 force ret=none args=p,v,v,v,v
  stub 021c07ac force ret=none args=p,v,v,v,v
  stub 021c0450 force ret=none args=p,v,v,v
  stub 021bebd8 ret=none args=p,v,v,v
  stub 021be3a8 ret=pick:0/1 args=p,v,v
  stub 021be338 ret=none args=v
  stub 021be348 ret=none args=v
  stub 021be358 ret=u8 args=v
  stub 021be368 ret=none args=v
  stub 021be378 ret=none args=v,v
  stub 021be388 ret=none args=v,v
  stub 021be66c ret=none args=p,v,v,v
  stub 021be620 ret=none args=p,v,v
  stub 021c6458 force ret=pick:0/1 args=v,v,v,v,v
  stub 021c99a0 force ret=none args=p,v
  stub 021c8810 force ret=none args=v
  stub 021c5620 force ret=none args=p,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021d5fc0 ret=none args=v,v
  stub 021c024c force ret=pick:0/0/1 args=p,v,v
  stub 021be7e4 ret=pick:0/0/1 args=p,v,v,v,outm:1:1
  stub 021be6ec force ret=pick:0/0/1 args=p,v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021be398 ret=none args=v,v,v,v
  stub 021beab4 force ret=pick:0/1 args=p,v,v,v,v
  stub 021be550 force ret=none args=p,v,v
  stub 021bec58 force ret=pick:0/1/2 args=p,v,v,v
  stub 021d6744 ret=none args=v,v,v,v,v,v
  stub 021d69ac ret=pick:0/1 args=v,v
  stub 021c0780 force ret=none args=p,v,v,v
  stub 021bfe60 force ret=none args=p,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021bff60 force ret=none args=p,v,v
  stub 021be994 ret=none args=p,v,v,v
  stub 021e0394 ret=none args=v,v
  stub 021be400 force ret=none args=p,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=pick:0/0xa5/0xa6/0xa7/0xa8/0x17 args=v
  stub 021d55a0 ret=pick:0/1 args=v,v
  stub 021be95c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021ef948 ret=pick:0/1 args=v
  stub 021be6a8 ret=none args=p,v,v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=u8 args=v
  stub 021be878 ret=pick:0/1/7 args=p,v,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021be8c4 ret=none args=p,v
  stub 021c5718 force ret=none args=p
  stub 021be8f4 ret=none args=p,v
  stub 021be9d0 ret=none args=p,v,v,v
