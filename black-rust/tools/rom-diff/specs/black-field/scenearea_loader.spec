# fndiff spec (F2b): fld_scenearea_loader.c (overlay 21 Thumb code 0x021AB7E4..0x021ABD50, the second part of F1 row 21.31; 20
# functions, all specced): the loader of a map's scene areas (two ROM archives, ARC 0x50 and 0x9E: an archive member is a list of 0x48-byte
# AREA records) and the scene-area callbacks of the callback table it hands out (`021AB888` -> ROM 0x021D8EF4 = {check table 0x021DDFD8,
# update table 0x021DDFE0, u16 2, u16 8}): checks 021AB890 (inside a ring sector around a centre) / 021ABA3C (inside a grid rectangle,
# grid maps only); updates 021AB908 (camera on a circle around a centre), 021AB9FC (fixed camera), 021ABABC / 021ABB54 / 021ABB78 (camera
# angles and distance interpolated along the rectangle), 021ABBF0 / 021ABC14 / 021ABC38 (those plus target offset 021ABC5C and the field of
# view 021ABD08). The camera itself (field_camera.c, G8: 0218E124 mode, 0218EC90 target, 0218ECF4 position, 0218ECD4 target offset,
# 0218ED34 / 0218ED48 angles, 0218ED58 distance, 0218ED64 field of view, 0218F05C / 0218F0D4) and the FX library (0207CA00 VEC_Subtract,
# 0207CBD0 VEC_Mag, 0207CCA4 VEC_Normalize, 0207CFB8 FX_Atan2Idx, 0207C768 FX_Div) are `force` stubs (D3: the port reaches them through
# FieldEnv by address); 0209C054 (64-bit multiply) and 0209C2B0 (unsigned divide) run as real code; FX_SinCosTable_ (main 0x020A1B38) is
# read as ROM data by the original and computed by the port.
# File bound: `Heap_AllocDebug(heap, 0x10, 1, "fld_scenearea_loader.c", 0x81)` in 021AB7E4 (string 0x021DE000); fld_scenearea.c ends at
# 0x021AB7E4; field_effect.c starts at 0x021ABD50 (its allocation names "field_effect.c", 0x021DE018). Grammar: svflow_specs/README.md
# (generic mode). Run: tools/check-fn.sh black-field scenearea_loader --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers (the two callback tables 0x021DDFD8 / 0x021DDFE0), the Ghidra
# index and a Thumb walk from every entry (20). A `bl` from main 0x020A478C into 021ABC46 is data (not a call).
#
# LOADER L (0x10, 021AB7E4): +0 the loaded area list (Heap), +4 u32 area count (size / 0x48), +8 / +0xC the two archive handles (ARC
# ids from ROM 0x021D8EEC = {0x50, 0x9E}).
# AREA (0x48) as the callbacks read it: ring sector (021AB890): +0 / +4 u32 angle range [a0, a1) (wrapping when a1 < a0), +8 / +0xC
# s32 distance range [r0, r1), +0x10 / +0x14 / +0x18 the centre; circle camera (021AB908): +0x10..+0x18 the centre (target), +0x1C u32
# angle (index = angle >> 4 into FX_SinCosTable_), +0x20 s32 distance; fixed camera (021AB9FC): +0x24 target, +0x30 position; grid rect
# (021ABA3C / 021ABBA0): +0 u16 x, +2 u16 z, +4 u16 axis (1: along x, else z; for the rect check also the width when not 1), +6 / +8
# u16 extents; interpolation (021ABABC): +0xC / +0xE u16 start angles, +0x10 s32 start distance, +0x14 / +0x16 u16 end angles,
# +0x18 s32 end distance, +0x1C u32 flag (021ABB54 / 021ABB78); target offset (021ABC5C): +0x20 start, +0x2C end; field of view
# (021ABD08): +0x38 / +0x3A u16 start / end.
#
# All blocks are `nowork`. A callback is `(SA, area, pos)`: SA is a value (only handed to the SA getters 021AB760 / 021AB764 of area
# `scenearea`, `force` stubs answering the camera / fieldmap), area an `arg buf 0x48`, pos an `arg buf 0x10` (VecFx32). The angle
# +0x1C of 021AB908 is drawn as a u16 (0..0xFFFF, the field's range: the original indexes FX_SinCosTable_ with it and reads past the
# table for larger values). Position components and area fields are drawn from shared small ranges so the inside / outside tests
# meet; the FX answers the comparisons and roundings depend on are drawn from the same values (VEC_Mag / FX_Atan2Idx answers on the
# ring bounds, FX_Div answers 0x800 / 0x1800 so the +0x800 rounding of the fx32 products matters). Stack locals reach stubs as
# `in:` / `out:` buffers. Closure stubs as core4 / F5b (checked with every function treated as ported: no fault, stray write or
# unstubbed callee).
area scenearea_loader
set field

