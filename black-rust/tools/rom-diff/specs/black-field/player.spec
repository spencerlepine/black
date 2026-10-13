# fndiff spec: fieldmap_ctrl_grid.c + field_player.c (overlay 21 Thumb code 0x021A2D88..0x021A36E4, F1 rows 21.20 / 21.21; F3a-spec):
# the grid MAP CTRL callbacks and FIELD_PLAYER, the front object of the player (grid / no-grid dispatch over the core).
# Run: tools/check-fn.sh black-field player --per-function
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
# Coverage (plain): 917/917 instructions; closure: 917/917.
area player
set field

# 021A2D88(fieldmap, _, dir): grid MAP CTRL create: w = Heap_AllocDebug(02188C84(fm), 0x10, 1, "fieldmap_ctrl_grid.c", 0x68);
# w +0 = heap, +4 = fm; p = 02188CB4(fm) (FIELD_PLAYER); 0216D5E0(021A3074(p), dir); 021A35E0(p, heap) (grid player);
# 02188D40(fm, 0216D888(MMDL)) (position source = the MMDL position); 02188CAC(fm, w)
fn 021a2d88 ret=none inputs=100
  arg u32 any
  arg u32 any
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
  stub 02030734 force ret=ptr:work:0xB00:1 args=v,v,v,v,v  # Heap_AllocDebug: the MAP CTRL work
  stub 0216d5e0 force ret=none args=v,v  # fldmmdl: MMDL set display dir (m, dir)
  stub 0216d888 force ret=u32 args=v  # fldmmdl: MMDL +0x44 (address of the position VecFx32)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 02188cac force ret=none args=v,v  # fieldmap +0x10C = v
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 02188d40 force ret=none args=v,v  # fieldmap +0xF0 = position source
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a35e0 ret=none args=v,v  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a4140 force ret=ptr:work:0x400:1 args=v,v  # player_grid

# 021A2DE4(fieldmap): Heap_Free(02188CA4(fm)) (MAP CTRL delete)
fn 021a2de4 ret=none inputs=100
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
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02188ca4 force ret=ptr:work:0xB00:1 args=v  # fieldmap +0x10C (MAP CTRL work)

# 021A2DF0(fieldmap): MAP CTRL frame: w = 02188CA4(fm); p = 02188CB4(fm); w +0xC == 0 (not held): t = 020362DC(), h = 0203630C()
# (pad); 021A35F0(p, t, h, 021A35BC(p) == 1 (flag 0x963: bit 0)) (grid player frame)
fn 021a2df0 ret=none inputs=100
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
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 020362dc force ret=pick:0/0x10/0x20/0x40/0x80/0x30/0xC0/1/2 args=  # key input (pad trigger)
  stub 0203630c force ret=pick:0/0x10/0x20/0x40/0x80/0x30/0xC0/2/0x402 args=  # key input (pad held)
  stub 02188ca4 force ret=ptr:work:0xB00:1 args=v  # fieldmap +0x10C (MAP CTRL work)
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 021a30bc ret=u32 args=v  # in-area
  stub 021a35bc ret=pick:0/1 args=v  # in-area
  stub 021a35f0 ret=none args=v,v,v,v  # in-area
  stub 021a3990 force ret=u32 args=v  # player_core
  stub 021a41b0 force ret=none args=v,v,v,v  # player_grid

# 021A2E34(fieldmap) -> 0216D888(021A3074(02188CB4(fm))) (address of the player MMDL position)
fn 021a2e34 ret=u32 inputs=100
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
  stub 0216d888 force ret=u32 args=v  # fldmmdl: MMDL +0x44 (address of the position VecFx32)
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A2E44(fieldmap, v): 02188CA4(fm) +0xC = v (hold the player; no caller in the field set)
fn 021a2e44 ret=none inputs=100
  arg u32 any
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
  stub 02188ca4 force ret=ptr:work:0xB00:1 args=v  # fieldmap +0x10C (MAP CTRL work)

# 021A2E50(x, fieldmap, &pos, sex, [heap]) -> player: Heap_AllocDebug(heap, 0x10, 1, "field_player.c", 0x4C); +0 fieldmap;
# +4 = 021A36E4(x, fieldmap, &pos, sex, heap) (the core)
fn 021a2e50 ret=u32 inputs=100
  arg u32 any
  arg u32 any
  arg work 0xA00
  arg u32 range 0 1
  arg u16 any
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
  stub 02030734 force ret=ptr:work:0x100:1 args=v,v,v,v,v  # Heap_AllocDebug: the FIELD_PLAYER block
  stub 021a36e4 force ret=ptr:work:0x200:1 args=v,v,in:12,v,v  # player_core

