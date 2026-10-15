# fndiff spec (F2b): field_buildmodel.c (overlay 21 Thumb code 0x0218B754..0x0218C9E8, F1 row 21.01; 89 functions, all specced): the
# map building models of a zone. It loads the zone's model list (archive 0xE5 / 0xE6: one 0x24-byte record per model kind) with its
# texture and animation resources, keeps a 400-byte map model id -> list index, places the models of each map block (0x120 placed-model
# entries: field_g3d_map handle + index + animation state word), runs the models' animations (4 kinds by the record's header: none /
# loop all / scripted states / by time of day, ROM tables 0x021D3D58..0x021D3D87), and makes DOOR objects (7 slots) for placed models
# found by position, with their open / close animations and sounds (ROM table 0x021D3D88). Logic only (D3): the G3D resources and
# objects (g3d_system.c, main 0x02046xxx / 0x02047xxx), field_g3d_map.c (ov10 0x021693D4..), field_g3d_mapper.c, the FX matrix /
# vector library and the sound calls are `force` stubs (the port reaches them through FieldEnv by address).
# File bound: `Heap_AllocDebug(heap, 0x13F8, 0, "field_buildmodel.c", 0x193)` in 0218B754 (string 0x021DD9CC, also used by the
# allocations in 0218BD28 / 0218BDD8 / 0218BEE8 / 0218C584 / 0218C724 / 0218C7F0); fieldmap.c ends at 0x0218B752; field_g3d_mapper.c
# starts at 0x0218C9E8 (its allocation names "field_g3d_mapper.c", 0x021DD9EC). Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-field buildmodel --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers (the ROM proc tables 0x021D3D58), the Ghidra index and a
# Thumb walk from every entry (89; F1's 89 counts the case label 0218C192 and misses the table entry 0218C08C). 0218B924 (`ldr r0,
# [r0, #0x18]; bx 0218BBF5`), 0218B9FC (`bx 0218C4ED`) and 0218BCE4 (stores, then `bx 0218BCF5`) are thunks with their own entries;
# 0218C150 and 0218C438 are empty procs. A `bl` decoded from the literal 0xFFFFF000 in 0218C1C4 is data.
#
# BM (0x13F8, 0218B754): +0 u16 heap, +4 G (handed to field_g3d_mapper 0218DC5C), +8 TOD (time of day, 0x10: +0 u32 time zone
# (020113D8(season): 0..4), +4 previous zone, +8 changed, +0xC u8 animation index = ROM 0x021D3D50[zone] = {0, 1, 2, 3, 3, 0, 0, 0},
# +0xD u8 season), +0x18 the model list LIST, +0x1C u8 map[400] (model id -> list index, 0x80 none), +0x1AC u16 model count (LIST[2] >>
# 1), +0x1B0 S (+0 8-byte array {animation object handle, 0} per model, +4 count, +8 a block), +0x1C8 / +0x1CC animation resource
# sets (count, array of 0x18), +0x1D0 the texture resource, +0x1D4 / +0x1D8 animation objects (count, array of 0x18), +0x1DC the 7
# door slots, +0x1F8 0x120 placed-model ENTRIES (0x10: +0 field_g3d_map handle (0 free), +4 s32 saved state, +8 u16 index, +0xA u16
# flag, +0xC -> the state word of the placed model T (its +0 the state / resource index, +4 a VecFx32 offset, +0x10 u16 rotation)).
# LIST: +2 u8 count * 2, +4 u32 offsets[]: records 0..count-1, then the animation data of each record (0218BC00); RECORD (0x24):
# +0 u16 model id, +2 u16 door kind, +4 u16 linked model id (0xFFFF none), +6 / +8 / +0xA s16 the linked model's offset (<< 12), +0x10
# the animation header HDR: +0 u8 kind 0..3, +2 u8 frames per state, +4 u32 offsets[4] (-1 none).
# Animation resource SET (0x18, 0218BEE8): +0 the record, +4 the model resource, +8 4 animation resources. Animation OBJECT (0x18,
# 0218BFB4): +0 the G3D object, +4 its SET, +8 u32 state[4] (0 off, 1 looping, 2 stopped, 3 playing, 4 playing backward). DOOR D (0x60,
# 0218C724 / 0218C7F0): +0 BM, +4 u32 the state s, +8 the entry (0 none), +0xC an animation OBJECT, +0x24 VecFx32 position, +0x30
# VecFx32 scale, +0x3C 3x3 rotation.
#
# All blocks are `nowork`. The BM arena is an `arg buf 0x10000` (the argument K the header of each block names: arg 0 when the
# function takes BM first, else a trailing argument the function never reads as a register) with fixed zones: BM at 0, LIST at 0x1400
# (offset table, 254 entries, each one of the 8 records at 0x1800 + 0x24 k), SETS at 0x2000 (127, +0 one of the records), OBJECTS at
# 0x2C00 (127, +4 one of the first 8 sets), the 7 DOORS at 0x3800 + 0x60 j, 16 T words at 0x3C00 + 0x20 k (every entry's +0xC points at
# one), the animation-resource answer of 02047674 at 0x3F00 (+8 -> 0x3F40), the list answer of the in-area 0218C584 stub at 0x3F80, and
# Heap_AllocDebug as a bump allocator (`seqptr`, 0x400 per block) from 0x4000. BM fields are drawn around the compared values (counts
# 0..8 or up to 127, map bytes 0..8 / 0x80 / any, model ids below 400, kinds 0..3, states 0..5, handles from a small set; the model
# count is any u16 in the blocks that only compare with it, while 0218BB78 / 0218BD28 (u8 loop counters) and 0218BDD8 loop over it;
# VEC_Add answers words of one small set (`outw`) that the box bounds and positions of 0218C584..0218C7D8 are drawn from). Pointers into
# the arena handed to a function as a scalar are fixed addresses (0x02E10000 + K * 0x10000 + zone offset).
# Closure stubs (as core4 / F5b / F11): a block lists the stubs of its callees and of the callees of its in-area descendants (checked
# with every function treated as ported: no fault, stray write or unstubbed callee); in-area stubs that hand out pointers answer
# `pick:` lists of the arena records / headers / entries; the calls through the ROM proc tables land on the in-area stubs of the table
# targets. 0204784C takes a frame pointer from 0218C2E4 (`in:4`) and a 0 from 0218C1C4 (`v`). 0204779C (a u16 count) is a closed
# `pick:0/1/2/4` everywhere but in 0218C0D8's own block (`pick+u16`): the callers loop over the count, and 0x10000 iterations per
# object in the closures that free whole arrays exhaust the run time.
area buildmodel
set field

# 0218B754(heap, G, s) -> BM: Heap_AllocDebug(heap, 0x13F8, 0, "field_buildmodel.c", 0x193); +0 u16 heap; +4 G; 0218BCE4(BM+8, s & 0xFF)
# (time of day, season s); the 7 door slots +0x1DC = 0; 0218C490(BM, entry) for the 0x120 entries at +0x1F8; +0x1B0..+0x1CC (8
# words), +0x1D4, +0x1D8 = 0 (+0x1D0 is left). Arena: arg 3 (a stack word the function never reads).
fn 0218b754 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 4 || any
  arg u32 any
  arg u32 any
  arg buf 0x1400
  stub 02030734 force ret=ptr:arg3:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218bce4 ret=none args=ptr,v  # in-area: time of day init (t, season)
  stub 0218c490 ret=none args=v,ptr  # in-area: clear an entry (BM, e)
  stub 0218bcf4 ret=none args=ptr  # in-area: time of day update (t)
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # the time zone of the hour in season s (0..4)

# 0218B7F8(BM): 0218C860(D) for every door slot set; 0218BE84(BM, BM+0x1B0); 0218BD94(BM); 0218BB64(BM); 0218BC6C(BM); Heap_Free(BM).
fn 0218b7f8 ret=none inputs=60
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218c860 ret=none args=v  # in-area: delete a door object (D)
  stub 0218be84 ret=none args=ptr,v  # in-area: free the animation objects (BM, S)
  stub 0218bd94 ret=none args=ptr  # in-area: free the animation resources
  stub 0218bb64 ret=none args=ptr  # in-area: free the model list
  stub 0218bc6c ret=none args=ptr  # in-area: free the textures
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 0218ba48 ret=none args=ptr,v  # in-area: clear the door slot of D
  stub 0218c0d8 ret=none args=ptr  # in-area: delete an animation object
  stub 0218c54c ret=none args=ptr,v  # in-area: swap an entry state (e, f)
  stub 0218bf84 ret=none args=ptr  # in-area: free an animation resource set
  stub 020470d0 force ret=none args=v  # g3d_system: release the upload
  stub 02046efc force ret=none args=v  # g3d_system: delete the resource
  stub 0204779c force ret=pick:0/1/2/4 args=v  # g3d_system: animation count (a u16; closed set outside 0218C0D8, see the header)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 0204762c force ret=none args=v  # g3d_system: delete an animation
  stub 02047788 force ret=u32 args=v  # g3d_system: model of the object
  stub 02047738 force ret=none args=v  # g3d_system: delete the object
  stub 02047504 force ret=none args=v  # g3d_system: delete the model

# 0218B83C(BM) (one frame): 0218BCF4(BM+8) (time of day); 0218C124(BM, D+0xC) for every door slot set; 0218C124(BM, +0x1D8 + 0x18 i)
# for i < +0x1D4.
fn 0218b83c ret=none inputs=100
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218bcf4 ret=none args=ptr  # in-area: time of day update (t)
  stub 0218c124 ret=none args=ptr,v  # in-area: animation frame (BM, obj)
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # the time zone of the hour in season s (0..4)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c150 ret=none  # in-area: frame proc, kind 0 (empty)
  stub 0218c1a4 ret=none args=v,ptr  # in-area: frame proc, kind 1 (BM, obj)
  stub 0218c1c4 ret=none args=v,ptr  # in-area: frame proc, kind 2 (BM, obj)
  stub 0218c154 ret=none args=ptr,v  # in-area: frame proc, kind 3 (BM, obj)
  stub 02047910 force ret=none args=v,v,v  # g3d_system: animation advance (obj, i, step)
  stub 020478b8 force ret=pick+u32:0/1 args=v,v,v  # g3d_system: animation advance, 0 at the end (obj, i, step)
  stub 0204784c force ret=none args=v,v,v  # g3d_system: animation frame (obj, i, &frame or 0)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)
  stub 0218bd20 ret=pick:0/1 args=ptr  # in-area: the time zone changed (t)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)

# 0218B88C(BM) (draw): 0218C934(D) for every door slot set.
fn 0218b88c ret=none inputs=60
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218c934 ret=none args=v  # in-area: draw a door (D)
  stub 02047a60 force ret=none args=v,in:0x3C  # g3d_system (G8): draw (obj, &transform)

