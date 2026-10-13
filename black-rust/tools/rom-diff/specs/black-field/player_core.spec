# fndiff spec: field_player_core.c (overlay 21 Thumb code 0x021A36E4..0x021A4140, F1 row 21.22; F3a-spec): PLAYER CORE, the player MMDL,
# move form, OBJ code changes, pad -> direction and the request state machine.
# Run: tools/check-fn.sh black-field player_core --per-function
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
# Coverage (plain): 1085/1085 instructions; closure: 1085/1085.
area player_core
set field

# 021A36E4(rec, fieldmap, &pos, sex, [heap]) -> core: Heap_AllocDebug(heap, 0x34, 1, "field_player_core.c", 0x97); +0xC rec, +8
# fieldmap, +4 = 02188C80(fieldmap) (GAMESYS), +0x20 = +0x22 = 9 (no dir); +0x1C = 0216DE94(MMDLSYS, 0xFF) (the player MMDL); +0x18 sex;
# code = 02012B34(rec) (saved OBJ code); no MMDL: a HEADER copied from ROM 0x021D89A0 (0x24 bytes) gets grid x/z = pos >> 16 (as tiles)
# and y = pos.y, OBJ code = code (0xFFFF: 021A31A4(sex, 021A3974(core)) = the code of the saved move form), +0x1C = 0216CE60(MMDLSYS,
# &header, 0); else the MMDL gets grid x/y/z (pos >> 16) and the position, code 0xFFFF: 021A3998(core); 0216DB80(+0x1C, 1)
fn 021a36e4 ret=u32 inputs=100
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
  stub 02030734 force ret=ptr:work:0x200:1 args=v,v,v,v,v  # Heap_AllocDebug: the CORE block
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02012b34 force ret=pick:0xFFFF/1/4/0x88 args=v  # PLAYER record +0x1C (OBJ code, 0xFFFF = none)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216ce60 force ret=u32 args=v,in:0x24,v  # fldmmdl: MMDL create (sys, &header, zone)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d844 force ret=none args=v,v  # fldmmdl: MMDL gx = v
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d874 force ret=none args=v,v  # fldmmdl: MMDL gz = v
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216db80 force ret=none args=v,v  # fldmmdl: MMDL move bit (m, on)
  stub 0216de94 force ret=nullable:4:u32 args=v,v  # fldmmdl: MMDL by OBJ id (sys, 0xFF)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c7c force ret=u32 args=v  # fieldmap +0x38 (MMDLSYS)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3998 ret=none args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b3c ret=none args=v,v  # in-area

# 021A3818(core): move state from the previous state and the MMDL acmd: s = +0x10 (cleared), v = +0x14 (move value);
# 0216D3CC(m) == 0 (acmd running): v 1 (walk): s 0 / 3 -> 1, else 2; v 2: 2; else 0216D454(m) == 1 (acmd done) and v != 0: v 1 / 2:
# s 0 -> keep 0, s 3 -> 0, else 3
fn 021a3818 ret=none inputs=300
  arg work 0x200
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
  fix work 0x210 u32 range 0 3
  fix work 0x214 u32 alt range 0 2 || range 3 4
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d454 force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd done (m)

# 021A3898(core, &out): tail 0216D88C(+0x1C, out) (MMDL position)
fn 021a3898 ret=none inputs=100
  arg work 0x200
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
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out

# 021A38A4(core, &pos): MMDL old grid = grid; grid x/y/z = pos >> 16 (tiles, s16); MMDL position = pos (warp the player)
fn 021a38a4 ret=none inputs=100
  arg work 0x200
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
  stub 0216d80c force ret=none args=v,v  # fldmmdl: MMDL old gx = v
  stub 0216d818 force ret=none args=v,v  # fldmmdl: MMDL old gy = v
  stub 0216d824 force ret=none args=v,v  # fldmmdl: MMDL old gz = v
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d844 force ret=none args=v,v  # fldmmdl: MMDL gx = v
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gy
  stub 0216d85c force ret=none args=v,v  # fldmmdl: MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d874 force ret=none args=v,v  # fldmmdl: MMDL gz = v
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v

