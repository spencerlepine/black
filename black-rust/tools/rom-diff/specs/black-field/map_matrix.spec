# fndiff spec: map_matrix.c + map_replace.c (overlay 10 Thumb code 0x02159B48..0x0215A07A, F1 area 10.11; F2a-spec).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field map_matrix --per-function
# Set `field` (main 10 11 18 20 21). Skipped: 02159BBC (thunk `ldr r3,=Heap_Free 020307B0; bx r3`: callers stub 020307B0).
#
# The arena is the `work` image (0x212C bytes at 0x02E00000): MAP MATRIX at work+0, MAP REPLACE at work+0x1600, scratch words
# (out pointers, stub-returned var slots) at work+0x1700..0x18FF. Callee results that are pointers use `ptr:work`/`ptr:argK`.
#
# MAP MATRIX (0x152C bytes, Heap_AllocDebug(heap, 0x152C, 1, "map_matrix.c", 0x4B) in 02159B48):
#   +0x000 u16 heap id
#   +0x002 u16 default zone id (4th argument of the load; written to every cell of +0x14 when the file has no zone table)
#   +0x004 u32 matrix id (ARC 9 member)
#   +0x008 u16 width (blocks), +0x00A u16 height (blocks)          02159BF4 / 02159BF8; 02159C00 sets both if w <= 30, h <= 30, w*h <= 900
#   +0x00C u16 layout flag (file +0: 1 = the file carries a zone table)
#   +0x010 u32 cell count = width * height                         02159BFC
#   +0x014 u16 zone[900] (cell = x + y * width; 0xFFFF after create) 02159BC4 (cell), 02159BD4 (world position)
#   +0x71C u32 land[900] (map container id per cell; 0xFFFFFFFF after create; address: 02159C20)
# MATRIX FILE (ARC 9 member, loaded by 02048F60(9, id, heap, heap) in 02159B94, freed after the copy):
#   +0 u16 layout flag, +2 u16 (unused here), +4 u16 width, +6 u16 height, +8 u32 land[w*h], then (flag 1) u32 zone[w*h] (stored as u16)
#   = nds_rom::map::matrix (A6): `flags` = +0, `width`/`height`, cells = land ids, `zones` = second table when flags == 1.
# World position -> block: 02159D68(v) = v / 4096 / 16 / 32 (fx32 -> unit -> tile -> 32-tile block), each step rounding toward zero
#   (the `asrs/lsrs/adds` sign fix of a signed division by a power of two).
#
# MAP REPLACE (0x2C bytes, Heap_AllocDebug(heap, 0x2C, 1, "map_replace.c", 0x77) in 02159D84):
#   +0x00 u16 heap id, +0x04 ARC 10 handle (020490F4(10, heap)), +0x08 u32 record count (member 0 size / 16)
#   +0x0C STATE (02159E8C): +0 u8 season (0202E83C), +1 u8 = (0215843C(..) != 0x15), +2 u8 = (+1 ? season + 1 : 0),
#                           +3..+0xC u8 flag[k] = (event var VAR(k) == VALUE(k)), k < 10 (ROM table 0x02170FB0)
#   +0x1A RECORD (16 bytes, record i of member 0, read by 02159DCC): +0 u16 matrix id, +2 u8 kind (0: replace land ids in the
#         matrix, 1: reload another matrix), +3 u8 selector (0: season, 1: STATE+1, 2: STATE+2, else key k of the ROM table ->
#         STATE+3+k, -1 if none -> STATE+2), +4 u16 value[selector result]; value[0] = the id to replace (02159E00)
# ROM table 0x02170FB0: 10 x {u8, u8 key, u16 event var id, u16 value} (keys 0..9, vars 0x4031..0x403A).
# ROM table 0x02170FEC: 17 x 0x14-byte records {u16 flag id 0x9B0.., u16 bits (bit 0, bit 1), u16 zone, u8 x, u8 y, u16 zone2, ...}
#   used by 02159F94 / 02159FA8 / 02159FE0 / 0215A014 (index 1..17; 0215A064 maps an out-of-range index to entry 0).
area map_matrix
set field

