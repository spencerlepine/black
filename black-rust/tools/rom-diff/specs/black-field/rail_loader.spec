# fndiff spec (F3b): `field_rail_loader.c` (overlay 21 Thumb code 0x0219EF14..0x0219F030; F1 row 21.12 head). Run:
# tools/check-fn.sh black-field rail_loader --per-function
#
# File bound: `Heap_AllocDebug(heap, 0x38, 1, "field_rail_loader.c", 0xC0)` in 0219EF44 (string 0x021DDE08). F1 put the row at
# 0x0219EF44..0x021A0100 (94 functions): 0219EF14 (the clear, F1 row 21.11) belongs here (only 0219EF44 / 0219EFFC call it), and
# 0x0219F030..0x021A0100 are MMDL billboard-actor draw procs (the proc table ROM 0x021D8558, the blact calls 0219C9DC / 02050848 /
# 02050880): render code (D3, U2), not specced here.
#
# RAIL LOADER (0x38): +0 the loaded rail file (ARC 0x4F member, `0204A298`), +4 its size, +8 the RAIL DATA (0x30, `0219F018`):
#   +8 / +0xA / +0xC / +0xE u16 counts (points of 0x70, lines of 0x48, two tables of 0x24), +0x10 .. +0x1C pointers to the four
#   tables (file + offset word 2..5), +0x2C the word at file + file[1]. Clear (0219EF14 on +8): zero 0x30 bytes, counts 0x14 / 3 / 3,
#   +0x18 / +0x1C / +0x20 = the default tables ROM 0x021DDDB8 / 0x021DDDA0 / 0x021DDDAC.
#
# Conventions (headers of the F5b / F6b / F14 specs): all blocks `nowork`; objects are `arg buf` arenas; closure stubs (a block lists
# the stubs of its callees and of the callees of all its in-area descendants; checked with every function treated as ported: no fault,
# stray write or unstubbed callee); every out-of-area stub is `force`. MI_CpuFill8 / _u32_div_f run as real code (ALLOW).
area rail_loader
set field

# FUN_0219EF14(r): clear the RAIL DATA r (0x30 bytes) and set the default counts / tables.
fn 0219ef14 ret=none inputs=20
  nowork
  arg buf 0x40

# FUN_0219EF44(heap): allocate the loader (0x38) and clear its rail data. arg1 (r1, overwritten) = arena.
fn 0219ef44 ret=u32 inputs=40
  nowork
  arg u32 any
  arg buf 0x40
  stub 02030734 force ret=ptr:arg1:0:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 0219ef14 ret=none args=v  # rail loader: clear the rail data (r)

# FUN_0219EF6C(ld): unload, then free the loader.
fn 0219ef6c ret=none inputs=40
  nowork
  arg buf 0x40
  fix arg0 0 u32 alt const 0 || any
  stub 0219effc ret=none args=v  # rail loader: unload (ld)
  stub 020307b0 force ret=none args=v  # Heap_Free (block)
  stub 0219ef14 ret=none args=v  # rail loader: clear the rail data (r)

# FUN_0219EF7C(ld, id, heap): load ARC 0x4F member `id` and point the rail data at its four tables (counts =
# table sizes / 0x70, 0x48, 0x24, 0x24); +0x34 = the word at file + file[1]. arg3 (r3, overwritten) = the file (arena).
fn 0219ef7c ret=none inputs=150
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg buf 0x400
  fix arg3 4 u32 mask 0x3FC
  fix arg3 8 u32 alt range 0 0x400 || any
  fix arg3 12 u32 alt range 0 0x400 || any
  fix arg3 16 u32 alt range 0 0x400 || any
  fix arg3 20 u32 alt range 0 0x400 || any
  fix arg3 24 u32 alt range 0 0x400 || any
  stub 0204a298 force ret=ptr:arg3:0:1 args=v,v,v,v,out:4  # archive: load a member (arc, id, 0, heap, &size)

# FUN_0219EFFC(ld): free the loaded file (if any, +0 = 0) and clear the rail data.
fn 0219effc ret=none inputs=40
  nowork
  arg buf 0x40
  fix arg0 0 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v  # Heap_Free (block)
  stub 0219ef14 ret=none args=v  # rail loader: clear the rail data (r)

# FUN_0219F018(ld): the rail data (ld + 8).
fn 0219f018 ret=u32 inputs=20
  nowork
  arg u32 any

# FUN_0219F01C(a, b, c, d, e, f): 02158768(a, (u16)c, e, (u16)f) (b and d unused).
fn 0219f01c ret=u32 inputs=40
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10 (a, u16 b, c, u16 d)
