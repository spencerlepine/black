# svflow spec: ai1 (overlay 96, trainer AI command interpreter `tr_ai.c`, 0x02187F00-0x0218A574; B14-spec).
# Harness-job owned; porting workers may not edit. Grammar: svflow_specs/README.md.
#
# Overlay 96 is ALL trainer AI (no other parts). It is a bytecode VM running on the generic script engine of
# ARM9 main (`black-script` vm.c, 0x020111EC..; the step is 0x02011298). 02187f00 creates the AI object (0xD8
# bytes) and its VM; 160 real functions follow: the command handlers of a 120-entry dispatch table at
# 0x0218A5A8 (indexed by the VM), their per-slot wrappers, and ~20 shared helpers. Veneers / `.byte` fragments
# that Ghidra and the asm split off are not specced (37 of 197 entries). File naming per the decomp.
#
# One area `ai1` for the whole overlay: a DP over the 189 in-range call edges shows at most ~13 edges cross for
# any balanced cut (the three dispatchers 0218a0f4 / 0218a160 / 0218a2fc are called from almost every handler),
# so a single area has zero internal crossing calls and is the minimal-crossing split.
#
# Objects (from the disassembly):
#   VM (black-script vm.c, 0x530 bytes; `arg buf 0x600`): the AI object stores it at ai+0xA0.. and runs it; the
#     command handlers get it as r0 and read script operands through 0201134c(vm) (next u32) / set the PC with
#     020113b0(vm, ptr).  Here it is a random buffer the stubs read from.
#   AI object (ai, 0xD8 bytes; `arg buf 0x100`, the handlers' r1): the trainer-AI work struct.
#     +0x00 u8 phase (021885bc state loop)   +0x01 u8 move slot index   +0x02 u16 score of the slot
#     +0x24 + 2*i s16[?] per-slot score      +0x30 + i s8[?] chosen slot per rank   +0x38 u32 handler result
#     +0x3C u32 working mask (copy of +0x40)  +0x40 u32 active-slot mask   +0x44 u8 flags (b0 running, b1=2 abort,
#       b4=0x10 done)   +0x45 u8   +0x46 u8 chosen move slot   +0x47 u8 chosen target slot   +0x48 u8[4] priority
#     +0x4C + 8*slot + 2*i u16[4][4] a per-slot table (0218a324 fills it)   +0x94 u8 own position   +0x95 u8 iter
#       position   +0x98 u32 kind (1 or 2)   +0x9C u32   +0xA0 u16 heap handle   +0xA4 / +0xA8 ptr (020490f4 /
#       02020f14 results)   +0xB0 main module M (021b8588 arg)   +0xB4 pokecon/flow (021c7e14 arg)   +0xB8 setup
#       (021b98ac arg via 0218a2fc)   +0xBC self BattleMon*   +0xC0 target BattleMon*   +0xC4 u32 status (handler
#       return value: 0 continue, 1 yield)   +0xC8 u32 = 4   +0xCC u32   +0xD0/+0xD4 u64 time (020870d0).
#   Dispatchers: 0218a0f4(vm, kind<=7, a, b, [stack c]) compares a/b per `kind` and, if true, sets a VM reg via
#     020113b0; returns the bool.  0218a160(ai, slot<=3) -> battle position.  0218a2fc(ai) -> self BattleMon*
#     (tail call 021b98ac).  0218a378/0218a384(ai, waza, id) -> move-data field (tail 0201bd28 / 0201beb8).
#
# Closure stubs (core4 convention): each block lists the stubs of its callees and of the callees of all its
# in-area descendants, so porting in any order needs no spec edit (checked by running every function with the
# whole area treated as ported: no fault, stray write or unstubbed callee; `svflow.py ai1 --all --strict-callees`
# is clean).  Every callee of ov96 is outside the area (ARM9 main, ov93 battle server / BattleMon / view), so all
# stubs are plain (none become `force`; nothing in another svflow area calls into here).  Loop-bound and
# switch-selector callees return bounded `pick:` values.  Pure helper 0209c054 (64-bit multiply) runs as real
# code (svflow ALLOW).  Thunks `ldr r3,[pc]; bx r3` are stubbed at their target (0218a2fc->021b98ac,
# 0218a308->021b9c00, 0218a378->0201bd28, 0218a384->0201beb8).  020870d0 (OS tick) is `ret=ticks:100` (Z21: the
# n-th read is n * 100 in r0:r1, so the 500-tick budget of 021885bc runs out on the 6th poll after the start read).
#
# Z21 (strength, see svflow_specs/README.md "Z21 additions"): the script operands of every command handler are
# `0201134c ... ret=nth:K0;K1;..` (slot 0..4, compared values around the getters' answers, list / table offsets, jump
# offsets any u32); the AI-object fields that thresholds and indices read are biased (`# Z21 biases` lines; positions
# 0..5); getters whose answers are compared answer from `pick:` sets around the compared constants; BattleMon getters
# log the mon (021d59b4 `v,v`; 021d57fc / 021d5b7c / 021d5b68 `v`; the thunk 021c81c8 `p,v,v`).  Blocks fixed for
# registration: 02188b1c / 02188b64 (a -1-terminated list at arg0+0x300, operand a = 0x100 / 0x108 / 0x110),
# 021895c4 (operand a 0..2), 02187f00 (02020f14 returns a value; the global table word is a `global`; the two
# allocations are disjoint `seqptr` blocks), 0218878c (slot ai[1] 0..3), 0218a324 (ai[0x95] bounded), 021885bc (the
# global table and its arena), 02187fb0 (021d5b68 answers keep the third candidate alive: no candidate = garbage index).
#
# ROM-side instruction coverage and smoke ports: see the B14-spec report.
area ai1
set battle+96

