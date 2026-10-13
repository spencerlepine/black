# fndiff spec: scrcmd core (F8-spec1), overlay 10 Thumb code 0x02159204..0x0215995E: script commands 0..48 and their 4 helpers
# (F1-SPLIT row 10.09 "scrcmd (unanchored)"; the commands of script_work.c that follow at 0x02159960.. are in script_sys).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_core --per-function
# Read from the disassembly (rizin on the field-set ELF; Ghidra's field set drops the arguments of every overlay call).
#
# Commands are fn(vm, ctx) -> 0 (next command) / 1 (yield), dispatched by vm.c 0x02011298 through the table 0x0217062C
# (609 entries, command id = index).  Command ids of this area (id  handler):
#    0 02159240    1 02159244    2 02159248    3 02159260    4 02159284    5 0215929c    6 021592a8    7 021592c0
#    8 021593dc    9 021593f0   10 02159404   11 02159420   12 0215942c   13 0215944c   14 0215946c   15 0215948c
#   16 021594b0   17 021594e0   18 02159854   19 02159874   20 021592d8   21 021592f0   22 0215930c   23 02159340
#   24 02159370   25 02159398   26 021593b8   27 02159570   28 021595cc   29 02159644   30 02159658   31 02159670
#   32 021596b4   33 021596e8   34 02159708   35 02159728   36 02159750   37 02159778   38 021597ac   39 021597cc
#   40 02159894   41 021598ac   42 021598c8   43 021597ec   44 0215980c   45 02159830   46 021598e4   47 021598ec
#   48 02159934
# 53 functions specced (49 command ids): every command-table target in the range, 02159204 (called by command 48),
# 0215932c (the compare of commands 23..26), the wait callbacks 02159254 (command 3) and 021595a0 (command 28).
# Not specced: 0215955c (a case label inside 021594e0; Ghidra/mkelf made it a FUN_), the alignment halfwords between the
# functions.  The range has no other code (a Thumb walk from every entry covers it except padding and literal pools).
#
# VM (main vm.c, layout in vm.spec): +0xC u8 call depth, +0xE u8 compare result (0 <, 1 ==, 2 >; written by commands 23..26,
# read by 31 / 32), +0x14 PC, +0x1C -> regs (u32[0x40]).  The "stack" commands 8..17 use the vm.c call stack as an expression
# stack (0201137C push, 02011398 pop).  Var operands: 02159ae8(vm, ctx) = address of the u16 var whose id is the next script
# word, 02159b10(vm, ctx) = value of the next word read as a var id or literal (script_sys).  Event flags: 020142E8 check,
# 02014314 set, 0201433C clear on the event work 0201296C(GAMEDATA) (black-field event_work).
#
# Spec conventions (as script_sys.spec): every block is `nowork`.  A command's arguments are two arenas:
#   arg0 VM arena 0x800: VM header at 0, regs at 0x40 (+0x1C fixed there; 256 slots so a script byte can index any reg),
#        script bytes at 0x500 (+0x14 fixed there), +0xE (compare result) in 0..2;
#   arg1 ctx arena 0x800: SCRCMD CTX at 0 (+0x20 -> the scrcmd work at 0x100), zone A 0x300 (event work blocks), zone B
#        0x400, var zone 0x600..0x7FF (02159ae8 answers `ptr:arg1:0x600:0x80`).
# vm.c, script_sys and every other callee are stubs (none of them deactivates here: check-fn only sees this area's PORTED);
# operand readers (02011330 / 0201134C / 02011398 / 02159ae8 / 02159b10) are `seq` stubs, so consecutive operands differ.
# Closure stubs: a block lists the stubs of its callees and of the callees of its in-area descendants, so porting order needs
# no spec edit (checked: every function of the area treated as ported runs without a fault, a stray write or an unstubbed
# callee).  Callees in scrcmd_work (0215a684, 0215a690, 0215aa24 / 34 / 48 / 4c) are plain stubs (each area is verified on its
# own).  The tail calls 02159254 -> 0215a690 and 021598e4 -> 021b1bb8 (`ldr r3,=X; bx r3`) land on the stub of X.
area scrcmd_core
set field

