# fndiff spec (P7): egg generation, overlay 21 of the field set (0x021C38F4-0x021C4570, the Pokemon-side
# logic of the day care; `sodateya.c` proper is F13). Run: tools/check-fn.sh black-pokemon egg --per-function
#
# Every block lists the stubs of its callees and of the callees of its in-area descendants (closure
# stubs), so the area can be ported in any order. In-area callees run as real code on both sides.
# Boundaries (stubbed, outside the area): the Pokemon accessors FUN_02017E1C (get) / FUN_02017E40 (put),
# the RNG FUN_02005728, the personal record allocator FUN_0201ADE4 / field reader FUN_0201AE1C / free
# FUN_020307B0 (FUN_0201AE14 is a thunk to it), the learnset reader FUN_0201ADD0, the archive readers
# FUN_02048F60 / FUN_02048F74, the item helpers FUN_02021288 / FUN_0202130C / FUN_020212B0, the profile
# readers FUN_02008530 / FUN_02008510, FUN_02017588 (init), FUN_02018A00 -> 02018A08 and FUN_02018198 ->
# 020181A0 (thunks, stubbed at their targets), the message / string routines and the PP / stat
# routines of the built Pokemon. FUN_02017ED8 (shininess) and MI_CpuFill8 run as real code.
#
# Records: `par` is the pair of parent handles (two u32), `egg` the 0x8C-byte work struct. Handles are
# few and fixed so both parents can be the same Pokemon (equal stub answers). The stub for the Pokemon
# getter answers from a short per-function list of the values the function compares with, because the
# stub cannot tell the field ids apart.
#
# The learnset readers FUN_021C4084 / FUN_021C4160 loop until a pair of 0xFFFF; a random stub buffer
# never has one, so the literal 0xFFFF (pool words 0x021C40F4 and 0x021C4234) is made the global
# `learnset_end` = 0 and the stub fills small halfwords (the Rust side reads the same global through
# EggHooks::learnset_end). The egg-move block (ARC 123) is a `buf` argument the stub returns a pointer into.
area egg
set field

# FUN_021C3A00(par): male first. Gender id 0x6E answers 0 / 1 / 2.
fn 021c3a00 ret=none inputs=600
  nowork
  region par 8
  arg region par
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0/1/2 args=v,v,v

# FUN_021C3A44(par): a Ditto first with a genderless second swaps.
fn 021c3a44 ret=none inputs=600
  nowork
  region par 8
  arg region par
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0/1/2/0x84/0x84 args=v,v,v

fn 021c3a70 ret=none inputs=100
  nowork
  region par 8
  arg region par
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200

fn 021c3bb0 ret=u32 inputs=300
  nowork
  region par 8
  arg region par
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0/1/5/0x84/0x84 args=v,v,v

fn 021c3c5c ret=u32 inputs=300
  nowork
  region par 8
  arg region par
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0/1/5/0x84/0x84 args=v,v,v

# FUN_021C3A7C(egg): clear 0x8C bytes (MI_CpuFill8 runs as real code), level = 1.
fn 021c3a7c ret=none inputs=100
  nowork
  region egg 140
  arg region egg

# FUN_021C3A90(par, egg): baby species of the base parent (FUN_021C4504 reads ARC 20 through the stub).
fn 021c3a90 ret=none inputs=400
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0/1/5/0x84/0x28/0x288/0x289/0x28a/0xffff/0x1f4 args=v,v,v
  stub 02048f74 ret=none args=out:2,v,v,v,v

fn 021c3aac ret=none inputs=500
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x1d/0x1d/0x20/0x21/0x22/0x23/0x1f/5/0x84 args=v,v,v
  stub 02005728 seq ret=lt:0 args=v

fn 021c3adc ret=none inputs=500
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x139/0x13a/0x138/0x13b/5/0x84 args=v,v,v
  stub 02005728 seq ret=lt:0 args=v