fn 02187f00 ret=u32
  nowork
  arg buf 0x600
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 range 1 8
  arg buf 0x100
  arg buf 0x600
  fix arg0 0x14 u32 addrof arg0 0x200
  global 0218a7c0 4                               # Z21: the global AI table word 0218a41c sets
  fix global0 0 u32 alt const 0 || addrof arg5 0xE0
  stub 0218a41c ret=u32 args=v,p
  stub 0201121c force ret=ptr:arg6:0:1 args=v,p
  stub 0201126c force ret=none args=p,v
  stub 0201c328 force ret=none args=v,v
  stub 0201c408 force ret=pick:0/1
  stub 02020f14 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg5:0:0xE0 args=v,v,v,v,v   # Z21: AI object at arg5+0, table at arg5+0xE0
  stub 020490f4 force ret=u32 args=v,v
  stub 021b8588 force ret=u32 args=p
  stub 021b86b8 force ret=u32 args=p

fn 02187fb0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg buf 0x20
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 range 0 3
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg1 0x00 u8 alt range 0 2 || any
  fix arg1 0x04 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x14 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x3C u32 alt range 0 15 || any
  fix arg1 0x40 u32 alt range 0 15 || any
  fix arg1 0x44 u8 alt mask 0x0F || any
  fix arg1 0xD0 u32 alt range 0 700 || any
  fix arg1 0xD4 u32 alt const 0 || any
  fix arg2 0 u8 alt range 0 3 || const 0xFF rep=4,1
  fix arg2 4 u8 alt range 1 2 || any rep=4,1
  global 0218a7c0 4
  fix global0 0 u32 addrof arg2
  stub 0218814c ret=u32 args=p,p,p
  stub 02188284 ret=u32 args=p,p,p
  stub 021885bc ret=u32 args=p,p
  stub 0218a2fc ret=u32 args=p
  stub 0218a310 ret=u32 args=p
  stub 0218a324 ret=u32 args=p
  stub 0218a378 ret=u32 args=p,v,v
  stub 0218a384 ret=u32 args=p,v,v
  stub 0218a494 ret=u32 args=p,p
  stub 0218a528 ret=u32 args=p,p
  stub 0201127c force ret=none args=p,v
  stub 02011290 force ret=none args=p
  stub 02011298 force seq ret=pick:0/1 args=p
  stub 02011324 force ret=ptr:arg1:0:1 args=p
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 0201beb8 force ret=pick:0/1/2 args=v,v
  stub 020307b0 force ret=none args=p
  stub 0203f0a8 force seq ret=u32
  stub 02049250 force ret=u32 args=v,v,v
  stub 020870d0 force ret=ticks:100   # Z21: a clock advancing 100 per read (the 500-tick budget of 021885bc)
  stub 021b8ce8 force ret=pick:0/1 args=p,v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021b9a6c force ret=pick:0/1 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5584 force ret=pick:0/1 args=p,v
  stub 021d5b68 force ret=nth:pick:0/1/2;pick:0/1/2;pick:0/2 args=v   # Z21: the 3rd candidate of 02187fb0 lives (none -> garbage index)
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 02188080 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a458 ret=u32 args=p
  stub 02011264 force ret=none args=p
  stub 02011324 force ret=ptr:arg1:0:1 args=p
  stub 0201c3c0 force ret=none
  stub 0201c408 force ret=pick:0/1
  stub 020307b0 force ret=none args=p
  stub 02049238 force ret=none args=p

