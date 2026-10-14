# fndiff spec (F13): the three GMEVENT functions at the end of F1's row 21.68 (`sodateya.c`, overlay 21 of the field set,
# 0x021C46F8-0x021C47AE). They are not day care code: two constructors and the proc of a GMEVENT that loads overlay 0x3F
# (63, linked at 0x021F3xxx, outside the field set) around a task that overlay runs on a field OBJ. Callers:
# fieldmap_event `FUN_0218B24C` -> 021C46F8(gs, fm, 1) (push down on a marked tile), script command 122 -> 021C474C(gs, fm,
# obj). Run: tools/check-fn.sh black-field ov63_event --per-function
#
# All blocks are `nowork`; the GMEVENT work that 020124E8 returns is an `arg buf 0x10` arena the function never reads as a
# register (`ptr:argK:0:1`). Every callee is a `force` stub: GMEVENT create 020122C0(gsys, parent, proc, work size) / work
# 020124E8(event), overlay load / unload 02034AC4 / 02034A5C (ARM, reached with `blx`), the player 02188CB4(fm), its OBJ
# 021A3074(player), 021A305C(player, x), and the overlay-63 entry points 021F36C0(gs, fm, obj, flag) (start, returns the
# task), 021F376C(task) (done?) and 021F3758(task) (free).
area ov63_event
set field

# FUN_021C46F8(gs, fm, x): the event for the player's OBJ (021A305C(player, x) first), flag 1. arg3 (r3, overwritten with
# the work size) is the event-work arena.
fn 021c46f8 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v
  stub 02034ac4 force ret=none args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 021a305c force ret=u32 args=v,v
  stub 021f36c0 force ret=u32 args=v,v,v,v

# FUN_021C474C(gs, fm, obj): the event for a given OBJ, flag 0. arg3 is the event-work arena.
fn 021c474c ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v
  stub 02034ac4 force ret=none args=v
  stub 021f36c0 force ret=u32 args=v,v,v,v

# FUN_021C4788(event, seq, work): the proc: once the task is done, free it, unload overlay 63, end (1); else 0.
fn 021c4788 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 021f376c force ret=pick:0/0/1/2 args=v
  stub 021f3758 force ret=none args=v
  stub 02034a5c force ret=none args=v
