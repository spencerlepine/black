# fndiff spec (F11): field_light.c (overlay 21 Thumb code 0x021A03D0..0x021A0CF4, F1 row 21.15; 29 functions, all specced): the
# time-of-day light of the field (4 directional lights with colours, the material colours, the fog and clear colours) from a keyframe
# table, cross-fades between keyframes, fade-to-colour requests, and the upload of the current light set.
# File bound: `Heap_AllocDebug(heap, 0x490, 1, "field_light.c", 0x167)` in 021A03D0 (string 0x021DDE38); field_zonefog.c ends at
# 0x021A03D0, field_subscreen.c starts at 0x021A0CF4. Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-field field_light --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry (29).
# 021A0734 (`strh; ldr r3, =0x021A0749; movs r1, #0x3D; bx r3`) and 021A065C (`adds r0, #0x484; bx 0x021A0CE5`) are thunks with
# their own work; 021A07B0 is an empty function (`bx lr`). Fragments: 021A0C96 / 021A0C98 (case labels inside 021A0C74).
#
# LIGHT (0x490, 021A03D0): +0 u16 mode (0 = follow the clock, 1 = a fade request runs), +2 u16 season (first argument of the time-zone
# table 02011400), +4 the G3D global state (the light upload target), +8 the FIELD FOG, +0xC count of keyframes, +0x10 the keyframes
# (0x34 bytes each, up to 16), +0x350 current keyframe, +0x354 u16 member of ARC 0x3D of the default table, +0x356 the current light
# (0x34, a keyframe), +0x38A u16 upload enabled, +0x38C dirty (upload pending), +0x390 the interpolator (0xF4), +0x484 the fade request
# (8: u16 state, u16 frames out, u16 frames back (0xFFFF none), u16 colour), +0x48C the last clock value (seconds of the day).
# KEYFRAME (0x34): +0 u16 time zone, +2 s16 minutes, +4 u8 light on[4], +8 u16 light colour[4] (RGB555), +0x10 VecFx16 light
# direction[4], +0x28 u16 diffuse, +0x2A ambient, +0x2C specular, +0x2E emission, +0x30 fog colour, +0x32 clear colour. Its time
# (021A07F8): 02011400(season, zone) (the hour) * 3600 + minutes * 60.
# INTERPOLATOR (0xF4, 021A0980): +0 u16 / +2 s16 the target's header, +4 u8 on[4], +8 4 x 12-byte colour ramps, +0x38 4 x 0x1C
# rotations {VecFx32 start, VecFx32 axis (cross product), u16 angle (acos of the dot product)}, +0xA8 6 x 12-byte colour ramps, +0xF0
# u16 frame, +0xF2 u16 frames. A colour ramp (021A0818): s16 {r0, r1 - r0, g0, g1 - g0, b0, b1 - b0}; a value at frame f of n
# (021A0860): c0 + (c1 - c0) * f / n per channel (signed division). A direction at frame f (021A0918): the start vector rotated about
# the axis by angle * f / n (FX_SinCosTable_ at main 0x020A1B38, read as ROM data; the port computes it) and normalised.
#
# The FX vector / matrix routines of the ARM9 library (0207CB68 cross, 0207CAA4 dot, 0207D184 acos index, 0207B65C rotation matrix,
# 0207BC08 matrix x vector, 0207CCA4 / 0207CDB8 normalise) and the G3 / hardware upload (02048CA4 light vector, 02048CDC light colour,
# 0206AA8C diffuse + ambient, 0206AAB0 specular + emission, 0207DD24 clear colour, field_fog 021A01C4 fog colour) are `force` stubs
# (D3: the port reaches them through FieldEnv by address); division (0209C2B0 / 0209C0A4) and MI_CpuCopy8 run as real code.
# All blocks are `nowork`; objects are `arg buf` arenas. Closure stubs (as core4 / F5b): a block lists the stubs of its callees and of
# the callees of its in-area descendants (checked with every function treated as ported: no fault, stray write or unstubbed callee).
# LIGHT fields are bounded: count 0..16, current keyframe 0..15, interpolator frame / frames 0..4 or any.
area field_light
set field