# 021A3920(core) -> tail 0216D5F8(+0x1C) (display dir)
fn 021a3920 ret=u32 inputs=100
  arg work 0x200
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

# 021A392C(core, dir): tail 0216D5E0(+0x1C, dir)
fn 021a392c ret=none inputs=100
  arg work 0x200
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
  stub 0216d5e0 force ret=none args=v,v  # fldmmdl: MMDL set display dir (m, dir)

# 021A3938(core, code): 021A3AD8(core, code) (change the MMDL OBJ code); 02012B30(+0xC, code) (save it in the PLAYER record)
fn 021a3938 ret=none inputs=100
  arg work 0x200
  arg u32 oneof 1 2 3 4 5 6 0x88 0x89 0x6d 0xa0
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
  stub 02012b30 force ret=none args=v,v  # PLAYER record +0x1C = OBJ code (u16)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area

# 021A394C(core): tail 02012B30(+0xC, 0xFFFF) (clear the saved OBJ code)
fn 021a394c ret=none inputs=100
  arg work 0x200
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
  stub 02012b30 force ret=none args=v,v  # PLAYER record +0x1C = OBJ code (u16)

# 021A395C(core) -> +0x8 (fieldmap)
fn 021a395c ret=u32 inputs=100
  arg work 0x200
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

# 021A3960(core) -> +0x1C (MMDL)
fn 021a3960 ret=u32 inputs=100
  arg work 0x200
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

# 021A3964(core, v): +0x14 = v (move value)
fn 021a3964 ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 2 || any
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

# 021A3968(core) -> +0x14 (move value)
fn 021a3968 ret=u32 inputs=100
  arg work 0x200
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

# 021A396C(core, v): +0x10 = v (move state)
fn 021a396c ret=none inputs=100
  arg work 0x200
  arg u32 alt range 0 3 || any
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

# 021A3970(core) -> +0x10 (move state)
fn 021a3970 ret=u32 inputs=100
  arg work 0x200
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

# 021A3974(core) -> form: tail 02012B28(+0xC) (PLAYER record +0x40: 0 walk, 1 cycle, 2 surf, 3 ..)
fn 021a3974 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)

# 021A3980(core, form): tail 02012B2C(+0xC, form)
fn 021a3980 ret=none inputs=100
  arg work 0x200
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
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form

# 021A398C(core) -> +0x18 (sex (table row))
fn 021a398c ret=u32 inputs=100
  arg work 0x200
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

# 021A3990(core) -> +0x4 (GAMESYS)
fn 021a3990 ret=u32 inputs=100
  arg work 0x200
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

# 021A3994(core) -> +0xC (PLAYER record)
fn 021a3994 ret=u32 inputs=100
  arg work 0x200
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

# 021A3998(core): 021A3B3C(core, 021A3974(core)) (MMDL OBJ code of the saved form)
fn 021a3998 ret=none inputs=100
  arg work 0x200
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
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b3c ret=none args=v,v  # in-area

# 021A39AC(core, i): code = 021A3190(sex, i) (OBJ code table 0x021D890C row sex, entry i); != 0216D598(m): 021A3AD8(core,
# code); +0x2C = 0
fn 021a39ac ret=none inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area

# 021A39E4(core) -> bool: 0216D7A8(m, 0) != 0xFFFF: +0x2C == 1: 1; else +0x2C += 1; 0
fn 021a39e4 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area

# 021A3A10(core) -> form: 021A31D0(0216D598(m)) (form index of the MMDL OBJ code, 10 = none)
fn 021a3a10 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 021a31d0 force ret=pick:0/1/2/3/4/10 args=v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area

# 021A3A20(core) -> bool: 021A31D0(0216D598(m)) != 10
fn 021a3a20 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 021a31d0 force ret=pick:0/1/2/3/4/10 args=v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area

