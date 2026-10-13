# fndiff spec: overlay-10 script commands of scrcmd_proc.c (F8-spec3), Thumb code 0x0215C63C..0x0215CE98 (F1-SPLIT row 10.15; the
# row's tail 0x0215CE98..0x0215CF00, commands 235..237, belongs to scrcmd_sodateya.c and is in scrcmd_sodateya.spec).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_proc --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  332 0215c6f0  334 0215c72c  336 0215c7e0  337 0215cb08  339 0215cbf0  340 0215ca64  341 0215cab4  355 0215cc38
#  356 0215cc4c  357 0215cce8  370 0215c95c  383 0215cd3c  386 0215cd80  390 0215cdb8  399 0215cdf0  406 0215ce70
#  489 0215c89c
# 22 functions specced (17 command ids).
#  * proc_call_wait 0x0215C63C(vm, ctx, ovl, proc table, param, cb, cbwork): the command's app-proc request.  W (0xC, heap 4) =
#    {param, cbwork, cb} goes into the SW + 0x3C slot (script_sys 02158FB4 / FB8 / FBC), 020120B4(GAMESYS, ovl, proc, param)
#    starts the proc and the VM waits on 0x0215C69D until 020120F0(GAMESYS) (proc running) is 0; then cb(W) (copies the proc's
#    result into script vars) or Heap_Free of param / cbwork, and the slot is freed.  Users: 334 (ovl 0x46, 0x021BB96C, result
#    callback 0x0215C701), 337 (0xDD / 0xDE / 0xE0), 339 (0xDF), 341 (0xAE), 370 (0x47).
#  * 336 / 489: the same through a field event 02014840 / 02014808(GAMESYS, field map, ovl, proc, param, cb, cbwork) called as a
#    child event (0215897C): 336 ovl 0x88 0x021C78A4 cb 0x0215C7C5, 489 ovl 0xAD 0x021B81D0 cb 0x0215C871.
#  * 340 event 02161B90 (event_field_proclink.c); 356 / 357 events 020124AC(GAMESYS, 2 / 3, 0x02154439 / 0x021543BD, &rec);
#    332 frees the proc work, 355 only yields; 383 / 386 / 390 / 399 / 406 call overlay 37 (0x021F36C0 / 36E8 / 3724 / 3768 /
#    3800) with overlay 0x25 loaded around the call (02034AC4 / 02034A5C).
#  * Roles of the procs (overlay ids, tables in overlays 21 / 37) are not identified here (UNVERIFIED; G2 / U jobs).
#  * 0x0215CC4C (356) reads its caller's r4 / r6 when the operands miss its cases (unset registers; 0 under Z19).
# Not specced: none (no veneers or fragments start in the range; Ghidra's LAB_ labels 0215c6c4 / c712 / c880 / c884 / ccae / ccb2
# / cd10 are branch targets inside 0215c69c / c700 / c870 / cc4c / cce8).
#
# Read from the disassembly of the raw OVY_10.sbin (rizin `-a arm -b 16 -m 0x02155160`; rizin on field.elf mis-disassembles
# mixed ARM / Thumb and Ghidra's field set drops the arguments of overlay calls).  Function list: command-table targets + init-
# proc table entries (0x021705F0) + `bl` targets from the whole field set + odd code pointers + labels, walked as Thumb from every
# entry (only 2-byte padding and literal pools stay uncovered).
#
# Spec conventions (as scrcmd_core.spec / scrcmd_work.spec): every block is `nowork`.  A command handler `fn(vm, ctx)` gets two
# arenas: arg0 = the VM (`arg buf 0x800`: regs at 0x40, script bytes at 0x500, PC +0x14, compare byte +0xE in 0..2), arg1 = the
# scrcmd ctx (`arg buf 0x800`: CTX at 0, SCRCMD WORK at 0x100, zone A 0x300 (field block, SW records), balloon work 0x1D4, zone B
# 0x400, vars 0x600..0x7FF where `02159ae8` points).  A helper taking the ctx in r0 gets the ctx arena as arg0.  A GMEVENT function
# `(event, seq, work)` gets `arg u32, arg buf 0x10 (seq, bounded), arg buf 0x40 (work)`; an init proc `(task, task + 0xC)` gets
# `arg buf 0x100, arg buf 0x10`.  Script-operand readers (vm.c 02011330 / 0201134c, script_sys 02159ae8 / 02159b10) are `seq`
# stubs.  Closure stubs as in core4 (every function of the area treated as ported: no fault, stray write or unstubbed callee).
# **Every stub outside the area carries `force`** (overlay addresses collide with battle `PORTED` entries and the callees in
# script_sys / scrcmd_work are ported; the F8 areas call each other only through forced stubs, so each is verified on its own).
# Indirect calls (`blx rN` through a fn argument) go to the fake callback 0x02EE0011 (stub 02ee0010).  UI callees are `ScriptHost`
# requests in the port (crates/black-script/src/host.rs); results that steer a branch are `pick:` stubs.
# F8-spec3 additions: Heap_AllocDebug (02030734) is a `seqptr` bump allocator into the ctx arena (blocks of a block never
# overlap); its 4th argument is the __FILE__ string (0x021728F0 "scrcmd_proc.c", 0x02172900 "scrcmd_sodateya.c", 0x02172914
# "scrcmd_musical.c"), which places the file boundaries.  Callees of other overlay-10 files, ARM9 main and overlays 11 / 21 / 37
# are force stubs; overlay 37 (0x021F3xxx, outside the field set) is loaded by 02034AC4(0x25) around the call.  An unset callee-
# saved register or stack word that a path reads is 0 under the Z19 entry state; the header of the block says so.
area scrcmd_proc
set field

