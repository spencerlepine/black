# fndiff spec (F3b): `field_player_nogrid.c` (overlay 21 Thumb code 0x021A5880..0x021A6CB8; F1 row 21.24): the player on the
# free-movement (rail) maps: pad -> move value by form (walk / cycle / surf) -> MMDL acmd, slope tiles (rail attributes), the
# cycling-road sound record, the input-hold record. Run: tools/check-fn.sh black-field player_nogrid --per-function
#
# File bound: `Heap_AllocDebug(heap, 0x24, 1, "field_player_nogrid.c", 0x1A0)` in 021A5880 (string 0x021DDF04);
# fieldmap_ctrl_nogrid_work.c starts at 021A6CB8. Function list: bl targets of the field set, odd code pointers (the slope proc
# table ROM 0x021D8ABC: 021A6908 / 6974 / 69D8 / 6A3C / 6A64 / 6A8C / 6AA0), the Ghidra index and a Thumb walk; fragments (switch
# cases / LAB_ pieces) 021A5A92, 021A5AFE, 021A5BE8, 021A5C64, 021A5FC0, 021A603C, 021A6390, 021A64E6, 021A699E, 021A6A02,
# 021A6A54, 021A6A7C, 021A6ABE; 021A5C76 (a `bl` from data in main) is inside 021A5C3C.
#
# NOGRID PLAYER N (0x24): +0 move state (0..3), +4 flags, +8 sound record (u8 playing, u8 kind 0..1, s16 frame 0..3), +0xC input
# record (u8 flags, s16 counter at +2, u32 held direction at +4), +0x14 PLAYER CORE, +0x18 FIELDMAP, +0x1C MMDL, +0x20 rail work.
#
# Arena: the last argument `buf 0x400`; N at +0x100 (fields drawn from their ranges), a LOCATION at +0x200, outs at +0x220; CORE /
# FIELDMAP / MMDL are values (all their accessors are `force` stubs). The overlay-21 data word 0x021DDEE0 (the rail MMDL header,
# +2 = OBJ code) is a `global`. Directions are 0..3 or 9 (none).
#
# Conventions (headers of the F3a / F5b / F6b specs): all blocks `nowork`; closure stubs (checked with every function treated as
# ported: no fault, stray write or unstubbed callee); every out-of-area stub is `force`; the slope proc table targets are in-area stubs
# of 021A682C (plain mode); stack locals reach stubs as `in:` / `out:` / `outn:` (the hit check's attribute out-word is 0 or a local).
area player_nogrid
set field

# FUN_021A5880(core, heap): create.
fn 021a5880 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x400
  global 021ddee0 0x24
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 02030734 force ret=ptr:arg2:0x100:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 021a395c force ret=u32 args=v  # PLAYER CORE: FIELDMAP (core)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 0216d5a0 force ret=pick+u32:0x45/0/1 args=v  # MMDL move code (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0219e348 force ret=u32 args=v  # MMDL rail work (m)
  stub 0219e2c0 force ret=none args=v,in:8  # MMDL rail: place at a LOCATION (m, &loc)
  stub 021a6b88 ret=none args=v  # no-grid player (in-area)
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # PLAYER CORE: move form (core)
  stub 021a3cb0 force ret=none args=v,v  # PLAYER CORE: request (core, bits)
  stub 021a3ce8 force ret=none args=v  # PLAYER CORE: run the requests (core)

# FUN_021A5920(n): delete (veneer to Heap_Free).
fn 021a5920 ret=none inputs=10
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021A5928(n, loc): rail header again, place at loc.
fn 021a5928 ret=none inputs=60
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E30200
  arg buf 0x400
  global 021ddee0 0x24
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5a0 force ret=pick+u32:0x45/0/1 args=v  # MMDL move code (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0219e348 force ret=u32 args=v  # MMDL rail work (m)
  stub 0219e2c0 force ret=none args=v,in:8  # MMDL rail: place at a LOCATION (m, &loc)

