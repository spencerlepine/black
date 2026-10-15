# fndiff spec (F3b): `rail_attr.c` (overlay 21 Thumb code 0x021BB088..0x021BB1F4; F1 row 21.54). Run:
# tools/check-fn.sh black-field rail_attr --per-function
#
# File bound: `Heap_AllocDebug(heap, 0x10, 1, "rail_attr.c", 0x5C / 0x86)` in 021BB088 / 021BB0B4 (string 0x021DE2C4); the
# next file (field_nogrid_mapper.c) starts at 021BB1F4.
#
# RAIL ATTR (0x10): +0 the loaded attribute file (ARC 0x4F member), +4 its size, +8 the number of lines, +0xC -> line[count]
# (allocated). The file: u32 count, then per line a record {u16 width, u16 length, u32 attr[width * length]}. The attribute at
# a rail location (021BB13C; LOCATION {u16 line, u8 kind, s16 x, u16 z}): kind 1 and line < count, x' = x + width / 2,
# 0 <= x' < width and 0 <= z < length: attr[x' + width * z], else 0xFFFFFFFF. Attribute predicates: 0xA0 / 0xA1, 0xA3, 0xA5,
# 0xA4, 0xA7, 0xA6, 0xA8, 0xB0.
#
# Conventions (headers of the F5b / F6b / F14 specs): all blocks `nowork`; objects are `arg buf` arenas (a trailing buffer
# the function never reads as a register holds the files and tables the stubs' pointer answers point into); closure stubs
# (checked with every function treated as ported: no fault, stray write or unstubbed callee); every out-of-area stub is
# `force`.
area rail_attr
set field

# FUN_021BB088(heap): allocate a RAIL ATTR (0x10). arg1 (r1, overwritten) = arena.
fn 021bb088 ret=u32 inputs=30
  nowork
  arg u32 any
  arg buf 0x40
  stub 02030734 force ret=ptr:arg1:0:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)

# FUN_021BB0A4(ra): unload, then free the RAIL ATTR.
fn 021bb0a4 ret=none inputs=40
  nowork
  arg buf 0x40
  fix arg0 0 u32 alt const 0 || any
  stub 021bb11c ret=none args=v  # rail attr: unload (ra)
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021BB0B4(ra, id, heap): load ARC 0x4F member `id` (size to ra + 4), allocate line[count] and point each
# entry at its record (records: {u16 w, u16 h, u32 attr[w * h]}). arg3 (r3, overwritten) = arena: the file at +0 (count
# 0..6, every halfword 0..7), the line table at +0x800.
fn 021bb0b4 ret=none inputs=200
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg buf 0x1000
  fix arg3 4 u32 mask 0x00070007 rep=500,4
  fix arg3 0 u32 alt range 0 6 || const 0
  stub 0204a298 force ret=ptr:arg3:0:1 args=v,v,v,v,out:4  # archive: load a member (arc, id, 0, heap, &size)
  stub 02030734 force ret=ptr:arg3:0x800:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line): the line table

# FUN_021BB11C(ra): when a file is loaded, free the line table and the file and zero +0 / +8 / +0xC.
fn 021bb11c ret=none inputs=40
  nowork
  arg buf 0x40
  fix arg0 0 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021BB13C(ra, loc): the attribute at the rail location (0xFFFFFFFF when outside). arg2 (r2, overwritten) =
# arena: eight line records (w, h 0..7) at +0x100 * k, the line table at +0.
fn 021bb13c ret=u32 inputs=1000
  nowork
  arg buf 0x40
  arg buf 0x10
  arg buf 0x1000
  fix arg0 8 u32 alt range 0 9 || range 0 64 || oneof 0x80000000 0xFFFFFFFF
  fix arg0 12 u32 addrof arg2 0
  fix arg2 0 u32 oneof 0x02E30100 0x02E30200 0x02E30300 0x02E30400 0x02E30500 0x02E30600 0x02E30700 0x02E30800 rep=64,4
  fix arg2 0x100 u16 alt range 1 7 || range 0 7 rep=8,0x100
  fix arg2 0x102 u16 alt range 1 7 || range 0 7 rep=8,0x100
  fix arg1 0 u16 alt range 0 9 || range 0 63
  fix arg1 2 u8 alt const 1 || const 1 || const 1 || any
  fix arg1 4 u16 alt range 0xFFFC 0xFFFF || range 0 4 || range 0 4 || any
  fix arg1 6 u16 alt range 0 7 || range 0 7 || any

# FUN_021BB188(attr): attr is 0xA0 or 0xA1 (low halfword).
fn 021bb188 ret=u32 inputs=60
  nowork
  arg u32 alt range 0x9E 0xA3 || any || range 0xFFFF009E 0xFFFF00A3

# FUN_021BB1A0(attr): attr == 0xA3.
fn 021bb1a0 ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any

# FUN_021BB1AC(attr): attr == 0xA5.
fn 021bb1ac ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any

# FUN_021BB1B8(attr): attr == 0xA4.
fn 021bb1b8 ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any

# FUN_021BB1C4(attr): attr == 0xA7.
fn 021bb1c4 ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any

# FUN_021BB1D0(attr): attr == 0xA6.
fn 021bb1d0 ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any

# FUN_021BB1DC(attr): attr == 0xA8.
fn 021bb1dc ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any

# FUN_021BB1E8(attr): attr == 0xB0.
fn 021bb1e8 ret=u32 inputs=120
  nowork
  arg u32 alt range 0x9E 0xB2 || any