fn 021880bc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02011324 force ret=ptr:arg1:0:1 args=p
  stub 0203f0a8 force seq ret=u32

fn 02188134 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02011324 force ret=ptr:arg1:0:1 args=p

fn 02188140 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02011324 force ret=ptr:arg1:0:1 args=p

fn 0218814c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg buf 0x20
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 range 0 3
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg1 0x00 u8 alt range 0 2 || any
  fix arg1 0x04 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x14 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x3C u32 alt range 0 15 || any
  fix arg1 0x40 u32 alt range 0 15 || any
  fix arg1 0x44 u8 alt mask 0x0F || any
  fix arg1 0xD0 u32 alt range 0 700 || any
  fix arg1 0xD4 u32 alt const 0 || any
  fix arg2 0 u8 alt range 0 3 || const 0xFF rep=4,1
  fix arg2 4 u8 alt range 1 2 || any rep=4,1
  global 0218a7c0 4
  fix global0 0 u32 addrof arg2
  stub 021885bc ret=u32 args=p,p
  stub 0218a2fc ret=u32 args=p
  stub 0218a310 ret=u32 args=p
  stub 0218a324 ret=u32 args=p
  stub 0218a384 ret=u32 args=p,v,v
  stub 0218a494 ret=u32 args=p,p
  stub 0218a528 ret=u32 args=p,p
  stub 0201127c force ret=none args=p,v
  stub 02011290 force ret=none args=p
  stub 02011298 force seq ret=pick:0/1 args=p
  stub 0201beb8 force ret=pick:0/1/2 args=v,v
  stub 020307b0 force ret=none args=p
  stub 0203f0a8 force seq ret=u32
  stub 02049250 force ret=u32 args=v,v,v
  stub 020870d0 force ret=ticks:100   # Z21: a clock advancing 100 per read (the 500-tick budget of 021885bc)
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021b9a6c force ret=pick:0/1 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5584 force ret=pick:0/1 args=p,v
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 02188284 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg buf 0x20
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 range 0 3
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg1 0x00 u8 alt range 0 2 || any
  fix arg1 0x04 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x14 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x3C u32 alt range 0 15 || any
  fix arg1 0x40 u32 alt range 0 15 || any
  fix arg1 0x44 u8 alt mask 0x0F || any
  fix arg1 0xD0 u32 alt range 0 700 || any
  fix arg1 0xD4 u32 alt const 0 || any
  fix arg2 0 u8 alt range 0 3 || const 0xFF rep=4,1
  fix arg2 4 u8 alt range 1 2 || any rep=4,1
  global 0218a7c0 4
  fix global0 0 u32 addrof arg2
  stub 021885bc ret=u32 args=p,p
  stub 0218a2fc ret=u32 args=p
  stub 0218a310 ret=u32 args=p
  stub 0218a324 ret=u32 args=p
  stub 0218a378 ret=u32 args=p,v,v
  stub 0218a384 ret=u32 args=p,v,v
  stub 0218a494 ret=u32 args=p,p
  stub 0218a528 ret=u32 args=p,p
  stub 0201127c force ret=none args=p,v
  stub 02011290 force ret=none args=p
  stub 02011298 force seq ret=pick:0/1 args=p
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 0201beb8 force ret=pick:0/1/2 args=v,v
  stub 020307b0 force ret=none args=p
  stub 0203f0a8 force seq ret=u32
  stub 02049250 force ret=u32 args=v,v,v
  stub 020870d0 force ret=ticks:100   # Z21: a clock advancing 100 per read (the 500-tick budget of 021885bc)
  stub 021b8ce8 force ret=pick:0/1 args=p,v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021b9a6c force ret=pick:0/1 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5584 force ret=pick:0/1 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 021885bc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg buf 0x20
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 range 0 3
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg1 0x00 u8 alt range 0 2 || any
  fix arg1 0x04 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x14 u32 alt range 0 3 || oneof 99 100 101 0xFFFFFFFF || any rep=4,4
  fix arg1 0x3C u32 alt range 0 15 || any
  fix arg1 0x40 u32 alt range 0 15 || any
  fix arg1 0x44 u8 alt mask 0x0F || any
  fix arg1 0xD0 u32 alt range 0 700 || any
  fix arg1 0xD4 u32 alt const 0 || any
  fix arg2 0 u8 alt range 0 3 || const 0xFF rep=4,1
  fix arg2 4 u8 alt range 1 2 || any rep=4,1
  global 0218a7c0 4                               # Z21: the global AI table (0218a494 / 0218a528)
  fix global0 0 u32 addrof arg2
  stub 0218a384 ret=u32 args=p,v,v
  stub 0218a494 ret=u32 args=p,p
  stub 0218a528 ret=u32 args=p,p
  stub 0201127c force ret=none args=p,v
  stub 02011290 force ret=none args=p
  stub 02011298 force seq ret=pick:0/1 args=p
  stub 0201beb8 force ret=pick:0/1/2 args=v,v
  stub 020307b0 force ret=none args=p
  stub 02049250 force ret=u32 args=v,v,v
  stub 020870d0 force ret=ticks:100   # Z21: a clock advancing 100 per read (the 500-tick budget of 021885bc)
  stub 021b9a6c force ret=pick:0/1 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5584 force ret=pick:0/1 args=p,v

