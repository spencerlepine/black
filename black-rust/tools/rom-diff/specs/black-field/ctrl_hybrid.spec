# fndiff spec (F3b): `fieldmap_ctrl_hybrid.c` (overlay 21 Thumb code 0x021A6DC4..0x021A7358; F1 row 21.26): the MAP CTRL of the maps that
# mix grid and rails (the hybrid entry of the MAP CTRL table ROM 0x021D8AF0: create 021A6E30, frame 021A6F24, delete 021A6F18, position
# 021A6F4C), the mode frames (ROM 0x021DDF38: grid 021A6F5C, rail 021A7010), the switches and the mode change 021A7258. 021A6DC4 has no
# caller in the field set. Run: tools/check-fn.sh black-field ctrl_hybrid --per-function
#
# File bound: `Heap_AllocDebug(heap, 0xC, 1, "fieldmap_ctrl_hybrid.c", 0xBD)` in 021A6E30 (string 0x021DDF64); field_ground_anime.c starts
# at 021A7360. HYBRID work (0xC): +0 mode (0 grid, 1 rail, 2 unset), +4 FIELD_PLAYER, +8 keep-direction flag. The mode is drawn 0..2
# (021A6F24 reads the 2-entry ROM proc table, so its block draws 0..1). Overlay-21 data 0x021DDF40 (the grid MMDL header, +2 = OBJ code)
# is a `global`. Arena: the last argument `buf 0x400` (the work at +0x100; FIELDMAP 02188CA4 answers it), positions / LOCATIONs at
# +0x200. Conventions: as the player_nogrid spec (all blocks `nowork`, closure stubs, every out-of-area stub `force`, stack locals as
# `in:` / `out:`); the frame procs of the ROM table are in-area stubs (plain mode).
area ctrl_hybrid
set field

