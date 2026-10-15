# fndiff spec (F3b): `field_nogrid_mapper.c` (overlay 21 Thumb code 0x021BB1F4..0x021BB484; F1 row 21.55). Run:
# tools/check-fn.sh black-field nogrid_mapper --per-function
#
# File bound: `Heap_AllocDebug(.., "field_nogrid_mapper.c", 0x47 / 0xDD / 0x1C9)` in 021BB1F4 / 021BB2F8 / 021BB3E8 (string
# 0x021DE2D0); fldeff_gyoe.c starts at 021BB484. Function list: bl targets of the field set, odd code pointers, the Ghidra
# index and a Thumb walk; 021BB35E (a `bl` from data in main) is a label inside 021BB348; 021BB388, 021BB3D0 and 021BB3DC
# (thunks with no caller in the field set) are specced like the other thunks.
#
# NOGRID MAPPER (0x18): +0 the RAIL system (field_rail 021B7884), +4 the RAIL LOADER (0219EF44), +8 the RAIL ATTR (021BB088),
# +0xC the camera area object, +0x10 the camera area loader (both given to the constructor), +0x14 the "no rail frame" flag
# (021BB260: flag == 0). A map's no-grid record {u16 rail file, u16 camera area file} (ARC 0x6D member, 021BB2F8) is loaded by
# 021BB278 (0xFFFF = none; the attribute file is member rail + count / 2 of ARC 0x4F).
# Thunks (`ldr r0, [r0, #k]; ldr r3, =T; [movs r1, #1;] bx r3`) are specced; their targets are `force` stubs (the thunk runs as
# real code and lands on the stub).
#
# Conventions (headers of the F5b / F6b / F14 specs): all blocks `nowork`; the mapper is `arg buf 0x40` (its pointer fields
# are values: the callees are stubs); a trailing `arg buf` the function never reads as a register is the arena the
# allocator's answer points into; closure stubs (checked with every function treated as ported: no fault, stray write or
# unstubbed callee); every out-of-area stub is `force`.
area nogrid_mapper
set field

# FUN_021BB1F4(heap, x, cam, camld): create the mapper: RAIL (heap, 0x30, x), RAIL LOADER, RAIL ATTR. arg4 =
# arena (the mapper).
fn 021bb1f4 ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02030734 force ret=ptr:arg4:0:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 021b7884 force ret=u32 args=v,v,v  # rail: create (heap, n, x)
  stub 0219ef44 force ret=u32 args=v  # rail loader: create (heap)
  stub 021bb088 force ret=u32 args=v  # rail attr: create (heap)

# FUN_021BB240(m): delete the RAIL ATTR, the RAIL, the RAIL LOADER and the mapper.
fn 021bb240 ret=none inputs=30
  nowork
  arg buf 0x40
  stub 021bb0a4 force ret=none args=v  # rail attr: delete (ra)
  stub 021b791c force ret=none args=v  # rail: delete (R)
  stub 0219ef6c force ret=none args=v  # rail loader: delete (ld)
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021BB260(m, f): +0x14 = (f == 0); tail call 021AB75C(m->cam, f).
fn 021bb260 ret=none inputs=40
  nowork
  arg buf 0x40
  arg u32 alt const 0 || any
  stub 021ab75c force ret=none args=v,v  # camera area: +0x18 = flag (cam, f)

