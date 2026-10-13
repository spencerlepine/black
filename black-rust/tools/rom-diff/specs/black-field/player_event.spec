# fndiff spec: field_player_grid_event.c (overlay 21 Thumb code 0x021C1024..0x021C1A08, F1 row 21.65; F3a-spec): the surf start / end tasks.
# Run: tools/check-fn.sh black-field player_event --per-function
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
# SURF START task work (0x18, Heap_AllocDebug(heap | 0x8000, 0x18, 1, "field_player_grid_event.c", 0x1E6) in 021C1024; task proc 021C10B4,
#   callers 021C4586..021C465C in F1 row 21.68): +0 u16 step 0..5, +2 s16 frame, +4 u16 done (021C1094), +6 u16 dir, +8 u16 two-tile jump
#   (021AB2E8 of the attribute), +0xC FIELDMAP, +0x10 FIELD_PLAYER, +0x14 GRID.
# SURF END task work (0x4C, line 0x2B3, 021C1228; task proc 021C1988): +0 u16 jump down (target lower than the player), +2 s16 step
#   (0x021DA128 table: 021C1318 start, 021C146C hop, 021C14F8 rise, 021C157C arc, 021C161C land, 021C1670 wait for the splash), +4 u16 dir,
#   +6 u16 done (021C12C4), +8 u16 frame, +0x10 gravity, +0x18.. VecFx32 draw offset (+0x1C bob), +0x24.. VecFx32 target (+0x28 its height),
#   +0x30 / +0x34 / +0x38 per-frame deltas x / y / z, +0x3C FIELD_PLAYER, +0x40 FIELDMAP, +0x44 GRID, +0x48 splash task.
# Coverage (plain): 1067/1067 instructions; closure: 1067/1067.
area player_event
set field

# 021C1024(player, dir, attr, heap) -> task: surf start: w = Heap_AllocDebug(heap & 0x7FFF | 0x8000, 0x18, 1,
# "field_player_grid_event.c", 0x1E6); +0xC = 021A3068(player) (fieldmap), +6 dir, +0x10 player, +0x14 = 021A36E0(player) (grid);
# 021AB2E8(021AB0F0(attr)) == 1: +8 = 1; 02030DA8(02188D30(fieldmap), 021C10B5, w, 0) (task)
fn 021c1024 ret=u32 inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || const 9 || any
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
  stub 02030734 force ret=ptr:work:0xC00:1 args=v,v,v,v,v  # Heap_AllocDebug: the task work
  stub 02030da8 force ret=u32 args=v,v,v,v  # task add (sys, proc, work, priority)
  stub 02188d30 force ret=u32 args=v  # fieldmap +0x110 (task system)
  stub 021a3068 force ret=u32 args=v  # player
  stub 021a36e0 force ret=ptr:work:0x400:1 args=v  # player
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab2e8 force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021C1094(task) -> 02030EE8(task) +4 (done)
fn 021c1094 ret=u32 inputs=100
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
  fix work 0xC04 u16 range 0 1
  stub 02030ee8 force ret=ptr:work:0xC00:1 args=v  # task -> work

# 021C10A0(task): Heap_Free(02030EE8(task)); 02030EAC(task)
fn 021c10a0 ret=none inputs=100
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
  stub 02030ee8 force ret=ptr:work:0xC00:1 args=v  # task -> work
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02030eac force ret=none args=v  # task delete (task)