# 02159B48(heap): m = Heap_AllocDebug(heap, 0x152C, 1, "map_matrix.c", 0x4B); m+0 = heap (u16); MIi_CpuClear32(-1, m+0x71C, 0xE10);
# MIi_CpuClear16(0xFFFF, m+0x14, 0x708); returns m
fn 02159b48 ret=ofs:work inputs=50
  arg u32 any
  stub 02030734 force ret=ptr:work:0:1 args=v,v,v,v,v

# 02159B94(m, id, zone, heap): data = 02048F60(9, id, heap, heap) (ARC 9 member); 02159D04(m, data, id, zone); Heap_Free(data)
# (returns Heap_Free's r0). The file data is the extra argument 4 (the function never reads it; the stub returns its address).
fn 02159b94 ret=none inputs=200
  arg work 0
  arg u32 alt range 0 254 || any
  arg u32 alt oneof 0xFFFF 0 || any
  arg u32 any
  arg buf 0x1C40
  fix arg4 0 u16 alt oneof 0 1 || any
  fix arg4 4 u16 alt range 0 30 || oneof 1 30
  fix arg4 6 u16 alt range 0 30 || oneof 1 30
  stub 02048f60 force ret=ptr:arg4:0:1 args=v,v,v,v
  stub 02159d04 ret=none args=v,v,v,v  # in-area
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 02159BC4(m, x, y): u16 m+0x14 [x + y * width] (the zone id of a block; no bounds check)
fn 02159bc4 ret=u16 inputs=200
  arg work 0
  arg u32 range 0 29
  arg u32 range 0 29
  fix work 0x8 u16 alt range 1 30 || range 1 30 || oneof 1 2 29 30
  fix work 0xA u16 alt range 1 30 || oneof 1 2 29 30
  fix work 0xC u16 alt oneof 0 1 || any

# 02159BD4(m, px, pz): 02159BC4(m, 02159D68(px), 02159D68(pz)) (zone id at a world position, fx32)
fn 02159bd4 ret=u16 inputs=200
  arg work 0
  arg s32 alt range 0 0x3BFFFFF || oneof 0 0xFFFFF 0x100000 0x1FFFFF 0x200000
  arg s32 alt range 0 0x3BFFFFF || oneof 0 0xFFFFF 0x100000 0x1FFFFF 0x200000
  fix work 0x8 u16 alt range 1 30 || range 1 30 || oneof 1 2 29 30
  fix work 0xA u16 alt range 1 30 || oneof 1 2 29 30
  fix work 0xC u16 alt oneof 0 1 || any
  stub 02159d68 ret=pick:0/1/2/5/29 args=v  # in-area
  stub 02159bc4 ret=u16 args=v,v,v  # in-area

# 02159BF4(m): u16 width (+8)
fn 02159bf4 ret=u16 inputs=20
  arg work 0

# 02159BF8(m): u16 height (+0xA)
fn 02159bf8 ret=u16 inputs=20
  arg work 0

# 02159BFC(m): u32 cell count (+0x10)
fn 02159bfc ret=u32 inputs=20
  arg work 0

# 02159C00(m, w, h): if w <= 30 && h <= 30 && w * h <= 900 (unsigned): +8 = w, +0xA = h (u16), +0x10 = w * h
fn 02159c00 ret=none inputs=200
  arg work 0
  arg u32 alt range 0 32 || oneof 29 30 31 || any
  arg u32 alt range 0 32 || oneof 29 30 31 || any

# 02159C20(m): m + 0x71C (address of the land table)
fn 02159c20 ret=ofs:work inputs=10
  arg work 0

# 02159C2C(m, x, y): 1 if 0 <= y < height && 0 <= x < width (signed compares), else 0
fn 02159c2c ret=bool inputs=200
  arg work 0
  arg s32 alt range 0 31 || oneof 0xFFFFFFFF 0x80000000 || any
  arg s32 alt range 0 31 || oneof 0xFFFFFFFF 0x80000000 || any
  fix work 0x8 u16 alt range 1 30 || range 1 30 || oneof 1 2 29 30
  fix work 0xA u16 alt range 1 30 || oneof 1 2 29 30
  fix work 0xC u16 alt oneof 0 1 || any