fn 0218871c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218875c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0203f0a8 force seq ret=u32

fn 0218872c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218875c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0203f0a8 force seq ret=u32

fn 0218873c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218875c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0203f0a8 force seq ret=u32

fn 0218874c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218875c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0203f0a8 force seq ret=u32

fn 0218875c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0203f0a8 force seq ret=u32

fn 0218878c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg1 0x01 u8 range 0 3
  stub 0201134c force seq ret=nth:pick:1/5/0xFFFFFFFB/0xFFFFFF9C/0x7FFFFFFF/0x80000000 args=p   # Z21: per-operand bounds

fn 021887b4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021887f4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/25/50/99/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5c44 force ret=pick:0/0x63/0x64/0x65/0x800/0x1000/0x19000/0x32000/0x32800/0x64000/0xFFFFF000 args=p

fn 021887c4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021887f4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/25/50/99/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5c44 force ret=pick:0/0x63/0x64/0x65/0x800/0x1000/0x19000/0x32000/0x32800/0x64000/0xFFFFF000 args=p

fn 021887d4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021887f4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/25/50/99/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5c44 force ret=pick:0/0x63/0x64/0x65/0x800/0x1000/0x19000/0x32000/0x32800/0x64000/0xFFFFF000 args=p

fn 021887e4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021887f4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/25/50/99/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5c44 force ret=pick:0/0x63/0x64/0x65/0x800/0x1000/0x19000/0x32000/0x32800/0x64000/0xFFFFF000 args=p

fn 021887f4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/25/50/99/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5c44 force ret=pick:0/0x63/0x64/0x65/0x800/0x1000/0x19000/0x32000/0x32800/0x64000/0xFFFFF000 args=p

fn 0218883c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218885c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62a8 force ret=pick:0/1/0x40 args=p

fn 0218884c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218885c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62a8 force ret=pick:0/1/0x40 args=p

fn 0218885c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62a8 force ret=pick:0/1/0x40 args=p

fn 0218889c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021888bc ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 021888ac ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021888bc ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 021888bc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 02188904 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188924 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v
  stub 021d6314 force ret=u32 args=p,v
  stub 021e8a78 force ret=pick:0/1/2 args=v

