# fndiff spec: field_player_grid.c (overlay 21 Thumb code 0x021A4140..0x021A5880, F1 row 21.23; F3a-spec): the grid player: pad -> move
# value by form -> MMDL acmd, footstep SE, ice / slide / forced-move tiles, slopes, the warp-ahead test.
# Run: tools/check-fn.sh black-field player_grid --per-function
# Grammar: svflow_specs/README.md (generic mode). Set `field` (main 10 11 18 20 21).
# F3a-spec: the F1-SPLIT rows of "F3a grid player" (21.20 fieldmap_ctrl_grid, 21.21 field_player, 21.22 field_player_core, 21.23
# field_player_grid, 21.37 fldeff_namipoke, 21.64 height_ex, 21.65 field_player_grid_event) as six areas: player (0x021A2D88..0x021A36E4),
# player_core (0x021A36E4..0x021A4140), player_grid (0x021A4140..0x021A5880), namipoke (0x021ACA38..0x021AD758), height_ex
# (0x021C0598..0x021C1024; 021C0598 sits before the F1 cut but only height_ex.c calls it) and player_event (0x021C1024..0x021C1A08).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry (the gaps are
# 2-byte padding and literal pools only): 330 entries, minus 5 false ones (021A3494 / 021A53AC / 021AD38C / 021C0BB8: words in main ARM code
# or data decode as bl; 021A3562: a literal of overlay 21 data), 3 veneers (`ldr r3,=T; bx r3`, callers stub T: 021A3810 / 021A41A8 ->
# Heap_Free 020307B0, 021A575C -> 021A550C), the 7 fldeff_encount.c heads 021AD758..021AD7A0 (create / delete pairs of the encounter
# effect table 0x021D903C, U2), and the 9 render-only functions of fldeff_namipoke.c (namipoke header) = 306 specced (player 72,
# player_core 64, player_grid 97, namipoke 18, height_ex 38, player_event 17). Ghidra misses 021A3E48, 021A3EE0, 021A408C, 021ACC3C,
# 021AD738, 021C0628, 021C06C0, 021C10B4, 021C1318, 021C161C (table targets / callers outside the field set) and lists switch cases as LAB_.
# Thin wrappers (`ldr r0,[r0,#k]; ldr r3,=T; bx r3`, `movs r1,#k; ldr r3,=T; bx r3`) are real functions and are specced.
# Closure stubs (as core4 / F4a / F6a): a block lists the stubs of its callees and of the callees of its in-area descendants and the
# shared fix lines, so an area can be ported in any order (checked with every function of the area treated as ported: no fault, stray
# write or unstubbed callee; `svflow.py AREA --all --strict-callees` and `--order-check` clean). Every out-of-area stub is `force`, also
# between the six areas (overlay addresses collide with battle PORTED entries; fldmmdl / fieldmap / eventdata / mmdl_move callees are
# ported or specced in other black-field areas). Indirect calls through ROM tables land on in-area stubs (plain) at the table targets;
# the table indices are bounded to the tables.
# Stub conventions: pointers to the caller's stack are `in:N` / `out:N` / `outm` (never `v`: no stack address reaches a hash); a stub
# called in a loop whose answer must change (polled callees, the OBJ code while an OBJ change is pending, attributes per footprint
# tile) is `seq`; `outv:N:V..` (F3a-spec grammar) answers out-parameters that are compared with a constant (020508EC's 0x3FFF).
# D3: the draw side (3D models, field-effect drawing, camera shake, SE) is only ever an address behind a stub.
# Arena (the `work` image): FIELDMAP work+0 (value only), FIELD_PLAYER work+0x100, PLAYER CORE work+0x200, PLAYER GRID work+0x400,
# the no-grid player work+0x600 (value only), the grid INPUT record work+0x800, the player MMDL work+0x900 (value only: MMDL fields are read
# through the force-stubbed fldmmdl getters), VecFx32 / out scratch work+0xA00.., MAP CTRL grid work work+0xB00, GMEVENT works work+0xC00
# (seq word work+0xBF0), HEIGHT_EX work+0x1000 (records work+0x1010), allocation answers / effect task works work+0x1800...
# The shared fix lines (every block of the player areas) keep FIELD_PLAYER +4 -> CORE, +8 -> GRID or 0, +0xC -> no-grid or 0, CORE +0x18
# (form-table row) 0..1, +0x1C -> MMDL, +0x30 (request step) 0..1, GRID +0xC -> CORE, and draw the state fields from their real ranges.
# FIELD_PLAYER (0x10, Heap_AllocDebug(heap, 0x10, 1, "field_player.c", 0x4C) in 021A2E50): +0 FIELDMAP, +4 -> CORE (021A36E4),
#   +8 -> GRID (021A4140, grid maps), +0xC -> the no-grid player (021A5880, F3b). Map kind = 02188CFC(fieldmap): 0 grid, 1 rail.
# MAP CTRL grid work (0x10, 021A2D88, the grid entry of the MAP CTRL table 0x021D88F8: +4 create 021A2D88, +8 frame 021A2DF0, +0xC
#   delete 021A2DE4, +0x10 021A2E34): +0 u16 heap, +4 FIELDMAP, +0xC u32 hold (nonzero: no pad input this frame; 021A2E44 sets it).
#   Frame: t = 020362DC() (pad trigger), h = 0203630C() (pad held), run allowed = event flag 0x963 (running shoes, 021A35BC) ->
#   021A35F0(player, t, h, flags bit 0) -> 021A41B0 (grid).
# PLAYER CORE (0x34, Heap_AllocDebug(heap, 0x34, 1, "field_player_core.c", 0x97) in 021A36E4): +4 GAMESYS, +8 FIELDMAP, +0xC PLAYER record
#   (GAMEDATA, 0x44: +0x1C saved OBJ code (0xFFFF none) 02012B30 / 02012B34, +0x40 move form 02012B28 / 02012B2C), +0x10 move state,
#   +0x14 move value (021A3964), +0x18 sex = form-table row, +0x1C the player MMDL (OBJ id 0xFF; created from the HEADER template ROM
#   0x021D89A0 when missing), +0x20 / +0x22 u16 last x / z direction of the pad (021A3C24), +0x24 request bits, +0x28 surf effect task
#   (021ACB38), +0x2C u16 param counter, +0x2E u16 OBJ-change counter, +0x30 s16 request step, +0x32 u16 request done.
# MOVE FORM (PLAYER record +0x40): 0 walk, 1 cycle, 2 surf, 3 surf variant (request 8, 021D05D8). FORM TABLE ROM 0x021D890C: 2 rows (sex) x
#   10 entries {u16 OBJ code, u16 form}: row 0 = (1,0) (2,1) (3,2) (3,3) (0x88,4) (0x8A,4) (0x8C,4) (0x9E,4) (0xA0,4) (0x9C,4), row 1 = codes
#   4 5 6 6 0x89 0x8B 0x8D 0x9F 0xA1 0x9D. 021A3190(sex, i) = code of entry i, 021A31A4(sex, form) = first code of form, 021A31D0(code) =
#   entry index (10 = none).
# REQUESTS (CORE +0x24 bit i = request i, lowest first, 021A3C94): procs ROM 0x021D89C4[i][step] (the dispatcher 021A3C6C runs procs while
#   one answers 1; every proc answers 0 and advances +0x30 / sets +0x32): 0 walk (021A3DA0: entry 0, form 0, drop the surf effect),
#   1 cycle (021A3E14, 021A3E48), 2 surf (021A3E54: entry 2, form 2, add the surf effect), 3 the saved form's code (021A3EE0, 021A3F10),
#   4 / 5 / 6 entries 4 / 5 / 6 (021A3F48 .. 021A3FB4), 7 entry 9 (021A3FC0, 021A3FDC), 8 surf variant (021A3FE8), 9 entry 8 (021A408C,
#   021A40A8). Callers: 021A2FD0 (bit 1 = walk), 021A4140 (form 2: bit 4, form 3: 0x100), 021C10B4 (bit 4 = surf).
# PAD -> DIRECTION (021A3BB4): x = 021A3C3C (0x20 left 2, 0x10 right 3), z = 021A3C54 (0x40 up 0, 0x80 down 1), 9 = none; both pressed: keep
#   the MMDL move direction if it is one of them and both are the remembered ones, else the newly pressed axis wins. Dirs 0 up, 1 down,
#   2 left, 3 right.
# PLAYER GRID (0x20, Heap_AllocDebug(heap, 0x20, 1, "field_player_grid.c", 0x17D) in 021A4140): +0 move state (0 stand, 1 walk / ride /
#   swim, 2 turn, 3 bump, 4, 5 slide, 6, 7 fall), +4 move value, +8 flags (1 forced move, 2, 4, 8 ice spin, 0x10 sliding, 0x20 running),
#   +0xC -> CORE, +0x10 FIELDMAP, +0x14 u16 / +0x16 u8 ice-spin counter (60 frames), +0x18 u16 footstep SE (0x546 = none) / +0x1A u16 its
#   frame counter, +0x1C u32 one-shot flag (021A5764 / 021A5768).
# INPUT (0x14, on the stack of 021A41B0): +0 pad trigger, +4 pad held (bit 1 = B: run), +8 u16 dir (9 none), +0xC flags (bit 0: running
#   allowed), +0x10 u32 (0; 1 = slow / no-run mode).
# MOVE VALUE (021A4444 by form: walk 021A44C8 (ice 021A46F0), cycle 021A48C8, surf 021A4B00) -> action 021A4CA8: walk form 0 none,
#   1 stand, 2 walk / run (acmd base 0xC, 0x10, 0x14 slow, 0x58 run, slopes 0xA7 / 0x4C / 0xA3), 3 turn (0x24), 4 bump (0x1C, SE 0x55A
#   unless a warp is ahead), 5 jump (0x38, SE 0x55E), 6 (+0 = 4), 7 slide (0x34), 8 (+0 = 6), 9 fall; cycle 0..5 (ride 0x14 / 0x10 / dust
#   0x9B), surf 0..4 (swim 0x14 / 0x10). Acmd = 02197E7C(dir, base) -> 0216D410(MMDL, acmd).
# FOOTSTEP SE (021A428C): flag tables 0x021D8A00 (mode 0) / 0x021D8A0C (mode 1) {u32 attribute-flag mask, u32 SE, s16 period walk / run},
#   then predicate tables 0x021D89F4 (empty) / 0x021D8A2C (10 entries) {predicate fn (effect_encount 021AB140 ..), u16 SE, s8 period walk /
#   run, u8 dir (4 = any) compared with the season}. Forced-move table 0x021D89EC [021A5458()] = 021A552C / 021A5538. Turn order on ice
#   ROM 0x021D8A24 = 3, 2, 0, 1.
# Coverage (plain): 2444/2445 instructions (021A4E4E, a `b` that no value of r4 reaches: the code is only 0xC / 0x10 / 0x58 when r7 is 1);
#   closure: 2427/2445 (hits that need two real callees to agree).
area player_grid
set field

