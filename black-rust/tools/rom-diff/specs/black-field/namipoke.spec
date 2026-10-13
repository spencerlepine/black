# fndiff spec: fldeff_namipoke.c (overlay 21 Thumb code 0x021ACA38..0x021AD758, F1 row 21.37; F3a-spec): the surf Pokemon field effect
# (state and position logic) and the splash effect of the surf start / end jumps.
# Run: tools/check-fn.sh black-field namipoke --per-function
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
# Render-only, not specced (D3, U2 rewrites them; callers stub them by address): 021ACA70 / 021AD410 (load the 3D models of the surf /
#   splash effects: fld3d 021C6AE0.. resource setup), 021ACB0C / 021AD530 (free them), 021ACCB0 / 021AD274 (draw the surf Pokemon: model
#   matrices, 0207C9D0 / 020520FC, fld3d 021C6658 / 021C6678), 021ACFD8 (free the surf animations), 021AD6A4 (free the splash animation),
#   021AD738 (splash draw position). F1 counted 4 render functions in the row.
# RESOURCE block (021ACA38: 0xC, 021AD3D4: 0x10): +0 field effect control, +4.. u16 model / animation handles (render side).
# SURF TASK work (field effect task work, 021AC1E0; init data 0x1C bytes from 021ACB38 copied to +0x1C by 021ACC3C): +9 u8 mode (0 off,
#   1 bobbing, 2 ridden: the MMDL draw offset follows 0216D8D0), +0xA u8 hide, +0xC u16 dir, +0x10 s32 bob height (0x400..0x4000, step
#   +0x14 = +-0x400), +0x18 u16 handle, +0x1C -> RESOURCE, +0x20 the MMDL, +0x38.. follower {u16 still, u16 dir (9 = none), VecFx32 pos,
#   VecFx32 last MMDL pos, u16 handle +0x54, VecFx32 scale +0x58}. The grid table 0x021D9184 (021ACC3C init, 021ACDC4 update, 021ACCAC
#   draw) and the rail table 0x021D915C (021ACFF8 update) differ in the follower maths (rail direction vectors 0219E784).
# SPLASH TASK work (021AD5E0, init data 0x10 from 021AD564 / 021AD598): +0..+8 offset (ROM 0x021D91F8[kind]), +0xC done, +0x10 u16
#   animation, +0x14 delete when done, +0x18 -> RESOURCE, +0x1C kind 0..4, +0x20 parent task (0 = fixed position).
# Arena (the `work` image): field effect control work+0 (value), task work work+0x1800, init data work+0x1C00 (+0 -> RESOURCE),
#   RESOURCE work+0x1E00, the MMDL work+0x900 (value).
# Coverage (plain): 943/943 instructions; closure: 941/943.
area namipoke
set field

# 021ACA38(ctrl, heap) -> res: Heap_AllocDebug(heap, 0xC, 1, "fldeff_namipoke.c", 0xAC); +0 ctrl; 021ACA70(res) (model load,
# render)
fn 021aca38 ret=u32 inputs=100
  arg u32 any
  arg u32 any
  stub 02030734 force ret=ptr:work:0x1E00:1 args=v,v,v,v,v  # the resource block
  stub 021aca70 force ret=none args=v  # namipoke render: load the surf Pokemon models (res) (G8/U2, not specced)

# 021ACA5C(ctrl, res): 021ACB0C(res) (model free, render); Heap_Free(res)
fn 021aca5c ret=none inputs=100
  arg u32 any
  arg u32 any
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021acb0c force ret=none args=v  # namipoke render: free the models (res) (not specced)

# 021ACB38(ctrl, dir, &pos, m, flag) -> task: init {021ABEA0(ctrl, 3) (resource), m, u16 OBJ id, u16 zone, u16 dir};
# MMDL +0 & 0x2000 (rail): 021ABFA8(ctrl, ROM 0x021D915C, &pos, flag, &init, 0); else ROM 0x021D9184 (grid effect table)
fn 021acb38 ret=u32 inputs=100
  arg u32 any
  arg u32 alt range 0 3 || any
  arg work 0xA00
  arg work 0x900
  arg u32 any
  stub 021abfa8 force ret=u32 args=v,v,in:12,v,in:14,v  # field effect: add task (ctrl, ROM table, &pos, prio, &init, flag)
  stub 0216d55c force ret=pick:0/1 args=v,v  # fldmmdl: MMDL +0 & v
  stub 0216d588 force ret=pick:0/1/2/0xff args=v  # fldmmdl: MMDL OBJ id
  stub 0216d590 force ret=pick:0x14d/0x14e/0xe7/0/1 args=v  # fldmmdl: MMDL zone id
  stub 021abea0 force ret=u32 args=v,v  # field effect: work of an effect id (ctrl, id)

