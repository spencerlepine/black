# fndiff spec: eventdata_system.c (overlay 10 Thumb code 0x0216237C..0x02163010, F1 row 10.23; F6a-spec): the zone event data of
# ARC 125 (one member per zone) and the lookups by position (grid VecFx32 or rail location), warp exits and trigger checks.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field eventdata --per-function
# Set `field` (main 10 11 18 20 21). F1 row 10.23 ("eventdata_system.c", 0x02161D98..0x02164768) holds four files; F6a-spec splits it by
# address into the areas wfbc_data (0x02161D98..0x0216237C), eventdata (0x0216237C..0x02163010), mmdl_move (0x02163010..0x021643AC) and
# eventdata_tail (0x021643AC..0x02164768); calls between them are `force` stubs (one: eventdata 02162970 -> eventdata_tail 02164708).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 197 entries in
# the row, minus 12 veneers (`ldr r3,=T; bx r3`, callers stub T: 02162024 -> 021622AC, 02162038 -> 02162330, 02162040 -> 02162358,
# 02162A64 -> 02162CE8, 02162A6C -> 02162ECC, 021626A8 -> 02162B68, 021626B0 -> 02162BCC, 02164454 / 5C / 64 / 6C -> 0215E34C,
# 02164474 -> 0215E364) and 3 false entries (02161E82, 021634A8 inside 02163464, 02163E72 inside 02163E58: data words in main
# 0x020A3E5A..0x020A481C decode as bl), = 185 specced (wfbc_data 40, eventdata 56, mmdl_move 70, eventdata_tail 19). Ghidra misses 02162370,
# 0216237C, 02162458, 02162478, 021624B0, 02162560, 021625E8, 021621C0, 02164240, 0216458C, 021645C0 (and splits 9 switch cases / tails
# into LAB_ functions). Render-only functions: none (the draw side is reached only through overlay-21 field-effect callees, all stubs).
# Closure stubs (as core4 / F4a): a block lists the stubs of its callees and of the callees of its in-area descendants and the shared fix
# lines, so the area can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed
# callee). Out-of-area callees are `force` stubs (overlay addresses collide with battle PORTED entries; ARM9 / fldmmdl callees are ported
# in other black-field areas). Indirect calls through ROM tables reach in-area stubs (plain) or force stubs at the table targets.
#
# EVENTDATA_SYSTEM (0xAA8 bytes, Heap_AllocDebug(heap, 0xAA8, 1, "eventdata_system.c", 0xAF) in 0216237C):
#   +0x04 ARC 125 handle (zone events), +0x08 ARC 126 handle (handed to field_encount_st 0215E2B8), +0x0C ARC 57 handle,
#   +0x10 u16 zone id (0216242C), +0x12 / +0x14 / +0x16 / +0x18 u16 bg / npc / warp / trigger counts, +0x1C / +0x20 / +0x24 / +0x28 the
#   record arrays, +0x2C..+0x32 u16 copies of the counts, +0x34 u32 field_encount_st ready (02162438), +0x38 field_encount_st work
#   (0215E2B8 .. 0x120; 021623DC clears bit 7 of +0x3F), +0x120 -> the TAIL after the records (021623DC sets sys + 0x2328, outside the
#   block, until the load), +0x124 u32 = the file's first word, +0x128 the member data (<= 0x880 bytes).
# ZONE EVENT FILE (ARC 125 member, read whole to +0x124 by 021624B0): u32 size (records + 4), u8 counts bg, npc, warp, trigger, the
#   records, then the TAIL at 4 + size (02162450 returns it; F7 / F8 read it).
# BG (20 bytes): +0 u16 script, +2 u16 kind (2 = hidden item: 02164708(script) is its flag, the item is there while the flag is clear),
#   +4 u16 facing rule (0: dir 0, 1: dir 3, 2: dir 2, 3: dir 1, 4: any, 5: dir 2 or 3, 6: dir 0 or 1), +6 u16 position kind (0 grid,
#   else rail), grid: +8 u32 tile x, +0xC u32 tile z, +0x10 s32 y (fx32 >> 12; hit if y - 2 <= pos.y >> 12 < y + 2); rail: +8 u16 line,
#   +0xA u16 front, +0xC s16 side (02162E9C, exact match).
# NPC (36 bytes) = the MMDL HEADER (F4a-spec): +2 OBJ code (02162774), +0xA event id (02162788), +0xC dir, +0x18 u32 kind (0 grid),
#   +0x1C u16 x, +0x1E u16 z, +0x20 fx32 y (02162748).
# WARP (20 bytes): +0 u16 destination zone, +2 u16 destination warp (0x100: LOCATION kind 3, else 2; 0xFFFF in +0 or +2 = disabled,
#   0216245C / 021626EC), +4 u8 exit direction (1..4 -> 0..3 (02162C54), else 1; 021626B8 pairs), +5 u8 (021626E8), +6 u16 position
#   kind (0 grid, 1 rail), grid: +8 / +0xA / +0xC s16 x / y / z in fx32 >> 12 units (16 per tile; 02162708 sets 16 t + 8), +0xE u16
#   width, +0x10 u16 depth (tiles; 02162D2C: x <= pos.x >> 12 <= x + 16 (w - 1), |pos.y >> 12 - y| <= 2, z likewise); rail: +8 u16
#   line, +0xA u16 front start, +0xC s16 side start, +0xE u16 front length, +0x10 u16 side length, +0x12 u16 flip mode (02162BCC).
# TRIGGER (22 bytes): +0 u16 script, +2 u16 value, +4 u16 var (0 = always; else *var(ew, var) == value), +6 u16 kind (021627C8: 1..4 =
#   dir 0..3 (0216279C), 6: 02162890), +8 u16 position kind (0 grid, else rail), grid: +0xA u16 x, +0xC u16 z, +0xE u16 width,
#   +0x10 u16 depth (tiles), +0x12 s16 y (fx32 >> 12; y - 2 <= pos.y >> 12 < y + 2), +0x14 u16 (unread); rail: +0xA line, +0xC front,
#   +0xE side, +0x10 front length, +0x12 side length.
# RAIL LOCATION (8 bytes): +0 u16 line, +2 u8 type (1), +3 u8 key (0201441C(dir)), +4 s16 side, +6 u16 front.
# Exit code (u16, 02162B68 / 02162BCC): width << 4 | offset | dir << 8; 02162AA4 maps an offset onto a door of another width.
#
# Arena (the `work` image): EVENTDATA_SYSTEM work+0 with bg x4 at work+0x140, npc x4 at work+0x200, warp x6 at work+0x340, trigger x6
# at work+0x400 (counts drawn 0..4 / 0..6), position VecFx32 work+0xB00, rail location work+0xB10, LOCATION work+0xB20, out vectors
# work+0xB40, event-work vars work+0xC00 (the var-address stub 020143CC answers there; the event-work handle is the constant 0x02E0C000).
area eventdata
set field