# 021A4140(core, heap) -> grid: Heap_AllocDebug(heap, 0x20, 1, "field_player_grid.c", 0x17D); +0xC core, +0x10 =
# 021A395C(core) (fieldmap), +0x18 = 0x546 (no SE); 021A5354(grid) (flag 2); form 2 (surf): 021A3CB0(core, 4) (request surf),
# 021A3CE8(core); form 3: 021A3CB0(core, 0x100) (request 8), 021A3CE8(core)
fn 021a4140 ret=u32 inputs=100
  arg work 0x200
  arg u32 any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02030734 force ret=ptr:work:0x400:1 args=v,v,v,v,v  # Heap_AllocDebug: the GRID block
  stub 021a395c force ret=u32 args=v  # player_core
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3cb0 force ret=none args=v,v  # player_core
  stub 021a3ce8 force ret=none args=v  # player_core
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a5354 ret=none args=v  # in-area

# 021A41B0(grid, a, b, flags): frame: dir = 021A3AB8(core, b) (pad -> dir); 021A4254(grid, dir) == 1: 021A3CC0(core);
# 021A55A8(grid, dir, 0) == 0: INPUT {a, b, u16 dir, flags, 0} on the stack; v = 021A4444(grid, &in); 021A4224(grid, v, &in);
# 021A4CA8(grid, v, &in); 021A43D8(grid, v, dir)
fn 021a41b0 ret=none inputs=100
  arg work 0x400
  arg u32 alt mask 0xF3 || any
  arg u32 alt mask 0xF3 || any
  arg u32 alt mask 3 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0202e83c force ret=pick:0/1/2/3/4 args=v  # GAMESYS getter (season / weather)
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02163f10 force ret=none args=v  # mmdl_move: grid position refresh (m)
  stub 02164010 force ret=pick:0/1 args=v  # mmdl_move: height refresh (m)
  stub 021640b0 force ret=pick:0/1 args=v  # mmdl_move: attribute refresh (m)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d548 force ret=none args=v,v  # fldmmdl: MMDL +0 |= v
  stub 0216d550 force ret=none args=v,v  # fldmmdl: MMDL +0 &= ~v
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # fldmmdl: MMDL +4 &= ~v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL move dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # fldmmdl: MMDL +0x74 (attribute)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3920 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a3a20 force ret=pick:0/1 args=v  # player_core
  stub 021a3ab8 force ret=pick:0/1/2/3/9 args=v,v  # player_core
  stub 021a3ac4 force ret=none args=v,v  # player_core
  stub 021a3cc0 force ret=pick:0/1 args=v  # player_core
  stub 021a4224 ret=none args=v,v,in:0x14  # in-area
  stub 021a4254 ret=pick:0/1 args=v,v  # in-area
  stub 021a428c ret=pick:0/1 args=v,v,in:12,v,v  # in-area
  stub 021a43d8 ret=none args=v,v,v  # in-area
  stub 021a4444 ret=pick:0/1/2/3/4/5/6/7/8/9 args=v,in:0x14  # in-area
  stub 021a44c8 ret=pick:0/1/2/3/4/5/7/8/9 args=v,in:0x14  # in-area
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4664 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4698 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a46f0 ret=pick:0/1/2/3/6/7/8/9 args=v,in:0x14  # in-area
  stub 021a4768 ret=pick:0/2/3/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a47b4 ret=pick:0/2/4/7 args=v,v,in:0x14  # in-area
  stub 021a482c ret=pick:2/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a4890 ret=pick:8/9 args=v,v,in:0x14  # in-area
  stub 021a48a4 ret=pick:1/9 args=v,v,in:0x14  # in-area
  stub 021a48c8 ret=pick:0/1/2/3/4/5 args=v,in:0x14  # in-area
  stub 021a491c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a4994 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4a74 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4aa8 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b00 ret=pick:0/1/2/3/4 args=v,in:0x14  # in-area
  stub 021a4b54 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b98 ret=pick:0/2/4 args=v,v,in:0x14  # in-area
  stub 021a4c1c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4c50 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4ca8 ret=none args=v,v,in:0x14  # in-area
  stub 021a4cf4 ret=none args=v,v,in:0x14  # in-area
  stub 021a4d8c ret=none args=v,v,in:0x14  # in-area
  stub 021a4d90 ret=none args=v,v,in:0x14  # in-area
  stub 021a4dc0 ret=none args=v,v,in:0x14  # in-area
  stub 021a4e78 ret=none args=v,v,in:0x14  # in-area
  stub 021a4e9c ret=none args=v,v,in:0x14  # in-area
  stub 021a4ed8 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f04 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f0c ret=none args=v,v,in:0x14  # in-area
  stub 021a4f50 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f58 ret=none args=v,v,in:0x14  # in-area
  stub 021a4fa8 ret=none args=v,v,in:0x14  # in-area
  stub 021a5010 ret=none args=v,v,in:0x14  # in-area
  stub 021a5014 ret=none args=v,v,in:0x14  # in-area
  stub 021a5044 ret=none args=v,v,in:0x14  # in-area
  stub 021a50bc ret=none args=v,v,in:0x14  # in-area
  stub 021a50e0 ret=none args=v,v,in:0x14  # in-area
  stub 021a511c ret=none args=v,v,in:0x14  # in-area
  stub 021a5148 ret=none args=v,v,in:0x14  # in-area
  stub 021a51a4 ret=none args=v,v,in:0x14  # in-area
  stub 021a51a8 ret=none args=v,v,in:0x14  # in-area
  stub 021a51d8 ret=none args=v,v,in:0x14  # in-area
  stub 021a5204 ret=none args=v,v,in:0x14  # in-area
  stub 021a5228 ret=none args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5328 ret=none args=v  # in-area
  stub 021a5334 ret=none args=v  # in-area
  stub 021a5340 ret=pick:0/1 args=v  # in-area
  stub 021a5354 ret=none args=v  # in-area
  stub 021a5360 ret=none args=v  # in-area
  stub 021a536c ret=none args=v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a53bc ret=pick:0/1 args=v  # in-area
  stub 021a5458 ret=pick:0/1 args=v  # in-area
  stub 021a54b4 ret=none args=v  # in-area
  stub 021a550c ret=none args=v  # in-area
  stub 021a552c ret=pick:0 args=v,v  # in-area
  stub 021a5538 ret=pick:0/1 args=v,v  # in-area
  stub 021a55a8 ret=pick:0/1 args=v,v,v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a55fc ret=pick:0/1 args=v  # in-area
  stub 021a5610 ret=pick:0/1 args=v  # in-area
  stub 021a5630 ret=pick:0/1/2/3/9 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56c0 ret=pick:0/1 args=v  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a57c0 ret=pick:0/1/2 args=v,v  # in-area
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab31c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab348 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab390 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab3e0 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab44c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ac528 force ret=none args=v,v  # fldeff_kemuri: dust (m, ctrl)