fn 02188914 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188924 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v
  stub 021d6314 force ret=u32 args=p,v
  stub 021e8a78 force ret=pick:0/1/2 args=v

fn 02188924 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v
  stub 021d6314 force ret=u32 args=p,v
  stub 021e8a78 force ret=pick:0/1/2 args=v

fn 02188980 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021889a0 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5bdc force ret=pick:0/1/2/0xFFFFFFFF args=p,v

fn 02188990 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021889a0 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5bdc force ret=pick:0/1/2/0xFFFFFFFF args=p,v

fn 021889a0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5bdc force ret=pick:0/1/2/0xFFFFFFFF args=p,v

fn 021889e8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188a08 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021c82d0 force ret=pick:0/1/2 args=v,v,v

fn 021889f8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188a08 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021c82d0 force ret=pick:0/1/2 args=v,v,v

fn 02188a08 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021c82d0 force ret=pick:0/1/2 args=v,v,v

fn 02188a54 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188ab4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188a64 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188ab4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188a74 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188ab4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188a84 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188ab4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188a94 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188ab4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188aa4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188ab4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188ab4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/7/0x80000000/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188ad8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188af8 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188ae8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188af8 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188af8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02188b1c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg0 0x300 u32 range 0 7 rep=4,4          # Z21: the -1-terminated script list (operand a = 0x100 / 0x108 / 0x110)
  fix arg0 0x310 u32 const 0xFFFFFFFF
  fix arg1 0x38 u32 alt range 0 7 || const 0xFFFFFFFF || any   # Z21: result word in / not in the list
  stub 0201134c force seq ret=nth:pick:0x100/0x108/0x110;u32 args=p
  stub 020113b0 force ret=none args=p,v

fn 02188b64 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  fix arg0 0x300 u32 range 0 7 rep=4,4          # Z21: the -1-terminated script list (operand a = 0x100 / 0x108 / 0x110)
  fix arg0 0x310 u32 const 0xFFFFFFFF
  fix arg1 0x38 u32 alt range 0 7 || const 0xFFFFFFFF || any   # Z21: result word in / not in the list
  stub 0201134c force seq ret=nth:pick:0x100/0x108/0x110;u32 args=p
  stub 020113b0 force ret=none args=p,v

fn 02188bb0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 02188c1c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 02188c88 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021c8138 force ret=u32 args=p

fn 02188ca0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4/5/6/7/8/9 args=p   # Z21: per-operand bounds
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d57fc force ret=u16 args=v
  stub 021e8cfc force ret=pick:0/1/2/3 args=v
  stub 021e8d04 force ret=pick:0/1/2/3 args=v

fn 02188d48 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0201c1d8 force ret=u16 args=v

fn 02188d5c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0201134c force seq ret=u32 args=p
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 02188e00 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 02188e24 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4/5/6/7/8;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021c81c8 force ret=pick:0/1/2/100 args=p,v,v

fn 02188e90 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021d5b7c force ret=pick:0/1/2 args=v

fn 02188f08 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 02188f14 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 02188f2c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 range 0 3
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a220 ret=u32 args=p,v,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 02019c90 force ret=nth:pick:0/5;pick:0/6;pick:7 args=v,v,v
  stub 0203f0a8 force seq ret=u32
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5444 force ret=u16 args=p
  stub 021d59b4 force ret=pick:0/5/0x17/0x2A/0x47/0x48 args=v,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v
  stub 021f8afc force ret=pick:0/0/0/1/0x47/0x100 args=v

fn 02188f50 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 02188f58 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4/5/6;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021c7dac force ret=pick:0/1/2/3/4/5/6/0xFFFFFFFF args=p,v,v,v
  stub 021d5440 force ret=u8 args=p

fn 02188fac ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188fcc ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v
  stub 021d62a8 force ret=pick:0/1/0x40 args=p

fn 02188fbc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02188fcc ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v
  stub 021d62a8 force ret=pick:0/1/0x40 args=p

fn 02188fcc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v
  stub 021d62a8 force ret=pick:0/1/0x40 args=p

fn 0218905c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021ef884 force ret=u32