# 0218B8AC(BM, zone, x, h2) (load): n = 0218BC1C(02013B38(zone)) (the zone's model set); 020140D8(x) picks the archives (0xE5, 0xB0,
# 0x15) or (0xE6, 0xB1, 0x14): 0218BB0C(BM, arc1, n) (model list), 0218BC40(BM, arc2, n, h2) (textures), 0218BB78(BM, k) (id map),
# 0218BD28(BM, h2) (animation resources), 0218BDD8(BM, BM+0x1B0) (animation objects). The model count is drawn 0..8 here (+0x1AC and
# the list header byte): in the closure every model allocates up to 5 blocks, and the bump allocator's blocks must stay in the arena.
fn 0218b8ac ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 range 0 8
  fix arg0 0x1402 u8 range 0 17
  stub 02013b38 force ret=pick+u32:0/1/2/0xD1/0xD2/0x100 args=v  # zonedata: the model set of a zone
  stub 0218bc1c ret=pick+u16:0/1/2/0x31 args=v  # in-area: model set -> member
  stub 020140d8 force ret=pick+u32:0/1 args=v  # predicate (archive choice)
  stub 0218bb0c ret=none args=ptr,v,v  # in-area: load the model list (BM, arc, member)
  stub 0218bc40 ret=u32 args=ptr,v,v,v  # in-area: load the textures (BM, arc, n, h2)
  stub 0218bb78 ret=none args=ptr,v  # in-area: build the id map (BM, k)
  stub 0218bd28 ret=none args=ptr,v  # in-area: animation resources (BM, h2)
  stub 0218bdd8 ret=none args=ptr,v  # in-area: animation objects (BM, S)
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap)
  stub 0204961c force ret=pick+u16:0/1/2/8 args=v  # archive member count (handle)
  stub 02049250 force ret=ptr:arg0:0x1400:1 args=v,v,v  # archive member load (handle, member, heap)
  stub 02049238 force ret=none args=v  # archive close (handle)
  stub 02046db8 force ret=u32 args=v,v  # g3d_system: texture resource (arc, member)
  stub 0218d0d8 force ret=none args=v,v  # field_g3d_mapper (G8) (resource, h2)
  stub 020470a8 force ret=pick+u32:0/1 args=v  # g3d_system: upload the resource
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218bee8 ret=none args=ptr,v,v,v  # in-area: animation resources of model i (BM, set, i, h2)
  stub 0218bfb4 ret=none args=ptr,v,v  # in-area: animation object (BM, obj, set)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i
  stub 02046c94 force ret=u32  # g3d_system: animation resource size
  stub 0218bc00 ret=u32 args=ptr,v  # in-area: second-table entry i
  stub 02046ce0 force ret=none args=v,v  # g3d_system: animation resource init (dst, data)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218bc8c ret=pick:0/1/2/3/4 args=ptr  # in-area: animation count of a header
  stub 0218bc10 ret=u32 args=ptr,v  # in-area: header entry i
  stub 02047400 force ret=u32 args=v,v,v  # g3d_system: model object (res, 0, textures)
  stub 02047564 force ret=u32 args=v,v,v  # g3d_system: animation (model, anim res, 0)
  stub 0204767c force ret=u32 args=v,in:16,v  # g3d_system: animated object (model, anims, 4)
  stub 0218c02c ret=none args=v,ptr  # in-area: init proc, kinds 0 / 2 (BM, obj)
  stub 0218c040 ret=none args=v,v  # in-area: init proc, kind 1 (BM, obj)
  stub 0218c08c ret=none args=ptr,v  # in-area: init proc, kind 3 (BM, obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)

# 0218B910(BM, x): the model record of model id x: 0218BBF4(+0x18, 0218BBD0(BM, x)).
fn 0218b910 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt range 0 399 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218bbd0 ret=mod:128 args=ptr,v  # in-area: the model index of id x (0..0x7F)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218B924(BM, i): thunk (`ldr r0, [r0, #0x18]; bx 0218BBF4`): the model record i of the list +0x18. i is drawn below 254 (the list's
# offset table; the original reads past it for larger i).
fn 0218b924 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt range 0 8 || range 0 253
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218B930(BM, x, &pos, &idx): r = 0218B910(BM, x); r+4 (the linked model) == 0xFFFF -> 0. Else *idx = 0218BBD0(BM, r+4); pos =
# {r+6, r+8, r+0xA} (s16 << 12); 1.
fn 0218b930 ret=u32 inputs=300
  nowork
  arg buf 0x10000
  arg u32 alt range 0 399 || any
  arg buf 0x10
  arg buf 0x10
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218b910 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: the model record of id x
  stub 0218bbd0 ret=mod:128 args=ptr,v  # in-area: the model index of id x (0..0x7F)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218B974(BM, h, list, n) (place the models of a map block): for each 16-byte entry j < n (signed): 0218BA04(BM, h, entry, c); when
# 0218B930(BM, id, &pos, &idx) == 1 (the model has a linked model) the linked one is placed too: pos += {x, y, -z}, rot = entry rot,
# c + 1 -> 0218C4A0(BM, h, &{idx, pos, rot}, c + 1); c += 1 per model. Answers c. ENTRY: +0 / +4 / +8 fx32 x, y, z, +0xC u16 rot,
# +0xE big-endian u16 model id.
fn 0218b974 ret=u32 inputs=300
  nowork
  arg buf 0x10000
  arg u32 alt oneof 0x02123450 0x02123460 0x02123470 || any
  arg buf 0x100
  arg u32 alt range 0 4 || range 0 16 || oneof 0x80000000 0xFFFFFFFF
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg2 0xE u8 alt range 0 1 || any rep=16,16
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218ba04 ret=u32 args=ptr,v,ptr,v  # in-area: place a model (BM, h, entry, c)
  stub 0218b930 ret=pick:0/1 args=ptr,v,out:12,out:4  # in-area: the linked model (BM, x, &pos, &idx)
  stub 0218c4a0 ret=u32 args=ptr,v,in:18,v  # in-area: place a model (BM, h, &desc, idx)
  stub 0218bbd0 ret=mod:128 args=ptr,v  # in-area: the model index of id x (0..0x7F)
  stub 0218b910 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: the model record of id x
  stub 021693d4 force ret=none args=v,in:18,v  # field_g3d_map (G8): place a model (h, &desc, idx)
  stub 02169444 force ret=u32 args=v,v  # field_g3d_map (G8): the state word of a placed model (h, idx)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218B9FC(BM, h): thunk to 0218C4EC (remove the placed models of h).
fn 0218b9fc ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 alt oneof 0x02123450 0x02123460 0x02123470 || const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218c4ec ret=none args=ptr,v  # in-area: remove the models of h (BM, h)
  stub 0218c518 ret=none args=v,ptr  # in-area: remove a placed model (BM, e)
  stub 0216940c force ret=none args=v,v  # field_g3d_map (G8): remove a placed model (h, idx)
  stub 0218c490 ret=none args=v,ptr  # in-area: clear an entry (BM, e)

# 0218BA04(BM, h, entry, c): 0218C4A0(BM, h, &{0218BBD0(BM, id), x, y, -z, rot}, c) (place one model); answers the entry.
fn 0218ba04 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 any
  arg buf 0x10
  arg u32 any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg2 0xE u8 alt range 0 1 || any
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218bbd0 ret=mod:128 args=ptr,v  # in-area: the model index of id x (0..0x7F)
  stub 0218c4a0 ret=u32 args=ptr,v,in:18,v  # in-area: place a model (BM, h, &desc, idx)
  stub 021693d4 force ret=none args=v,in:18,v  # field_g3d_map (G8): place a model (h, &desc, idx)
  stub 02169444 force ret=u32 args=v,v  # field_g3d_map (G8): the state word of a placed model (h, idx)

# 0218BA48(BM, D): the door slot holding D = 0.
fn 0218ba48 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 alt oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 || const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40

# 0218BA70(BM, D): the first empty door slot = D.
fn 0218ba70 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 alt oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 || const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40

# 0218BA98(BM, code): (code & 0xFFF0) == 0xFFF0 and i = code & 0xF below 7 -> door slot i; else 0.
fn 0218ba98 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt range 0xFFF0 0xFFFF || range 0x1FFF0 0x1FFFF || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40

# 0218BACC(BM, D): D == 0 -> 0; the door slot i holding D -> 0xFFF0 | i; else 0.
fn 0218bacc ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 || const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40

# 0218BB04(BM): BM + 0x1B0.
fn 0218bb04 ret=u32 inputs=20
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40

# 0218BB0C(BM, arc, member): h = 020490F4(arc, (heap & 0x7FFF) | 0x8000); member >= 0204961C(h) (count) -> member = 0; +0x18 =
# 02049250(h, member, heap) (the model list); +0x1AC = list[2] >> 1, 0x80 when above 0x190 (dead: list[2] >> 1 is at most 0x7F;
# the 2 instructions coverage misses); 02049238(h).
fn 0218bb0c ret=none inputs=200
  nowork
  arg buf 0x10000
  arg u32 any
  arg u32 alt range 0 8 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 020490f4 force ret=u32 args=v,v  # archive open (arc, heap)
  stub 0204961c force ret=pick+u16:0/1/2/8 args=v  # archive member count (handle)
  stub 02049250 force ret=ptr:arg0:0x1400:1 args=v,v,v  # archive member load (handle, member, heap)
  stub 02049238 force ret=none args=v  # archive close (handle)

# 0218BB64(BM): Heap_Free(+0x18); +0x1AC = 0.
fn 0218bb64 ret=none inputs=40
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 020307b0 force ret=none args=v  # Heap_Free

# 0218BB78(BM, k): the id map +0x1C (400 bytes) = 0x80; map[record(i)+0] = i for i < +0x1AC (u8 i); the empty bytes = map[k]. k is
# drawn below 400 (the map's size; the original reads past it for larger k).
fn 0218bb78 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 range 0 399
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218BBD0(BM, x): x >= 400 -> x = 0; v = map[x]; v >= 0x80 or v >= +0x1AC -> 0; v.
fn 0218bbd0 ret=u32 inputs=300
  nowork
  arg buf 0x10000
  arg u32 alt range 0 399 || near 400 2 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any

# 0218BBF4(list, i): list + list[4 + 4 i]. i is drawn inside the 0x400-byte offset table.
fn 0218bbf4 ret=u32 inputs=100
  nowork
  arg buf 0x400
  arg u32 range 0 0xFE

# 0218BC00(list, i): list + list[4 + 4 ((list[2] >> 1) + i)] (the second table, after the model records).
fn 0218bc00 ret=u32 inputs=100
  nowork
  arg buf 0x400
  arg u32 range 0 127

# 0218BC10(hdr, i): hdr + hdr[4 + 4 i].
fn 0218bc10 ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg u32 alt range 0 3 || range 0 0x3E

# 0218BC1C(a): a >= 210 -> (a - 210) & 0xFFFF; a >= 2 -> ((a - 2) / 4) & 0xFFFF (signed, rounding toward zero); else 0.
fn 0218bc1c ret=u32 inputs=200
  nowork
  arg u32 alt range 0 2 || near 210 3 || range 0 300 || any

# 0218BC40(BM, arc, n, h2): +0x1D0 = 02046DB8(arc, n) (texture resource); h2 != 0 -> 0218D0D8(+0x1D0, h2); answers 020470A8(+0x1D0).
fn 0218bc40 ret=u32 inputs=100
  nowork
  arg buf 0x10000
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 02046db8 force ret=u32 args=v,v  # g3d_system: texture resource (arc, member)
  stub 0218d0d8 force ret=none args=v,v  # field_g3d_mapper (G8) (resource, h2)
  stub 020470a8 force ret=pick+u32:0/1 args=v  # g3d_system: upload the resource

# 0218BC6C(BM): +0x1D0 != 0 -> 020470D0(+0x1D0), 02046EFC(+0x1D0), +0x1D0 = 0.
fn 0218bc6c ret=none inputs=60
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 020470d0 force ret=none args=v  # g3d_system: release the upload
  stub 02046efc force ret=none args=v  # g3d_system: delete the resource

# 0218BC8C(hdr): the number of the 4 words hdr+4.. that are not -1.
fn 0218bc8c ret=u32 inputs=100
  nowork
  arg buf 0x20
  fix arg0 4 u32 alt const 0xFFFFFFFF || any rep=4,4

# 0218BCAC(r): r + 0x10 (the animation header of a model record).
fn 0218bcac ret=u32 inputs=20
  nowork
  arg u32 any

# 0218BCB0(r): u16 r+2 (the model kind) below 16 -> ROM table 0x021D3DC4[kind] (a 16-byte local copy), else 0.
fn 0218bcb0 ret=u32 inputs=200
  nowork
  arg buf 0x10
  fix arg0 2 u16 alt range 0 16 || any

# 0218BCE4(t, s): t+0 = 5; t+0xD = s (u8); tail call 0218BCF4(t).
fn 0218bce4 ret=none inputs=60
  nowork
  arg buf 0x20
  arg u32 any
  stub 0218bcf4 ret=none args=ptr  # in-area: time of day update (t)
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # the time zone of the hour in season s (0..4)