# 021A4224(grid, v, &in): +4 = v; 021A3AC4(core, in +4) (remember the keys); 021A536C(grid) (clear flag 4); flag 0x20 off;
# v == 2: 021A5360 (clear flag 2); +0x1C = 0
fn 021a4224 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a3ac4 force ret=none args=v,v  # player_core
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5360 ret=none args=v  # in-area
  stub 021a536c ret=none args=v  # in-area

# 021A4254(grid, dir) -> 0216D3CC(m) == 1 (acmd settable): 1; dir != 9 and +0 == 3 and dir != display dir: 1; else 0
fn 021a4254 ret=u32 inputs=300
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  fix work 0x400 u32 alt const 3 || range 0 7
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A428C(grid, v, &pos, mode, [dir]) -> SE played: footstep SE: run = form 1 (2) or flag 0x20 (1); a = attribute at pos
# (02163E34); flags = 021AB0F8(a): table 0x021D8A00 / 0x021D8A0C (mode) {mask, SE, s16 period[2]} first match -> SE, period; still
# none: table 0x021D89F4 / 0x021D8A2C {predicate, u16 SE, s8 period[2], u8 dir (4 = any)} first predicate(a) with dir match;
# still 0x546: mode 1: +0x1A = 0; else (running): a new SE resets +0x1A, played every period frames (+0x18 / +0x1A); SE != 0x546:
# 020061E4(SE), 1
fn 021a428c ret=u32 inputs=400
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg work 0xA00
  arg u32 range 0 1
  arg u32 alt range 0 4 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x7FFFFF || any
  fix work 0xA04 u32 alt range 0 0x7FFFFF || any
  fix work 0xA08 u32 alt range 0 0x7FFFFF || any
  stub 021ab0f8 force ret=pick:0/0x20/0x22/1 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab31c force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab328 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab304 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab2f8 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab364 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab41c force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab390 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab3d4 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 021ab3e0 force ret=pick:0/0/0/1 args=v  # effect_encount: attribute predicate (rare true)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area

