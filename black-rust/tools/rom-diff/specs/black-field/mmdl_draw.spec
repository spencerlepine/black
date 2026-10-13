# fndiff spec: the MMDL draw-side glue at the start of F1 row 21.08's tail (overlay 21 Thumb code 0x02197C4C..0x02197D3C; F4b-spec1).
# Hand-written by the F4b-spec1 job; hand edits are fine, keep the conventions.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field mmdl_draw --per-function
# Set `field` (main 10 11 18 20 21).
#
# File: F1 puts 0x0219603C..0x02197D3A in `ircbattlemenu.c` (row 21.08), but that file's code ends at 0x02197C4A (`pop {r4-r6, pc}` +
# padding after the last ircbattlemenu.c function, whose __FILE__ literal is at 0x02197A9C). The four functions here are MMDL code:
# fldmmdl.c (overlay 10) calls 0x02197C4C (from 0216CD54, MMDLSYS draw delete) and 0x02197C78 (from 0216D3C6, the per-MMDL draw
# step); the blact / g3dobj draw side and field_player call the other two. They allocate nothing, so no __FILE__ string names their
# file: either the head of fldmmdl_acmd.c (whose __FILE__ literal is first used at 0x02197D98) or a small file of its own between
# ircbattlemenu.c and fldmmdl_acmd.c (.data order: "ircbattlemenu.c" 0x021DDBC4, then fldmmdl_acmd.c's .data 0x021DDBD4..0x021DDBEF).
# Logic only (D3): the draw procs and the blact / g3dobj systems they reach are `force` stubs.
#
# MMDLSYS / MMDL layouts: fldmmdl.spec header (crates/black-field/README.md "F4a-spec"). Every callee is outside the area (fldmmdl.c
# accessors in overlay 10, ported in the `fldmmdl` area; fldmmdl_blact / fldmmdl_g3dobj in overlay 21, U2), so every stub is
# `force` (overlay addresses collide with battle PORTED entries, see F2a-spec). No in-area calls: closure = plain.
area mmdl_draw
set field

# 02197C4C(sys): delete the draw systems of an MMDLSYS: if 0216D520(sys) (draw system +0x24) is set, 0219C5D0(sys) (fldmmdl_blact:
# delete the billboard-actor system); if 0216D528(sys) (+0x28, the 3D-object system) is set, 0219D488(sys) (fldmmdl_g3dobj delete);
# then 0216D95C(sys, 0) (draw side off).
fn 02197c4c ret=none inputs=100
  nowork
  arg u32 any
  stub 0216d520 force ret=pick:0/0x02E00800 args=v  # fldmmdl: MMDLSYS draw system (+0x24) or null
  stub 0219c5d0 force ret=none args=v  # fldmmdl_blact: delete the actor system (sys)
  stub 0216d528 force ret=pick:0/0x02E00900 args=v  # fldmmdl: MMDLSYS +0x28 (3D-object system) or null
  stub 0219d488 force ret=none args=v  # fldmmdl_g3dobj: delete (sys)
  stub 0216d95c force ret=none args=v,v  # fldmmdl: draw side on / off (sys, 0)

# 02197C78(m): the per-frame draw of one MMDL: unless the MMDLSYS flag 4 is set (0216D4D4(sys, 4)), and only once the draw side of
# the MMDL is ready (0216DAA0: move bit 2), call its draw proc (0216D758: draw table +4). Returns nothing (r0 is whatever the last
# callee left; the only caller, 0216D3B4, ignores it).
fn 02197c78 ret=none inputs=100
  nowork
  arg u32 any
  stub 0216d67c force ret=u32 args=v  # fldmmdl: MMDL -> MMDLSYS (+0x88)
  stub 0216d4d4 force ret=pick:0/4 args=v,v  # fldmmdl: MMDLSYS flags & mask (sys, 4)
  stub 0216daa0 force ret=pick:0/1 args=v  # fldmmdl: draw ready (move bit 2)
  stub 0216d758 force ret=none args=v  # fldmmdl: run the draw proc (draw table +4)

# 02197C9C(m): 1 when the MMDL's animation is held: paused (0216DB10: move bit 8) without an acmd set (0216DC4C: move bit 0x200),
# or status bit 0x10 (0216D55C(m, 0x10)); else 0.
fn 02197c9c ret=u32 inputs=100
  nowork
  arg u32 any
  stub 0216db10 force ret=pick:0/1 args=v  # fldmmdl: paused (move bit 8)
  stub 0216dc4c force ret=pick:0/1 args=v  # fldmmdl: acmd set (move bit 0x200)
  stub 0216d55c force ret=pick:0/0x10 args=v,v  # fldmmdl: status & mask (m, 0x10)

# 02197CCC(m, &out): the draw position: out (VecFx32) = position (+0x44, 0216D88C) + VecFx32 +0x50 (0216D8B0) + +0x5C (0216D8D0) +
# +0x68 (0216D8F0) + the s8 offsets << 12 (0216D92C), per component, wrapping (the five stack vectors are filled by the getters).
fn 02197ccc ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  stub 0216d88c force ret=none args=v,out:12  # fldmmdl: position VecFx32 (+0x44) -> out
  stub 0216d8b0 force ret=none args=v,out:12  # fldmmdl: VecFx32 +0x50 -> out
  stub 0216d8d0 force ret=none args=v,out:12  # fldmmdl: VecFx32 +0x5C -> out
  stub 0216d8f0 force ret=none args=v,out:12  # fldmmdl: VecFx32 +0x68 -> out
  stub 0216d92c force ret=none args=v,out:12  # fldmmdl: s8 offsets +0x7E..+0x80 << 12 -> out (s32[3])