# 0218BCF4(t) (time of day TOD: +0 zone, +4 previous, +8 changed, +0xC u8 animation index, +0xD u8 season): +4 = +0; +0 =
# 020113D8(+0xD); +8 = (+0 != +4); +0xC = ROM 0x021D3D50[+0] (8 bytes).
fn 0218bcf4 ret=none inputs=100
  nowork
  arg buf 0x20
  fix arg0 0 u32 alt range 0 4 || any
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # the time zone of the hour in season s (0..4)

# 0218BD20(t): +8 (the zone changed).
fn 0218bd20 ret=u32 inputs=40
  nowork
  arg buf 0x20

# 0218BD24(t): u8 +0xC (the animation index of the zone).
fn 0218bd24 ret=u32 inputs=40
  nowork
  arg buf 0x20

# 0218BD28(BM, h2): n = +0x1AC; n != 0 -> +0x1C8 = n; +0x1CC = Heap_AllocDebug(heap, 0x18 n, 1, file, 0x45F); 0218BEE8(BM, +0x1CC +
# 0x18 i, i, h2) for i < n (u8 i).
fn 0218bd28 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218bee8 ret=none args=ptr,v,v,v  # in-area: animation resources of model i (BM, set, i, h2)
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 02046c94 force ret=u32  # g3d_system: animation resource size
  stub 0218bc00 ret=u32 args=ptr,v  # in-area: second-table entry i
  stub 02046ce0 force ret=none args=v,v  # g3d_system: animation resource init (dst, data)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218bc8c ret=pick:0/1/2/3/4 args=ptr  # in-area: animation count of a header
  stub 0218bc10 ret=u32 args=ptr,v  # in-area: header entry i
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218BD94(BM): +0x1CC != 0 -> 0218BF84(+0x1CC + 0x18 i) for i < +0x1C8; Heap_Free(+0x1CC); +0x1C8 = +0x1CC = 0.
fn 0218bd94 ret=none inputs=100
  nowork
  arg buf 0x10000
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1CC u32 alt addrof arg0 0x2000 || const 0
  stub 0218bf84 ret=none args=ptr  # in-area: free an animation resource set
  stub 020307b0 force ret=none args=v  # Heap_Free

# 0218BDD8(BM, S): n = +0x1AC; n != 0 -> S+4 = n; S+0 = Heap_AllocDebug(heap, 8 n, 1, file, 0x490); +0x1D4 = n; +0x1D8 =
# Heap_AllocDebug(heap, 0x18 n, 1, file, 0x494); for i < n: 0218BFB4(BM, +0x1D8 + 0x18 i, +0x1CC + 0x18 i), S+0[i] = {obj i +0, 0}.
fn 0218bdd8 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 alt addrof arg0 0x1B0 || addrof arg0 0x3F80
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218bfb4 ret=none args=ptr,v,v  # in-area: animation object (BM, obj, set)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 02047400 force ret=u32 args=v,v,v  # g3d_system: model object (res, 0, textures)
  stub 02047564 force ret=u32 args=v,v,v  # g3d_system: animation (model, anim res, 0)
  stub 0204767c force ret=u32 args=v,in:16,v  # g3d_system: animated object (model, anims, 4)
  stub 0218c02c ret=none args=v,ptr  # in-area: init proc, kinds 0 / 2 (BM, obj)
  stub 0218c040 ret=none args=v,v  # in-area: init proc, kind 1 (BM, obj)
  stub 0218c08c ret=none args=ptr,v  # in-area: init proc, kind 3 (BM, obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)

# 0218BE84(BM, S): S+0 / S+8 != 0 -> Heap_Free, = 0; +0x1D8 != 0 -> 0218C0D8(+0x1D8 + 0x18 i) for i < +0x1D4, Heap_Free(+0x1D8),
# +0x1D4 = +0x1D8 = 0.
fn 0218be84 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 alt addrof arg0 0x1B0 || addrof arg0 0x3F80
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1D8 u32 alt addrof arg0 0x2C00 || const 0
  fix arg0 0x3F80 u32 alt const 0 || any
  fix arg0 0x3F88 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 0218c0d8 ret=none args=ptr  # in-area: delete an animation object
  stub 0204779c force ret=pick:0/1/2/4 args=v  # g3d_system: animation count (a u16; closed set outside 0218C0D8, see the header)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 0204762c force ret=none args=v  # g3d_system: delete an animation
  stub 02047788 force ret=u32 args=v  # g3d_system: model of the object
  stub 02047738 force ret=none args=v  # g3d_system: delete the object
  stub 02047504 force ret=none args=v  # g3d_system: delete the model

# 0218BEE8(BM, set, i, h2) (animation resources of model i): set+0 = r = 0218B924(BM, i); set+4 = Heap_AllocDebug(heap, 02046C94(), 1,
# file, 0x4D0), 02046CE0(set+4, 0218BC00(+0x18, i)); hdr = 0218BCAC(r); for j < 4: j < 0218BC8C(hdr) -> set+8+4j = Heap_AllocDebug(heap,
# 02046C94(), 1, file, 0x4DB), 02046CE0(set+8+4j, 0218BC10(hdr, j)); else set+8+4j = 0.
fn 0218bee8 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8
  arg u32 alt range 0 8 || range 0 126
  arg u32 any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 02046c94 force ret=u32  # g3d_system: animation resource size
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218bc00 ret=u32 args=ptr,v  # in-area: second-table entry i
  stub 02046ce0 force ret=none args=v,v  # g3d_system: animation resource init (dst, data)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218bc8c ret=pick:0/1/2/3/4 args=ptr  # in-area: animation count of a header
  stub 0218bc10 ret=u32 args=ptr,v  # in-area: header entry i
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218BF84(set): Heap_Free and clear set+8+4j (j < 4) and set+4 when set.
fn 0218bf84 ret=none inputs=60
  nowork
  arg buf 0x20
  fix arg0 4 u32 alt const 0 || any rep=5,4
  stub 020307b0 force ret=none args=v  # Heap_Free

# 0218BFB4(BM, obj, set) (animation object of a model): hdr = 0218BCAC(set+0); a = 02047400(set+4, 0, +0x1D0); local[j] = set+8+4j ?
# 02047564(a, set+8+4j, 0) : 0 (j < 4); obj+0 = 0204767C(a, local, 4); obj+4 = set; the init proc of the animation kind (ROM table
# 0x021D3D58[hdr+0], 12 bytes per kind: 0218C02C / 0218C040 / 0218C02C / 0218C08C)(BM, obj). The sets' animation words are 0 or
# any.
fn 0218bfb4 ret=none inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E12C00 0x02E12C18 0x02E12C30 0x02E12C48 0x02E12C60 0x02E12C78 0x02E12C90 0x02E12CA8
  arg u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x2008 u32 alt const 0 || any rep=8,0x18
  fix arg0 0x200C u32 alt const 0 || any rep=8,0x18
  fix arg0 0x2010 u32 alt const 0 || any rep=8,0x18
  fix arg0 0x2014 u32 alt const 0 || any rep=8,0x18
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 02047400 force ret=u32 args=v,v,v  # g3d_system: model object (res, 0, textures)
  stub 02047564 force ret=u32 args=v,v,v  # g3d_system: animation (model, anim res, 0)
  stub 0204767c force ret=u32 args=v,in:16,v  # g3d_system: animated object (model, anims, 4)
  stub 0218c02c ret=none args=v,ptr  # in-area: init proc, kinds 0 / 2 (BM, obj)
  stub 0218c040 ret=none args=v,v  # in-area: init proc, kind 1 (BM, obj)
  stub 0218c08c ret=none args=ptr,v  # in-area: init proc, kind 3 (BM, obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)

# 0218C02C(BM, obj) (init, kinds 0 / 2): the 4 states obj+8.. = 0.
fn 0218c02c ret=none inputs=40
  nowork
  arg u32 any
  arg buf 0x20

# 0218C040(BM, obj) (init, kind 1): c = header+2 (the frames per state); states j < min(4, c): 020477A4(obj+0, j), 02047804(obj+0, j),
# = 1; the rest = 0.
fn 0218c040 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 oneof 0x02E32C00 0x02E32C18 0x02E32C30 0x02E32C48 0x02E32C60 0x02E32C78 0x02E32C90 0x02E32CA8
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)

# 0218C08C(BM, obj) (init, kind 3): z = 0218BD24(BM+8) (the time-of-day animation); state z: 020477A4 / 02047804(obj+0, z), = 1; the
# other states = 0.
fn 0218c08c ret=none inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E12C00 0x02E12C18 0x02E12C30 0x02E12C48 0x02E12C60 0x02E12C78 0x02E12C90 0x02E12CA8
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)

# 0218C0D8(obj): obj+0 != 0 -> for i < 0204779C(obj+0): 02047790(obj+0, i) != 0 -> 0204762C(it); r = 02047788(obj+0); 02047738(obj+0);
# obj+0 = obj+4 = 0; 02047504(r).
fn 0218c0d8 ret=none inputs=100
  nowork
  arg buf 0x20
  fix arg0 0 u32 alt const 0 || any
  stub 0204779c force ret=pick+u16:0/1/2/4 args=v  # g3d_system: animation count (u16)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 0204762c force ret=none args=v  # g3d_system: delete an animation
  stub 02047788 force ret=u32 args=v  # g3d_system: model of the object
  stub 02047738 force ret=none args=v  # g3d_system: delete the object
  stub 02047504 force ret=none args=v  # g3d_system: delete the model

# 0218C124(BM, obj) (one frame): obj+0 != 0 -> the frame proc of the animation kind (ROM table 0x021D3D5C: 0218C150 / 0218C1A4 / 0218C1C4
# / 0218C154)(BM, obj).
fn 0218c124 ret=none inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E12C00 0x02E12C18 0x02E12C30 0x02E12C48 0x02E12C60 0x02E12C78 0x02E12C90 0x02E12CA8 0x02E1380C 0x02E1386C
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c150 ret=none  # in-area: frame proc, kind 0 (empty)
  stub 0218c1a4 ret=none args=v,ptr  # in-area: frame proc, kind 1 (BM, obj)
  stub 0218c1c4 ret=none args=v,ptr  # in-area: frame proc, kind 2 (BM, obj)
  stub 0218c154 ret=none args=ptr,v  # in-area: frame proc, kind 3 (BM, obj)
  stub 02047910 force ret=none args=v,v,v  # g3d_system: animation advance (obj, i, step)
  stub 020478b8 force ret=pick+u32:0/1 args=v,v,v  # g3d_system: animation advance, 0 at the end (obj, i, step)
  stub 0204784c force ret=none args=v,v,v  # g3d_system: animation frame (obj, i, &frame or 0)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)
  stub 0218bd20 ret=pick:0/1 args=ptr  # in-area: the time zone changed (t)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)

# 0218C150(): empty (frame proc of kind 0).
fn 0218c150 ret=none inputs=10
  nowork

# 0218C154(BM, obj) (frame, kind 3): z = 0218BD24(BM+8); 0218BD20(BM+8) (the zone changed) -> 0218C388(obj), 020477A4 / 02047804(obj+0,
# z), state z = 1; else 02047910(obj+0, z, 0x1000) (advance).
fn 0218c154 ret=none inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E12C00 0x02E12C18 0x02E12C30 0x02E12C48 0x02E12C60 0x02E12C78 0x02E12C90 0x02E12CA8
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)
  stub 0218bd20 ret=pick:0/1 args=ptr  # in-area: the time zone changed (t)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047910 force ret=none args=v,v,v  # g3d_system: animation advance (obj, i, step)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)

# 0218C1A4(BM, obj) (frame, kind 1): 02047910(obj+0, j, 0x1000) for j < 4.
fn 0218c1a4 ret=none inputs=60
  nowork
  arg u32 any
  arg buf 0x20
  stub 02047910 force ret=none args=v,v,v  # g3d_system: animation advance (obj, i, step)

