# fndiff spec: fieldmap_func.c (overlay 21 Thumb code 0x021BB890..0x021BBCB8, F1 area 21.57; F2a-spec).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field fieldmap_func --per-function
#
# FIELDMAP FUNC SYSTEM (0x18 bytes, Heap_AllocDebug(heap, 0x18, 1, "fieldmap_func.c", 0x46) in 021BB890):
#   +0x00 fieldmap pointer, +0x04 heap id, +0x08 s32 slot count, +0x0C -> SLOT[count] (0x18 bytes each, alloc line 0x4A),
#   +0x10 task system A (02030CBC(count, block of 02030CB0(count) bytes), line 0x56; run by 021BB948),
#   +0x14 task system B (same, line 0x59; run by 021BB954)
# SLOT (0x18 bytes; free when +4 == 0): +0x00 -> system, +0x04 task A handle (021BB961 = update), +0x08 task B handle
#   (021BB979 = draw), +0x0C -> FUNC DATA, +0x10 -> work block (FUNC DATA +4 bytes, line 0xC3), +0x14 overlay id (-1 = none)
# FUNC DATA (ROM table of the caller, at least 0x18 bytes): +0x00 u32 task priority, +0x04 u16 work size, +0x08 create(slot, fieldmap,
#   work), +0x0C delete(..), +0x10 update(..), +0x14 draw(..) (each nullable; called with blx)
# In the specs the system is work+0, its slot array work+0x100 (8 slots), FUNC DATA work+0x400; the callbacks are FAKE addresses
# 0x02EE0011 (create), 0x02EE0021 (delete), 0x02EE0031 (update), 0x02EE0041 (draw) of the unused page 0x02EE0000.
#
# PASS-BY RING (at object + 0x30E0, 021BBA70 .. 021BBC58): 30 ENTRIES of 0x24 bytes, then +0x438 u8 oldest index, +0x439 u8 next
#   index (both wrap at 30), +0x43C s32 total count, +0x440 s32 low mark (count - 29 at most). ENTRY: +0x00 16 bytes (u16 string,
#   0201F41C copy of 8 chars), +0x10 u16 x4, +0x18 u16 key, +0x1A 6-byte id (all 0xFF = empty, ROM 0x021D9AE0), +0x20 u8, +0x21 u8,
#   +0x22 u8. Callees at 0x0217F894 belong to an overlay that shares the ov18 address range (stubbed by address).
area fieldmap_func
set field

# 021BB890(fieldmap, heap, count): s = Heap_AllocDebug(heap, 0x18, ..0x46); s+0 = fieldmap, +4 = heap, +8 = count;
# +0xC = Heap_AllocDebug(heap, count * 0x18, ..0x4A); n = 02030CB0(count); +0x14 = 02030CBC(count, Heap_AllocDebug(heap, n, ..0x56));
# +0x10 = 02030CBC(count, Heap_AllocDebug(heap, n, ..0x59)); returns s (Ghidra drops everything after the first alloc)
fn 021bb890 ret=ofs:work inputs=60
  arg u32 any
  arg u32 any
  arg u32 alt range 0 8 || any
  stub 02030734 force ret=ptr:work:0:1 args=v,v,v,v,v
  stub 02030cb0 force ret=u32 args=v  # task-system size for n tasks (n * 32 + 0x38)
  stub 02030cbc force ret=u32 args=v,v  # task system init (n, block): returns the block

# 021BB90C(s): for i < s+8 (signed): 021BBA1C(s+0xC + 0x18 i); Heap_Free(+0xC), Heap_Free(+0x10), Heap_Free(+0x14), Heap_Free(s)
fn 021bb90c ret=none inputs=200
  arg work 0
  fix work 0x0 u32 any
  fix work 0x8 s32 alt range 0 8 || oneof 0 1 8
  fix work 0xC u32 addrof work 0x100
  fix work 0x100 u32 addrof work 0 rep=8,0x18
  fix work 0x104 u32 alt const 0 || any rep=8,0x18
  fix work 0x10C u32 addrof work 0x400 rep=8,0x18
  fix work 0x114 u32 alt oneof 0xFFFFFFFF || any rep=8,0x18
  fix work 0x404 u16 alt const 0 || range 1 0x400
  fix work 0x408 u32 alt const 0 || const 0x02EE0011
  fix work 0x40C u32 alt const 0 || const 0x02EE0021
  fix work 0x410 u32 alt const 0 || const 0x02EE0031
  fix work 0x414 u32 alt const 0 || const 0x02EE0041
  stub 021bba1c ret=none args=v  # in-area
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02030eac force ret=none args=v  # task delete (task)
  stub 02034a5c force ret=none args=v  # overlay unload (id; -1 = none)
  stub 02ee0020 force ret=none args=v,v,v  # fieldmap_func FUNC DATA delete(slot, fieldmap, work)