# 021A3A3C(core) -> bool: d = 0219C9DC(0216D520(0216D67C(m))); 02197C9C(m) == 1 and 0219C648(m) == 1: 1; p = 0216D7A8(m, 0);
# p != 0xFFFF and 020508EC(d, p, &u16 out) and out == 0x3FFF: 1; else 0
fn 021a3a3c ret=u32 inputs=100
  arg work 0x200
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
  stub 020508ec force ret=pick:0/1 args=v,v,outv:2:0x3FFF/0/1/0x3FFE  # (obj, id, &u16 out): found? (out 0x3FFF = ..)
  stub 0216d520 force ret=u32 args=v  # fldmmdl: MMDLSYS +0x24 (draw system)
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 02197c9c force ret=pick:0/1 args=v  # fldmmdl_acmd: MMDL predicate
  stub 0219c648 force ret=pick:0/1 args=v  # fldmmdl_blact: MMDL predicate
  stub 0219c9dc force ret=u32 args=v  # fldmmdl_blact: draw system getter

# 021A3AA0(core) -> 0216D3CC(m) == 1 (acmd settable)
fn 021a3aa0 ret=u32 inputs=100
  arg work 0x200
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
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area

# 021A3AB8(core, keys) -> dir: tail 021A3BB4(core, (u16)keys)
fn 021a3ab8 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL move dir
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3bb4 ret=pick:0/1/2/3/9 args=v,v  # in-area
  stub 021a3c3c ret=pick:2/3/9 args=v  # in-area
  stub 021a3c54 ret=pick:0/1/9 args=v  # in-area

# 021A3AC4(core, keys): tail 021A3C24(core, (u16)keys)
fn 021a3ac4 ret=none inputs=100
  arg work 0x200
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
  stub 021a3c24 ret=none args=v,v  # in-area
  stub 021a3c3c ret=pick:2/3/9 args=v  # in-area
  stub 021a3c54 ret=pick:0/1/9 args=v  # in-area

# 021A3AD0(core, v): +0x28 = v (surf effect task)
fn 021a3ad0 ret=none inputs=100
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

# 021A3AD4(core) -> +0x28 (surf effect task)
fn 021a3ad4 ret=u32 inputs=100
  arg work 0x200
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

# 021A3AD8(core, code): OBJ code change of the player MMDL: sys = 0216D67C(m); code or the current code 1 / 4 (walk):
# 0219D208(m, code) (model), 0216CD88(sys); else 0216E204(m, 021A31A4(sex, 0)) (the walk code first), 0216CD88(sys), 0219D208(m,
# code), 0216CD88(sys)
fn 021a3ad8 ret=none inputs=100
  arg work 0x200
  arg u32 alt oneof 1 4 2 5 3 6 0x88 0x89 || any
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area

# 021A3B3C(core, form): code = 021A31A4(sex, form); != 0216D598(m): 021A3AD8(core, code); 021A3980(core, form)
fn 021a3b3c ret=none inputs=100
  arg work 0x200
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
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area

# 021A3B78(core) -> bool: MMDL OBJ code 1 or 4 (walk codes)
fn 021a3b78 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area

# 021A3B94(core) -> bool: (u16)0216D7A8(m, 0) != 0xFFFF
fn 021a3b94 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)

# 021A3BB4(core, keys) -> dir: x = 021A3C3C(keys) (left 2 / right 3 / 9), z = 021A3C54(keys) (up 0 / down 1 / 9); one of
# them 9: the other; both: d = 0216D608(m) (move dir); d == 9: z; x and z equal to the remembered +0x20 / +0x22: z if d == z, x if
# d == x; else z if z != +0x22, else x (a newly pressed direction wins)
fn 021a3bb4 ret=u32 inputs=300
  arg work 0x200
  arg u32 alt oneof 0x50 0x60 0x90 0xA0 || mask 0xF0 || any
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
  fix work 0x220 u16 alt range 2 3 || const 9
  fix work 0x222 u16 alt range 0 1 || const 9
  stub 0216d608 force ret=pick:0/1/2/3/9 args=v  # fldmmdl: MMDL move dir (9 = none)
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3c3c ret=pick:2/3/9 args=v  # in-area
  stub 021a3c54 ret=pick:0/1/9 args=v  # in-area