# FUN_021A5964(n): drop the rail work.
fn 021a5964 ret=none inputs=10
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A596C(n, trig, held): one frame.
fn 021a596c ret=none inputs=600
  nowork
  arg u32 const 0x2E40100
  arg u32 any
  arg u32 alt mask 3 || any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3ab8 force ret=pick+u32:0/1/2/3/9 args=v,v  # PLAYER CORE: pad -> direction (core, held)
  stub 021a3cc0 force ret=u32 args=v  # PLAYER CORE (core)
  stub 021a6bb4 ret=none args=v,v,v  # no-grid player (in-area)
  stub 021a682c ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a5a68 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b4c ret=none args=v,v,v  # no-grid player (in-area)
  stub 021a5ad4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b7c ret=none args=v  # no-grid player (in-area)
  stub 021a6c30 ret=none args=v  # no-grid player (in-area)
  stub 021a6908 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6974 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a69d8 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6a3c ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6a64 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6a8c ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6aa0 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a67a4 ret=pick:0/1/2/3/4/5/6/7 args=v  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # PLAYER CORE: move form (core)
  stub 021a64b4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a6014 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a5c3c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a3ac4 force ret=none args=v,v  # PLAYER CORE (core, held)
  stub 021a6738 ret=none args=v  # no-grid player (in-area)
  stub 021a672c ret=none args=v  # no-grid player (in-area)
  stub 021a635c ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f98 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5bc0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 021a6b08 ret=none args=v,v  # no-grid player (in-area)
  stub 021a6b50 ret=none args=v,v  # no-grid player (in-area)
  stub 021a6b30 ret=none args=v  # no-grid player (in-area)
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6744 ret=none args=v  # no-grid player (in-area)
  stub 021a675c ret=none args=v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a6774 ret=none args=v  # no-grid player (in-area)
  stub 021a678c ret=none args=v  # no-grid player (in-area)
  stub 021a6b9c ret=none args=v  # no-grid player (in-area)
  stub 021a6c4c ret=pick+u32:0/1/2/3/9 args=v,v  # no-grid player (in-area)
  stub 021a6c64 ret=pick:0/1 args=v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 0216d7dc force ret=u32 args=v  # MMDL: map attribute of its tile (m)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab348 force ret=pick+u32:0/1/0/2 args=v  # map attribute predicate (v)
  stub 021bb1ac force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1b8 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1c4 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1d0 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1dc force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1e8 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 021a666c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a661c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a658c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a652c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6200 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a61b0 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a60dc ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a607c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5e28 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5dd8 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5d04 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ca4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6478 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6454 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6428 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a63f0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a63ec ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6338 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62fc ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62d8 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62b4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a627c ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6278 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f74 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f38 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f14 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5edc ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ea4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ea0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5d8 force ret=none args=v,v  # MMDL: draw direction (m, d)
  stub 02006148 force ret=u32 args=v  # sound: handle of an SE (id)
  stub 020061f8 force ret=none args=v  # sound: stop (handle)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A59DC(n): state 3.
fn 021a59dc ret=u32 inputs=30
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A59EC(n): flag 1.
fn 021a59ec ret=u32 inputs=30
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)

# FUN_021A5A00(n, loc): thunk 0219E2C0.
fn 021a5a00 ret=none inputs=10
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E30200
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 0219e2c0 force ret=none args=v,in:8  # MMDL rail: place at a LOCATION (m, &loc)

# FUN_021A5A0C(n, out): thunk 0219E2F4.
fn 021a5a0c ret=none inputs=10
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E30220
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 0219e2f4 force ret=none args=v,out:8  # MMDL rail: its LOCATION (m, &out)

# FUN_021A5A18(n, dir, out): thunk 0219E664.
fn 021a5a18 ret=none inputs=10
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 const 0x2E40220
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)

# FUN_021A5A24(n, out): thunk 021B8028 (rail work position).
fn 021a5a24 ret=none inputs=10
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E30220
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 021b8028 force ret=none args=v,out:12  # rail: position (w, &out)

# FUN_021A5A30(n): the rail work.
fn 021a5a30 ret=u32 inputs=10
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A5A34(n): end a bump.
fn 021a5a34 ret=none inputs=60
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d674 force ret=none args=v,v  # MMDL (m, v)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A5A68(n, dir, trig, held, flag): move value by form.
fn 021a5a68 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E60100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg5 0x100 u32 alt range 0 3 || any
  fix arg5 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg5 0x108 u8 alt const 0 || const 1
  fix arg5 0x109 u8 range 0 1
  fix arg5 0x10A u16 range 0 3
  fix arg5 0x10C u8 alt mask 0x0F || any
  fix arg5 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg5 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # PLAYER CORE: move form (core)
  stub 021a64b4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a6014 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a5c3c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v  # no-grid player (in-area)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 021a666c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a661c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a658c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a652c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6200 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a61b0 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a60dc ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a607c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5e28 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5dd8 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5d04 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ca4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5AD4(n, value, dir, trig, held, flag): action by form.