# 021BB948(s): 02030D28(s+0x10) (tail call: run task system A)
fn 021bb948 ret=none inputs=20
  arg work 0
  fix work 0x0 u32 any
  fix work 0x8 s32 alt range 0 8 || oneof 0 1 8
  fix work 0xC u32 addrof work 0x100
  fix work 0x100 u32 addrof work 0 rep=8,0x18
  fix work 0x104 u32 alt const 0 || any rep=8,0x18
  fix work 0x10C u32 addrof work 0x400 rep=8,0x18
  fix work 0x114 u32 alt oneof 0xFFFFFFFF || any rep=8,0x18
  stub 02030d28 force ret=none args=v  # task system run (sys)

# 021BB954(s): 02030D28(s+0x14) (tail call: run task system B)
fn 021bb954 ret=none inputs=20
  arg work 0
  fix work 0x0 u32 any
  fix work 0x8 s32 alt range 0 8 || oneof 0 1 8
  fix work 0xC u32 addrof work 0x100
  fix work 0x100 u32 addrof work 0 rep=8,0x18
  fix work 0x104 u32 alt const 0 || any rep=8,0x18
  fix work 0x10C u32 addrof work 0x400 rep=8,0x18
  fix work 0x114 u32 alt oneof 0xFFFFFFFF || any rep=8,0x18
  stub 02030d28 force ret=none args=v  # task system run (sys)

# 021BB960(task, slot): f = slot->data->update (+0x10); if f: f(slot, *slot->system (fieldmap), slot+0x10 work)
fn 021bb960 ret=none inputs=60
  arg u32 any
  arg work 0x100
  fix work 0x100 u32 addrof work 0
  fix work 0x10C u32 addrof work 0x400
  fix work 0x404 u16 alt const 0 || range 1 0x400
  fix work 0x408 u32 alt const 0 || const 0x02EE0011
  fix work 0x40C u32 alt const 0 || const 0x02EE0021
  fix work 0x410 u32 alt const 0 || const 0x02EE0031
  fix work 0x414 u32 alt const 0 || const 0x02EE0041
  stub 02ee0030 force ret=none args=v,v,v  # fieldmap_func FUNC DATA update(slot, fieldmap, work)

# 021BB978(task, slot): f = slot->data->draw (+0x14); if f: f(slot, *slot->system, slot+0x10)
fn 021bb978 ret=none inputs=60
  arg u32 any
  arg work 0x100
  fix work 0x100 u32 addrof work 0
  fix work 0x10C u32 addrof work 0x400
  fix work 0x404 u16 alt const 0 || range 1 0x400
  fix work 0x408 u32 alt const 0 || const 0x02EE0011
  fix work 0x40C u32 alt const 0 || const 0x02EE0021
  fix work 0x410 u32 alt const 0 || const 0x02EE0031
  fix work 0x414 u32 alt const 0 || const 0x02EE0041
  stub 02ee0040 force ret=none args=v,v,v  # fieldmap_func FUNC DATA draw(slot, fieldmap, work)

# 021BB990(overlay, s, data, _): first free slot e (+4 == 0) of s (i < s+8): overlay != -1: 02034AC4(overlay); e+0x14 = overlay;
# e+4 = 02030DA8(s+0x10, 021BB961, e, *data); e+8 = 02030DA8(s+0x14, 021BB979, e, *data); e+0 = s; e+0xC = data;
# data+4 (u16) != 0: e+0x10 = Heap_AllocDebug((u16) s+4, data+4, 1, "fieldmap_func.c", 0xC3); data+8 != 0: (data+8)(e, *s, e+0x10);
# returns e; no free slot: 0 (r3 is not read)
fn 021bb990 ret=ofs:work inputs=200
  arg s32 alt oneof 0xFFFFFFFF 0 3 || any
  arg work 0
  arg work 0x400
  fix work 0x0 u32 any
  fix work 0x8 s32 alt range 0 8 || oneof 0 1 8
  fix work 0xC u32 addrof work 0x100
  fix work 0x100 u32 addrof work 0 rep=8,0x18
  fix work 0x104 u32 alt const 0 || any rep=8,0x18
  fix work 0x10C u32 addrof work 0x400 rep=8,0x18
  fix work 0x114 u32 alt oneof 0xFFFFFFFF || any rep=8,0x18
  fix work 0x404 u16 alt const 0 || range 1 0x400
  fix work 0x408 u32 alt const 0 || const 0x02EE0011
  fix work 0x40C u32 alt const 0 || const 0x02EE0021
  fix work 0x410 u32 alt const 0 || const 0x02EE0031
  fix work 0x414 u32 alt const 0 || const 0x02EE0041
  stub 02034ac4 force ret=none args=v  # overlay load (id; -1 = none)
  stub 02030da8 force ret=u32 args=v,v,v,v
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line): blocks that dereference the result override with ptr:
  stub 02ee0010 force ret=none args=v,v,v  # fieldmap_func FUNC DATA create(slot, fieldmap, work)