# 02159204(ctx): helper of command 48: a task event that re-runs the 14 init procs: event = 020122C0(GAMESYS, 0,
# 0x021591B5 (script_sys 021591b4), 0x18); work = 020124E8(event) = {+0 GAMESYS, +4 ctx, +8 SW, +0xC 0, +0x10 0, +0x14 15};
# returns the event.  arg1 = arena the event work lands in (r1 is not read).
fn 02159204 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x40
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 020122c0 ret=u32 args=v,v,v,v
  stub 020124e8 ret=ptr:arg1:0:1 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0215a5f4 ret=u32 args=v

# 02159240: command 0: no-op (returns 0, next command).
fn 02159240 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100

# 02159244: command 1: no-op (returns 0).
fn 02159244 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100

# 02159248: command 2: end the script (02011290 vm stop) and yield.
fn 02159248 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011290 ret=none args=v

# 02159254(vm, ctx): wait callback of command 3 (`adds r0, r1, 0` + tail call): 0215a690(ctx) (frame countdown).
fn 02159254 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a690 ret=pick:0/1 args=v

# 02159260: command 3: wait N frames: 0215a684(ctx, u16 operand) sets the countdown (scrcmd work +0x106), then
# vm wait (020113D0) on 0x02159255 (02159254 -> 0215a690); yields.
fn 02159260 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 ret=u16 args=v seq
  stub 0215a684 ret=none args=v,v
  stub 020113d0 ret=none args=v,v

# 02159284: command 4: call relative: off = u32 operand; 020113B4(vm, PC + off) (PC after the operand).
fn 02159284 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113b4 ret=none args=v,v
  stub 0201134c ret=u32 args=v seq

# 0215929c: command 5: return (020113C4).
fn 0215929c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113c4 ret=none args=v

# 021592a8: command 6: reads GAMEDATA (unused) and one value operand (02159b10, discarded); no effect but consuming
# the operand (UNVERIFIED: a debug/no-op command).
fn 021592a8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215a5e4 ret=u32 args=v

# 021592c0: command 7: pop the expression stack (02011398) and consume a value operand (discarded); UNVERIFIED.
fn 021592c0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=u32 args=v seq
  stub 02159b10 ret=u16 args=v,v seq

# 021592d8: command 20: reg[b0] = b1 (two script bytes; VM regs = u32 array at +0x1C).
fn 021592d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100

# 021592f0: command 21: reg[b0] = u32 operand.
fn 021592f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201134c ret=u32 args=v seq

# 0215930c: command 22: reg[b0] = reg[b1].
fn 0215930c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100

# 0215932c(a, b): compare: 0 if a < b, 1 if equal, 2 if a > b (unsigned).  Result of the compare commands 23..26
# (stored in the VM byte +0xE, read by the conditional commands 31 / 32).
fn 0215932c ret=u32 inputs=200
  nowork
  arg u32 alt range 0 3 || any
  arg u32 alt rel 0 range 0 2 || range 0 3 || any

# 02159340: command 23: VM+0xE = cmp(reg[b0] & 0xFF, reg[b1] & 0xFF) (0215932c).
fn 02159340 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215932c ret=pick:0/1/2 args=v,v

# 02159370: command 24: VM+0xE = cmp(reg[b0] & 0xFF, b1).
fn 02159370 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215932c ret=pick:0/1/2 args=v,v

# 02159398: command 25: VM+0xE = cmp(var, u16 literal) (var = *02159ae8).
fn 02159398 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02011330 ret=pick:0/1/2/0x8000/0xFFFF args=v seq
  stub 0215932c ret=pick:0/1/2 args=v,v

# 021593b8: command 26: VM+0xE = cmp(var a, var b).
fn 021593b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215932c ret=pick:0/1/2 args=v,v

# 021593dc: command 8: push the u16 literal operand (0201137C) on the VM expression/call stack.
fn 021593dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 ret=u16 args=v seq
  stub 0201137c ret=none args=v,v

# 021593f0: command 9: push the value of a var/literal operand (02159b10).
fn 021593f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 0201137c ret=none args=v,v