# 021A03D0(member, now, season, fog, g3d, heap) -> LIGHT: L = Heap_AllocDebug(heap, 0x490, 1, "field_light.c", 0x167); L+2 = season;
# 021A0734(L, member) (load ARC 0x3D member); L+0x38A = 1; L+0x350 = k = 021A07B4(L, now); the keyframe k copied to L+0x356 (0x34);
# L+0x38C = 1; L+8 = fog; L+4 = g3d. Arena: arg 6 (a stack word the function never reads).
fn 021a03d0 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 8 || any
  arg u32 alt range 0 86400 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x500
  stub 02030734 force ret=ptr:arg6:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 021a0734 ret=none args=v,v  # in-area: load the default table (L, member)
  stub 021a07b4 ret=pick:0/1/2/15 args=v,v  # in-area: keyframe of a time (L, now)
  stub 021a0748 ret=none args=v,v,v  # in-area: load a keyframe table (L, arc, member)
  stub 021a07f8 ret=pick+u32:0/3600/86400 args=v,v  # in-area: keyframe time (season, keyframe)
  stub 020490a8 force ret=pick:0/0x34/0x68/0x9C/0x340/0x33/0x35 args=v,v  # archive member size (arc, member): keyframe count 0..16
  stub 02048f4c force ret=none args=out:0x340,v,v  # archive member load (dst, arc, member)
  stub 0207cdb8 force ret=none args=in:6,out:6  # VEC_Fx16Normalize(src, dst)
  stub 02011400 force ret=pick+u32:0/1/6/12/23/24 args=v,v  # the hour of time zone z in season s (ROM table, main)

# 021A0448(L): 021A07B0() (empty); Heap_Free(L).
fn 021a0448 ret=none inputs=40
  nowork
  arg buf 0x10
  stub 021a07b0 ret=none  # in-area: empty
  stub 020307b0 force ret=none args=v  # Heap_Free

