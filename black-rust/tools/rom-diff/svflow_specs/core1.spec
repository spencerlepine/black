# svflow spec: core1 (0x021BB968-0x021BD1D0; B10f, written by B10f-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 additions, plus `ret=ofs:work`, B10f-spec).
# GENERATED from a template by a macro expander: the stub lines of a callee are identical in every function that calls it.
#
# What this area is: the first part of the flow core (btl_server_flow.c): create/destroy of the SvFlowWork, the
# per-turn "action list" (6 records of 16 bytes at work+0x7E0, count u8 at work+0x782) and its helpers.
# SvFlowWork fields seen here ("P" = the work image, `arg work`):
#   +0x00 server-side object S (pointer; server_side's real code reads it: slots at S+0x14, S+0xCB4 list)
#   +0x04 main module   +0x08 pokecon (party container)   +0x0C command queue buffer   +0x14 u32 state   +0x18 u32 mode
#   +0x1C, +0x3E0, +0x4B8, +0x1AB8 helper structs (address only: passed to stubs/veneers)   +0x1D74 queue header word
#   +0x77C u8 "already built" flag, +0x77D/+0x783 u8, +0x780 u16 heap id, +0x782 u8 record count, +0x784 u8
#   +0x7E0 6 action records * 16: +0 BattleMon ptr, +4 action word (low nibble = kind, bit 10 flag, bits 7.. fields),
#          +8 priority word, +0xC u8 client id, +0xD u8 done flag, +0xE u8;  +0x840 scratch record
#   +0x4A4 u32, +0x4CE list {u8 count; u8 ids[]}, +0x1F78 u32, +0x1FE8 u8
# The region S (a 0x1000-byte server-side object) and the region Q (0xC00-byte command queue) are separate `region`s
# whose addresses are stored in work[0] / work[0xC] by `fix`; real callees from server_side / cmd_queue / events run on
# them on both sides. 021cc8d8 / 021cc95c (cmd_queue, real) work on the header word at work+0x1D74.
#
# Rules (README): callees ported in cmd_queue / server_side / events run as REAL code on both sides: 021bb85c 021bb878
# 021bb890 021bb8a8 021bb8b0 021bb920 021bb930 (server_side), 021cc8d8 021cc95c 021cd4b0 (cmd_queue), 021cc8c8 (events).
# 021cb730 (events `use_item`) is a `force` stub in 021bcce0 (it needs a BattleMon model). Everything else outside
# this area, and every core1 function not yet in PORTED, is a stub. `bx pc` veneers (021bbb08 ...) are stubbed at their own
# address (the real targets are unloaded 0x0689xxxx code); the thunk 021b8ddc is stubbed at its target 021b8d08.
# Stub argument kinds follow the REGISTERS set at each call site, not Ghidra's prototypes.
#
# Skipped (not specced): veneers 021bbb08 021bbb18 021bbb28 021bbb38 021bbcc4 021bbf88 021bbf98 021bc0d8 021bc408 021bc4cc
# 021bc540 021bceec 021bd138 (`bx pc` into unloaded overlay code), thunk 021bbb3c (`ldr ip,[pc]; bx ip` into 0x06898bfc);
# jump-table/switch continuation blocks inside 021bc610 (table 021bc728) and 021bcce0 (table 021bcda0) are part of those functions.
# Porters: server_side.spec stubs 021bb9d8 021bbb48 021bbb60 021bbc34 021bbc44 021bbd20 021bbd2c 021bbd80 021bbd8c 021bc360
# 021bc390 021bc55c, and an old stub of a function becomes inactive the moment it is in a PORTED list: a port of one of those
# needs `force` on the stub line in server_side.spec (harness job; the Rust server_side port reaches them through env.call).
area core1

# ---- create / destroy -------------------------------------------------------------------------------------------