# FUN_021BB278(m, rec, heap): unload, then load the rail (rec +0) and the camera area (rec +2) files (0xFFFF =
# none).
fn 021bb278 ret=none inputs=200
  nowork
  arg buf 0x40
  arg buf 0x10
  arg u32 any
  fix arg1 0 u16 alt const 0xFFFF || range 0 20
  fix arg1 2 u16 alt const 0xFFFF || range 0 20
  stub 021bb348 ret=none args=v  # nogrid mapper: unload (m)
  stub 02049030 force ret=u32 args=v  # archive: member count (arc)
  stub 0219ef7c force ret=none args=v,v,v  # rail loader: load ARC 0x4F member (ld, id, heap)
  stub 0219f018 force ret=u32 args=v  # rail loader: the rail data (ld + 8)
  stub 021b7934 force ret=none args=v,v  # rail: set the rail data (R, data)
  stub 021bb0b4 force ret=none args=v,v,v  # rail attr: load ARC 0x4F member (ra, id, heap)
  stub 021ab840 force ret=none args=v,v,v,v  # camera area loader: load (ld, 0, id, heap)
  stub 021ab880 force ret=u32 args=v  # camera area loader: data (ld)
  stub 021ab884 force ret=u32 args=v  # camera area loader: count (ld)
  stub 021ab888 force ret=u32 args=  # camera area loader: constant
  stub 021ab634 force ret=u32 args=v,v,v,v  # camera area set (cam, a, b, c)
  stub 021ab650 force ret=none args=v  # camera area clear (cam)
  stub 021b7954 force ret=none args=v  # rail: clear the rail data (R)
  stub 0219effc force ret=none args=v  # rail loader: unload (ld)
  stub 021ab868 force ret=u32 args=v  # camera area loader: unload (ld)
  stub 021bb11c force ret=none args=v  # rail attr: unload (ra)

# FUN_021BB2F8(m, id, heap): load the no-grid record `id` (ARC 0x6D) into a temporary buffer and 021BB278 it.
# arg3 (r3, overwritten) = arena (the record).
fn 021bb2f8 ret=none inputs=200
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg buf 0x20
  fix arg3 0 u16 alt const 0xFFFF || range 0 20
  fix arg3 2 u16 alt const 0xFFFF || range 0 20
  stub 020490a8 force ret=u32 args=v,v  # archive: member size (arc, id)
  stub 02030734 force ret=ptr:arg3:0:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 02048f4c force ret=none args=v,v,v  # archive: load a member into a buffer (buf, arc, id)
  stub 021bb278 ret=none args=v,v,v  # nogrid mapper: load from a record (m, rec, heap)
  stub 020307b0 force ret=none args=v  # Heap_Free (block)
  stub 021bb348 ret=none args=v  # nogrid mapper: unload (m)
  stub 02049030 force ret=u32 args=v  # archive: member count (arc)
  stub 0219ef7c force ret=none args=v,v,v  # rail loader: load ARC 0x4F member (ld, id, heap)
  stub 0219f018 force ret=u32 args=v  # rail loader: the rail data (ld + 8)
  stub 021b7934 force ret=none args=v,v  # rail: set the rail data (R, data)
  stub 021bb0b4 force ret=none args=v,v,v  # rail attr: load ARC 0x4F member (ra, id, heap)
  stub 021ab840 force ret=none args=v,v,v,v  # camera area loader: load (ld, 0, id, heap)
  stub 021ab880 force ret=u32 args=v  # camera area loader: data (ld)
  stub 021ab884 force ret=u32 args=v  # camera area loader: count (ld)
  stub 021ab888 force ret=u32 args=  # camera area loader: constant
  stub 021ab634 force ret=u32 args=v,v,v,v  # camera area set (cam, a, b, c)
  stub 021ab650 force ret=none args=v  # camera area clear (cam)
  stub 021b7954 force ret=none args=v  # rail: clear the rail data (R)
  stub 0219effc force ret=none args=v  # rail loader: unload (ld)
  stub 021ab868 force ret=u32 args=v  # camera area loader: unload (ld)
  stub 021bb11c force ret=none args=v  # rail attr: unload (ra)

# FUN_021BB348(m): unload the camera area, the RAIL data, the RAIL LOADER, the camera area loader, the RAIL ATTR.
fn 021bb348 ret=none inputs=30
  nowork
  arg buf 0x40
  stub 021ab650 force ret=none args=v  # camera area clear (cam)
  stub 021b7954 force ret=none args=v  # rail: clear the rail data (R)
  stub 0219effc force ret=none args=v  # rail loader: unload (ld)
  stub 021ab868 force ret=u32 args=v  # camera area loader: unload (ld)
  stub 021bb11c force ret=none args=v  # rail attr: unload (ra)