# 021A2E90(player): +8 != 0: 021A41A8 (Heap_Free veneer) (grid); +0xC != 0: 021A5920 (no-grid delete); 021A3810 (Heap_Free)
# (core); Heap_Free(player)
fn 021a2e90 ret=none inputs=100
  arg work 0x100
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
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021a5920 force ret=none args=v  # field_player_nogrid: delete

# 021A2EB8(player): save the player into its PLAYER record r = 021A3994(core): position (0216D88C) -> 02012A90(r, &pos),
# dir 0216D5F8 -> 02012AD4; map kind 0 (02188CFC): 02012ABC(r, 0); else rail location (0219E2F4) -> 02012AA4(r, &loc), 02012ABC(r, 1)
fn 021a2eb8 ret=none inputs=100
  arg work 0x100
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
  stub 02012a90 force ret=none args=v,in:12  # PLAYER record: position = *v
  stub 02012aa4 force ret=none args=v,in:8  # PLAYER record: rail location = *v
  stub 02012abc force ret=none args=v,v  # PLAYER record: position kind (rec, 0 grid / 1 rail)
  stub 02012ad4 force ret=none args=v,v  # PLAYER record: direction
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # rail: MMDL rail location -> out
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3994 force ret=u32 args=v  # player_core

# 021A2F20(player, a, keys, flags) -> grid map: 021C0CA4(+8, a, keys, flags) (step events), else 0
fn 021a2f20 ret=u32 inputs=100
  arg work 0x100
  arg u32 any
  arg u32 alt mask 0xF0 || any
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021c0ca4 force ret=pick:0/0x02E00C00 args=v,v,v,v  # height_ex

# 021A2F48(player, ..): tail 021A3CB0(+4, ..) (set requests (core, bits))
fn 021a2f48 ret=none inputs=100
  arg work 0x100
  arg u32 alt mask 0x3FF || any
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
  stub 021a3cb0 force ret=none args=v,v  # player_core

# 021A2F54(player, ..): tail 021A3CC0(+4, ..) (requests done? (core))
fn 021a2f54 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3cc0 force ret=pick:0/1 args=v  # player_core

# 021A2F60(player, ..): tail 021A3CE8(+4, ..) (run requests (core))
fn 021a2f60 ret=none inputs=100
  arg work 0x100
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
  stub 021a3ce8 force ret=none args=v  # player_core

# 021A2F6C(player): grid map: 021A53D0(+8) (end a turn); else 021A5A34(+0xC)
fn 021a2f6c ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a53d0 force ret=none args=v  # player_grid
  stub 021a5a34 force ret=none args=v  # field_player_nogrid (nogrid)

# 021A2F8C(player, ..): tail 021A3AD0(+4, ..) (set the surf effect task (core, v))
fn 021a2f8c ret=none inputs=100
  arg work 0x100
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
  stub 021a3ad0 force ret=none args=v,v  # player_core

# 021A2F98(player, ..): tail 021A3AD4(+4, ..) (surf effect task (core))
fn 021a2f98 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3ad4 force ret=nullable:2:u32 args=v  # player_core

# 021A2FA4(player, ..): tail 021A3AB8(+4, ..) (dir of keys (core, keys))
fn 021a2fa4 ret=u32 inputs=100
  arg work 0x100
  arg u32 alt mask 0xF0 || any
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
  stub 021a3ab8 force ret=pick:0/1/2/3/9 args=v,v  # player_core

# 021A2FB0(player): grid map: 021A5778(+8) (move state 3?); else 021A59DC(+0xC)
fn 021a2fb0 ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a5778 force ret=pick:0/1 args=v  # player_grid
  stub 021a59dc force ret=pick:0/1 args=v  # field_player_nogrid predicate

# 021A2FD0(player): 021A3CB0(+4, 1) (request walk); 021A3CE8(+4) (run it)
fn 021a2fd0 ret=none inputs=100
  arg work 0x100
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
  stub 021a3cb0 force ret=none args=v,v  # player_core
  stub 021a3ce8 force ret=none args=v  # player_core