fn 021a5ad4 ret=none inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # PLAYER CORE: move form (core)
  stub 021a635c ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f98 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5bc0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 021a6478 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6454 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6428 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a63f0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a63ec ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6338 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62fc ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62d8 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62b4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a627c ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6278 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f74 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f38 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f14 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5edc ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ea4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ea0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A5B4C(n, held, value).
fn 021a5b4c ret=none inputs=40
  nowork
  arg u32 const 0x2E40100
  arg u32 any
  arg u32 alt range 0 3 || any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3ac4 force ret=none args=v,v  # PLAYER CORE (core, held)
  stub 021a6738 ret=none args=v  # no-grid player (in-area)
  stub 021a672c ret=none args=v  # no-grid player (in-area)

# FUN_021A5B6C(n, m, dir, out): thunk 0219E424 (hit check).
fn 021a5b6c ret=u32 inputs=40
  nowork
  arg u32 const 0x2E50100
  arg u32 any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 0x2E50220
  arg buf 0x400
  fix arg4 0x100 u32 alt range 0 3 || any
  fix arg4 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg4 0x108 u8 alt const 0 || const 1
  fix arg4 0x109 u8 range 0 1
  fix arg4 0x10A u16 range 0 3
  fix arg4 0x10C u8 alt mask 0x0F || any
  fix arg4 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg4 0x110 u32 alt const 9 || range 0 3 || any
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5B7C(n): sound record.
fn 021a5b7c ret=none inputs=200
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 021a6b08 ret=none args=v,v  # no-grid player (in-area)
  stub 021a6b50 ret=none args=v,v  # no-grid player (in-area)
  stub 021a6b30 ret=none args=v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d5d8 force ret=none args=v,v  # MMDL: draw direction (m, d)
  stub 02006148 force ret=u32 args=v  # sound: handle of an SE (id)
  stub 020061f8 force ret=none args=v  # sound: stop (handle)

# FUN_021A5BC0(n, value, trig, held, dir, flag): walk action.
fn 021a5bc0 ret=none inputs=400
  nowork
  arg u32 const 0x2E70100
  arg u32 alt range 0 6 || any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a5f74 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f38 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5f14 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5edc ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ea4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ea0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A5C3C(n, trig, held, dir, flag): walk move value by state.
fn 021a5c3c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E60100
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg5 0x100 u32 alt range 0 3 || any
  fix arg5 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg5 0x108 u8 alt const 0 || const 1
  fix arg5 0x109 u8 range 0 1
  fix arg5 0x10A u16 range 0 3
  fix arg5 0x10C u8 alt mask 0x0F || any
  fix arg5 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg5 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a5e28 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5dd8 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5d04 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5ca4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5CA4(n, m, trig, held, dir, flag): move value.
fn 021a5ca4 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a5d04 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5D04(n, m, trig, held, dir, flag): move value.
fn 021a5d04 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5ca4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5DD8(n, m, trig, held, dir, flag): move value.
fn 021a5dd8 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5ca4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5d04 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5E28(n, m, trig, held, dir, flag): move value.
fn 021a5e28 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 021a5d04 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5ca4 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A5EA0(n, m, trig, held, dir, flag): action.
fn 021a5ea0 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A5EA4(n, m, trig, held, dir, flag): action.
fn 021a5ea4 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A5EDC(n, m, trig, held, dir, flag): action.
fn 021a5edc ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A5F14(n, m, trig, held, dir, flag): action.
fn 021a5f14 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A5F38(n, m, trig, held, dir, flag): action.
fn 021a5f38 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A5F74(n, m, trig, held, dir, flag): action.
fn 021a5f74 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A5F98(n, value, trig, held, dir, flag): cycle action.
fn 021a5f98 ret=none inputs=400
  nowork
  arg u32 const 0x2E70100
  arg u32 alt range 0 6 || any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a6338 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62fc ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62d8 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a62b4 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a627c ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6278 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A6014(n, trig, held, dir, flag): cycle move value by state.