# 0218C1C4(BM, obj) (frame, kind 2): per state j < 4: 1 -> 02047910(obj+0, j, 0x1000); 3 -> 020478B8(obj+0, j, 0x1000) == 0 -> 2; 4 ->
# 020478B8(obj+0, j, -0x1000) == 0 -> 0204784C(obj+0, j, 0), 2; others nothing.
fn 0218c1c4 ret=none inputs=300
  nowork
  arg u32 any
  arg buf 0x20
  fix arg1 8 u32 alt range 0 5 || any rep=4,4
  stub 020478b8 force ret=pick+u32:0/1 args=v,v,v  # g3d_system: animation advance, 0 at the end (obj, i, step)
  stub 0204784c force ret=none args=v,v,v  # g3d_system: animation frame (obj, i, &frame or 0)
  stub 02047910 force ret=none args=v,v,v  # g3d_system: animation advance (obj, i, step)

# 0218C23C(obj, k) (play forward once): c = header+2; for j < c: s = k c + j: 020477A4(obj+0, s), 02047804(obj+0, s), state s = 3. k is drawn 0..3
# (0218C3B0 bounds it; the states past 3 are the words after the object).
fn 0218c23c ret=none inputs=200
  nowork
  arg u32 oneof 0x02E32C00 0x02E32C18 0x02E32C30 0x02E32C48 0x02E32C60 0x02E32C78 0x02E32C90 0x02E32CA8
  arg u32 range 0 3
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)

# 0218C290(obj, k) (loop): c = header+2; for j < c: s = k c + j: 020477A4(obj+0, s), 02047804(obj+0, s), state s = 1. k is drawn 0..3
# (0218C3B0 bounds it; the states past 3 are the words after the object).
fn 0218c290 ret=none inputs=200
  nowork
  arg u32 oneof 0x02E32C00 0x02E32C18 0x02E32C30 0x02E32C48 0x02E32C60 0x02E32C78 0x02E32C90 0x02E32CA8
  arg u32 range 0 3
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)

# 0218C2E4(obj, k) (play backward once): c = header+2; for j < c: s = k c + j: 020477A4(obj+0, s); v = u16 (02047674(02047790(obj+0,
# s))+8)+4 << 12; 0204784C(obj+0, s, &v) (the last frame); state s = 4. k is drawn 0..3 (as 0218C23C).
fn 0218c2e4 ret=none inputs=200
  nowork
  arg u32 oneof 0x02E32C00 0x02E32C18 0x02E32C30 0x02E32C48 0x02E32C60 0x02E32C78 0x02E32C90 0x02E32CA8
  arg u32 range 0 3
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg2:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C350(obj, k) (stop): c = header+2; the states k c + j (j < c) that are not 0 = 2. k is drawn 0..3 (as 0218C23C).
fn 0218c350 ret=none inputs=200
  nowork
  arg u32 oneof 0x02E32C00 0x02E32C18 0x02E32C30 0x02E32C48 0x02E32C60 0x02E32C78 0x02E32C90 0x02E32CA8
  arg u32 range 0 3
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)

# 0218C388(obj): the states j < 4 that are not 0: 020477D4(obj+0, j), = 0.
fn 0218c388 ret=none inputs=100
  nowork
  arg buf 0x20
  fix arg0 8 u32 alt const 0 || any rep=4,4
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)

# 0218C3B0(obj, k, mode): k = k & 0xFF, 0 when >= 4; the state proc of the animation kind (ROM table 0x021D3D60: 0218C438 / 0218C438 /
# 0218C3E0 / 0218C438)(obj, k, mode).
fn 0218c3b0 ret=none inputs=300
  nowork
  arg u32 oneof 0x02E42C00 0x02E42C18 0x02E42C30 0x02E42C48 0x02E42C60 0x02E42C78 0x02E42C90 0x02E42CA8
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 5 || any
  arg buf 0x10000
  fix arg3 0 u16 alt range 0 4 || any
  fix arg3 8 u32 alt range 0 4 || any
  fix arg3 0xC u32 alt range 0 4 || any
  fix arg3 0x10 u32 alt const 0 || const 1 || any
  fix arg3 0x14 u8 alt range 0 3 || any
  fix arg3 0x15 u8 alt range 0 3 || any
  fix arg3 0x18 u32 addrof arg3 0x1400
  fix arg3 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg3 0x1AC u16 alt range 0 8 || range 0 127
  fix arg3 0x1B0 u32 alt const 0 || any
  fix arg3 0x1B8 u32 alt const 0 || any
  fix arg3 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg3 0x1CC u32 addrof arg3 0x2000
  fix arg3 0x1D0 u32 alt const 0 || any
  fix arg3 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg3 0x1D8 u32 addrof arg3 0x2C00
  fix arg3 0x1DC u32 alt const 0 || oneof 0x02E43800 0x02E43860 0x02E438C0 0x02E43920 0x02E43980 0x02E439E0 0x02E43A40 rep=7,4
  fix arg3 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg3 0x204 u32 oneof 0x02E43C00 0x02E43C20 0x02E43C40 0x02E43C60 0x02E43C80 0x02E43CA0 0x02E43CC0 0x02E43CE0 0x02E43D00 0x02E43D20 0x02E43D40 0x02E43D60 0x02E43D80 0x02E43DA0 0x02E43DC0 0x02E43DE0 rep=288,16
  fix arg3 0x1402 u8 alt range 0 17 || any
  fix arg3 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg3 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg3 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1810 u8 range 0 3 rep=8,0x24
  fix arg3 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg3 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x2000 u32 oneof 0x02E41800 0x02E41824 0x02E41848 0x02E4186C 0x02E41890 0x02E418B4 0x02E418D8 0x02E418FC rep=127,0x18
  fix arg3 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg3 0x2C04 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=127,0x18
  fix arg3 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x3800 u32 addrof arg3 0 rep=7,0x60
  fix arg3 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg3 0x3808 u32 alt const 0 || oneof 0x02E401F8 0x02E40208 0x02E40218 0x02E40228 0x02E413E8 rep=7,0x60
  fix arg3 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg3 0x3810 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=7,0x60
  fix arg3 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg3 0x3F08 u32 addrof arg3 0x3F40
  stub 0218bcac ret=pick:0x02E41810/0x02E41834/0x02E41858/0x02E4187C/0x02E418A0/0x02E418C4/0x02E418E8/0x02E4190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c438 ret=none  # in-area: state proc (empty)
  stub 0218c3e0 ret=none args=v,v,v  # in-area: state proc, kind 2 (obj, k, mode)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg3:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C3E0(obj, k, mode) (state proc, kind 2): mode 0: 0218C388(obj), 0218C23C(obj, k); 1: 0218C388, 0218C2E4; 2: 0218C388, 0218C290;
# 3: 0218C350(obj, k); 4: 0218C388; others nothing. k is drawn 0..3 (0218C3B0 bounds it).
fn 0218c3e0 ret=none inputs=300
  nowork
  arg u32 oneof 0x02E42C00 0x02E42C18 0x02E42C30 0x02E42C48 0x02E42C60 0x02E42C78 0x02E42C90 0x02E42CA8
  arg u32 range 0 3
  arg u32 alt range 0 5 || any
  arg buf 0x10000
  fix arg3 0 u16 alt range 0 4 || any
  fix arg3 8 u32 alt range 0 4 || any
  fix arg3 0xC u32 alt range 0 4 || any
  fix arg3 0x10 u32 alt const 0 || const 1 || any
  fix arg3 0x14 u8 alt range 0 3 || any
  fix arg3 0x15 u8 alt range 0 3 || any
  fix arg3 0x18 u32 addrof arg3 0x1400
  fix arg3 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg3 0x1AC u16 alt range 0 8 || range 0 127
  fix arg3 0x1B0 u32 alt const 0 || any
  fix arg3 0x1B8 u32 alt const 0 || any
  fix arg3 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg3 0x1CC u32 addrof arg3 0x2000
  fix arg3 0x1D0 u32 alt const 0 || any
  fix arg3 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg3 0x1D8 u32 addrof arg3 0x2C00
  fix arg3 0x1DC u32 alt const 0 || oneof 0x02E43800 0x02E43860 0x02E438C0 0x02E43920 0x02E43980 0x02E439E0 0x02E43A40 rep=7,4
  fix arg3 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg3 0x204 u32 oneof 0x02E43C00 0x02E43C20 0x02E43C40 0x02E43C60 0x02E43C80 0x02E43CA0 0x02E43CC0 0x02E43CE0 0x02E43D00 0x02E43D20 0x02E43D40 0x02E43D60 0x02E43D80 0x02E43DA0 0x02E43DC0 0x02E43DE0 rep=288,16
  fix arg3 0x1402 u8 alt range 0 17 || any
  fix arg3 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg3 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg3 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1810 u8 range 0 3 rep=8,0x24
  fix arg3 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg3 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x2000 u32 oneof 0x02E41800 0x02E41824 0x02E41848 0x02E4186C 0x02E41890 0x02E418B4 0x02E418D8 0x02E418FC rep=127,0x18
  fix arg3 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg3 0x2C04 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=127,0x18
  fix arg3 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x3800 u32 addrof arg3 0 rep=7,0x60
  fix arg3 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg3 0x3808 u32 alt const 0 || oneof 0x02E401F8 0x02E40208 0x02E40218 0x02E40228 0x02E413E8 rep=7,0x60
  fix arg3 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg3 0x3810 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=7,0x60
  fix arg3 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg3 0x3F08 u32 addrof arg3 0x3F40
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 0218bcac ret=pick:0x02E41810/0x02E41834/0x02E41858/0x02E4187C/0x02E418A0/0x02E418C4/0x02E418E8/0x02E4190C args=v  # in-area: r + 0x10 (the animation header)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg3:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C438(): empty (state proc of kinds 0 / 1 / 3).
fn 0218c438 ret=none inputs=10
  nowork

# 0218C43C(obj, k): k = k & 0xFF, 0 when >= 4; state k: 0 -> 1, 1 -> 0, 2 -> 1, 3 / 4 -> 0, others 0 (1 = stopped).
fn 0218c43c ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg u32 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 5 || any rep=4,4

# 0218C47C(obj, j): 02047828(obj+0, j, &v); v (the caller's r3 before the call, overwritten by the callee).
fn 0218c47c ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg u32 any
  stub 02047828 force ret=u32 args=v,v,out:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C490(BM, e) (clear a placed-model entry): e+0 = 0; +4 = -1; u16 +8 = +0xA = 0; +0xC = 0.
fn 0218c490 ret=none inputs=40
  nowork
  arg u32 any
  arg buf 0x10

# 0218C4A0(BM, h, desc, idx) (place a model): the first free entry e of the 0x120 at +0x1F8 (e+0 == 0): 021693D4(h, desc, idx); e+0 = h;
# +4 = -1; u16 +8 = idx; u16 +0xA = 1; +0xC = 02169444(h, idx); answers e; none free -> 0. The entries' handles are zero rarely here
# (1 in 150) so the "none free" answer occurs.
fn 0218c4a0 ret=u32 inputs=600
  nowork
  arg buf 0x10000
  arg u32 any
  arg buf 0x20
  arg u32 any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1F8 u32 alt const 0 || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF || range 1 0xFFFFFFFF rep=288,16
  stub 021693d4 force ret=none args=v,in:18,v  # field_g3d_map (G8): place a model (h, &desc, idx)
  stub 02169444 force ret=u32 args=v,v  # field_g3d_map (G8): the state word of a placed model (h, idx)

# 0218C4EC(BM, h): 0218C518(BM, e) for every entry e with e+0 == h.
fn 0218c4ec ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 alt oneof 0x02123450 0x02123460 0x02123470 || const 0 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  stub 0218c518 ret=none args=v,ptr  # in-area: remove a placed model (BM, e)
  stub 0216940c force ret=none args=v,v  # field_g3d_map (G8): remove a placed model (h, idx)
  stub 0218c490 ret=none args=v,ptr  # in-area: clear an entry (BM, e)