# 021C10B4(task, w): surf start proc by +0 (0..5): 0: pos = MMDL tile centre (02164280), one step in dir (2 with +8),
# height (02163E58); +0x28 surf effect = 021ACB38(ctrl, dir, &pos, m, 0) -> 021A2F8C(player, task); 021AA900(encount, s16 tile xyz)
# true: 021AAB1C(encount, 1); step; 1: +2 += 1, > 15: step; 2: acmd 02197E7C(dir, 0x34) (+8: 0x38 jump), step; 3: 0216D454 (done):
# 0216D480, 021ACBB8(021A2F98(player), 1), 021A2F48(player, 4) (request surf), step, 021A2F54(player) == 1: 021A35A0(player, 1),
# +4 = 1, step
fn 021c10b4 ret=none inputs=100
  arg u32 any
  arg work 0xC00
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
  fix work 0xC00 u16 alt range 0 5 || any
  fix work 0xC02 s16 alt range 14 16 || range 0 20
  fix work 0xC06 u16 alt range 0 3 || any
  fix work 0xC08 u16 range 0 1
  fix work 0xC10 u32 addrof work 0x100
  fix work 0xC14 u32 addrof work 0x400
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 02164280 force ret=none args=v,v,out:12  # mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d454 force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd done (m)
  stub 0216d480 force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd end (m)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 02188ce8 force ret=u32 args=v  # fieldmap +0x90 (encounter object)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a2f48 force ret=none args=v,v  # player
  stub 021a2f54 force ret=pick:0/1 args=v  # player
  stub 021a2f8c force ret=none args=v,v  # player
  stub 021a2f98 force ret=nullable:2:u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021a35a0 force ret=none args=v,v  # player
  stub 021aa900 force ret=pick:0/1 args=v,in:6  # field_encount: (encount, &s16 grid xyz) predicate
  stub 021aab1c force ret=none args=v,v  # field_encount (encount, 1)
  stub 021acb38 force ret=u32 args=v,v,in:12,v,v  # namipoke
  stub 021acbb8 force ret=none args=v,v  # namipoke

# 021C1228(player, dir, _, heap) -> task: surf end: w = Heap_AllocDebug(heap | 0x8000, 0x4C, 1, .., 0x2B3); +4 dir, +0x3C
# player, +0x40 fieldmap; pos = MMDL position two steps ahead; height there (02163E58) > pos.y: +0 = 1 (jump down); +0x44 = grid;
# 02030DA8(tasks, 021C1989, w, 0)
fn 021c1228 ret=u32 inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || const 9 || any
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
  stub 02030734 force ret=ptr:work:0xC00:1 args=v,v,v,v,v  # Heap_AllocDebug: the task work
  stub 02030da8 force ret=u32 args=v,v,v,v  # task add (sys, proc, work, priority)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 02188d30 force ret=u32 args=v  # fieldmap +0x110 (task system)
  stub 021a3068 force ret=u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021a36e0 force ret=ptr:work:0x400:1 args=v  # player

# 021C12C4(task) -> 02030EE8(task) +6 (done)
fn 021c12c4 ret=u32 inputs=100
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
  fix work 0xC06 u16 range 0 1
  stub 02030ee8 force ret=ptr:work:0xC00:1 args=v  # task -> work

# 021C12D0(task): Heap_Free(02030EE8(task)); 02030EAC(task)
fn 021c12d0 ret=none inputs=100
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
  stub 02030ee8 force ret=ptr:work:0xC00:1 args=v  # task -> work
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02030eac force ret=none args=v  # task delete (task)

# 021C12E4(m, &off): 0216D8E0(m, &(off.x, off.y + 0x1000, off.z + 0x5000)) (draw offset of the surf ride)
fn 021c12e4 ret=none inputs=100
  arg work 0x900
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
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v

# 021C1318(w) -> 1: jump-off start: 0216DB4C(m, 1); pos = MMDL position; one / two tiles ahead (02164208 0x10000 /
# 0x20000) -> tile centres (02164280) -> +0x24 target; height there (02163E58 -> +0x28); +0x30 / +0x38 = (target - pos) / 32 per frame;
# +0x34 = 0x6000, +0x1C = 0x1000; 021ACBB8(021A2F98(player), 2); kind = dir 2 / 3 ? 3 : 1; 021ABE30(ctrl, &9, 1); +0x48 =
# 021AD564(ctrl, kind, surf task) (splash); 021C6B18(fieldmap, 0x10, 0x20000) (shake); SE 0x6E0; +2 += 1
fn 021c1318 ret=u32 inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 02164280 force ret=none args=v,v,out:12  # mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 021a2f98 force ret=nullable:2:u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021abe30 force ret=none args=v,in:4,v  # field effect: register (ctrl, &id, n)
  stub 021acbb8 force ret=none args=v,v  # namipoke
  stub 021ad564 force ret=u32 args=v,v,v  # namipoke
  stub 021c6b18 force ret=none args=v,v,v  # fld3d / camera shake (fieldmap, 0x10, amount)

