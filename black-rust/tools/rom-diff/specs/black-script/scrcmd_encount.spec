# fndiff spec: overlay-21 script command file scrcmd_encount.c (F8-spec4), Thumb code 0x021B5190..0x021B5700: F1-SPLIT row 21.48
# (0x021B51BC..0x021B56E0) plus its static helper 0x021B5190 (called only by cmd 561) and the trailing literal pool.
# __FILE__ "scrcmd_encount.c" (0x021DE234) is passed by 0x021B51BC (cmd 561).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_encount --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  319 021b52fc  320 021b536c  321 021b538c  322 021b53a4  323 021b53bc  324 021b5498  325 021b54fc  326 021b542c
#  327 021b54d4  328 021b5528  418 021b5610  419 021b5650  420 021b568c  561 021b51bc
# 19 functions specced (14 command ids).
#  * 561: a random id from the ARC 126 (0x7E) records of the zone (0215E2B8 per table row 0x021D98D0[0..6]; rec[5] / rec[6] add
#    the 5 ids at +0xC0 / +0xD4, unique, into a 0x78-byte list), picked with 02005728(n) (random below n).
#  * 319..328: the field effect object E = 02188C58(fm) (fieldmap camera / effect calls 0218Exxx / 0218Fxxx): 319 / 320 turn it
#    off (021b52a4 also from the init proc 0x021B52E0 of the table 0x021705F0), 321 / 322 0218EC54 / 0218EC60, 323 / 326 / 324
#    0218F900 / 0218F9C4 with a stack record built from operands (326 from ARC 163 member a, 02048F4C), 327 0218FA40, 325 waits on
#    0218FE50 (GMEVENT 0x021B55F1), 328 starts the shake event 0x021D2D50 (scrcmd_misc21).
#  * 418 / 419 call the overlay at 0x021F36C0 / 0x021F36C8 (outside the field set); 420 loads the proc 0x38 (021BB990) with one
#    of 4 tables 0x021F4344 / 4374 / 438C / 435C.
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
area scrcmd_encount
set field

# add_unique(u16 *list, n, v): v in list[0..n) -> n; else list[n] = v, n + 1 (u16).  Helper of cmd 561.
fn 021b5190 ret=u16 inputs=200
  nowork
  arg buf 0x100
  arg u32 range 0 0x7F
  arg u16 alt range 0 3 || any
  fix arg0 0 u16 alt range 0 3 || any rep=0x80,2

# cmd 561: var; zone = 02012984(GAMEDATA); A = 020490F4(0x7E, 0x8015) (ARC 126 handle); L = Heap_AllocDebug(0x8015, 0x78, 1,
# "scrcmd_encount.c", 0xF8); n = 0; for k in 0..6: if 0215E2B8(&rec, A, u16 table 0x021D98D0[k], zone): rec[5] -> the 5
# ids rec+0xC0 + 4i (& 0x7FF), rec[6] -> the 5 ids rec+0xD4 + 4i, each added with 021b5190(L, n, id); *var = L[02005728(n)];
# Heap_Free(L); 02049238(A); 0.  rec (0xE8 bytes) is a stack record the stub fills (`outm` masks: small ids, flags 0/1).
fn 021b51bc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012984 force ret=u16 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02005728 force ret=lt:0 args=v
  stub 02049238 force ret=u32 args=v
  stub 021b5190 ret=pick:0/1/2/3/5 args=v,v,v
  stub 020307b0 force ret=none args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 020490f4 force ret=u32 args=v,v
  stub 0215e2b8 force ret=pick:0/1 args=outm:0xE8:3/0/0xFF/0xFF/3/1/0xFF/0xFF,v,v,v

# encount_off(field map, E): 0218F878(E); 0218F0F8(E) -> 0218F05C(E); 02188CFC(field map) == 1 -> 021BB260(02188C5C(field
# map), 1); 02159168(0).  (r0 = 02159168's result, ignored by both callers.)
fn 021b52a4 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 0218f0f8 force ret=pick:0/1 args=v
  stub 021bb260 force ret=none args=v,v
  stub 02159168 force ret=none args=v
  stub 0218f05c force ret=none args=v
  stub 02188c5c force ret=u32 args=v
  stub 0218f878 force ret=none args=v
  stub 02188cfc force ret=pick:0/1/2 args=v

# init proc (table 0x021705F0): GAMESYS = task[0]; fm = 0201210C(GAMESYS); 021b52a4(fm, 02188C58(fm)); 1.
fn 021b52e0 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 02188c58 force ret=u32 args=v
  stub 021b52a4 ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  # closure: 021b52a4
  stub 0218f0f8 force ret=pick:0/1 args=v
  stub 021bb260 force ret=none args=v,v
  stub 02159168 force ret=none args=v
  stub 0218f05c force ret=none args=v
  stub 02188c5c force ret=u32 args=v
  stub 0218f878 force ret=none args=v
  stub 02188cfc force ret=pick:0/1/2 args=v

