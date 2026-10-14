# svflow spec: core3 (0x021BF324-0x021C0F24; B10h, written by B10h-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 / B10f-spec / B10f-spec2 additions).
#
# What this area is: the third part of the flow core (btl_server_flow.c): "who is affected by a move/ability" helpers built on
# a mon list object (the list lives in unloaded 0x0689xxxx code, reached through `bx pc` veneers), the HANDEX/event wrappers
# (bf6e8 runs one event on one mon, bf78c is its message-builder core), the ability/abilities gate (c001c "can this ability
# be used", c0450 its failure message, c0520 the failure message table), the damage list (c091c, c0a4c ... c0d90: a record
# {u8 hit count; u8 miss count; u8 total; u16 id|flags; u16 damage ...} built per target) and its readers (c0c38 ... c0d70).
# SvFlowWork fields seen here ("P" = the work image, `arg work`):
#   +0x04 main module, +0x08 pokecon (party container), +0x0C command queue buffer (pointer), +0x1D74 packed queue header word
#   +0x1AB8 / +0x1AE4 / +0x1F7C helper structs (address only), +0x1F78 u32 state (2/3/4 are tested), +0x1FE9 u8 (c091c clears it)
#   +0x4A8 pointer to a small per-turn struct (bytes +4, +5 written), +0x4B4 / +0x858 / +0x85C / +0x860 pointers passed on to
#   callees (+0x858+0x14 / +0x858+0x18 are also loaded as pointers), +0x77F u8
# Message-builder pattern (d74f8 begin / d75ec,d75a8,d7630 add argument / d70f0 send / d775c read back / d7540 end): each is a stub.
#
# Rules (README): callees ported in cmd_queue / server_side / events / core1 run as REAL code on both sides: 021cc8d8 021cc95c
# 021cd4b0 021cd4fc 021cd568 021cd64c (cmd_queue), 021cc8a0 021cc8b0 021cc8b8 021cc8c0 (events: word table of 24 entries),
# 021bc2b0 (core1, from c0450), and 021bec08 (core2 smoke port, from c0d90). Everything else outside this area, and every core3
# function not yet in PORTED, is a stub (a stub of a core3 function becomes inactive the moment that address is in a PORTED
# list). Callees of a real callee are stubbed here (core1's 021bc2b0: 021b8b9c 021b9aec 021b8bc4 021b9b98; core2's 021bec08:
# 021d5440 021b8d08 021b8ff4). `bx pc` veneers are stubbed at their own address (the real targets are unloaded 0x0689xxxx code).
# Stub argument kinds follow the REGISTERS set at each call site, not Ghidra's prototypes (hidden pass-through of r0..r3 is
# common here: 021c6110 gets a 3rd argument from the 021d5c08 result in c0 .. bfe60, 021e899c / 021d6314 pass their result on).
# Every function that calls a cmd_queue function declares the region Q block (queue at work+0xC, write count near the 3000 cap
# is excluded, scratch words 021f3b44, the fatal 02034714); the tables passed to events fns are `arg buf 0x60` (24 words).
# Pointers that are only handed on to a stub are `p`; scalars the function does not dereference are `u32 any`.
# Loops over the mon list use `nullable:3:u32 seq` stubs on the veneers that advance the iterator (each address its own seq).
#
# Uninitialised stack: c0d90 reads one stack slot ([sp+0x1c], the bits above bit 1 of a flag word it passes on) that it never
# writes; the emulator zeroes the stack, so that value is 0 there (a port must use 0; in the cartridge it is garbage).
# 021c0a4c / 021c1e74 style "out" parameters that are also inputs (a u16 on the stack that 021c1678 patches) are modelled `out:2`
# (the stub overwrites it; the earlier 021c15d4 result is therefore not observable in the log).
#
# Skipped (not specced): 50 two-byte `bx pc` veneers into unloaded 0x0689xxxx code (each followed by a 6-byte ARM thunk
# `ldr ip,[pc]; bx ip` that Ghidra names thunk_EXT_FUN_0689xxxx: 021bf3cc 021c0ee8 021c0ef8 are halves of veneers 021bf3c8
# 021c0ee4 021c0ef4):
#   021bf3b8 021bf3c8 021bf46c 021bf47c 021bf48c 021bf49c 021bf5e8 021bf5f8 021bf608 021bf618 021bf628 021bf638 021bf648 021bf658
#   021bf668 021bf678 021bf688 021bf698 021bf6a8 021bf6b8 021bf6c8 021bf6d8 021bf894 021bf8a4 021bf8b4 021bf8c4 021bf944 021bf954
#   021bf964 021bf974 021bf984 021bf994 021bfe30 021bfe40 021bfe50 021c000c 021c0868 021c0878 021c0888 021c0a1c 021c0a2c 021c0a3c
#   021c0c04 021c0c14 021c0c24 021c0ec4 021c0ed4 021c0ee4 021c0ef4 021c0f04 021c0f14
# (their callers stub them at these addresses). 021c0c34 is a real 4-byte getter and is specced. The 26-entry switch table of
# 021c0520 (halfword offsets at 021c0546) and the literal pools are part of their functions, not functions.
# Smoke ports (B10h-spec, in core3.rs): see the PORTED list there.
area core3

