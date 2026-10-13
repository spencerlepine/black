# fndiff spec: height_ex.c and the unanchored functions after it (overlay 21 Thumb code 0x021C0598..0x021C1024, F1 row 21.64; F3a-spec).
# Run: tools/check-fn.sh black-field height_ex --per-function
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
# HEIGHT_EX (8, Heap_AllocDebug(heap, 8, 0, "height_ex.c", 0x5A) in 021C05D4): +0 count, +4 -> records (count x 0x1C, line 0x5B): +0 tile x,
#   +4 tile z, +8 width, +0xC depth, +0x10 height value, +0x14 in use, +0x18 extra. Callers: fieldmap 0218CA3E (create) / 0218DDDC.. (lookup);
#   021C0628 (set a record) and 021C06C0 (set the height) have no caller in the field set.
# The rest of the row has no debug-name anchor (other source files): 021C06CC / 021C06E0 / 021C0780 a script-callback GMEVENT (callbacks
#   021C0808 / 080C / 0810 in ROM 0x021DA08C, handed to 020146F8); 021C0820 / 021C0844 / 021C0904 .. and 021C0AA0 / 021C0AC0 / 021C0B3C ..
#   two build-model animation GMEVENTs started by script commands (021B2B14 / 021B2ACC): models found around the player (0218C584), up to 6
#   created one by one (ROM offsets 0x021DA098 / 0x021DA0E0, SE 0x568) and animated; 021C0CA4 / 021C0CE4 / 021C0D08 / 021C0DDC / 021C0E4C /
#   021C0E98 the boulder push (strength): field_player 021A2F20 asks 021C0CA4 each step; a pushable MMDL in front (0216E5FC) whose
#   footprint (021C0D08: 0 floor, 1 blocked, 2 floor and holes, 3 holes only) is free starts the push event (work 0x24: +0 GRID, +4 dir,
#   +6 fall into a hole, +8 frame, +0xC target height, +0x10 draw offset, +0x14 shake, +0x1C boulder MMDL, +0x20 player MMDL).
# Coverage (plain): 1140/1140 instructions; closure: 1140/1140.
area height_ex
set field

# 021C0598(x, z, &rec) -> rec +0x14 (in use) and rec +0 <= x <= rec +0 + rec +8 - 1 and rec +4 <= z <= rec +4 + rec +0xC - 1
fn 021c0598 ret=u32 inputs=400
  arg s32 alt range 0 6 || range 0 6 || range 0 6 || any
  arg s32 alt range 0 6 || range 0 6 || range 0 6 || any
  arg work 0x1010
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
  fix work 0x1010 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1014 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x102C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1030 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1048 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x104C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1064 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1068 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1080 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1084 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x109C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x10A0 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1018 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x101C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1034 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1038 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1050 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1054 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x106C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1070 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1088 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x108C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A4 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A8 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1024 u32 alt const 1 || range 0 1 || any
  fix work 0x1040 u32 alt const 1 || range 0 1 || any
  fix work 0x105C u32 alt const 1 || range 0 1 || any
  fix work 0x1078 u32 alt const 1 || range 0 1 || any
  fix work 0x1094 u32 alt const 1 || range 0 1 || any
  fix work 0x10B0 u32 alt const 1 || range 0 1 || any

# 021C05D4(n, heap) -> hx: Heap_AllocDebug(heap, 8, 0, "height_ex.c", 0x5A); +4 = Heap_AllocDebug(heap, n * 0x1C, 0, .., 0x5B);
# +0 = n; every record +0x14 = 0 (u8 loop)
fn 021c05d4 ret=ofs:arg2 inputs=100
  nowork
  arg u32 range 0 8
  arg u32 any
  arg buf 0x400
  stub 02030734 force ret=seqptr:arg2:0:0x100 args=v,v,v,v,v  # Heap_AllocDebug (two blocks: hx, records)

