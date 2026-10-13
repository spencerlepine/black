# fndiff spec: overlay-21 script command file scrcmd_fld_battle.c (F8-spec4), Thumb code 0x021B5700..0x021B5A18 (F1-SPLIT row
# 21.49).  __FILE__ "scrcmd_fld_battle.c" (0x021DE248) is passed by 0x021B5824.
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_fld_battle --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  441 021b5700  442 021b574c
# 6 functions specced (2 command ids).
#  * 441: count the mons a regulation record allows (0201A354(v, h) = record T; bit i of T+0xA and 0200CED4(zukan?, i) for i <
#    0x28B, at most 0x20) into the word-set list 02158F74(SW) (0201EE60) -> var.
#  * 442: regulation check of the party / the battle box (0200BAC8 save, 0200BBA8 == 1 -> box B = 0200BAF4(S, h)) with 02019ED4
#    (v 0x14..0x16) or 0201A090; both fail -> var = 3 / 0, else the party-select event 0x021B59B1 (party_select_list.c
#    021CC720 / 021CC810 / 021CC7B4) writes the choice (3 on cancel) to the var.  Roles UNVERIFIED.
# Not specced (0):
#   none
#
# Read from the disassembly of the raw OVY_21.sbin (rizin `-a arm -b 16 -m 0x02187F00`; rizin on field.elf mis-disassembles
# mixed ARM / Thumb and Ghidra's field set drops the arguments of overlay calls).  Function list: command-table targets + init-
# proc table entries (0x021705F0) + `bl` targets from the whole field set + odd code pointers + labels, walked as Thumb from every
# entry (only 2-byte padding and literal pools stay uncovered).
#
# Spec conventions (as scrcmd_core.spec / scrcmd_work.spec / scrcmd_proc.spec): every block is `nowork`.  A command handler
# `fn(vm, ctx)` gets two arenas: arg0 = the VM (`arg buf 0x800`: regs at 0x40, script bytes at 0x500, PC +0x14, compare byte +0xE
# in 0..2), arg1 = the scrcmd ctx (`arg buf 0x800`: CTX at 0, SCRCMD WORK at 0x100, zone A 0x300 (Heap_AllocDebug blocks, SW
# records), zone B 0x400, 0x500 (records a stub hands out), vars 0x600..0x7FF where `02159ae8` points).  A helper that takes the
# ctx in r0 gets the ctx arena as arg0.  A GMEVENT function `(event, seq, work)` gets `arg u32, arg buf 0x10 (seq, bounded),
# arg buf 0x40/0x100 (work)`; an init proc `(task, task + 0xC)` gets `arg buf 0x100, arg buf 0x10`; a TCB callback `(tcb, work)`
# gets `arg u32, arg buf`.  Script-operand readers (vm.c 02011330 / 0201134c, script_sys 02159ae8 / 02159b10) are `seq` stubs.
# Closure stubs as in core4 (every function of the area treated as ported: no fault, stray write or unstubbed callee).
# **Every stub outside the area carries `force`** (overlay addresses collide with battle `PORTED` entries and the callees in
# script_sys / scrcmd_work are ported; the F8 areas call each other only through forced stubs, so each is verified on its own).
# Indirect calls (`blx rN` through a fn argument) go to the fake callback 0x02EE0011 (stub 02ee0010).  UI callees are `ScriptHost`
# requests in the port (crates/black-script/src/host.rs); results that steer a branch are `pick:` stubs.  Heap_AllocDebug
# (02030734) is a `seqptr` bump allocator into an arena (blocks of a block never overlap); its 4th argument is the __FILE__ string,
# which places the file boundaries (F8-spec4: 0x021DE210 "scrcmd_shop.c", 0x021DE220 "scrcmd_proc_fld.c", 0x021DE234
# "scrcmd_encount.c", 0x021DE248 "scrcmd_fld_battle.c", 0x021DE25C "scrcmd_c03center.c", 0x021DE270 "scrcmd_pokemon_fld.c").
# Callees of other overlay-21 files, overlays 10 / 11 / 20, ARM9 main and the overlays at 0x021F3xxx (outside the field set) are
# force stubs.  An unset callee-saved register or stack word that a path reads is 0 under the Z19 entry state; the block says so.
area scrcmd_fld_battle
set field

# cmd 441: L = 02158F74(SW) (a list); v; var; h = 0215A5CC(ctx) (heap id); *var = 021b57cc(GAMEDATA, L, v, h); 0.
fn 021b5700 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5cc force ret=u16 args=v
  stub 021b57cc ret=u32 args=v,v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  # closure: 021b57cc
  stub 0201ee60 force ret=none args=v,v,v
  stub 0201a354 force ret=ptr:arg1:0x500:1 args=v,v
  stub 021b5824 ret=u32 args=v,v,v,v
  stub 020307b0 force ret=none args=v
  stub 0200ced4 force ret=pick:0/0/0/1 args=v,v
  stub 0200ca1c force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v