# alloc the work (heap id = u16 stack arg), store server, main, pokecon, queue, mode; call init; returns the work address
fn 021bb968 ret=u32
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u16 any
  stub 02030734 ret=ptr:work:0:1 args=v,v,v,v,v
  stub 0201b70c ret=u32 args=
  stub 021bb9d8 ret=none args=p
  stub 021d6eac ret=none args=
  stub 021ef874 ret=none args=v
  stub 021bbb08 ret=none args=p
  stub 021bbb18 ret=none args=p
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021bbb28 ret=none args=p,v,v,v
  stub 021c7c18 force ret=none args=p
  stub 021c8588 ret=none args=p
  stub 021d8230 ret=none args=p
  stub 021c5474 force ret=none args=p
  stub 021bbb38 ret=none args=

# init: clear the sub-structs and the state bytes
fn 021bb9d8 ret=none
  arg work
  stub 021d6eac ret=none args=
  stub 021ef874 ret=none args=v
  stub 021bbb08 ret=none args=p
  stub 021bbb18 ret=none args=p
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021bbb28 ret=none args=p,v,v,v
  stub 021c7c18 force ret=none args=p
  stub 021c8588 ret=none args=p
  stub 021bbb38 ret=none args=
  stub 021c5474 force ret=none args=p
  stub 021d8230 ret=none args=p

# free the work
fn 021bbb48 ret=none
  arg work
  stub 02049238 ret=none args=v
  stub 020307b0 ret=none args=p

# check every client's party: returns 1 when something was queued
fn 021bbb60 ret=u32
  arg work
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  stub 021b8714 ret=ptr:work:0x1000:16 args=v
  stub 021c3b68 force ret=pick:0/1 args=p,v,v
  stub 021b9bf0 ret=nullable:3:u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bd8f0 force ret=none args=p,v,v,v
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021b8750 ret=none args=v,v,v
  stub 021bda00 force ret=pick:0/1 args=p

# small flag clears
fn 021bbc34 ret=none
  arg work

fn 021bbd20 ret=none
  arg work

fn 021bbd80 ret=none
  arg work

# turn start (state 0): build the action list
fn 021bbc44 ret=u32
  arg work
  arg u32 any
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix work 0x77c u8 alt const 0 || any
  stub 021c84e8 ret=none args=p
  stub 021bbcc4 ret=none args=p
  stub 021d6ee8 ret=none args=
  stub 021d74e4 ret=none args=
  stub 021bbcd4 ret=none args=p
  stub 021bc610 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,v
  stub 021bbfa8 force ret=u8 args=p,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021d5bdc ret=pick:0/1 args=v,v
  stub 021c6110 force ret=none args=p,v,v

# iterate the 021bd2b4/021bd34c list; for each entry whose BattleMon passes 021d5bdc call 021c6110
fn 021bbcd4 ret=none
  arg work
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021d5bdc ret=pick:0/1 args=v,v
  stub 021c6110 force ret=none args=p,v,v

# like 021bbc44 for a resumed turn: builds the list unless 0x77c is set, returns 3 when 021bc220 asks
fn 021bbd2c ret=u32
  arg work
  arg u32 any
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix work 0x77c u8 alt const 0 || any
  stub 021d74e4 ret=none args=
  stub 021bc220 force ret=pick:0/1 args=p,p
  stub 021bbfa8 force ret=u8 args=p,v

# action-list builder / executor (the big one)
fn 021bbd8c ret=u32 inputs=400
  arg work
  arg u32 any
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  fix work 0x77c u8 alt const 0 || any
  stub 021d74e4 ret=none args=
  stub 021bbf88 ret=pick:0/1 args=p,v
  stub 021bc610 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bdc98 force ret=none args=p,v,v
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021bcce0 ret=pick:0/1/2/3/4/5/6/6 args=p,p
  stub 021bda00 force ret=pick:0/1 args=p
  stub 021c5200 force ret=pick:0/1 args=p
  stub 021bc4dc ret=pick:0/1 args=p,p
  stub 021bbf98 ret=pick:0/1 args=p,v
  stub 021c4614 force ret=none args=p
  stub 021bc2b0 ret=pick:0/1 args=p
  stub 021c6158 force ret=none args=p,p
  stub 021bc540 ret=pick:0/1/2/3/4 args=p,v,out:4
  stub 021b8b9c ret=pick:0/1 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b8bc4 ret=pick:0/1 args=v,v
  stub 021b9b98 ret=pick:0/1 args=v
  stub 021c859c ret=none args=p
  stub 021c85b8 ret=none args=p
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021d5fc0 ret=none args=v,v
  stub 021bc418 force ret=none args=p,v,v
  stub 021d69ac ret=pick:0/1 args=v,v
  stub 021c0780 force ret=none args=p,v,v,v