# # true if either mon is hit-immune through the list object 021bf3b8 / 021bf3c8 and its BattleMon field 0x11 is 99, or the first has
# # flag 0x1c, or flag 0x1d with its stored id equal to the second mon's id (args: work, mon A, mon B)
fn 021bf324 ret=bool
  arg work
  arg u32 any
  arg u32 any
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bf3b8 ret=pick:0/1 args=p,v,v
  stub 021bf3c8 ret=pick:0/1 args=p,v,v
  stub 021d59b4 ret=pick:99/0/1/99/50 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62d8 ret=pick:0/1/2/3/4/5 args=v,v

# # fill the 24-word table (stack arg 0, events 021cc8a0 / 021cc8b0 real) with 6 and set table[id] = the result of 021c6e3c for each
# # mon of the list (r3, veneers); on result 0 the mon is removed from the list and reported
fn 021bf3d8 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x60
  stub 021bf46c ret=none args=v
  stub 021bf47c seq ret=nullable:3:u32 args=v
  stub 021bf49c seq ret=nullable:3:u32 args=v
  stub 021bf48c ret=none args=v,v
  stub 021c6e3c force ret=pick:0/1/2 args=ptr,v,v,ptr,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c7c24 force ret=none args=ptr,p,v,v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021ba1b4 force ret=none args=v,v

# # four passes over the list (events 0x2b / 0x2c / the ability pass with message 0x5b,0x20b / 0x2d): run 021bf6e8 for every mon
# # that is not excluded by 021bf324, remove the mons it handled (rec = u16 move id at +0, table = stack arg 0)
fn 021bf4ac ret=none inputs=400
  arg work
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg buf 0x60
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021bf5e8 ret=none args=v
  stub 021bf5f8 seq ret=nullable:3:u32 args=v
  stub 021bf608 ret=none args=v,v
  stub 021bf618 seq ret=nullable:3:u32 args=v
  stub 021bf628 ret=none args=v
  stub 021bf638 seq ret=nullable:3:u32 args=v
  stub 021bf648 ret=none args=v,v
  stub 021bf658 seq ret=nullable:3:u32 args=v
  stub 021bf668 ret=none args=v
  stub 021bf678 seq ret=nullable:3:u32 args=v
  stub 021bf688 ret=none args=v,v
  stub 021bf698 seq ret=nullable:3:u32 args=v
  stub 021bf6a8 ret=none args=v
  stub 021bf6b8 seq ret=nullable:3:u32 args=v
  stub 021bf6c8 ret=none args=v,v
  stub 021bf6d8 seq ret=nullable:3:u32 args=v
  stub 021bf324 ret=pick:0/1 args=ptr,v,v
  stub 021bf6e8 ret=pick:0/1 args=ptr,ptr,v,v,ptr,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021c6668 force ret=pick:0/1 args=p,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bf78c force ret=pick:0/1 args=ptr,ptr,v,v,v,ptr,p,outm:4:1/0/0/0
  stub 021c8828 force ret=pick:0/1 args=ptr
  stub 021c99a0 force ret=none args=ptr,ptr
  stub 021c8810 force ret=none args=ptr
  stub 021c88e4 force ret=pick:0/1 args=ptr
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021bf3b8 ret=pick:0/1 args=p,v,v
  stub 021bf3c8 ret=pick:0/1 args=p,v,v
  stub 021d59b4 ret=pick:99/0/1/99/50 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62d8 ret=pick:0/1/2/3/4/5 args=v,v