# 021A43D8(grid, v, dir): v != 2: +0x18 = 0x546, +0x1A = 0; else pos = MMDL position, season = 0202E83C(GAMESYS);
# 021A428C(grid, 2, &pos, 0, season) == 0: pos one step in dir (02164208(dir, &pos, 0x10000)), 021A428C(grid, 2, &pos, 1, season)
fn 021a43d8 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0202e83c force ret=pick:0/1/2/3/4 args=v  # GAMESYS getter (season / weather)
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a428c ret=pick:0/1 args=v,v,in:12,v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab31c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab390 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab3e0 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4444(grid, &in) -> move value by form: 0 walk 021A44C8, 1 cycle 021A48C8, 2 / 3 surf 021A4B00; else 0
fn 021a4444 ret=u32 inputs=100
  arg work 0x400
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02163f10 force ret=none args=v  # mmdl_move: grid position refresh (m)
  stub 02164010 force ret=pick:0/1 args=v  # mmdl_move: height refresh (m)
  stub 021640b0 force ret=pick:0/1 args=v  # mmdl_move: attribute refresh (m)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a44c8 ret=pick:0/1/2/3/4/5/7/8/9 args=v,in:0x14  # in-area
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4664 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4698 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a46f0 ret=pick:0/1/2/3/6/7/8/9 args=v,in:0x14  # in-area
  stub 021a4768 ret=pick:0/2/3/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a47b4 ret=pick:0/2/4/7 args=v,v,in:0x14  # in-area
  stub 021a482c ret=pick:2/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a4890 ret=pick:8/9 args=v,v,in:0x14  # in-area
  stub 021a48a4 ret=pick:1/9 args=v,v,in:0x14  # in-area
  stub 021a48c8 ret=pick:0/1/2/3/4/5 args=v,in:0x14  # in-area
  stub 021a491c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a4994 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4a74 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4aa8 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b00 ret=pick:0/1/2/3/4 args=v,in:0x14  # in-area
  stub 021a4b54 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b98 ret=pick:0/2/4 args=v,v,in:0x14  # in-area
  stub 021a4c1c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4c50 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a53bc ret=pick:0/1 args=v  # in-area
  stub 021a54b4 ret=none args=v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a55fc ret=pick:0/1 args=v  # in-area
  stub 021a5610 ret=pick:0/1 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56c0 ret=pick:0/1 args=v  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A448C(grid, dir) -> dir 9: 0; dir == display dir or +0 1 / 3: 1; else 2
fn 021a448c ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A44BC(grid) -> tail 021A3AA0(core) (acmd settable)
fn 021a44bc ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a3aa0 force ret=pick:0/1 args=v  # player_core

# 021A44C8(grid, &in) -> walk move value: 021A5610 (ice under the player) or 021A53BC (flag 0x10): 021A46F0(grid, in);
# else by +0: 0 021A4544, 1 021A4598, 2 021A4664, 3 021A4698 (grid, m, in); else 0
fn 021a44c8 ret=u32 inputs=100
  arg work 0x400
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02163f10 force ret=none args=v  # mmdl_move: grid position refresh (m)
  stub 02164010 force ret=pick:0/1 args=v  # mmdl_move: height refresh (m)
  stub 021640b0 force ret=pick:0/1 args=v  # mmdl_move: attribute refresh (m)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4664 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4698 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a46f0 ret=pick:0/1/2/3/6/7/8/9 args=v,in:0x14  # in-area
  stub 021a4768 ret=pick:0/2/3/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a47b4 ret=pick:0/2/4/7 args=v,v,in:0x14  # in-area
  stub 021a482c ret=pick:2/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a4890 ret=pick:8/9 args=v,v,in:0x14  # in-area
  stub 021a48a4 ret=pick:1/9 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a53bc ret=pick:0/1 args=v  # in-area
  stub 021a54b4 ret=none args=v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a55fc ret=pick:0/1 args=v  # in-area
  stub 021a5610 ret=pick:0/1 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56c0 ret=pick:0/1 args=v  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4544(grid, m, &in) -> 0216D3CC(m) == 1: dir 9: 1 (stop); dir != display dir and in +0x10 == 0: (021A4544 only: in +0xC bit 0
# clear or in +4 bit 1 clear:) 3 (turn); else 021A4598; else 0
fn 021a4544 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4598(grid, m, &in) -> walk step: acmd running: 0; dir 9: 021A4544; hit = 021A5264(grid, m, dir, &a); in +0x10 == 1
# and hit lacks bit 0x10: hit = 0; a == -1: 4; v = 021AB0F0(a), f = 021AB0F8(a); hit bit 2: ledge dir of v (021AB158 / 164 / 170 /
# 17C -> 0..3) == dir: 5 (jump); hit != 0: 4 (blocked); f bit 1 clear or in +0x10 == 1: 2 (walk); else 4
fn 021a4598 ret=u32 inputs=400
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021ab158 force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 021ab164 force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 021ab170 force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 021ab17c force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4664(grid, m, &in) -> 0216D3CC(m) == 0: 0; dir 9: 021A4544 / 021A4598(first); else (second)
fn 021a4664 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4698(grid, m, &in) -> dir != 9 and dir != display dir: 0216D4B4(m) (cancel the acmd), (second); else acmd running: 0;
# dir 9: (first); else (second)  [021A4544 / 021A4598]
fn 021a4698 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A46F0(grid, &in) -> ice / sliding move value by +0 (0..7): 021A4768, 021A47B4, 021A4664, 021A4698, 021A482C,
# 021A4890, 021A48A4 (grid, m, in); else 0
fn 021a46f0 ret=u32 inputs=100
  arg work 0x400
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02163f10 force ret=none args=v  # mmdl_move: grid position refresh (m)
  stub 02164010 force ret=pick:0/1 args=v  # mmdl_move: height refresh (m)
  stub 021640b0 force ret=pick:0/1 args=v  # mmdl_move: attribute refresh (m)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a4544 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4598 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4664 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4698 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4768 ret=pick:0/2/3/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a47b4 ret=pick:0/2/4/7 args=v,v,in:0x14  # in-area
  stub 021a482c ret=pick:2/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a4890 ret=pick:8/9 args=v,v,in:0x14  # in-area
  stub 021a48a4 ret=pick:1/9 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a54b4 ret=none args=v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56c0 ret=pick:0/1 args=v  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4768(grid, m, &in) -> acmd settable: dir 9: 021A482C; dir != display dir and in +0x10 == 0: 3; else 021A47B4; else 0
fn 021a4768 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a47b4 ret=pick:0/2/4/7 args=v,v,in:0x14  # in-area
  stub 021a482c ret=pick:2/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A47B4(grid, m, &in) -> slide step: acmd running: 0; dir 9: 021A4768; hit = 021A569C(grid, m, dir, &a); a != -1:
# hit bit 0x100 (ice ahead): hit & ~0x100 == 0: 2; else 021AB11C(a) == 0: 021A53A4(grid, 1), 7; else 4
fn 021a47b4 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a4768 ret=pick:0/2/3/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a482c ret=pick:2/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A482C(grid, m, &in) -> dir 9: flag 8 clear: 021A56F4 (start); +0x16 == 1 and 0219F7CC(m, +0x14): +0x16 = 0;
# +0x14 += 1; == 60: 021A5708, 021A53A4(grid, 1), 7; else 6; dir set: 021A5708(grid), 021A47B4(grid, m, in)
fn 021a482c ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 0219f7cc force ret=pick:0/1 args=v,v  # rail predicate (m, counter)
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a4768 ret=pick:0/2/3/4/6/7 args=v,v,in:0x14  # in-area
  stub 021a47b4 ret=pick:0/2/4/7 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a569c ret=pick:0/0x100/0x102/2/4 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a56f4 ret=none args=v  # in-area
  stub 021a5708 ret=none args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4890(grid, m, &in) -> 0216D3CC(m) == 1: 9; else 8
fn 021a4890 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)

