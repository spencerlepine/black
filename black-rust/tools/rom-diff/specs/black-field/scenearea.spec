# fndiff spec (F2b): fld_scenearea.c (overlay 21 Thumb code 0x021AB5F4..0x021AB7E4, the first part of F1 row 21.31; 12 functions,
# all specced): the SCENE AREA system of a map. A map has a list of scene areas (0x48 bytes each, loaded by fld_scenearea_loader.c,
# area `scenearea_loader`); every frame the player position is tested against each area with the area's CHECK callback, and when the
# area the player is in changes, the EXIT callback of the old area and the ENTER callback of the new one run (else the UPDATE callback
# of the current one). The callbacks (camera set-ups of the area) come from a callback table (`021AB888` answers the ROM table
# 0x021D8EF4 = {check table 0x021DDFD8 (2 entries), update table 0x021DDFE0 (8 entries), u16 2, u16 8}).
# File bound: `Heap_AllocDebug(heap, 0x20, 1, "fld_scenearea.c", 0x51)` in 021AB5F4 (string 0x021DDFC8); effect_encount.c ends at
# 0x021AB5F4 (its last function 021AB5AC); fld_scenearea_loader.c starts at 021AB7E4 (its allocation names "fld_scenearea_loader.c",
# 0x021DE000). Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field scenearea --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry (12).
#
# SCENE AREA SYSTEM SA (0x20, 021AB5F4): +0 u32 the first creation argument (the field system; the callbacks hand it to the camera
# getter 0218E124 through 021AB760), +4 u32 the second (021AB764; the fieldmap, read by the grid-rect check), +8 the area list (0x48 each),
# +0xC the callback table descriptor CB, +0x10 u32 area count, +0x14 s32 the area the player is in (-1 none), +0x18 u32 enabled (the
# create sets 1; 021AB75C sets it; 021AB660 does nothing while it is 0), +0x1C u16 the id of the last callback run (0xFFFF none).
# AREA (0x48): +0x00..+0x3F the callback parameters (area `scenearea_loader`), +0x40 u16 check id, +0x42 u16 update id, +0x44 u16 enter
# id, +0x46 u16 exit id.
# CB: +0 the check table, +4 the update table (callback addresses), +8 u16 check count, +0xA u16 update count; a callback is
# `f(SA, area, pos)` (the check answers 1 when `pos` is inside the area).
#
# All blocks are `nowork`; objects are `arg buf` arenas. The SA arena (0x400): SA at 0, the area list at 0x100 (count 0..8), CB at
# 0x380 with its check table at 0x390 (4 words) and update table at 0x3A0 (24 words), counts bounded so that every table index stays
# in the arena; the table words are 0 (no callback) or the FAKE callbacks 0x02EE0011 (check) / 0x02EE0021 (update), `force` stubs at
# 0x02EE0010 / 0x02EE0020 (a port calls them through FieldEnv by address). Closure stubs (as core4 / F5b): a block lists the stubs of
# its callees and of the callees of its in-area descendants (checked with every function treated as ported: no fault, stray write or
# unstubbed callee). Pointer arguments of in-area stubs whose own block declares them `buf` are `ptr` (B18b).
area scenearea
set field

# 021AB5F4(heap, a, b) -> SA: Heap_AllocDebug(heap & 0xFFFF, 0x20, 1, "fld_scenearea.c", 0x51); +0 = a; +4 = b; +0x14 = -1; +0x18 = 1
# (+8..+0x10 and +0x1C are left as allocated). Arena: arg 3 (a stack word the function never reads).
fn 021ab5f4 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 4 || any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 02030734 force ret=ptr:arg3:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)

# 021AB624(SA): 021AB650(SA) (clear the list); Heap_Free(SA).
fn 021ab624 ret=none inputs=40
  nowork
  arg buf 0x40
  stub 021ab650 ret=none args=ptr  # in-area: clear the list (SA)
  stub 020307b0 force ret=none args=v  # Heap_Free

# 021AB634(SA, areas, count, CB): 021AB650(SA); +8 = areas; +0xC = CB; +0x10 = count; +0x14 = -1.
fn 021ab634 ret=none inputs=60
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 alt range 0 8 || any
  arg u32 any
  stub 021ab650 ret=none args=ptr  # in-area: clear the list (SA)

# 021AB650(SA): +8 = +0xC = +0x10 = 0; +0x14 = -1.
fn 021ab650 ret=none inputs=40
  nowork
  arg buf 0x40