# # run one event (stack arg 1 = event id) on mon r3 inside a queue bracket (cc8d8 / cc95c on the header word at work+0x1D74)
fn 021bf6e8 ret=u32 inputs=400
  arg work
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg buf 0x60
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021bf78c force ret=pick:0/1 args=ptr,ptr,v,v,v,ptr,p,outm:4:1/0/0/0
  stub 021c8828 force ret=pick:0/1 args=ptr
  stub 021c99a0 force ret=none args=ptr,ptr
  stub 021c8810 force ret=none args=ptr
  stub 021c88e4 force ret=pick:0/1 args=ptr
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# # message builder for one event (id r2): arguments are the two mon ids, the move record at r1 (u16 +0, u8 +6, bit 0 of the word at
# # +0x10), the hem pointer (stack arg 2) and table[id of stack arg 0] (events 021cc8c0 real); result of 021d775c(0x40), and
# # *stack arg 3 = result of 021d775c(0x51)
fn 021bf78c ret=u32 inputs=300
  arg work
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x60
  arg u32 any
  arg buf 4
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c8810 force ret=none args=ptr
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# # one pass over the list r3: for every mon that is not excluded by 021bf324 and passes 021c6948 (u16 move id at r1+0), remove it and
# # apply 021c572c
fn 021bf83c ret=none
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  stub 021bf894 ret=none args=v
  stub 021bf8a4 seq ret=nullable:3:u32 args=v
  stub 021bf8b4 ret=none args=v,v
  stub 021bf8c4 seq ret=nullable:3:u32 args=v
  stub 021bf324 ret=pick:0/1 args=ptr,v,v
  stub 021c6948 force ret=pick:0/1 args=p,v,v
  stub 021c572c force ret=none args=ptr,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bf3b8 ret=pick:0/1 args=p,v,v
  stub 021bf3c8 ret=pick:0/1 args=p,v,v
  stub 021d59b4 ret=pick:99/0/1/99/50 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62d8 ret=pick:0/1/2/3/4/5 args=v,v

# # skipped when the list r3 has exactly one mon and it is the mon r2; else for every mon that fails both 021bf9a4 and 021bfa60,
# # remove it and apply 021c572c
fn 021bf8d4 ret=none
  arg work
  arg buf 0x10
  arg u32 oneof 0 1 2 3
  arg u32 any
  stub 021bf944 ret=pick:0/1/1/2 args=v
  stub 021bf954 ret=pick:0/1/2/3 args=v,v
  stub 021bf964 ret=none args=v
  stub 021bf974 seq ret=nullable:3:u32 args=v
  stub 021bf984 ret=none args=v,v
  stub 021bf994 seq ret=nullable:3:u32 args=v
  stub 021bf9a4 ret=pick:0/1 args=ptr,v,v,ptr
  stub 021bfa60 ret=pick:0/1 args=ptr,v,v,ptr
  stub 021c572c force ret=none args=ptr,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021bf324 ret=pick:0/1 args=ptr,v,v
  stub 021c69b8 force ret=pick:0/1 args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c6a04 force ret=u32 args=p,v,v,ptr
  stub 021d59b4 ret=u32 args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021d7704 ret=none args=v,v
  stub 021d7908 ret=u8 args=v,v
  stub 021d7b10 ret=u8 args=v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d78ac ret=mod:101 args=v
  stub 021bf3b8 ret=pick:0/1 args=p,v,v
  stub 021bf3c8 ret=pick:0/1 args=p,v,v
  stub 021d62d8 ret=pick:0/1/2/3/4/5 args=v,v