# FUN_021A6DC4(h, fm): re-enter the current mode.
fn 021a6dc4 ret=none inputs=200
  nowork
  arg u32 const 0x2E30100
  arg u32 any
  arg buf 0x400
  global 021ddf40 0x24
  fix arg2 0x100 u32 alt const 0 || const 1 || const 2
  fix arg2 0x108 u32 alt const 0 || any
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a71e4 ret=pick:0/1 args=v,v,out:8,v  # MAP CTRL (in-area)
  stub 021a70d0 ret=none args=v,v,v,v,in:8,v,v  # MAP CTRL (in-area)
  stub 021a7198 ret=pick:0/1 args=v,v,out:12,v  # MAP CTRL (in-area)
  stub 021a712c ret=none args=v,v,v,v,in:12,v,v  # MAP CTRL (in-area)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 021bb480 force ret=u32 args=v  # no-grid mapper -> RAIL system
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 021b7a84 force ret=pick:0/1 args=v,in:12,in:12,out:8,out:12  # rail: nearest line hit of a segment (R, &a, &b, &loc, &pos)
  stub 0201441c force ret=u32 args=v  # direction code
  stub 021b7cd8 force ret=pick:0/1 args=v,in:8,v,out:8  # rail: one step (R, &loc, dir, &out)
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 021a35f0 force ret=none args=v,v,v,v  # FIELD_PLAYER: grid frame (pl, trig, held, flags)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)
  stub 02188c98 force ret=u32 args=v  # FIELDMAP -> map attribute grid
  stub 02164208 force ret=none args=v,in:12,v  # advance a position in a direction (dir, &pos, dist)
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (grid, &pos)
  stub 021ab11c force ret=pick+u32:0/1 args=v  # map attribute predicate (attr)
  stub 021a35bc force ret=pick+u32:0/1 args=v  # FIELD_PLAYER: running allowed (pl)
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A6E30(fm, pos, dir): create.
fn 021a6e30 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 const 0x2E40200
  arg u32 any
  arg buf 0x400
  global 021ddf40 0x24
  fix arg3 0x100 u32 alt const 0 || const 1 || const 2
  fix arg3 0x108 u32 alt const 0 || any
  stub 02188c80 force ret=u32 args=v  # FIELDMAP -> GAMESYS
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER record
  stub 02188c84 force ret=u32 args=v  # FIELDMAP -> heap
  stub 02030734 force ret=ptr:arg3:0x100:1 args=v,v,v,v,v  # Heap_AllocDebug (heap, size, flag, file, line)
  stub 02188cac force ret=none args=v,v  # FIELDMAP: MAP CTRL work (fm, w)
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 02012ac0 force ret=pick+u32:0/1/2 args=v  # PLAYER record: position kind (rec)
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 021a35e0 force ret=none args=v,v  # FIELD_PLAYER: create the grid part (pl, heap)
  stub 021a3664 force ret=none args=v,v  # FIELD_PLAYER: create the no-grid part (pl, heap)
  stub 021a305c force ret=u32 args=v,v  # FIELD_PLAYER: direction (pl, dir)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 0216d888 force ret=u32 args=v  # MMDL -> position (pointer)
  stub 02188d40 force ret=none args=v,v  # FIELDMAP: camera target (fm, &pos)
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 02012ab8 force ret=ptr:arg3:0x200:1 args=v  # PLAYER record -> rail location (pointer)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A6F18(fm): delete.
fn 021a6f18 ret=none inputs=20
  nowork
  arg u32 any
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 02188ca4 force ret=ptr:arg1:0x100:1 args=v  # FIELDMAP -> MAP CTRL work
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021A6F24(fm, out): frame.
fn 021a6f24 ret=none inputs=300
  nowork
  arg u32 any
  arg u32 const 0x2E30220
  arg buf 0x400
  global 021ddf40 0x24
  fix arg2 0x100 u32 alt const 0 || const 1 || const 2
  fix arg2 0x108 u32 alt const 0 || any
  fix arg2 0x100 u32 alt const 0 || const 1
  stub 02188ca4 force ret=ptr:arg2:0x100:1 args=v  # FIELDMAP -> MAP CTRL work
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 021a6f5c ret=none args=v,v  # MAP CTRL (in-area)
  stub 021a7010 ret=none args=v,v  # MAP CTRL (in-area)
  stub 020362dc force ret=pick+u32:0/0x10/0x20/0x40/0x80/1/2 args=  # pad trigger
  stub 0203630c force ret=pick+u32:0/0x10/0x20/0x40/0x80/2/0x402 args=  # pad held
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 021a2fa4 force ret=pick+u32:0/1/2/3/9 args=v,v  # FIELD_PLAYER: pad -> direction (pl, held)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a2fb0 force ret=pick:0/1 args=v  # FIELD_PLAYER predicate (pl)
  stub 021a30c8 force ret=u32 args=v  # FIELD_PLAYER: attribute here
  stub 021a3128 force ret=u32 args=v,v  # FIELD_PLAYER: attribute ahead (pl, dir)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021bb1a0 force ret=pick+u32:0/1/1 args=v  # rail attribute 0xA3 (v)
  stub 021ab11c force ret=pick+u32:0/1 args=v  # map attribute predicate (attr)
  stub 021a71e4 ret=pick:0/1 args=v,v,out:8,v  # MAP CTRL (in-area)
  stub 021a70d0 ret=none args=v,v,v,v,in:8,v,v  # MAP CTRL (in-area)
  stub 021a35bc force ret=pick+u32:0/1 args=v  # FIELD_PLAYER: running allowed (pl)
  stub 021a35f0 force ret=none args=v,v,v,v  # FIELD_PLAYER: grid frame (pl, trig, held, flags)
  stub 021a7198 ret=pick:0/1 args=v,v,out:12,v  # MAP CTRL (in-area)
  stub 021a712c ret=none args=v,v,v,v,in:12,v,v  # MAP CTRL (in-area)
  stub 021a3698 force ret=u32 args=v  # FIELD_PLAYER: keep direction
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 021bb480 force ret=u32 args=v  # no-grid mapper -> RAIL system
  stub 021b7a84 force ret=pick:0/1 args=v,in:12,in:12,out:8,out:12  # rail: nearest line hit of a segment (R, &a, &b, &loc, &pos)
  stub 0201441c force ret=u32 args=v  # direction code
  stub 021b7cd8 force ret=pick:0/1 args=v,in:8,v,out:8  # rail: one step (R, &loc, dir, &out)
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 02188c98 force ret=u32 args=v  # FIELDMAP -> map attribute grid
  stub 02164208 force ret=none args=v,in:12,v  # advance a position in a direction (dir, &pos, dist)
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (grid, &pos)
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A6F4C(fm): the player MMDL position.
fn 021a6f4c ret=u32 inputs=20
  nowork
  arg u32 any
  arg buf 0x400
  fix arg1 0x100 u32 alt const 0 || const 1 || const 2
  fix arg1 0x108 u32 alt const 0 || any
  stub 02188cb4 force ret=u32 args=v  # FIELDMAP -> FIELD_PLAYER
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216d888 force ret=u32 args=v  # MMDL -> position (pointer)