# 0216237C(heap) -> sys: sys = Heap_AllocDebug(heap, 0xAA8, 1, "eventdata_system.c", 0xAF); +4 = ARC 125 (zone events), +8 =
# ARC 126, +0xC = ARC 57 handles (archive open (arc, heap)); returns sys. Arena: argument 1 (an unused register)
fn 0216237c ret=ofs:arg1 inputs=40
  nowork
  arg u32 any
  arg buf 0x1000
  stub 02030734 force ret=ptr:arg1:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap): handle

# 021623BC(sys): archive close +4, +8, +0xC; Heap_Free(sys)
fn 021623bc ret=none inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02049238 force ret=none args=v  # archive close (handle)

# 021623DC(sys): 02162530(sys); +0x120 = sys + 0x2328 (sic: past the 0xAA8 block; replaced by the load); +0x34 = 0;
# +0x3F &= 0x7F (a byte of the field_encount_st work)
fn 021623dc ret=none inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162530 ret=none args=v  # in-area

# 02162408(sys, zone, x): 021623DC(sys); 0216242C(sys, zone, x); 02162438(sys, zone, x)
fn 02162408 ret=none inputs=100
  arg work 0
  arg u32 any
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02013ecc force ret=u32 args=v  # zonedata: zone id -> ARC 125 member (zone events)
  stub 020493d4 force ret=u32 args=v,v  # archive member size (handle, member)
  stub 0204944c force ret=none args=v,v,v,v,out:0x10  # archive read (handle, member, offset 0, size, dst)
  stub 0215e2b8 force ret=none args=v,v,v,v  # field_encount_st init (work, ARC 126 handle, zone, x)
  stub 021623dc ret=none args=v  # in-area
  stub 0216242c ret=none args=v,v  # in-area
  stub 02162438 ret=none args=v,v,v  # in-area
  stub 021624b0 ret=none args=v,v  # in-area
  stub 02162530 ret=none args=v  # in-area