# 021A3C24(core, keys): +0x20 = 021A3C3C(keys), +0x22 = 021A3C54(keys) (remember the pressed directions)
fn 021a3c24 ret=none inputs=100
  arg work 0x200
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
  stub 021a3c3c ret=pick:2/3/9 args=v  # in-area
  stub 021a3c54 ret=pick:0/1/9 args=v  # in-area

# 021A3C3C(keys) -> 0x20 (left): 2, 0x10 (right): 3, else 9
fn 021a3c3c ret=u32 inputs=100
  nowork
  arg u32 alt mask 0xF0 || any

# 021A3C54(keys) -> 0x40 (up): 0, 0x80 (down): 1, else 9
fn 021a3c54 ret=u32 inputs=100
  nowork
  arg u32 alt mask 0xF0 || any

# 021A3C6C(core, req) -> done: +0x32 == 0: loop: proc = ROM 0x021D89C4[req][+0x30 (s16 seq)]; proc(core) == 1: again
# (the request procs 021A3DA0 .. 021A40A8 all return 0); return +0x32
fn 021a3c6c ret=u32 inputs=100
  arg work 0x200
  arg u32 range 0 9
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 02164240 force ret=none args=v,in:2,in:2,v  # mmdl_move: move an s16 grid pair in dir (dir, &x, &z, n)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216ddc0 force ret=u32 args=v,v,v,v  # fldmmdl: MMDL at grid (sys, x, z, flag)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 0219f5c8 force ret=none args=v,v  # rail (m, flag)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad0 ret=none args=v,v  # in-area
  stub 021a3ad4 ret=nullable:2:u32 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area
  stub 021a3da0 ret=pick:0 args=v  # in-area
  stub 021a3e14 ret=pick:0 args=v  # in-area
  stub 021a3e48 ret=pick:0 args=v  # in-area
  stub 021a3e54 ret=pick:0 args=v  # in-area
  stub 021a3ee0 ret=pick:0 args=v  # in-area
  stub 021a3f10 ret=pick:0 args=v  # in-area
  stub 021a3f48 ret=pick:0 args=v  # in-area
  stub 021a3f64 ret=pick:0 args=v  # in-area
  stub 021a3f70 ret=pick:0 args=v  # in-area
  stub 021a3f8c ret=pick:0 args=v  # in-area
  stub 021a3f98 ret=pick:0 args=v  # in-area
  stub 021a3fb4 ret=pick:0 args=v  # in-area
  stub 021a3fc0 ret=pick:0 args=v  # in-area
  stub 021a3fdc ret=pick:0 args=v  # in-area
  stub 021a3fe8 ret=pick:0 args=v  # in-area
  stub 021a408c ret=pick:0 args=v  # in-area
  stub 021a40a8 ret=pick:0 args=v  # in-area
  stub 021a4114 ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021acb38 force ret=u32 args=v,v,in:12,v,v  # namipoke
  stub 021acbc8 force ret=none args=v,v  # namipoke
  stub 021d05d8 force ret=none args=v,v  # (MMDL, field effect control)

# 021A3C94(bits) -> index of the lowest set bit among 0..9, 10 = none
fn 021a3c94 ret=u32 inputs=100
  nowork
  arg u32 alt mask 0x3FF || any || const 0

# 021A3CB0(core, bits): +0x24 = bits (requests), +0x30 = +0x32 = +0x2C = +0x2E = 0
fn 021a3cb0 ret=none inputs=100
  arg work 0x200
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