# execute records from index r1 on; returns the next index / count
fn 021bbfa8 ret=u8 inputs=400
  arg work
  arg u8 range 0 7
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  fix work 0x14 u32 oneof 0 0 1 6 2 3 4 5
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021bdc2c force ret=none args=p
  stub 021bc0e8 ret=none args=p,p,v
  stub 021bcce0 ret=pick:0/1/2/3/4/5/6/6 args=p,p
  stub 021c5200 force ret=pick:0/1 args=p
  stub 021bc2b0 ret=pick:0/1 args=p
  stub 021c446c force ret=pick:0/1 args=p
  stub 021bc0d8 ret=pick:0/1 args=p,v
  stub 021c8534 ret=pick:0/1 args=p
  stub 021bc4dc ret=pick:0/1 args=p,p
  stub 021c6158 force ret=none args=p,p
  stub 021c859c ret=none args=p
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c85b8 ret=none args=p
  stub 021b8b9c ret=pick:0/1 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b8bc4 ret=pick:0/1 args=v,v
  stub 021b9b98 ret=pick:0/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v
  stub 021d7b10 ret=u32 args=v,v
  stub 021d62a8 ret=pick:0/1 args=v
  stub 021d7540 ret=none args=v
  stub 021d78ac ret=pick:0/1 args=v seq
  stub 021bceec ret=pick:0/1 args=p,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d69ac ret=pick:0/1 args=v,v
  stub 021c0780 force ret=none args=p,v,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021b86b8 ret=pick:0/1/2 args=v
  stub 021bde98 force ret=none args=p,v,in:4,v
  stub 021c60ec force ret=none args=p,v,v
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021bc418 force ret=none args=p,v,v
  stub 021cb730 force ret=pick:0/1/2 args=p,v,v,v,v
  stub 021bd708 force ret=pick:0/1 args=p,v,v,v
  stub 021c54bc force ret=none args=p,v
  stub 021bdc98 force ret=none args=p,v,v
  stub 021bda00 force ret=pick:0/1 args=p
  stub 021bc540 ret=pick:0/1/2/3/4 args=p,v,out:4
  stub 021b8d08 ret=pick:0/1/2/3 args=v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021bd3e8 force ret=none args=p,v
  stub 021b86e4 ret=pick:0/1/2 args=v
  stub 021bd584 force ret=pick:0/1 args=p,v
  stub 0201bd28 ret=u32 args=v,v
  stub 021c6350 force ret=u8 args=v,v

# for every record sort key: recompute the priority word (bits 0..12 = 021bc948, 13.. = 021bc8ec / 021c6350)
fn 021bc0e8 ret=none inputs=400
  arg work
  arg work 0x7e0
  arg u32 range 0 6
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021bc948 ret=u16 args=p,v,v
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021bc8ec ret=u8 args=p,v,v
  stub 021c6350 force ret=u8 args=v,v
  stub 021bc874 ret=none args=p,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v
  stub 021d7b10 ret=u32 args=v,v
  stub 021d62a8 ret=pick:0/1 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d78ac ret=pick:0/1 args=v seq

# per client: for each queued action word of kind 3 (switch-in) not flagged, call 021bd9d4
fn 021bc220 ret=none inputs=300
  arg work
  arg buf 0x40
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  fix arg1 0x30 u8 range 0 3 rep=4,1
  fix arg1 0 u32 oneof 3 3 3 0 1 2 bits=0:4 rep=12,4
  fix arg1 0 u32 alt const 0 || const 1 bits=10:1 rep=12,4
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021bda00 force ret=pick:0/1 args=p
  stub 021c5200 force ret=pick:0/1 args=p