# 0216242C(sys, zone): +0x10 = zone (u16); tail 021624B0(sys, zone)
fn 0216242c ret=none inputs=100
  arg work 0
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02013ecc force ret=u32 args=v  # zonedata: zone id -> ARC 125 member (zone events)
  stub 020493d4 force ret=u32 args=v,v  # archive member size (handle, member)
  stub 0204944c force ret=none args=v,v,v,v,out:0x10  # archive read (handle, member, offset 0, size, dst)
  stub 021624b0 ret=none args=v,v  # in-area

# 02162438(sys, zone, x): 0215E2B8(sys + 0x38, +8 (ARC 126 handle), zone, x) (field_encount_st init); +0x34 = 1
fn 02162438 ret=none inputs=100
  arg work 0
  arg u32 any
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 0215e2b8 force ret=none args=v,v,v,v  # field_encount_st init (work, ARC 126 handle, zone, x)

# 02162450(sys) -> +0x120 (the tail after the records)
fn 02162450 ret=u32 inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162458(sys) -> +0x34 (field_encount_st ready)
fn 02162458 ret=u32 inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 0216245C(sys, i): i < warp count (+0x16): warp i +0 = +2 = 0xFFFF (disables the warp)
fn 0216245c ret=none inputs=100
  arg work 0
  arg u32 alt range 0 6 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162478(sys, pos) -> bool: a trigger of kind 6 (+6) that contains pos (02162F0C); no callers in the field set
fn 02162478 ret=u32 inputs=300
  arg work 0
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162f0c ret=pick:0/1 args=v,v  # in-area

# 021624B0(sys, zone): m = 02013ECC(zone); n = member size (+4 handle, m); archive read (+4, m, 0, n, sys + 0x124); +0x120 =
# sys + 0x128 + *(sys + 0x124) (the tail); counts = the 4 bytes at +0x128 (bg, npc, warp, trigger) -> +0x12..+0x18 and +0x2C..+0x32;
# records from +0x12C: +0x1C bg (20 B), +0x20 npc (36 B), +0x24 warp (20 B), +0x28 trigger (22 B)
fn 021624b0 ret=none inputs=100
  arg work 0
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02013ecc force ret=u32 args=v  # zonedata: zone id -> ARC 125 member (zone events)
  stub 020493d4 force ret=u32 args=v,v  # archive member size (handle, member)
  stub 0204944c force ret=none args=v,v,v,v,out:0x10  # archive read (handle, member, offset 0, size, dst)

# 02162530(sys): u16 +0x2C..+0x32 and +0x12..+0x18 = 0; +0x1C..+0x28 = 0; MI_CpuFill8(sys + 0x128, 0, 0x880) (tail)
fn 02162530 ret=none inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 0216255C(sys) -> sys + 0x38 (field_encount_st work)
fn 0216255c ret=ofs:work inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162560(sys) -> warp count (u16 +0x16); no callers in the field set
fn 02162560 ret=u32 inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162564(sys, pos) -> index of the first grid warp that contains pos (02162D2C), else 0xFFFF
fn 02162564 ret=u32 inputs=300
  arg work 0
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162d2c ret=pick:0/1 args=v,v  # in-area

# 02162598(sys, rail) -> index of the first rail warp that contains the rail location (02162E00), else 0xFFFF
fn 02162598 ret=u32 inputs=300
  arg work 0
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162e00 ret=pick:0/1 args=v,v  # in-area

# 021625CC(sys, i) -> warp i (+0x24 + 0x14 i) if +0x24 != 0 and i < count, else 0
fn 021625cc ret=ofs:work inputs=100
  arg work 0
  arg u32 alt range 0 6 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  fix work 0x24 u32 alt const 0 || addrof work 0x340 || addrof work 0x340