# 021C0628(i, x, z, w, d, h, k, hx): i < n and record i not in use: x, z, w, d, +0x10 = h, +0x18 = k, +0x14 = 1
# (no caller in the field set)
fn 021c0628 ret=none inputs=100
  arg u32 range 0 7
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg work 0x1000
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010
  fix work 0x1010 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1014 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x102C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1030 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1048 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x104C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1064 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1068 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1080 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1084 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x109C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x10A0 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1018 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x101C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1034 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1038 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1050 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1054 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x106C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1070 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1088 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x108C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A4 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A8 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1024 u32 alt const 1 || range 0 1 || any
  fix work 0x1040 u32 alt const 1 || range 0 1 || any
  fix work 0x105C u32 alt const 1 || range 0 1 || any
  fix work 0x1078 u32 alt const 1 || range 0 1 || any
  fix work 0x1094 u32 alt const 1 || range 0 1 || any
  fix work 0x10B0 u32 alt const 1 || range 0 1 || any

# 021C0674(hx): Heap_Free(+4); Heap_Free(hx)
fn 021c0674 ret=none inputs=100
  arg work 0x1000
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 021C0688(hx) -> +0 (count)
fn 021c0688 ret=u32 inputs=100
  arg work 0x1000
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010

# 021C068C(x, z, hx, i) -> 021C0598(x, z, &rec[i]) != 0
fn 021c068c ret=u32 inputs=300
  arg s32 alt range 0 6 || range 0 6 || range 0 6 || any
  arg s32 alt range 0 6 || range 0 6 || range 0 6 || any
  arg work 0x1000
  arg u32 range 0 5
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010
  fix work 0x1010 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1014 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x102C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1030 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1048 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x104C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1064 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1068 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1080 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1084 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x109C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x10A0 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1018 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x101C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1034 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1038 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1050 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1054 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x106C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1070 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1088 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x108C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A4 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A8 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1024 u32 alt const 1 || range 0 1 || any
  fix work 0x1040 u32 alt const 1 || range 0 1 || any
  fix work 0x105C u32 alt const 1 || range 0 1 || any
  fix work 0x1078 u32 alt const 1 || range 0 1 || any
  fix work 0x1094 u32 alt const 1 || range 0 1 || any
  fix work 0x10B0 u32 alt const 1 || range 0 1 || any
  stub 021c0598 ret=pick:0/1 args=v,v,in:0x18  # in-area

# 021C06A8(i, hx) -> rec[i] +0x10 (height)
fn 021c06a8 ret=u32 inputs=100
  arg u32 range 0 5
  arg work 0x1000
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010
  fix work 0x1010 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1014 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x102C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1030 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1048 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x104C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1064 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1068 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1080 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1084 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x109C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x10A0 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1018 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x101C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1034 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1038 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1050 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1054 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x106C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1070 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1088 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x108C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A4 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A8 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1024 u32 alt const 1 || range 0 1 || any
  fix work 0x1040 u32 alt const 1 || range 0 1 || any
  fix work 0x105C u32 alt const 1 || range 0 1 || any
  fix work 0x1078 u32 alt const 1 || range 0 1 || any
  fix work 0x1094 u32 alt const 1 || range 0 1 || any
  fix work 0x10B0 u32 alt const 1 || range 0 1 || any

# 021C06B4(i, hx) -> rec[i] +0x18
fn 021c06b4 ret=u32 inputs=100
  arg u32 range 0 5
  arg work 0x1000
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010
  fix work 0x1010 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1014 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x102C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1030 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1048 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x104C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1064 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1068 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1080 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1084 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x109C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x10A0 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1018 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x101C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1034 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1038 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1050 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1054 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x106C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1070 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1088 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x108C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A4 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A8 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1024 u32 alt const 1 || range 0 1 || any
  fix work 0x1040 u32 alt const 1 || range 0 1 || any
  fix work 0x105C u32 alt const 1 || range 0 1 || any
  fix work 0x1078 u32 alt const 1 || range 0 1 || any
  fix work 0x1094 u32 alt const 1 || range 0 1 || any
  fix work 0x10B0 u32 alt const 1 || range 0 1 || any