# 1 when every client's party passes 021b8b9c/021b9b98 (stack flag bytes)
fn 021bc2b0 ret=bool
  arg work
  stub 021b8b9c ret=pick:0/1 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b8bc4 ret=pick:0/1 args=v,v
  stub 021b9b98 ret=pick:0/1 args=v

# 1 when some client on our team has a usable party
fn 021bc310 ret=u32
  arg work
  stub 021b8b9c ret=pick:0/1 args=v,v
  stub 021b8bc4 ret=pick:0/1 args=v,v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b98 ret=pick:0/1 args=v

# queue header cleared, then 021bd584 on the first mon of our client
fn 021bc360 ret=u32
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b990c ret=u32 args=v,v,v
  stub 021bd584 force ret=pick:0/1 args=p,v

# queue header cleared, run 021bc3b0, result = queue got a command (the stub cannot write the queue: always 0 until 021bc3b0 is ported)
fn 021bc390 ret=u32
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  stub 021bc3b0 ret=none args=p

# push cmd 0x29 (client, count) for every client whose slot is used (mode 1)
fn 021bc3b0 ret=none inputs=300
  arg work
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x18 u32 oneof 1 1 1 0 2
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021bc408 ret=pick:0/1/2/3 args=p,v,out:4

# switch announce: cmd 0x48, then the incoming mon bookkeeping
fn 021bc418 ret=none inputs=300
  arg work
  arg u32 range 0 3
  arg u32 range 0 5
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9c50 ret=none args=v,v,out:4,out:4
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c5108 force ret=none args=p,v,v
  stub 021ef948 ret=pick:0/1 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c5620 force ret=none args=p,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021d8548 ret=none args=v
  stub 021dcfb4 ret=none args=v
  stub 021bc4cc ret=none args=p,v,v,v,v

# collect the clients' switch lists into the list at r1 (u8 count + ids), 021bb930 on S
fn 021bc4dc ret=u32 inputs=300
  arg work
  arg work 0x4ce
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  stub 021bc540 ret=pick:0/1/2/3/4 args=p,v,out:4

# work + 0x4B8
fn 021bc550 ret=u32
  arg work

# u8 at work+0x784
fn 021bc55c ret=u8
  arg work

# for each record with a move-use action: run the held-item/ability check 021e02dc and apply
fn 021bc568 ret=none inputs=300
  arg work
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021d59b4 ret=u32 args=v,v
  stub 021e02dc ret=pick:0/1 args=v,v,v
  stub 021c6394 force ret=none args=v,v,v
  stub 021e0394 ret=none args=v,v

# fill the records from the clients' action words (r1 = S-like buffer: counts at +0x30, words at client*12 + j*4)
fn 021bc610 ret=u8 inputs=400
  arg work
  arg buf 0x40
  arg work 0x7e0
  arg u32 any
  region S 0x1000
  fix work 0 u32 addrof region:S
  fix region:S 0x1f u8 alt const 255 || any rep=4,12
  fix region:S 0x1d u8 range 0 3 rep=4,12
  fix region:S 0xCBA u8 alt range 0 6 || any
  fix arg1 0x30 u8 range 0 3 rep=4,1
  stub 021d83e0 ret=pick:0/1 args=v
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021b9ab0 ret=pick:0/1/2/3/4/5 args=v
  stub 021b9bf0 ret=nullable:3:u32 args=v,v
  stub 021b86b8 ret=pick:0/1/2 args=v
  stub 021bc8ec ret=u8 args=p,v,v
  stub 021bc948 ret=u16 args=p,v,v
  stub 021bc874 ret=none args=p,v
  stub 021c6350 force ret=u8 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v
  stub 021d7b10 ret=u32 args=v,v
  stub 021d62a8 ret=pick:0/1 args=v
  stub 021d7540 ret=none args=v
  stub 021d78ac ret=pick:0/1 args=v seq
  stub 0201bd28 ret=u32 args=v,v

# sort the first n records by priority (descending), ties by 021d78ac(2)
fn 021bc874 ret=none inputs=300
  arg work 0x7e0
  arg u32 range 0 6
  fix work 0x7e8 u32 range 0 3 rep=6,16
  stub 021d78ac ret=pick:0/1 args=v seq

