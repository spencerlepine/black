# fndiff spec (F3b): `fieldmap_ctrl_nogrid_work.c` (overlay 21 Thumb code 0x021A6CB8..0x021A6DC4; F1 row 21.25): the MAP CTRL of the
# free-movement (rail) maps (the no-grid entry of the MAP CTRL table ROM 0x021D8ADC: create 021A6CB8, delete 021A6D30, frame 021A6D3C,
# position 021A6D48) and its work (021A6D58 .. 021A6DC0). Run: tools/check-fn.sh black-field ctrl_nogrid --per-function
#
# File bound: `Heap_AllocDebug(heap, 8, 1, "fieldmap_ctrl_nogrid_work.c", 0x3E)` in 021A6D58 (string 0x021DDF1C); fieldmap_ctrl_hybrid.c
# starts at 021A6DC4. Work (8): +0 FIELD_PLAYER. Arena: the last argument `buf 0x400` (the work at +0x100; FIELDMAP 02188CA4 answers it);
# positions at +0x200. Conventions: as the player_nogrid spec (all blocks `nowork`, closure stubs, every out-of-area stub `force`).
area ctrl_nogrid
set field

# FUN_021A6CB8(fm, pos, dir): create.
fn 021a6cb8 ret=none inputs=40
  nowork
  arg u32 any
  arg u32 const 0x2E40200
  arg u32 any
  arg buf 0x400
  fix arg3 0x100 u32 alt const 0 || const 1 || const 2
  fix arg3 0x108 u32 alt const 0 || any
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 02188c84 force ret=u32 args=v  # FIELDMAP -> heap
  stub 021a6d58 ret=u32 args=v,v  # MAP CTRL (in-area)
  stub 02188cac force ret=none args=v,v  # FIELDMAP: MAP CTRL work (fm, w)
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216d888 force ret=u32 args=v  # MMDL -> position (pointer)
  stub 02188d40 force ret=none args=v,v  # FIELDMAP: camera target (fm, &pos)
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02012ab8 force ret=ptr:arg3:0x200:1 args=v  # PLAYER record -> rail location (pointer)
  stub 021a36b0 force ret=none args=v,v  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36d4 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021a305c force ret=u32 args=v,v  # FIELD_PLAYER: direction (pl, dir)
  stub 02030734 force ret=ptr:arg3:0x100:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 021a3664 force ret=none args=v,v  # FIELD_PLAYER: create the no-grid part (pl, heap)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)

# FUN_021A6D30(fm): delete.
fn 021a6d30 ret=none inputs=20
  nowork
  arg u32 any
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 02188ca4 force ret=ptr:arg1:0x100:1 args=v  # FIELDMAP -> MAP CTRL work
  stub 021a6d9c ret=none args=v  # MAP CTRL (in-area)
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021A6D3C(fm): frame.
fn 021a6d3c ret=none inputs=30
  nowork
  arg u32 any
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 02188ca4 force ret=ptr:arg1:0x100:1 args=v  # FIELDMAP -> MAP CTRL work
  stub 021a6da4 ret=none args=v  # MAP CTRL (in-area)
  stub 020362dc force ret=pick+u32:0/0x10/0x20/0x40/0x80/1/2 args=  # pad trigger
  stub 0203630c force ret=pick+u32:0/0x10/0x20/0x40/0x80/2/0x402 args=  # pad held
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)

# FUN_021A6D48(fm): the player MMDL position.
fn 021a6d48 ret=u32 inputs=20
  nowork
  arg u32 any
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216d888 force ret=u32 args=v  # MMDL -> position (pointer)

# FUN_021A6D58(fm, heap): the work.
fn 021a6d58 ret=u32 inputs=40
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x400
  fix arg2 0x100 u32 alt const 0 || const 1 || const 2
  fix arg2 0x108 u32 alt const 0 || any
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02030734 force ret=ptr:arg2:0x100:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 021a3664 force ret=none args=v,v  # FIELD_PLAYER: create the no-grid part (pl, heap)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)

# FUN_021A6D9C(w): free (veneer).
fn 021a6d9c ret=none inputs=10
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021A6DA4(w): frame.
fn 021a6da4 ret=none inputs=30
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 020362dc force ret=pick+u32:0/0x10/0x20/0x40/0x80/1/2 args=  # pad trigger
  stub 0203630c force ret=pick+u32:0/0x10/0x20/0x40/0x80/2/0x402 args=  # pad held
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)

# FUN_021A6DC0(w): the FIELD_PLAYER.
fn 021a6dc0 ret=u32 inputs=10
  nowork
  arg u32 const 0x2E20100
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