# 0218C518(BM, e): 0216940C(e+0, u16 e+8); 0218C490(BM, e).
fn 0218c518 ret=none inputs=60
  nowork
  arg u32 any
  arg buf 0x10
  stub 0216940c force ret=none args=v,v  # field_g3d_map (G8): remove a placed model (h, idx)
  stub 0218c490 ret=none args=v,ptr  # in-area: clear an entry (BM, e)

# 0218C530(e): *(e+0xC) == -1 -> (e+4) & 0xFF; else *(e+0xC) & 0xFF (the model's animation state).
fn 0218c530 ret=u32 inputs=100
  nowork
  arg buf 0x20
  fix arg0 0xC u32 addrof arg0 0x10
  fix arg0 0x10 u32 alt const 0xFFFFFFFF || any

# 0218C54C(e, f): f != 0 -> *(e+0xC) = e+4, e+4 = -1, u16 e+0xA = 1; else e+4 = *(e+0xC), *(e+0xC) = -1, u16 e+0xA = 0.
fn 0218c54c ret=none inputs=100
  nowork
  arg buf 0x20
  arg u32 alt const 0 || any
  fix arg0 0xC u32 addrof arg0 0x10

# 0218C574(BM, e, x): *(e+0xC) = 0218BBD0(BM, x).
fn 0218c574 ret=none inputs=100
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8
  arg u32 alt range 0 399 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218bbd0 ret=mod:128 args=ptr,v  # in-area: the model index of id x (0..0x7F)

# 0218C584(BM, rect, x, &n) -> list: L = Heap_AllocDebug(heap, 0x80, 1, file, 0x749); for every used entry e (e+0 != 0) with x == 0 or
# 0218C674(BM, e) == x: p = 02169510(e+0) + *(e+0xC)+4 (0207C9D0); rect+8 <= p.x <= rect+0xC and rect+0 <= p.z <= rect+4 (signed) ->
# L[n++] = e; *(&n) = n; answers L (the original writes past the 0x80 bytes for more than 32 hits).
fn 0218c584 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg buf 0x10
  arg u32 alt const 0 || range 0 10
  arg buf 0x10
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg1 0 u32 alt oneof 0 0x10000 0x20000 0x30000 0xFFFF0000 0xFFFE0000 0x50000 || any rep=4,4
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218c674 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr,v  # in-area: the door kind of an entry (BM, e)
  stub 02169510 force ret=none args=v,out:12  # field_g3d_map (G8): the block position (h, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outw:12:0/0x10000/0x20000/0x30000/0xFFFF0000/0xFFFE0000/0x50000  # VEC_Add(a, b, out) (FX)
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 0218bcb0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr  # in-area: door kind of a record (ROM table)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218C638(BM, x, rect): L = 0218C584(BM, rect, x, &n); r = L[0]; Heap_Free(L); r (also when n == 0: what the list block holds).
fn 0218c638 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt const 0 || range 0 10
  arg buf 0x10
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg2 0 u32 alt oneof 0 0x10000 0x20000 0x30000 0xFFFF0000 0xFFFE0000 0x50000 || any rep=4,4
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218c584 ret=ptr:arg0:0x3F80:1 args=ptr,in:16,v,out:4  # in-area: entries in a box (BM, &rect, x, &n)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218c674 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr,v  # in-area: the door kind of an entry (BM, e)
  stub 02169510 force ret=none args=v,out:12  # field_g3d_map (G8): the block position (h, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outw:12:0/0x10000/0x20000/0x30000/0xFFFF0000/0xFFFE0000/0x50000  # VEC_Add(a, b, out) (FX)
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 0218bcb0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr  # in-area: door kind of a record (ROM table)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218C654(BM, x, pos): 0218C9C4(&rect, pos) (a 4 x 6 grid square box around pos); 0218C638(BM, x, &rect).
fn 0218c654 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt const 0 || range 0 10
  arg buf 0x10
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg2 0 u32 alt oneof 0 0x10000 0x20000 0x30000 0xFFFF0000 0xFFFE0000 0x50000 || any
  fix arg2 8 u32 alt oneof 0 0x10000 0x20000 0x30000 0xFFFF0000 0xFFFE0000 0x50000 || any
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218c9c4 ret=none args=out:16,ptr  # in-area: box around a position (&rect, pos)
  stub 0218c638 ret=pick:0/0x02E101F8/0x02E10208/0x02E10218/0x02E10228/0x02E113E8 args=ptr,v,in:16  # in-area: the first entry in a box (BM, x, &rect)
  stub 0218c584 ret=ptr:arg0:0x3F80:1 args=ptr,in:16,v,out:4  # in-area: entries in a box (BM, &rect, x, &n)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218c674 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr,v  # in-area: the door kind of an entry (BM, e)
  stub 02169510 force ret=none args=v,out:12  # field_g3d_map (G8): the block position (h, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outw:12:0/0x10000/0x20000/0x30000/0xFFFF0000/0xFFFE0000/0x50000  # VEC_Add(a, b, out) (FX)
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 0218bcb0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr  # in-area: door kind of a record (ROM table)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218C674(BM, e): v = 0218C530(e) (0 when >= 0x80 or >= +0x1AC); 0218BCB0(0218B924(BM, v)) (the door kind of its model).
fn 0218c674 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 0218bcb0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr  # in-area: door kind of a record (ROM table)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218C69C(BM, e, k, mode): v = 0218C530(e) (bounded as 0218C674); 0218C3B0(+0x1D8 + 0x18 v, k, mode).
fn 0218c69c ret=none inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 5 || any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218c3b0 ret=none args=v,v,v  # in-area: set the animation state (obj, k, mode)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c438 ret=none  # in-area: state proc (empty)
  stub 0218c3e0 ret=none args=v,v,v  # in-area: state proc, kind 2 (obj, k, mode)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg0:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C6D0(e, out): 02169510(e+0, &p); 0207C9D0(*(e+0xC)+4, &p, out) (the model position).
fn 0218c6d0 ret=none inputs=100
  nowork
  arg buf 0x20
  arg buf 0x10
  fix arg0 0xC u32 addrof arg0 0x10
  stub 02169510 force ret=none args=v,out:12  # field_g3d_map (G8): the block position (h, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outw:12:0/0x10000/0x20000/0x30000/0xFFFF0000/0xFFFE0000/0x50000  # VEC_Add(a, b, out) (FX)

# 0218C6F4(BM, e, j): v = 0218C530(e) (bounded as 0218C674); 0218C47C(+0x1D8 + 0x18 v, j).
fn 0218c6f4 ret=u32 inputs=100
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8
  arg u32 any
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218c47c ret=u32 args=ptr,v  # in-area: animation frame (obj, j)
  stub 02047828 force ret=u32 args=v,v,out:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C724(BM, e) -> D (the door object of a placed model): D = Heap_AllocDebug(heap, 0x60, 0, file, 0x7E9); +0 = BM; +8 = e; +4 = 0xFFFF;
# t = *(e+0xC); 0207AF1C(D+0x3C, sin, cos of u16 t+0x10) (rotation, FX_SinCosTable_); D+0x24 = 02169510(e+0) + t+4 (0207C9D0) +
# 0218DC5C(+4) (0207C9D0); D+0x30 / +0x34 / +0x38 = 0x1000; 0218BFB4(BM, D+0xC, +0x1CC + 0x18 t+0); 0218BA70(BM, D); 0218C54C(e, 0).
# t+0 is drawn 0..126 here (a resource index; -1 would index before the resource array).
fn 0218c724 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x3C00 u32 alt range 0 7 || range 0 126 rep=16,0x20
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0207af1c force ret=none args=out:0x24,v,v  # MTX_RotY33(&m, sin, cos) (FX)
  stub 02169510 force ret=none args=v,out:12  # field_g3d_map (G8): the block position (h, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outw:12:0/0x10000/0x20000/0x30000/0xFFFF0000/0xFFFE0000/0x50000  # VEC_Add(a, b, out) (FX)
  stub 0218dc5c force ret=none args=v,out:12  # field_g3d_mapper (G8): the map offset (G, &out)
  stub 0218bfb4 ret=none args=ptr,v,v  # in-area: animation object (BM, obj, set)
  stub 0218ba70 ret=none args=ptr,v  # in-area: put D in a free door slot
  stub 0218c54c ret=none args=ptr,v  # in-area: swap an entry state (e, f)
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 02047400 force ret=u32 args=v,v,v  # g3d_system: model object (res, 0, textures)
  stub 02047564 force ret=u32 args=v,v,v  # g3d_system: animation (model, anim res, 0)
  stub 0204767c force ret=u32 args=v,in:16,v  # g3d_system: animated object (model, anims, 4)
  stub 0218c02c ret=none args=v,ptr  # in-area: init proc, kinds 0 / 2 (BM, obj)
  stub 0218c040 ret=none args=v,v  # in-area: init proc, kind 1 (BM, obj)
  stub 0218c08c ret=none args=ptr,v  # in-area: init proc, kind 3 (BM, obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)

# 0218C7D8(BM, x, pos): e = 0218C654(BM, x, pos); e == 0 -> 0; else 0218C724(BM, e). t+0 drawn as 0218C724. The first word of
# the list block (0218C584's allocation, the answer when no model is in the box) is 0 or an entry (else 0218C724 reads through it).
fn 0218c7d8 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt const 0 || range 0 10
  arg buf 0x10
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x3C00 u32 alt range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x4000 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8
  fix arg2 0 u32 alt oneof 0 0x10000 0x20000 0x30000 0xFFFF0000 0xFFFE0000 0x50000 || any
  fix arg2 8 u32 alt oneof 0 0x10000 0x20000 0x30000 0xFFFF0000 0xFFFE0000 0x50000 || any
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 0218c654 ret=pick:0/0x02E101F8/0x02E10208/0x02E10218/0x02E10228/0x02E113E8 args=ptr,v,ptr  # in-area: the first entry near pos (BM, x, pos)
  stub 0218c724 ret=u32 args=ptr,v  # in-area: door object of an entry (BM, e)
  stub 0218c9c4 ret=none args=out:16,ptr  # in-area: box around a position (&rect, pos)
  stub 0218c638 ret=pick:0/0x02E101F8/0x02E10208/0x02E10218/0x02E10228/0x02E113E8 args=ptr,v,in:16  # in-area: the first entry in a box (BM, x, &rect)
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0207af1c force ret=none args=out:0x24,v,v  # MTX_RotY33(&m, sin, cos) (FX)
  stub 02169510 force ret=none args=v,out:12  # field_g3d_map (G8): the block position (h, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outw:12:0/0x10000/0x20000/0x30000/0xFFFF0000/0xFFFE0000/0x50000  # VEC_Add(a, b, out) (FX)
  stub 0218dc5c force ret=none args=v,out:12  # field_g3d_mapper (G8): the map offset (G, &out)
  stub 0218bfb4 ret=none args=ptr,v,v  # in-area: animation object (BM, obj, set)
  stub 0218ba70 ret=none args=ptr,v  # in-area: put D in a free door slot
  stub 0218c54c ret=none args=ptr,v  # in-area: swap an entry state (e, f)
  stub 0218c584 ret=ptr:arg0:0x3F80:1 args=ptr,in:16,v,out:4  # in-area: entries in a box (BM, &rect, x, &n)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 02047400 force ret=u32 args=v,v,v  # g3d_system: model object (res, 0, textures)
  stub 02047564 force ret=u32 args=v,v,v  # g3d_system: animation (model, anim res, 0)
  stub 0204767c force ret=u32 args=v,in:16,v  # g3d_system: animated object (model, anims, 4)
  stub 0218c02c ret=none args=v,ptr  # in-area: init proc, kinds 0 / 2 (BM, obj)
  stub 0218c040 ret=none args=v,v  # in-area: init proc, kind 1 (BM, obj)
  stub 0218c08c ret=none args=ptr,v  # in-area: init proc, kind 3 (BM, obj)
  stub 0218c674 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr,v  # in-area: the door kind of an entry (BM, e)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)
  stub 0218c530 ret=pick+u8:0/1/2/3/7/127/128 args=ptr  # in-area: animation state of an entry
  stub 0218b924 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: model record i
  stub 0218bcb0 ret=pick:0/1/2/3/4/5/6/7/8/9 args=ptr  # in-area: door kind of a record (ROM table)
  stub 0218bbf4 ret=pick:0x02E11800/0x02E11824/0x02E11848/0x02E1186C/0x02E11890/0x02E118B4/0x02E118D8/0x02E118FC args=ptr,v  # in-area: list entry i