# FUN_021A6F5C(fm, h): grid frame.
fn 021a6f5c ret=none inputs=400
  nowork
  arg u32 any
  arg u32 const 0x2E30100
  arg buf 0x400
  global 021ddf40 0x24
  fix arg2 0x100 u32 alt const 0 || const 1 || const 2
  fix arg2 0x108 u32 alt const 0 || any
  stub 020362dc force ret=pick+u32:0/0x10/0x20/0x40/0x80/1/2 args=  # pad trigger
  stub 0203630c force ret=pick+u32:0/0x10/0x20/0x40/0x80/2/0x402 args=  # pad held
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 021a2fa4 force ret=pick+u32:0/1/2/3/9 args=v,v  # FIELD_PLAYER: pad -> direction (pl, held)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a2fb0 force ret=pick:0/1 args=v  # FIELD_PLAYER predicate (pl)
  stub 021a30c8 force ret=u32 args=v  # FIELD_PLAYER: attribute here
  stub 021a3128 force ret=u32 args=v,v  # FIELD_PLAYER: attribute ahead (pl, dir)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021bb1a0 force ret=pick+u32:0/1/1 args=v  # rail attribute 0xA3 (v)
  stub 021ab11c force ret=pick+u32:0/1 args=v  # map attribute predicate (attr)
  stub 021a71e4 ret=pick:0/1 args=v,v,out:8,v  # MAP CTRL (in-area)
  stub 021a70d0 ret=none args=v,v,v,v,in:8,v,v  # MAP CTRL (in-area)
  stub 021a35bc force ret=pick+u32:0/1 args=v  # FIELD_PLAYER: running allowed (pl)
  stub 021a35f0 force ret=none args=v,v,v,v  # FIELD_PLAYER: grid frame (pl, trig, held, flags)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 021bb480 force ret=u32 args=v  # no-grid mapper -> RAIL system
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 021b7a84 force ret=pick:0/1 args=v,in:12,in:12,out:8,out:12  # rail: nearest line hit of a segment (R, &a, &b, &loc, &pos)
  stub 0201441c force ret=u32 args=v  # direction code
  stub 021b7cd8 force ret=pick:0/1 args=v,in:8,v,out:8  # rail: one step (R, &loc, dir, &out)
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A7010(fm, h): rail frame.
fn 021a7010 ret=none inputs=400
  nowork
  arg u32 any
  arg u32 const 0x2E30100
  arg buf 0x400
  global 021ddf40 0x24
  fix arg2 0x100 u32 alt const 0 || const 1 || const 2
  fix arg2 0x108 u32 alt const 0 || any
  stub 020362dc force ret=pick+u32:0/0x10/0x20/0x40/0x80/1/2 args=  # pad trigger
  stub 0203630c force ret=pick+u32:0/0x10/0x20/0x40/0x80/2/0x402 args=  # pad held
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 021a2fa4 force ret=pick+u32:0/1/2/3/9 args=v,v  # FIELD_PLAYER: pad -> direction (pl, held)
  stub 0216d3cc force ret=pick+u32:0/1/2 args=v  # MMDL: acmd ended (m)
  stub 021a2fb0 force ret=pick:0/1 args=v  # FIELD_PLAYER predicate (pl)
  stub 021a30c8 force ret=u32 args=v  # FIELD_PLAYER: attribute here
  stub 021a3128 force ret=u32 args=v,v  # FIELD_PLAYER: attribute ahead (pl, dir)
  stub 021ab0f0 force ret=u32 args=v  # map attribute: value (attr)
  stub 021bb1a0 force ret=pick+u32:0/1/1 args=v  # rail attribute 0xA3 (v)
  stub 021ab11c force ret=pick+u32:0/1 args=v  # map attribute predicate (attr)
  stub 021a7198 ret=pick:0/1 args=v,v,out:12,v  # MAP CTRL (in-area)
  stub 021a712c ret=none args=v,v,v,v,in:12,v,v  # MAP CTRL (in-area)
  stub 021a3698 force ret=u32 args=v  # FIELD_PLAYER: keep direction
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)
  stub 0216d608 force ret=pick+u32:0/1/2/3 args=v  # MMDL move direction (m)
  stub 02188c98 force ret=u32 args=v  # FIELDMAP -> map attribute grid
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 02164208 force ret=none args=v,in:12,v  # advance a position in a direction (dir, &pos, dist)
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (grid, &pos)
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 021a35bc force ret=pick+u32:0/1 args=v  # FIELD_PLAYER: running allowed (pl)
  stub 021a35f0 force ret=none args=v,v,v,v  # FIELD_PLAYER: grid frame (pl, trig, held, flags)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A70D0(fm, h, dir, keep, loc, trig, held): to the rails.