# cmd 442: v, var; h = 0215A5CC(ctx); P = 0201A354(v, h) (a regulation record); flag = (u16)(v - 0x14) <= 2; ev =
# 021b58b0(GAMESYS, flag, P, var, h); Heap_Free(P); ev == 0 -> 0; else child event, 1.
fn 021b574c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=pick:0/1/0x13/0x14/0x15/0x16/0x17 args=v,v seq
  stub 021b58b0 ret=nullable:2:u32 args=v,v,v,v,v
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201a354 force ret=u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 021b58b0
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0200bac8 force ret=u32 args=v
  stub 0200bba8 force ret=pick:0/1/2 args=v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 0201a090 force ret=pick:0/1 args=v,v,p
  stub 02019ed4 force ret=pick:0/1 args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0200baf4 force ret=nullable:2:u32 args=v,v
  stub 02012ebc force ret=u32 args=v

# count_mons(GAMEDATA, list, v, h): cnt = 0; T = 0201A354(v, h); M = 021b5824(T, GAMEDATA, &cnt, h); list[i] = M[i] for i <
# cnt via 0201EE60(list, i, M[i]); Heap_Free(M); Heap_Free(T); cnt.  Arena: arg 4 (never read; `ptr:` / `seqptr:` target).
fn 021b57cc ret=u16 inputs=200
  nowork
  arg u32 any
  arg buf 0x100
  arg u32 any
  arg u16 any
  arg buf 0x800
  fix arg4 0x50A u32 alt const 0 || any rep=21,4
  stub 0201ee60 force ret=none args=v,v,v
  stub 0201a354 force ret=ptr:arg4:0x500:1 args=v,v
  stub 021b5824 ret=ptr:arg4:0x580:1 args=v,v,outm:2:7/0,v
  stub 020307b0 force ret=none args=v
  # closure: 021b5824
  stub 0200ced4 force ret=pick:0/0/0/1 args=v,v
  stub 0200ca1c force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg4:0x300:0x40 args=v,v,v,v,v

# collect(T, GAMEDATA, u16 *cnt, h): *cnt = 0; Z = 0200CA1C(GAMEDATA); M = Heap_AllocDebug((h & 0x7FFF) | 0x8000, 0x40, 1,
# "scrcmd_fld_battle.c", 0xAC); for i in 0..0x28B: bit i of T+0xA set and 0200CED4(Z, i) -> M[(*cnt)++] = i, stop at 0x20;
# returns M (an address in the arena arg 4).
fn 021b5824 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg u32 any
  arg buf 0x10
  arg u16 any
  arg buf 0x800
  fix arg0 0xA u32 alt const 0 || any rep=21,4
  stub 0200ced4 force ret=pick:0/0/0/1 args=v,v
  stub 0200ca1c force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg4:0x300:0x40 args=v,v,v,v,v

# regulation_event(GAMESYS, flag, P, u16 *var, h (stack)): GD = 020120F4(GAMESYS); party = 02012934(GD); ok1 = flag ?
# 02019ED4(P, party) : 0201A090(P, party, &x); S = 0200BAC8(02012EBC(GD)); 0200BBA8(S) != 1 -> *var = ok1 ? 3 : 0, 0; B =
# 0200BAF4(S, h); ok2 = the same check on B; ok1 && ok2 -> Heap_Free(B) (nonzero), *var = 3, 0; else ev = 020122C0(GAMESYS,
# 0, 0x021B59B1, 0x1C), W = 020124E8(ev) = {GAMESYS, field map, -, party, B, u8 ok1 != 0, u8 ok2 != 0, u16 h, var}; ev.
fn 021b58b0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 range 0 1
  arg u32 any
  arg buf 0x10
  arg u16 any
  arg buf 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0200bac8 force ret=u32 args=v
  stub 0200bba8 force ret=pick:0/1/2 args=v
  stub 020124e8 force ret=ptr:arg5:0:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 020307b0 force ret=none args=v
  stub 0201a090 force ret=pick:0/1 args=v,v,p
  stub 02019ed4 force ret=pick:0/1 args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0200baf4 force ret=nullable:2:u32 args=v,v
  stub 02012ebc force ret=u32 args=v

# GMEVENT of 021b58b0 (event, seq, W): seq 0: W+8 = 021CC720(W+4, W+0xC, W+0x10, W+0x14 u8, W+0x15 u8, W+0x16 u16)
# (party_select_list.c), seq 1, 0; seq 1: 021CC810(W+8) != 0 -> *W+0x18 = 021CC7B4(W+8, &r) (r == 1 -> 3), Heap_Free(W+0x10),
# 1; else 0.
fn 021b59b0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 0x18 u32 addrof arg2 0x30
  stub 021cc810 force ret=pick:0/1 args=v
  stub 021cc720 force ret=u32 args=v,v,v,v,v,v
  stub 021cc7b4 force ret=u16 args=v,outm:4:1/0/0/0
  stub 020307b0 force ret=none args=v