# 0215c63c(vm, ctx, ovl, proc, param, cb, cbwork) (proc_call_wait): W = Heap_AllocDebug(4, 0xC, 1, "scrcmd_proc.c", 0x8B)
# = {param, cbwork, cb}; *02158FB4(SW) (the SW + 0x3C slot) = W; 020120B4(GAMESYS, ovl, proc, param) (start the app
# proc); wait on 0x0215C69D.  Stack args 5..7 = param, cb, cbwork.  ScriptHost: an app-proc request (UNVERIFIED role).
fn 0215c63c ret=none inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 020120b4 force ret=none args=v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# wait callback of 0215c63c: 020120F0(GAMESYS) (proc running) -> 0; else W = 02158FB8(SW): W.cb -> cb(W), else
# Heap_Free(W.param) / Heap_Free(W.cbwork) when nonzero; 02158FBC(SW) (free W, clear the slot); 1.
fn 0215c69c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x408 u32 alt const 0 || const 0x02EE0011
  stub 02158fbc force ret=none args=v
  stub 020120f0 force ret=pick:0/1 args=v
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158fb8 force ret=ptr:arg1:0x400:1 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 332: 02158FBC(SW) (free the proc work at SW + 0x3C); 1 (yield).
fn 0215c6f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158fbc force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v

# callback of cmd 334 (W): R = W.param, O = W.cbwork = {u16 *a, u16 *b}: *a = R+0x34 != 0, *b = R+0x38;
# Heap_Free(O), Heap_Free(R).
fn 0215c700 ret=none inputs=200
  nowork
  arg buf 0x100
  fix arg0 0 u32 addrof arg0 0x40
  fix arg0 4 u32 addrof arg0 0x80
  fix arg0 0x80 u32 addrof arg0 0xC0
  fix arg0 0x84 u32 addrof arg0 0xC4
  fix arg0 0x74 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v

# cmd 334: m = value (0 -> 4, 1 -> 5, else 0), a, b = vars; R = 0202E474(GAMEDATA, 0, m, 4); O = alloc(8) = {a, b};
# 0215c63c(vm, ctx, 0x46, 0x021BB96C, R, 0x0215C701, O); 1.
fn 0215c72c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=pick:0/1/2/0x30 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0202e474 force ret=u32 args=v,v,v,v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215c63c ret=none args=v,v,v,v,v,v,v
  # closure: 0215c63c
  stub 020113d0 force ret=none args=v,v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 020120b4 force ret=none args=v,v,v,v