fn 021a70d0 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 const 0x2E80100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || any
  arg u32 const 0x2E80200
  arg u32 any
  arg u32 any
  arg buf 0x400
  global 021ddf40 0x24
  fix arg7 0x100 u32 alt const 0 || const 1 || const 2
  fix arg7 0x108 u32 alt const 0 || any
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 021a35f0 force ret=none args=v,v,v,v  # FIELD_PLAYER: grid frame (pl, trig, held, flags)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A712C(fm, h, dir, keep, pos, trig, held): to the grid.
fn 021a712c ret=none inputs=200
  nowork
  arg u32 any
  arg u32 const 0x2E80100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || any
  arg u32 const 0x2E80200
  arg u32 any
  arg u32 any
  arg buf 0x400
  global 021ddf40 0x24
  fix arg7 0x100 u32 alt const 0 || const 1 || const 2
  fix arg7 0x108 u32 alt const 0 || any
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0219f058 force ret=none args=v,out:2,out:2  # MMDL billboard animation values (m, &a, &b)
  stub 021a3674 force ret=none args=v,v,v  # FIELD_PLAYER: no-grid frame (pl, trig, held)
  stub 0216d5f8 force ret=pick+u32:0/1/2/3 args=v  # MMDL facing direction (m)
  stub 021a7258 ret=none args=v,v,v,in:8,v,v,v  # MAP CTRL (in-area)
  stub 021a35bc force ret=pick+u32:0/1 args=v  # FIELD_PLAYER: running allowed (pl)
  stub 021a35f0 force ret=none args=v,v,v,v  # FIELD_PLAYER: grid frame (pl, trig, held, flags)
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)