# 021A48A4(grid, m, &in) -> 021A56C0(grid) == 1 (on the ground): 021A54B4(grid), 021A53A4(grid, 0), 1; else 9
fn 021a48a4 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02163f10 force ret=none args=v  # mmdl_move: grid position refresh (m)
  stub 02164010 force ret=pick:0/1 args=v  # mmdl_move: height refresh (m)
  stub 021640b0 force ret=pick:0/1 args=v  # mmdl_move: attribute refresh (m)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a54b4 ret=none args=v  # in-area
  stub 021a56c0 ret=pick:0/1 args=v  # in-area

# 021A48C8(grid, &in) -> cycle move value by +0: 021A491C, 021A4994, 021A4A74, 021A4AA8; else 0
fn 021a48c8 ret=u32 inputs=100
  arg work 0x400
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a491c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a4994 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a4a74 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4aa8 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A491C(grid, m, &in) -> 0216D3CC(m) == 1: dir 9: 1 (stop); dir != display dir and in +0x10 == 0: (021A4544 only: in +0xC bit 0
# clear or in +4 bit 1 clear:) 3 (turn); else 021A4994; else 0
fn 021a491c ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a4994 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4960(v) -> 021AB140 / 021AB404 / 021AB410 / 021AB328 (v) any nonzero
fn 021a4960 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4994(grid, m, &in) -> cycle step: as 021A4598 up to the jump; hit != 0: 4; f bit 1 clear and 021A4960(v) == 0: 2;
# else in +0x10 == 1: 2; else 4
fn 021a4994 ret=u32 inputs=400
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021ab158 force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 021ab164 force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 021ab170 force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 021ab17c force ret=pick:0/0/1 args=v  # effect_encount: ledge predicate (rare true)
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a491c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4A74(grid, m, &in) -> 0216D3CC(m) == 0: 0; dir 9: 021A491C / 021A4994(first); else (second)
fn 021a4a74 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a491c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a4994 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4AA8(grid, m, &in) -> dir != 9 and dir != display dir: 0216D4B4(m) (cancel the acmd), (second); else acmd running: 0;
# dir 9: (first); else (second)  [021A491C / 021A4994]
fn 021a4aa8 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a491c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4960 ret=pick:0/1 args=v  # in-area
  stub 021a4994 ret=pick:0/2/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab140 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab158 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab164 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab170 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab17c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4B00(grid, &in) -> surf move value by +0: 021A4B54, 021A4B98, 021A4C1C, 021A4C50; else 0
fn 021a4b00 ret=u32 inputs=100
  arg work 0x400
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a4b54 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b98 ret=pick:0/2/4 args=v,v,in:0x14  # in-area
  stub 021a4c1c ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4c50 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4B54(grid, m, &in) -> 0216D3CC(m) == 1: dir 9: 1 (stop); dir != display dir and in +0x10 == 0: (021A4544 only: in +0xC bit 0
# clear or in +4 bit 1 clear:) 3 (turn); else 021A4B98; else 0
fn 021a4b54 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4b98 ret=pick:0/2/4 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4B98(grid, m, &in) -> surf step: acmd running: 0; dir 9: 021A4B54; hit = 021A5264; in +0x10 == 1 and no bit 0x10:
# hit = 0; a == -1 or hit != 0: 4; f bit 1 set (water): 2; in +0x10 == 1: 2; else 4
fn 021a4b98 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4b54 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4C1C(grid, m, &in) -> 0216D3CC(m) == 0: 0; dir 9: 021A4B54 / 021A4B98(first); else (second)
fn 021a4c1c ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4b54 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b98 ret=pick:0/2/4 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4C50(grid, m, &in) -> dir != 9 and dir != display dir: 0216D4B4(m) (cancel the acmd), (second); else acmd running: 0;
# dir 9: (first); else (second)  [021A4B54 / 021A4B98]
fn 021a4c50 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a4b54 ret=pick:0/1/2/3/4/5 args=v,v,in:0x14  # in-area
  stub 021a4b98 ret=pick:0/2/4 args=v,v,in:0x14  # in-area
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=pick:0/1/2/3/0x20/0x22 args=v  # attribute flags of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4CA8(grid, v, &in): act on move value v by form: 0 021A4CF4, 1 021A4FA8, 2 / 3 021A5148
fn 021a4ca8 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # fldmmdl: MMDL +4 &= ~v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3920 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3a20 force ret=pick:0/1 args=v  # player_core
  stub 021a4cf4 ret=none args=v,v,in:0x14  # in-area
  stub 021a4d8c ret=none args=v,v,in:0x14  # in-area
  stub 021a4d90 ret=none args=v,v,in:0x14  # in-area
  stub 021a4dc0 ret=none args=v,v,in:0x14  # in-area
  stub 021a4e78 ret=none args=v,v,in:0x14  # in-area
  stub 021a4e9c ret=none args=v,v,in:0x14  # in-area
  stub 021a4ed8 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f04 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f0c ret=none args=v,v,in:0x14  # in-area
  stub 021a4f50 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f58 ret=none args=v,v,in:0x14  # in-area
  stub 021a4fa8 ret=none args=v,v,in:0x14  # in-area
  stub 021a5010 ret=none args=v,v,in:0x14  # in-area
  stub 021a5014 ret=none args=v,v,in:0x14  # in-area
  stub 021a5044 ret=none args=v,v,in:0x14  # in-area
  stub 021a50bc ret=none args=v,v,in:0x14  # in-area
  stub 021a50e0 ret=none args=v,v,in:0x14  # in-area
  stub 021a511c ret=none args=v,v,in:0x14  # in-area
  stub 021a5148 ret=none args=v,v,in:0x14  # in-area
  stub 021a51a4 ret=none args=v,v,in:0x14  # in-area
  stub 021a51a8 ret=none args=v,v,in:0x14  # in-area
  stub 021a51d8 ret=none args=v,v,in:0x14  # in-area
  stub 021a5204 ret=none args=v,v,in:0x14  # in-area
  stub 021a5228 ret=none args=v,v,in:0x14  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a5630 ret=pick:0/1/2/3/9 args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a57c0 ret=pick:0/1/2 args=v,v  # in-area
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab44c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ac528 force ret=none args=v,v  # fldeff_kemuri: dust (m, ctrl)

# 021A4CF4(grid, v, &in): walk form: v 0..9 -> 021A4D8C (none), 4D90 (stop), 4DC0 (walk), 4E78 (turn), 4E9C (bump), 4ED8
# (jump), 4F04, 4F0C (slide), 4F50, 4F58 (fall) (grid, m, in)
fn 021a4cf4 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # fldmmdl: MMDL +4 &= ~v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3920 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a3a20 force ret=pick:0/1 args=v  # player_core
  stub 021a4d8c ret=none args=v,v,in:0x14  # in-area
  stub 021a4d90 ret=none args=v,v,in:0x14  # in-area
  stub 021a4dc0 ret=none args=v,v,in:0x14  # in-area
  stub 021a4e78 ret=none args=v,v,in:0x14  # in-area
  stub 021a4e9c ret=none args=v,v,in:0x14  # in-area
  stub 021a4ed8 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f04 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f0c ret=none args=v,v,in:0x14  # in-area
  stub 021a4f50 ret=none args=v,v,in:0x14  # in-area
  stub 021a4f58 ret=none args=v,v,in:0x14  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a5630 ret=pick:0/1/2/3/9 args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a57c0 ret=pick:0/1/2 args=v,v  # in-area
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab44c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4D8C(grid, m, &in): nothing
fn 021a4d8c ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A4D90(grid, m, &in): dir = in dir (9: display dir); 0216D410(m, 02197E7C(dir, 0 (stand))); +0 = 0; 021A3964(core, 0)
fn 021a4d90 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A4DC0(grid, m, &in): walk: in +0x10 == 1: code 0x14 (slow walk), no run; else in +0xC bit 0 and in +4 bit 1 (B held):
# run when the OBJ code has a form (021A3A20): code 0x58 + flag 0x20, unless the attribute under the player (02163E34) is 021AB44C:
# code 0xC, flag 0x20 off; not runnable: 0x10; else 0xC; 021A57C0(grid, dir) == 1 (slope down): 0xC -> 0xA7, 0x10 -> 0x4C, 0x58 -> 0xA3;
# 0216D410(m, 02197E7C(dir, code)); +0 = 1; 021A3964(core, 1)
fn 021a4dc0 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a3a20 force ret=pick:0/1 args=v  # player_core
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a57c0 ret=pick:0/1/2 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab44c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4E78(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x24)); +0 = 2; 021A3964(core, 2)
fn 021a4e78 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A4E9C(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x1C)); +0 = 3; 021A3964(core, 0); 021A5810(grid, dir) == 0 (no warp ahead): SE 0x55A (bump)
fn 021a4e9c ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area