# 02159C48(m, px, pz): 02159C2C(m, 02159D68(px), 02159D68(pz))
fn 02159c48 ret=bool inputs=200
  arg work 0
  arg s32 alt range 0 0x3FFFFFF || oneof 0xFFFFFFFF 0xFFF00000 || any
  arg s32 alt range 0 0x3FFFFFF || oneof 0xFFFFFFFF 0xFFF00000 || any
  fix work 0x8 u16 alt range 1 30 || range 1 30 || oneof 1 2 29 30
  fix work 0xA u16 alt range 1 30 || oneof 1 2 29 30
  fix work 0xC u16 alt oneof 0 1 || any
  stub 02159d68 ret=pick:0/1/2/5/29 args=v  # in-area
  stub 02159c2c ret=pick:0/1 args=v,v,v  # in-area

# 02159C68(m, rep, heap): k = 02159E00(rep, &old, &new); k == 2: 02159B94(m, new & 0xFFFF, m->zone (u16 +2), heap) (reload);
# k == 1: every land[i] (i < count) equal to old becomes new; else nothing. The file buffer for the reload is argument 3.
fn 02159c68 ret=none inputs=300
  arg work 0
  arg work 0x1600
  arg u32 any
  arg buf 0x1C40
  fix work 0x8 u16 alt range 1 30 || range 1 30 || oneof 1 2 29 30
  fix work 0xA u16 alt range 1 30 || oneof 1 2 29 30
  fix work 0xC u16 alt oneof 0 1 || any
  fix work 0x10 u32 alt range 0 900 || oneof 0 1 900
  fix work 0x71C u32 alt oneof 0 1 2 3 || any rep=32,4
  fix work 0x160C u8 alt range 0 5 || range 0 3
  fix work 0x160D u8 range 0 1
  fix work 0x160E u8 range 0 5
  fix work 0x160F u8 range 0 1 rep=10,1
  fix work 0x161C u8 alt oneof 0 1 2 || range 0 9 || any
  fix work 0x161D u8 alt oneof 0 1 2 || range 0 9 || any
  fix work 0x161E u16 alt oneof 0 1 2 3 || any
  fix work 0x1620 u16 alt oneof 0 1 2 3 || any
  fix work 0x1622 u16 alt oneof 0 1 2 3 || any
  fix arg3 0 u16 alt oneof 0 1 || any
  fix arg3 4 u16 alt range 0 30 || oneof 1 30
  fix arg3 6 u16 alt range 0 30 || oneof 1 30
  stub 02159e00 ret=pick:0/1/2 args=v,outm:4:3/0/0/0,out:4  # in-area
  stub 02159b94 ret=none args=v,v,v,v  # in-area
  stub 02159e60 ret=pick:0/1/3/9/0xFFFFFFFF args=v  # in-area
  stub 02048f60 force ret=ptr:arg3:0:1 args=v,v,v,v
  stub 02159d04 ret=none args=v,v,v,v  # in-area
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 02159CC4(m, gamesys, heap): rep = 02159D84(heap, gamesys); n = 02159DC8(rep); for i < n: if 02159DCC(rep, i) == m->matrix id (+4):
# 02159C68(m, rep, heap); 02159DEC(rep). The matrix file buffer for a reload is argument 3.
fn 02159cc4 ret=none inputs=300
  arg work 0
  arg u32 any
  arg u32 any
  arg buf 0x1C40
  fix work 0x8 u16 alt range 1 30 || range 1 30 || oneof 1 2 29 30
  fix work 0xA u16 alt range 1 30 || oneof 1 2 29 30
  fix work 0xC u16 alt oneof 0 1 || any
  fix work 0x4 u32 alt range 0 3 || any
  fix work 0x10 u32 alt range 0 900 || oneof 0 1
  fix work 0x71C u32 alt oneof 0 1 2 3 || any rep=32,4
  fix work 0x1608 u32 range 0 4
  fix work 0x160C u8 alt range 0 5 || range 0 3
  fix work 0x160D u8 range 0 1
  fix work 0x160E u8 range 0 5
  fix work 0x160F u8 range 0 1 rep=10,1
  fix work 0x161C u8 alt oneof 0 1 2 || range 0 9 || any
  fix work 0x161D u8 alt oneof 0 1 2 || range 0 9 || any
  fix work 0x161A u16 range 0 3
  fix work 0x161E u16 alt oneof 0 1 2 3 || any
  fix work 0x1620 u16 alt oneof 0 1 2 3 || any
  fix arg3 0 u16 alt oneof 0 1 || any
  fix arg3 4 u16 alt range 0 30 || oneof 1 30
  fix arg3 6 u16 alt range 0 30 || oneof 1 30
  stub 02159d84 ret=ptr:work:0x1600:1 args=v,v  # in-area
  stub 02159dc8 ret=pick:0/1/2/3 args=v  # in-area
  stub 02159dcc ret=pick:0/1/2/3 args=v,v  # in-area
  stub 02159c68 ret=none args=v,v,v  # in-area
  stub 02159dec ret=none args=v  # in-area
  stub 02030734 force ret=ptr:work:0x1600:1 args=v,v,v,v,v
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap): handle
  stub 020493d4 force ret=pick:0/16/32/48/64/0x80 args=v,v
  stub 02159e8c ret=none args=v,v  # in-area
  stub 0204944c force ret=none args=v,v,v,v,outm:0x10:0x3/0/0x1/0x3
  stub 02159e00 ret=pick:0/1/2 args=v,outm:4:3/0/0/0,out:4  # in-area
  stub 02159b94 ret=none args=v,v,v,v  # in-area
  stub 02049238 force ret=none args=v  # archive close (handle)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 020120f4 force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 02012138 force ret=u32 args=v  # main
  stub 0202e83c force ret=pick:0/1/2/3 args=v
  stub 0215843c force ret=pick:0/21/22 args=v  # ov10
  stub 020143cc force ret=ptr:work:0x1700:8 args=v,v
  stub 02159e60 ret=pick:0/1/3/9/0xFFFFFFFF args=v  # in-area
  stub 02048f60 force ret=ptr:arg3:0:1 args=v,v,v,v
  stub 02159d04 ret=none args=v,v,v,v  # in-area

