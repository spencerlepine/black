# fndiff spec: the unanchored overlay-10 script commands (F8-spec3): F1-SPLIT row 10.33 (0x0216A420..0x0216AC40, 21 fns), row
# 10.35 (0x0216B504..0x0216B86C, 11 fns) and cmd 359 at 0x0215D34C..0x0215D452 (the first function of scrcmd_musical.c, X2:
# specced because F1 put it in row 10.16; not to be ported unless musicals come into scope).  Grammar: svflow_specs/README.md
# (generic mode).  Run: tools/check-fn.sh black-script scrcmd_misc10 --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  359 0215d34c  506 0216a420  507 0216a440  508 0216a470  509 0216a4cc  510 0216a510  511 0216a560  512 0216a594
#  513 0216a5c4  514 0216a5fc  515 0216a634  516 0216a674  517 0216a6b4  518 0216a6ec  519 0216a72c  520 0216a774
#  528 0216b63c  529 0216b6dc  530 0216b578  531 0216b5ac  532 0216b720  533 0216b81c  535 0216b60c  540 0216b7b0
#  541 0216b7f8  542 0216b504  543 0216b538  549 0216a7a0  550 0216a884  559 0216abd8
# 33 functions specced (30 command ids).
#  * 506..520: calls into overlay 21 0x021D0A48..0x021D1028 and overlay-10 events 02168BF0 / 02168CF4 / 02168E5C
#    (fld_btl_inst_tool.c row 10.30) on the record *02012F50(GAMEDATA); 513 / 514 the save block 02008C58 (block 0x26).  Roles
#    UNVERIFIED (the callee files are F10 jobs).
#  * 549 / 550: move a field object: GMEVENT 0x0216A838 (vertical move by d / 2 tiles over n frames, signed divide 0209C0A4) and
#    0x0216A8F0 (a 3-phase curved path from FX_SinCosTable 0x020A1B38, 0x6E + 0x50 + 0x50 frames).
#  * 0x0216AAE4(x, party, season) (called from outside the area): seasonal forms of Deerling (0x249) / Sawsbuck (0x24A) in the
#    party + Pokedex registration.
#  * 559: report_event.c (rows 10.34 / 10.36) lookups 0216AED0 / 0216AF1C and event 0216AF74.
#  * 528..543: save block 0x34 (0200C0F0) / 0x3F (0200E394) accessors 0200C224..0200C384 / 0200E3A0..0200E4F8; 532 picks the
#    best row of the ROM tables 0x0217219C (30 counts) / 0x021721BC (17-byte rows); 541 event 021670F8.  Roles UNVERIFIED.
#  * 0x0216B720 (532) stores an unset stack word when no row wins (0 under Z19).
# Not specced: none; LAB_ 0216a87c / a91c / a91e / a9b8 / aa40 / b560 and 0215d3a0 are branch targets inside 0216a838 / a8f0 /
# b538 / 0215d34c.
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
area scrcmd_misc10
set field

# cmd 359 (scrcmd_musical.c, X2): v = value, b = script byte; 02158F58(SW); b <= 1: m = 02012F80(GAMEDATA), event =
# 020122C0(GAMESYS, 0, 0x0215DE59, 0xC), W = {02155A98(GAMESYS, GAMEDATA, v & 0xFF, b != 0, m), m, b != 0} (zeroed first),
# 0215898C(SW, event) (end callback), 02012A30(GAMEDATA), 02011290(vm); b >= 2: A = alloc(4) -> B = alloc(0x14) = {0, 1,
# 0200A640(02012F44(GAMEDATA)) (`bx lr`)}, call event 02014808(GAMESYS, field map, 0x70, 0x021F8360, B, 0x0215DF6D, A); 1.
fn 0215d34c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg0 0x500 u8 alt range 0 3 || any
  stub 02012f80 force ret=u32 args=v
  stub 02014808 force ret=u32 args=v,v,v,v,v,v,v
  stub 0201210c force ret=u32 args=v
  stub 0215898c force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011290 force ret=none args=v
  stub 02155a98 force ret=u32 args=v,v,v,v,v
  stub 02012a30 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x20 args=v,v,v,v,v
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0200a640 force ret=u32 args=v
  stub 02012f44 force ret=u32 args=v
  stub 02158f58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 020124e8 force ret=ptr:arg1:0x400:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 506: 021D0A48(GAMESYS, 02012F50(GAMEDATA)); 0.
fn 0216a420 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 021d0a48 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v