fn 0218906c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218908c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/0x1B/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 0218907c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218908c ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/0x1B/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 0218908c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/0x1B/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 021890c0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189100 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 021890d0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189100 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 021890e0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189100 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 021890f0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189100 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189100 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189150 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189170 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189160 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189170 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189170 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 021891d4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021891f4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 021891e4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021891f4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 021891f4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 021892d8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021892f8 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3/0x1B/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 021892e8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021892f8 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3/0x1B/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 021892f8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3/0x1B/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 021893ac ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 021893b4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 021893cc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 021893d4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 021893dc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b40 force ret=pick:0/1/2/3/0x17/0xFFFF args=p

fn 02189400 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0218a3f4 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 02020f20 force ret=u32 args=v,v
  stub 02021014 force ret=u32 args=p,v
  stub 020307b0 force ret=none args=p
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b40 force ret=pick:0/1/2/3/0x17/0xFFFF args=p

fn 02189434 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 0218945c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5bdc force ret=pick:0/1/2/0xFFFFFFFF args=p,v

fn 0218948c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d69ac force ret=u8 args=p,v

fn 021894b4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 021894c4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 021894d4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d6740 force ret=u16 args=p

fn 021894f8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 02189500 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 02189524 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 02189548 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d679c force ret=u16 args=p
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 02189590 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v

fn 021895ac ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02011290 force ret=none args=p

fn 021895c4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2;u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189620 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189640 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 02189630 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189640 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 02189640 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 02189668 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v

fn 02189694 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3/0x100 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d57fc force ret=u16 args=v
  stub 021e8cfc force ret=pick:0/1/2/3 args=v
  stub 021e8d04 force ret=pick:0/1/2/3 args=v

fn 021896e0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 range 0 3
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a220 ret=u32 args=p,v,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/5/7/0x17/0x2A/0x47 args=p   # Z21: per-operand bounds
  stub 02019c90 force ret=nth:pick:0/5;pick:0/6;pick:7 args=v,v,v
  stub 0203f0a8 force seq ret=u32
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5444 force ret=u16 args=p
  stub 021d59b4 force ret=pick:0/5/0x17/0x2A/0x47/0x48 args=v,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v
  stub 021f8afc force ret=pick:0/0/0/1/0x47/0x100 args=v

fn 02189718 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5bdc force ret=pick:0/1/2/0xFFFFFFFF args=p,v

fn 02189758 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;pick:0/1/2/3/0x17/0xFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b40 force ret=pick:0/1/2/3/0x17/0xFFFF args=p

fn 0218979c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021ef948 force ret=pick:0/1/2 args=v

fn 021897cc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021c82d0 force ret=pick:0/1/2 args=v,v,v

fn 02189800 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v
  stub 021d5c44 force ret=pick:0/0x63/0x64/0x65/0x800/0x1000/0x19000/0x32000/0x32800/0x64000/0xFFFFF000 args=p

fn 0218989c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d554c force ret=pick:0/1/2 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 02189978 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0218a3f4 ret=u32 args=p,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 02020f20 force ret=u32 args=v,v
  stub 02021014 force ret=u32 args=p,v
  stub 020307b0 force ret=none args=p
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b40 force ret=pick:0/1/2/3/0x17/0xFFFF args=p
  stub 021d62c4 force ret=pick:0/1/2 args=p,v

fn 021899c4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021d5584 force ret=pick:0/1 args=p,v

fn 021899dc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d5478 force ret=pick:0/1/2/3/4 args=p

fn 02189a2c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 02189a44 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a378 ret=u32 args=p,v,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 02189a6c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021c81d0 force ret=u16 args=p,v,v

fn 02189a9c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5ba4 force ret=u16 args=p

fn 02189ac0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a308 ret=u32 args=p,v
  stub 0218a390 ret=u32 args=p,v,v,v
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4 args=p
  stub 021b9c00 force ret=u32 args=p,v
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 02189b78 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0201134c force seq ret=u32 args=p
  stub 020113b0 force ret=none args=p,v
  stub 021c7dac force ret=pick:0/1/2/3/4/5/6/0xFFFFFFFF args=p,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 02189bfc ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0218a390 ret=u32 args=p,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 02189c8c ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189cec ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189d30 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 02189d38 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 02189d40 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any