fn 021a6014 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E60100
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg5 0x100 u32 alt range 0 3 || any
  fix arg5 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg5 0x108 u8 alt const 0 || const 1
  fix arg5 0x109 u8 range 0 1
  fix arg5 0x10A u16 range 0 3
  fix arg5 0x10C u8 alt mask 0x0F || any
  fix arg5 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg5 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a6200 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a61b0 ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a60dc ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a607c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A607C(n, m, trig, held, dir, flag): move value.
fn 021a607c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a60dc ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A60DC(n, m, trig, held, dir, flag): move value.
fn 021a60dc ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a607c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A61B0(n, m, trig, held, dir, flag): move value.
fn 021a61b0 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a607c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a60dc ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A6200(n, m, trig, held, dir, flag): move value.
fn 021a6200 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 021a60dc ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a607c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 0219d724 force ret=u32 args=v,in:8  # MMDL rail: attribute at a LOCATION (m, &loc)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 021ab158 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab164 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab170 force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 021ab17c force ret=pick:0/1/0 args=v  # map attribute: one-way tile predicate (v)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A6278(n, m, trig, held, dir, flag): action.
fn 021a6278 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A627C(n, m, trig, held, dir, flag): action.
fn 021a627c ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A62B4(n, m, trig, held, dir, flag): action.
fn 021a62b4 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A62D8(n, m, trig, held, dir, flag): action.
fn 021a62d8 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A62FC(n, m, trig, held, dir, flag): action.
fn 021a62fc ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A6338(n, m, trig, held, dir, flag): action.
fn 021a6338 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A635C(n, value, trig, held, dir, flag): surf action.
fn 021a635c ret=none inputs=400
  nowork
  arg u32 const 0x2E70100
  arg u32 alt range 0 6 || any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 021a6478 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6454 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a6428 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a63f0 ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a63ec ret=none args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A63EC(n, m, trig, held, dir, flag): action.
fn 021a63ec ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A63F0(n, m, trig, held, dir, flag): action.
fn 021a63f0 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A6428(n, m, trig, held, dir, flag): action.
fn 021a6428 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A6454(n, m, trig, held, dir, flag): action.
fn 021a6454 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)

# FUN_021A6478(n, m, trig, held, dir, flag): action.
fn 021a6478 ret=none inputs=100
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6c6c ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)

# FUN_021A64B4(n, trig, held, dir, flag): surf move value by state.
fn 021a64b4 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E60100
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg5 0x100 u32 alt range 0 3 || any
  fix arg5 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg5 0x108 u8 alt const 0 || const 1
  fix arg5 0x109 u8 range 0 1
  fix arg5 0x10A u16 range 0 3
  fix arg5 0x10C u8 alt mask 0x0F || any
  fix arg5 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg5 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 021a666c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a661c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a658c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a652c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A652C(n, m, trig, held, dir, flag): move value.
fn 021a652c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a658c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A658C(n, m, trig, held, dir, flag): move value.
fn 021a658c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a652c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A661C(n, m, trig, held, dir, flag): move value.
fn 021a661c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a652c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a658c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A666C(n, m, trig, held, dir, flag): move value.
fn 021a666c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70100
  arg u32 any
  arg u32 any
  arg u32 alt mask 3 || any
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 1 || any
  arg buf 0x400
  fix arg6 0x100 u32 alt range 0 3 || any
  fix arg6 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg6 0x108 u8 alt const 0 || const 1
  fix arg6 0x109 u8 range 0 1
  fix arg6 0x10A u16 range 0 3
  fix arg6 0x10C u8 alt mask 0x0F || any
  fix arg6 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg6 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 021a658c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a652c ret=pick:0/1/2/3/4/5 args=v,v,v,v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab0f8 force ret=pick+u32:0/1/2/3 args=v  # map attribute: flags (attr)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A66E4(n, bits): set flags.
fn 021a66e4 ret=none inputs=20
  nowork
  arg u32 const 0x2E30100
  arg u32 any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A66EC(n, bits): clear flags.
fn 021a66ec ret=none inputs=20
  nowork
  arg u32 const 0x2E30100
  arg u32 any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A66F8(n, bits): test flags.
fn 021a66f8 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30100
  arg u32 alt mask 0x7F || any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6708(n): set / clear one flag (thunk).