# 021A4ED8(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x38)); +0 = 1; 021A3964(core, 1); SE 0x55E
fn 021a4ed8 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A4F04(grid, m, &in): +0 = 4
fn 021a4f04 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A4F0C(grid, m, &in): slide: dir = in dir (9: 021A5630(grid), a free direction); 0216D410(m, 02197E7C(dir, 0x34));
# 0216DB4C(m, 1), 0216DCE8(m, 1), MMDL +4 |= 0x8000; +0 = 5
fn 021a4f0c ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3920 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a5630 ret=pick:0/1/2/3/9 args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A4F50(grid, m, &in): +0 = 6
fn 021a4f50 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A4F58(grid, m, &in): fall: pos = MMDL position; 02163E58(m, &pos, &h); pos.y > h: y -= 0x8000, below h: y = h,
# MMDL +4 &= ~0x4000, +0x1C = 1; position = pos; +0 = 7
fn 021a4f58 ret=none inputs=300
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d88c force ret=none args=v,outm:12:0xFF/0xFF/0xFF/0x7F/0/0x80/3/0/0xFF/0xFF/0xFF/0x7F  # fldmmdl: MMDL position (y small)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 0216d570 force ret=none args=v,v  # fldmmdl: MMDL +4 &= ~v
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A4FA8(grid, v, &in): cycle form: v 0..5 -> 021A5010 (none), 5014 (stand), 5044 (ride), 50BC (turn), 50E0 (bump), 511C
# (jump)
fn 021a4fa8 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5010 ret=none args=v,v,in:0x14  # in-area
  stub 021a5014 ret=none args=v,v,in:0x14  # in-area
  stub 021a5044 ret=none args=v,v,in:0x14  # in-area
  stub 021a50bc ret=none args=v,v,in:0x14  # in-area
  stub 021a50e0 ret=none args=v,v,in:0x14  # in-area
  stub 021a511c ret=none args=v,v,in:0x14  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a57c0 ret=pick:0/1/2 args=v,v  # in-area
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab41c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ac528 force ret=none args=v,v  # fldeff_kemuri: dust (m, ctrl)

# 021A5010(grid, m, &in): nothing
fn 021a5010 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A5014(grid, m, &in): dir = in dir (9: display dir); 0216D410(m, 02197E7C(dir, 0 (stand))); +0 = 0; 021A3964(core, 0)
fn 021a5014 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A5044(grid, m, &in): cycle: in +0x10 == 1: 0x14, no check; else 0x14, and the attribute in front (021A5788(grid, 9,
# &a)) 021AB41C (dust): code 0x9B, 021AC528(m, effect control); 021A57C0(grid, dir) == 1 and code 0x14: 0x10; 0216D410(m,
# 02197E7C(dir, code)); +0 = 1; 021A3964(core, 1)
fn 021a5044 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a57c0 ret=pick:0/1/2 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab41c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ac528 force ret=none args=v,v  # fldeff_kemuri: dust (m, ctrl)

# 021A50BC(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x24)); +0 = 2; 021A3964(core, 2)
fn 021a50bc ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A50E0(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x1C)); +0 = 3; 021A3964(core, 0); 021A5810(grid, dir) == 0: SE 0x55A
fn 021a50e0 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area

# 021A511C(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x38)); +0 = 1; 021A3964(core, 1); SE 0x55E
fn 021a511c ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A5148(grid, v, &in): surf form: v 0..4 -> 021A51A4 (none), 51A8 (stand), 51D8 (swim), 5204 (turn), 5228 (bump)
fn 021a5148 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 9 || any
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a51a4 ret=none args=v,v,in:0x14  # in-area
  stub 021a51a8 ret=none args=v,v,in:0x14  # in-area
  stub 021a51d8 ret=none args=v,v,in:0x14  # in-area
  stub 021a5204 ret=none args=v,v,in:0x14  # in-area
  stub 021a5228 ret=none args=v,v,in:0x14  # in-area
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area

# 021A51A4(grid, m, &in): nothing
fn 021a51a4 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A51A8(grid, m, &in): dir = in dir (9: display dir); 0216D410(m, 02197E7C(dir, 0 (stand))); +0 = 0; 021A3964(core, 0)
fn 021a51a8 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A51D8(grid, m, &in): code = in +0x10 == 1 ? 0x14 : 0x10; 0216D410(m, 02197E7C(dir, code)); +0 = 1; 021A3964(core, 1)
fn 021a51d8 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A5204(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x24)); +0 = 2; 021A3964(core, 2)
fn 021a5204 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A5228(grid, m, &in): 0216D410(m, 02197E7C(in dir, 0x1C)); +0 = 3; 021A3964(core, 0); 021A5810(grid, dir) == 0: SE 0x55A
fn 021a5228 ret=none inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg work 0x800
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5810 ret=pick:0/1 args=v,v  # in-area