# 02159D04(m, data, id, zone): +4 = id, +2 = zone (u16), +8 = width (data+4), +0xA = height (data+6), +0xC = flag (data+0),
# +0x10 = w * h; MIi_CpuCopy32(data + 8, m + 0x71C, 4 * w * h); flag == 1: zone[i] = (u16) data[8 + 4 * (w * h + i)] for i < w * h;
# else MIi_CpuClear16(zone, m + 0x14, 0x708)
fn 02159d04 ret=none inputs=200
  arg work 0
  arg buf 0x1C40
  arg u32 any
  arg u32 any
  fix arg1 0 u16 alt oneof 0 1 || any
  fix arg1 4 u16 alt range 0 30 || oneof 1 30
  fix arg1 6 u16 alt range 0 30 || oneof 1 30

# 02159D68(v): ((v / 4096) / 16) / 32, each a signed division rounding toward zero (fx32 world position -> block)
fn 02159d68 ret=u32 inputs=300
  arg s32 alt any || range 0 0x3FFFFFF || oneof 0xFFFFFFFF 0xFFFFF000 0xFFF00000 0xFFF00001 0x100000 0xFFFFF 0x80000000 0x7FFFFFFF

# 02159D84(heap, gamesys): r = Heap_AllocDebug(heap, 0x2C, 1, "map_replace.c", 0x77); r+0 = heap (u16); r+4 = 020490F4(10, heap);
# r+8 = (u16 020493D4(r+4, 0)) >> 4 (record count); 02159E8C(r + 0xC, gamesys); returns r
fn 02159d84 ret=ofs:work inputs=100
  arg u32 any
  arg u32 any
  stub 02030734 force ret=ptr:work:0x1600:1 args=v,v,v,v,v
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap): handle
  stub 020493d4 force ret=u32 args=v,v  # archive member size (handle, member)
  stub 02159e8c ret=none args=v,v  # in-area
  stub 020120f4 force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 02012138 force ret=u32 args=v  # main
  stub 0202e83c force ret=u32 args=v  # main
  stub 0215843c force ret=pick:0/21/22 args=v  # ov10
  stub 020143cc force ret=ptr:work:0x1700:8 args=v,v