# 021625E8(warp) -> warp +2 == 0x100
fn 021625e8 ret=u32 inputs=100
  arg work 0x340
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 021625FC(sys, loc, i, code) -> bool: w = 021625CC(sys, i); 0: 0; LOCATION clear (loc); loc +0 = w +2 == 0x100 ? 3 : 2;
# w +6 == 0 (grid): 02162C84(w, code, &v); 02014110(loc, zone (+0x10), (s16) i, w +4 (u8), code, v.x, v.y, v.z); else (rail):
# 02162D90(w, code, &r); 02014134(loc, zone, (s16) i, w +4, code, r +0 (u16), r +6 (u16), r +4 (s16)); 1
fn 021625fc ret=u32 inputs=100
  arg work 0
  arg work 0xB20
  arg u32 alt range 0 6 || any
  arg u32 alt mask 0x3FF || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020140f8 force ret=none args=v  # LOCATION clear (loc)
  stub 02014110 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, grid form (loc, zone, warp idx, dir, code, x, y, z)
  stub 02014134 force ret=none args=out:0x1C,v,v,v,v,v,v,v  # LOCATION init, rail form (loc, zone, warp idx, dir, code, line, front, side)
  stub 0201441c force ret=u8 args=v  # dir -> rail key
  stub 021625cc ret=nullable:4:ptr:work:0x340:1 args=v,v  # in-area
  stub 02162aa4 ret=u16 args=v,v,v,v,v  # in-area
  stub 02162c54 ret=pick:0/1/2/3 args=v  # in-area
  stub 02162c84 ret=none args=v,v,out:12  # in-area
  stub 02162d90 ret=none args=v,v,out:8  # in-area

# 02162694(r, loc, x): LOCATION set (loc, r +0, r +2 (u16), x)
fn 02162694 ret=none inputs=100
  arg work 0x340
  arg work 0xB20
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 0201416c force ret=none args=v,v,v,v  # LOCATION set (loc, zone, warp id, x)

# 021626B8(warp, dir) -> bool: (dir, warp +4) is one of (0, 2), (1, 1), (2, 4), (3, 3)
fn 021626b8 ret=u32 inputs=100
  arg work 0x340
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 021626E8(warp) -> warp +5 (u8)
fn 021626e8 ret=u32 inputs=100
  arg work 0x340
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 021626EC(warp) -> warp +0 == 0xFFFF or warp +2 == 0xFFFF
fn 021626ec ret=u32 inputs=100
  arg work 0x340
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162708(sys, i, x, y, z) (z: stack u16): i <= warp count (sic: <=) and +0x24 != 0 and warp i +6 == 0: +8 = 16 x + 8,
# +0xA = 16 y, +0xC = 16 z + 8 (u16)
fn 02162708 ret=none inputs=100
  arg work 0
  arg u32 alt range 0 6 || any
  arg u32 any
  arg u32 any
  arg u16 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162740(sys) -> npc records (+0x20)
fn 02162740 ret=u32 inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162744(sys) -> npc count (u16 +0x14)
fn 02162744 ret=u32 inputs=100
  arg work 0
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162748(sys, i, dir, x, y, z) (y: stack u32, z: stack u16): i < npc count and npc i +0x18 == 0 (grid): +0xC = dir,
# +0x1C = x (u16), +0x20 = y (u32), +0x1E = z (u16)
fn 02162748 ret=none inputs=100
  arg work 0
  arg u32 alt range 0 4 || any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u16 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162774(sys, i, code): i < npc count: npc i +2 (OBJ code) = code
fn 02162774 ret=none inputs=100
  arg work 0
  arg u32 alt range 0 4 || any
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162788(sys, i, id): i < npc count: npc i +0xA (event id / script) = id
fn 02162788 ret=none inputs=100
  arg work 0
  arg u32 alt range 0 4 || any
  arg u32 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 0216279C(dir) -> 0 / 1 / 2 / 3 -> 1 / 2 / 3 / 4, else 0
fn 0216279c ret=u32 inputs=100
  nowork
  arg u32 alt range 0 5 || any

# 021627C8(sys, ew, pos, dir) -> trigger: +0x28 != 0; k = 0216279C(dir); first trigger with 02162F0C(t, pos) and t +6 == k and
# (t +4 == 0 or *var(ew, t +4) == t +2); else 0
fn 021627c8 ret=ofs:work inputs=300
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  fix work 0x28 u32 alt const 0 || addrof work 0x400 || addrof work 0x400
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 0216279c ret=pick:0/1/2/3/4 args=v  # in-area
  stub 02162f0c ret=pick:0/1 args=v,v  # in-area

