# fndiff spec: shinka_check.c (P5): the evolution check and its helpers, main ARM9 code
# (0x0201B2B0-0x0201B9F0; T3-FINDINGS.md (a), "ARC 019 evolution").
# Run: tools/check-fn.sh black-pokemon evolution --per-function
#
# Function map (rizin on the field ELF; Ghidra splits FUN_0201B2B0 into pieces and shows 0x0201B3B8 /
# 0x0201B406 as functions, but they are the record-walk loop head and a case label INSIDE 0x0201B2B0, so
# they have no entry of their own here):
#   0201B2B0 (ctx, mon, mode, param, tod, outp, heap): the evolution check. mode 0 = level-up family
#            (29 method cases at 0x0201B3B8, jump table 0x0201B3D4), 1 = trade (cases 5/6/7: param = partner
#            mon), 2 and 3 = item use (cases 8/17/18; mode 3 skips the held-item block), > 3 = nothing.
#   0201B70C (species) alloc-load ARC 19; 0201B718 (handle, member, idx, kind) one field of a record read
#   through FUN_0204932C; 0201B764 (species, dst) ARC 19 load; 0201B774 (mon, move) knows-move test.
#   0201B7A0 / B7B4 / B7EC / B818 / B850: friendship adjustment (B850 does the work; the others pick a delta table).
#
# Boundary model. The mon records are really encrypted Pokemon (same recipe as poke_tool.spec: PID with
# shuffle row 0 mostly, checksum, ciphered blocks and party extension), and the Pokemon getters
# FUN_02017E1C / FUN_02017E88 / FUN_02017E40 (ported in poke_tool.rs) run as real code. Only their
# own unported callee, the gender routine FUN_02017F50, is a stub (`pick:0/1/2`).
# The ARC 19 load FUN_02048F4C (ArcLoadMember) is stubbed with a pointer-only argument and writes nothing:
# the 42-byte heap block (7 records of u16 method, u16 param, u16 target) is planted by the input
# generator instead (the growth.spec trick), so record parameters can be drawn from the same small
# value sets as the mon's fields (items 80/81, moves 33/45, levels) and the equality cases of the
# method switch are reached. The allocator FUN_02030734 returns that block (`ptr:arg7`), the held-item
# effect FUN_02020FE4 (`pick:0x40/...`: 0x40 is the evolution-blocking effect), the day/night test
# FUN_02011410 and the party species test FUN_0201AC44 are stubs.
# Item values {80, 81}, move values {33, 45}, location table entries {195, 196, 197, 155, 203}.
area evolution
set field