# 02159404: command 10: var = pop (02011398; strh).
fn 02159404 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=u32 args=v seq
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq

# 02159420: command 11: pop and discard.
fn 02159420 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=u32 args=v seq

# 0215942c: command 12: a = pop, b = pop, push(b + a).
fn 0215942c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=u32 args=v seq
  stub 0201137c ret=none args=v,v

# 0215944c: command 13: a = pop, b = pop, push(b - a).
fn 0215944c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=u32 args=v seq
  stub 0201137c ret=none args=v,v

# 0215946c: command 14: a = pop, b = pop, push(b * a).
fn 0215946c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=u32 args=v seq
  stub 0201137c ret=none args=v,v

# 0215948c: command 15: a = pop, b = pop, push(b / a) (_u32_div_f: b on a == 0).
fn 0215948c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=pick:0/1/2/3/7/0x100/0xFFFFFFFF/0x12345 args=v seq
  stub 0201137c ret=none args=v,v

# 021594b0: command 16: push(flag set?) = 020142E8(event work 0201296C(GAMEDATA), value) (event flag check).
fn 021594b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 0201296c ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0201137c ret=none args=v,v

# 021594e0: command 17: a = pop, b = pop, op = u16 operand; push(b OP a) with op 0 <, 1 ==, 2 >, 3 <=, 4 >=, 5 !=
# (unsigned), 6 (b == 1 || a == 1), 7 (b == 1 && a == 1); other ops push 0.  Jump table at 0x02159508.
fn 021594e0 ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011398 ret=pick:0/1/2/5 args=v seq
  stub 02011330 ret=pick:0/1/2/3/4/5/6/7/8 args=v seq
  stub 0201137c ret=none args=v,v

# 02159570: command 27: start script (u16 id) in a parallel VM of this script event: 021589a0(SW, zone 02159b38(ctx),
# id); yields (UNVERIFIED role: "run a sub-script without waiting").
fn 02159570 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021589a0 ret=pick:0/1/2/3 args=v,v,v
  stub 02011330 ret=u16 args=v seq
  stub 0215a5f4 ret=u32 args=v
  stub 02159b38 ret=u16 args=v
  stub 0215a5dc ret=u32 args=v

# 021595a0(vm, ctx): wait callback of command 28: 021587e0(SW, ctx byte +0x1A = the VM slot) == 0 (that VM ended) ->
# 0215aa34(ctx) (restore the saved script vars), done 1; else 0.
fn 021595a0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021587e0 ret=pick:0/1 args=v,v
  stub 0215a5f4 ret=u32 args=v
  stub 0215aa34 ret=none args=v
  stub 0215aa4c ret=u8 args=v

# 021595cc: command 28: call a script (u16 id) and wait: ctx mode 0 (0215a5d4 == 0): slot = 021589a0(SW, zone, id),
# 0215aa48(ctx, slot), 0215aa24(ctx) (save script vars), wait on 0x021595A1, yield; mode != 0: 0215aa24, 02158a34(GAMESYS,
# SW, id, 0215a5d8(ctx)) (run it synchronously), 0215aa34 (restore vars), next.
fn 021595cc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021589a0 ret=pick:0/1/2/3 args=v,v,v
  stub 0215aa24 ret=none args=v
  stub 0215a5d8 ret=pick:0/1/2 args=v
  stub 0215aa48 ret=none args=v,v
  stub 02011330 ret=u16 args=v seq
  stub 020113d0 ret=none args=v,v
  stub 0215a5d4 ret=pick:0/1 args=v
  stub 0215a5f4 ret=u32 args=v
  stub 02158a34 ret=none args=v,v,v,v
  stub 0215aa34 ret=none args=v
  stub 02159b38 ret=u16 args=v
  stub 0215a5dc ret=u32 args=v

# 02159644: command 29: end the script (0215a5f4 read unused; 02011290 stop), yield.
fn 02159644 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011290 ret=none args=v
  stub 0215a5f4 ret=u32 args=v

# 02159658: command 30: jump relative: off = u32 operand; 020113B0(vm, PC + off).
fn 02159658 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113b0 ret=none args=v,v
  stub 0201134c ret=u32 args=v seq