# 021A3CC0(core) -> done: i = 021A3C94(+0x24); i >= 10: 1; else 021A3C6C(core, i) == 1
fn 021a3cc0 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 02164240 force ret=none args=v,in:2,in:2,v  # mmdl_move: move an s16 grid pair in dir (dir, &x, &z, n)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216ddc0 force ret=u32 args=v,v,v,v  # fldmmdl: MMDL at grid (sys, x, z, flag)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 0219f5c8 force ret=none args=v,v  # rail (m, flag)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad0 ret=none args=v,v  # in-area
  stub 021a3ad4 ret=nullable:2:u32 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3c6c seq ret=pick:0/1 args=v,v  # in-area
  stub 021a3c94 ret=pick:0/1/2/3/4/5/8/9/10 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area
  stub 021a3da0 ret=pick:0 args=v  # in-area
  stub 021a3e14 ret=pick:0 args=v  # in-area
  stub 021a3e48 ret=pick:0 args=v  # in-area
  stub 021a3e54 ret=pick:0 args=v  # in-area
  stub 021a3ee0 ret=pick:0 args=v  # in-area
  stub 021a3f10 ret=pick:0 args=v  # in-area
  stub 021a3f48 ret=pick:0 args=v  # in-area
  stub 021a3f64 ret=pick:0 args=v  # in-area
  stub 021a3f70 ret=pick:0 args=v  # in-area
  stub 021a3f8c ret=pick:0 args=v  # in-area
  stub 021a3f98 ret=pick:0 args=v  # in-area
  stub 021a3fb4 ret=pick:0 args=v  # in-area
  stub 021a3fc0 ret=pick:0 args=v  # in-area
  stub 021a3fdc ret=pick:0 args=v  # in-area
  stub 021a3fe8 ret=pick:0 args=v  # in-area
  stub 021a408c ret=pick:0 args=v  # in-area
  stub 021a40a8 ret=pick:0 args=v  # in-area
  stub 021a4114 ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021acb38 force ret=u32 args=v,v,in:12,v,v  # namipoke
  stub 021acbc8 force ret=none args=v,v  # namipoke
  stub 021d05d8 force ret=none args=v,v  # (MMDL, field effect control)

# 021A3CE8(core): i = 021A3C94(+0x24); i < 10: loop 021A3C6C(core, i) until it answers nonzero
fn 021a3ce8 ret=none inputs=100
  arg work 0x200
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 02164240 force ret=none args=v,in:2,in:2,v  # mmdl_move: move an s16 grid pair in dir (dir, &x, &z, n)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216ddc0 force ret=u32 args=v,v,v,v  # fldmmdl: MMDL at grid (sys, x, z, flag)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 0219f5c8 force ret=none args=v,v  # rail (m, flag)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad0 ret=none args=v,v  # in-area
  stub 021a3ad4 ret=nullable:2:u32 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3c6c seq ret=pick:0/1 args=v,v  # in-area
  stub 021a3c94 ret=pick:0/1/2/3/4/5/8/9/10 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area
  stub 021a3da0 ret=pick:0 args=v  # in-area
  stub 021a3e14 ret=pick:0 args=v  # in-area
  stub 021a3e48 ret=pick:0 args=v  # in-area
  stub 021a3e54 ret=pick:0 args=v  # in-area
  stub 021a3ee0 ret=pick:0 args=v  # in-area
  stub 021a3f10 ret=pick:0 args=v  # in-area
  stub 021a3f48 ret=pick:0 args=v  # in-area
  stub 021a3f64 ret=pick:0 args=v  # in-area
  stub 021a3f70 ret=pick:0 args=v  # in-area
  stub 021a3f8c ret=pick:0 args=v  # in-area
  stub 021a3f98 ret=pick:0 args=v  # in-area
  stub 021a3fb4 ret=pick:0 args=v  # in-area
  stub 021a3fc0 ret=pick:0 args=v  # in-area
  stub 021a3fdc ret=pick:0 args=v  # in-area
  stub 021a3fe8 ret=pick:0 args=v  # in-area
  stub 021a408c ret=pick:0 args=v  # in-area
  stub 021a40a8 ret=pick:0 args=v  # in-area
  stub 021a4114 ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021acb38 force ret=u32 args=v,v,in:12,v,v  # namipoke
  stub 021acbc8 force ret=none args=v,v  # namipoke
  stub 021d05d8 force ret=none args=v,v  # (MMDL, field effect control)

