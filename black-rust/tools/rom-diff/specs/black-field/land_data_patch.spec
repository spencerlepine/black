# fndiff spec: land_data_patch.c (overlay 21 Thumb code 0x021C939C..0x021C94BC, F1 area 21.82; F2a-spec).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field land_data_patch --per-function
#
# LAND DATA PATCH (4 bytes, Heap_AllocDebug(heap, 4, 1, "land_data_patch.c", 0x4F) in 021C939C):
#   +0 -> PATCH FILE (ARC 156 member `id`, loaded with 0204A1BC(0x9C, id, 0, heap))
# PATCH FILE: +0 u16 offset of the ATTRIBUTE section, +2 u16 offset of the BUILDING section (both from the file start)
#   ATTRIBUTE section: +0 u16 row width (cells), +4 cells of 0x18 bytes (row-major); 021C93E4 copies a rectangle of it
#   into a destination block of the same layout (u16 width at +0, cells at +4)
#   BUILDING section: +0 u32 count, +4 records of 16 bytes {s32 x, s32 y, s32 z, u32 ..}; 021C9450 offsets x by +dx << 16 and
#   z by -dz << 16 and hands each record to 0218BA04 (field_buildmodel, F2b)
# Cell size 0x18 and the record fields are read from the copy arithmetic only (UNVERIFIED meaning; A6 decodes the map
# container's permission grid with 8-byte cells, so this is a different block).
# Arena: work image; the patch object at work+0, the file at work+0x100 (attribute section at file+0x400, building section at
# file+0x40), the destination attribute block at work+0x1000.
area land_data_patch
set field

# 021C939C(id, heap): p = Heap_AllocDebug(heap, 4, 1, "land_data_patch.c", 0x4F); p+0 = 0204A1BC(0x9C, id, 0, heap); returns p
fn 021c939c ret=ofs:work inputs=50
  arg u32 any
  arg u32 any
  stub 02030734 force ret=ptr:work:0:1 args=v,v,v,v,v
  stub 0204a1bc force ret=u32 args=v,v,v,v  # load archive member (arc, member, ..)

# 021C93D0(p): Heap_Free(p+0); Heap_Free(p)
fn 021c93d0 ret=none inputs=20
  arg work 0
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 021C93E4(p, dst, sx, sy, dx, dy, w, h): src = 021C94AC(p) (attribute section); for i < h (unsigned):
# MI_CpuCopy8(src + 4 + (sx + (sy + i) * src.width) * 0x18, dst + 4 + (dx + (dy + i) * dst.width) * 0x18, w * 0x18)
fn 021c93e4 ret=none inputs=300
  arg work 0
  arg work 0x1000
  arg u32 range 0 3
  arg u32 range 0 3
  arg u32 range 0 3
  arg u32 range 0 3
  arg u32 range 0 4
  arg u32 alt range 0 4 || oneof 0 1
  fix work 0x0 u32 addrof work 0x100
  fix work 0x100 u16 const 0x400
  fix work 0x102 u16 const 0x40
  fix work 0x140 u32 alt range 0 8 || oneof 0 1
  fix work 0x500 u16 range 4 8
  fix work 0x1000 u16 range 4 8
  stub 021c94ac ret=ptr:work:0x500:1 args=v  # in-area

# 021C9450(p, a, b, base, dx, dz): sec = 021C94B4(p) (building section); for i < sec.count: copy record i (16 bytes) to the stack,
# x += dx << 16, z -= dz << 16, 0218BA04(a, b, &rec, base + i); returns base + count
fn 021c9450 ret=u32 inputs=200
  arg work 0
  arg u32 any
  arg u32 any
  arg u32 alt range 0 20 || any
  arg u32 alt range 0 64 || any
  arg u32 alt range 0 64 || any
  fix work 0x0 u32 addrof work 0x100
  fix work 0x100 u16 const 0x400
  fix work 0x102 u16 const 0x40
  fix work 0x140 u32 alt range 0 8 || oneof 0 1
  fix work 0x500 u16 range 4 8
  fix work 0x1000 u16 range 4 8
  stub 021c94b4 ret=ptr:work:0x140:1 args=v  # in-area
  stub 0218ba04 force ret=none args=v,v,in:16,v

# 021C94AC(p): file + u16 file+0 (the attribute section)
fn 021c94ac ret=ofs:work inputs=50
  arg work 0
  fix work 0x0 u32 addrof work 0x100
  fix work 0x100 u16 alt range 0 0x1000 || any

# 021C94B4(p): file + u16 file+2 (the building section)
fn 021c94b4 ret=ofs:work inputs=50
  arg work 0
  fix work 0x0 u32 addrof work 0x100
  fix work 0x102 u16 alt range 0 0x1000 || any
