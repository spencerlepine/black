# fndiff spec: waza_tool.c (P4): move data access, the learnset reader and the move-learning helpers of
# poke_tool.c, main ARM9 code (0x0201BD28-0x0201C408, 0x0201ADD0, 0x02017FB0-0x020182C0, 0x0201B920-0x0201BA68).
# Run: tools/check-fn.sh black-pokemon waza --per-function
#
# Move records (ARC 21, 36 bytes) are loaded by ArcLoadMember (FUN_02048F4C(dst, 0x15, move)) into a small
# cache the game builds with FUN_0201C328 (global 0x02146530 -> struct { u32 *bufs; u16 *ids; int count },
# `count` records of 0x24 bytes; the slot is the first with the move's id or the first free one (id 0), and
# when all are taken with other ids the LAST slot is reused) or, without a cache (global 0 -> static record
# at 0x02146534), loaded in place on every call. The cache users below set that up with `region`s: cache
# (the struct), bufs (the u32 pointer array), ids (the u16 array), rec0..rec2 (the records), planted with
# `addrof` pointers and a count of 1..3. The ARC load is a stub with `out:36`: the loaded record is
# mix()-derived bytes, which exercises every byte of the getter. FUN_0201BDB4 (the record getter, 32 params)
# and FUN_0201BF44 (flag test) are pure functions of a record region.
#
# Move getter (BDB4) parameters, from the jump table at 0x0201BDC4 (rizin; Ghidra mis-decompiles it, A7d):
#   0..3 bytes 0..3; 4 byte 4 (101 -> 100); 5 byte 5; 6 signed byte 6; 7 byte 0xE (6 -> 0); 8/9 hi/lo nibble of
#   byte 7; 10 byte 0xF; 11 u16 at 8; 12..15 bytes 0xA..0xD; 16/19/22 byte 0x15 + (p-16); 17/20/23 signed byte
#   0x18 + (p-17); 18/21/24 byte 0x1B + (p-18); 25/26 max(0, signed byte 0x12/0x13); 27 byte 0x14; 28 u16 at 0x10;
#   29 0; 30/31 -(signed byte 0x12/0x13) if negative else 0; > 31: 0.
area waza
set field

# FUN_0201BD28(move, param): the move record of `move` through the cache (or loaded in place), then
# FUN_0201BDB4(record, param). Only the slot search, the ARC load and the getter are in this function.
fn 0201bd28 ret=u32 inputs=1500
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  arg u32 alt range 0 31 || range 0 31 || range 0 31 || oneof 32 33 0xFFFF 0x80000000 0xFFFFFFFF
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  stub 02048f4c ret=none args=out:36,v,v

# FUN_0201BDB4(rec, param): the record getter (see the table above). The record is a 36-byte region.
fn 0201bdb4 ret=u32 inputs=1200
  nowork
  region rec 0x24
  arg region rec
  arg u32 alt range 0 31 || range 0 31 || range 0 31 || oneof 32 33 0xFFFF 0x80000000 0xFFFFFFFF
  fix region:rec 4 u8 alt const 0x65 || const 0x64 || any
  fix region:rec 0xe u8 alt const 6 || const 5 || any
  fix region:rec 6 u8 alt const 0x80 || const 0x7f || any
  fix region:rec 0x12 u8 alt const 0x80 || const 0 || const 1 || const 0xFF || any
  fix region:rec 0x13 u8 alt const 0x80 || const 0 || const 1 || const 0xFF || any
  fix region:rec 0x18 u8 alt const 0x80 || const 0 || const 0x7f || any rep=3,1

# FUN_0201BF44(rec, bit): 1 if bit `bit` (the LOW BYTE of r1, shifted as a register shift: 32..255 give 0) of the
# flag word at rec+0x20 is set.
fn 0201bf44 ret=u32 inputs=600
  nowork
  region rec 0x24
  arg region rec
  arg u32 alt range 0 31 || range 0 31 || oneof 32 33 63 64 255 256 257 0x120 0xFFFFFFFF
  fix region:rec 0x20 u32 alt const 0 || const 0xFFFFFFFF || any

# FUN_0201C158(move, pp_ups): max PP = base + base * 20 * min(ups, 3) / 100 (all in u8), base = BD28(move, 5).
fn 0201c158 ret=u32 inputs=600
  nowork
  arg u16 any
  arg u32 alt range 0 3 || range 0 3 || range 0 8 || any
  stub 0201bd28 force ret=mod:120 args=v,v