fn 021c3b0c ret=none inputs=300
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x1ea/0x1e9/0x1eb/5/0x84 args=v,v,v

fn 021c3b2c ret=none inputs=800
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0 u32 alt oneof 0x12a 0x168 0x1b7 0x1b6 0x1be 0x1ca 0x196 0x1b8 0x1b1 0x12a 0x168 0 || any
  stub 02017e1c ret=pick:0xfe/0xff/0x13a/0x13b/0x13c/0x13d/0x13e/0x13f/0x140/0/0x84/5 args=v,v,v

fn 021c3bcc ret=none inputs=100
  nowork
  region egg 140
  arg u32 any
  arg region egg

fn 021c3bd4 ret=none inputs=500
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x1a6/0x1a7/0x1a5/0x1a8/5/0x84/0x10/0x20 args=v,v,v

fn 021c3c04 ret=none inputs=500
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x19c/0x19d/0x19b/0x19e/5/0x84/0x10/0x20 args=v,v,v

fn 021c3c30 ret=none inputs=500
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x226/0x225/0x227/5/0x84/0x10/0x20 args=v,v,v

fn 021c3c78 ret=none inputs=200
  nowork
  region egg 140
  arg u32 any
  arg region egg
  stub 02005728 ret=lt:0 args=v

fn 021c3c88 ret=none inputs=800
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0xe5/0xe5/0xe5/0/1/3/7/24/0x84 args=v,v,v
  stub 02005728 seq ret=lt:0 args=v

fn 021c3d1c ret=none inputs=800
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  arg u32 any
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0/1/2/3/4/5/10/65 args=v,v,v
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0201ae1c ret=pick:0/1/2/3/4/5/10/65 args=v,v
  stub 020307b0 ret=none args=v
  stub 02005728 seq ret=lt:0 args=v

fn 021c3da8 ret=none inputs=400
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c ret=pick:0x84/0x84/1/5 args=v,v,v
  stub 02005728 seq ret=lt:0 args=v

fn 021c3de4 ret=none inputs=500
  nowork
  region egg 140
  arg u32 any
  arg region egg
  arg u32 any
  fix region:egg 0x4c u32 alt range 0 3 || any
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0201ae1c ret=pick:0/0/1/2/50/100/0xffff args=v,v
  stub 020307b0 ret=none args=v

fn 021c3e38 ret=none inputs=300
  nowork
  region egg 140
  arg region egg
  arg u32 any
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0201ae1c ret=pick:0/1/5/20/120/255 args=v,v
  stub 020307b0 ret=none args=v

fn 021c3e64 ret=none inputs=800
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0x58 u32 alt const 0 || const 1 rep=12,4
  fix region:egg 0x88 u32 range 0 4
  stub 02017e1c ret=pick:0x126/0x121/0x122/0x123/0x124/0x125/0/1/0xe5 args=v,v,v
  stub 02005728 seq ret=lt:0 args=v

fn 021c3f60 ret=none inputs=500
  nowork
  region egg 140
  arg u32 any
  arg region egg
  fix region:egg 0x58 u32 const 0 rep=12,4
  fix region:egg 0x58 u32 alt const 0 || const 1 rep=3,4
  fix region:egg 0x70 u32 alt const 0 || const 0 || const 1 rep=2,4
  fix region:egg 0x88 u32 alt const 0 || const 0 || range 0 3 || const 4
  stub 02005728 seq ret=lt:0 args=v

fn 021c3fc0 ret=none inputs=400
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0x58 u32 alt const 0 || const 0 || const 1 rep=12,4
  stub 02017e1c ret=pick:1/2/3/4/5/6/7/8/9/10/31/0x1f args=v,v,v

fn 021c4018 ret=none inputs=100
  nowork
  region egg 140
  arg u32 any
  arg region egg
  stub 02008530 ret=u32 args=v

fn 021c4024 ret=none inputs=100
  nowork
  region egg 140
  arg u32 any
  arg region egg
  stub 02005728 ret=lt:0 args=v