# # message builder 0x1c: mon ids at keys 3 / 4, u16 at r3+0 at key 0x12; returns the read-back of key 0x51
fn 021bf9a4 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# # only in mode 2 (021b8588 of work[4]): not when the item/ability r3 has flag 0xb; 1 when 021b9a6c of the two clients' values is 0
fn 021bfa00 ret=bool
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021b8588 force ret=pick:0/1/2/2/3 args=v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b9a6c force ret=pick:0/1 args=v,v

# # can the attack hit mon B: early 1s from 021bf324 / 021c69b8 / flag 0x20, else the accuracy builder 0x33 (keys 0x27 / 0x28 signed
# # stages, key 0x4b, 0x35 result) and a percent roll (021d78ac(100) < min(100, 021d7b10(...)))
fn 021bfa60 ret=u32 inputs=500
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021bf324 ret=pick:0/1 args=ptr,v,v
  stub 021c69b8 force ret=pick:0/1 args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c6a04 force ret=u32 args=p,v,v,ptr
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021d7704 ret=none args=v,v
  stub 021d7540 ret=none args=v
  stub 021d7908 ret=u8 args=v,v
  stub 021d7b10 ret=u8 args=v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d78ac ret=mod:101 args=v
  stub 021bf3b8 ret=pick:0/1 args=p,v,v
  stub 021bf3c8 ret=pick:0/1 args=p,v,v
  stub 021d62d8 ret=pick:0/1/2/3/4/5 args=v,v

# # rewrite a queued command 0x30 at position r3 (cd568 real) from the record r2 (u8 +2,+3,+4, u16 +0 or r1), then clear +4, set bit 1 of +5
fn 021bfbd0 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 alt near 2990 24 || range 0 2900 || range 0 2900 || range 0 2900
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg2 0 u16 alt const 0 || any

# # use an ability-triggered effect on mon r1 (returns 1..4)
fn 021bfc08 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  fix region:P 5 u8 any
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021c66ac force ret=pick:0/1 args=p,v,v
  stub 021c5718 force ret=none args=ptr
  stub 021c6720 force ret=pick:0/1 args=p,v,v
  stub 021bfd24 ret=pick:0/1 args=ptr,v,v,v,v
  stub 021e896c ret=u32 args=v,v
  stub 021c5c1c force ret=none args=ptr,v,v,v
  stub 021c6878 force ret=none args=p,v
  stub 021c68a8 force ret=pick:0/1 args=p,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021bfe60 ret=none args=ptr,v
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c676c force ret=pick:0/1 args=p,v,v,v,outm:1:0x1f,outm:4:1/0/0/0
  stub 021bfe30 ret=pick:0/1 args=v
  stub 021bfe40 ret=u32 args=v,v
  stub 021bfe50 ret=u32 args=p,v
  stub 021c6848 force ret=none args=p,v
  stub 021d5c30 ret=pick:0/1 args=v
  stub 021c5620 force ret=none args=ptr,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021d5c08 seq ret=pick:16/16/16/0/1/2 args=v
  stub 021c6110 force ret=none args=ptr,v,v

# # queue bracket around 021c676c (stack arg 0 = u16 move id; locals: a 5-bit target index starting 0x1f, a miss flag starting 0);
# # on success put 0x30 (r2, counter byte work+0x77F or the list result, id, 0) and 0x31 messages
fn 021bfd24 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u16 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c676c force ret=pick:0/1 args=p,v,v,v,outm:1:0x1f,outm:4:1/0/0/0
  stub 021bfe30 ret=pick:0/1 args=v
  stub 021bfe40 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bfe50 ret=u32 args=p,v
  stub 021c5718 force ret=none args=ptr
  stub 021c6848 force ret=none args=p,v
  stub 021d5c30 ret=pick:0/1 args=v

# # clear flag 0x1a, then run 021c6110 until the mon's state (021d5c08) is 0x10
fn 021bfe60 ret=none
  arg work
  arg u32 any
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c5620 force ret=none args=ptr,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021d5fc0 ret=none args=v,v
  stub 021d5c08 seq ret=pick:16/16/16/0/1/2 args=v
  stub 021c6110 force ret=none args=ptr,v,v