# FUN_021A7198(fm, h, out, dir): grid entry ahead.
fn 021a7198 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 const 0x2E50100
  arg u32 const 0x2E50220
  arg u32 alt range 0 3 || const 9 || any
  arg buf 0x400
  fix arg4 0x100 u32 alt const 0 || const 1 || const 2
  fix arg4 0x108 u32 alt const 0 || any
  stub 02188c98 force ret=u32 args=v  # FIELDMAP -> map attribute grid
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 02164208 force ret=none args=v,in:12,v  # advance a position in a direction (dir, &pos, dist)
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (grid, &pos)
  stub 021ab11c force ret=pick+u32:0/1 args=v  # map attribute predicate (attr)

# FUN_021A71E4(fm, h, loc, dir): rail entry below.
fn 021a71e4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 const 0x2E50100
  arg u32 const 0x2E50220
  arg u32 alt range 0 3 || const 9 || any
  arg buf 0x400
  fix arg4 0x100 u32 alt const 0 || const 1 || const 2
  fix arg4 0x108 u32 alt const 0 || any
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 021bb480 force ret=u32 args=v  # no-grid mapper -> RAIL system
  stub 021a3038 force ret=none args=v,out:12  # FIELD_PLAYER: position (pl, &out)
  stub 021b7a84 force ret=pick:0/1 args=v,in:12,in:12,out:8,out:12  # rail: nearest line hit of a segment (R, &a, &b, &loc, &pos)
  stub 0201441c force ret=u32 args=v  # direction code
  stub 021b7cd8 force ret=pick:0/1 args=v,in:8,v,out:8  # rail: one step (R, &loc, dir, &out)

# FUN_021A7258(fm, h, mode, at, dir, a, b): change the mode.
fn 021a7258 ret=none inputs=300
  nowork
  arg u32 any
  arg u32 const 0x2E80100
  arg u32 alt range 0 2 || any
  arg u32 const 0x2E80200
  arg u32 alt range 0 3 || const 9 || any
  arg u32 any
  arg u32 any
  arg buf 0x400
  global 021ddf40 0x24
  fix arg7 0x100 u32 alt const 0 || const 1 || const 2
  fix arg7 0x108 u32 alt const 0 || any
  stub 02188c5c force ret=u32 args=v  # FIELDMAP -> no-grid mapper
  stub 02188c58 force ret=u32 args=v  # FIELDMAP -> camera
  stub 021a3074 force ret=u32 args=v  # FIELD_PLAYER -> MMDL
  stub 0216dc4c force ret=pick:0/1 args=v  # MMDL: acmd set (m)
  stub 0216d4b4 force ret=none args=v  # MMDL: cancel the acmd (m)
  stub 0216d598 force ret=u32 args=v  # MMDL OBJ code (m)
  stub 021a368c force ret=u32 args=v  # FIELD_PLAYER (pl)
  stub 0216e0ac force ret=none args=v,v  # MMDL: apply a header (m, &header)
  stub 0216e084 force ret=none args=v,v,v,v,v  # MMDL: grid position (m, x, y, z, dir)
  stub 0216d88c force ret=none args=v,out:12  # MMDL: position (m, &out)
  stub 021a3044 force ret=none args=v,in:12  # FIELD_PLAYER: position (pl, &pos)
  stub 021bb3b8 force ret=none args=v  # no-grid mapper: no current RAIL WORK (m)
  stub 0218e13c force ret=none args=v,v  # field camera (cam, v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218e160 force ret=none args=v  # field camera (cam)
  stub 0216d5e0 force ret=none args=v,v  # MMDL: facing direction (m, d)
  stub 0216d600 force ret=none args=v,v  # MMDL: draw direction (m, dir)
  stub 0219f0b0 force ret=none args=v,v,v  # MMDL billboard animation values (m, a, b)
  stub 021a3680 force ret=none args=v,in:8  # FIELD_PLAYER: rail location (pl, &loc)
  stub 021a36a4 force ret=u32 args=v  # FIELD_PLAYER -> rail work
  stub 021bb3ac force ret=none args=v,v  # no-grid mapper: current RAIL WORK (m, w)
  stub 0218f05c force ret=none args=v  # field camera (cam)