# cmd 507: v = value; 021D0A74(*02012F50(GAMEDATA), v); 0.
fn 0216a440 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021d0a74 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 508: a, b = values, var; event 02168BF0(GAMESYS, *02012F50(GAMEDATA), a, b, var); call it; 1.
fn 0216a470 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02168bf0 force ret=u32 args=v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 509: v = value, var; var = 021D0AC0(GAMESYS, *02012F50(GAMEDATA), v); 0.
fn 0216a4cc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021d0ac0 force ret=u16 args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 510: a, b = values; event 021D0BD8(GAMESYS, *02012F50(GAMEDATA), a, b); call it; 1.
fn 0216a510 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021d0bd8 force ret=u32 args=v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 511: event 02168CF4(GAMESYS, *02012F50(GAMEDATA)); call it; 1.
fn 0216a560 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02168cf4 force ret=u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 512: v = value; 021D0AB8(*02012F50(GAMEDATA), v); 0.
fn 0216a594 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021d0ab8 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 513: v = value; 02008E68(02008C58(SAVE), v & 0xFF); 0.
fn 0216a5c4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02008e68 force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02008c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 514: var = 02008E78(02008C58(SAVE)); 0.
fn 0216a5fc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02008e78 force ret=u16 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02008c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 515: var; event 021D0BE4(GAMESYS, *02012F50(GAMEDATA), var); call it; 1.
fn 0216a634 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021d0be4 force ret=u32 args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 516: a, b = vars; 021D0DB0(GAMESYS, *02012F50(GAMEDATA), a, b); 0.
fn 0216a674 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 021d0db0 force ret=none args=v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 517: var = 021D0FE0(GAMESYS, *02012F50(GAMEDATA)); 0.
fn 0216a6b4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021d0fe0 force ret=u16 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 518: v = value; event 021D0FE8(GAMESYS, *02012F50(GAMEDATA), v); call it; 1.
fn 0216a6ec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021d0fe8 force ret=u32 args=v,v,v
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 519: 02012F50(GAMEDATA) (unused); a, b = values; event 02168E5C(GAMESYS, a, b); call it; 1.
fn 0216a72c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02168e5c force ret=u32 args=v,v,v

# cmd 520: var = 021D1028(GAMESYS); 0.
fn 0216a774 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021d1028 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 549: id, d, n = values; event = 020122C0(GAMESYS, 0, 0x0216A839, 0x18), W = {.., +4 u16 n, +6 u16 0, +8 mmdl, +0xC pos};
# mmdl = 0216DE94(02188C7C(field map), id); 0216D88C(mmdl, &W.pos); 0216D568(mmdl, 0x8000); W[0] = (d << 15) - W.pos.y;
# call it; 1.  (Moves an object vertically by d / 2 tiles over n frames.)
fn 0216a7a0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 0216d568 force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188c7c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT fn of cmd 549 (event, seq, W): W.n_done += 1; pos = W.pos; pos.y += W[0] * n_done / n (0209C0A4 signed divide);
# 0216D89C(W.mmdl, &pos); done (1) once n_done >= n, else 0.
fn 0216a838 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg2 4 u16 range 1 0x40
  fix arg2 6 u16 alt range 0 0x40 || any
  stub 0216d89c force ret=none args=v,in:12

# cmd 550: id = value; event = 020122C0(GAMESYS, 0, 0x0216A8F1, 0x14), W = {0, mmdl, pos}; mmdl = 0216DE94(02188C7C(field
# map), id); 0216D88C(mmdl, &W.pos); call it; 1.
fn 0216a884 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188c7c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT fn of cmd 550 (event, seq, W = {t, mmdl, pos}): t += 1; *seq 0: pos += (sin, cos)(t * 0x20000 / 0x6E) scaled by
# 3t * 0x10000 / 0x6E (FX_SinCosTable 0x020A1B38, 64-bit products 0209C054), t >= 0x6E -> t = 0, seq 1; 1: pos += the
# same with t * 0x10000 / 0x50 and a fixed 3.0, t >= 0x50 -> seq 2; 2: radius (0x50 - t) * 3 * 0x10000 / 0x50, done once
# t >= 0x50; 0216D89C(mmdl, &pos) every step; other seq: 0.
fn 0216a8f0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 0 u32 alt range 0 0x70 || range 0x6C 0x70
  stub 0216d89c force ret=none args=v,in:12