# 021A2FE4(player): grid map: 021A548C(+8) == 1 or 021A54A0(+8) == 1: 021A396C(+4, 2); else 021A3818(+4); no-grid:
# 021A59EC(+0xC) == 1: 021A396C(+4, 2); else 021A3818(+4) (update the move state)
fn 021a2fe4 ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a3818 force ret=none args=v  # player_core
  stub 021a396c force ret=none args=v,v  # player_core
  stub 021a548c force ret=pick:0/1 args=v  # player_grid
  stub 021a54a0 force ret=pick:0/1 args=v  # player_grid
  stub 021a59ec force ret=pick:0/1 args=v  # field_player_nogrid predicate

# 021A3038(player, ..): tail 021A3898(+4, ..) (MMDL position -> out (core, &pos))
fn 021a3038 ret=none inputs=100
  arg work 0x100
  arg work 0xA00
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
  stub 021a3898 force ret=none args=v,out:12  # player_core

# 021A3044(player, ..): tail 021A38A4(+4, ..) (warp to a position (core, &pos))
fn 021a3044 ret=none inputs=100
  arg work 0x100
  arg work 0xA00
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
  stub 021a38a4 force ret=none args=v,in:12  # player_core

# 021A3050(player, ..): tail 021A3920(+4, ..) (MMDL display dir (core))
fn 021a3050 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3920 force ret=pick:0/1/2/3 args=v  # player_core

# 021A305C(player, ..): tail 021A392C(+4, ..) (set the MMDL display dir (core, dir))
fn 021a305c ret=none inputs=100
  arg work 0x100
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
  stub 021a392c force ret=none args=v,v  # player_core

# 021A3068(player, ..): tail 021A395C(+4, ..) (fieldmap (core))
fn 021a3068 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a395c force ret=u32 args=v  # player_core

# 021A3074(player, ..): tail 021A3960(+4, ..) (the player MMDL (core))
fn 021a3074 ret=u32 inputs=100
  arg work 0x100
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

# 021A3080(player, ..): tail 021A3968(+4, ..) (move value (core))
fn 021a3080 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3968 force ret=pick:0/1/2 args=v  # player_core

# 021A308C(player, ..): tail 021A3970(+4, ..) (move state (core))
fn 021a308c ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3970 force ret=pick:0/1/2/3 args=v  # player_core

# 021A3098(player, ..): tail 021A3974(+4, ..) (move form (core))
fn 021a3098 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core

# 021A30A4(player, ..): tail 021A3980(+4, ..) (set the move form (core, form))
fn 021a30a4 ret=none inputs=100
  arg work 0x100
  arg u32 alt range 0 9 || any
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
  stub 021a3980 force ret=none args=v,v  # player_core

# 021A30B0(player, ..): tail 021A398C(+4, ..) (sex (core))
fn 021a30b0 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a398c force ret=pick:0/1 args=v  # player_core

# 021A30BC(player, ..): tail 021A3990(+4, ..) (GAMESYS (core))
fn 021a30bc ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3990 force ret=u32 args=v  # player_core

# 021A30C8(player) -> attribute under the player: grid: 021AB0D4(02188C98(fm), &pos) with pos = 021A3038; rail map:
# 021BB3C4(02188C5C(fm), &loc) with loc = 0219E2F4(MMDL); else 0 (r4 = 0 before the second kind test)
fn 021a30c8 ret=u32 inputs=100
  arg work 0x100
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
  stub 02188c5c force ret=u32 args=v  # fieldmap +0x3C (rail map)
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # rail: MMDL rail location -> out
  stub 021a3038 ret=none args=v,out:12  # in-area
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3898 force ret=none args=v,out:12  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (map, &pos)
  stub 021bb3c4 force ret=u32 args=v,in:8  # rail_attr: attribute at a rail location (rail map, &loc)