# 0216282C(sys, ew, pos, dir) -> trigger: as 021627C8 with 02162F74 (no height check)
fn 0216282c ret=ofs:work inputs=300
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 0216279c ret=pick:0/1/2/3/4 args=v  # in-area
  stub 02162f74 ret=pick:0/1 args=v,v  # in-area

# 02162890(sys, ew, pos) -> trigger: as 021627C8 with kind 6
fn 02162890 ret=ofs:work inputs=300
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 02162f0c ret=pick:0/1 args=v,v  # in-area

# 021628E4(sys, ew, rail) -> trigger: first with 02162FC8(t, rail) and *var(ew, t +4) == t +2; else 0
fn 021628e4 ret=ofs:work inputs=600
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 02162fc8 ret=pick:0/1 args=v,v  # in-area

# 02162928(sys, ew, pos, dir) -> script (+0) of 021627C8(..) or 0xFFFF
fn 02162928 ret=u32 inputs=100
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 0216279c ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021627c8 ret=nullable:2:ptr:work:0x400:1 args=v,v,v,v  # in-area
  stub 02162f0c ret=pick:0/1 args=v,v  # in-area

# 02162940(sys, ew, pos) -> script of 02162890(..) or 0xFFFF
fn 02162940 ret=u32 inputs=100
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 02162890 ret=nullable:2:ptr:work:0x400:1 args=v,v,v  # in-area
  stub 02162f0c ret=pick:0/1 args=v,v  # in-area

# 02162958(sys, ew, rail) -> script of 021628E4(..) or 0xFFFF
fn 02162958 ret=u32 inputs=600
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020143cc force ret=ptr:work:0xC00:0x10 args=v,v  # event_work var address (ew, var id)
  stub 021628e4 ret=nullable:2:ptr:work:0x400:1 args=v,v,v  # in-area
  stub 02162fc8 ret=pick:0/1 args=v,v  # in-area

# 02162970(sys, ew, pos, dir) -> script: for each bg b (+0x1C, count +0x12): b +6 == 0 ? 02162E48(b, pos) : 02162E9C(b, pos);
# hit: b +2 == 2 (hidden item): flag 02164708(b +0) not set (check_flag(ew, ..) == 0): b +0; else by b +4: 0: dir == 0, 1: dir == 3,
# 2: dir == 2, 3: dir == 1, 4: any, 5: dir - 2 (u16) < 2, 6: dir < 2 -> b +0; else 0xFFFF
fn 02162970 ret=u32 inputs=600
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  fix work 0x1C u32 alt const 0 || addrof work 0x140 || addrof work 0x140 || addrof work 0x140
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 02162e48 ret=pick:0/1 args=v,v  # in-area
  stub 02162e9c ret=pick:0/1 args=v,v  # in-area
  stub 02164708 force ret=u32 args=v  # eventdata_tail

# 02162A54(sys, ew, pos, dir) -> 02162970(..)
fn 02162a54 ret=u32 inputs=100
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 02162970 ret=pick:0/1/0xFFFF args=v,v,v,v  # in-area
  stub 02162e48 ret=pick:0/1 args=v,v  # in-area
  stub 02162e9c ret=pick:0/1 args=v,v  # in-area
  stub 02164708 force ret=u32 args=v  # eventdata_tail

# 02162A5C(sys, ew, pos, dir) -> 02162970(..)
fn 02162a5c ret=u32 inputs=100
  arg work 0
  arg u32 const 0x02E0C000
  arg work 0xB00
  arg u32 alt range 0 4 || any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 02162970 ret=pick:0/1/0xFFFF args=v,v,v,v  # in-area
  stub 02162e48 ret=pick:0/1 args=v,v  # in-area
  stub 02162e9c ret=pick:0/1 args=v,v  # in-area
  stub 02164708 force ret=u32 args=v  # eventdata_tail