# 0218C7F0(BM, x, src) -> D (a door object without a placed model): D = Heap_AllocDebug(heap, 0x60, 0, file, 0x825); +0 = BM; +8 = 0;
# +4 = 0xFFFF; D+0x24..+0x60 = src (0x3C bytes); 0218BFB4(BM, D+0xC, +0x1CC + 0x18 0218BBD0(BM, x)); 0218BA70(BM, D).
fn 0218c7f0 ret=u32 inputs=200
  nowork
  arg buf 0x10000
  arg u32 alt range 0 399 || any
  arg buf 0x40
  fix arg0 0 u16 alt range 0 4 || any
  fix arg0 8 u32 alt range 0 4 || any
  fix arg0 0xC u32 alt range 0 4 || any
  fix arg0 0x10 u32 alt const 0 || const 1 || any
  fix arg0 0x14 u8 alt range 0 3 || any
  fix arg0 0x15 u8 alt range 0 3 || any
  fix arg0 0x18 u32 addrof arg0 0x1400
  fix arg0 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127
  fix arg0 0x1B0 u32 alt const 0 || any
  fix arg0 0x1B8 u32 alt const 0 || any
  fix arg0 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg0 0x1CC u32 addrof arg0 0x2000
  fix arg0 0x1D0 u32 alt const 0 || any
  fix arg0 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg0 0x1D8 u32 addrof arg0 0x2C00
  fix arg0 0x1DC u32 alt const 0 || oneof 0x02E13800 0x02E13860 0x02E138C0 0x02E13920 0x02E13980 0x02E139E0 0x02E13A40 rep=7,4
  fix arg0 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg0 0x204 u32 oneof 0x02E13C00 0x02E13C20 0x02E13C40 0x02E13C60 0x02E13C80 0x02E13CA0 0x02E13CC0 0x02E13CE0 0x02E13D00 0x02E13D20 0x02E13D40 0x02E13D60 0x02E13D80 0x02E13DA0 0x02E13DC0 0x02E13DE0 rep=288,16
  fix arg0 0x1402 u8 alt range 0 17 || any
  fix arg0 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg0 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg0 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg0 0x1810 u8 range 0 3 rep=8,0x24
  fix arg0 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg0 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg0 0x2000 u32 oneof 0x02E11800 0x02E11824 0x02E11848 0x02E1186C 0x02E11890 0x02E118B4 0x02E118D8 0x02E118FC rep=127,0x18
  fix arg0 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg0 0x2C04 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=127,0x18
  fix arg0 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg0 0x3800 u32 addrof arg0 0 rep=7,0x60
  fix arg0 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg0 0x3808 u32 alt const 0 || oneof 0x02E101F8 0x02E10208 0x02E10218 0x02E10228 0x02E113E8 rep=7,0x60
  fix arg0 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg0 0x3810 u32 oneof 0x02E12000 0x02E12018 0x02E12030 0x02E12048 0x02E12060 0x02E12078 0x02E12090 0x02E120A8 rep=7,0x60
  fix arg0 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg0 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg0 0x3F08 u32 addrof arg0 0x3F40
  fix arg0 0x1AC u16 alt range 0 8 || range 0 127 || any
  stub 02030734 force ret=seqptr:arg0:0x4000:0x400 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0218bbd0 ret=mod:128 args=ptr,v  # in-area: the model index of id x (0..0x7F)
  stub 0218bfb4 ret=none args=ptr,v,v  # in-area: animation object (BM, obj, set)
  stub 0218ba70 ret=none args=ptr,v  # in-area: put D in a free door slot
  stub 0218bcac ret=pick:0x02E11810/0x02E11834/0x02E11858/0x02E1187C/0x02E118A0/0x02E118C4/0x02E118E8/0x02E1190C args=v  # in-area: r + 0x10 (the animation header)
  stub 02047400 force ret=u32 args=v,v,v  # g3d_system: model object (res, 0, textures)
  stub 02047564 force ret=u32 args=v,v,v  # g3d_system: animation (model, anim res, 0)
  stub 0204767c force ret=u32 args=v,in:16,v  # g3d_system: animated object (model, anims, 4)
  stub 0218c02c ret=none args=v,ptr  # in-area: init proc, kinds 0 / 2 (BM, obj)
  stub 0218c040 ret=none args=v,v  # in-area: init proc, kind 1 (BM, obj)
  stub 0218c08c ret=none args=ptr,v  # in-area: init proc, kind 3 (BM, obj)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 0218bd24 ret=pick:0/1/2/3 args=ptr  # in-area: the animation index of the time zone (t)

# 0218C860(D): D != 0 -> 0218BA48(D+0, D); 0218C0D8(D+0xC); D+8 != 0 -> 0218C54C(D+8, 1); Heap_Free(D).
fn 0218c860 ret=none inputs=100
  nowork
  arg u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40
  arg buf 0x10000
  fix arg1 0 u16 alt range 0 4 || any
  fix arg1 8 u32 alt range 0 4 || any
  fix arg1 0xC u32 alt range 0 4 || any
  fix arg1 0x10 u32 alt const 0 || const 1 || any
  fix arg1 0x14 u8 alt range 0 3 || any
  fix arg1 0x15 u8 alt range 0 3 || any
  fix arg1 0x18 u32 addrof arg1 0x1400
  fix arg1 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg1 0x1AC u16 alt range 0 8 || range 0 127
  fix arg1 0x1B0 u32 alt const 0 || any
  fix arg1 0x1B8 u32 alt const 0 || any
  fix arg1 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg1 0x1CC u32 addrof arg1 0x2000
  fix arg1 0x1D0 u32 alt const 0 || any
  fix arg1 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg1 0x1D8 u32 addrof arg1 0x2C00
  fix arg1 0x1DC u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40 rep=7,4
  fix arg1 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg1 0x204 u32 oneof 0x02E23C00 0x02E23C20 0x02E23C40 0x02E23C60 0x02E23C80 0x02E23CA0 0x02E23CC0 0x02E23CE0 0x02E23D00 0x02E23D20 0x02E23D40 0x02E23D60 0x02E23D80 0x02E23DA0 0x02E23DC0 0x02E23DE0 rep=288,16
  fix arg1 0x1402 u8 alt range 0 17 || any
  fix arg1 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg1 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg1 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1810 u8 range 0 3 rep=8,0x24
  fix arg1 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg1 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x2000 u32 oneof 0x02E21800 0x02E21824 0x02E21848 0x02E2186C 0x02E21890 0x02E218B4 0x02E218D8 0x02E218FC rep=127,0x18
  fix arg1 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg1 0x2C04 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=127,0x18
  fix arg1 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x3800 u32 addrof arg1 0 rep=7,0x60
  fix arg1 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg1 0x3808 u32 alt const 0 || oneof 0x02E201F8 0x02E20208 0x02E20218 0x02E20228 0x02E213E8 rep=7,0x60
  fix arg1 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg1 0x3810 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=7,0x60
  fix arg1 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg1 0x3F08 u32 addrof arg1 0x3F40
  stub 0218ba48 ret=none args=ptr,v  # in-area: clear the door slot of D
  stub 0218c0d8 ret=none args=ptr  # in-area: delete an animation object
  stub 0218c54c ret=none args=ptr,v  # in-area: swap an entry state (e, f)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 0204779c force ret=pick:0/1/2/4 args=v  # g3d_system: animation count (a u16; closed set outside 0218C0D8, see the header)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 0204762c force ret=none args=v  # g3d_system: delete an animation
  stub 02047788 force ret=u32 args=v  # g3d_system: model of the object
  stub 02047738 force ret=none args=v  # g3d_system: delete the object
  stub 02047504 force ret=none args=v  # g3d_system: delete the model

# 0218C88C(D, s, mode): D != 0 -> D+4 = s, 0218C3B0(D+0xC, s, mode).
fn 0218c88c ret=none inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E43800 0x02E43860 0x02E438C0 0x02E43920 0x02E43980 0x02E439E0 0x02E43A40
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 5 || any
  arg buf 0x10000
  fix arg3 0 u16 alt range 0 4 || any
  fix arg3 8 u32 alt range 0 4 || any
  fix arg3 0xC u32 alt range 0 4 || any
  fix arg3 0x10 u32 alt const 0 || const 1 || any
  fix arg3 0x14 u8 alt range 0 3 || any
  fix arg3 0x15 u8 alt range 0 3 || any
  fix arg3 0x18 u32 addrof arg3 0x1400
  fix arg3 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg3 0x1AC u16 alt range 0 8 || range 0 127
  fix arg3 0x1B0 u32 alt const 0 || any
  fix arg3 0x1B8 u32 alt const 0 || any
  fix arg3 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg3 0x1CC u32 addrof arg3 0x2000
  fix arg3 0x1D0 u32 alt const 0 || any
  fix arg3 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg3 0x1D8 u32 addrof arg3 0x2C00
  fix arg3 0x1DC u32 alt const 0 || oneof 0x02E43800 0x02E43860 0x02E438C0 0x02E43920 0x02E43980 0x02E439E0 0x02E43A40 rep=7,4
  fix arg3 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg3 0x204 u32 oneof 0x02E43C00 0x02E43C20 0x02E43C40 0x02E43C60 0x02E43C80 0x02E43CA0 0x02E43CC0 0x02E43CE0 0x02E43D00 0x02E43D20 0x02E43D40 0x02E43D60 0x02E43D80 0x02E43DA0 0x02E43DC0 0x02E43DE0 rep=288,16
  fix arg3 0x1402 u8 alt range 0 17 || any
  fix arg3 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg3 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg3 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1810 u8 range 0 3 rep=8,0x24
  fix arg3 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg3 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x2000 u32 oneof 0x02E41800 0x02E41824 0x02E41848 0x02E4186C 0x02E41890 0x02E418B4 0x02E418D8 0x02E418FC rep=127,0x18
  fix arg3 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg3 0x2C04 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=127,0x18
  fix arg3 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x3800 u32 addrof arg3 0 rep=7,0x60
  fix arg3 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg3 0x3808 u32 alt const 0 || oneof 0x02E401F8 0x02E40208 0x02E40218 0x02E40228 0x02E413E8 rep=7,0x60
  fix arg3 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg3 0x3810 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=7,0x60
  fix arg3 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg3 0x3F08 u32 addrof arg3 0x3F40
  stub 0218c3b0 ret=none args=v,v,v  # in-area: set the animation state (obj, k, mode)
  stub 0218bcac ret=pick:0x02E41810/0x02E41834/0x02E41858/0x02E4187C/0x02E418A0/0x02E418C4/0x02E418E8/0x02E4190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c438 ret=none  # in-area: state proc (empty)
  stub 0218c3e0 ret=none args=v,v,v  # in-area: state proc, kind 2 (obj, k, mode)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg3:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C89C(D, mode): D != 0 -> 0218C3B0(D+0xC, D+4, mode).
fn 0218c89c ret=none inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40
  arg u32 alt range 0 5 || any
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218c3b0 ret=none args=v,v,v  # in-area: set the animation state (obj, k, mode)
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c438 ret=none  # in-area: state proc (empty)
  stub 0218c3e0 ret=none args=v,v,v  # in-area: state proc, kind 2 (obj, k, mode)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg2:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C8B0(D, s): D == 0 -> 1; 0218C8FC(D, s) == 1 -> 0218C88C(D, s, 3), 1; else 0.