# priority word of a mon: event-variable stack evaluation (speed with modifiers)
fn 021bc8ec ret=u8
  arg work
  arg u32 any
  arg u32 any
  stub 0201bd28 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v
  stub 021d7540 ret=none args=v

# speed/priority value (u16) of a mon; second arg flag inverts the result
fn 021bc948 ret=u16 inputs=400
  arg work
  arg u32 any
  arg u32 alt const 0 || const 1
  stub 021d59b4 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v
  stub 021d7b10 ret=u32 args=v,v
  stub 021d62a8 ret=pick:0/1 args=v
  stub 021d7540 ret=none args=v

# record whose mon id equals r1, else null
fn 021bca08 ret=ofs:work
  arg work
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# next record after r1 whose mon id equals r2, else null
fn 021bca50 ret=ofs:work inputs=300
  arg work
  arg u32 oneof 0x02E007E0 0x02E007F0 0x02E00800 0x02E00810 0x02E00820 0x02E00830 0x02E00840 0x02E00850
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# first not-done kind-1 record from index r2 whose 021d8340 equals r1
fn 021bcaac ret=ofs:work inputs=300
  arg work
  arg u32 range 0 7
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021d8340 ret=pick:0/1/2/3/4/5 args=v

# first record with action id r1 whose mon is on side r2 and alive (021bcaac / 021bcb68 walk)
fn 021bcb14 ret=ofs:work inputs=300
  arg work
  arg u32 range 0 7
  arg u32 any
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021bcaac ret=nullable:3:ptr:work:0x7e0:1 args=p,v,v seq
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b9a24 ret=pick:0/1 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021bcb68 ret=pick:0/1/2/3/4/5/6 args=p,p
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021d8340 ret=pick:0/1/2/3/4/5 args=v

# index of record r1 in the list (or the count)
fn 021bcb68 ret=u8
  arg work
  arg u32 oneof 0x02E007E0 0x02E007F0 0x02E00800 0x02E00810 0x02E00820 0x02E00830 0x02E00840 0x02E00850 0x02E00000
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16

# move record r1 so that it sits before the first not-done record at or after index r2 (insertion); returns its old index or -1
fn 021bcb98 ret=i32 inputs=400
  arg work
  arg u32 oneof 0x02E007E0 0x02E007F0 0x02E00800 0x02E00810 0x02E00820 0x02E00830 0x02E00840 0x02E00850 0x02E00000
  arg s32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16

# remove record r1 from the list and append it at the end
fn 021bcc50 ret=none inputs=300
  arg work
  arg u32 oneof 0x02E007E0 0x02E007F0 0x02E00800 0x02E00810 0x02E00820 0x02E00830 0x02E00840 0x02E00850 0x02E00000
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16

# execute one action record (kind switch, table 021bcda0)
fn 021bcce0 ret=u8 inputs=800
  arg work
  arg u32 oneof 0x02E007E0 0x02E007F0 0x02E00800 0x02E00810 0x02E00820 0x02E00830
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  fix work 0x14 u32 oneof 0 5 1 2 3 4
  fix work 0x7ed u8 alt const 0 || const 0 || const 0 || any rep=6,16
  stub 021c859c ret=none args=p
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bceec ret=pick:0/1 args=p,v
  stub 021b86e4 ret=pick:0/1/2 args=v
  stub 021d69ac ret=pick:0/1 args=v,v
  stub 021c0780 force ret=none args=p,v,v,v
  stub 021bcefc ret=none args=p,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021bc418 force ret=none args=p,v,v
  stub 021bc568 ret=none args=p
  stub 021bcfac ret=u8 args=p,v
  stub 021bde98 force ret=none args=p,v,in:4,v
  stub 021cb730 force ret=pick:0/1/2 args=p,v,v,v,v
  stub 021bd708 force ret=pick:0/1 args=p,v,v,v
  stub 021bdc98 force ret=none args=p,v,v
  stub 021bda00 force ret=pick:0/1 args=p
  stub 021bd584 force ret=pick:0/1 args=p,v
  stub 021bd3e8 force ret=none args=p,v
  stub 021c54bc force ret=none args=p,v
  stub 021c60ec force ret=none args=p,v,v
  stub 021bcf70 ret=none args=p,v,v
  stub 021c85b8 ret=none args=p
  stub 021d5b68 ret=pick:0/0/0/1 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021b86b8 ret=pick:0/1/2 args=v
  stub 021b8d08 ret=pick:0/1/2/3 args=v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021d5844 ret=u32 args=v
  stub 021b871c ret=u32 args=v
  stub 02018480 ret=pick:0/1 args=v,v
  stub 021b85a0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d78ac ret=pick:0/3/30/45/70/1000/60000 args=v