# 021AB7E4(heap) -> L: Heap_AllocDebug(heap, 0x10, 1, "fld_scenearea_loader.c", 0x81); L+8+4i = 020490F4(ARC[i], heap) (archive
# open; ARC = ROM 0x021D8EEC {0x50, 0x9E}), i = 0, 1. Arena: arg 1 (a stack word the function never reads).
fn 021ab7e4 ret=u32 inputs=60
  nowork
  arg u32 alt range 0 4 || any
  arg buf 0x20
  stub 02030734 force ret=ptr:arg1:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap)

# 021AB81C(L): 021AB868(L) (free the list); 02049238(L+8+4i) (archive close), i = 0, 1; Heap_Free(L).
fn 021ab81c ret=none inputs=60
  nowork
  arg buf 0x20
  fix arg0 0 u32 alt const 0 || any
  stub 021ab868 ret=none args=ptr  # in-area: free the area list (L)
  stub 02049238 force ret=none args=v  # archive close (handle)
  stub 020307b0 force ret=none args=v  # Heap_Free

# 021AB840(L, i, member, heap): L+0 = 0204A39C(archive i, member, 0, heap, &size) (load the member on the heap); L+4 = size / 0x48
# (unsigned divide 0209C2B0). The heap is the caller's r3, passed on untouched.
fn 021ab840 ret=none inputs=200
  nowork
  arg buf 0x100
  arg u32 alt range 0 1 || range 0 0x3C
  arg u32 any
  arg u32 any
  stub 0204a39c force ret=u32 args=v,v,v,v,out:4  # archive member load (handle, member, 0, heap, &size)

# 021AB868(L): L+0 != 0 -> Heap_Free(L+0), L+0 = L+4 = 0.
fn 021ab868 ret=none inputs=60
  nowork
  arg buf 0x20
  fix arg0 0 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v  # Heap_Free

# 021AB880(L): +0 (the area list).
fn 021ab880 ret=u32 inputs=20
  nowork
  arg buf 0x20

# 021AB884(L): +4 (the area count).
fn 021ab884 ret=u32 inputs=20
  nowork
  arg buf 0x20

# 021AB888(): ROM 0x021D8EF4 (the callback table descriptor).
fn 021ab888 ret=u32 inputs=10
  nowork

# 021AB890(SA, area, pos) (check: ring sector): d = pos - centre (0207CA00), d.y = 0; len = |d| (0207CBD0); normalise d (0207CCA4);
# ang = FX_Atan2Idx(d.x, d.z) (0207CFB8). Inside when r0 <= len < r1 (signed) and ang in [a0, a1) (a1 >= a0) or ang <= a1 / a0 <= ang
# < 0x10000 (a1 < a0); answers 1 / 0.
fn 021ab890 ret=u32 inputs=600
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0 u32 alt oneof 0 0x4000 0x8000 0xC000 0xFFFF || range 0 0x10000 || any
  fix arg1 4 u32 alt oneof 0 0x4000 0x8000 0xC000 0xFFFF || range 0 0x10000 || any
  fix arg1 8 u32 alt oneof 0 0x1000 0x8000 0x10000 || range 0 0x80000000 || any
  fix arg1 0xC u32 alt oneof 0 0x1000 0x8000 0x10000 || range 0 0x80000000 || any
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract(a, b, out) (FX)
  stub 0207cbd0 force ret=pick+u32:0/0x1000/0x8000/0x10000 args=in:12  # VEC_Mag(v) (FX)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize(v, out) (FX)
  stub 0207cfb8 force ret=pick+u16:0/0x4000/0x8000/0xC000/0xFFFF args=v,v  # FX_Atan2Idx(y, x) (FX)