fn 0218c8b0 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40
  arg u32 alt range 0 4 || any
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218c8fc ret=pick:0/1 args=v,v  # in-area: door animation s stopped (D, s)
  stub 0218c88c ret=none args=v,v,v  # in-area: door state (D, s, mode)
  stub 0218c43c ret=pick:0/1 args=ptr,v  # in-area: animation k stopped (obj, k)
  stub 0218c3b0 ret=none args=v,v,v  # in-area: set the animation state (obj, k, mode)
  stub 0218bcac ret=pick:0x02E31810/0x02E31834/0x02E31858/0x02E3187C/0x02E318A0/0x02E318C4/0x02E318E8/0x02E3190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c438 ret=none  # in-area: state proc (empty)
  stub 0218c3e0 ret=none args=v,v,v  # in-area: state proc, kind 2 (obj, k, mode)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg2:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C8D8(D): D == 0 -> 1; 0218C910(D) == 1 -> 0218C89C(D, 3), 1; else 0.
fn 0218c8d8 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40
  arg buf 0x10000
  fix arg1 0 u16 alt range 0 4 || any
  fix arg1 8 u32 alt range 0 4 || any
  fix arg1 0xC u32 alt range 0 4 || any
  fix arg1 0x10 u32 alt const 0 || const 1 || any
  fix arg1 0x14 u8 alt range 0 3 || any
  fix arg1 0x15 u8 alt range 0 3 || any
  fix arg1 0x18 u32 addrof arg1 0x1400
  fix arg1 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg1 0x1AC u16 alt range 0 8 || range 0 127
  fix arg1 0x1B0 u32 alt const 0 || any
  fix arg1 0x1B8 u32 alt const 0 || any
  fix arg1 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg1 0x1CC u32 addrof arg1 0x2000
  fix arg1 0x1D0 u32 alt const 0 || any
  fix arg1 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg1 0x1D8 u32 addrof arg1 0x2C00
  fix arg1 0x1DC u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40 rep=7,4
  fix arg1 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg1 0x204 u32 oneof 0x02E23C00 0x02E23C20 0x02E23C40 0x02E23C60 0x02E23C80 0x02E23CA0 0x02E23CC0 0x02E23CE0 0x02E23D00 0x02E23D20 0x02E23D40 0x02E23D60 0x02E23D80 0x02E23DA0 0x02E23DC0 0x02E23DE0 rep=288,16
  fix arg1 0x1402 u8 alt range 0 17 || any
  fix arg1 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg1 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg1 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1810 u8 range 0 3 rep=8,0x24
  fix arg1 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg1 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x2000 u32 oneof 0x02E21800 0x02E21824 0x02E21848 0x02E2186C 0x02E21890 0x02E218B4 0x02E218D8 0x02E218FC rep=127,0x18
  fix arg1 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg1 0x2C04 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=127,0x18
  fix arg1 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x3800 u32 addrof arg1 0 rep=7,0x60
  fix arg1 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg1 0x3808 u32 alt const 0 || oneof 0x02E201F8 0x02E20208 0x02E20218 0x02E20228 0x02E213E8 rep=7,0x60
  fix arg1 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg1 0x3810 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=7,0x60
  fix arg1 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg1 0x3F08 u32 addrof arg1 0x3F40
  stub 0218c910 ret=pick:0/1 args=v  # in-area: door animation stopped (D)
  stub 0218c89c ret=none args=v,v  # in-area: door state (D, mode)
  stub 0218c43c ret=pick:0/1 args=ptr,v  # in-area: animation k stopped (obj, k)
  stub 0218c3b0 ret=none args=v,v,v  # in-area: set the animation state (obj, k, mode)
  stub 0218bcac ret=pick:0x02E21810/0x02E21834/0x02E21858/0x02E2187C/0x02E218A0/0x02E218C4/0x02E218E8/0x02E2190C args=v  # in-area: r + 0x10 (the animation header)
  stub 0218c438 ret=none  # in-area: state proc (empty)
  stub 0218c3e0 ret=none args=v,v,v  # in-area: state proc, kind 2 (obj, k, mode)
  stub 0218c388 ret=none args=ptr  # in-area: all animations off (obj)
  stub 0218c23c ret=none args=v,v  # in-area: play forward (obj, k)
  stub 0218c2e4 ret=none args=v,v  # in-area: play backward (obj, k)
  stub 0218c290 ret=none args=v,v  # in-area: loop (obj, k)
  stub 0218c350 ret=none args=v,v  # in-area: stop (obj, k)
  stub 020477d4 force ret=none args=v,v  # g3d_system: animation off (obj, i)
  stub 020477a4 force ret=none args=v,v  # g3d_system: animation on (obj, i)
  stub 02047804 force ret=none args=v,v  # g3d_system: animation rewind (obj, i)
  stub 02047790 force ret=pick+u32:0/0x02123400 args=v,v  # g3d_system: animation i (obj, i)
  stub 02047674 force ret=ptr:arg1:0x3F00:1 args=v  # g3d_system: animation resource of an animation
  stub 0204784c force ret=none args=v,v,in:4  # g3d_system: animation frame (obj, i, &frame)

# 0218C8FC(D, s): D == 0 -> 0; 0218C43C(D+0xC, s).
fn 0218c8fc ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40
  arg u32 alt range 0 4 || any
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218c43c ret=pick:0/1 args=ptr,v  # in-area: animation k stopped (obj, k)

# 0218C910(D): D == 0 -> 0; 0218C43C(D+0xC, D+4).
fn 0218c910 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40
  arg buf 0x10000
  fix arg1 0 u16 alt range 0 4 || any
  fix arg1 8 u32 alt range 0 4 || any
  fix arg1 0xC u32 alt range 0 4 || any
  fix arg1 0x10 u32 alt const 0 || const 1 || any
  fix arg1 0x14 u8 alt range 0 3 || any
  fix arg1 0x15 u8 alt range 0 3 || any
  fix arg1 0x18 u32 addrof arg1 0x1400
  fix arg1 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg1 0x1AC u16 alt range 0 8 || range 0 127
  fix arg1 0x1B0 u32 alt const 0 || any
  fix arg1 0x1B8 u32 alt const 0 || any
  fix arg1 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg1 0x1CC u32 addrof arg1 0x2000
  fix arg1 0x1D0 u32 alt const 0 || any
  fix arg1 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg1 0x1D8 u32 addrof arg1 0x2C00
  fix arg1 0x1DC u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40 rep=7,4
  fix arg1 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg1 0x204 u32 oneof 0x02E23C00 0x02E23C20 0x02E23C40 0x02E23C60 0x02E23C80 0x02E23CA0 0x02E23CC0 0x02E23CE0 0x02E23D00 0x02E23D20 0x02E23D40 0x02E23D60 0x02E23D80 0x02E23DA0 0x02E23DC0 0x02E23DE0 rep=288,16
  fix arg1 0x1402 u8 alt range 0 17 || any
  fix arg1 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg1 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg1 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1810 u8 range 0 3 rep=8,0x24
  fix arg1 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg1 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x2000 u32 oneof 0x02E21800 0x02E21824 0x02E21848 0x02E2186C 0x02E21890 0x02E218B4 0x02E218D8 0x02E218FC rep=127,0x18
  fix arg1 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg1 0x2C04 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=127,0x18
  fix arg1 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x3800 u32 addrof arg1 0 rep=7,0x60
  fix arg1 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg1 0x3808 u32 alt const 0 || oneof 0x02E201F8 0x02E20208 0x02E20218 0x02E20228 0x02E213E8 rep=7,0x60
  fix arg1 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg1 0x3810 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=7,0x60
  fix arg1 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg1 0x3F08 u32 addrof arg1 0x3F40
  stub 0218c43c ret=pick:0/1 args=ptr,v  # in-area: animation k stopped (obj, k)

# 0218C924(D): D == 0 -> 0; u16 (*(D+0x10))+0 +2 (the kind of the door's model).
fn 0218c924 ret=u32 inputs=100
  nowork
  arg u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40
  arg buf 0x10000
  fix arg1 0 u16 alt range 0 4 || any
  fix arg1 8 u32 alt range 0 4 || any
  fix arg1 0xC u32 alt range 0 4 || any
  fix arg1 0x10 u32 alt const 0 || const 1 || any
  fix arg1 0x14 u8 alt range 0 3 || any
  fix arg1 0x15 u8 alt range 0 3 || any
  fix arg1 0x18 u32 addrof arg1 0x1400
  fix arg1 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg1 0x1AC u16 alt range 0 8 || range 0 127
  fix arg1 0x1B0 u32 alt const 0 || any
  fix arg1 0x1B8 u32 alt const 0 || any
  fix arg1 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg1 0x1CC u32 addrof arg1 0x2000
  fix arg1 0x1D0 u32 alt const 0 || any
  fix arg1 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg1 0x1D8 u32 addrof arg1 0x2C00
  fix arg1 0x1DC u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40 rep=7,4
  fix arg1 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg1 0x204 u32 oneof 0x02E23C00 0x02E23C20 0x02E23C40 0x02E23C60 0x02E23C80 0x02E23CA0 0x02E23CC0 0x02E23CE0 0x02E23D00 0x02E23D20 0x02E23D40 0x02E23D60 0x02E23D80 0x02E23DA0 0x02E23DC0 0x02E23DE0 rep=288,16
  fix arg1 0x1402 u8 alt range 0 17 || any
  fix arg1 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg1 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg1 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1810 u8 range 0 3 rep=8,0x24
  fix arg1 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg1 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x2000 u32 oneof 0x02E21800 0x02E21824 0x02E21848 0x02E2186C 0x02E21890 0x02E218B4 0x02E218D8 0x02E218FC rep=127,0x18
  fix arg1 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg1 0x2C04 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=127,0x18
  fix arg1 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x3800 u32 addrof arg1 0 rep=7,0x60
  fix arg1 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg1 0x3808 u32 alt const 0 || oneof 0x02E201F8 0x02E20208 0x02E20218 0x02E20228 0x02E213E8 rep=7,0x60
  fix arg1 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg1 0x3810 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=7,0x60
  fix arg1 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg1 0x3F08 u32 addrof arg1 0x3F40

# 0218C934(D): D != 0 -> 02047A60(D+0xC, D+0x24) (draw).
fn 0218c934 ret=none inputs=100
  nowork
  arg u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40
  arg buf 0x10000
  fix arg1 0 u16 alt range 0 4 || any
  fix arg1 8 u32 alt range 0 4 || any
  fix arg1 0xC u32 alt range 0 4 || any
  fix arg1 0x10 u32 alt const 0 || const 1 || any
  fix arg1 0x14 u8 alt range 0 3 || any
  fix arg1 0x15 u8 alt range 0 3 || any
  fix arg1 0x18 u32 addrof arg1 0x1400
  fix arg1 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg1 0x1AC u16 alt range 0 8 || range 0 127
  fix arg1 0x1B0 u32 alt const 0 || any
  fix arg1 0x1B8 u32 alt const 0 || any
  fix arg1 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg1 0x1CC u32 addrof arg1 0x2000
  fix arg1 0x1D0 u32 alt const 0 || any
  fix arg1 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg1 0x1D8 u32 addrof arg1 0x2C00
  fix arg1 0x1DC u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40 rep=7,4
  fix arg1 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg1 0x204 u32 oneof 0x02E23C00 0x02E23C20 0x02E23C40 0x02E23C60 0x02E23C80 0x02E23CA0 0x02E23CC0 0x02E23CE0 0x02E23D00 0x02E23D20 0x02E23D40 0x02E23D60 0x02E23D80 0x02E23DA0 0x02E23DC0 0x02E23DE0 rep=288,16
  fix arg1 0x1402 u8 alt range 0 17 || any
  fix arg1 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg1 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg1 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1810 u8 range 0 3 rep=8,0x24
  fix arg1 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg1 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x2000 u32 oneof 0x02E21800 0x02E21824 0x02E21848 0x02E2186C 0x02E21890 0x02E218B4 0x02E218D8 0x02E218FC rep=127,0x18
  fix arg1 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg1 0x2C04 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=127,0x18
  fix arg1 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x3800 u32 addrof arg1 0 rep=7,0x60
  fix arg1 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg1 0x3808 u32 alt const 0 || oneof 0x02E201F8 0x02E20208 0x02E20218 0x02E20228 0x02E213E8 rep=7,0x60
  fix arg1 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg1 0x3810 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=7,0x60
  fix arg1 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg1 0x3F08 u32 addrof arg1 0x3F40
  stub 02047a60 force ret=none args=v,in:0x3C  # g3d_system (G8): draw (obj, &transform)