# 02162A74(sys, i, x, y, z) (z: stack u16): i <= bg count (sic) and +0x1C != 0 and bg i +6 == 0: +8 = x, +0xC = z (u32),
# +0x10 = y << 4
fn 02162a74 ret=none inputs=100
  arg work 0
  arg u32 alt range 0 4 || any
  arg u32 any
  arg u32 any
  arg u16 any
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162AA4(code, dir, flip, mode, n) -> u16 (n: stack u16): code 0: 0; o = code & 0xF, w = (code >> 4) & 0xF, d = (code >> 8) & 0xF;
# (d, dir) in (0,3) (1,2) (3,0) (2,1): o = w - 1 - o; flip: mode 0: d - 2 < 2 / mode 1: d < 2 / mode 2: o = w - 1 - o; then re-centres o
# from width w to width n (odd / even cases) and clamps to 0..n-1
fn 02162aa4 ret=u32 inputs=100
  nowork
  arg u32 alt mask 0x3FF || any
  arg u32 alt range 0 4 || any
  arg u32 alt oneof 0 1 || any
  arg u32 alt range 0 3 || any
  arg u16 alt range 0 9 || any

# 02162B68(warp, pos) -> exit code: k = 02162C54(warp) << 8; w +0xE > 1: w << 4 | ((pos.x >> 12) - x) / 16 | k; w +0x10 > 1:
# likewise with z; else k | 0x10
fn 02162b68 ret=u32 inputs=100
  arg work 0x340
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162c54 ret=pick:0/1/2/3 args=v  # in-area

# 02162BCC(warp, rail) -> exit code (rail form): flips by warp +0x12 (0, 2: front; 1, 2: side); +0x10 > 1: side offset;
# +0xE > 1: front offset; else k | 0x10
fn 02162bcc ret=u32 inputs=100
  arg work 0x340
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162c54 ret=pick:0/1/2/3 args=v  # in-area

# 02162C54(warp) -> warp +4 (u8): 1 -> 0, 2 -> 1, 3 -> 2, 4 -> 3, else 1
fn 02162c54 ret=u32 inputs=100
  arg work 0x340
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162C84(warp, code, out): out = (x, y, z) << 12 (s16 +8, +0xA, +0xC); +0xE > 1: out.x += 02162AA4(code, dir, 0, 0, +0xE) << 16;
# else +0x10 > 1: out.z += ..
fn 02162c84 ret=none inputs=100
  arg work 0x340
  arg u32 alt mask 0x3FF || any
  arg work 0xB40
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 02162aa4 ret=u16 args=v,v,v,v,v  # in-area
  stub 02162c54 ret=pick:0/1/2/3 args=v  # in-area

# 02162CE8(warp, out): out = (x, y, z) << 12; +0xE > 1: out.x += (+0xE >> 1) << 16; else +0x10 > 1: out.z += ..
fn 02162ce8 ret=none inputs=100
  arg work 0x340
  arg work 0xB40
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162D2C(warp, pos) -> bool: +6 == 1: 0; x <= pos.x >> 12 <= x + 16 (w - 1), y - 2 <= pos.y >> 12 <= y + 2,
# z <= pos.z >> 12 <= z + 16 (d - 1)
fn 02162d2c ret=u32 inputs=300
  arg work 0x340
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162D90(warp, code, out rail): k = 02162C54(warp); out +0 = +8, +2 = 1, +3 = 0201441C(k), +4 = +0xC (s16), +6 = +0xA;
# +0x10 > 1: out +4 += 02162AA4(code, k, 1, +0x12, +0x10); else +0xE > 1: out +6 += 02162AA4(code, k, 1, +0x12, +0xE)
fn 02162d90 ret=none inputs=100
  arg work 0x340
  arg u32 alt mask 0x3FF || any
  arg work 0xB40
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 0201441c force ret=u8 args=v  # dir -> rail key
  stub 02162aa4 ret=u16 args=v,v,v,v,v  # in-area
  stub 02162c54 ret=pick:0/1/2/3 args=v  # in-area

# 02162E00(warp, rail) -> bool: +6 == 0: 0; +8 == rail +0; +0xA <= rail +6 < +0xA + +0xE; +0xC <= rail +4 < +0xC + +0x10
fn 02162e00 ret=u32 inputs=300
  arg work 0x340
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162E48(bg, pos) -> bool: +6 == 1: 0; +8 == tile x, +0xC == tile z (fx / 0x10000 toward 0), +0x10 - 2 <= pos.y >> 12 < +0x10 + 2
fn 02162e48 ret=u32 inputs=300
  arg work 0x140
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162E9C(bg, rail) -> bool: +6 == 0: 0; +8 == rail +0, +0xA == rail +6, +0xC == rail +4
fn 02162e9c ret=u32 inputs=300
  arg work 0x140
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162ECC(trigger, out): out.z = (z << 16) + FX_Mul(h << 16, 0x2000); out.x = (x << 16) + FX_Mul(w << 16, 0x2000);
# out.y = y << 12 (the centre)
fn 02162ecc ret=none inputs=100
  arg work 0x400
  arg work 0xB40
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2
  stub 0207c768 force ret=u32 args=v,v  # FX_Mul (a, b)