# 021ACBB8(task, mode): task != 0: 021AC1E0(task) +9 = mode (u8: 0 off, 1 bob, 2 ..)
fn 021acbb8 ret=none inputs=100
  arg u32 any
  arg u32 alt range 0 2 || any
  stub 021ac1e0 force ret=ptr:work:0x1800:1 args=v  # field effect: task work

# 021ACBC8(task, on): task != 0: work +0xA = on == 1 ? 0 : 1
fn 021acbc8 ret=none inputs=100
  arg u32 any
  arg u32 alt range 0 1 || any
  stub 021ac1e0 force ret=ptr:work:0x1800:1 args=v  # field effect: task work

# 021ACBE4(d) -> min(d, 0x40000) / 0xA3D * 0x28 (s32 division)
fn 021acbe4 ret=u32 inputs=100
  nowork
  arg s32 alt range 0 0x50000 || any

# 021ACC04(&a, &b, dir) -> dir 0: b.z - a.z, 1: a.z - b.z, 2: b.x - a.x, 3: a.x - b.x; else 0
fn 021acc04 ret=u32 inputs=100
  nowork
  region a 0x10
  region b 0x10
  arg region a
  arg region b
  arg u32 alt range 0 3 || any

# 021ACC3C(task, w): init proc: i = 021AC1AC(task) (init data); w +0x1C.. = i (0x1C bytes); 021AC1CC(task, &i +0x10) (pos);
# w +0xC = i +0xC (dir); 021ABE18(*(w +0x1C)); w +0x18 = res +8; w +9 = 021AC1A0(task); w +0x14 = 0x400, +0x10 = 0x1000; w +0x54 =
# res +0xA; w +0x38 = 1, +0x3A = 9, +0x58 / +0x5C / +0x60 = 0x1000; 021AC178(task)
fn 021acc3c ret=none inputs=100
  arg u32 any
  arg work 0x1800
  fix work 0x1C00 u32 addrof work 0x1E00
  fix work 0x1E00 u32 any
  stub 021abe18 force ret=u32 args=v  # field effect: 3D resource system of a control
  stub 021ac178 force ret=none args=v  # field effect: task update (task)
  stub 021ac1a0 force ret=u32 args=v  # field effect: task +param (task)
  stub 021ac1ac force ret=ptr:work:0x1C00:1 args=v  # field effect: task init data (task)
  stub 021ac1cc force ret=none args=v,in:12  # field effect: task position = *v

# 021ACCAC(task, w): nothing (draw proc of the grid table)
fn 021accac ret=none inputs=100
  arg u32 any
  arg work 0x1800

# 021ACDC4(task, w): update (grid): w +9 == 0: 021ACCB0 (draw); else w +0xC = MMDL display dir; mode 1: bob w +0x10 += +0x14,
# clamped to 0x400..0x4000 (step negated at a bound); offset (0, bob + 0x1000, 0x5000), (+0x8000 more for OBJ codes 0xA0 / 0xA1) ->
# 0216D8E0(m, &off); mode 2: 0216D8D0(m, &off); task position = 02197CCC (MMDL draw position) - off (mode != 2: y += bob - 0x1000);
# mode 2 or +0xA == 1: +0x3A = 9 (+0x38 = 1 if +0xA); else the follower (w +0x38..) tracks the MMDL: dir changed: copy the
# position; same position: approach by 0x4000 per frame along dir; distance 021ACC04 >= 0x40000: snap; speed 021ACBE4 < 40: +0x38 = 1
# else +0x38 = 0 and the scale +0x58.. = speed; then 021ACCB0 (draw)
fn 021acdc4 ret=none inputs=600
  arg u32 any
  arg work 0x1800
  fix work 0x1809 u8 alt range 0 2 || any
  fix work 0x180A u8 range 0 1
  fix work 0x180C u16 range 0 3
  fix work 0x1810 u32 alt range 0x400 0x4000 || any
  fix work 0x1814 u32 alt oneof 0x100 0xFFFFFF00 0x400 || any
  fix work 0x181C u32 addrof work 0x1E00
  fix work 0x1820 u32 addrof work 0x900
  fix work 0x1838 u16 range 0 1
  fix work 0x183A u16 alt range 0 3 || const 9 || any
  fix work 0x183C u32 alt range 0 0x7FFFF || any
  fix work 0x1840 u32 alt range 0 0x7FFFF || any
  fix work 0x1844 u32 alt range 0 0x7FFFF || any
  fix work 0x1848 u32 alt range 0 0x7FFFF || any
  fix work 0x184C u32 alt range 0 0x7FFFF || any
  fix work 0x1850 u32 alt range 0 0x7FFFF || any
  fix work 0x183A u16 alt range 0 3 || const 9
  fix work 0x1848 u32 alt const 0 || const 0 || const 0 || any
  fix work 0x184C u32 alt const 0 || const 0 || const 0 || any
  fix work 0x1850 u32 alt const 0 || const 0 || const 0 || any
  stub 0216d88c force ret=none args=v,outm:12:0/0/0/0/0/0/0/0/0/0/0/0  # fldmmdl: MMDL position (zero: the follower test can match)
  stub 0216d598 force seq ret=pick:1/2/3/4/5/6/0x88/0x89/0x6d/0xa0/0xa1/0 args=v  # fldmmdl: MMDL OBJ code
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d8d0 force ret=none args=v,out:12  # fldmmdl: MMDL draw offset VecFx32 -> out
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 02197ccc force ret=none args=v,out:12  # fldmmdl_acmd: MMDL draw position -> out
  stub 021abe18 force ret=u32 args=v  # field effect: 3D resource system of a control
  stub 021ac1cc force ret=none args=v,in:12  # field effect: task position = *v
  stub 021acbe4 ret=pick:0/0x28/0x50/0x3e8 args=v  # in-area
  stub 021acc04 ret=pick:0/0x1000/0x40000/0xFFFFF000 args=in:12,in:12,v  # in-area
  stub 021accb0 force ret=none args=v,v  # namipoke render: draw (task, w) (not specced)
  stub 021c66ec force ret=none args=v,v,v  # fld3d: animation loop step (res, handle, 0x1000) (render)