fn 021c4078 ret=u32 inputs=50
  nowork
  stub 02005728 ret=lt:0 args=v

fn 021c4030 ret=none inputs=800
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0x30 u32 alt const 0 || const 0 || any
  fix region:egg 0x34 u32 alt any || range 0 15 || const 0x10005
  stub 02017e1c ret=pick:0/0/1/2/3/4/5 args=v,v,v
  stub 02005728 seq ret=pick:0/5/0x10000/0xffff0000/0x12345678/7/0x30002 args=v

fn 021c4084 ret=none inputs=600
  nowork
  region egg 140
  global 021c40f4 4
  arg region egg
  fix region:egg 0x38 u32 range 0 4
  fix region:egg 0x3c u32 alt range 0 3 rep=4,4
  fix region:egg 8 u32 range 0 2
  fix global0 0 u32 const 0
  stub 0201add0 ret=none args=v,v,outm:0x68:0x01/0/0x03/0

# FUN_021C40F8(par, egg, heap): ARC 123 member (stub) = a count then moves; the third argument is
# not read, the 4th is the block the stub returns a pointer into.
fn 021c40f8 ret=none inputs=800
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  arg u32 any
  arg buf 0x40
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0x38 u32 range 0 4
  fix region:egg 0x3c u32 alt range 0 5 rep=4,4
  fix arg3 0 u16 alt range 0 12 || range 0 3
  fix arg3 2 u16 range 0 5 rep=24,2
  stub 02017e1c ret=pick:0/1/2/3/4/5 args=v,v,v
  stub 02048f60 ret=ptr:arg3:0:1 args=v,v,v
  stub 020307b0 ret=none args=v

fn 021c4160 ret=none inputs=800
  nowork
  region par 8
  region egg 140
  global 021c4234 4
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0x38 u32 range 0 4
  fix region:egg 0x3c u32 alt range 0 3 rep=4,4
  fix global0 0 u32 const 0
  stub 02017e1c ret=pick:0/1/2/3/4/5 args=v,v,v
  stub 0201add0 ret=none args=v,v,outm:0x68:0x01/0/0/0

fn 021c4238 ret=none inputs=300
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  arg u32 any
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0x38 u32 range 0 4
  fix region:egg 0x3c u32 alt range 0 7 rep=4,4
  fix region:egg 0 u32 alt range 0 649 || range 0x280 0x290
  fix region:egg 4 u32 any
  stub 02017e1c ret=pick:0/1/2/3/4/5/6/7 args=v,v,v
  stub 020212b0 ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 02021288 ret=pick:0/1/1/1/1 args=v
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0202130c ret=pick:0/5/31/32/33/63/64/95/96/100/255 args=v
  stub 0201ae1c ret=pick:0xffffffff/0/1/0x80000000/0x10/0x7fffffff/0xaaaaaaaa args=v,v
  stub 020307b0 ret=none args=v

fn 021c42bc ret=none inputs=500
  nowork
  region par 8
  region egg 140
  arg region par
  arg region egg
  fix region:par 0 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:par 4 u32 oneof 0x02300000 0x02300100 0x02300200
  fix region:egg 0 u32 alt const 0xac || const 0xac || any
  fix region:egg 0x38 u32 range 0 4
  fix region:egg 0x3c u32 alt range 0 0x160 rep=4,4
  stub 02017e1c ret=pick:0xec/0xec/0/1/0xe5 args=v,v,v

fn 021c42f4 ret=none inputs=500
  nowork
  region egg 140
  arg region egg
  arg u32 any
  fix region:egg 0x38 u32 range 0 4
  fix region:egg 0x3c u32 alt range 0 6 rep=4,4