# 02159DC8(rep): u32 +8 (record count)
fn 02159dc8 ret=u32 inputs=10
  arg work 0x1600

# 02159DCC(rep, i): 0204944C(rep+4 handle, 0, i * 16, 16, rep + 0x1A) (read record i); returns u16 rep+0x1A (its matrix id)
fn 02159dcc ret=u16 inputs=100
  arg work 0x1600
  arg u32 alt range 0 64 || any
  stub 0204944c force ret=none args=v,v,v,v,out:0x10  # archive read (handle, member, offset, size, dst): fills dst

# 02159DEC(rep): 02049238(rep+4 handle) (archive close); Heap_Free(rep)
fn 02159dec ret=none inputs=20
  arg work 0x1600
  stub 02049238 force ret=none args=v  # archive close (handle)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 02159E00(rep, &old, &new): selector = RECORD+3: 0 -> s = STATE+0, 1 -> STATE+1, 2 -> STATE+2, else STATE[3 + 02159E60(selector)]
# (index -1 = STATE+2); *old = value[0], *new = value[s] (u16 at RECORD + 4 + 2 s, stored as u32); old == new: 0;
# else RECORD+2 == 1: 2, == 0: 1, else 0. The out pointers are work+0x1700 / +0x1704.
fn 02159e00 ret=u32 inputs=400
  arg work 0x1600
  arg work 0x1700
  arg work 0x1704
  fix work 0x160C u8 alt range 0 5 || range 0 3
  fix work 0x160D u8 range 0 1
  fix work 0x160E u8 range 0 5
  fix work 0x160F u8 range 0 1 rep=10,1
  fix work 0x161C u8 alt oneof 0 1 2 || range 0 9 || any
  fix work 0x161D u8 alt oneof 0 1 2 || range 0 9 || any
  fix work 0x161E u16 alt oneof 0 1 2 || any
  fix work 0x1620 u16 alt oneof 0 1 2 || any
  fix work 0x1622 u16 alt oneof 0 1 2 || any
  stub 02159e60 ret=pick:0/1/3/9/0xFFFFFFFF args=v  # in-area

# 02159E60(key): index k < 10 with ROM table 0x02170FB0[k].key (byte +1) == key, else -1
fn 02159e60 ret=u32 inputs=100
  arg u32 alt range 0 11 || any

# 02159E8C(state, gamesys): gd = 020120F4(gamesys); ev = 0201296C(gd); x = 02012138(gamesys); s = 0202E83C(gamesys); +0 = s;
# +1 = (0215843C(x) != 0x15); +2 = +1 ? s + 1 : 0; for k < 10: +3+k = (*020143CC(ev, VAR(k)) == VALUE(k)) (ROM table 0x02170FB0)
fn 02159e8c ret=none inputs=200
  arg work 0x160C
  arg u32 any
  fix work 0x1700 u16 alt oneof 0x1220 0x224 0x206 0x1209 0x316 0x1227 0x830 0x408 0x120 0x1028 || any rep=8,4
  stub 020120f4 force ret=u32 args=v  # main
  stub 0201296c force ret=u32 args=v  # main
  stub 02012138 force ret=u32 args=v  # main
  stub 0202e83c force ret=u32 args=v  # main
  stub 0215843c force ret=pick:0/21/22 args=v  # ov10
  stub 020143cc force ret=ptr:work:0x1700:8 args=v,v

# 02159F04(_, key): address of the first ROM table 0x02170FB0 entry whose key (byte +1) == key, else 0 (r0 is not read)
fn 02159f04 ret=u32 inputs=100
  arg u32 any
  arg u32 alt range 0 11 || any

# 02159F28(gd, key, on): e = 02159F04(gd, key); if e: v = 020143CC(0201296C(gd), e->var); *v = on ? e->value : 0 (u16)
fn 02159f28 ret=none inputs=200
  arg u32 any
  arg u32 alt range 0 11 || any
  arg u32 alt oneof 0 1 || any
  stub 02159f04 ret=pick:0/0x02170FB0/0x02170FB6/0x02170FE6 args=p,v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 020143cc force ret=ptr:work:0x1700:8 args=v,v