# 021C06C0(i, v, hx): rec[i] +0x10 = v (no caller in the field set)
fn 021c06c0 ret=none inputs=100
  arg u32 range 0 5
  arg u32 any
  arg work 0x1000
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
  fix work 0x1000 u32 range 0 6
  fix work 0x1004 u32 addrof work 0x1010
  fix work 0x1010 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1014 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x102C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1030 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1048 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x104C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1064 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1068 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1080 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1084 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x109C u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x10A0 u32 alt range 0 3 || range 0 3 || range 0 3 || any
  fix work 0x1018 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x101C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1034 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1038 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1050 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1054 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x106C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1070 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1088 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x108C u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A4 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x10A8 u32 alt range 1 4 || range 1 4 || range 1 4 || any
  fix work 0x1024 u32 alt const 1 || range 0 1 || any
  fix work 0x1040 u32 alt const 1 || range 0 1 || any
  fix work 0x105C u32 alt const 1 || range 0 1 || any
  fix work 0x1078 u32 alt const 1 || range 0 1 || any
  fix work 0x1094 u32 alt const 1 || range 0 1 || any
  fix work 0x10B0 u32 alt const 1 || range 0 1 || any

# 021C06CC(gsys) -> 020122C0(gsys, 0, 021C06E1, 0) (GMEVENT)
fn 021c06cc ret=u32 inputs=100
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
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)

# 021C06E0(event, &seq) -> done: gsys = 020124E4(ev); fm = 0201210C(gsys); seq 0: 0216D9A0(MMDLSYS) (pause); mode
# 0202647C(02012138(gsys)) == 2 and 02012B70(GAMEDATA) != 1: child = 020122C0(gsys, 0, 021C0781, 0); else 0200FB38(), child =
# 020146F8(gsys, fm, 21, ROM 0x021DA08C (callbacks 021C0809 / 080D / 0811), gsys); 02012374(ev, child); seq 1; seq 1: fm != 0 and
# 02188BE8(fm) == 1: 0216D9E4(MMDLSYS) (resume), 1
fn 021c06e0 ret=u32 inputs=100
  arg u32 any
  arg work 0xBF0
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
  fix work 0xBF0 u32 alt range 0 1 || any
  stub 0201210c force ret=pick:0/0x02E00000 args=v  # GAMESYS -> FIELDMAP (or null)
  stub 0200fb38 force ret=u32 args=  # main: system object (no arguments)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012138 force ret=u32 args=v  # GAMESYS getter
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020124e4 force ret=u32 args=v  # GMEVENT -> GAMESYS
  stub 02012b70 force ret=pick:0/1/2 args=v  # GAMEDATA byte getter
  stub 020146f8 force ret=u32 args=v,v,v,v,v  # script event create (gsys, fieldmap, 21, ROM 0x021DA08C, gsys)
  stub 0202647c force ret=pick:0/1/2/3 args=v  # mode
  stub 0216d9a0 force ret=none args=v  # fldmmdl: MMDLSYS pause all
  stub 0216d9e4 force ret=none args=v  # fldmmdl: MMDLSYS resume all
  stub 02188be8 force ret=pick:0/1 args=v  # fieldmap predicate
  stub 02188c7c force ret=u32 args=v  # fieldmap +0x38 (MMDLSYS)

# 021C0780(event, &seq) -> done: seq 0: 0203CD98() != 0: 0203CEAC(0); seq 1; 1: 0203CDBC() == 1: 0200FBA0(),
# 02012124(gsys, 0), seq 2; 2: 020121CC(gsys) == 1: seq 3 else 1; 3: o = 02026474(1); o and 02026454(02012138(gsys)): 02164FC4(o), 1
fn 021c0780 ret=u32 inputs=100
  arg u32 any
  arg work 0xBF0
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
  fix work 0xBF0 u32 alt range 0 3 || any
  stub 0200fba0 force ret=none args=  # main (no arguments)
  stub 02012124 force ret=none args=v,v  # GAMESYS: set flag (gsys, 0)
  stub 02012138 force ret=u32 args=v  # GAMESYS getter
  stub 020121cc force ret=pick:0/1 args=v  # GAMESYS predicate
  stub 020124e4 force ret=u32 args=v  # GMEVENT -> GAMESYS
  stub 02026454 force ret=pick:0/1 args=v  # predicate
  stub 02026474 force ret=pick:0/0x02E00C80 args=v  # object or null (1)
  stub 0203cd98 force ret=pick:0/1 args=  # (no arguments)
  stub 0203cdbc force seq ret=pick:0/1 args=  # polled (no arguments)
  stub 0203ceac force ret=none args=v  # (0)
  stub 02164fc4 force ret=none args=v  # eventdata_tail: (obj)

# 021C0808(..) -> 1 (callback)
fn 021c0808 ret=u32 inputs=100
  nowork