fn 021a6708 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6714(n): set / clear one flag (thunk).
fn 021a6714 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6720(n): set / clear one flag (thunk).
fn 021a6720 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A672C(n): set / clear one flag (thunk).
fn 021a672c ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6738(n): set / clear one flag (thunk).
fn 021a6738 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6744(n): set / clear one flag (thunk).
fn 021a6744 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6750(n): set / clear one flag (thunk).
fn 021a6750 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A675C(n): set / clear one flag (thunk).
fn 021a675c ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6768(n): set / clear one flag (thunk).
fn 021a6768 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6774(n): set / clear one flag (thunk).
fn 021a6774 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6780(n): set / clear one flag (thunk).
fn 021a6780 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A678C(n): set / clear one flag (thunk).
fn 021a678c ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6798(n): set / clear one flag (thunk).
fn 021a6798 ret=none inputs=20
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A67A4(n): slope kind.
fn 021a67a4 ret=u32 inputs=400
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d7dc force ret=u32 args=v  # MMDL: map attribute of its tile (m)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab348 force ret=pick+u32:0/1/0/2 args=v  # map attribute predicate (v)
  stub 021bb1ac force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1b8 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1c4 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1d0 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1dc force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1e8 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)

# FUN_021A682C(n, dir, x): slope proc.
fn 021a682c ret=u32 inputs=600
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || const 0 || any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a67a4 ret=pick:0/1/2/3/4/5/6/7 args=v  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6908 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6974 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a69d8 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6a3c ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6a64 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6a8c ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6aa0 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 0216d7dc force ret=u32 args=v  # MMDL: map attribute of its tile (m)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021ab348 force ret=pick+u32:0/1/0/2 args=v  # map attribute predicate (v)
  stub 021bb1ac force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1b8 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1c4 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1d0 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1dc force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 021bb1e8 force ret=pick+u32:0/1/0/0/0/0/0 args=v  # rail attribute predicate (v)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a6744 ret=none args=v  # no-grid player (in-area)
  stub 021a675c ret=none args=v  # no-grid player (in-area)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a6774 ret=none args=v  # no-grid player (in-area)
  stub 021a678c ret=none args=v  # no-grid player (in-area)
  stub 021a6b9c ret=none args=v  # no-grid player (in-area)
  stub 021a6c4c ret=pick+u32:0/1/2/3/9 args=v,v  # no-grid player (in-area)
  stub 021a6c64 ret=pick:0/1 args=v  # no-grid player (in-area)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)

# FUN_021A6870(n): end a slide.
fn 021a6870 ret=none inputs=200
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A6908(n, dir, x): slope proc.
fn 021a6908 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A6974(n, dir, x): slope proc.
fn 021a6974 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a6744 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A69D8(n, dir, x): slope proc.
fn 021a69d8 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a675c ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A6A3C(n, dir, x): slope proc.
fn 021a6a3c ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a6908 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A6A64(n, dir, x): slope proc.
fn 021a6a64 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 021a6908 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A6A8C(n, dir, x): slope proc.
fn 021a6a8c ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a6908 ret=pick+u32:0/1/2/3/9 args=v,v,v  # no-grid player (in-area)
  stub 021a6774 ret=none args=v  # no-grid player (in-area)
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 021a5b6c ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # no-grid player (in-area)
  stub 021a6870 ret=none args=v  # no-grid player (in-area)
  stub 021a6708 ret=none args=v  # no-grid player (in-area)
  stub 021a6750 ret=none args=v  # no-grid player (in-area)
  stub 021a6768 ret=none args=v  # no-grid player (in-area)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0219e424 force ret=pick+u32:0/1/2/3/0x10/0x12 args=v,v,outn:4:0xFF/0xFF/0xFF/0xFF  # MMDL rail: hit check (m, dir, &attr or 0)
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)
  stub 021a6714 ret=none args=v  # no-grid player (in-area)
  stub 021a6720 ret=none args=v  # no-grid player (in-area)
  stub 021642a4 force ret=pick:0/1/2/3 args=v  # opposite direction (d)
  stub 0216d60c force ret=none args=v,v  # MMDL: move direction (m, d)
  stub 021a6780 ret=none args=v  # no-grid player (in-area)
  stub 021a6798 ret=none args=v  # no-grid player (in-area)
  stub 021a6bac ret=none args=v  # no-grid player (in-area)