# # *r3 = 0; if mon r2 is not fainted / locked: messages and stat changes (returns 1); if flag 7: message 0x5b,0x20b and *r3 = 1
fn 021bfeac ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg buf 4
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021d5c30 ret=pick:0/1 args=v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c60ec force ret=none args=ptr,v,v
  stub 021c0780 ret=none args=ptr,v,v,v
  stub 021e899c ret=u32 args=v
  stub 021c2910 force ret=none args=ptr,v,v,v,v,v,v,v
  stub 021d69a0 ret=none args=v,v,v

# # end of a bind-like effect: look up the partner (021d69ac(mon, 4) - 1 < 24), clear its flag 0x21 and tell the clients
fn 021bff60 ret=none inputs=400
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d69ac ret=pick:0/1/2/9/24/25/26/255 args=v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c5620 force ret=none args=ptr,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021c6110 force ret=none args=ptr,v,v
  stub 021c000c ret=pick:0/1 args=p,v
  stub 021c5d88 force ret=none args=ptr,v,v
  stub 021c0780 ret=none args=ptr,v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d69a0 ret=none args=v,v,v

# # can mon r1 use the ability / effect r2 (r3 = skip the move-type test): returns 1 and prints the failure reason (021c0450) or
# # 0 (plus the success message / mode bookkeeping)
fn 021c001c ret=u32 inputs=1500
  arg work
  arg u32 any
  arg u32 alt const 0xa5 || oneof 1 2 3 4 5 || range 0 0xff || any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix work 0x1f78 u32 alt const 4 || const 2 || const 3 || range 0 6
  stub 021c03d8 ret=none args=ptr,v
  stub 021d62a8 ret=pick:1/1/2/3/0 args=v
  stub 021c0274 ret=pick:0/0/1 args=ptr,v,v,v
  stub 021c0404 ret=pick:0/0/1/1 args=ptr,v,v
  stub 021d5778 ret=pick:0/1/2/3/4 args=v,v
  stub 021d5584 ret=pick:0/0/0/1/5 args=v,v
  stub 021c02cc ret=pick:0/0/0/0/1 args=ptr,v,v,v
  stub 021d5bb0 ret=pick:0/0/0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=pick:0/1/2/3/4/5/0xa5 args=v   # Z23 (V-0024 class): 021e8b80 answers 0 for kinds 0/5/6/7
  stub 0201beb8 ret=pick:0/0/0/1 args=v,v
  stub 021ef948 ret=pick:0/1/1 args=v
  stub 021d67b8 ret=pick:0xa5/1/2/3/4 args=v
  stub 021d67a8 ret=pick:1/2/3/4/5 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021ef91c ret=pick:0/1/1 args=v,v,v
  stub 021c031c ret=pick:0/0/0/1 args=ptr,v
  stub 021d7af0 ret=pick:0/1 args=v
  stub 021c03ac ret=pick:0/1 args=ptr,v
  stub 021c0450 ret=none args=ptr,v,v,v
  stub 021c0520 ret=none args=ptr,v,v,v
  stub 021c5480 force ret=none args=ptr,v,v
  stub 021c557c force ret=none args=ptr,v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5604 force ret=none args=ptr,v,v
  stub 021d78ac ret=mod:4 args=v
  stub 021c54fc force ret=none args=ptr,v
  stub 021d6130 ret=pick:0/1 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021d61a8 ret=pick:0/1 args=v
  stub 021c5620 force ret=none args=ptr,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021c54c8 force ret=none args=ptr,v
  stub 021b8b9c force ret=pick:0/1 args=v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b8bc4 force ret=pick:0/1 args=v,v
  stub 021b9b98 force ret=pick:0/1 args=v
  stub 021c63d0 force ret=u16 args=ptr,v
  stub 021c4f64 force ret=pick:0/1 args=ptr,v
  stub 021d7d18 ret=u32 args=v
  stub 021c5c1c force ret=none args=ptr,v,v,v
  stub 021c6564 force ret=none args=v,v,v,v