# 021C080C(..) -> 1 (callback)
fn 021c080c ret=u32 inputs=100
  nowork

# 021C0810(a, b, gsys) -> 1: 02012124(gsys, 0) (callback)
fn 021c0810 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012124 force ret=none args=v,v  # GAMESYS: set flag (gsys, 0)

# 021C0820(gsys, parent, n) -> ev = 020122C0(gsys, parent, 021C0845, 0x38); 021C0904(020124E8(ev), gsys, n); ev
fn 021c0820 ret=u32 inputs=100
  arg u32 any
  arg u32 any
  arg u32 alt range 0 8 || any
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
  fix work 0xC14 u8 alt range 0 6 || any
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:work:0xC00:1 args=v  # GMEVENT work (event)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 0218c584 force ret=nullable:2:ptr:work:0xE00:1 args=v,in:16,v,out:4  # buildmodel: find models in a rect (map, &rect, kind, &n): list (u32 id) or null
  stub 0218c6d0 force ret=none args=v,v  # buildmodel: model position (id, &out)
  stub 0218dc6c force ret=u32 args=v  # map: buildmodel system (draw object)
  stub 021a3038 force ret=none args=v,out:12  # player
  stub 021c0904 ret=none args=v,v,v  # in-area

# 021C0844(event, &seq, w) -> done: 5 steps: 0: every 15 frames 021C09B0(w) (add a model), all added (+0x14 <= +0x15):
# step 1; 1: after 10 frames 021C0A10 (start animations), 02012374(ev, 02029830(GAMESYS, 0x514)); 2: 021C0A70 (animations done):
# 021C0A48; 3: 02012374(ev, 02029620(GAMESYS, 0, 6)); 4: 021C0988 (delete), 1; +0x16 += 1 (021C09A8 sets the step, +0x16 = 0)
fn 021c0844 ret=u32 inputs=100
  arg u32 any
  arg work 0xBF0
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
  fix work 0xBF0 u32 alt range 0 4 || any
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 02029620 force ret=u32 args=v,v,v  # event constructor (gsys, 0, 6)
  stub 02029830 force ret=u32 args=v,v  # event constructor (gsys, 0x514)
  stub 020520fc force ret=none args=v,v,v,out:0x24  # MTX_Rot(x, y, z, &mtx33) (math)
  stub 0207c9d0 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(&v, &m, &out) (math)
  stub 02188c80 force ret=u32 args=v  # fieldmap +4 (GAMESYS)
  stub 0218c69c force ret=none args=v,v,v,v  # buildmodel (map, id, 0, 0)
  stub 0218c7f0 force ret=u32 args=v,v,in:12  # buildmodel: create (map, 0x62, &pos)
  stub 0218c860 force ret=none args=v  # buildmodel: delete (obj)
  stub 0218c88c force ret=none args=v,v,v  # buildmodel: animation (obj, i, n)
  stub 0218c8fc force ret=pick:0/1 args=v,v  # buildmodel: animation done (obj, i)
  stub 021c0988 ret=none args=v  # in-area
  stub 021c09a8 ret=none args=v,v,v  # in-area
  stub 021c09b0 ret=none args=v  # in-area
  stub 021c0a10 ret=none args=v  # in-area
  stub 021c0a48 ret=none args=v  # in-area
  stub 021c0a70 ret=pick:0/1 args=v  # in-area

# 021C0904(w, gsys, n): fm = 0201210C(gsys); +0 heap, +4 fm, +0x14 = min(n, 6), +0x15 = 0, +0x18 = 0218DC6C(map), +0x34 = 0;
# rect +-5 tiles around the player (021A3038 position); list = 0218C584(bm, &rect, 3, &cnt); list: +0x34 = *list, 0218C6D0(*list,
# w +8) (model position); Heap_Free(list)
fn 021c0904 ret=none inputs=100
  arg work 0xC00
  arg u32 any
  arg u32 alt range 0 8 || any
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
  fix work 0xE00 u32 any
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 0218c584 force ret=nullable:2:ptr:work:0xE00:1 args=v,in:16,v,out:4  # buildmodel: find models in a rect (map, &rect, kind, &n): list (u32 id) or null
  stub 0218c6d0 force ret=none args=v,v  # buildmodel: model position (id, &out)
  stub 0218dc6c force ret=u32 args=v  # map: buildmodel system (draw object)
  stub 021a3038 force ret=none args=v,out:12  # player