# 021BBA1C(e): if e+4: data = e+0xC; data->delete (+0xC): f(e, *e->system, e+0x10); data+4 (u16) != 0: Heap_Free(e+0x10);
# 02030EAC(e+4); 02030EAC(e+8); e+0x14 != -1: 02034A5C(e+0x14) (overlay unload); MI_CpuFill8(e, 0, 0x18)
fn 021bba1c ret=none inputs=200
  arg work 0x100
  fix work 0x100 u32 addrof work 0
  fix work 0x104 u32 alt const 0 || any
  fix work 0x10C u32 addrof work 0x400
  fix work 0x114 u32 alt oneof 0xFFFFFFFF || any
  fix work 0x404 u16 alt const 0 || range 1 0x400
  fix work 0x408 u32 alt const 0 || const 0x02EE0011
  fix work 0x40C u32 alt const 0 || const 0x02EE0021
  fix work 0x410 u32 alt const 0 || const 0x02EE0031
  fix work 0x414 u32 alt const 0 || const 0x02EE0041
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02030eac force ret=none args=v  # task delete (task)
  stub 02034a5c force ret=none args=v  # overlay unload (id; -1 = none)
  stub 02ee0020 force ret=none args=v,v,v  # fieldmap_func FUNC DATA delete(slot, fieldmap, work)

# 021BBA6C(e): e+0x10 (the slot's work block)
fn 021bba6c ret=u32 inputs=20
  arg work 0x100

# 021BBA70(obj, rec, pos, flag): ring = obj + 0x30E0; rec == 0 && pos == 0: fatal; rec && !021BBC58(ring, rec+0x64, rec+0x24): return;
# e = 021BBBD8(ring); rec == 0: 020084F8(**obj, e, 8) (string copy), fatal when pos == 0 && flag == 0, e+0x10.. = pos[0..3], +0x21 = flag,
# +0x18 = 0, 02087C6C(e+0x1A) (own id), +0x20 = 02008550(**obj), +0x22 = 0;
# rec != 0: 0201F41C(rec+0xC, e, 8); e+0x10.. = rec+0x1C..+0x22, +0x21 = rec[7] & 7, +0x18 = rec+0x24, MI_CpuCopy8(rec+0x64, e+0x1A, 6),
# +0x20 = rec[7] bit 3; gd = 020120F4((*obj)+0xC); +0x22 = 0217F894(0201283C(gd), rec) == 1 ? 1 : 0200EDC4(0200ED48(02012EBC(gd)), rec+0x5C) == 1 ? 2 : 0;
# then: count (+0x43C) == low (+0x440): low += 1; count += 1; count - low >= 30: low = count - 29
fn 021bba70 ret=none inputs=300
  region rec 0x80
  region pos 0x10
  arg buf 0x3600
  arg u32 alt const 0 || addrof region:rec
  arg u32 alt const 0 || addrof region:pos || addrof region:pos
  arg u32 alt const 0 || oneof 1 2 || any
  fix arg0 0 u32 addrof arg0 0x3580
  fix arg0 0x3518 u8 range 0 29
  fix arg0 0x3519 u8 range 0 29
  fix arg0 0x351C s32 alt range 0 40 || oneof 0 1 29 30 31 || any
  fix arg0 0x3520 s32 alt range 0 40 || any
  fix arg0 0x30FC u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x30FE u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x30FA u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x30F8 u16 range 0 3 rep=30,0x24
  fix region:rec 0x24 u16 range 0 3
  stub 02034714 force noreturn ret=none args=v,v,v
  stub 021bbc58 ret=pick:0/1 args=v,v,v  # in-area
  stub 021bbbd8 ret=ptr:arg0:0x30E0:1 args=v
  stub 0201f41c force ret=none args=v,out:16,v
  stub 02087c6c force ret=none args=out:6  # copies the 6-byte player id/name block of a global to dst
  stub 02008550 force ret=u32 args=v  # main
  stub 020120f4 force ret=u32 args=v  # main
  stub 0201283c force ret=u32 args=v  # main
  stub 02012ebc force ret=u32 args=v  # main
  stub 020071cc force ret=u32 args=v  # main
  stub 0217f894 force ret=pick:0/1/2 args=v,v
  stub 0200edc4 force ret=pick:0/1/2 args=v,v  # main
  stub 021bbc2c ret=nullable:4:ptr:arg0:0:1 args=v,v