# 021AB660(SA, pos) -> the area the player is in (-1 none): disabled (+0x18 == 0) -> -1 at once. Else +0x1C = 0xFFFF; found = the last
# area i whose check callback (021AB798(CB, area.check, SA, area, pos)) answers nonzero; found != +0x14: the old area's exit callback
# (021AB7C0(CB, area.exit, ...), +0x1C = area.exit) unless +0x14 == -1, then the new area's enter callback unless found == -1 (+0x1C =
# area.enter), +0x14 = found; found == +0x14 != -1: the update callback (+0x1C = area.update). Answers +0x14.
fn 021ab660 ret=u32 inputs=600
  nowork
  arg buf 0x400
  arg u32 any
  fix arg0 8 u32 addrof arg0 0x100
  fix arg0 0xC u32 addrof arg0 0x380
  fix arg0 0x10 u32 alt range 0 4 || range 0 8
  fix arg0 0x14 u32 alt const 0xFFFFFFFF || range 0 3 || range 0 8
  fix arg0 0x18 u32 alt const 1 || const 0 || any
  fix arg0 0x380 u32 addrof arg0 0x390
  fix arg0 0x384 u32 addrof arg0 0x3A0
  fix arg0 0x388 u16 alt const 2 || range 0 4 || range 0 0x1B
  fix arg0 0x38A u16 alt const 8 || range 0 9 || range 0 0x1B
  fix arg0 0x390 u32 alt const 0x02EE0011 || const 0 rep=4,4
  fix arg0 0x3A0 u32 alt const 0x02EE0021 || const 0 rep=24,4
  fix arg0 0x140 u16 alt range 0 3 || range 0 9 || any rep=8,0x48
  fix arg0 0x142 u16 alt range 0 9 || any rep=8,0x48
  fix arg0 0x144 u16 alt range 0 9 || any rep=8,0x48
  fix arg0 0x146 u16 alt range 0 9 || any rep=8,0x48
  stub 021ab798 ret=pick+u32:0/1 args=ptr,v,v,v,v  # in-area: run the check callback (CB, id, SA, area, pos)
  stub 021ab7c0 ret=u32 args=ptr,v,v,v,v  # in-area: run an update callback (CB, id, SA, area, pos)
  stub 021ab768 ret=pick:0/1 args=ptr,v  # in-area: check callback exists (CB, id)
  stub 02ee0010 force ret=pick+u32:0/1 args=v,v,v  # FAKE check callback (SA, area, pos)
  stub 021ab780 ret=pick:0/1 args=ptr,v  # in-area: update callback exists (CB, id)
  stub 02ee0020 force ret=u32 args=v,v,v  # FAKE update / enter / exit callback (SA, area, pos)

# 021AB75C(SA, v): +0x18 = v (enable / disable).
fn 021ab75c ret=none inputs=40
  nowork
  arg buf 0x40
  arg u32 alt range 0 1 || any

# 021AB760(SA): +0.
fn 021ab760 ret=u32 inputs=20
  nowork
  arg buf 0x40

# 021AB764(SA): +4.
fn 021ab764 ret=u32 inputs=20
  nowork
  arg buf 0x40

# 021AB768(CB, id): 1 when id < check count (u16 +8) and check table[id] != 0, else 0.
fn 021ab768 ret=u32 inputs=200
  nowork
  arg buf 0x40
  arg u32 alt range 0 0xD || any
  fix arg0 0 u32 addrof arg0 0x10
  fix arg0 4 u32 addrof arg0 0x10
  fix arg0 8 u16 alt const 2 || range 0 0xC
  fix arg0 0xA u16 alt const 8 || range 0 0xC
  fix arg0 0x10 u32 alt const 0 || any rep=12,4

# 021AB780(CB, id): 1 when id < update count (u16 +0xA) and update table[id] != 0, else 0.
fn 021ab780 ret=u32 inputs=200
  nowork
  arg buf 0x40
  arg u32 alt range 0 0xD || any
  fix arg0 0 u32 addrof arg0 0x10
  fix arg0 4 u32 addrof arg0 0x10
  fix arg0 8 u16 alt const 2 || range 0 0xC
  fix arg0 0xA u16 alt const 8 || range 0 0xC
  fix arg0 0x10 u32 alt const 0 || any rep=12,4

# 021AB798(CB, id, SA, area, pos): 021AB768(CB, id) -> check table[id](SA, area, pos) (its answer); else 0. id 0..11 and every table
# word a FAKE callback (the plain run stubs 021AB768, whose answer may be 1 for any id; its own block draws the other cases).
fn 021ab798 ret=u32 inputs=200
  nowork
  arg buf 0x40
  arg u32 range 0 0xB
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x10
  fix arg0 4 u32 addrof arg0 0x10
  fix arg0 8 u16 alt const 2 || range 0 0xC
  fix arg0 0xA u16 alt const 8 || range 0 0xC
  fix arg0 0x10 u32 const 0x02EE0011 rep=12,4
  stub 021ab768 ret=pick:0/1 args=ptr,v  # in-area: check callback exists (CB, id)
  stub 02ee0010 force ret=pick+u32:0/1 args=v,v,v  # FAKE check callback (SA, area, pos)

# 021AB7C0(CB, id, SA, area, pos): 021AB780(CB, id) -> update table[id](SA, area, pos) (its answer); else 0 (021AB780's answer).
# id 0..11 and every table word a FAKE callback (as 021AB798).
fn 021ab7c0 ret=u32 inputs=200
  nowork
  arg buf 0x40
  arg u32 range 0 0xB
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x10
  fix arg0 4 u32 addrof arg0 0x10
  fix arg0 8 u16 alt const 2 || range 0 0xC
  fix arg0 0xA u16 alt const 8 || range 0 0xC
  fix arg0 0x10 u32 const 0x02EE0021 rep=12,4
  stub 021ab780 ret=pick:0/1 args=ptr,v  # in-area: update callback exists (CB, id)
  stub 02ee0020 force ret=u32 args=v,v,v  # FAKE update / enter / exit callback (SA, area, pos)