# 021C0988(w): 0218C860(+0x1C[i]) for i < +0x14
fn 021c0988 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c860 force ret=none args=v  # buildmodel: delete (obj)

# 021C09A8(w, &seq, v): *seq = v; +0x16 = 0
fn 021c09a8 ret=none inputs=100
  arg work 0xC00
  arg work 0xBF0
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

# 021C09B0(w): i = +0x15 < +0x14: m = 020520FC rotation (0, 0, 0); 0207C9D0(ROM 0x021DA098[i] (12 bytes), m, &p);
# p += w +8 position? (the multiply writes p); +0x1C[i] = 0218C7F0(+0x18, 0x62, &p); +0x15 += 1; SE 0x568
fn 021c09b0 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020520fc force ret=none args=v,v,v,out:0x24  # MTX_Rot(x, y, z, &mtx33) (math)
  stub 0207c9d0 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(&v, &m, &out) (math)
  stub 0218c7f0 force ret=u32 args=v,v,in:12  # buildmodel: create (map, 0x62, &pos)

# 021C0A10(w): 0218C88C(+0x1C[i], 0, 0) for i < +0x15; +0x34 != 0: 0218C69C(+0x18, +0x34, 0, 0)
fn 021c0a10 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c69c force ret=none args=v,v,v,v  # buildmodel (map, id, 0, 0)
  stub 0218c88c force ret=none args=v,v,v  # buildmodel: animation (obj, i, n)

# 021C0A48(w): 0218C88C(+0x1C[i], 0, 3) for i < +0x15
fn 021c0a48 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c88c force ret=none args=v,v,v  # buildmodel: animation (obj, i, n)

# 021C0A70(w) -> 0218C8FC(+0x1C[i], 0) == 1 for every i < +0x15 (else 0)
fn 021c0a70 ret=u32 inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c8fc force ret=pick:0/1 args=v,v  # buildmodel: animation done (obj, i)

# 021C0AA0(gsys, parent) -> ev = 020122C0(gsys, parent, 021C0AC1, 0x3C); 021C0B3C(020124E8(ev), gsys); ev
fn 021c0aa0 ret=u32 inputs=100
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
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:work:0xC00:1 args=v  # GMEVENT work (event)
  stub 02012934 force ret=u32 args=v  # GAMEDATA getter
  stub 0201aa1c force ret=pick:0/1/2/3/6/7 args=v  # count (u8)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 0218c584 force ret=nullable:2:ptr:work:0xE00:1 args=v,in:16,v,out:4  # buildmodel: find models in a rect (map, &rect, kind, &n): list (u32 id) or null
  stub 0218c6d0 force ret=none args=v,v  # buildmodel: model position (id, &out)
  stub 0218c724 force ret=u32 args=v,v  # buildmodel (map, id)
  stub 0218dc6c force ret=u32 args=v  # map: buildmodel system (draw object)
  stub 021a3038 force ret=none args=v,out:12  # player
  stub 021c0b3c ret=none args=v,v  # in-area
  stub 021c0c94 ret=pick:0/1/2/3/6/7 args=v  # in-area

# 021C0AC0(event, &seq, w) -> done: 0: every 20 frames 021C0BE8 (add), all added: step 1; 1: after 10 frames 021C0C48;
# 2: 021C0C80 != 0: step 3; 3: 021C0BD8, 1
fn 021c0ac0 ret=u32 inputs=100
  arg u32 any
  arg work 0xBF0
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
  fix work 0xBF0 u32 alt range 0 3 || any
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020520fc force ret=none args=v,v,v,out:0x24  # MTX_Rot(x, y, z, &mtx33) (math)
  stub 0207c9d0 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(&v, &m, &out) (math)
  stub 0218c7f0 force ret=u32 args=v,v,in:12  # buildmodel: create (map, 0x62, &pos)
  stub 0218c860 force ret=none args=v  # buildmodel: delete (obj)
  stub 0218c88c force ret=none args=v,v,v  # buildmodel: animation (obj, i, n)
  stub 0218c8d8 force ret=pick:0/1 args=v  # buildmodel: animation done (obj)
  stub 021c0bd8 ret=none args=v  # in-area
  stub 021c0be8 ret=none args=v  # in-area
  stub 021c0c48 ret=none args=v  # in-area
  stub 021c0c80 ret=pick:0/1 args=v  # in-area