# 021ACFF8(task, w): update (rail): as 021ACDC4 with the offsets along the rail direction vectors 0219E784 (normalised
# 0207CDB8): bob offset (-dx * 0x7000 >> 12, bob + 0x5000, -dz * 0x7000 >> 12); follower distance 0207CF3C
fn 021acff8 ret=none inputs=300
  arg u32 any
  arg work 0x1800
  fix work 0x1809 u8 alt range 0 2 || any
  fix work 0x180A u8 range 0 1
  fix work 0x180C u16 range 0 3
  fix work 0x1810 u32 alt range 0x400 0x4000 || any
  fix work 0x1814 u32 alt oneof 0x100 0xFFFFFF00 0x400 || any
  fix work 0x181C u32 addrof work 0x1E00
  fix work 0x1820 u32 addrof work 0x900
  fix work 0x1838 u16 range 0 1
  fix work 0x183A u16 alt range 0 3 || const 9 || any
  fix work 0x183C u32 alt range 0 0x7FFFF || any
  fix work 0x1840 u32 alt range 0 0x7FFFF || any
  fix work 0x1844 u32 alt range 0 0x7FFFF || any
  fix work 0x1848 u32 alt range 0 0x7FFFF || any
  fix work 0x184C u32 alt range 0 0x7FFFF || any
  fix work 0x1850 u32 alt range 0 0x7FFFF || any
  fix work 0x183A u16 alt range 0 3 || const 9
  fix work 0x1848 u32 alt const 0 || const 0 || const 0 || any
  fix work 0x184C u32 alt const 0 || const 0 || const 0 || any
  fix work 0x1850 u32 alt const 0 || const 0 || const 0 || any
  stub 0216d88c force ret=none args=v,outm:12:0/0/0/0/0/0/0/0/0/0/0/0  # fldmmdl: MMDL position (zero: the follower test can match)
  stub 0207cdb8 force ret=none args=in:6,out:6  # VecFx16 normalize (&in, &out) (math)
  stub 0207cf3c force ret=u32 args=in:12,in:12  # VEC_Distance(&a, &b) (math)
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v  # fldmmdl: MMDL display dir
  stub 0216d8e0 force ret=none args=v,in:12  # fldmmdl: MMDL draw offset VecFx32 = *v
  stub 02197ccc force ret=none args=v,out:12  # fldmmdl_acmd: MMDL draw position -> out
  stub 0219e784 force ret=none args=v,v,out:6  # rail: direction vector of dir (m, dir, &VecFx16)
  stub 021abe18 force ret=u32 args=v  # field effect: 3D resource system of a control
  stub 021ac1cc force ret=none args=v,in:12  # field effect: task position = *v
  stub 021acbe4 ret=pick:0/0x28/0x50/0x3e8 args=v  # in-area
  stub 021c66ec force ret=none args=v,v,v  # fld3d: animation loop step (res, handle, 0x1000) (render)