# callback of cmd 336 (W): *(u16 *)W[1] = (W[0]+4 == 1); tail call Heap_Free(W[0]).
fn 0215c7c4 ret=none inputs=200
  nowork
  arg buf 0x100
  fix arg0 0 u32 addrof arg0 0x40
  fix arg0 4 u32 addrof arg0 0x80
  fix arg0 0x44 u32 alt range 0 2 || any
  stub 020307b0 force ret=none args=v

# cmd 336: var; A = alloc(8) = {GAMEDATA}, B = alloc(8) = {A, var}; event 02014840(GAMESYS, field map, 0x88, 0x021C78A4,
# A, 0x0215C7C5, B); call it; 1.
fn 0215c7e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02014840 force ret=u32 args=v,v,v,v,v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# callback of cmd 489 (W): P = W[0]: *(u16 *)W[1] = (P+0x1A byte == 1); Heap_Free(P+0x10 word); 0216F940(P) (thunk to
# Heap_Free).
fn 0215c870 ret=none inputs=200
  nowork
  arg buf 0x100
  fix arg0 0 u32 addrof arg0 0x40
  fix arg0 4 u32 addrof arg0 0x80
  fix arg0 0x5a u8 alt range 0 2 || any
  stub 020307b0 force ret=none args=v
  stub 0216f940 force ret=none args=v

# cmd 489: var, i = value; mon = 0201AB28(party, i); x = 02012944(GAMEDATA); y = 0216F948(mon, 4); P = 0216F918(4) =
# {mon, x, 0, GAMESYS, y, .. +0x19 = 1}; W = alloc(8) = {P, var}; event 02014808(GAMESYS, field map, 0xAD, 0x021B81D0, P,
# 0x0215C871, W); call it; 1.
fn 0215c89c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012944 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 force ret=u32 args=v,v
  stub 0216f948 force ret=u32 args=v,v
  stub 02014808 force ret=u32 args=v,v,v,v,v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0216f918 force ret=ptr:arg1:0x3C0:1 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 370: x = 02158344(02012138(GAMESYS)) (comm state record or 0); W = alloc(0x40); if x and (x+0x86 & 7) !=
# 02012B58(GAMEDATA) and 02175FB4(x + 0x1AC + 0x28 * 0215841C(GAMEDATA)) == 1 and x+0x922 == 1: i = 0215841C(GAMEDATA),
# W+4 = 20 halfwords of 0215840C(x, i), W+0x2C = 16 bytes of x+0x914, W+0x3D = 1; else 0200BE48(SAVE), 02175648(GAMEDATA,
# W+0x2C), W+0x3D = 0, i = 0; W[0] = GAMESYS, W+0x3C = i; 0215c63c(vm, ctx, 0x47, 0x021BA758, W, 0, 0); 1.
fn 0215c95c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  arg buf 0x1000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg2 0x86 u16 alt range 0 3 || any
  fix arg2 0x922 u8 alt const 1 || any
  stub 02012ebc force ret=u32 args=v
  stub 02158344 force ret=nullable:2:ptr:arg2:0:1 args=v
  stub 0200be48 force ret=none args=v
  stub 02175648 force ret=none args=v,out:16
  stub 0215840c force ret=ptr:arg2:0x980:1 args=v,v
  stub 0215841c force ret=pick:0/1/2/3 args=v
  stub 02012b58 force ret=pick:0/1/2/3 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 02175fb4 force ret=pick:0/1 args=v
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215c63c ret=none args=v,v,v,v,v,v,v
  # closure: 0215c63c
  stub 020113d0 force ret=none args=v,v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 020120b4 force ret=none args=v,v,v,v