# # not for 0xa5: if 021c02cc(work, mon, id, 0x1f) is nonzero print the failure (021c0450 with that reason) and return 1
fn 021c024c ret=bool
  arg work
  arg u32 any
  arg u32 alt const 0xa5 || any
  stub 021c02cc ret=pick:0/0/1 args=ptr,v,v,v
  stub 021c0450 ret=none args=ptr,v,v,v
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8b9c force ret=pick:0/1 args=v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b8bc4 force ret=pick:0/1 args=v,v
  stub 021b9b98 force ret=pick:0/1 args=v
  stub 021c63d0 force ret=u16 args=ptr,v
  stub 021c4f64 force ret=pick:0/1 args=ptr,v
  stub 021c0520 ret=none args=ptr,v,v,v
  stub 021d7d18 ret=u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5c1c force ret=none args=ptr,v,v,v
  stub 021c5480 force ret=none args=ptr,v,v
  stub 021c6564 force ret=none args=v,v,v,v
  stub 021c5604 force ret=none args=ptr,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d78ac ret=mod:4 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# # message builder 0x1d: key 2 = mon id, key 0x22 = r3, key 0x12 = r2; returns the read-back of key 0x41
fn 021c0274 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# # message builder r3: key 0x12 = r2, key 2 = mon id; returns the read-back of key 0x22
fn 021c02cc ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v

# # mode 3: message 0x5a,0xbc, return 1; else flag 6: clear it with message when 021d61a8 else roll 021d7af0(0x32)
fn 021c031c ret=u32
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix work 0x1f78 u32 alt const 3 || range 0 5
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d61a8 ret=pick:0/1 args=v
  stub 021c5620 force ret=none args=ptr,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021c54c8 force ret=none args=ptr,v
  stub 021d7af0 ret=pick:0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # flag 7 and a 50 % roll
fn 021c03ac ret=bool
  arg work
  arg u32 any
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c54fc force ret=none args=ptr,v
  stub 021d7af0 ret=pick:0/1 args=v

fn 021c03d8 ret=none
  arg work
  arg u32 any
  stub 021d62a8 ret=pick:0/1/2/3 args=v
  stub 021d6130 ret=pick:0/1 args=v
  stub 021c557c force ret=none args=ptr,v,v,v

fn 021c0404 ret=u32
  arg work
  arg u32 any
  arg u32 any
  stub 021d62a8 ret=pick:0/1/2/3 args=v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021d7af0 ret=pick:0/1 args=v
  stub 021c557c force ret=none args=ptr,v,v,v

# # failure reason r3 (5 = the party-wide case: 021c4f64 then core1's 021bc2b0 (real), 0x17 = sleep-like with 021d7d18(2)):
# # print message / queue bracket around 021c6564
fn 021c0450 ret=none inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 alt const 5 || oneof 0 1 2 3 4 0x17 || range 0 0x20
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8b9c force ret=pick:0/1 args=v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b8bc4 force ret=pick:0/1 args=v,v
  stub 021b9b98 force ret=pick:0/1 args=v
  stub 021c63d0 force ret=u16 args=ptr,v
  stub 021c4f64 force ret=pick:0/1 args=ptr,v
  stub 021c0520 ret=none args=ptr,v,v,v
  stub 021d7d18 ret=u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5c1c force ret=none args=ptr,v,v,v
  stub 021c5480 force ret=none args=ptr,v,v
  stub 021c6564 force ret=none args=v,v,v,v
  stub 021c5604 force ret=none args=ptr,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d78ac ret=mod:4 args=v

# # the failure-message table: case r3 (0..0x19, else the default 0x5a,0x47) puts a 0x5a / 0x5b text message (cd64c real) for the mon
fn 021c0520 ret=none inputs=500
  arg work
  arg u32 any
  arg u32 any
  arg u32 range 0 0x1c
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5604 force ret=none args=ptr,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d78ac ret=mod:4 args=v

# # put command 0x25 (mon id, r2 & 0xff, r3) and apply the change (021d69a0)
fn 021c0780 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d69a0 ret=none args=v,v,v