# 0218C944(D, k, &se): D == 0 -> 0; v = 0218C924(D); se = 0; k >= 4 -> 0; the ROM table 0x021D3D88 (6 x {u16 kind, u16 se[4]}): kind
# == v -> se = se[k], 1; else 0.
fn 0218c944 ret=u32 inputs=300
  nowork
  arg u32 alt const 0 || oneof 0x02E43800 0x02E43860 0x02E438C0 0x02E43920 0x02E43980 0x02E439E0 0x02E43A40
  arg u32 alt range 0 4 || any
  arg buf 0x10
  arg buf 0x10000
  fix arg3 0 u16 alt range 0 4 || any
  fix arg3 8 u32 alt range 0 4 || any
  fix arg3 0xC u32 alt range 0 4 || any
  fix arg3 0x10 u32 alt const 0 || const 1 || any
  fix arg3 0x14 u8 alt range 0 3 || any
  fix arg3 0x15 u8 alt range 0 3 || any
  fix arg3 0x18 u32 addrof arg3 0x1400
  fix arg3 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg3 0x1AC u16 alt range 0 8 || range 0 127
  fix arg3 0x1B0 u32 alt const 0 || any
  fix arg3 0x1B8 u32 alt const 0 || any
  fix arg3 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg3 0x1CC u32 addrof arg3 0x2000
  fix arg3 0x1D0 u32 alt const 0 || any
  fix arg3 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg3 0x1D8 u32 addrof arg3 0x2C00
  fix arg3 0x1DC u32 alt const 0 || oneof 0x02E43800 0x02E43860 0x02E438C0 0x02E43920 0x02E43980 0x02E439E0 0x02E43A40 rep=7,4
  fix arg3 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg3 0x204 u32 oneof 0x02E43C00 0x02E43C20 0x02E43C40 0x02E43C60 0x02E43C80 0x02E43CA0 0x02E43CC0 0x02E43CE0 0x02E43D00 0x02E43D20 0x02E43D40 0x02E43D60 0x02E43D80 0x02E43DA0 0x02E43DC0 0x02E43DE0 rep=288,16
  fix arg3 0x1402 u8 alt range 0 17 || any
  fix arg3 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg3 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg3 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg3 0x1810 u8 range 0 3 rep=8,0x24
  fix arg3 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg3 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg3 0x2000 u32 oneof 0x02E41800 0x02E41824 0x02E41848 0x02E4186C 0x02E41890 0x02E418B4 0x02E418D8 0x02E418FC rep=127,0x18
  fix arg3 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg3 0x2C04 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=127,0x18
  fix arg3 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg3 0x3800 u32 addrof arg3 0 rep=7,0x60
  fix arg3 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg3 0x3808 u32 alt const 0 || oneof 0x02E401F8 0x02E40208 0x02E40218 0x02E40228 0x02E413E8 rep=7,0x60
  fix arg3 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg3 0x3810 u32 oneof 0x02E42000 0x02E42018 0x02E42030 0x02E42048 0x02E42060 0x02E42078 0x02E42090 0x02E420A8 rep=7,0x60
  fix arg3 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg3 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg3 0x3F08 u32 addrof arg3 0x3F40
  stub 0218c924 ret=pick+u16:0/1/2/5/13/14/15 args=v  # in-area: door kind (D)

# 0218C990(D, &se): D == 0 -> 0; 0218C944(D, D+4, &se).
fn 0218c990 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40
  arg buf 0x10
  arg buf 0x10000
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 8 u32 alt range 0 4 || any
  fix arg2 0xC u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || const 1 || any
  fix arg2 0x14 u8 alt range 0 3 || any
  fix arg2 0x15 u8 alt range 0 3 || any
  fix arg2 0x18 u32 addrof arg2 0x1400
  fix arg2 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg2 0x1AC u16 alt range 0 8 || range 0 127
  fix arg2 0x1B0 u32 alt const 0 || any
  fix arg2 0x1B8 u32 alt const 0 || any
  fix arg2 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg2 0x1CC u32 addrof arg2 0x2000
  fix arg2 0x1D0 u32 alt const 0 || any
  fix arg2 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg2 0x1D8 u32 addrof arg2 0x2C00
  fix arg2 0x1DC u32 alt const 0 || oneof 0x02E33800 0x02E33860 0x02E338C0 0x02E33920 0x02E33980 0x02E339E0 0x02E33A40 rep=7,4
  fix arg2 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg2 0x204 u32 oneof 0x02E33C00 0x02E33C20 0x02E33C40 0x02E33C60 0x02E33C80 0x02E33CA0 0x02E33CC0 0x02E33CE0 0x02E33D00 0x02E33D20 0x02E33D40 0x02E33D60 0x02E33D80 0x02E33DA0 0x02E33DC0 0x02E33DE0 rep=288,16
  fix arg2 0x1402 u8 alt range 0 17 || any
  fix arg2 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg2 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg2 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg2 0x1810 u8 range 0 3 rep=8,0x24
  fix arg2 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg2 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg2 0x2000 u32 oneof 0x02E31800 0x02E31824 0x02E31848 0x02E3186C 0x02E31890 0x02E318B4 0x02E318D8 0x02E318FC rep=127,0x18
  fix arg2 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg2 0x2C04 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=127,0x18
  fix arg2 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg2 0x3800 u32 addrof arg2 0 rep=7,0x60
  fix arg2 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg2 0x3808 u32 alt const 0 || oneof 0x02E301F8 0x02E30208 0x02E30218 0x02E30228 0x02E313E8 rep=7,0x60
  fix arg2 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg2 0x3810 u32 oneof 0x02E32000 0x02E32018 0x02E32030 0x02E32048 0x02E32060 0x02E32078 0x02E32090 0x02E320A8 rep=7,0x60
  fix arg2 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg2 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg2 0x3F08 u32 addrof arg2 0x3F40
  stub 0218c944 ret=pick:0/1 args=v,v,out:2  # in-area: door SE (D, k, &se)
  stub 0218c924 ret=pick+u16:0/1/2/5/13/14/15 args=v  # in-area: door kind (D)

# 0218C9A4(D): 0218C990(D, &se) == 0 -> 0; else 02006224(02006148(se)) (play the door sound).
fn 0218c9a4 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40
  arg buf 0x10000
  fix arg1 0 u16 alt range 0 4 || any
  fix arg1 8 u32 alt range 0 4 || any
  fix arg1 0xC u32 alt range 0 4 || any
  fix arg1 0x10 u32 alt const 0 || const 1 || any
  fix arg1 0x14 u8 alt range 0 3 || any
  fix arg1 0x15 u8 alt range 0 3 || any
  fix arg1 0x18 u32 addrof arg1 0x1400
  fix arg1 0x1C u8 alt range 0 8 || const 0x80 || any rep=400,1
  fix arg1 0x1AC u16 alt range 0 8 || range 0 127
  fix arg1 0x1B0 u32 alt const 0 || any
  fix arg1 0x1B8 u32 alt const 0 || any
  fix arg1 0x1C8 u32 alt range 0 8 || range 0 127
  fix arg1 0x1CC u32 addrof arg1 0x2000
  fix arg1 0x1D0 u32 alt const 0 || any
  fix arg1 0x1D4 u32 alt range 0 8 || range 0 127
  fix arg1 0x1D8 u32 addrof arg1 0x2C00
  fix arg1 0x1DC u32 alt const 0 || oneof 0x02E23800 0x02E23860 0x02E238C0 0x02E23920 0x02E23980 0x02E239E0 0x02E23A40 rep=7,4
  fix arg1 0x1F8 u32 alt const 0 || oneof 0x02123450 0x02123460 0x02123470 || any rep=288,16
  fix arg1 0x204 u32 oneof 0x02E23C00 0x02E23C20 0x02E23C40 0x02E23C60 0x02E23C80 0x02E23CA0 0x02E23CC0 0x02E23CE0 0x02E23D00 0x02E23D20 0x02E23D40 0x02E23D60 0x02E23D80 0x02E23DA0 0x02E23DC0 0x02E23DE0 rep=288,16
  fix arg1 0x1402 u8 alt range 0 17 || any
  fix arg1 0x1404 u32 oneof 0x400 0x424 0x448 0x46C 0x490 0x4B4 0x4D8 0x4FC rep=254,4
  fix arg1 0x1800 u16 alt range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1802 u16 alt oneof 1 2 5 13 14 15 || range 0 16 || any rep=8,0x24
  fix arg1 0x1804 u16 alt const 0xFFFF || range 0 15 || range 0 399 rep=8,0x24
  fix arg1 0x1810 u8 range 0 3 rep=8,0x24
  fix arg1 0x1812 u8 alt range 0 4 || any rep=8,0x24
  fix arg1 0x1814 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1818 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x181C u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x1820 u32 alt const 0xFFFFFFFF || range 0 0x40 rep=8,0x24
  fix arg1 0x2000 u32 oneof 0x02E21800 0x02E21824 0x02E21848 0x02E2186C 0x02E21890 0x02E218B4 0x02E218D8 0x02E218FC rep=127,0x18
  fix arg1 0x2C00 u32 alt const 0 || any rep=127,0x18
  fix arg1 0x2C04 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=127,0x18
  fix arg1 0x2C08 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C0C u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C10 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x2C14 u32 alt range 0 5 || any rep=8,0x18
  fix arg1 0x3800 u32 addrof arg1 0 rep=7,0x60
  fix arg1 0x3804 u32 alt range 0 3 || const 0xFFFF || any rep=7,0x60
  fix arg1 0x3808 u32 alt const 0 || oneof 0x02E201F8 0x02E20208 0x02E20218 0x02E20228 0x02E213E8 rep=7,0x60
  fix arg1 0x380C u32 alt const 0 || any rep=7,0x60
  fix arg1 0x3810 u32 oneof 0x02E22000 0x02E22018 0x02E22030 0x02E22048 0x02E22060 0x02E22078 0x02E22090 0x02E220A8 rep=7,0x60
  fix arg1 0x3814 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3818 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x381C u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3820 u32 alt range 0 5 || any rep=7,0x60
  fix arg1 0x3C00 u32 alt const 0xFFFFFFFF || range 0 7 || range 0 126 rep=16,0x20
  fix arg1 0x3F08 u32 addrof arg1 0x3F40
  stub 0218c990 ret=pick:0/1 args=v,out:2  # in-area: door SE (D, &se)
  stub 02006148 force ret=u32 args=v  # sound: the SE player of an SE
  stub 02006224 force ret=u32 args=v  # sound: play
  stub 0218c944 ret=pick:0/1 args=v,v,out:2  # in-area: door SE (D, k, &se)
  stub 0218c924 ret=pick+u16:0/1/2/5/13/14/15 args=v  # in-area: door kind (D)

# 0218C9C4(rect, pos): rect = {pos.z - 0x30000, pos.z + 0x30000, pos.x - 0x20000, pos.x + 0x20000}.
fn 0218c9c4 ret=none inputs=100
  nowork
  arg buf 0x10
  arg buf 0x10