fn 02189d48 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a308 ret=u32 args=p,v
  stub 0201134c force seq ret=u32 args=p
  stub 021b8de4 force ret=u8 args=p,v
  stub 021b99b8 force ret=pick:1/2/3 args=p,v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9c00 force ret=u32 args=p,v
  stub 021b9c98 force ret=pick:0xFFFFFFFF/0/1/2/3 args=p,v
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v
  stub 021d5b7c force ret=pick:0/1/2 args=v

fn 02189e70 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189e90 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 02189e80 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189e90 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 02189e90 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5b68 force ret=pick:0/1/2 args=v

fn 02189ed0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 02189ef8 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d6d7c force ret=pick:0/1/2 args=p

fn 02189f34 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4 args=p   # Z21: per-operand bounds
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5444 force ret=u16 args=p

fn 02189f58 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189f98 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02189f68 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189f98 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02189f78 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189f98 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02189f88 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 02189f98 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02189f98 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0201134c force seq ret=nth:pick:0/1/0x40/0x7F/0x80/0xFF/0x100/0xFFFFFFFF;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v

fn 02189fc0 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 021895ac ret=u32 args=p,p
  stub 0218a378 ret=u32 args=p,v,v
  stub 02011290 force ret=none args=p
  stub 0201134c force seq ret=nth:pick:0/0/1;pick:0/1/2/3/0xFFFFFFFF;pick:0x100/0x140;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 0218a034 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a160 ret=u8 args=p,v
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021c82f4 force ret=pick:0/1/2 args=v,v,v

fn 0218a074 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0a4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 0218a084 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0a4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 0218a094 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0a4 ret=u32 args=p,p,v
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 0218a0a4 ret=u32
  nowork
  arg buf 0x600
  arg buf 0x100
  arg u8 any
  fix arg0 0x14 u32 addrof arg0 0x200
  fix arg1 0x94 u8 range 0 5
  fix arg1 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg1 0x01 u8 alt range 0 3 || any
  fix arg1 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg1 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg1 0x45 u8 alt range 0 3 || any
  fix arg1 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg1 0x98 u32 alt range 0 3 || any
  fix arg1 0xCC u32 alt range 0 255 || any
  stub 0218a0f4 ret=pick:0/1 args=p,v,v,v,v
  stub 0218a160 ret=u8 args=p,v
  stub 0218a2fc ret=u32 args=p
  stub 0201134c force seq ret=nth:pick:0/1/2/3/4;u32 args=p   # Z21: per-operand bounds
  stub 020113b0 force ret=none args=p,v
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d59b4 force ret=pick:0/1/2/5/6/7/8/0x17/0x2A/0x47/0x48/100/0xFFFFFFFF args=v,v

fn 0218a0f4 ret=u32
  nowork
  arg buf 0x600
  arg u8 range 0 8
  arg s32 alt range 0 100 || oneof 0x7FFFFFFF 0x80000000 0xFFFFFFFF || any
  arg s32 alt rel 2 const 0 || rel 2 const 1 || rel 2 const 0xFFFFFFFF || range 0 100 || any
  arg u32 any
  fix arg0 0x14 u32 addrof arg0 0x200
  stub 020113b0 force ret=none args=p,v

fn 0218a160 ret=u32
  nowork
  arg buf 0x100
  arg u8 range 0 4
  fix arg0 0x98 u32 alt range 0 3 || any        # Z21: battle kind 0..3 (front_pos)
  fix arg0 0x4C u16 alt range 0 3 || any rep=24,2
  stub 021b8c38 force ret=pick:0/1/2/3/4/5 args=v,v

fn 0218a220 ret=u32
  nowork
  arg buf 0x100
  arg u8 alt range 0 3 || any
  arg u8 range 0 5
  fix arg0 0x98 u32 range 0 3        # Z21: battle kind 0..3 (other kinds: front_pos returns the AI address, which 0218a220 adds to ai+0x7C)
  fix arg0 0x4C u16 alt range 0 3 || any rep=24,2
  stub 0218a2fc ret=u32 args=p
  stub 02019c90 force ret=nth:pick:0/5;pick:0/6;pick:7 args=v,v,v
  stub 0203f0a8 force seq ret=u32
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d5444 force ret=u16 args=p
  stub 021d59b4 force ret=pick:0/5/0x17/0x2A/0x47/0x48 args=v,v
  stub 021d62c4 force ret=pick:0/1/2 args=p,v
  stub 021f8afc force ret=pick:0/0/0/1/0x47/0x100 args=v