# 02162F0C(trigger, pos) -> bool: +8 == 1: 0; x <= tile x < x + w, z <= tile z < z + h, y - 2 <= pos.y >> 12 < y + 2
fn 02162f0c ret=u32 inputs=300
  arg work 0x400
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162F74(trigger, pos) -> bool: as 02162F0C without the height check
fn 02162f74 ret=u32 inputs=300
  arg work 0x400
  arg work 0xB00
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

# 02162FC8(trigger, rail) -> bool: +8 == 0: 0; +0xA == rail +0; +0xC <= rail +6 < +0xC + +0x10; +0xE <= rail +4 < +0xE + +0x12
fn 02162fc8 ret=u32 inputs=300
  arg work 0x400
  arg work 0xB10
  fix work 0x10 u16 any
  fix work 0x12 u16 alt range 1 4 || const 0
  fix work 0x14 u16 alt range 1 4 || const 0
  fix work 0x16 u16 alt range 1 6 || const 0
  fix work 0x18 u16 alt range 1 6 || const 0
  fix work 0x1C u32 addrof work 0x140
  fix work 0x20 u32 addrof work 0x200
  fix work 0x24 u32 addrof work 0x340
  fix work 0x28 u32 addrof work 0x400
  fix work 0x142 u16 oneof 0 1 2 3 rep=4,0x14
  fix work 0x144 u16 alt range 0 6 || range 0 6 || any rep=4,0x14
  fix work 0x146 u16 alt const 0 || const 0 || oneof 0 1 || any rep=4,0x14
  fix work 0x148 u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x14C u32 alt const 0 || const 0 || range 0 1 || any rep=4,0x14
  fix work 0x150 u32 alt range 0 3 || range 0 3 || range 0 7 || any rep=4,0x14
  fix work 0x218 u32 alt oneof 0 1 || any rep=4,0x24
  fix work 0x340 u16 alt const 0xFFFF || any rep=6,0x14
  fix work 0x342 u16 alt oneof 0x100 0xFFFF || any rep=6,0x14
  fix work 0x344 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x345 u8 alt range 0 5 || any rep=6,0x14
  fix work 0x346 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x14
  fix work 0x348 u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34A u16 alt range 0 7 || range 0 1 || any rep=6,0x14
  fix work 0x34C u16 alt range 0 0x20 || range 0 1 || any rep=6,0x14
  fix work 0x34E u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x350 u16 alt range 1 3 || range 0 4 || any rep=6,0x14
  fix work 0x352 u16 alt range 0 3 || any rep=6,0x14
  fix work 0x402 u16 range 0 1 rep=6,0x16
  fix work 0x404 u16 alt const 0 || range 0x4000 0x4003 rep=6,0x16
  fix work 0x406 u16 alt range 0 4 || const 6 || any rep=6,0x16
  fix work 0x408 u16 alt const 0 || const 0 || oneof 0 1 || any rep=6,0x16
  fix work 0x40A u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40C u16 alt const 0 || const 0 || range 0 1 || any rep=6,0x16
  fix work 0x40E u16 alt range 0 2 || any rep=6,0x16
  fix work 0x410 u16 alt range 0 2 || any rep=6,0x16
  fix work 0x412 u16 alt range 0 3 || range 0 3 || range 0 7 || any rep=6,0x16
  fix work 0xB00 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB04 u32 alt range 0 0x3FFF || range 0 0x3FFF || range 0 0x7FFF || any
  fix work 0xB08 u32 alt range 0 0xFFFF || range 0 0xFFFF || range 0 0x1FFFF || any
  fix work 0xB10 u16 alt range 0 1 || any
  fix work 0xB14 u16 alt range 0 1 || range 0 4 || any
  fix work 0xB16 u16 alt range 0 1 || range 0 4 || any
  fix work 0xC00 u16 range 0 1 rep=32,0x2