# 021A5264(grid, m, dir, &a) -> hit bits: x = gx + dx(dir), z = gz + dz(dir); hit = 02163994(m, x, gy, z, dir); pos = MMDL
# position one step in dir; 02163E34(m, &pos, a) == 0: *a = -1; hit bit 1 and 021AB2DC(021AB0F0(*a)) (bridge?) and 0216E5B0(MMDLSYS,
# &pos) == 1: hit &= ~2
fn 021a5264 ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg u32 alt range 0 3 || const 9 || any
  arg u32 addrof work 0xA30
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A5304(grid, bits): +8 |= bits
fn 021a5304 ret=none inputs=100
  arg work 0x400
  arg u32 alt mask 0x3F || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A530C(grid, bits): +8 &= ~bits
fn 021a530c ret=none inputs=100
  arg work 0x400
  arg u32 alt mask 0x3F || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A5318(grid, bits) -> +8 & bits != 0
fn 021a5318 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt mask 0x3F || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A5328(grid): tail 021A5304(grid, 1)
fn 021a5328 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5304 ret=none args=v,v  # in-area

# 021A5334(grid): tail 021A530C(grid, 1)
fn 021a5334 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a530c ret=none args=v,v  # in-area

# 021A5340(grid) -> flag 1 set (021A5318(grid, 1) != 0)
fn 021a5340 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area

# 021A5354(grid): tail 021A5304(grid, 2)
fn 021a5354 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5304 ret=none args=v,v  # in-area

# 021A5360(grid): tail 021A530C(grid, 2)
fn 021a5360 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a530c ret=none args=v,v  # in-area

# 021A536C(grid): tail 021A530C(grid, 4)
fn 021a536c ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a530c ret=none args=v,v  # in-area

# 021A5378(grid, on): flag 8 = on == 1
fn 021a5378 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 1 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area

# 021A5390(grid) -> flag 8 set
fn 021a5390 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area

# 021A53A4(grid, on): flag 0x10 = on == 1
fn 021a53a4 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 1 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area

# 021A53BC(grid) -> flag 0x10 set
fn 021a53bc ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area

# 021A53D0(grid): +0 == 3 (turning): d = display dir; 0216D4B4(m) (cancel), 0216D5E0(m, d), 0216D674(m, 0);
# 021A3964(core, 0)
fn 021a53d0 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5e0 force ret=none args=v,v  # fldmmdl: MMDL set display dir (m, dir)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d674 force ret=none args=v,v  # fldmmdl: MMDL +0x2A = v
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core

# 021A5408(grid) -> stop by +0: 0: 0; 1 / 2 / 3: cancel the acmd keeping the display dir (as 021A53D0), 1; else 1
fn 021a5408 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d4b4 force ret=none args=v  # fldmmdl: MMDL acmd clear (m)
  stub 0216d5e0 force ret=none args=v,v  # fldmmdl: MMDL set display dir (m, dir)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d674 force ret=none args=v,v  # fldmmdl: MMDL +0x2A = v
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a572c ret=ptr:work:0x900:1 args=v  # in-area

# 021A5458(grid) -> flag 2 clear and 021AB348(021AB0F0(MMDL attribute)) == 1 (a forced-move tile under the player)
fn 021a5458 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # fldmmdl: MMDL +0x74 (attribute)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab348 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A548C(grid) -> 021A5458(grid) != 0
fn 021a548c ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # fldmmdl: MMDL +0x74 (attribute)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5458 ret=pick:0/1 args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab348 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A54A0(grid) -> 021A53BC(grid) == 1 (sliding)
fn 021a54a0 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a53bc ret=pick:0/1 args=v  # in-area

# 021A54B4(grid): stop sliding: 0216DB4C(m, 0), 0216DCE8(m, 0), MMDL +4 |= 0x140; 02164010(m); y = pos.y >> 16 as tile;
# 0216D85C(m, y); 021640B0(m); 02163F10(m)
fn 021a54b4 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163f10 force ret=none args=v  # mmdl_move: grid position refresh (m)
  stub 02164010 force ret=pick:0/1 args=v  # mmdl_move: height refresh (m)
  stub 021640b0 force ret=pick:0/1 args=v  # mmdl_move: attribute refresh (m)
  stub 0216d568 force ret=none args=v,v  # fldmmdl: MMDL +4 |= v
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216dce8 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A550C(grid): flag 1 set: MMDL +0 &= ~0x18 (0216D550); clear flag 1
fn 021a550c ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d550 force ret=none args=v,v  # fldmmdl: MMDL +0 &= ~v
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5334 ret=none args=v  # in-area
  stub 021a5340 ret=pick:0/1 args=v  # in-area

# 021A552C(grid, dir) -> 0: 021A550C(grid) (table 0x021D89EC entry 0)
fn 021a552c ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 0216d550 force ret=none args=v,v  # fldmmdl: MMDL +0 &= ~v
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5334 ret=none args=v  # in-area
  stub 021a5340 ret=pick:0/1 args=v  # in-area
  stub 021a550c ret=none args=v  # in-area

# 021A5538(grid, dir) -> forced move (table entry 1): d = MMDL move dir; 021A5264(grid, m, d, &a) != 0: 021A550C,
# 021A5354 (flag 2), 021A3964(core, 0), 0; else flag 1, 0216D410(m, 02197E7C(d, 0x10)), MMDL +0 |= 0x18, +0 = 1, 021A3964(core, 1), 1
fn 021a5538 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # fldmmdl: MMDL +0 |= v
  stub 0216d550 force ret=none args=v,v  # fldmmdl: MMDL +0 &= ~v
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL move dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5328 ret=none args=v  # in-area
  stub 021a5334 ret=none args=v  # in-area
  stub 021a5340 ret=pick:0/1 args=v  # in-area
  stub 021a5354 ret=none args=v  # in-area
  stub 021a550c ret=none args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A55A8(grid, dir, skip) -> skip == 0 and ROM 0x021D89EC[021A5458(grid)](grid, dir) == 1: 1; else 021A5334(grid), 0