# FUN_0201B2B0(ctx, mon, mode, param, tod, outp, heap) -> target species (0 = none). Stack args: tod (u8
# for FUN_02011410), outp (0 = use a local; else a u32 out word that gets the winning method, or 15 if the
# Shedinja-style case 15 was met), heap id. Arg 7 is the 0x40-byte block FUN_02030734 hands out (arg 7 of
# the harness; the function ignores that stack word) and arg 8 stands for the high halfword of the mon's PID, which the
# level-up cases 12/13 split on ((PID >> 16) % 10 < 5: remainder 5 needs an odd high half, i.e. shuffle row 8, where
# the planted row-0 fields are not in place; those cases only need level, record and PID). param: for mode 1 the
# partner mon (region), else a location id / item id.
fn 0201b2b0 ret=u32 inputs=8000
  nowork
  region mon 0xDC
  region partner 0xDC
  region out 4
  arg u32 alt const 0 || any
  arg region mon
  arg u32 alt range 0 3 || range 0 3 || range 0 3 || range 0 3 || oneof 4 0xFFFFFFFF
  arg u32 ifeq 2 1 addrof region:partner || alt oneof 195 196 197 155 203 || oneof 155 203 || oneof 80 81 || oneof 80 81 || oneof 80 81 || range 0 300 || any
  arg u8 any
  arg u32 alt const 0 || addrof region:out || addrof region:out
  arg u32 any
  arg buf 0x40
  arg u32 alt oneof 0 4 6 16 24 36 44 56 || oneof 5 25 45 65 || oneof 4 5 6 14 15 16 || any
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 2 u16 rel 8 const 0
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 64 || const 588 || const 616 || const 588 || const 616 || range 1 649 || const 0x28a
  fix region:mon 0x0a u16 alt oneof 80 81 || oneof 80 81 || oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x14 u8 alt range 200 255 || range 219 221 || range 0 255
  fix region:mon 0x1f u8 alt range 100 255 || range 0 255
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x18 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x92 u16 alt range 0 3 || range 0 3 || range 0 400
  fix region:mon 0x94 u16 alt range 0 3 || range 0 3 || range 0 400
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  fix region:partner 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:partner 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 3 || const 1 || const 4 || any
  fix region:partner 0x08 u16 alt const 588 || const 616 || const 616 || range 1 649
  fix region:partner 0x0a u16 alt oneof 80 81 || oneof 80 81 || oneof 80 81 || range 0 0x290 || any
  fix region:partner 0x14 u8 alt range 200 255 || range 219 221 || range 0 255
  fix region:partner 0x1f u8 alt range 100 255 || range 0 255
  fix region:partner 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:partner 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:partner 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:partner 8 0x80 u16@6 skipif=u16@4&2
  fix region:partner 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:partner 0x92 u16 alt range 0 3 || range 0 3 || range 0 400
  fix region:partner 0x94 u16 alt range 0 3 || range 0 3 || range 0 400
  crypt region:partner 0x88 0x54 u32@0 skipif=u16@4&1
  fix arg7 0 u16 alt range 0 28 || range 0 28 || range 0 28 || range 0 40 || oneof 6 7 8 17 18 19 20 21 22 25 26 27 28 rep=7,6
  fix arg7 2 u16 alt range 1 100 || oneof 33 45 80 81 || oneof 80 81 || oneof 80 81 || oneof 33 45 || range 100 255 || near 220 2 rep=7,6
  fix arg7 4 u16 alt range 1 649 || range 1 649 || range 1 649 || const 0 rep=7,6
  stub 02020fe4 ret=pick:0x40/0/1/2/3/4/5/6 args=v,v,v
  stub 02011410 ret=pick:0/1 args=v
  stub 0201ac44 ret=pick:0/1 args=v,v
  stub 02030734 ret=ptr:arg7:0:1 args=v,v,v,v,v
  stub 020307b0 ret=none args=v
  stub 02048f4c ret=none args=v,v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_0201B70C(species): alloc-load of ARC 19 member `species`: tail call FUN_020490F4(0x13, species).
fn 0201b70c ret=u32
  nowork
  arg u32 alt range 0 649 || any
  stub 020490f4 ret=u32 args=v,v

# FUN_0201B718(handle, member, idx, kind): read the 42-byte member through FUN_0204932C into a local,
# return the u16 method (kind 0) / param (1) / target (2) of record idx, else 0.
fn 0201b718 ret=u32
  nowork
  arg u32 any
  arg u32 alt range 0 649 || any
  arg u32 range 0 6
  arg u32 alt range 0 2 || range 0 2 || oneof 3 4 0xFFFFFFFF
  stub 0204932c ret=none args=v,v,out:42

# FUN_0201B764(species, dst): ArcLoadMember(dst, 0x13, species) (tail call, arguments reordered).
fn 0201b764 ret=none
  nowork
  arg u32 alt range 0 649 || any
  arg u32 any
  stub 02048f4c ret=none args=v,v,v