# 021C0B3C(w, gsys): as 021C0904 with n = 021C0C94(gsys) (party count), kind 8; +0x34 != 0: +0x38 = 0218C724(bm, +0x34)
fn 021c0b3c ret=none inputs=100
  arg work 0xC00
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
  fix work 0xE00 u32 any
  stub 0201210c force ret=u32 args=v  # GAMESYS -> FIELDMAP
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012934 force ret=u32 args=v  # GAMEDATA getter
  stub 0201aa1c force ret=pick:0/1/2/3/6/7 args=v  # count (u8)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 02188cb4 force ret=ptr:work:0x100:1 args=v  # fieldmap +0x8C (FIELD_PLAYER)
  stub 0218c584 force ret=nullable:2:ptr:work:0xE00:1 args=v,in:16,v,out:4  # buildmodel: find models in a rect (map, &rect, kind, &n): list (u32 id) or null
  stub 0218c6d0 force ret=none args=v,v  # buildmodel: model position (id, &out)
  stub 0218c724 force ret=u32 args=v,v  # buildmodel (map, id)
  stub 0218dc6c force ret=u32 args=v  # map: buildmodel system (draw object)
  stub 021a3038 force ret=none args=v,out:12  # player
  stub 021c0c94 ret=pick:0/1/2/3/6/7 args=v  # in-area

# 021C0BD8(w): +0x34 != 0: 0218C860(+0x38)
fn 021c0bd8 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c860 force ret=none args=v  # buildmodel: delete (obj)

# 021C0BE8(w): as 021C09B0 with the offsets ROM 0x021DA0E0
fn 021c0be8 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 020520fc force ret=none args=v,v,v,out:0x24  # MTX_Rot(x, y, z, &mtx33) (math)
  stub 0207c9d0 force ret=none args=in:12,in:0x24,out:12  # MTX_MultVec33(&v, &m, &out) (math)
  stub 0218c7f0 force ret=u32 args=v,v,in:12  # buildmodel: create (map, 0x62, &pos)

# 021C0C48(w): 0218C88C(+0x1C[i], 0, 2) for i < +0x15; +0x34 != 0: 0218C88C(+0x38, 0, 0)
fn 021c0c48 ret=none inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c88c force ret=none args=v,v,v  # buildmodel: animation (obj, i, n)

# 021C0C80(w) -> +0x34 != 0: 0218C8D8(+0x38); else 1
fn 021c0c80 ret=u32 inputs=100
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
  fix work 0xC14 u8 alt range 0 6 || any
  fix work 0xC15 u8 alt range 0 6 || any
  fix work 0xC16 u16 alt range 0 30 || any
  fix work 0xC34 u32 alt const 0 || any
  stub 0218c8d8 force ret=pick:0/1 args=v  # buildmodel: animation done (obj)

# 021C0C94(gsys) -> 0201AA1C(02012934(020120F4(gsys))) (count)
fn 021c0c94 ret=u32 inputs=100
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
  stub 02012934 force ret=u32 args=v  # GAMEDATA getter
  stub 0201aa1c force ret=pick:0/1/2/3/6/7 args=v  # count (u8)

# 021C0CA4(grid, a, keys, flags) -> event: core = 021A5728(grid); 021A572C(grid); dir = 021A3AB8(core, keys);
# 021A44BC(grid) == 0 (acmd busy): 0; else 021C0CE4(grid, dir, flags)
fn 021c0ca4 ret=u32 inputs=100
  arg work 0x400
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
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:work:0xC00:1 args=v  # GMEVENT work (event)
  stub 021639c0 force ret=pick:0/1/2/3/4/0x10 args=v,v  # mmdl_move: hit check in dir (m, dir): hit bits
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL size z (u8)
  stub 0216e5fc force ret=pick:0/1 args=v  # fldmmdl: OBJ code predicate (pushable boulder?)
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 021a395c force ret=u32 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3990 force ret=u32 args=v  # player_core
  stub 021a3ab8 force ret=pick:0/1/2/3/9 args=v,v  # player_core
  stub 021a448c force ret=pick:0/1/2 args=v,v  # player_grid
  stub 021a44bc force ret=pick:0/1 args=v  # player_grid
  stub 021a5728 force ret=ptr:work:0x200:1 args=v  # player_grid
  stub 021a572c force ret=ptr:work:0x900:1 args=v  # player_grid
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (map, &pos)
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021c0ce4 ret=pick:0/0x02E00C00 args=v,v,v  # in-area
  stub 021c0d08 ret=pick:0/1/2/3 args=v,v,v  # in-area
  stub 021c0ddc ret=pick:0/0x02E00C00 args=v,v,v  # in-area
  stub 021c0e4c ret=u32 args=v,v,v,v  # in-area
  stub 021c19b4 force ret=pick:0/0x02E00980 args=v,v  # player_event