# 021A0458(L, now): count 0 -> nothing. Mode 0: t = time(k), p = time(k - 1) (k = 0: time(count - 1) when <= now, then t += 86400 and no
# step, else p = 0); unless p <= now < t (or the wrap) k = (k + 1) % count and the cross-fade 021A0974 to keyframe k starts. Mode 1:
# 021A0C74(request, interpolator, current, keyframe k) answers 1 -> mode 0. Then a running interpolator steps (021A0B14, 021A0B4C into
# the current light, dirty = 1); L+0x48C = now.
fn 021a0458 ret=none inputs=1500
  nowork
  arg buf 0x500
  arg u32 alt oneof 0 3600 86400 || oneof 1 3599 3601 86399 || range 0 90000 || any
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 0xC u32 alt range 1 16 || const 0
  fix arg0 0x350 u32 alt const 0 || range 0 15
  fix arg0 0x38A u16 alt const 0 || any
  fix arg0 0x38C u32 alt const 0 || any
  fix arg0 0x480 u16 alt range 0 4 || any
  fix arg0 0x482 u16 alt range 0 4 || any
  fix arg0 0x484 u16 alt range 0 4 || any
  fix arg0 0x488 u16 alt const 0xFFFF || any
  fix arg0 0x12 s16 alt range 0 59 || any rep=16,0x34
  fix arg0 0 u16 alt const 0 || const 1 || any
  stub 021a07f8 ret=pick+u32:0/3600/86400 args=v,v  # in-area: keyframe time (season, keyframe)
  stub 021a0974 ret=none args=v,v,v  # in-area: cross-fade, 60 frames (I, cur, target)
  stub 021a0c74 ret=pick:0/1 args=v,v,v,v  # in-area: fade request step (req, I, cur, target)
  stub 021a0b34 ret=pick:0/1 args=v  # in-area: interpolator running
  stub 021a0b14 ret=none args=v  # in-area: frame + 1
  stub 021a0b4c ret=none args=v,v  # in-area: light of the frame (I, out)
  stub 02011400 force ret=pick+u32:0/1/6/12/23/24 args=v,v  # the hour of time zone z in season s (ROM table, main)
  stub 021a0980 ret=none args=v,v,v,v  # in-area: cross-fade (I, cur, target, frames)
  stub 021a0a50 ret=none args=v,v,v,v  # in-area: fade to a colour (I, cur, colour, frames)
  stub 021a0860 ret=u16 args=v,v,v  # in-area: colour at a frame (ramp, f, n)
  stub 021a0918 ret=none args=v,v,v,v  # in-area: direction at a frame (rot, f, n, out)
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 0207b65c force ret=none args=out:0x24,in:12,v,v  # MTX_RotAxis33(out, axis, sin, cos)
  stub 0207bc08 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(v, m, out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize(src, dst)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0568(L, force): L+0x38C != 0 or force -> 021A066C(L, L+8, L+4) (upload), L+0x38C = 0.
fn 021a0568 ret=none inputs=100
  nowork
  arg buf 0x500
  arg u32 alt const 0 || any
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 0xC u32 alt range 1 16 || const 0
  fix arg0 0x350 u32 alt const 0 || range 0 15
  fix arg0 0x38A u16 alt const 0 || any
  fix arg0 0x38C u32 alt const 0 || any
  fix arg0 0x480 u16 alt range 0 4 || any
  fix arg0 0x482 u16 alt range 0 4 || any
  fix arg0 0x484 u16 alt range 0 4 || any
  fix arg0 0x488 u16 alt const 0xFFFF || any
  fix arg0 0x12 s16 alt range 0 59 || any rep=16,0x34
  stub 021a066c ret=none args=v,v,v  # in-area: upload (L, fog, g3d)
  stub 02048ca4 force ret=none args=v,v,in:6  # G3D global state: light vector (g3d, i, &VecFx16)
  stub 02048cdc force ret=none args=v,v,in:2  # G3D global state: light colour (g3d, i, &colour)
  stub 0206aa8c force ret=none args=v,v,v  # G3_MaterialColorDiffAmb(diffuse, ambient, set vertex colour)
  stub 0206aab0 force ret=none args=v,v,v  # G3_MaterialColorSpecEmi(specular, emission, shininess table)
  stub 021a01c4 force ret=none args=v,v  # field_fog (G8): fog colour (fog, colour)
  stub 0207dd24 force ret=none args=v,v,v,v,v  # G3X_SetClearColor(colour, alpha, depth, polygon id, fog)

# 021A0590(L, arc, member, frames): 021A07B0(); 021A0748(L, arc, member) (load); k = 021A07B4(L, L+0x48C) -> L+0x350; 021A0980(L+0x390,
# L+0x356, keyframe k, frames) (cross-fade from the current light).
fn 021a0590 ret=none inputs=100
  nowork
  arg buf 0x500
  arg u32 any
  arg u32 any
  arg u32 alt range 0 60 || any
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 0xC u32 alt range 1 16 || const 0
  fix arg0 0x350 u32 alt const 0 || range 0 15
  fix arg0 0x38A u16 alt const 0 || any
  fix arg0 0x38C u32 alt const 0 || any
  fix arg0 0x480 u16 alt range 0 4 || any
  fix arg0 0x482 u16 alt range 0 4 || any
  fix arg0 0x484 u16 alt range 0 4 || any
  fix arg0 0x488 u16 alt const 0xFFFF || any
  fix arg0 0x12 s16 alt range 0 59 || any rep=16,0x34
  stub 021a07b0 ret=none  # in-area: empty
  stub 021a0748 ret=none args=v,v,v  # in-area: load a keyframe table (L, arc, member)
  stub 021a07b4 ret=pick:0/1/2/15 args=v,v  # in-area: keyframe of a time (L, now)
  stub 021a0980 ret=none args=v,v,v,v  # in-area: cross-fade (I, cur, target, frames)
  stub 020490a8 force ret=pick:0/0x34/0x68/0x9C/0x340/0x33/0x35 args=v,v  # archive member size (arc, member): keyframe count 0..16
  stub 02048f4c force ret=none args=out:0x340,v,v  # archive member load (dst, arc, member)
  stub 0207cdb8 force ret=none args=in:6,out:6  # VEC_Fx16Normalize(src, dst)
  stub 021a07f8 ret=pick+u32:0/3600/86400 args=v,v  # in-area: keyframe time (season, keyframe)
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 02011400 force ret=pick+u32:0/1/6/12/23/24 args=v,v  # the hour of time zone z in season s (ROM table, main)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A05D8(L): 021A07B0(); 021A0734(L, L+0x354) (reload the default table); k = 021A07B4(L, L+0x48C) -> L+0x350; 021A0974(L+0x390,
# L+0x356, keyframe k) (60 frames).
fn 021a05d8 ret=none inputs=100
  nowork
  arg buf 0x500
  fix arg0 0 u16 alt range 0 2 || any
  fix arg0 0xC u32 alt range 1 16 || const 0
  fix arg0 0x350 u32 alt const 0 || range 0 15
  fix arg0 0x38A u16 alt const 0 || any
  fix arg0 0x38C u32 alt const 0 || any
  fix arg0 0x480 u16 alt range 0 4 || any
  fix arg0 0x482 u16 alt range 0 4 || any
  fix arg0 0x484 u16 alt range 0 4 || any
  fix arg0 0x488 u16 alt const 0xFFFF || any
  fix arg0 0x12 s16 alt range 0 59 || any rep=16,0x34
  stub 021a07b0 ret=none  # in-area: empty
  stub 021a0734 ret=none args=v,v  # in-area: load the default table (L, member)
  stub 021a07b4 ret=pick:0/1/2/15 args=v,v  # in-area: keyframe of a time (L, now)
  stub 021a0974 ret=none args=v,v,v  # in-area: cross-fade, 60 frames (I, cur, target)
  stub 021a0748 ret=none args=v,v,v  # in-area: load a keyframe table (L, arc, member)
  stub 021a07f8 ret=pick+u32:0/3600/86400 args=v,v  # in-area: keyframe time (season, keyframe)
  stub 021a0980 ret=none args=v,v,v,v  # in-area: cross-fade (I, cur, target, frames)
  stub 020490a8 force ret=pick:0/0x34/0x68/0x9C/0x340/0x33/0x35 args=v,v  # archive member size (arc, member): keyframe count 0..16
  stub 02048f4c force ret=none args=out:0x340,v,v  # archive member load (dst, arc, member)
  stub 0207cdb8 force ret=none args=in:6,out:6  # VEC_Fx16Normalize(src, dst)
  stub 02011400 force ret=pick+u32:0/1/6/12/23/24 args=v,v  # the hour of time zone z in season s (ROM table, main)
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0618(L, back, out, colour): 021A0C50(L+0x484, (u16) out, (u16) colour, back) (fade request with a way back); L+0 = 1.
fn 021a0618 ret=none inputs=60
  nowork
  arg buf 0x500
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021a0c50 ret=none args=v,v,v,v  # in-area: request with a way back

# 021A063C(L, colour, out): 021A0C60(L+0x484, (u16) out, colour) (fade request, no way back); L+0 = 1.
fn 021a063c ret=none inputs=60
  nowork
  arg buf 0x500
  arg u32 any
  arg u32 any
  stub 021a0c60 ret=none args=v,v,v  # in-area: request

# 021A065C(L): thunk to 021A0CE4(L + 0x484) (a fade request is set).
fn 021a065c ret=u32 inputs=60
  nowork
  arg buf 0x500
  fix arg0 0x484 u16 alt const 0 || any
  stub 021a0ce4 ret=pick:0/1 args=v  # in-area: request set

# 021A066C(L, fog, g3d): L+0x38A == 0 -> nothing; else per light i: on (L+0x35A+i) -> 02048CA4(g3d, i, &dir i), 02048CDC(g3d, i,
# &colour i) of the current light, off -> a zero vector / colour (stack locals); 0206AA8C(diffuse, ambient, 1), 0206AAB0(specular,
# emission, 0), 021A01C4(fog, fog colour), 0207DD24(clear colour, 31, 0x7FFF, 0, 0).
fn 021a066c ret=none inputs=100
  nowork
  arg buf 0x500
  arg u32 any
  arg u32 any
  fix arg0 0x38A u16 alt const 0 || any
  fix arg0 0x35A u8 alt const 0 || any rep=4,1
  stub 02048ca4 force ret=none args=v,v,in:6  # G3D global state: light vector (g3d, i, &VecFx16)
  stub 02048cdc force ret=none args=v,v,in:2  # G3D global state: light colour (g3d, i, &colour)
  stub 0206aa8c force ret=none args=v,v,v  # G3_MaterialColorDiffAmb(diffuse, ambient, set vertex colour)
  stub 0206aab0 force ret=none args=v,v,v  # G3_MaterialColorSpecEmi(specular, emission, shininess table)
  stub 021a01c4 force ret=none args=v,v  # field_fog (G8): fog colour (fog, colour)
  stub 0207dd24 force ret=none args=v,v,v,v,v  # G3X_SetClearColor(colour, alpha, depth, polygon id, fog)

# 021A0734(L, member): L+0x354 = (u16) member; tail call 021A0748(L, 0x3D, member).
fn 021a0734 ret=none inputs=60
  nowork
  arg buf 0x500
  arg u32 any
  stub 021a0748 ret=none args=v,v,v  # in-area: load a keyframe table (L, arc, member)
  stub 020490a8 force ret=pick:0/0x34/0x68/0x9C/0x340/0x33/0x35 args=v,v  # archive member size (arc, member): keyframe count 0..16
  stub 02048f4c force ret=none args=out:0x340,v,v  # archive member load (dst, arc, member)
  stub 0207cdb8 force ret=none args=in:6,out:6  # VEC_Fx16Normalize(src, dst)

# 021A0748(L, arc, member): L+0xC = 020490A8(arc, member) / 0x34 (file size); 02048F4C(L+0x10, arc, member) (load the file); every
# light direction of the L+0xC keyframes normalised in place (0207CDB8(v, v)).
fn 021a0748 ret=none inputs=100
  nowork
  arg buf 0x500
  arg u32 any
  arg u32 any
  stub 020490a8 force ret=pick:0/0x34/0x68/0x9C/0x340/0x33/0x35 args=v,v  # archive member size (arc, member): keyframe count 0..16
  stub 02048f4c force ret=none args=out:0x340,v,v  # archive member load (dst, arc, member)
  stub 0207cdb8 force ret=none args=in:6,out:6  # VEC_Fx16Normalize(src, dst)

# 021A07B0(): empty.
fn 021a07b0 ret=none inputs=10
  nowork

# 021A07B4(L, now): the first keyframe k (of L+0xC) with time(k - 1) <= now < time(k) (time(-1) = 0, signed compares), else 0.
fn 021a07b4 ret=u32 inputs=300
  nowork
  arg buf 0x500
  arg u32 alt oneof 0 1 3599 3600 3601 43200 86399 || range 0 90000 || any
  fix arg0 0xC u32 alt range 1 16 || const 0
  fix arg0 0x10 u16 alt range 0 4 || any rep=16,0x34
  fix arg0 0x12 s16 alt range 0 59 || any rep=16,0x34
  stub 021a07f8 ret=pick+u32:0/3600/86400 args=v,v  # in-area: keyframe time (season, keyframe)
  stub 02011400 force ret=pick+u32:0/1/6/12/23/24 args=v,v  # the hour of time zone z in season s (ROM table, main)

# 021A07F8(season, keyframe): 02011400(season, kf+0) * 3600 + (s16) kf+2 * 60.
fn 021a07f8 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 3 || any
  arg buf 0x10
  stub 02011400 force ret=pick+u32:0/1/6/12/23/24 args=v,v  # the hour of time zone z in season s (ROM table, main)

# 021A0818(ramp, c0, c1): RGB555 ramp {r0, r1 - r0, g0, g1 - g0, b0, b1 - b0} (u16 stores).
fn 021a0818 ret=none inputs=100
  nowork
  arg buf 0x10
  arg u32 alt range 0 0x7FFF || any
  arg u32 alt range 0 0x7FFF || any

# 021A0860(ramp, f, n): RGB555 of c0 + (c1 - c0) * f / n per channel (s16 sums, signed division; n = 0 gives the dividend).
fn 021a0860 ret=u16 inputs=200
  nowork
  arg buf 0x10
  arg u32 alt range 0 64 || any
  arg u32 alt range 0 64 || oneof 0 0xFFFFFFFF || any

# 021A08CC(rot, a, b): rot+0.. = a (VecFx16 -> VecFx32), rot+0xC.. = 0207CB68(a, b) (cross product), rot+0x18 = (u16)
# 0207D184(0207CAA4(a, b)) (the angle).
fn 021a08cc ret=none inputs=100
  nowork
  arg buf 0x20
  arg buf 0x8
  arg buf 0x8
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0918(rot, f, n, out): a = (u16)(rot.angle * f / n) >> 4 (signed division); 0207B65C(&m, &rot.axis, sin a, cos a); 0207BC08(&rot.start,
# &m, &v); 0207CCA4(&v, &v); out = (s16) v.
fn 021a0918 ret=none inputs=200
  nowork
  arg buf 0x20
  arg u32 alt range 0 64 || any
  arg u32 alt range 0 64 || oneof 0 0xFFFFFFFF || any
  arg buf 0x8
  stub 0207b65c force ret=none args=out:0x24,in:12,v,v  # MTX_RotAxis33(out, axis, sin, cos)
  stub 0207bc08 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(v, m, out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize(src, dst)

# 021A0974(I, cur, target): 021A0980(I, cur, target, 60).
fn 021a0974 ret=none inputs=60
  nowork
  arg buf 0x100
  arg buf 0x40
  arg buf 0x40
  stub 021a0980 ret=none args=v,v,v,v  # in-area: cross-fade (I, cur, target, frames)
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0980(I, cur, target, n): I+0xF0 = 0, I+0xF2 = n; header from the target; colour ramps cur -> target, rotations of the directions
# (021A08CC), on flags from the target.
fn 021a0980 ret=none inputs=100
  nowork
  arg buf 0x100
  arg buf 0x40
  arg buf 0x40
  arg u32 alt range 0 60 || any
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0A50(I, cur, colour, n): as 021A0980 towards one colour for every ramp, directions kept (rotation of cur to itself), header and
# on flags from cur.
fn 021a0a50 ret=none inputs=100
  nowork
  arg buf 0x100
  arg buf 0x40
  arg u32 alt range 0 0x7FFF || any
  arg u32 alt range 0 60 || any
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0B14(I): I+0xF0 < I+0xF2 -> I+0xF0 + 1.
fn 021a0b14 ret=none inputs=60
  nowork
  arg buf 0x100
  fix arg0 0xF0 u16 alt range 0 4 || any
  fix arg0 0xF2 u16 alt range 0 4 || any

# 021A0B34(I): I+0xF0 < I+0xF2 (running).
fn 021a0b34 ret=u32 inputs=60
  nowork
  arg buf 0x100
  fix arg0 0xF0 u16 alt range 0 4 || any
  fix arg0 0xF2 u16 alt range 0 4 || any

# 021A0B4C(I, out): the light at frame I+0xF0 of I+0xF2: header, colours (021A0860), directions (021A0918), on flags.
fn 021a0b4c ret=none inputs=100
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0xF0 u16 alt range 0 4 || any
  fix arg0 0xF2 u16 alt range 0 4 || any
  stub 021a0860 ret=u16 args=v,v,v  # in-area: colour at a frame (ramp, f, n)
  stub 021a0918 ret=none args=v,v,v,v  # in-area: direction at a frame (rot, f, n, out)
  stub 0207b65c force ret=none args=out:0x24,in:12,v,v  # MTX_RotAxis33(out, axis, sin, cos)
  stub 0207bc08 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(v, m, out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize(src, dst)

# 021A0C50(req, out, back, colour): req = {1, out, back, colour} (u16).
fn 021a0c50 ret=none inputs=40
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any

# 021A0C60(req, out, colour): req = {1, out, 0xFFFF, colour} (u16).
fn 021a0c60 ret=none inputs=40
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 any

# 021A0C74(req, I, cur, target): by req+0: 0 -> 1; 1 -> 021A0A50(I, cur, colour, out), state 2; 2 -> while running 0, then (back !=
# 0xFFFF) 021A0980(I, cur, target, back), state 3; 3 -> when done state 0; else 0. Answers 0 except in state 0.
fn 021a0c74 ret=u32 inputs=300
  nowork
  arg buf 0x10
  arg buf 0x100
  arg buf 0x40
  arg buf 0x40
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 4 u16 alt const 0xFFFF || any
  fix arg1 0xF0 u16 alt range 0 4 || any
  fix arg1 0xF2 u16 alt range 0 4 || any
  stub 021a0a50 ret=none args=v,v,v,v  # in-area: fade to a colour (I, cur, colour, frames)
  stub 021a0b34 ret=pick:0/1 args=v  # in-area: interpolator running
  stub 021a0980 ret=none args=v,v,v,v  # in-area: cross-fade (I, cur, target, frames)
  stub 021a0818 ret=none args=v,v,v  # in-area: colour ramp (ramp, c0, c1)
  stub 021a08cc ret=none args=v,v,v  # in-area: rotation (rot, a, b)
  stub 0207cb68 force ret=none args=in:6,in:6,out:6  # VEC_Fx16CrossProduct(a, b, out)
  stub 0207caa4 force ret=u32 args=in:6,in:6  # VEC_Fx16DotProduct(a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx(x)

# 021A0CE4(req): req+0 != 0.
fn 021a0ce4 ret=u32 inputs=40
  nowork
  arg buf 0x10
  fix arg0 0 u16 alt const 0 || any