# event-variable wrappers: cc8d8 / cc95c bracket
fn 021bcefc ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021bcf34 ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bcf34 ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

fn 021bcf70 ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# can the mon use the item / hold its turn: returns 0..4
fn 021bcfac ret=u8 inputs=400
  arg work
  arg u32 any
  stub 021b86b8 ret=pick:0/1/2 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 ret=pick:0/1/2/3 args=v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021d5844 ret=u32 args=v
  stub 021b871c ret=u32 args=v
  stub 02018480 ret=pick:0/1 args=v,v
  stub 021b85a0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021bd080 ret=pick:0/1 args=p,v
  stub 021d59b4 ret=pick:0/5/30/90/100/500/60000 args=v,v
  stub 021d78ac ret=pick:0/3/30/45/70/1000/60000 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v

fn 021bd080 ret=u32
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/1/2/500/10000/20000/40000 args=v
  stub 021d7540 ret=none args=v

# redirect/replace the action of the record for mon id r1 (side r2)
fn 021bd0c0 ret=u32 inputs=300
  arg work
  arg u32 range 0 5
  arg u32 any
  # all records hold one mon: the original does not null-check 021bca50 (a done kind-6 record with no later record of
  # the mon faults in the ROM; the port returns 0 there), so keep at least two records of the same mon
  fix work 0x782 u8 range 2 6
  fix work 0x7e0 u32 const 0x1234 rep=6,16
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021bca08 ret=nullable:4:ptr:work:0x7e0:1 args=p,v
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021bca50 ret=ptr:work:0x7e0:1 args=p,p,v
  stub 021bde48 force ret=pick:0/1 args=v,v
  stub 021bd138 ret=u32 args=p,v
  stub 021d82fc ret=none args=v,v
  stub 021bcce0 force ret=pick:0/1/2/3/4/5/6/6 args=p,p
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

fn 021bd148 ret=u32
  arg work
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021bca08 ret=nullable:4:ptr:work:0x7e0:1 args=p,v
  stub 021bcb98 ret=pick:0xFFFFFFFF/0/1/3/5 args=p,p,v seq
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

fn 021bd170 ret=u32 inputs=300
  arg work
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021bcaac ret=nullable:3:ptr:work:0x7e0:1 args=p,v,v seq
  stub 021bcb98 ret=pick:0xFFFFFFFF/0/1/3/5 args=p,p,v seq
  stub 021d8454 ret=pick:0/1/1/2/3/4/5/6/6/7 args=v
  stub 021d8340 ret=pick:0/1/2/3/4/5 args=v

fn 021bd1b0 ret=u32
  arg work
  arg u32 range 0 7
  fix work 0x782 u8 range 0 6
  fix work 0x7e4 u32 oneof 0 1 2 3 3 3 4 5 6 6 7 8 9 bits=0:4 rep=6,16
  fix work 0x7e4 u32 alt const 0 || const 1 bits=10:1 rep=6,16
  fix work 0x7e8 u32 range 0 3 rep=6,16
  fix work 0x7ec u8 range 0 3 rep=6,16
  fix work 0x7ed u8 alt const 0 || const 0 || any rep=6,16
  stub 021bca08 ret=nullable:4:ptr:work:0x7e0:1 args=p,v
  stub 021bcc50 ret=none args=p,p
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