# 021C0CE4(grid, dir, flags) -> k = 021A448C(grid, dir); flags bit 0: 021C0DDC(grid, dir, k); else 0
fn 021c0ce4 ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
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
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:work:0xC00:1 args=v  # GMEVENT work (event)
  stub 021639c0 force ret=pick:0/1/2/3/4/0x10 args=v,v  # mmdl_move: hit check in dir (m, dir): hit bits
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL size z (u8)
  stub 0216e5fc force ret=pick:0/1 args=v  # fldmmdl: OBJ code predicate (pushable boulder?)
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 021a395c force ret=u32 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3990 force ret=u32 args=v  # player_core
  stub 021a448c force ret=pick:0/1/2 args=v,v  # player_grid
  stub 021a5728 force ret=ptr:work:0x200:1 args=v  # player_grid
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (map, &pos)
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021c0d08 ret=pick:0/1/2/3 args=v,v,v  # in-area
  stub 021c0ddc ret=pick:0/0x02E00C00 args=v,v,v  # in-area
  stub 021c0e4c ret=u32 args=v,v,v,v  # in-area
  stub 021c19b4 force ret=pick:0/0x02E00980 args=v,v  # player_event

# 021C0D08(m, dir, map) -> boulder footprint: pos = MMDL position one step in dir; for the size x (0216D91C) x z (0216D924)
# tiles: a = 021AB0D4(map, &p); 021AB11C(021AB0F0(a))? (blocked): 021AB2DC(v) == 0: 1 (stop at once), else mark "hole"; else mark
# "floor"; -> 0, 2 (holes and floor), 3 (holes only)
fn 021c0d08 ret=u32 inputs=300
  arg work 0x900
  arg u32 alt range 0 3 || const 9 || any
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
  stub 021ab0d4 force seq ret=u32 args=v,in:12  # map attribute at a position (map, &pos) (seq: one per tile)
  stub 0216d91c force ret=pick:1/2/3/0 args=v  # fldmmdl: MMDL size x (u8)
  stub 0216d924 force ret=pick:2/1/3/0 args=v  # fldmmdl: MMDL size z (u8)
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)

# 021C0DDC(grid, dir, k) -> event: k == 1: m = 021C19B4(grid, dir) (MMDL in front); m and 0216E5FC(OBJ code) == 1
# (boulder): r = 021C0D08(m, dir, map); hit = 021639C0(m, dir) & ~1; hit bit 2 and r 2 / 3: clear it; hit == 0: 021C0E4C(grid, dir, m,
# r == 3); else 0
fn 021c0ddc ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
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
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:work:0xC00:1 args=v  # GMEVENT work (event)
  stub 021639c0 force ret=pick:0/1/2/3/4/0x10 args=v,v  # mmdl_move: hit check in dir (m, dir): hit bits
  stub 02164208 force ret=none args=v,in:12,v  # mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: MMDL position VecFx32 -> out
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL size z (u8)
  stub 0216e5fc force ret=pick:0/1 args=v  # fldmmdl: OBJ code predicate (pushable boulder?)
  stub 02188c98 force ret=u32 args=v  # fieldmap +0x48 (map draw object)
  stub 021a395c force ret=u32 args=v  # player_core
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3990 force ret=u32 args=v  # player_core
  stub 021a5728 force ret=ptr:work:0x200:1 args=v  # player_grid
  stub 021ab0d4 force ret=u32 args=v,in:12  # map attribute at a position (map, &pos)
  stub 021ab0f0 force ret=pick:0/1/2/0x10/0x20/0x40 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021ab2dc force ret=pick:0/1 args=v  # effect_encount: map attribute predicate (attribute value)
  stub 021c0d08 ret=pick:0/1/2/3 args=v,v,v  # in-area
  stub 021c0e4c ret=u32 args=v,v,v,v  # in-area
  stub 021c19b4 force ret=pick:0/0x02E00980 args=v,v  # player_event

