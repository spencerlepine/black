# fndiff spec: overlay-21 script command file scrcmd_proc_fld.c (F8-spec4), Thumb code 0x021B4BEC..0x021B5190: F1-SPLIT row 21.47
# (0x021B4D60..) plus the tail of row 21.46 (0x021B4BEC..0x021B4D60).  __FILE__ "scrcmd_proc_fld.c" (0x021DE220) is passed by
# 0x021B4D60 (cmd 335), whose result callback 0x021B4D14 sits right before it; the handlers 0x021B4BEC..0x021B4D0A (commands
# 314, 351..354, 358) are the same kind (field events from 020124AC, like scrcmd_proc.c's 356 / 357) and sit with it, so the
# file starts at 0x021B4BEC (inferred: no __FILE__ use there).  The helper 0x021B5190 is scrcmd_encount.c's (its callers are
# in cmd 561).  Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_proc_fld --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  188 021b5128  314 021b4bec  333 021b4fb0  335 021b4d60  338 021b4f68  342 021b4ee0  351 021b4c14  352 021b4c34
#  353 021b4c54  354 021b4c98  358 021b4cdc  376 021b503c  377 021b5094  380 021b50d0  381 021b50f8  411 021b4f1c
#  444 021b5154  536 021b4e58  537 021b4e9c  580 021b5008
# 21 functions specced (20 command ids).  Every command builds a field event or app proc and calls it as a child event
# (0215897C(SW, ev)) or switches to it (342: 0215898C + 02011290 stop): 333 021C5BD0, 335 the proc 0xAA 0x021EF78C through
# 02014840 (party-select-like: P (0x2C) = {GAMEDATA, party, bag, mystatus, trainer, .., m}, result callback 0x021B4D15 writes
# P+0x28 == 1 to the var), 338 020146F8 proc 0xDB 0x02185F0C, 353 / 354 / 358 events 020124AC kind 6 / 4 / 1, 376 021AA0A0,
# 377 021BE92C, 381 0216E95C, 411 02168BBC, 536 / 537 021CD2A4 mode 5 / 6, 580 021D30B8; queries 188 / 314 / 351 / 352 / 380 /
# 444 write a var.  Event / proc roles UNVERIFIED (named by address).
# Not specced (0):
#   none
# Fragments (not functions): 0x021B4C08 (decodes as data; a "bl" target from a data word in main).
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
area scrcmd_proc_fld
set field

# cmd 314: var; *var = 021896F4(02012138(GAMESYS)) (u16); 1.
fn 021b4bec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 021896f4 force ret=u32 args=v

# cmd 351: var; *var = 02009CE4(0201283C(GAMEDATA)) (u16); 0.
fn 021b4c14 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5e4 force ret=u32 args=v
  stub 0201283c force ret=u32 args=v
  stub 02009ce4 force ret=u32 args=v

# cmd 352: var; *var = 02009A34(0201283C(GAMEDATA)) (u16); 0.
fn 021b4c34 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5e4 force ret=u32 args=v
  stub 02009a34 force ret=u32 args=v
  stub 0201283c force ret=u32 args=v

# cmd 353: rec = {field map, 0}; ev = 020124AC(GAMESYS, 6, 0x02154C21, &rec); 0215897C(SW, ev) (child event); 1.
fn 021b4c54 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020124ac force ret=u32 args=v,v,v,in:8
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 354: rec = {field map, 0}; ev = 020124AC(GAMESYS, 4, 0x021542C1, &rec); child event; 1.
fn 021b4c98 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020124ac force ret=u32 args=v,v,v,in:8
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 358: ev = 020124AC(GAMESYS, 1, 0x02154695, field map) (r3 is the field map itself here); child event; 1.
fn 021b4cdc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020124ac force ret=u32 args=v,v,v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# callback of cmd 335 (W = {GAMEDATA, u16 *var, P}): if P+0x24 == 4: 0200BB58(0200BAC8(02012EBC(W[0])), P+8) and
# Heap_Free(P+8); *var = (P+0x28 u16 == 1); Heap_Free(P).
fn 021b4d14 ret=none inputs=200
  nowork
  arg buf 0x100
  fix arg0 4 u32 addrof arg0 0x40
  fix arg0 8 u32 addrof arg0 0x80
  fix arg0 0xA4 u32 alt const 4 || any
  fix arg0 0xA8 u16 alt range 0 2 || any
  stub 0200bac8 force ret=u32 args=v
  stub 0200bb58 force ret=none args=v,v
  stub 02012ebc force ret=u32 args=v
  stub 020307b0 force ret=none args=v