# cmd 319: fm, E = 02188C58(fm); 02159144(0); 0218F0F8(E) -> {0218F0A8(E); ev = 020122C0(GAMESYS, 0, 0x021B55C9, 0); child
# event; 02188CFC(fm) == 1 -> 021BB260(02188C5C(fm), 0)} else 0218F818(E); 1.
fn 021b52fc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0218f0f8 force ret=pick:0/1 args=v
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 021bb260 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 0218f0a8 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02188c5c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cfc force ret=pick:0/1/2 args=v
  stub 02188c58 force ret=u32 args=v
  stub 0218f818 force ret=none args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 320: 021b52a4(fm, 02188C58(fm)); 0.
fn 021b536c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c58 force ret=u32 args=v
  stub 021b52a4 ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b52a4
  stub 0218f0f8 force ret=pick:0/1 args=v
  stub 021bb260 force ret=none args=v,v
  stub 02159168 force ret=none args=v
  stub 0218f05c force ret=none args=v
  stub 02188c5c force ret=u32 args=v
  stub 0218f878 force ret=none args=v
  stub 02188cfc force ret=pick:0/1/2 args=v

# cmd 321: 0218EC54(02188C58(field map)); 0.
fn 021b538c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec54 force ret=none args=v

# cmd 322: 0218EC60(02188C58(field map)); 0.
fn 021b53a4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c58 force ret=u32 args=v
  stub 0218ec60 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 323: E; rec (0x48 bytes, stack) = {+0x24 u16 op, +0x26 u16 op, +0x28 u32 op, +0xC / +0x10 / +0x14 u32 ops, +0x30 0,
# +0x34 / 0x38 / 0x3C 1, +0x40 0, +0x44 1; the other words unset (0 under Z19)}; 0218F900(E, &rec, u16 op); 0.
fn 021b53bc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0218f900 force ret=none args=v,in:0x48,v
  stub 0201210c force ret=u32 args=v
  stub 0201134c force ret=u32 args=v seq
  stub 02011330 force ret=u16 args=v seq
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 326: E; a, b = u16 ops; 02048F4C(&t, 0xA3, a) (ARC 163 member a into the 0x2C-byte stack record t); rec = {+0x24 t.u16[0],
# +0x26 t.u16[1], +0x28 t[1], +0xC..+0x18 t[2..5], +0x30..+0x48 t[5..11]} (the rest unset); 0218F900(E, &rec, b); 0.
fn 021b542c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0218f900 force ret=none args=v,in:0x48,v
  stub 0201210c force ret=u32 args=v
  stub 02048f4c force ret=none args=out:0x2C,v,v
  stub 02011330 force ret=u16 args=v seq
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 324: E; rec = 6 words of 1; 0218F9C4(E, &rec, u16 op); 0.
fn 021b5498 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0218f9c4 force ret=none args=v,in:0x18,v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 327: 0218FA40(E, u16 op); 0.
fn 021b54d4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0218fa40 force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 325: ev = 020122C0(GAMESYS, 0, 0x021B55F1, 0); child event; 1.
fn 021b54fc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 328: v0..v7 = values; rec (0x18, stack) = {u16 v0, u16 v1, u16 v4, u16 v5, u16 v6, u16 0, u16 v7, u8 v3, u8 v3, u32 v2,
# u32 0}; ev = 021D2D50(GAMESYS, &rec) (scrcmd_misc21); child event; 1.
fn 021b5528 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 021d2d50 force ret=u32 args=v,in:0x18
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# GMEVENT of cmd 319 (event): E = 02188C58(0201210C(020124E4(event))); 0218F0E8(E) == 0 -> 0218F818(E), 1 (done); else 0.
fn 021b55c8 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 020124e4 force ret=u32 args=v
  stub 0218f0e8 force ret=pick:0/1 args=v
  stub 0201210c force ret=u32 args=v
  stub 02188c58 force ret=u32 args=v
  stub 0218f818 force ret=none args=v

# GMEVENT of cmd 325 (event): 0218FE50(E) != 0 -> 0 (wait); else 1.
fn 021b55f0 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 02188c58 force ret=u32 args=v
  stub 0218fe50 force ret=pick:0/1 args=v
  stub 020124e4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 418: v; ev = 021F36C0(GAMESYS, field map, (u16)(v - 1)) (overlay at 0x021F3xxx); child event; 1.
fn 021b5610 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f36c0 force ret=u32 args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 419: v; 0218E17C(02188C58(field map), 021F36C8((u16)(v - 1))); 0.
fn 021b5650 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f36c8 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218e17c force ret=none args=v,v
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 420: X = 02188D24(field map); v = value: 1 / 2 / 3 / 4 -> 021BB990(0x38, X, 0x021F4344 / 0x021F4374 / 0x021F438C /
# 0x021F435C) (jump table; 0 and > 4 do nothing); 0.
fn 021b568c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188d24 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4/5 args=v,v seq
  stub 021bb990 force ret=u32 args=v,v,v
  stub 0215a5dc force ret=u32 args=v