# 021BBBD8(ring): count (+0x43C) > 0 and next (+0x439) == oldest (+0x438): oldest += 1 (0 at 30); slot = next; next += 1 (0 at 30);
# returns ring + slot * 0x24
fn 021bbbd8 ret=ofs:arg0 inputs=200
  arg buf 0x480
  fix arg0 0x438 u8 range 0 29
  fix arg0 0x439 u8 range 0 29
  fix arg0 0x43C s32 alt range 0 40 || oneof 0 1 29 30 31 || any
  fix arg0 0x1A u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x1C u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x1E u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x18 u16 range 0 3 rep=30,0x24
  fix arg0 0x438 u8 alt range 0 29 || oneof 29 0

# 021BBC2C(ring, i): e = ring + (i % 30) * 0x24 (unsigned); memcmp(e+0x1A, 0x021D9AE0 (6 x 0xFF), 6) == 0 ? 0 : e
fn 021bbc2c ret=ofs:arg0 inputs=200
  arg buf 0x480
  arg u32 alt range 0 60 || any
  fix arg0 0x438 u8 range 0 29
  fix arg0 0x439 u8 range 0 29
  fix arg0 0x43C s32 alt range 0 40 || oneof 0 1 29 30 31 || any
  fix arg0 0x1A u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x1C u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x1E u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x18 u16 range 0 3 rep=30,0x24

# 021BBC58(ring, id, key): for i = count (+0x43C) down while i > count - 30: i < 0: 1; e = 021BBC2C(ring, i);
# e && memcmp(e+0x1A, id, 6) == 0: return e+0x18 (u16) != key; afterwards 1
fn 021bbc58 ret=bool inputs=3000
  region id 0x8
  arg buf 0x480
  arg u32 addrof region:id
  arg u32 alt range 0 3 || range 0 3 || const 0 || any
  fix arg0 0x438 u8 range 0 29
  fix arg0 0x439 u8 range 0 29
  fix arg0 0x43C s32 alt range 0 40 || oneof 0 1 29 30 31 || const 29 || any
  fix arg0 0x1A u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x1C u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x1E u16 alt const 0xFFFF || any rep=30,0x24
  fix arg0 0x18 u16 range 0 3 rep=30,0x24
  fix region:id 0 u16 alt const 0x1234 || const 0x1234 || const 0xFFFF
  fix region:id 2 u16 alt const 0x1234 || const 0x1234 || const 0xFFFF
  fix region:id 4 u16 alt const 0x1234 || const 0x1234 || const 0xFFFF
  fix arg0 0x1A u16 alt const 0x1234 || const 0xFFFF || const 0x5678 || const 0x5678 || const 0x5678 || const 0x5678 rep=30,0x24
  fix arg0 0x1C u16 alt const 0x1234 || const 0xFFFF rep=30,0x24
  fix arg0 0x1E u16 alt const 0x1234 || const 0xFFFF rep=30,0x24
  # Z22: entry 0 (the 30th entry examined for count 29: the loop-bound edge) matches the id more often, with key 0
  fix arg0 0x18 u16 alt const 0 || range 0 3
  fix arg0 0x1A u16 alt const 0x1234 || const 0x1234 || const 0xFFFF
  fix arg0 0x1C u16 alt const 0x1234 || const 0x1234 || const 0xFFFF
  fix arg0 0x1E u16 alt const 0x1234 || const 0x1234 || const 0xFFFF
  stub 021bbc2c ret=nullable:4:ptr:arg0:0:1 args=v,v