# FUN_0201B774(mon, move): 1 if one of the four move slots (ids 0x36..0x39) equals `move`.
fn 0201b774 ret=u32 inputs=800
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt oneof 33 45 || oneof 33 45 || const 0 || range 0 560 || any
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 64 || const 588 || const 616 || const 588 || const 616 || range 1 649 || const 0x28a
  fix region:mon 0x0a u16 alt oneof 80 81 || oneof 80 81 || oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x14 u8 alt range 200 255 || range 219 221 || range 0 255
  fix region:mon 0x1f u8 alt range 100 255 || range 0 255
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x92 u16 alt range 0 3 || range 0 3 || range 0 400
  fix region:mon 0x94 u16 alt range 0 3 || range 0 3 || range 0 400
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_0201B850(mon, table, location, heap): friendship adjustment. table = 3 signed bytes (delta for friendship
# < 100, < 200, >= 200). Deltas get +1 for the ball id 11 (id 0x98), +1 when the mon was met at `location`
# (id 0x96, FUN_02013E4C(location) != 0), x150/100 with the held effect 0x35; then the result is passed through
# FUN_02011014 (event bonus), added to friendship (id 9) and clamped to 0..255. Mons of species 0 / 0x28A are skipped.
# The table is the argument `table` (a buffer of 3 bytes); ball id 11 is at D+0x1B (0x83), the met location at D+0x18 (0x80).
fn 0201b850 ret=none inputs=1500
  nowork
  region mon 0xDC
  arg region mon
  arg buf 0x10
  arg u32 alt const 0 || range 0 30 || any
  arg u32 any
  fix arg1 0 u8 alt range 0 5 || range 0 10 || range 0 0xFF || const 0x80 || const 0xFE rep=3,1
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 64 || const 588 || const 616 || const 588 || const 616 || range 1 649 || const 0x28a
  fix region:mon 0x0a u16 alt oneof 80 81 || oneof 80 81 || oneof 80 81 || range 0 0x290 || any
  fix region:mon 0x14 u8 alt range 200 255 || range 219 221 || range 0 255
  fix region:mon 0x1f u8 alt range 100 255 || range 0 255
  fix region:mon 0x28 u16 alt oneof 33 45 || oneof 33 45 || const 0 || range 1 560 rep=4,2
  fix region:mon 0x38 u32 alt const 0 || const 0 || const 0 || const 1 bits=30:1
  fix region:mon 0x83 u8 alt const 11 || range 0 20 || any
  fix region:mon 0x80 u16 alt const 0 || const 1 || const 2 || const 30 || any
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || oneof 33 45 80 81 || range 1 100
  fix region:mon 0x92 u16 alt range 0 3 || range 0 3 || range 0 400
  fix region:mon 0x94 u16 alt range 0 3 || range 0 3 || range 0 400
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  stub 02013e4c ret=pick:0/1/2/30 args=v
  stub 02020fe4 ret=pick:0x35/0/1/2/3 args=v,v,v
  stub 02011014 ret=pick:0/1/2/0xFFFFFFFE/0xFFFFFFF0 args=v

# FUN_0201B7A0(mon, idx, location, heap): B850 with the delta table 0x0209E45C + 3 * idx.
fn 0201b7a0 ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 5 || any
  arg u32 any
  arg u32 any
  stub 0201b850 force ret=none args=v,v,v,v

# FUN_0201B7B4(mon, item, location, heap): the three deltas come from the item data (FUN_02020FE4(item, 0x3C + i, heap),
# i = 0..2, kept as bytes), then B850(mon, those 3 bytes, location, heap).
fn 0201b7b4 ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 626 || any
  arg u32 any
  arg u32 any
  stub 02020fe4 ret=u32 args=v,v,v
  stub 0201b850 force ret=none args=v,in:3,v,v

# FUN_0201B7EC(mon, location, heap): with probability 1/2 (top bit of FUN_0203F0A8 clear) B850 with the table 0x0209E465.
fn 0201b7ec ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0203f0a8 ret=u32 args=
  stub 0201b850 force ret=none args=v,v,v,v

# FUN_0201B818(party, location, heap): B850 with the table 0x0209E462 on every party member (count = party[1]).
fn 0201b818 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0201aa18 ret=mod:7 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 0201b850 force ret=none args=v,v,v,v