# 021C0E4C(grid, dir, m, fall) -> ev = 020122C0(021A3990(core), 0, 021C0E99, 0x24); w = 020124E8(ev): memset 0x24;
# +0 grid, +4 dir, +6 fall, +0x1C m, +0x20 = 021A3960(core) (player MMDL); ev
fn 021c0e4c ret=u32 inputs=100
  arg work 0x400
  arg u32 alt range 0 3 || const 9 || any
  arg work 0x900
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
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:work:0xC00:1 args=v  # GMEVENT work (event)
  stub 021a3960 force ret=ptr:work:0x900:1 args=v  # player_core
  stub 021a3990 force ret=u32 args=v  # player_core
  stub 021a5728 force ret=ptr:work:0x200:1 args=v  # player_grid

# 021C0E98(event, &seq, w) -> done: boulder push: 0: both MMDLs settable (0216D3CC; the player also when 021A5408(grid)):
# acmd 02197E7C(dir, 8) on the boulder, 02197E7C(dir, 0x20) on the player, SE 0x674, step 1; 1: both done (0216D454): fall == 0:
# step 4; else height under the boulder (02163E58 -> w +0xC), y -= 0xE000 (OBJ code 0x6D) / 0x1A000, 0216DB4C(m, 1),
# 0216DAE0(m, 0), SE 0x675, step 2; 2: lower the boulder by 0x4000 per frame to w +0xC, then +0x14 = 0x2000, step 3; 3: shake:
# draw offset w +0x10.. = 0216D8E0, w +0x14 negated, +8 += 1, every 4th frame |w +0x14| -= 0x1000; 0: reset the position
# (0216E014(m, &pos, dir)), 0216D8E0, 0216E57C(m), step 4; 4: 1
fn 021c0e98 ret=u32 inputs=500
  arg u32 any
  arg work 0xBF0
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
  fix work 0xBF0 u32 alt range 0 4 || any
  fix work 0xC00 u32 addrof work 0x400
  fix work 0xC04 u16 alt range 0 3 || any
  fix work 0xC06 u16 range 0 1
  fix work 0xC08 u16 alt range 0 8 || any
  fix work 0xC0C u32 alt range 0 0x8000 || any
  fix work 0xC14 u32 alt oneof 0x2000 0xFFFFE000 0x1000 0 0xFFFFF000 || any
  fix work 0xC1C u32 addrof work 0x980
  fix work 0xC20 u32 addrof work 0x900
  stub 0216d88c force ret=none args=v,outm:12:0/0/0/0/0xFF/0x7F/0/0/0/0/0/0  # fldmmdl: MMDL position (y small)
  stub 020061e4 force ret=none args=v  # sound: play SE (id)
  stub 02163e58 force ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0x3/0x80  # mmdl_move: height at a position (m, &pos, &y)
  stub 0216d3cc force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd settable (m)
  stub 0216d410 force ret=none args=v,v  # fldmmdl: MMDL set acmd (m, code)
  stub 0216d454 force ret=pick:0/1 args=v  # fldmmdl: MMDL acmd done (m)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d89c force ret=none args=v,in:12  # fldmmdl: MMDL position VecFx32 = *v
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 0216dae0 force ret=none args=v,v  # fldmmdl: MMDL move bit set/clear (m, on)
  stub 0216db4c force ret=none args=v,v  # fldmmdl: MMDL dir lock (m, on)
  stub 0216e014 force ret=none args=v,in:12,v  # fldmmdl: MMDL set position + dir (m, &pos, dir)
  stub 0216e57c force ret=none args=v  # fldmmdl: MMDL refresh (m)
  stub 02197e7c force ret=pick:0/1/2/0x10/0x24/0x38 args=v,v  # fldmmdl_acmd: acmd code of (dir, base)
  stub 021a5408 force ret=pick:0/1 args=v  # player_grid