# 021AB908(SA, area, pos) (update: circle camera): cam = 021AB760(SA); 0218E124(cam, 2); target T = centre; d = pos - (centre with y =
# 0), d.y = 0, normalised into n (0207CA00 / 0207CCA4); i = angle >> 4: h = (sin(i) * dist + 0x800) >> 12, r = (cos(i) * dist +
# 0x800) >> 12 (64-bit products); camera position = {n.x * r, h, n.z * r} (fx32 products, rounded) + T; 0218EC90(cam, &T);
# 0218ECF4(cam, &position).
fn 021ab908 ret=none inputs=400
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0x1C u32 alt range 0 0xFFFF || oneof 0 0x4000 0x8000 0xC000 0xFFFF
  fix arg1 0x20 u32 alt range 0 0x200000 || any
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract(a, b, out) (FX)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize(v, out) (FX)
  stub 0218ec90 force ret=none args=v,in:12  # field_camera (G8): target (cam, &vec)
  stub 0218ecf4 force ret=none args=v,in:12  # field_camera (G8): position (cam, &vec)

# 021AB9FC(SA, area, pos) (update: fixed camera): cam = 021AB760(SA); 0218E124(cam, 2); 0218EC90(cam, &area+0x24 (target, copied));
# 0218ECF4(cam, &area+0x30 (position, copied)).
fn 021ab9fc ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 0218ec90 force ret=none args=v,in:12  # field_camera (G8): target (cam, &vec)
  stub 0218ecf4 force ret=none args=v,in:12  # field_camera (G8): position (cam, &vec)

# 021ABA3C(SA, area, pos) (check: grid rectangle): 02188CFC(021AB764(SA)) == 1 (a rail map) -> 0. Else gx = (u16)(((pos.x - 0x8000)
# >> 12) / 16) and gz likewise (signed division, rounding toward zero); w / h = (+8, +6) when +4 == 1, else (+4, +8); inside when x
# <= gx < x + w and z <= gz < z + h (answers 1 / 0).
fn 021aba3c ret=u32 inputs=600
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0 u16 alt range 0 0x30 || any
  fix arg1 2 u16 alt range 0 0x30 || any
  fix arg1 4 u16 alt const 1 || range 0 0x20 || any
  fix arg1 6 u16 alt range 0 0x20 || any
  fix arg1 8 u16 alt range 0 0x20 || any
  fix arg2 0 u32 alt range 0 0x400000 || any
  fix arg2 8 u32 alt range 0 0x400000 || any
  stub 021ab764 force ret=u32 args=ptr  # scenearea: SA+4 (the fieldmap)
  stub 02188cfc force ret=pick+u32:0/1 args=v  # fieldmap: map kind (1 rail)

# 021ABABC(SA, area, pos) (update: interpolated camera): cam = 021AB760(SA); 0218E124(cam, 0); 021ABBA0(area, pos, &len, &t); f =
# FX_Div(t, len) (0207C768); angle 1 = +0xC + (+0x14 - +0xC) * f / 4096, angle 2 = +0xE + (+0x16 - +0xE) * f / 4096 (32-bit
# products, signed division rounding toward zero); distance = +0x10 + ((+0x18 - +0x10) * f + 0x800) >> 12 (64-bit); 0218ED34(cam,
# angle 1); 0218ED48(cam, angle 2); 0218ED58(cam, distance).
fn 021ababc ret=none inputs=300
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ed34 force ret=none args=v,v  # field_camera (G8): angle 1 (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field_camera (G8): angle 2 (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field_camera (G8): distance (cam, fx32)

# 021ABB54(SA, area, pos): cam = 021AB760(SA); +0x1C != 0 -> 0218F0D4(cam); 021ABABC(SA, area, pos).
fn 021abb54 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0x1C u32 alt const 0 || any
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 0218f0d4 force ret=none args=v  # field_camera (G8)
  stub 021ababc ret=none args=v,ptr,ptr  # in-area: interpolated camera (SA, area, pos)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ed34 force ret=none args=v,v  # field_camera (G8): angle 1 (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field_camera (G8): angle 2 (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field_camera (G8): distance (cam, fx32)

# 021ABB78(SA, area, pos): cam = 021AB760(SA); 021ABABC(SA, area, pos); +0x1C != 0 -> 0218F05C(cam).
fn 021abb78 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0x1C u32 alt const 0 || any
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 021ababc ret=none args=v,ptr,ptr  # in-area: interpolated camera (SA, area, pos)
  stub 0218f05c force ret=none args=v  # field_camera (G8)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ed34 force ret=none args=v,v  # field_camera (G8): angle 1 (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field_camera (G8): angle 2 (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field_camera (G8): distance (cam, fx32)

# 021ABBA0(area, pos, &len, &t): p = *pos with x, z -= 0x8000; *len = +8 << 16; *t = p.x - (+0 << 16) when +4 == 1, else p.z - (+2
# << 16).
fn 021abba0 ret=none inputs=200
  nowork
  arg buf 0x48
  arg buf 0x10
  arg buf 0x10
  arg buf 0x10
  fix arg0 4 u16 alt const 1 || range 0 2 || any