fn 021a55a8 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  arg u32 alt const 0 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d548 force ret=none args=v,v  # fldmmdl: MMDL +0 |= v
  stub 0216d550 force ret=none args=v,v  # fldmmdl: MMDL +0 &= ~v
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL move dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # fldmmdl: MMDL +0x74 (attribute)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3964 force ret=none args=v,v  # player_core
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5328 ret=none args=v  # in-area
  stub 021a5334 ret=none args=v  # in-area
  stub 021a5340 ret=pick:0/1 args=v  # in-area
  stub 021a5354 ret=none args=v  # in-area
  stub 021a5458 ret=pick:0/1 args=v  # in-area
  stub 021a550c ret=none args=v  # in-area
  stub 021a552c ret=pick:0 args=v,v  # in-area
  stub 021a5538 ret=pick:0/1 args=v,v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab348 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A55DC(v) -> 021AB404(v) == 1 or 021AB410(v) == 1 (ice)
fn 021a55dc ret=u32 inputs=100
  nowork
  arg u32 any
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A55FC(v) -> 021AB404(v) == 1
fn 021a55fc ret=u32 inputs=100
  nowork
  arg u32 any
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A5610(grid) -> 021A55FC(021AB0F0(a)) == 1 with a = attribute under the player (021A5788(grid, 9, &a))
fn 021a5610 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a55fc ret=pick:0/1 args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A5630(grid) -> dir: d = display dir; up to 4 times: 021A5788(grid, d, &a) == 1 and not ice (021A55DC) and
# 021AB11C(a) == 0: d; d = ROM 0x021D8A24[d] (3, 2, 0, 1: turn); 9
fn 021a5630 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3920 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021a5788 ret=pick:0/1 args=v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A569C(grid, m, dir, &a) -> 021A5264(..) | (ice at *a: 0x100)
fn 021a569c ret=u32 inputs=100
  arg work 0x400
  arg u32 addrof work 0x900
  arg u32 alt range 0 3 || const 9 || any
  arg u32 addrof work 0xA30
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163994 force ret=pick:0/1/2/4/8/0x10/0x12/0x20 args=v,v,v,v,v  # mmdl_move: step hit check (m, x, y, z, dir): hit bits
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e5b0 force ret=pick:0/1 args=v,in:12  # fldmmdl: MMDL at a position (sys, &pos)
  stub 021a5264 ret=pick:0/1/2/4/0x10/0x12/0x14 args=v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021a55dc ret=pick:0/1 args=v  # in-area
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab404 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab410 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A56C0(grid) -> pos.y <= height at pos (02163E58)
fn 021a56c0 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A56F4(grid): 021A5378(grid, 1); 021A39AC(core, 7) (form entry 7)
fn 021a56f4 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a39ac force ret=none args=v,v  # player_core
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area

# 021A5708(grid): +0x14 = 0, +0x16 = 0; 021A53A4(grid, 0); 021A5378(grid, 0); 021A3998(core)
fn 021a5708 ret=none inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area

# 021A5728(grid) -> +0xC (core)
fn 021a5728 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A572C(grid) -> tail 021A3960(+0xC) (the MMDL)
fn 021a572c ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A5738(grid, keep): flag 8: n = +0x14; 021A5708(grid); keep == 1: +0x14 = n, +0x16 = 1
fn 021a5738 ret=none inputs=100
  arg work 0x400
  arg u32 range 0 1
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 021a3998 force ret=none args=v  # player_core
  stub 021a5304 ret=none args=v,v  # in-area
  stub 021a530c ret=none args=v,v  # in-area
  stub 021a5318 ret=pick:0/1 args=v,v  # in-area
  stub 021a5378 ret=none args=v,v  # in-area
  stub 021a5390 ret=pick:0/1 args=v  # in-area
  stub 021a53a4 ret=none args=v,v  # in-area
  stub 021a5708 ret=none args=v  # in-area

# 021A5764(grid) -> +0x1C
fn 021a5764 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A5768(grid, on): +0x1C = on == 1
fn 021a5768 ret=none inputs=100
  arg work 0x400
  arg u32 alt range 0 1 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A5778(grid) -> +0 == 3
fn 021a5778 ret=u32 inputs=100
  arg work 0x400
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any

# 021A5788(grid, dir, &a) -> pos = MMDL position, dir != 9: one step in dir; 02163E34(m, &pos, a)
fn 021a5788 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  arg u32 addrof work 0xA30
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e34 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # mmdl_move: attribute at a position (m, &pos, &attr)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A57C0(grid, dir) -> slope: pos one step in dir; 02163E58(m, &pos, &h) == 1: pos.y < h: 1 (up), > h: 2; else 0
fn 021a57c0 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A5810(grid, dir) -> warp ahead: ev = 02012844(GAMEDATA); pos one step in dir; i = 02162564(ev, &pos); 0xFFFF: 0;
# else 021626B8(021625CC(ev, i), dir) != 0
fn 021a5810 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  fix work 0x104 u32 addrof work 0x200
  fix work 0x108 u32 alt addrof work 0x400 || const 0
  fix work 0x10C u32 alt addrof work 0x600 || const 0
  fix work 0x210 u32 alt range 0 3 || any
  fix work 0x214 u32 alt range 0 2 || any
  fix work 0x218 u32 range 0 1
  fix work 0x21C u32 addrof work 0x900
  fix work 0x220 u16 alt range 0 3 || const 9
  fix work 0x222 u16 alt range 0 3 || const 9
  fix work 0x224 u32 alt mask 0x3FF || const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 0x100 0x200
  fix work 0x228 u32 alt const 0 || any
  fix work 0x22C u16 range 0 2
  fix work 0x230 s16 range 0 1
  fix work 0x232 u16 range 0 1
  fix work 0x400 u32 alt range 0 7 || any
  fix work 0x404 u32 alt range 0 9 || any
  fix work 0x408 u32 alt mask 0x3F || any
  fix work 0x40C u32 addrof work 0x200
  fix work 0x414 u16 alt near 59 2 || range 0 70
  fix work 0x416 u8 range 0 1
  fix work 0x418 u16 alt const 0x546 || oneof 0x679 0x67a 0x67c || any
  fix work 0x41A u16 range 0 4
  fix work 0x41C u32 range 0 1
  fix work 0x800 u32 alt mask 0xF3 || any
  fix work 0x804 u32 alt mask 0xF3 || any
  fix work 0x808 u16 alt range 0 3 || const 9
  fix work 0x80C u32 alt mask 3 || any
  fix work 0x810 u32 alt range 0 1 || any
  fix work 0xB0C u32 alt const 0 || any
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 02162564 force ret=pick:0/1/0xFFFF args=v,in:12  # eventdata: warp index at a position (sys, &pos)
  stub 021625cc force ret=u32 args=v,v  # eventdata: warp record (sys, idx)
  stub 021626b8 force ret=pick:0/1 args=v,v  # eventdata: warp exits in dir (warp, dir)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