# FUN_0201ADD0(species, form, dst): the learnset reader: ArcLoadMember(dst, 0x12, FUN_0201AFD4(species, form)).
# FUN_0201AFD4 (area personal) is a stub here; the learnset (ARC 18, (u16 move, u16 level) pairs ending in
# 0xFFFF 0xFFFF) lands in `dst`, which only the callee's side effects touch.
fn 0201add0 ret=none
  nowork
  arg u16 alt range 0 649 || any
  arg u32 alt range 0 5 || any
  arg u32 any
  stub 0201afd4 ret=u32 args=v,v
  stub 02048f4c ret=none args=v,v,v

# --- cache users (spec only): the same slot search as FUN_0201BD28 followed by a small accessor ---

# FUN_0201BEB8(move, bit): record of `move`, then FUN_0201BF44(record, bit) (flag test).
fn 0201beb8 ret=u32 inputs=800
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  arg u32 alt range 0 31 || oneof 32 255 256 0xFFFFFFFF
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  stub 02048f4c ret=none args=out:36,v,v

# FUN_0201BF58(move, a1, a2, a3): a 16-bit value from the record's bytes 0xB..0xD (4 + 6 + 6 bits) when byte
# 0xB is nonzero, returned sign-extended; else the LOW HALFWORD OF THE CALLER'S r3 (the function's first
# push {r3, ...} slot is read back uninitialised: an original quirk), hence the three extra arguments.
fn 0201bf58 ret=u32 inputs=800
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  arg u32 any
  arg u32 any
  arg u32 any
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  stub 02048f4c ret=none args=out:36,v,v

# FUN_0201C020(move): the number of nonzero bytes among bytes 0x15..0x17 (u8).
fn 0201c020 ret=u32 inputs=600
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  fix region:rec0 0x15 u8 alt const 0 || any rep=3,1
  fix region:rec1 0x15 u8 alt const 0 || any rep=3,1
  fix region:rec2 0x15 u8 alt const 0 || any rep=3,1
  stub 02048f4c ret=none args=out:36,v,v

# FUN_0201C1F4(move): 1 if byte 4 of the record is 101, else 0.
fn 0201c1f4 ret=u32 inputs=600
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  fix region:rec0 4 u8 alt const 0x65 || any
  fix region:rec1 4 u8 alt const 0x65 || any
  fix region:rec2 4 u8 alt const 0x65 || any
  stub 02048f4c ret=none args=out:36,v,v

# FUN_0201C284(move): 1 if byte 0xE of the record is 6, else 0.
fn 0201c284 ret=u32 inputs=600
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  fix region:rec0 0xe u8 alt const 6 || any
  fix region:rec1 0xe u8 alt const 6 || any
  fix region:rec2 0xe u8 alt const 6 || any
  stub 02048f4c ret=none args=out:36,v,v

# FUN_0201C0BC(move, idx, out): *out = signed byte 0x18 + idx of the record if byte 0x15 + idx is nonzero, else 0.
fn 0201c0bc ret=none inputs=800
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  arg u16 alt range 0 560 || range 1 560 || any
  arg u32 alt range 0 2 || range 0 10
  arg buf 4
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || rel 0 const 0 || rel 0 const 0 || range 1 560 rep=3,2
  stub 02048f4c ret=none args=out:36,v,v

# Thin wrappers around FUN_0201BD28(move, param): C1BC param 1 (tail call), C1C8 param 0 (u8), C1D8 param 3 (u16),
# C1E8 param 2 (tail call), C314 (param 3 != 0 -> 1). BD28 itself is stubbed (`force`); its own spec checks it.
fn 0201c1bc ret=u32
  nowork
  arg u16 alt range 0 560 || range 1 560 || any
  stub 0201bd28 force ret=u32 args=v,v
fn 0201c1c8 ret=u32
  nowork
  arg u16 alt range 0 560 || range 1 560 || any
  stub 0201bd28 force ret=u32 args=v,v
fn 0201c1d8 ret=u32
  nowork
  arg u16 alt range 0 560 || range 1 560 || any
  stub 0201bd28 force ret=u32 args=v,v
fn 0201c1e8 ret=u32
  nowork
  arg u16 alt range 0 560 || range 1 560 || any
  stub 0201bd28 force ret=u32 args=v,v