# cmd 340: a, b = values; event 02161B90(GAMESYS, 02158F50(SW), a, b, 0); call it; 1.
fn 0215ca64 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f50 force ret=u32 args=v
  stub 02161b90 force ret=u32 args=v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 341: v = value; W = alloc(8) = {u16 v, GAMEDATA}; 0215c63c(vm, ctx, 0xAE, 0x021B74A0, W, 0, 0); 1.
fn 0215cab4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215c63c ret=none args=v,v,v,v,v,v,v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  # closure: 0215c63c
  stub 020113d0 force ret=none args=v,v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 020120b4 force ret=none args=v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 337: m, v = values; m != 2 -> 02165344(); m 0: W = alloc(8) = {GAMEDATA, v}, proc 0xDD 0x021BA1A4; m 1: the same with
# 0xDE 0x021BA250; m 2: W = alloc(4) = {02012944(GAMEDATA)}, proc 0xE0 0x021B9BF8; (0215c63c(vm, ctx, ovl, proc, W, 0, 0));
# other m: nothing; 1.
fn 0215cb08 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02165344 force ret=none args=
  stub 02012944 force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 020120f4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215c63c ret=none args=v,v,v,v,v,v,v
  # closure: 0215c63c
  stub 020113d0 force ret=none args=v,v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 020120b4 force ret=none args=v,v,v,v
  stub 0215a5f4 force ret=u32 args=v

# cmd 339: var; W = alloc(8) = {var}; 0215c63c(vm, ctx, 0xDF, 0x021BC8C4, W, 0, 0); 1.
fn 0215cbf0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0215c63c ret=none args=v,v,v,v,v,v,v
  # closure: 0215c63c
  stub 020113d0 force ret=none args=v,v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 020120b4 force ret=none args=v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 355: (reads SW / GAMESYS only); 1 (yield).
fn 0215cc38 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 356: t, s = values; rec = {field map, 1, s 0 / 1 -> s, t 15..19 -> t - 15}; event 020124AC(GAMESYS, 2, 0x02154439,
# &rec); call it; 1.  For s > 1 / t outside 15..19 the original stores its caller's r6 / r4 (unset; 0 under the Z19 entry
# state, which the port must take as given).
fn 0215cc4c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020124ac force ret=u32 args=v,v,v,in:16
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/15/16/17/18/19/20 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 357: v = value; rec = {field map, v == 0 ? 1 : 2}; event 020124AC(GAMESYS, 3, 0x021543BD, &rec); call it; 1.
fn 0215cce8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020124ac force ret=u32 args=v,v,v,in:8
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 383: a, b = vars; overlay 0x25 loaded around 021F36C0(GAMESYS, *a, *b) (02034AC4 load / 02034A5C unload); 0.
fn 0215cd3c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f36c0 force ret=none args=v,v,v
  stub 02034ac4 force ret=none args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5dc force ret=u32 args=v
  stub 02034a5c force ret=none args=v

# cmd 386: v = value; overlay 0x25: 021F36E8(GAMESYS, v); 0.
fn 0215cd80 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02034ac4 force ret=none args=v
  stub 021f36e8 force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5dc force ret=u32 args=v
  stub 02034a5c force ret=none args=v

# cmd 390: v = value; overlay 0x25: 021F3724(GAMESYS, v); 0.
fn 0215cdb8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02034ac4 force ret=none args=v
  stub 021f3724 force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5dc force ret=u32 args=v
  stub 02034a5c force ret=none args=v

# cmd 399: 8 vars -> a stack array; overlay 0x25: 021F3768(GAMESYS, &array); 0.
fn 0215cdf0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02034ac4 force ret=none args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021f3768 force ret=none args=v,in:32
  stub 0215a5dc force ret=u32 args=v
  stub 02034a5c force ret=none args=v

# cmd 406: overlay 0x25: 021F3800(GAMESYS); 0.
fn 0215ce70 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3800 force ret=none args=v
  stub 02034ac4 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02034a5c force ret=none args=v