# 021AD3D4(ctrl, heap) -> res: Heap_AllocDebug(heap, 0x10, 1, "fldeff_namipoke.c", 0x3CF); +0 ctrl; 021AD410(res) (render)
fn 021ad3d4 ret=u32 inputs=100
  arg u32 any
  arg u32 any
  stub 02030734 force ret=ptr:work:0x1E00:1 args=v,v,v,v,v  # the resource block
  stub 021ad410 force ret=none args=v  # namipoke render: load the splash models (res) (not specced)

# 021AD3FC(ctrl, res): 021AD530(res) (render free); Heap_Free(res)
fn 021ad3fc ret=none inputs=100
  arg u32 any
  arg u32 any
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021ad530 force ret=none args=v  # namipoke render: free the splash models (res) (not specced)

# 021AD564(ctrl, kind, parent) -> task: init {0, 021ABEA0(ctrl, 9), kind, parent}; 021ABFA8(ctrl, ROM 0x021D9170, 0, 0,
# &init, 1) (the splash effect)
fn 021ad564 ret=u32 inputs=100
  arg u32 any
  arg u32 alt range 0 4 || any
  arg u32 any
  stub 021abfa8 force ret=u32 args=v,v,v,v,in:16,v  # field effect: add task (ctrl, ROM table, 0, 0, &init, 1)
  stub 021abea0 force ret=u32 args=v,v  # field effect: work of an effect id (ctrl, id)

# 021AD598(ctrl, kind, x) -> task: init {1, 021ABEA0(ctrl, 9), kind, 0}; 021ABFA8(ctrl, ROM 0x021D9170, x, 0, &init, 1)
fn 021ad598 ret=u32 inputs=100
  arg u32 any
  arg u32 alt range 0 4 || any
  arg u32 any
  stub 021abfa8 force ret=u32 args=v,v,v,v,in:16,v  # field effect: add task (ctrl, ROM table, x, 0, &init, 1)
  stub 021abea0 force ret=u32 args=v,v  # field effect: work of an effect id (ctrl, id)

# 021AD5CC(task) -> task != 0: 021AC1E0(task) +0xC (done); else 1
fn 021ad5cc ret=u32 inputs=100
  arg u32 any
  fix work 0x180C u32 alt range 0 1 || any
  stub 021ac1e0 force ret=ptr:work:0x1800:1 args=v  # field effect: task work

# 021AD5E0(task, w): splash init: w +0x14.. = 021AC1AC(task) (0x10 bytes); 021ABE18(*(w +0x18)); kind w +0x1C (0..4) picks
# the model handle res +4 / +6 / +0xA / +8 / +0xC -> w +0x10 = 021C6590(..) (animation); parent w +0x20 != 0: offset ROM 0x021D91F8
# [kind] (12 bytes) -> w +0..; pos = 021AC1B8(parent) + offset; 021AC1CC(task, &pos)
fn 021ad5e0 ret=none inputs=100
  arg u32 any
  arg work 0x1800
  fix work 0x1C04 u32 addrof work 0x1E00
  fix work 0x1C08 u32 range 0 4
  fix work 0x1C0C u32 alt const 0 || any
  stub 021c6590 force ret=u16 args=v,v,v,v  # fld3d: animation create (res, handle, 0, 0) (render)
  stub 021abe18 force ret=u32 args=v  # field effect: 3D resource system of a control
  stub 021ac1ac force ret=ptr:work:0x1C00:1 args=v  # field effect: task init data (task)
  stub 021ac1b8 force ret=none args=v,out:12  # field effect: task position -> out
  stub 021ac1cc force ret=none args=v,in:12  # field effect: task position = *v

# 021AD6B8(task, w): splash update: w +0xC == 1 and +0x14 == 1: 021AC154(task) (delete); else kind 0: w +0xC == 0 and
# 021C6694(..) == 0 (animation over): +0xC = 1; else 021C66EC(..) (loop); parent: follow its position + w offset
fn 021ad6b8 ret=none inputs=100
  arg u32 any
  arg work 0x1800
  fix work 0x180C u32 alt range 0 1 || any
  fix work 0x1814 u32 alt range 0 1 || any
  fix work 0x1818 u32 addrof work 0x1E00
  fix work 0x181C u32 range 0 4
  fix work 0x1820 u32 alt const 0 || any
  stub 021c6694 force ret=pick:0/1 args=v,v,v  # fld3d: animation step, done? (render)
  stub 021c66ec force ret=none args=v,v,v  # fld3d: animation loop step (render)
  stub 021abe18 force ret=u32 args=v  # field effect: 3D resource system of a control
  stub 021ac154 force ret=none args=v  # field effect: delete task
  stub 021ac1b8 force ret=none args=v,out:12  # field effect: task position -> out
  stub 021ac1cc force ret=none args=v,in:12  # field effect: task position = *v