# # 021c07d4 (builder) then 021c0898 with its low byte
fn 021c07ac ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021c07d4 ret=u32 args=ptr,v,v,v,v
  stub 021c0898 ret=pick:0/1 args=ptr,v,v,v
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021c0868 ret=u32 args=v
  stub 021c0878 seq ret=nullable:3:u32 args=v,v
  stub 021c0888 seq ret=nullable:3:u32 args=v,v
  stub 021d5584 ret=pick:0/1/2/3/5 args=v,v
  stub 021c08cc ret=none args=ptr,v,v,v
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5608 ret=none args=v,v,v
  stub 021d56cc ret=none args=v,v

# # message builder 0x4e: count of the list (stack arg 0) at key 5, the first mons' ids at keys 6.., mon id at key 3, r3 at 0x12,
# # r2 at 0x17, key 0x20 = 1; returns the read-back of key 0x20
fn 021c07d4 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=u32 args=v
  stub 021d7540 ret=none args=v
  stub 021c0868 ret=u32 args=v
  stub 021c0878 seq ret=nullable:3:u32 args=v,v
  stub 021c0888 seq ret=nullable:3:u32 args=v,v

# # min(r3, 021d5584(mon, r2)) as a count; if nonzero apply 021c08cc and return 1
fn 021c0898 ret=bool
  arg work
  arg u32 any
  arg u32 any
  arg u32 alt range 0 5 || any
  stub 021d5584 ret=pick:0/1/2/3/5 args=v,v
  stub 021c08cc ret=none args=ptr,v,v,v
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5608 ret=none args=v,v,v
  stub 021d56cc ret=none args=v,v

# # unless 021b911c(work[4], 5): put command 4 (id, r2, r3), apply 021d5608, 021d56cc, put command 6 (id, r2)
fn 021c08cc ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5608 ret=none args=v,v,v
  stub 021d56cc ret=none args=v,v

# # build the damage record for one target (r1 = u16 move id at +0, r3 = list; stack args 0 / 1 = table, flag word)
fn 021c091c ret=none inputs=400
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg buf 0x60
  arg u32 any
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  fix region:P 5 u8 any
  stub 021c19d8 force ret=none args=ptr,v,ptr
  stub 021c0a1c ret=none args=v
  stub 021c71e0 force ret=pick:0/1 args=p,v,v,ptr
  stub 021c14e0 force ret=none args=ptr,v,p,v
  stub 021c0a2c ret=pick:0/1 args=v
  stub 021c1194 force ret=pick:0/1 args=v
  stub 021c0a3c ret=pick:0/1/2 args=v
  stub 021c0f24 force ret=u32 args=ptr,ptr,v,v,ptr
  stub 021c0d90 ret=u32 args=ptr,ptr,v,v,ptr,out:4,v
  stub 021c2048 force ret=none args=ptr,v,v,v
  stub 021c1a54 force ret=none args=ptr,ptr,v,v,v,v
  stub 021bfbd0 ret=none args=ptr,v,ptr,v
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  region R1 0x100
  fix work 0x86c u32 addrof region:R1
  region R2 0x100
  fix work 0x870 u32 addrof region:R2
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c0ec4 ret=pick:0/1/2 args=v
  stub 021c0ed4 ret=pick:0/1/2/3 args=v
  stub 021c0ee4 ret=none args=v,v,v
  stub 021c0ef4 ret=none args=v,v,v
  stub 021c0a4c ret=none args=ptr,v,v,ptr,ptr,v,v
  stub 021c0f04 ret=pick:0/1 args=v
  stub 021c0f14 ret=pick:0/1 args=v
  stub 021c11a4 force ret=u32 args=ptr,ptr,v,v,v,v,v,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021c0c04 ret=none args=v
  stub 021c0c14 seq ret=nullable:3:u32 args=v
  stub 021c0c24 seq ret=nullable:3:u32 args=v
  stub 021c6bf8 force ret=pick:0/1 args=ptr,v,v,v
  stub 021c1e74 force ret=pick:0/1 args=ptr,v,v,ptr,v,v,v,v,out:2
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c15d4 force ret=u16 args=v,v
  stub 021c1678 force ret=u32 args=ptr,v,v,v,out:2
  stub 021c7c24 force ret=none args=ptr,p,v,v,v