fn 0201c314 ret=u32
  nowork
  arg u16 alt range 0 560 || range 1 560 || any
  stub 0201bd28 force ret=mod:3 args=v,v

# FUN_0201C184(move): 2 for 240, 1 for 241, 4 for 201, 3 for 258, else 0 (signed compares).
fn 0201c184 ret=u32 inputs=300
  nowork
  arg s32 alt oneof 240 241 201 258 || range 0 300 || any

# FUN_0201C328(n, heap), the cache constructor, is NOT specified: it makes four allocations of different sizes from
# FUN_02030734 (S, bufs, ids, n records) and stores pointers into the blocks; the stub model can hand out only
# addresses on a fixed grid that cannot be made disjoint, so aliased blocks corrupt each other's pointers (a
# fault in the ROM run). The cache layout it builds is what every cache user below plants.

# FUN_0201C3C0(): free the records, the arrays and S, and clear the global.
fn 0201c3c0 ret=none inputs=300
  nowork
  global 02146530 4
  global 02146534 0x24
  region cache 12
  region bufs 12
  region ids 8
  region rec0 0x24
  region rec1 0x24
  region rec2 0x24
  fix global0 0 u32 alt addrof region:cache || addrof region:cache || const 0
  fix region:cache 0 u32 addrof region:bufs
  fix region:cache 4 u32 addrof region:ids
  fix region:cache 8 u32 range 1 3
  fix region:bufs 0 u32 addrof region:rec0
  fix region:bufs 4 u32 addrof region:rec1
  fix region:bufs 8 u32 addrof region:rec2
  fix region:ids 0 u16 alt const 0 || const 0 || const 0 || range 1 560 rep=3,2
  stub 020307b0 ret=none args=v

# FUN_0201C408(): 1 if the cache exists (global nonzero), else 0.
fn 0201c408 ret=u32 inputs=100
  nowork
  global 02146530 4
  fix global0 0 u32 alt const 0 || any

# --- move slot helpers of poke_tool.c / shinka_check.c (spec only). The Pokemon getters/setters are real code ---

# FUN_0201B920(mon, idx): 1 if the slot's PP (id 0x3A+idx) is below the maximum FUN_0201C158(move, PP ups).
fn 0201b920 ret=u32 inputs=800
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 3 || range 0 3 || range 0 4
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_0201B970(mon, idx, add): raise the PP of slot idx by `add` (0x7F = to the maximum) up to FUN_0201C158(...);
# 0 if the slot is empty or already full, else 1.
fn 0201b970 ret=u32 inputs=1000
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 3 || range 0 3 || range 0 4
  arg u32 alt const 0x7F || range 0 60 || any
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_0201B9F0(mon): fully restore: copy the current-HP field (id 0xA1) into id 0xA0, clear id 0x9D, and top up
# the PP of every slot whose PP is below the maximum (B920 + B970 as real code).
fn 0201b9f0 ret=none inputs=600
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_0201BA34(party): FUN_0201B9F0 on every member (count at +4, members at +8, stride 0xDC) whose id 0xA9 is nonzero.
fn 0201ba34 ret=none inputs=300
  nowork
  region party 0x29C
  arg region party
  fix region:party 4 u32 range 0 3
  fix region:party 0x8 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:party 0xc u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:party 0x10 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:party 0x12 u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:party 0x30 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:party 0x38 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:party 0x3c u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:party 0x40 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:party 0xe u16 alt sum16 0x10 0x80 || sum16 0x10 0x80 || sum16 0x10 0x80 || any
  crypt region:party 0x10 0x80 u16@0xe skipif=u16@0xc&2
  fix region:party 0x94 u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:party 0x98 u16 alt range 0 400 || range 0 400 || any
  crypt region:party 0x90 0x54 u32@0x8 skipif=u16@0xc&1
  fix region:party 0xe4 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:party 0xe8 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:party 0xec u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:party 0xee u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:party 0x10c u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:party 0x114 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:party 0x118 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:party 0x11c u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:party 0xea u16 alt sum16 0xec 0x80 || sum16 0xec 0x80 || sum16 0xec 0x80 || any
  crypt region:party 0xec 0x80 u16@0xea skipif=u16@0xe8&2
  fix region:party 0x170 u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:party 0x174 u16 alt range 0 400 || range 0 400 || any
  crypt region:party 0x16c 0x54 u32@0xe4 skipif=u16@0xe8&1
  fix region:party 0x1c0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:party 0x1c4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:party 0x1c8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:party 0x1ca u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:party 0x1e8 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:party 0x1f0 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:party 0x1f4 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:party 0x1f8 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:party 0x1c6 u16 alt sum16 0x1c8 0x80 || sum16 0x1c8 0x80 || sum16 0x1c8 0x80 || any
  crypt region:party 0x1c8 0x80 u16@0x1c6 skipif=u16@0x1c4&2
  fix region:party 0x24c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:party 0x250 u16 alt range 0 400 || range 0 400 || any
  crypt region:party 0x248 0x54 u32@0x1c0 skipif=u16@0x1c4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# --- move learning (spec only): poke_tool.c 0x02017FB0-0x020182C0 ---