# FUN_021BB36C(m): RAIL frame state; unless +0x14, RAIL frame (R, 0).
fn 021bb36c ret=none inputs=60
  nowork
  arg buf 0x40
  fix arg0 0x14 u32 alt const 0 || any
  stub 021b79e4 force ret=u32 args=v  # rail: frame state (R)
  stub 021b79f0 force ret=u32 args=v,v  # rail: frame (R, force)

# FUN_021BB388(m): thunk: 021B79F0(m->rail, 1).
fn 021bb388 ret=u32 inputs=20
  nowork
  arg buf 0x40
  stub 021b79f0 force ret=u32 args=v,v  # rail: frame (R, force)

# FUN_021BB394(m): thunk: 021B7970(m->rail).
fn 021bb394 ret=none inputs=20
  nowork
  arg buf 0x40
  stub 021b7970 force ret=none args=v  # rail: (R)

# FUN_021BB3A0(m, obj): thunk: 021B79AC(m->rail, obj).
fn 021bb3a0 ret=none inputs=20
  nowork
  arg buf 0x40
  arg u32 any
  stub 021b79ac force ret=none args=v,v  # rail: (R, obj)

# FUN_021BB3AC(m, x): thunk: 021B79C4(m->rail, x).
fn 021bb3ac ret=none inputs=20
  nowork
  arg buf 0x40
  arg u32 any
  stub 021b79c4 force ret=none args=v,v  # rail: +0x14 = x, +0x18 = 1 (R, x)

# FUN_021BB3B8(m): thunk: 021B79CC(m->rail).
fn 021bb3b8 ret=none inputs=20
  nowork
  arg buf 0x40
  stub 021b79cc force ret=none args=v  # rail: +0x14 = 0 (R)

# FUN_021BB3C4(m, loc): thunk: 021BB13C(m->attr, loc) (the attribute at a rail location).
fn 021bb3c4 ret=u32 inputs=30
  nowork
  arg buf 0x40
  arg buf 0x10
  stub 021bb13c force ret=u32 args=v,in:8  # rail attr: attribute at a rail location (ra, &loc)

# FUN_021BB3D0(m, loc): thunk: 021B7D68(m->rail, loc).
fn 021bb3d0 ret=none inputs=20
  nowork
  arg buf 0x40
  arg buf 0x10
  stub 021b7d68 force ret=none args=v,in:8  # rail: run the location callback (R, &loc)

# FUN_021BB3DC(m, id, f): thunk: 021B7C04(m->rail, id, f).
fn 021bb3dc ret=u32 inputs=30
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  stub 021b7c04 force ret=u32 args=v,v,v  # rail: (R, id, f)

# FUN_021BB3E8(m, zone, line, z, x, &loc, heap): load the rail file of `zone` (no-grid record of ARC 0x6D via
# 02013C9C) into a temporary RAIL LOADER and let the RAIL compute the location (021B7C58). arg7 (stack word 3, unread) =
# arena (the record).
fn 021bb3e8 ret=none inputs=150
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg buf 0x20
  fix arg7 0 u16 alt const 0xFFFF || range 0 20
  fix arg7 2 u16 alt const 0xFFFF || range 0 20
  stub 02013c9c force ret=u32 args=v  # zone data: rail file id of a zone (zone)
  stub 020490a8 force ret=u32 args=v,v  # archive: member size (arc, id)
  stub 02030734 force ret=ptr:arg7:0:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 02048f4c force ret=none args=v,v,v  # archive: load a member into a buffer (buf, arc, id)
  stub 0219ef44 force ret=u32 args=v  # rail loader: create (heap)
  stub 0219ef7c force ret=none args=v,v,v  # rail loader: load ARC 0x4F member (ld, id, heap)
  stub 0219f018 force ret=u32 args=v  # rail loader: the rail data (ld + 8)
  stub 021b7c58 force ret=none args=v,v,v,v,v,out:8  # rail: location from a line (R, data, line, z, x, &loc)
  stub 0219ef6c force ret=none args=v  # rail loader: delete (ld)
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021BB480(m): the RAIL system (+0).
fn 021bb480 ret=u32 inputs=20
  nowork
  arg buf 0x10