# 021A3128(player, dir) -> attribute in front: grid: 021AB0D4(map, &pos) with pos = 021A3364(player, dir); rail:
# 021BB3C4(rail map, &loc) with loc = 0219E664(MMDL, dir); else the stale r6 (0 under Z19: callers never pass kind >= 2)
fn 021a3128 ret=u32 inputs=100
  arg work 0x100
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164280 force ret=none args=v,v,out:12  # mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 02188c5c force ret=u32 args=v  # fieldmap +0x3C (rail map)
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # rail: MMDL rail location -> out
  stub 0219e664 force ret=none args=v,v,out:8  # rail: location in dir (m, dir, &loc)
  stub 0219e784 force ret=none args=v,v,out:6  # rail: direction vector of dir (m, dir, &VecFx16)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a330c ret=none args=v,v,out:2,out:2,out:2  # in-area
  stub 021a3364 ret=none args=v,v,out:12  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (map, &pos)
  stub 021b7c38 force ret=pick:0x1000/0x2000/0x10000 args=v  # rail: unit length (rail map)
  stub 021b7d38 force ret=none args=v,in:8,out:12  # rail: location -> VecFx32 (rail map, &loc, &pos)
  stub 021bb3c4 force ret=u32 args=v,in:8  # rail_attr: attribute at a rail location (rail map, &loc)
  stub 021bb480 force ret=u32 args=v  # field_nogrid_mapper: rail map of a map

# 021A3190(sex, i) -> u16 ROM 0x021D890C[sex * 0x28 + i * 4] (OBJ code of form-table entry i)
fn 021a3190 ret=u32 inputs=100
  nowork
  arg u32 range 0 1
  arg u32 range 0 9

# 021A31A4(sex, form) -> OBJ code of the first entry of row sex whose form (u16 +2) is form; none: 1
fn 021a31a4 ret=u32 inputs=100
  nowork
  arg u32 range 0 1
  arg u32 alt range 0 5 || any

# 021A31D0(code) -> form of OBJ code (first match in both rows: the entry index 0..9), 10 = none
fn 021a31d0 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 1 2 3 4 5 6 0x88 0x89 0x8a 0x8b 0x8c 0x8d 0x9c 0x9d 0x9e 0x9f 0xa0 0xa1 || any

# 021A3208(player) -> bool: form (021A3098) not 2 / 3 (surfing forms)
fn 021a3208 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3098 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core

# 021A3220(player, ..): tail 021A3938(+4, ..) (change + save the OBJ code (core, code))
fn 021a3220 ret=none inputs=100
  arg work 0x100
  arg u32 oneof 1 2 3 4 5 6 0x88 0x89
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
  stub 021a3938 force ret=none args=v,v  # player_core

# 021A322C(player, ..): tail 021A394C(+4, ..) (clear the saved OBJ code (core))
fn 021a322c ret=none inputs=100
  arg work 0x100
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
  stub 021a394c force ret=none args=v  # player_core

# 021A3238(player, ..): tail 021A3A20(+4, ..) (OBJ code has a form (core))
fn 021a3238 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3a20 force ret=pick:0/1 args=v  # player_core

# 021A3244(player, ..): tail 021A3A3C(+4, ..) (MMDL busy (core))
fn 021a3244 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3a3c force ret=pick:0/1 args=v  # player_core

# 021A3250(player) -> MMDL in front: grid: 021A3460(player, &x, &y, &z) (front tile); 0216DDC0(MMDLSYS, x, z, 0); rail:
# 0219E634(MMDL, &loc); 0219E570(MMDLSYS, &loc, 0)
fn 021a3250 ret=u32 inputs=100
  arg work 0x100
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216ddc0 force ret=u32 args=v,v,v,v  # fldmmdl: MMDL at grid (sys, x, z, flag)
  stub 02188c7c force ret=u32 args=v  # fieldmap +0x38 (MMDLSYS)
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e570 force ret=u32 args=v,in:8,v  # rail: MMDL at a rail location (sys, &loc, 0)
  stub 0219e634 force ret=pick:0/1 args=v,out:8  # rail: front location of an MMDL (m, &loc)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a330c ret=none args=v,v,out:2,out:2,out:2  # in-area
  stub 021a3460 ret=none args=v,out:2,out:2,out:2  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A32A8(player, h) -> MMDL in front at height: grid: front tile; 0216DDDC(MMDLSYS, x, z, y << 16, h, 0); rail: as 021A3250
fn 021a32a8 ret=u32 inputs=100
  arg work 0x100
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # fldmmdl: MMDL at grid + height (sys, x, z, y, height, flag)
  stub 02188c7c force ret=u32 args=v  # fieldmap +0x38 (MMDLSYS)
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e570 force ret=u32 args=v,in:8,v  # rail: MMDL at a rail location (sys, &loc, 0)
  stub 0219e634 force ret=pick:0/1 args=v,out:8  # rail: front location of an MMDL (m, &loc)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a330c ret=none args=v,v,out:2,out:2,out:2  # in-area
  stub 021a3460 ret=none args=v,out:2,out:2,out:2  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A330C(player, dir, &x, &y, [&z]): grid map: x, y, z (s16) = MMDL grid + (dx, 0, dz) of dir (021641E8 / 021641F8)