fn 0218a2fc ret=u32
  nowork
  arg buf 0x100
  stub 021b98ac force ret=nullable:8:u32 args=p,v

fn 0218a308 ret=u32
  nowork
  arg buf 0x100
  arg u8 range 0 4
  stub 021b9c00 force ret=u32 args=p,v

fn 0218a310 ret=u32
  nowork
  arg buf 0x100

fn 0218a324 ret=u32
  nowork
  arg buf 0x100
  fix arg0 0x95 u8 range 0 5
  # Z21 biases: slot / move / result / script kind / remembered moves / battle kind / turn word
  fix arg0 0x01 u8 alt range 0 3 || any
  fix arg0 0x02 u16 alt range 0 3 || range 0x100 0x103 || any
  fix arg0 0x38 u32 alt range 0 7 || oneof 0x80000000 0xFFFFFFFF || any
  fix arg0 0x45 u8 alt range 0 3 || any
  fix arg0 0x4C u16 alt range 0 3 || any rep=24,2
  fix arg0 0x98 u32 alt range 0 3 || any
  fix arg0 0xCC u32 alt range 0 255 || any
  stub 0218a2fc ret=u32 args=p
  stub 021b98ac force ret=nullable:8:u32 args=p,v
  stub 021d67a8 force ret=pick:0/1/2/3/0xB6/0xC5/0xCB args=p

fn 0218a378 ret=u32
  nowork
  arg buf 0x100
  arg u16 any
  arg u16 any
  stub 0201bd28 force ret=pick:0/1/2/3/0x1B/0xFFFF args=v,v

fn 0218a384 ret=u32
  nowork
  arg buf 0x100
  arg u16 any
  arg u16 any
  stub 0201beb8 force ret=pick:0/1/2 args=v,v

fn 0218a390 ret=u32
  nowork
  arg buf 0x100
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021c7e14 force ret=pick:0/1/2/5/6/7/100 args=p,v,v,v,v,v
  stub 021d5440 force ret=u8 args=p
  stub 021d5448 force ret=pick:0/1/2/3/4 args=p
  stub 021d54c0 force ret=pick:0/1/2/3/0xB6 args=p,v

fn 0218a3f4 ret=u32
  nowork
  arg buf 0x100
  arg u16 any
  arg u8 range 0 2
  stub 02020f20 force ret=u32 args=v,v
  stub 02021014 force ret=u32 args=p,v
  stub 020307b0 force ret=none args=p

fn 0218a41c ret=u32
  nowork
  arg u16 any
  arg buf 0x20
  global 0218a7c0 4
  fix global0 0 u32 addrof arg1
  stub 02030734 force ret=u32 args=v,v,v,v,v

fn 0218a458 ret=u32
  nowork
  arg buf 0x20
  global 0218a7c0 4
  fix global0 0 u32 addrof arg0
  stub 020307b0 force ret=none args=p

fn 0218a494 ret=u32
  nowork
  arg buf 0x100
  arg buf 0x20
  global 0218a7c0 4
  fix global0 0 u32 addrof arg1
  fix arg0 0x45 u8 alt range 0 3 || any          # Z21: script kind vs the table kinds
  fix arg1 0 u8 alt range 0 3 || const 0xFF rep=4,1
  fix arg1 4 u8 alt range 1 2 || any rep=4,1
  stub 02049250 force ret=u32 args=v,v,v

fn 0218a528 ret=u32
  nowork
  arg buf 0x100
  arg buf 0x20
  global 0218a7c0 4
  fix global0 0 u32 addrof arg1
  fix arg0 0x45 u8 alt range 0 3 || any          # Z21: script kind vs the table kinds
  fix arg1 0 u8 alt range 0 3 || const 0xFF rep=4,1
  fix arg1 4 u8 alt range 1 2 || any rep=4,1
  stub 020307b0 force ret=none args=p