# 02159670: command 31: conditional jump: cond = script byte, off = u32 operand; cond 0xFF: jump unless pop() == 1;
# else jump when COND[cond * 3 + VM+0xE] == 1 (6 x 3-byte table at 0x021705DC: < == > <= >= !=).
fn 02159670 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg0 0x500 u8 alt range 0 5 || const 0xFF
  stub 02011398 ret=pick:0/1/2 args=v seq
  stub 0201134c ret=u32 args=v seq
  stub 020113b0 ret=none args=v,v

# 021596b4: command 32: conditional call: cond = script byte, off = u32 operand; COND[cond * 3 + VM+0xE] == 1 ->
# 020113B4(vm, PC + off).
fn 021596b4 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg0 0x500 u8 range 0 5
  stub 020113b4 ret=none args=v,v
  stub 0201134c ret=u32 args=v seq

# 021596e8: command 33: 02014474(02012B38(GAMEDATA), u16 operand) (UNVERIFIED: set a GAMEDATA sub-record value).
fn 021596e8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 ret=u16 args=v seq
  stub 02012b38 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02014474 ret=none args=v,v

# 02159708: command 34: var = 02014460(02012B38(GAMEDATA)).
fn 02159708 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012b38 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02014460 ret=u16 args=v

# 02159728: command 35: set event flag (02014314(event work, value)).
fn 02159728 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201296c ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215a5e4 ret=u32 args=v
  stub 02014314 ret=none args=v,v

# 02159750: command 36: clear event flag (0201433C(event work, value)).
fn 02159750 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201296c ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215a5e4 ret=u32 args=v
  stub 0201433c ret=none args=v,v

# 02159778: command 37: var = event flag (020142E8(event work, value)).
fn 02159778 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201296c ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq

# 021597ac: command 38: var += value (u16).
fn 021597ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 021597cc: command 39: var -= value (u16).
fn 021597cc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 021597ec: command 43: var *= value (u16).
fn 021597ec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 0215980c: command 44: var /= value (_u32_div_f; var unchanged on 0).
fn 0215980c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/3/7/0x100/0xFFFF args=v,v seq

# 02159830: command 45: var %= value (_u32_div_f remainder; 0 on 0).
fn 02159830 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/3/7/0x100/0xFFFF args=v,v seq

# 02159854: command 18: var &= value.
fn 02159854 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 02159874: command 19: var |= value.
fn 02159874 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 02159894: command 40: var = u16 literal.
fn 02159894 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02011330 ret=u16 args=v seq

# 021598ac: command 41: var = var2.
fn 021598ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq

# 021598c8: command 42: var = value (var or literal).
fn 021598c8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 021598e4: command 46: tail call (`ldr r3; bx r3`) to overlay 21 021b1bb8(vm, ctx).
fn 021598e4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b1bb8 ret=u32 args=v,v

# 021598ec: command 47: 021b1be8(ctx); if 020288C4(02012A30(GAMEDATA)): child event 0202972C(GAMESYS, 0) via 0215897c
# (call it) and yield 1, else 0 (UNVERIFIED role).
fn 021598ec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020288c4 ret=pick:0/1 args=v
  stub 021b1be8 ret=none args=v
  stub 0202972c ret=u32 args=v,v
  stub 02012a30 ret=u32 args=v
  stub 0215a5f4 ret=u32 args=v
  stub 020120f4 ret=u32 args=v
  stub 0215897c ret=none args=v,v
  stub 0215a5dc ret=u32 args=v

# 02159934: command 48: event = 02159204(ctx) (re-run the init procs); event != 0 -> 0215897c(SW, event) (call it); yields.
fn 02159934 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215897c ret=none args=v,v
  stub 0215a5dc ret=u32 args=v
  stub 0215a5f4 ret=u32 args=v
  stub 02159204 ret=nullable:3:u32 args=v
  # closure: 02159204
  stub 020122c0 ret=u32 args=v,v,v,v
  stub 020124e8 ret=ptr:arg1:0x300:1 args=v