fn 021a330c ret=none inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || const 9 || any
  arg u32 addrof work 0xA20
  arg u32 addrof work 0xA24
  arg u32 addrof work 0xA28
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A3364(player, dir, &out): position in front: grid: tile (021A330C) -> centre 02164280(x, z, out), out.y = y << 16;
# rail: rail map = 021BB480(02188C5C(fm)); loc = 0219E2F4(MMDL); 021B7D38(rail map, &loc, &p); 0219E784(MMDL, dir, &d) (VecFx16);
# u = 021B7C38(rail map); out = p + (d * u + 0x800) >> 12 (64-bit products)
fn 021a3364 ret=none inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || const 9 || any
  arg work 0xA00
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02164280 force ret=none args=v,v,out:12  # mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 02188c5c force ret=u32 args=v  # fieldmap +0x3C (rail map)
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e2f4 force ret=none args=v,outm:8:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # rail: MMDL rail location -> out
  stub 0219e784 force ret=none args=v,v,out:6  # rail: direction vector of dir (m, dir, &VecFx16)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a330c ret=none args=v,v,out:2,out:2,out:2  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021b7c38 force ret=pick:0x1000/0x2000/0x10000 args=v  # rail: unit length (rail map)
  stub 021b7d38 force ret=none args=v,in:8,out:12  # rail: location -> VecFx32 (rail map, &loc, &pos)
  stub 021bb480 force ret=u32 args=v  # field_nogrid_mapper: rail map of a map