# cmd 335: var, m = operands; P = alloc(0x2C) = {GAMEDATA, 0201293C / 02012934 / 0201292C / 02012944(GAMEDATA), +0x14 0,
# +0x18 0200873C(SAVE), +0x1C 0200CA38(0200CA1C(GAMEDATA)), +0x24 m}; m == 4: S = 0200BAC8(SAVE), P+8 = 0200BAF4(S, 4),
# P+0x14 = 0200BBFC(S, 3); W = alloc(0xC) = {GAMEDATA, var, P}; ev = 02014840(GAMESYS, field map, 0xAA, 0x021EF78C, P,
# 0x021B4D15, W); child event; 1.  ScriptHost: an app-proc request (proc 0xAA; UNVERIFIED role).
fn 021b4d60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4/5 args=v,v seq
  stub 0200ca1c force ret=u32 args=v
  stub 0201292c force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 02012934 force ret=u32 args=v
  stub 0200ca38 force ret=u16 args=v
  stub 0215897c force ret=none args=v,v
  stub 0200873c force ret=u32 args=v
  stub 0201293c force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  stub 02014840 force ret=u32 args=v,v,v,v,v,v,v
  stub 02012944 force ret=u32 args=v
  stub 0200bac8 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0200baf4 force ret=u32 args=v,v
  stub 0200bbfc force ret=u32 args=v,v

# cmd 536: var; ev = 021CD2A4(GAMESYS, field map, 0, 5, var); child event; 1.
fn 021b4e58 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021cd2a4 force ret=u32 args=v,v,v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 537: var; ev = 021CD2A4(GAMESYS, field map, 0, 6, var); child event; 1.
fn 021b4e9c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021cd2a4 force ret=u32 args=v,v,v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 342: v; ev = 0215F8A8(GAMESYS, v); 0215898C(SW, ev) (switch to the event); 02011290(vm) (stop); 1.
fn 021b4ee0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215f8a8 force ret=u32 args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215898c force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011290 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 411: S = 02009210(02012EBC(GAMEDATA)); v; 020090D8(S, 1); ev = 02168BBC(GAMESYS, v); child event; 1.
fn 021b4f1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02168bbc force ret=u32 args=v,v
  stub 02009210 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215897c force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020090d8 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 338: ev = 020146F8(GAMESYS, field map, 0xDB, 0x02185F0C, GAMEDATA); child event; 1.
fn 021b4f68 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020146f8 force ret=u32 args=v,v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 333: v, var; ev = 021C5BD0(GAMESYS, v == 1, var); child event; 1.
fn 021b4fb0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2 args=v,v seq
  stub 021c5bd0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 580: var; ev = 021D30B8(GAMESYS, var); child event; 1.
fn 021b5008 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021d30b8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 376: a, b, c; 02158F54(SW); R = 02158F58(SW); ev = 021AA0A0(02188CE8(R+4), a, (u8)b, c); child event; 1.
fn 021b503c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021aa0a0 force ret=u32 args=v,v,v,v
  stub 02188ce8 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215897c force ret=none args=v,v

# cmd 377: ev = 021BE92C(GAMESYS, field map, 0, 0, 1, 0, 0); child event; 1.
fn 021b5094 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 380: var; *var = 0216EFCC(02012A44(GAMEDATA)) (u16); 0.
fn 021b50d0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216efcc force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02012a44 force ret=u32 args=v

# cmd 381: ev = 0216E95C(GAMESYS, field map, 4); child event; 1.
fn 021b50f8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216e95c force ret=u32 args=v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 188: var; *var = 021AAB48(02188CE8(field map)) (u16); 0.
fn 021b5128 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021aab48 force ret=u32 args=v
  stub 02188ce8 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 444: S = 0200D418(02012EBC(GAMEDATA)); i = (u8)value; 0200D488(S, i) == 0 -> 0215E440(GAMEDATA, i); 0.
fn 021b5154 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215e440 force ret=none args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0200d488 force ret=pick:0/1 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200d418 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