# 021C146C(w) -> 0: move the MMDL by +0x30 / +0x38; +0x1C negated; +8 += 1, 16 frames: +8 = 0, +0x1C = 0, +2 += 1,
# 021AC154(+0x48), +0x48 = 021AD564(ctrl, 2, surf task); 021C12E4(m, w +0x18)
fn 021c146c ret=u32 inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 021a2f98 force ret=nullable:2:u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021ad564 force ret=u32 args=v,v,v  # namipoke
  stub 021c12e4 ret=none args=v,in:12  # in-area

# 021C14F8(w) -> 0: rise: y += +0x34; >= +0x28 (target height): y = target, +2 += 1, +0x30 / +0x38 = (target - pos) / 16,
# +0x10 = -0x8000 / 11, +0x34 = 0x6000, 021C6B18(fieldmap, 0x10, 0x20000), 021AC154(+0x48); position = pos
fn 021c14f8 ret=u32 inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021c6b18 force ret=none args=v,v,v  # fld3d / camera shake (fieldmap, 0x10, amount)

# 021C157C(w) -> 0: arc: x / z step toward +0x24 / +0x2C (clamped), y += +0x34, +0x34 += +0x10 (gravity); +8 += 1,
# 16 frames: position = target, +0x18.. = 0, +2 += 1; 021C12E4(m, w +0x18)
fn 021c157c ret=u32 inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021c12e4 ret=none args=v,in:12  # in-area

# 021C161C(w) -> 0: land: 0216DB4C(m, 0); 0216E014(m, &target, dir); 021ACBB8(surf task, 1); +0x48 = 021AD564(ctrl, 0,
# surf task); SE 0x6E2; +2 += 1
fn 021c161c ret=u32 inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216e014 force ret=none args=v,in:12,v  # fldmmdl: MMDL set position + dir (m, &pos, dir)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 021a2f98 force ret=nullable:2:u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021acbb8 force ret=none args=v,v  # namipoke
  stub 021ad564 force ret=u32 args=v,v,v  # namipoke

# 021C1670(w) -> 0: +0x48 != 0 and 021AD5CC(+0x48) == 1 (splash over): 021AC154(+0x48), 021ABE64(ctrl, 9),
# 021A35A0(player, 1), +0x48 = 0, +6 = 1 (done)
fn 021c1670 ret=u32 inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 021a35a0 force ret=none args=v,v  # player
  stub 021abe64 force ret=none args=v,v  # field effect: unregister (ctrl, id)
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021ad5cc force ret=pick:0/1 args=v  # namipoke

# 021C16AC(w): jump-off down (+0 == 1) by +2 (0..3): 0: as 021C1318 (one + two tiles, rise -0x6000), SE 0x6E1, then the
# first frame of 1; 1: move by +0x30 / +0x38, +0x1C negated, 16 frames: next, splash 2; 021C12E4; 2: fall y += +0x34 to +0x28,
# then deltas / 16, shake 0x10, splash 0; 3: move, 16 frames: 0216DB4C(m, 0), 0216E014(m, &target, dir), 021AC154, 021ABE64(ctrl, 9),
# 021A35A0(player, 1), +6 = 1, SE 0x6E2, +2 += 1
fn 021c16ac ret=none inputs=100
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 02164280 force ret=none args=v,v,out:12  # mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216e014 force ret=none args=v,in:12,v  # fldmmdl: MMDL set position + dir (m, &pos, dir)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 021a2f98 force ret=nullable:2:u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021a35a0 force ret=none args=v,v  # player
  stub 021abe30 force ret=none args=v,in:4,v  # field effect: register (ctrl, &id, n)
  stub 021abe64 force ret=none args=v,v  # field effect: unregister (ctrl, id)
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021acbb8 force ret=none args=v,v  # namipoke
  stub 021ad564 force ret=u32 args=v,v,v  # namipoke
  stub 021c12e4 ret=none args=v,in:12  # in-area
  stub 021c6b18 force ret=none args=v,v,v  # fld3d / camera shake (fieldmap, 0x10, amount)