# FUN_02017FB0(mon): forget all four moves, then learn every move of the species' learnset (ARC 18, stubbed: the
# global 0x02146330 holds the planted (u16 move, u16 level) pairs, ending at 0xFFFF 0xFFFF) up to the mon's
# level (FUN_02018438 -> the growth routine FUN_0201850C, stubbed) through FUN_02018094 / FUN_020180FC.
fn 02017fb0 ret=none inputs=600
  nowork
  global 02146330 0x100
  region mon 0xDC
  arg region mon
  fix global0 0 u16 alt range 1 560 || range 1 560 || const 0 rep=64,2
  fix global0 2 u16 alt range 1 100 || range 1 30 || range 1 100 rep=32,4
  fix global0 0x1c u32 alt const 0xFFFFFFFF || any
  fix global0 0x3c u32 alt const 0xFFFFFFFF || any
  fix global0 0x7c u32 alt const 0xFFFFFFFF || any
  fix global0 0xfc u32 const 0xFFFFFFFF
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201850c ret=mod:101 args=v,v,v
  stub 0201add0 force ret=none args=v,v,v
  stub 0201bd28 force ret=u32 args=v,v
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_02018094(mon, move): 0xFFFF after learning `move` into the first empty slot (FUN_020181FC), 0xFFFE if it
# is already known, else 0xFFFF (all four slots full: the caller then shifts them).
fn 02018094 ret=u32 inputs=800
  nowork
  region mon 0xDC
  arg region mon
  arg u16 alt oneof 33 45 || range 1 560 || any
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201bd28 force ret=u32 args=v,v
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_020180FC(mon, move): forget the first move and append `move` as the 4th, shifting the other three (their
# PP and PP-up counts move with them); the new move gets FUN_0201BD28(move, 5) PP and 0 PP ups.
fn 020180fc ret=none inputs=600
  nowork
  region mon 0xDC
  arg region mon
  arg u16 alt oneof 33 45 || range 1 560 || any
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201bd28 force ret=u32 args=v,v
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_020181A0(mon): set every nonempty slot's PP to FUN_0201C158(move, PP ups).
fn 020181a0 ret=none inputs=600
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_020181FC(mon, move, slot): put `move` into `slot` with 0 PP ups and max PP (FUN_0201C158(move, 0)); 0 PP for move 0.
fn 020181fc ret=none inputs=600
  nowork
  region mon 0xDC
  arg region mon
  arg u16 alt oneof 33 45 0 || range 1 560 || any
  arg u32 alt range 0 3 || range 0 3 || range 0 4
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_02018240(mon, move): FUN_02018094 with FUN_020182A8 as the slot writer (keeps the old PP if smaller).
fn 02018240 ret=u32 inputs=800
  nowork
  region mon 0xDC
  arg region mon
  arg u16 alt oneof 33 45 || range 1 560 || any
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_020182A8(mon, move, slot): put `move` into `slot`, 0 PP ups, PP = min(old PP, max PP) when the slot held a move.
fn 020182a8 ret=none inputs=600
  nowork
  region mon 0xDC
  arg region mon
  arg u16 alt oneof 33 45 0 || range 1 560 || any
  arg u32 alt range 0 3 || range 0 3 || range 0 4
  fix region:mon 0x0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 649 || range 1 649 || const 0x28a
  fix region:mon 0xa u16 alt oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x30 u8 alt range 0 40 || range 0 5 || const 0 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || any
  crypt region:mon 0x88 0x54 u32@0x0 skipif=u16@0x4&1
  stub 0201c158 force ret=mod:64 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v