# # fill the damage record at stack arg 2 for every mon of the list r2: {+0 hit count, +1 miss count, +2 total, entries of
# # {u16 +4+4i, u16 +6+4i bit fields: 0-4 mon id, 5-8 table value, 9-12 roll, 13 flag, 14 flag, 15 flag}}
fn 021c0a4c ret=none inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x60
  arg u32 any
  arg buf 0x100
  stub 021c0c04 ret=none args=v
  stub 021c0c14 seq ret=nullable:3:u32 args=v
  stub 021c0c24 seq ret=nullable:3:u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c6bf8 force ret=pick:0/1 args=ptr,v,v,v
  stub 021c1e74 force ret=pick:0/1 args=ptr,v,v,ptr,v,v,v,v,out:2
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c15d4 force ret=u16 args=v,v
  stub 021c1678 force ret=u32 args=ptr,v,v,v,out:2
  stub 021c7c24 force ret=none args=ptr,p,v,v,v

# # getter: byte at +2 (the entry count of a damage record)
fn 021c0c34 ret=u8
  arg buf 0x10

# # copy out the entries of a damage record whose bit 15 is set: mon pointer (021b9934 of work[8], id), u16, table value, flag
fn 021c0c38 ret=u32
  arg work
  arg buf 0x40
  arg buf 0x40
  arg buf 0x20
  arg buf 0x40
  arg buf 0x10
  fix arg1 2 u8 range 0 12
  stub 021b9934 force ret=u32 args=v,v

# # the mon pointers of every entry of a damage record; returns the count
fn 021c0cb4 ret=u8
  arg work
  arg buf 0x40
  arg buf 0x40
  fix arg1 2 u8 range 0 12
  stub 021b9934 force ret=u32 args=v,v

# # like 021c0c38 but for the entries whose bit 15 is clear, with one more output byte array
fn 021c0ce8 ret=u32
  arg work
  arg buf 0x40
  arg buf 0x40
  arg buf 0x20
  arg buf 0x40
  arg buf 0x10
  arg buf 0x10
  fix arg1 2 u8 range 0 12
  stub 021b9934 force ret=u32 args=v,v

# # sum of the u16 values of a damage record
fn 021c0d70 ret=u32
  arg buf 0x40
  fix arg0 2 u8 range 0 12

# # build the two damage records (021c0a4c, work+0x858 and +0x85c) for the list r3 and run 021c11a4 on each; *stack arg 1 = the
# # 021cd4fc(queue, 0x30) result; returns the sum of the two 021c11a4 results
fn 021c0d90 ret=u32 inputs=400
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg buf 0x60
  arg buf 4
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  region R1 0x100
  fix work 0x86c u32 addrof region:R1
  region R2 0x100
  fix work 0x870 u32 addrof region:R2
  stub 021c0ec4 ret=pick:0/1/2 args=v
  stub 021c0ed4 ret=pick:0/1/2/3 args=v
  stub 021c0ee4 ret=none args=v,v,v
  stub 021c0ef4 ret=none args=v,v,v
  stub 021c0a4c ret=none args=ptr,v,v,ptr,ptr,v,v
  stub 021c0f04 ret=pick:0/1 args=v
  stub 021c0f14 ret=pick:0/1 args=v
  stub 021c11a4 force ret=u32 args=ptr,ptr,v,v,v,v,v,v
  stub 021ba1b4 force ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021c0c04 ret=none args=v
  stub 021c0c14 seq ret=nullable:3:u32 args=v
  stub 021c0c24 seq ret=nullable:3:u32 args=v
  stub 021c6bf8 force ret=pick:0/1 args=ptr,v,v,v
  stub 021c1e74 force ret=pick:0/1 args=ptr,v,v,ptr,v,v,v,v,out:2
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c15d4 force ret=u16 args=v,v
  stub 021c1678 force ret=u32 args=ptr,v,v,v,out:2
  stub 021c7c24 force ret=none args=ptr,p,v,v,v