# 021A3D08(core, code) -> bool: code == MMDL OBJ code: +0x32 = 1, return (r0 of the getter, nonzero); MMDL in a walk code
# (021A3B78): 021A3AD8(core, code), +0x32 = 1, 1; else 021A3AD8(core, 021A3190(sex, 0)) (walk code first), 0
fn 021a3d08 ret=u32 inputs=100
  arg work 0x200
  arg u32 alt oneof 1 2 3 4 5 6 0x88 0x89 || any
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area

# 021A3D50(core, i) -> 021A3D08(core, 021A3190(sex, i))
fn 021a3d50 ret=u32 inputs=100
  arg work 0x200
  arg u32 range 0 9
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area

# 021A3D6C(core, i) -> bool: 021A3B94(core) == 1 (param set): +0x2E += 1; 021A3AD8(core, 021A3190(sex, i)); +0x32 = 1; 1
fn 021a3d6c ret=u32 inputs=100
  arg work 0x200
  arg u32 range 0 9
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area

# 021A3DA0(core) -> 0 (request 0 proc, walk): code of entry 0; change if different; 021A3980(core, 0) (form walk);
# 021A4114(core); a surf effect task (021A3AD4): 021AC154(task), 021A3AD0(core, 0); 0216D8E0(m, &(0, 0, 0)) (draw offset); +0x32 = 1
fn 021a3da0 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad0 ret=none args=v,v  # in-area
  stub 021a3ad4 ret=nullable:2:u32 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a4114 ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect: delete task

# 021A3E14(core) -> 0 (request 1 proc 0, cycle): 021A3980(core, 1); 021A4114(core); 021A3D50(core, 1) != 1: +0x30 += 1
fn 021a3e14 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area
  stub 021a4114 ret=none args=v  # in-area

# 021A3E48(core) -> 0: 021A3D6C(core, 1)
fn 021a3e48 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area

# 021A3E54(core) -> 0 (request 2 proc, surf): code of entry 2, change if different; form != 2: 021A3980(core, 2);
# 021A4114(core); no surf effect task: +0x28 = 021ACB38(02188CD0(+8), display dir, &pos, m, 1) (add the surf Pokemon); +0x32 = 1
fn 021a3e54 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad0 ret=none args=v,v  # in-area
  stub 021a3ad4 ret=nullable:2:u32 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a4114 ret=none args=v  # in-area
  stub 021acb38 force ret=u32 args=v,v,in:12,v,v  # namipoke

# 021A3EE0(core) -> 0: 021A3D08(core, 021A31A4(sex, form)) != 1: +0x30 += 1
fn 021a3ee0 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area

# 021A3F10(core) -> 0: 021A3B94(core) == 1: +0x2E += 1, 021A3AD8(core, 021A31A4(sex, form)), +0x32 = 1
fn 021a3f10 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area

# 021A3F48(core) -> 0: 021A3D50(core, 4) != 1: +0x30 += 1
fn 021a3f48 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area

# 021A3F64(core) -> 0: 021A3D6C(core, 4)
fn 021a3f64 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area

# 021A3F70(core) -> 0: 021A3D50(core, 5) != 1: +0x30 += 1
fn 021a3f70 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area

# 021A3F8C(core) -> 0: 021A3D6C(core, 5)
fn 021a3f8c ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area

# 021A3F98(core) -> 0: 021A3D50(core, 6) != 1: +0x30 += 1
fn 021a3f98 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area

# 021A3FB4(core) -> 0: 021A3D6C(core, 6)
fn 021a3fb4 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area