# FUN_021A6AA0(n, dir, x): slope proc.
fn 021a6aa0 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E40100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a66f8 ret=pick:0/1 args=v,v  # no-grid player (in-area)
  stub 021a678c ret=none args=v  # no-grid player (in-area)
  stub 021a6b9c ret=none args=v  # no-grid player (in-area)
  stub 021a6c4c ret=pick+u32:0/1/2/3/9 args=v,v  # no-grid player (in-area)
  stub 021a6c64 ret=pick:0/1 args=v  # no-grid player (in-area)
  stub 0216d548 force ret=none args=v,v  # MMDL: status bits on (m, bits)
  stub 02197e7c force ret=u32 args=v,v  # acmd code of a direction (dir, base)
  stub 0216d410 force ret=none args=v,v  # MMDL: set an acmd (m, code)
  stub 021a3964 force ret=none args=v,v  # PLAYER CORE: move state (core, st)
  stub 0216d550 force ret=none args=v,v  # MMDL: status bits off (m, bits)

# FUN_021A6B08(e, kind): start the sound record.
fn 021a6b08 ret=none inputs=60
  nowork
  arg u32 const 0x2E30108
  arg u32 alt range 0 1 || any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)

# FUN_021A6B30(e): stop it.
fn 021a6b30 ret=none inputs=30
  nowork
  arg u32 const 0x2E20108
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any
  stub 02006148 force ret=u32 args=v  # sound: handle of an SE (id)
  stub 020061f8 force ret=none args=v  # sound: stop (handle)

# FUN_021A6B50(e, m): its frame.
fn 021a6b50 ret=none inputs=60
  nowork
  arg u32 const 0x2E30108
  arg u32 any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d5d8 force ret=none args=v,v  # MMDL: draw direction (m, d)

# FUN_021A6B88(r): clear the input record.
fn 021a6b88 ret=none inputs=10
  nowork
  arg u32 const 0x2E2010C
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6B9C(r): hold.
fn 021a6b9c ret=none inputs=10
  nowork
  arg u32 const 0x2E2010C
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6BAC(r): thunk to 021A6B88.
fn 021a6bac ret=none inputs=10
  nowork
  arg u32 const 0x2E2010C
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6BB4(r, dir, m): input record frame.
fn 021a6bb4 ret=none inputs=400
  nowork
  arg u32 const 0x2E4010C
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt range 0 3 || any
  fix arg3 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg3 0x108 u8 alt const 0 || const 1
  fix arg3 0x109 u8 range 0 1
  fix arg3 0x10A u16 range 0 3
  fix arg3 0x10C u8 alt mask 0x0F || any
  fix arg3 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg3 0x110 u32 alt const 9 || range 0 3 || any
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)

# FUN_021A6C30(r): end of frame.
fn 021a6c30 ret=none inputs=40
  nowork
  arg u32 const 0x2E2010C
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6C4C(r, dir): held direction.
fn 021a6c4c ret=u32 inputs=40
  nowork
  arg u32 const 0x2E3010C
  arg u32 alt range 0 3 || const 9 || any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6C64(r): bit 1.
fn 021a6c64 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E2010C
  arg buf 0x400
  fix arg1 0x100 u32 alt range 0 3 || any
  fix arg1 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg1 0x108 u8 alt const 0 || const 1
  fix arg1 0x109 u8 range 0 1
  fix arg1 0x10A u16 range 0 3
  fix arg1 0x10C u8 alt mask 0x0F || any
  fix arg1 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg1 0x110 u32 alt const 9 || range 0 3 || any

# FUN_021A6C6C(n, dir): an event in front.
fn 021a6c6c ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30100
  arg u32 alt range 0 3 || const 9 || any
  arg buf 0x400
  fix arg2 0x100 u32 alt range 0 3 || any
  fix arg2 0x104 u32 alt const 0 || mask 0x7F || any
  fix arg2 0x108 u8 alt const 0 || const 1
  fix arg2 0x109 u8 range 0 1
  fix arg2 0x10A u16 range 0 3
  fix arg2 0x10C u8 alt mask 0x0F || any
  fix arg2 0x10E u16 alt range 28 31 || range 0 4 || any
  fix arg2 0x110 u32 alt const 9 || range 0 3 || any
  stub 021a3960 force ret=u32 args=v  # PLAYER CORE: the player MMDL (core)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0219e664 force ret=none args=v,v,out:8  # MMDL rail: LOCATION in front (m, dir, &out)
  stub 02162598 force ret=pick+u32:0/1/0xFFFF args=v,in:8  # eventdata: event at a rail LOCATION (ed, &loc)