# 021A3460(player, &x, &y, &z): 021A330C(player, display dir, &x, &y, &z)
fn 021a3460 ret=none inputs=100
  arg work 0x100
  arg u32 addrof work 0xA20
  arg u32 addrof work 0xA24
  arg u32 addrof work 0xA28
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a3074 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a330c ret=none args=v,v,out:2,out:2,out:2  # in-area
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A3484(player, dir, &out): grid: out = (dx << 16, 0, dz << 16) normalised (0207CCA4); rail: s16 vector of 0219E784(MMDL,
# dir) widened into out
fn 021a3484 ret=none inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || const 9 || any
  arg work 0xA00
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
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize(&in, &out) (math)
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 0219e784 force ret=none args=v,v,out:6  # rail: direction vector of dir (m, dir, &VecFx16)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A34EC(player, ..): tail 021A3A10(+4, ..) (form of the OBJ code (core))
fn 021a34ec ret=u32 inputs=100
  arg work 0x100
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
  stub 021a3a10 force ret=pick:0/1/2/3/4/10 args=v  # player_core

# 021A34F8(player, ..): tail 021A39E4(+4, ..) (param counter (core))
fn 021a34f8 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a39e4 force ret=pick:0/1 args=v  # player_core

# 021A3504(player, attr, v) -> form 2 / 3: 0; else 021AB5AC(attr, v)
fn 021a3504 ret=u32 inputs=100
  arg work 0x100
  arg u32 any
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
  stub 021a3098 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021ab5ac force ret=pick:0/1 args=v,v  # attribute predicate (attr, value)

# 021A3524(player, attr, value) -> 021AB11C(attr) == 0 and form 2: 021AB310(021AB0F0(value)); else 0
fn 021a3524 ret=u32 inputs=100
  arg work 0x100
  arg u32 any
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
  stub 021a3098 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3974 force ret=pick:0/1/2/3 args=v  # player_core
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab310 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021A3550(player, on): grid map: 021A5738(+8, on)
fn 021a3550 ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a5738 force ret=none args=v,v  # player_grid

# 021A356C(player): grid map: 021A550C(+8)
fn 021a356c ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a550c force ret=none args=v  # player_grid

# 021A3584(player) -> grid map: 021A5764(+8) (+0x1C); else 0
fn 021a3584 ret=u32 inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a5764 force ret=pick:0/1 args=v  # player_grid

# 021A35A0(player, on): grid map: 021A5768(+8, on)
fn 021a35a0 ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a5768 force ret=none args=v,v  # player_grid

# 021A35BC(player) -> event flag 0x963 (020142E8(EVENT WORK of the GAMEDATA of 021A30BC(player)))
fn 021a35bc ret=u32 inputs=100
  arg work 0x100
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
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 021a30bc ret=u32 args=v  # in-area
  stub 021a3990 force ret=u32 args=v  # player_core

# 021A35E0(player, heap): +8 = 021A4140(+4, heap) (create the grid player)
fn 021a35e0 ret=none inputs=100
  arg work 0x100
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
  stub 021a4140 force ret=ptr:work:0x400:1 args=v,v  # player_grid

# 021A35F0(player, a, b, flags): 021A41B0(+8, a, b, flags) (grid player frame)
fn 021a35f0 ret=none inputs=100
  arg work 0x100
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
  stub 021a41b0 force ret=none args=v,v,v,v  # player_grid

# 021A35FC(player, &x, &y, &z): u16 x, y, z = the MMDL grid position
fn 021a35fc ret=none inputs=100
  arg work 0x100
  arg u32 addrof work 0xA20
  arg u32 addrof work 0xA24
  arg u32 addrof work 0xA28
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
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core

# 021A3624(player) -> grid map and 021A54A0(+8) == 1
fn 021a3624 ret=u32 inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a54a0 force ret=pick:0/1 args=v  # player_grid

# 021A3644(player): grid map and 021A54A0(+8) == 1: 021A54B4(+8)
fn 021a3644 ret=none inputs=100
  arg work 0x100
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
  stub 02188cfc force ret=pick:0/1 args=v  # fieldmap map kind (0 grid, 1 rail)
  stub 021a54a0 force ret=pick:0/1 args=v  # player_grid
  stub 021a54b4 force ret=none args=v  # player_grid

# 021A3664(player): +0xC = 021A5880(+4) (create the no-grid player)
fn 021a3664 ret=none inputs=100
  arg work 0x100
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
  stub 021a5880 force ret=u32 args=v  # field_player_nogrid: create (core)

# 021A3674(player, ..): tail 021A596C(+0xC, ..) (no-grid player, F3b)
fn 021a3674 ret=none inputs=100
  arg work 0x100
  arg u32 any
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
  stub 021a596c force ret=none args=v,v,v  # field_player_nogrid (nogrid, v, v)

# 021A3680(player, ..): tail 021A5928(+0xC, ..) (no-grid player, F3b)
fn 021a3680 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a5928 force ret=u32 args=v,v  # field_player_nogrid (nogrid, v)

# 021A368C(player, ..): tail 021A5964(+0xC, ..) (no-grid player, F3b)
fn 021a368c ret=none inputs=100
  arg work 0x100
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
  stub 021a5964 force ret=none args=v  # field_player_nogrid (nogrid)

# 021A3698(player, ..): tail 021A59EC(+0xC, ..) (no-grid player, F3b)
fn 021a3698 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a59ec force ret=pick:0/1 args=v  # field_player_nogrid predicate

# 021A36A4(player, ..): tail 021A5A30(+0xC, ..) (no-grid player, F3b)
fn 021a36a4 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a5a30 force ret=u32 args=v  # field_player_nogrid (nogrid)

# 021A36B0(player, ..): tail 021A5A00(+0xC, ..) (no-grid player, F3b)
fn 021a36b0 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a5a00 force ret=u32 args=v,v  # field_player_nogrid (nogrid, v)

# 021A36BC(player, ..): tail 021A5A0C(+0xC, ..) (no-grid player, F3b)
fn 021a36bc ret=u32 inputs=100
  arg work 0x100
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
  stub 021a5a0c force ret=u32 args=v,v  # field_player_nogrid (nogrid, v)

# 021A36C8(player, ..): tail 021A5A18(+0xC, ..) (no-grid player, F3b)
fn 021a36c8 ret=u32 inputs=100
  arg work 0x100
  arg u32 any
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
  stub 021a5a18 force ret=u32 args=v,v,v  # field_player_nogrid (nogrid, v, v)

# 021A36D4(player, ..): tail 021A5A24(+0xC, ..) (no-grid player, F3b)
fn 021a36d4 ret=u32 inputs=100
  arg work 0x100
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
  stub 021a5a24 force ret=u32 args=v,v  # field_player_nogrid (nogrid, v)

# 021A36E0(player) -> +8 (the grid player)
fn 021a36e0 ret=u32 inputs=100
  arg work 0x100
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