# FUN_021C4340(mon, egg, profile, place, heap): the heap id is the stack argument.
fn 021c4340 ret=none inputs=500
  nowork
  region egg 140
  arg u32 any
  arg region egg
  arg u32 any
  arg u32 any
  arg u32 any
  fix region:egg 0x58 u32 alt const 0 || const 0 || const 1 rep=12,4
  fix region:egg 0x4c u32 alt range 0 3 || any
  stub 02017588 ret=none args=v,v,v,v,v,v,v,v
  stub 02017e40 ret=none args=v,v,v
  stub 02018a08 ret=none args=v,v,v
  stub 02008510 ret=u32 args=v,v
  stub 02045808 ret=none args=v
  stub 02045ec0 ret=u32 args=p,v
  stub 0202f190 ret=none args=v,v,v,v,v
  stub 020181a0 ret=none args=v
  stub 020185c4 ret=none args=v
  stub 0201b9f0 ret=none args=v

fn 021c447c ret=u32 inputs=600
  nowork
  arg u32 any
  arg u32 alt range 0 0x290 || any
  arg u32 any
  arg u32 alt range 0 0xffff || any
  stub 0201ade4 ret=u32 args=v,v,v
  stub 02021288 ret=pick:0/1/1/1 args=v
  stub 0202130c ret=pick:0/5/31/32/33/63/64/95/96/100/255 args=v
  stub 0201ae1c ret=pick:0xffffffff/0/1/0x80000000/0x10/0x7fffffff/0xaaaaaaaa args=v,v
  stub 020307b0 ret=none args=v

fn 021c4504 ret=u32 inputs=300
  nowork
  arg u32 alt range 0 0x290 || any
  stub 02048f74 ret=none args=out:2,v,v,v,v

fn 021c452c ret=u8 inputs=500
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0201ae1c ret=pick:0/1/2/3/4/5/10/65 args=v,v
  stub 020307b0 ret=none args=v

# FUN_021C38F4(heap, profile, place, parent0, parent1, mon): the whole pipeline; `mon` is only
# passed on. The 7th argument is the egg-move block of the stub (not read by the function).
fn 021c38f4 ret=none inputs=500
  nowork
  global 021c40f4 4
  global 021c4234 4
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 oneof 0x02300000 0x02300100 0x02300200
  arg u32 oneof 0x02300000 0x02300100 0x02300200
  arg u32 any
  arg buf 0x40
  fix global0 0 u32 const 0
  fix global1 0 u32 const 0
  fix arg6 0 u16 range 0 12
  fix arg6 2 u16 range 0 5 rep=24,2
  stub 02017e1c ret=pick:0/1/2/3/5/0x84/0xe5/0xec/0x1d/0x20/0x139/0x1ea/0xac/0x1a6/0x19c/0x226/0x126/0x121/0x12a/0xfe args=v,v,v
  stub 02017e40 ret=none args=v,v,v
  stub 02005728 seq ret=lt:0 args=v
  stub 0201ade4 ret=u32 args=v,v,v
  stub 0201ae1c ret=pick:0/1/2/3/4/5/10/65/0xffffffff args=v,v
  stub 020307b0 ret=none args=v
  stub 0201add0 ret=none args=v,v,outm:0x68:0x01/0/0/0
  stub 02048f60 ret=ptr:arg6:0:1 args=v,v,v
  stub 02048f74 ret=none args=out:2,v,v,v,v
  stub 020212b0 ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 02021288 ret=pick:0/1/1/1 args=v
  stub 0202130c ret=pick:0/5/31/32/33/63/64/95/96/100/255 args=v
  stub 02008530 ret=u32 args=v
  stub 02017588 ret=none args=v,v,v,v,v,v,v,v
  stub 02018a08 ret=none args=v,v,v
  stub 02008510 ret=u32 args=v,v
  stub 02045808 ret=none args=v
  stub 02045ec0 ret=u32 args=p,v
  stub 0202f190 ret=none args=v,v,v,v,v
  stub 020181a0 ret=none args=v
  stub 020185c4 ret=none args=v
  stub 0201b9f0 ret=none args=v