# 021ABBF0(SA, area, pos): 021ABABC, 021ABC5C, 021ABD08 (SA, area, pos).
fn 021abbf0 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  stub 021ababc ret=none args=v,ptr,ptr  # in-area: interpolated camera (SA, area, pos)
  stub 021abc5c ret=none args=v,ptr,ptr  # in-area: target offset (SA, area, pos)
  stub 021abd08 ret=none args=v,ptr,ptr  # in-area: field of view (SA, area, pos)
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ed34 force ret=none args=v,v  # field_camera (G8): angle 1 (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field_camera (G8): angle 2 (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field_camera (G8): distance (cam, fx32)
  stub 0218ecd4 force ret=none args=v,in:12  # field_camera (G8): target offset (cam, &vec)
  stub 0218ed64 force ret=none args=v,v  # field_camera (G8): field of view (cam, u16)

# 021ABC14(SA, area, pos): 021ABB54, 021ABC5C, 021ABD08 (SA, area, pos).
fn 021abc14 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0x1C u32 alt const 0 || any
  stub 021abb54 ret=none args=v,ptr,ptr  # in-area (SA, area, pos)
  stub 021abc5c ret=none args=v,ptr,ptr  # in-area: target offset (SA, area, pos)
  stub 021abd08 ret=none args=v,ptr,ptr  # in-area: field of view (SA, area, pos)
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 0218f0d4 force ret=none args=v  # field_camera (G8)
  stub 021ababc ret=none args=v,ptr,ptr  # in-area: interpolated camera (SA, area, pos)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ecd4 force ret=none args=v,in:12  # field_camera (G8): target offset (cam, &vec)
  stub 0218ed64 force ret=none args=v,v  # field_camera (G8): field of view (cam, u16)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 0218ed34 force ret=none args=v,v  # field_camera (G8): angle 1 (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field_camera (G8): angle 2 (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field_camera (G8): distance (cam, fx32)

# 021ABC38(SA, area, pos): 021ABB78, 021ABC5C, 021ABD08 (SA, area, pos).
fn 021abc38 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  fix arg1 0x1C u32 alt const 0 || any
  stub 021abb78 ret=none args=v,ptr,ptr  # in-area (SA, area, pos)
  stub 021abc5c ret=none args=v,ptr,ptr  # in-area: target offset (SA, area, pos)
  stub 021abd08 ret=none args=v,ptr,ptr  # in-area: field of view (SA, area, pos)
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 021ababc ret=none args=v,ptr,ptr  # in-area: interpolated camera (SA, area, pos)
  stub 0218f05c force ret=none args=v  # field_camera (G8)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ecd4 force ret=none args=v,in:12  # field_camera (G8): target offset (cam, &vec)
  stub 0218ed64 force ret=none args=v,v  # field_camera (G8): field of view (cam, u16)
  stub 0218e124 force ret=u32 args=v,v  # field_camera (G8): the camera, set its mode (cam, mode)
  stub 0218ed34 force ret=none args=v,v  # field_camera (G8): angle 1 (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field_camera (G8): angle 2 (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field_camera (G8): distance (cam, fx32)

# 021ABC5C(SA, area, pos) (update: target offset): cam = 021AB760(SA); 021ABBA0(area, pos, &len, &t); offset k = start k + FX_Div(((end k
# - start k) * t + 0x800) >> 12, len) for k = x, y, z (start +0x20, end +0x2C; 64-bit products); 0218ECD4(cam, &offset).
fn 021abc5c ret=none inputs=300
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ecd4 force ret=none args=v,in:12  # field_camera (G8): target offset (cam, &vec)

# 021ABD08(SA, area, pos) (update: field of view): cam = 021AB760(SA); 021ABBA0(area, pos, &len, &t); f = FX_Div(t, len); 0218ED64(cam,
# (u16)(+0x38 + (+0x3A - +0x38) * f / 4096)) (32-bit product, signed division rounding toward zero).
fn 021abd08 ret=none inputs=300
  nowork
  arg u32 any
  arg buf 0x48
  arg buf 0x10
  stub 021ab760 force ret=u32 args=ptr  # scenearea: SA+0 (the field system)
  stub 021abba0 ret=none args=ptr,ptr,out:4,out:4  # in-area: rect position (area, pos, &len, &t)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1800/0xFFFFF800 args=v,v  # FX_Div(a, b) (FX)
  stub 0218ed64 force ret=none args=v,v  # field_camera (G8): field of view (cam, u16)