# 02159F54(gd, key): e = 02159F04(gd, key); e ? (*020143CC(0201296C(gd), e->var) == e->value) : 0
fn 02159f54 ret=bool inputs=200
  arg u32 any
  arg u32 alt range 0 11 || any
  fix work 0x1700 u16 alt oneof 0x1220 0x224 0x206 0x1209 0x316 0x1227 0x830 0x408 0x120 0x1028 || any rep=8,4
  stub 02159f04 ret=pick:0/0x02170FB0/0x02170FB6/0x02170FE6 args=p,v  # in-area
  stub 0201296c force ret=u32 args=v  # main
  stub 020143cc force ret=ptr:work:0x1700:8 args=v,v

# 02159F80(): 1
fn 02159f80 ret=u32 inputs=5
  nowork

# 02159F84(i): 1 if 1 <= i <= 17 (signed low bound), else 0
fn 02159f84 ret=bool inputs=100
  nowork
  arg s32 alt range 0 18 || any

# 02159F94(i): u16 +8 of ROM table 0x02170FEC entry [0215A064(i)] (the zone of fly point i)
fn 02159f94 ret=u16 inputs=60
  nowork
  arg s32 alt range 0 18 || any
  stub 0215a064 ret=pick:0/1/5/16 args=v  # in-area
  stub 02159f84 ret=pick:0/1 args=v  # in-area
  stub 02159f80 ret=u32 args=  # in-area

# 02159FA8(i, loc): e = ROM table 0x02170FEC entry [0215A064(i)]; 0215A048(loc, e+4 zone2 (u16), 0, low byte of e+6, high byte of e+6)
fn 02159fa8 ret=none inputs=60
  arg s32 alt range 0 18 || any
  arg u32 any
  stub 0215a064 ret=pick:0/1/5/16 args=v  # in-area
  stub 0215a048 ret=none args=v,v,p,v,v  # in-area
  stub 02159f84 ret=pick:0/1 args=v  # in-area
  stub 02159f80 ret=u32 args=  # in-area
  stub 02014184 force ret=u32 args=v,v,v,v,v,v  # main

# 02159FE0(zone): 1 + the first index k < 17 of ROM table 0x02170FEC with u16 +4 == zone and bit 0 of +2 set, else 0
fn 02159fe0 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x186 0x18e 0x8 0x14 0x29 0x41 0x63 0x6d 0x73 0x7a 0x197 0x92 0x19d 0x93 0x1 0x1a9 0xd6 || range 0 0x1ff

# 0215A014(gd, zone): for the first entry k < 17 of ROM table 0x02170FEC with u16 +8 == zone and bit 1 of +2 set:
# 02014314(0201296C(gd), u16 +0 flag id) (set the flag); none: nothing
fn 0215a014 ret=none inputs=100
  nowork
  arg u32 any
  arg u32 alt oneof 0x185 0x18d 0x6 0x10 0x1c 0x3e 0x60 0x6b 0x71 0x78 0x196 0x88 0x19c 0x93 0x0 0x1a8 0xd6 || range 0 0x1ff
  stub 0201296c force ret=u32 args=v  # main
  stub 02014314 force ret=u32 args=v,v  # main

# 0215A048(loc, zone, _, x, y (stack, u16)): 02014184(loc, zone, 1, x << 16, 0, y << 16) (sets a LOCATION; r2 is not read)
fn 0215a048 ret=none inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt range 0 255 || any
  arg u32 alt range 0 255 || any
  stub 02014184 force ret=u32 args=v,v,v,v,v,v  # main

# 0215A064(i): (02159F84(i) ? i : 02159F80()) - 1 (0-based entry of ROM table 0x02170FEC)
fn 0215a064 ret=u32 inputs=60
  nowork
  arg s32 alt range 0 18 || any
  stub 02159f84 ret=pick:0/1 args=v  # in-area
  stub 02159f80 ret=u32 args=  # in-area