# 021C1988(task, w): +0 != 0: 021C16AC(w); else loop: ROM 0x021DA128[+2](w) (021C1318 .. 021C1670) while it answers 1
fn 021c1988 ret=none inputs=100
  arg u32 any
  arg work 0xC00
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
  fix work 0xC00 u16 range 0 1
  fix work 0xC02 s16 range 0 5
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC08 u16 alt range 14 16 || range 0 20
  fix work 0xC10 u32 alt oneof 0xFFFFF000 0x1000 0 || any
  fix work 0xC1C u32 alt oneof 0x1000 0xFFFFF000 0 || any
  fix work 0xC24 u32 alt range 0 0x7FFFF || any
  fix work 0xC28 u32 alt range 0 0x7FFFF || any
  fix work 0xC2C u32 alt range 0 0x7FFFF || any
  fix work 0xC30 u32 alt range 0 0x2000 || any
  fix work 0xC34 u32 alt range 0 0x6000 || any
  fix work 0xC38 u32 alt range 0 0x2000 || any
  fix work 0xC3C u32 addrof work 0x100
  fix work 0xC44 u32 addrof work 0x400
  fix work 0xC48 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 02164280 force ret=none args=v,v,out:12  # mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216e014 force ret=none args=v,in:12,v  # fldmmdl: MMDL set position + dir (m, &pos, dir)
  stub 02188cd0 force ret=u32 args=v  # fieldmap +0x94 (field effect control)
  stub 021a2f98 force ret=nullable:2:u32 args=v  # player
  stub 021a3074 force ret=ptr:work:0x900:1 args=v  # player
  stub 021a35a0 force ret=none args=v,v  # player
  stub 021abe30 force ret=none args=v,in:4,v  # field effect: register (ctrl, &id, n)
  stub 021abe64 force ret=none args=v,v  # field effect: unregister (ctrl, id)
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021acbb8 force ret=none args=v,v  # namipoke
  stub 021ad564 force ret=u32 args=v,v,v  # namipoke
  stub 021ad5cc force ret=pick:0/1 args=v  # namipoke
  stub 021c12e4 ret=none args=v,in:12  # in-area
  stub 021c1318 seq ret=pick:0/1 args=v  # in-area
  stub 021c146c seq ret=pick:0/1 args=v  # in-area
  stub 021c14f8 seq ret=pick:0/1 args=v  # in-area
  stub 021c157c seq ret=pick:0/1 args=v  # in-area
  stub 021c161c seq ret=pick:0/1 args=v  # in-area
  stub 021c1670 seq ret=pick:0/1 args=v  # in-area
  stub 021c16ac ret=none args=v  # in-area
  stub 021c6b18 force ret=none args=v,v,v  # fld3d / camera shake (fieldmap, 0x10, amount)

# 021C19B4(grid, dir) -> MMDL in front: m = 021A572C(grid); 0216DDDC(MMDLSYS, gx + dx(dir), gz + dz(dir), y, 1, 0)
fn 021c19b4 ret=u32 inputs=100
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
  stub 021641e8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dx of dir
  stub 021641f8 force ret=pick:0/1/0xFFFFFFFF args=v  # mmdl_move: dz of dir
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # fldmmdl: MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # fldmmdl: MMDL position y (fx32)
  stub 0216dddc force ret=u32 args=v,v,v,v,v,v  # fldmmdl: MMDL at grid + height (sys, x, z, y, height, flag)
  stub 021a572c force ret=ptr:work:0x900:1 args=v  # player_grid