# 021A3FC0(core) -> 0: 021A3D50(core, 9) != 1: +0x30 += 1
fn 021a3fc0 ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area

# 021A3FDC(core) -> 0: 021A3D6C(core, 9)
fn 021a3fdc ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area

# 021A3FE8(core) -> 0 (request 8 proc 0): code of entry 3, change if different; form != 3: 021A3980(core, 3); no surf
# effect task: +0x28 = 021ACB38(ctrl, dir, &pos, m, 1); 021D05D8(m, ctrl); +0x28 != 0: 021ACBC8(+0x28, 0); +0x32 = 1
fn 021a3fe8 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02012b2c force ret=none args=v,v  # PLAYER record +0x40 = form
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a3980 ret=none args=v,v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad0 ret=none args=v,v  # in-area
  stub 021a3ad4 ret=nullable:2:u32 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021acb38 force ret=u32 args=v,v,in:12,v,v  # namipoke
  stub 021acbc8 force ret=none args=v,v  # namipoke
  stub 021d05d8 force ret=none args=v,v  # (MMDL, field effect control)

# 021A408C(core) -> 0: 021A3D50(core, 8) != 1: +0x30 += 1
fn 021a408c ret=u32 inputs=100
  arg work 0x200
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
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b78 ret=pick:0/1 args=v  # in-area
  stub 021a3d08 ret=pick:0/1 args=v,v  # in-area
  stub 021a3d50 ret=pick:0/1 args=v,v  # in-area

# 021A40A8(core) -> 0: 021A3D6C(core, 8); +0x32 == 1 and form 2: x, z = MMDL grid; 02164240(1, &x, &z, 1) (one step down);
# k = 0216DDC0(sys, x, z, 1) != 0; 0219F5C8(m, k)
fn 021a40a8 ret=u32 inputs=100
  arg work 0x200
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
  stub 02012b28 force ret=pick:0/1/2/3 args=v  # PLAYER record +0x40 (move form: 0 walk, 1 cycle, 2 surf, 3 ..)
  stub 02164240 force ret=none args=v,in:2,in:2,v  # mmdl_move: move an s16 grid pair in dir (dir, &x, &z, n)
  stub 0216cd88 force ret=none args=v  # fldmmdl: MMDLSYS (sys)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d7a8 force seq ret=pick:0xFFFF/0/1/2 args=v,v  # fldmmdl: MMDL param (m, 0) (0xFFFF = none)
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216ddc0 force ret=u32 args=v,v,v,v  # fldmmdl: MMDL at grid (sys, x, z, flag)
  stub 0216e204 force ret=none args=v,v  # fldmmdl: MMDL change OBJ code (m, code)
  stub 0219d208 force ret=none args=v,v  # fldmmdl_g3dobj: change model (m, code)
  stub 0219f5c8 force ret=none args=v,v  # rail (m, flag)
  stub 021a3190 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a31a4 force ret=pick:1/2/3/4/5/6/0x88/0x89 args=v,v  # player
  stub 021a3960 ret=ptr:work:0x900:1 args=v  # in-area
  stub 021a3974 ret=pick:0/1/2/3 args=v  # in-area
  stub 021a398c ret=pick:0/1 args=v  # in-area
  stub 021a3ad8 ret=none args=v,v  # in-area
  stub 021a3b94 ret=pick:0/1 args=v  # in-area
  stub 021a3d6c ret=pick:0/1 args=v,v  # in-area

# 021A4114(core): gd = 020120F4(02188C80(+8)); 02029D8C(02012A30(gd), gd, (u16)02188C9C(+8)) (zone id)
fn 021a4114 ret=none inputs=100
  arg work 0x200
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
  stub 02012a30 force ret=u32 args=v  # GAMEDATA getter
  stub 02029d8c force ret=none args=v,v,v  # (gamedata getter, gamedata, zone id)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 02188c9c force ret=u16 args=v  # fieldmap zone id