# 0216aae4(x, party, season): the seasonal forms: form = season 0..3 (else 0); for each party mon: species 0x249 (Deerling)
# or 0x24A (Sawsbuck) with get(mon, 0x6F) != form -> 02017988(mon, form) (set form) and, first time per species,
# 0200CFA0(0200CA1C(x), mon) (Pokedex register).
fn 0216aae4 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt range 0 4 || any
  stub 0200cfa0 force ret=none args=v,v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02017988 force ret=none args=v,v
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/3/0x249/0x24A args=v,v,v

# cmd 559: v = value, var; p = 0216AED0(ctx, v); p and (q = 0216AF1C(ctx, p, 4, &k)) -> call event 0216AF74(GAMESYS,
# field map, q, k, var, 4); 1.
fn 0216abd8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216aed0 force ret=nullable:3:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216af74 force ret=u32 args=v,v,v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0216af1c force ret=nullable:3:u32 args=v,v,v,out:4

# cmd 542: var = 0200C224(0200C0F0(SAVE)); 0.
fn 0216b504 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c224 force ret=pick:0/1/4/5/6 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200c0f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 543: s = 0200C0F0(SAVE), n = 0200C224(s); n < 5 -> 0200C22C(s, n + 1), 02027138((n + 1) & 0xFF); 0.
fn 0216b538 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c224 force ret=pick:0/1/4/5/6 args=v
  stub 0200c22c force ret=none args=v,v
  stub 0200c0f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02027138 force ret=none args=v
  stub 02012ebc force ret=u32 args=v

# cmd 530: var = 0200C32C(0200C0F0(SAVE)); 0.
fn 0216b578 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200c32c force ret=pick:0/1/2 args=v
  stub 0200c0f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 531: a, b, c = vars; *a, *b, *c = 0200C33C(s, 0 / 1 / 2) with s = 0200C0F0(SAVE); 0.
fn 0216b5ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c33c force ret=u16 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200c0f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 535: var = 0216BB18(SAVE); 0.
fn 0216b60c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0216bb18 force ret=u16 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 528: s = 0200C0F0(SAVE); 0200E394(SAVE) (unused); k, a0, a1, a2 = values; 0200C334(s, k & 0xFF); for i 0..2:
# 0200C344(s, i, a_i & 0xFF), 0200C358(s, i, 0); 0200C384(s, clock lo, hi) (0203F524 u64); 0.
fn 0216b63c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c344 force ret=none args=v,v,v
  stub 0203f524 force ret=u64 args=
  stub 0200c384 force ret=none args=v,v,v
  stub 0200c0f0 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200e394 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0200c334 force ret=none args=v,v
  stub 0200c358 force ret=none args=v,v,v
  stub 02012ebc force ret=u32 args=v

# cmd 529: s = 0200C0F0(SAVE); 0200E394(SAVE) (unused); 0200C334(s, 0); 0200C344(s, i, 0xFF) for i 0..2; 0.
fn 0216b6dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c344 force ret=none args=v,v,v
  stub 0200c0f0 force ret=u32 args=v
  stub 0200e394 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0200c334 force ret=none args=v,v
  stub 02012ebc force ret=u32 args=v

# cmd 532: e = 0200E394(SAVE); c = value, var; n = byte table 0x0217219C[c]; over k = 1..n the max (signed, from -1) of
# 0200E4F8(e, c, k) + 0200E4B0(e, c, k) picks row byte 0x021721BC[17 c + k - 1]; var = that byte (none: the unset stack word,
# 0 under the Z19 entry state); 0.
fn 0216b720 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=pick:0/1/7/12/19/22/25/27/28/29/30 args=v,v seq
  stub 0200e4b0 force ret=u32 args=v,v,v
  stub 0200e394 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0200e4f8 force ret=u32 args=v,v,v
  stub 02012ebc force ret=u32 args=v

# cmd 540: x = 0200E3A0(0200E394(SAVE)); a, b = values; 0200E3E8(x, a & 0xFF, b); 02027100(x); 0.
fn 0216b7b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200e3a0 force ret=u32 args=v
  stub 02027100 force ret=none args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 0200e3e8 force ret=none args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200e394 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 541: event 021670F8(GAMESYS); call it; 1.
fn 0216b7f8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021670f8 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 533: s = 0200C0E8(SAVE); v = value & 0xFF, var; v == 0200C32C(s) -> 0216B8B8(SAVE, var), else 0216B91C(SAVE, v, var); 0.
fn 0216b81c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c0e8 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200c32c force ret=pick:0/1/2 args=v
  stub 02159b10 force ret=pick:0/1/2/0x102 args=v,v seq
  stub 0216b91c force ret=none args=v,v,v
  stub 0216b8b8 force ret=none args=v,v
  stub 02012ebc force ret=u32 args=v
