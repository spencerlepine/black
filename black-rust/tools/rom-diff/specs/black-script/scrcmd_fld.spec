# fndiff spec: overlay-21 field script commands (F8-spec2), Thumb code 0x021AE948..0x021B0084 (the first part of F1-SPLIT row
# 21.42, "scrcmd (unanchored)", 0x021AE948..0x021B0806; the rest of the row, from the yes / no event on, is scrcmd_talk).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_fld --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id; F1 slot = id + 15):
#   76 021af75c   77 021af7c4   78 021af7e4   79 021af7d4   80 021af844   81 021af878   82 021af928   83 021af958
#   84 021af9a4   85 021af9f0   86 021af8a8   87 021af908   88 021af918   89 021afac8   90 021afafc   91 021afa38
#   92 021afa80   94 021afb2c   95 021afb5c   96 021afbc4   97 021afc18   98 021afc28   99 021afb90  127 021ae9f0
#  128 021ae948  129 021ae994  130 021aea3c  131 021ae9cc  132 021aea8c  133 021aeabc  134 021aeb3c  136 021aebb0
#  138 021aec1c  139 021aec40  142 021aec84  146 021aecc0  147 021aed00  148 021aed24  152 021aede0  155 021aeec0
#  158 021aefd8  159 021af0bc  160 021aef00  161 021af0f8  162 021af130  163 021af458  164 021af484  165 021af4b0
#  166 021af294  167 021af2bc  168 021af2e4  169 021af2f4  170 021af370  171 021af380  172 021af448  181 021af544
#  182 021af594  183 021af5e4  184 021af634  185 021af684  186 021af6c4  187 021af6e8  189 021af71c  190 021afd44
#  191 021afe64  192 021aff7c  193 021afdac  194 021afcd8  195 021afecc  196 021afefc  538 021afc38  539 021afc88
#  584 021aee28  587 021af044  589 021af3d8  595 021aef10  596 021aef60  601 021aee78  602 021b0000  607 021af1b0
# 97 functions specced (80 command ids): trainer-eye / trainer-battle starts (127..148, overlay-21 event_trainer_eye.c and
# overlay-10 0216E818 / 0216EAB0), field events (152..172, 584..607: 02028xxx / 02029xxx event factories, sound 020061E4..),
# the WORDSET commands 76..99 / 538 / 539 (word / number insertion for messages: 0201EExx..0201F1xx, black-text wordset), name /
# string queries 181..189, warps / map changes 190..196 / 602 (02184D38.. / 02185xxx), 4 init procs (slots 10, 12, 13; 021af174 /
# 021aefa4 / 021af1d0) and 13 helpers / wait callbacks.
# Not specced: 021af238 (the tail `strb; pop` of 021af21c that a stray label marks as a function), 021b001a (a mid-function
# address of 021b0000 that a data word in main (0x0209DDE4) happens to point at).
# Notes: 021ae948 uses its caller's r5 when the value is not 0 / 1 (the spec keeps it 0 / 1); 021af794 / 021af8d8 / 021afbec are
# shared bodies entered by `ldr r2, =fn; ldr r3, =body; bx r3` thunks (cmds 77 / 79, 87 / 88, 97 / 98): the thunk blocks stub
# both the body and its real fn.
#
# Read from the disassembly of the raw OVY_21.sbin (rizin `-a arm -b 16 -m 0x02187F00`; rizin on field.elf mis-disassembles
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
# **Every stub outside the area carries `force`** (overlay-21 addresses collide with battle `PORTED` entries; the other F8 areas
# call each other only through forced stubs, so each is verified on its own).  Indirect calls (`blx rN` through a fn argument) go
# to the fake callback 0x02EE0011 (stub 02ee0010).  Window / menu / fade callees (field_msgbg.c 0x021901EC.., fade.c 0x0204E7xx)
# are `ScriptHost` requests in the port (crates/black-script/src/host.rs); results that steer a branch are `pick:` stubs.
#
# Init-proc mask bits (0x02159144 set / 0x02159168 clear / 0x02159190 test; F7 `proc_mask_*`): 1 talk window, 2 balloon window,
# 3 system window, 4 window of cmds 67 / 579, 5 sized info window (cmd 58), 6 window of cmd 69, 9 msgbg flag (cmd 51), 10 / 12 /
# 13 the field events of cmds 161 / 595 / 152 etc.; the init proc of bit N (table 0x021705F0 slot N) closes what the bit says is
# open when a script ends early.
area scrcmd_fld
set field

# cmd 128: trainer-eye record r = 02159124(SW, n) for n = value 0 / 1 (other n: r is the caller's r5, never set: the
# value is bounded to 0 / 1); r +0x18 = 021ae454(heap, field block +4, r, n) (event_trainer_eye.c); 0.
fn 021ae948 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159124 force ret=ptr:arg1:0x300:0x10 args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=pick:0/1 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021ae454 force ret=u32 args=v,v,v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 129: child event 021ae430(GAMESYS, rec0 +0x18, rec1 +0x18) of the two trainer-eye records (02159124(SW, 0 / 1)); yield.
fn 021ae994 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159124 force ret=ptr:arg1:0x300:0x10 args=v,v
  stub 021ae430 force ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 131: var = u16 of trainer-eye record 0 +0x14.
fn 021ae9cc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02159124 force ret=ptr:arg1:0x300:0x10 args=v,v

# cmd 127: n = value; var = n < 2 ? (record n ? u16 record n +0x10 : 0) : 0 (record = 02159124(SW, n)).
fn 021ae9f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/2/0xFFFF args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02159124 force ret=nullable:4:ptr:arg1:0x300:0x10 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# cmd 130: n = value; var = n < 2 and record n set ? 0216D588(*record n) (the MMDL's OBJ id) : 0.
fn 021aea3c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159124 force ret=nullable:4:ptr:arg1:0x300:0x10 args=v,v
  stub 0216d588 force ret=u16 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=pick:0/1/2/0xFFFF args=v,v seq
  stub 0215a5f4 force ret=u32 args=v

# cmd 132: var = 0215A460(script id) (02158F8C(SW); the 02158F58 refresh result is unused).
fn 021aea8c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a460 force ret=u16 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02158f8c force ret=u16 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 133: a, b, c = values; d = 0215A4C8(a); b == 0 and 0215A4B4(a) == 1 -> b = a; child event
# 0216E818(GAMESYS, field block +4, d, 0, a, b, c) (trainer battle; stack args a, b, c); 021aedac(ctx); yield.
fn 021aeabc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a4c8 force ret=u16 args=v
  stub 02158f8c force ret=u16 args=v
  stub 021aedac ret=none args=v
  stub 02159b10 force ret=pick:0/1/2/0x10/0xFFFF args=v,v seq
  stub 0215a4b4 force ret=pick:0/1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0216e818 force ret=u32 args=v,v,v,v,v,v,v
  stub 0215897c force ret=none args=v,v
  # closure: 021aedac
  stub 0215a5e4 force ret=u32 args=v
  stub 02014024 force ret=pick:0/1 args=v
  stub 02012f2c force ret=u32 args=v
  stub 02008df0 force ret=u32 args=v,v
  stub 02188c9c force ret=u32 args=v

# cmd 134: a, b, c, d = values; child event 0216E818(GAMESYS, field block +4, 1, a, b, c, d) (stack args b, c, d);
# 021aedac(ctx); yield.
fn 021aeb3c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f8c force ret=u16 args=v
  stub 021aedac ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0216e818 force ret=u32 args=v,v,v,v,v,v,v
  stub 0215897c force ret=none args=v,v
  # closure: 021aedac
  stub 0215a5e4 force ret=u32 args=v
  stub 02014024 force ret=pick:0/1 args=v
  stub 02012f2c force ret=u32 args=v
  stub 02008df0 force ret=u32 args=v,v
  stub 02188c9c force ret=u32 args=v

# cmd 136: three vars = (0, 2, 0x18) if 0215A4B4(0215A460(script id)) == 0, else (3, 5, 6) if 0215A4A0(script id) == 0,
# else (7, 9, 0xA) (u16 compares).
fn 021aebb0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a460 force ret=u16 args=v
  stub 0215a4a0 force ret=pick:0/1/0x10000 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02158f8c force ret=u16 args=v
  stub 0215a4b4 force ret=pick:0/1/0x10000 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 138: var = 0215A4C8(value).
fn 021aec1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a4c8 force ret=u16 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# cmd 139: child event 02029AF4(GAMESYS, 02029E20(0202A230(value, 1))); yield (02158F54 / GAMEDATA getters unused).
fn 021aec40 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02029e20 force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0202a230 force ret=pick:0/1 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02029af4 force ret=u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 142: child event 021BE92C(GAMESYS, field map, 0, 0, 1, 0, 0) (stack args 1, 0, 0); yield.
fn 021aec84 ret=u32 inputs=200
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

# cmd 146: v = value; var = 0202A230(v, 9) ? 1 : 0202A230(v, 0xB) ? 2 : 0.
fn 021aecc0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0202a230 force ret=pick:0/1 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# cmd 147: var = 0202A230(value, 0xB).
fn 021aed00 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0202a230 force ret=pick:0/1 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# cmd 148: a, b, c, d = values; e = 0215A4C8(a); b == 0 and 0215A4B4(a) == 1 -> b = a; child event
# 0216EAB0(GAMESYS, field block +4, e, 0, a, b, c, d) (stack args a, b, c, d); yield.
fn 021aed24 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a4c8 force ret=u16 args=v
  stub 02158f8c force ret=u16 args=v
  stub 02159b10 force ret=pick:0/1/2/0x10/0xFFFF args=v,v seq
  stub 0216eab0 force ret=u32 args=v,v,v,v,v,v,v,v
  stub 0215a4b4 force ret=pick:0/1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215897c force ret=none args=v,v

# 021aedac(ctx): 02014024(02188C9C(field block +4)) != 0 -> 02008DF0(02012F2C(GAMEDATA), 0x20).
fn 021aedac ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02014024 force ret=pick:0/1 args=v
  stub 02012f2c force ret=u32 args=v
  stub 02008df0 force ret=u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg0:0x300:0x10 args=v
  stub 02188c9c force ret=u32 args=v

# cmd 152: child event 02029984(GAMESYS, u16 operand); 02028BA0(02012A30(GAMEDATA), 2); init-proc bit 13 on; yield.
fn 021aede0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02028ba0 force ret=none args=v,v
  stub 02029984 force ret=u32 args=v,v
  stub 02159144 force ret=none args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02012a30 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 584: child event 02029A04(GAMESYS, u16 a, u16 b); 02028BA0(02012A30(GAMEDATA), 2); init-proc bit 13 on; yield.
fn 021aee28 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02028ba0 force ret=none args=v,v
  stub 02029a04 force ret=u32 args=v,v,v
  stub 02159144 force ret=none args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02012a30 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 601: child event 02029A74(GAMESYS, u16 operand); 02028BA0(02012A30(GAMEDATA), 2); init-proc bit 13 on; yield.
fn 021aee78 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02028ba0 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02012a30 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02029a74 force ret=u32 args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 155: skips a u16 operand; var = 02005F38().
fn 021aeec0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=u16 args=v seq
  stub 02005f38 force ret=pick:0/1 args=
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# wait callback of cmd 160: done (1) once 02005F38() == 0.
fn 021aeee0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02005f38 force ret=pick:0/1 args=
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 160: wait on 0x021AEEE1; yield.
fn 021aef00 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 595: a, b = values; 02028A58(02012A30(GAMEDATA), a & 0xFF, b & 0xFF); init-proc bit 12 on; 0.
fn 021aef10 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159144 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020120f4 force ret=u32 args=v
  stub 02028a58 force ret=none args=v,v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 596: 02028A58(02012A30(GAMEDATA), 0x7F, value & 0xFF); init-proc bit 12 off; 0.
fn 021aef60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159168 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020120f4 force ret=u32 args=v
  stub 02028a58 force ret=none args=v,v,v
  stub 0215a5dc force ret=u32 args=v

# init proc (0x021705F0 slot 12; task, seq): seq 0: 02028A58(02012A30(GAMEDATA of *task), 0x7F, 0), seq = 1, 0; seq 1: 1;
# other: 0.
fn 021aefa4 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  stub 02012a30 force ret=u32 args=v
  stub 02028a58 force ret=none args=v,v,v
  stub 020120f4 force ret=u32 args=v

# cmd 158: child event 020298A0(GAMESYS, 02029DF4(GAMEDATA, 02188C9C(field map), 02012984(GAMEDATA)), 0x5A, 0x3C);
# 02028BDC(02012A30(GAMEDATA), 2); init-proc bit 13 off; yield.
fn 021aefd8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020298a0 force ret=u32 args=v,v,v,v
  stub 02012984 force ret=u16 args=v
  stub 02159168 force ret=none args=v
  stub 02028bdc force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02188c9c force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02029df4 force ret=u32 args=v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 587: as cmd 158 with a u16 operand instead of 0x5A: child event 020298A0(GAMESYS, 02029DF4(..), operand, 0x3C).
fn 021af044 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020298a0 force ret=u32 args=v,v,v,v
  stub 02012984 force ret=u16 args=v
  stub 02159168 force ret=none args=v
  stub 02028bdc force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02188c9c force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02029df4 force ret=u32 args=v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 159: child event 02029AF4(GAMESYS, value); yield.
fn 021af0bc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02029af4 force ret=u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 161: child event 020295A4(GAMESYS, value); init-proc bit 10 on; yield.
fn 021af0f8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020295a4 force ret=u32 args=v,v
  stub 02159144 force ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 162: child event 02029620(GAMESYS, a, b) (values); init-proc bit 10 off; yield.
fn 021af130 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02029620 force ret=u32 args=v,v,v
  stub 02159168 force ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# init proc (slot 10; task, seq): seq 0: child event 02029620(*task, 0, 0) of the SW at task +8, seq = 1, 0; seq 1: 1;
# other: 0.
fn 021af174 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02029620 force ret=u32 args=v,v,v
  stub 0215897c force ret=none args=v,v

# cmd 607: 02028BDC(02012A30(GAMEDATA), 2); init-proc bit 13 off; 0.
fn 021af1b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159168 force ret=none args=v
  stub 02012a30 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02028bdc force ret=none args=v,v

# init proc (slot 13; task): 02028BDC(02012A30(GAMEDATA of *task), 2); 1.
fn 021af1d0 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 02012a30 force ret=u32 args=v
  stub 02028bdc force ret=none args=v,v
  stub 020120f4 force ret=u32 args=v

# 021af1e8(SW): for bit i < 5 set in the byte at SW +0x40 (02158F88): 02006224(i) == 0 -> clear the bit.
fn 021af1e8 ret=none inputs=200
  nowork
  arg buf 0x100
  stub 02158f88 force ret=ptr:arg0:0x40:1 args=v
  stub 02006224 force ret=pick:0/1 args=v

# 021af21c(SW, x): set bit 02006148(x) of the byte at SW +0x40.
fn 021af21c ret=none inputs=200
  nowork
  arg buf 0x100
  arg u32 any
  stub 02158f88 force ret=ptr:arg0:0x40:1 args=v
  stub 02006148 force ret=pick:0/1/2/3/4 args=v

# 021af23c(SW): 1 if some bit i < 5 of the byte at SW +0x40 has 02006224(i) == 1; else the byte = 0 and 0.
fn 021af23c ret=u32 inputs=200
  nowork
  arg buf 0x100
  stub 02158f88 force ret=ptr:arg0:0x40:1 args=v
  stub 02006224 force ret=pick:0/1 args=v

# 021af26c(SW): 020061F8(i) for every bit i < 5 set in the byte at SW +0x40; the byte = 0.
fn 021af26c ret=none inputs=200
  nowork
  arg buf 0x100
  stub 02158f88 force ret=ptr:arg0:0x40:1 args=v
  stub 020061f8 force ret=none args=v

# cmd 166: v = value; 021af1e8(SW); 020061E4(v); 021af21c(SW, v); 0.
fn 021af294 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020061e4 force ret=none args=v
  stub 021af1e8 ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021af21c ret=none args=v,v
  # closure: 021af1e8 / 021af21c
  stub 02158f88 force ret=ptr:arg0:0x40:1 args=v
  stub 02006224 force ret=pick:0/1 args=v
  stub 02006148 force ret=pick:0/1/2/3/4 args=v

# cmd 167: 021af26c(SW); 0.
fn 021af2bc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021af26c ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  # closure: 021af26c
  stub 02158f88 force ret=ptr:arg1:0x40:1 args=v
  stub 020061f8 force ret=none args=v

# wait callback of cmd 168: done once 021af23c(SW) == 0.
fn 021af2cc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5f4 force ret=u32 args=v
  stub 021af23c ret=pick:0/1 args=v
  # closure: 021af23c
  stub 02158f88 force ret=ptr:arg1:0x40:1 args=v
  stub 02006224 force ret=pick:0/1 args=v

# cmd 168: wait on 0x021AF2CD; yield.
fn 021af2e4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 169: child event 02029830(GAMESYS, u16 operand); yield.
fn 021af2f4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=u16 args=v seq
  stub 02029830 force ret=u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# wait callback of cmd 170: 02005F38() == 0 and 020289B8(02012A30(GAMEDATA)) == 0 -> child event 02029620(GAMESYS, 0, 6),
# done; else 0.
fn 021af324 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02029620 force ret=u32 args=v,v,v
  stub 020289b8 force ret=pick:0/1 args=v
  stub 02012a30 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02005f38 force ret=pick:0/1 args=
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 170: wait on 0x021AF325; yield.
fn 021af370 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 171: a, b = values; child event E = 020122C0(GAMESYS, 0, 0x021AF4E9, 0xC) with work {u16 a, u16 b, 0, ctx}; yield.
fn 021af380 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 589: as cmd 171 with work +4 = 1.
fn 021af3d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# wait callback of cmd 172: done once 02006BD0(scrcmd work +0x108) == 0.
fn 021af430 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02006bd0 force ret=pick:0/1 args=v
  stub 0215a6ac force ret=u32 args=v

# cmd 172: wait on 0x021AF431; yield.
fn 021af448 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 163: 0202C0D0(0202B578(02012134(GAMESYS)), value); 0.
fn 021af458 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0202c0d0 force ret=none args=v,v
  stub 02012134 force ret=u32 args=v
  stub 0202b578 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 164: 0202C0E4(0202B578(02012134(GAMESYS)), value); 0.
fn 021af484 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0202c0e4 force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02012134 force ret=u32 args=v
  stub 0202b578 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 165: var = 0202C0F8(0202B578(02012134(GAMESYS)), value).
fn 021af4b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0202c0f8 force ret=u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02012134 force ret=u32 args=v
  stub 0202b578 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT fn of cmds 171 / 589 (event, seq, work {u16 a, u16 b, u32 mode, ctx}): 02005C64() != 0 -> 0 (wait); else
# 0215A6B8(ctx, 02006984(..)) (scrcmd work +0x108 = the handle): mode != 0 -> 02006CAC(&tmp) and
# 02006984(b, a, 0x40, 0, 0, 0, 0, &tmp), else 02006984(a, b, 0x40, 0, 0, 0, 0, 0); 1 (done).
fn 021af4e8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  fix arg2 4 u32 alt const 0 || any
  stub 0215a6b8 force ret=none args=v,v
  stub 02006984 force ret=u32 args=v,v,v,v,v,v,v,p
  stub 02006cac force ret=none args=out:4
  stub 02005c64 force ret=pick:0/1 args=

# cmd 181: a, b = values; var = 02007E50(0201292C(GAMEDATA), a, b, heap).
fn 021af544 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201292c force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02007e50 force ret=u32 args=v,v,v,v

# cmd 182: a, b = values; var = 02007F1C(0201292C(GAMEDATA), a, b, heap).
fn 021af594 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201292c force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02007f1c force ret=u32 args=v,v,v,v

# cmd 183: a, b = values; var = 02007E3C(0201292C(GAMEDATA), a, b, heap).
fn 021af5e4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201292c force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02007e3c force ret=u32 args=v,v,v,v

# cmd 184: a, b = values; var = 02007F68(0201292C(GAMEDATA), a, b, heap).
fn 021af634 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02007f68 force ret=u32 args=v,v,v,v
  stub 0201292c force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq

# cmd 185: var = 02007FB8(0201292C(GAMEDATA), value, heap).
fn 021af684 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201292c force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02007fb8 force ret=u32 args=v,v,v

# cmd 186: var = 021af750(value) (= u16 02021288(value)).
fn 021af6c4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 021af750 ret=u16 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  # closure: 021af750
  stub 02021288 force ret=u32 args=v

# cmd 187: var = 02020FE4(value, 5, heap).
fn 021af6e8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5cc force ret=u16 args=v
  stub 02020fe4 force ret=pick:4/0/1 args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# cmd 189: var = 02020FE4(value, 0xF, heap).
fn 021af71c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5cc force ret=u16 args=v
  stub 02020fe4 force ret=pick:4/0/1 args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# 021af750(x): u16 02021288(x).
fn 021af750 ret=u16 inputs=200
  nowork
  arg u32 any
  stub 02021288 force ret=u32 args=v

# cmd 76: b = script byte; 0201F02C(WORDSET, b, 02012944(GAMEDATA)) (02158F58 refresh unused); 0.
fn 021af75c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0201f02c force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# 021af794(vm, ctx, fn): shared body of cmds 77 / 79 (`ldr r2, =fn; ldr r3, =0x021AF795; bx r3`): b = script byte,
# v = value; fn(WORDSET, b, v); 0.  arg2 = fake callback 0x02EE0011 (stub 02ee0010).
fn 021af794 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  arg u32 const 0x02EE0011
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v,v,v

# cmd 77: 021af794 with fn = 0x0201EF41 (WORDSET word set, 0201EF40(WORDSET, b, v)).
fn 021af7c4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021af794 ret=none args=v
  stub 0201ef40 force ret=u32 args=v,v,v
  # closure: 021af794
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 79: 021af794 with fn = 0x0201EF4D (0201EF4C(WORDSET, b, v)).
fn 021af7d4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021af794 ret=none args=v
  stub 0201ef4c force ret=u32 args=v,v,v
  # closure: 021af794
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 78: b = script byte, v = value, w = value, c = script byte: w > 1 -> 0201EF5C(WORDSET, b, v); c != 0 ->
# 0201EF4C(WORDSET, b, v); else 0201EF40(WORDSET, b, v); returns that call's r0.
fn 021af7e4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg0 0x501 u8 alt const 0 || any
  stub 0201ef40 force ret=u32 args=v,v,v
  stub 0201ef4c force ret=u32 args=v,v,v
  stub 02159b10 force ret=pick:0/1/2/5/0xFFFF args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201ef5c force ret=u32 args=v,v,v

# cmd 80: b = script byte; 0201EF34(WORDSET, b, 020212B0(value)); 0.
fn 021af844 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020212b0 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0201ef34 force ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v

# cmd 81: b = script byte; 0201EF34(WORDSET, b, value); 0.
fn 021af878 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0201ef34 force ret=none args=v,v,v

# cmd 86: b = script byte; 0201F020(WORDSET, b, value); 0.
fn 021af8a8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0201f020 force ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v

# 021af8d8(vm, ctx, fn): shared body of cmds 87 / 88 (same shape as 021af794): fn(WORDSET, script byte, value); 0.
# arg2 = fake callback 0x02EE0011.
fn 021af8d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  arg u32 const 0x02EE0011
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v,v,v

# cmd 87: 021af8d8 with fn = 0x0201EE61 (0201EE60).
fn 021af908 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021af8d8 ret=none args=v
  stub 0201ee60 force ret=none args=v
  # closure: 021af8d8
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 88: 021af8d8 with fn = 0x0201EEC5 (0201EEC4).
fn 021af918 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021af8d8 ret=none args=v
  stub 0201eec4 force ret=none args=v
  # closure: 021af8d8
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 82: b = script byte; 0201F14C(WORDSET, b, value); 0.
fn 021af928 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0201f14c force ret=none args=v,v,v

# cmd 83: b = script byte; 0201EE84(WORDSET, b, 0201AB28(party (02012934(GAMEDATA)), value)) (mon nickname).
fn 021af958 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ee84 force ret=none args=v,v,v
  stub 0201ab28 force ret=nullable:5:u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v

# cmd 84: as cmd 83 with 0201EEF4 (another word of the party mon).
fn 021af9a4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab28 force ret=nullable:5:u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201eef4 force ret=none args=v,v,v

# cmd 85: b = script byte; 0201EE84(WORDSET, b, 021C34F8(02188D50(field map), value)) (a daycare mon, sodateya.c).
fn 021af9f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201ee84 force ret=none args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 91: as cmd 85 with 0201EEF4.
fn 021afa38 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188d50 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201eef4 force ret=none args=v,v,v
  stub 021c34f8 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 92: b = script byte; v, n = values; 0201EFA4(WORDSET, b, v, n, 0, 1) (number word: stack args 0, 1).
fn 021afa80 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v

# cmd 89: b = script byte; 0201F158(WORDSET, b, 02013E4C(value)).
fn 021afac8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02013e4c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201f158 force ret=none args=v,v,v

# cmd 90: b = script byte; 0201EFC8(WORDSET, b, value); 0.
fn 021afafc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0201efc8 force ret=none args=v,v,v

# cmd 94: b = script byte; 0201F048(WORDSET, b, value); 0.
fn 021afb2c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0201f048 force ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v

# cmd 95: b = script byte; 0201F198(WORDSET, b, value & 0xFF); 0.
fn 021afb5c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0201f198 force ret=none args=v,v,v

# cmd 99: b = script byte; 0201F1B0(WORDSET, b, value & 0xFF); 0.
fn 021afb90 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f74 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0201f1b0 force ret=none args=v,v,v

# cmd 96: b = script byte; 0201F184(WORDSET, b, value); 0.
fn 021afbc4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0201f184 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# 021afbec(vm, ctx, fn): shared body of cmds 97 / 98: b = script byte; fn(WORDSET, b, value & 0xFF); 0.  arg2 = fake
# callback 0x02EE0011.
fn 021afbec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  arg u32 const 0x02EE0011
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v,v,v

# cmd 97: 021afbec with fn = 0x0201F0F9 (0201F0F8).
fn 021afc18 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021afbec ret=none args=v
  stub 0201f0f8 force ret=none args=v
  # closure: 021afbec
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 98: 021afbec with fn = 0x0201F125 (0201F124).
fn 021afc28 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021afbec ret=none args=v
  stub 0201f124 force ret=none args=v
  # closure: 021afbec
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02ee0010 force ret=none args=v

# cmd 538: b = script byte; 0201EFA4(WORDSET, b, 0200C1DC(0200C0F0(02012EBC(GAMEDATA))), 9, 1, 1); 0.
fn 021afc38 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 0200c0f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0200c1dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 539: as cmd 538 with 0200C1C4.
fn 021afc88 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0200c1c4 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 0200c0f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 194: a, b, c, d = u16 operands; pos = {b << 16, 0, c << 16}; 02012374(02012104(GAMESYS), 02184D38(GAMESYS,
# field map, a, &pos, d, 0)) (call a field event); yield.
fn 021afcd8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v
  stub 02184d38 force ret=u32 args=v,v,v,in:12,v,v

# cmd 190: a, b, c, d = u16 operands; pos = {b << 16, 0, c << 16}; 02012374(02012104(GAMESYS), 02185534(GAMESYS,
# field map, a, &pos, d)); yield.
fn 021afd44 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02185534 force ret=u32 args=v,v,v,in:12,v
  stub 02012374 force ret=none args=v,v

# cmd 193: a, b, c = u16 operands; 021A3038(02188CB4(field map), &p1) (player position); 0216282C(02012844(GAMEDATA),
# 0201296C(GAMEDATA), &p1, 9) != 0 -> 02162A6C(it, &p2); pos = {b << 16, 0, c << 16}; 02012374(02012104(GAMESYS),
# 02184E40(GAMESYS, field map, &p2, a, &pos)); 021BB578(02188CD0(field map), 021A3074(..), 0, 1); yield.
fn 021afdac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02184e40 force ret=u32 args=v,v,in:12,v,in:12
  stub 02012844 force ret=u32 args=v
  stub 02012104 force ret=u32 args=v
  stub 021bb578 force ret=none args=v,v,v,v
  stub 0201210c force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 0216282c force ret=nullable:2:u32 args=v,v,in:0xC,v
  stub 02162a6c force ret=none args=v,out:0xC
  stub 02011330 force ret=u16 args=v seq
  stub 02188cd0 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021a3038 force ret=none args=v,out:0xC
  stub 02012374 force ret=none args=v,v
  stub 021a3074 force ret=u32 args=v

# cmd 191: a, b, c, d = u16 operands; pos = {b << 16, 0, c << 16}; 02012374(02012104(GAMESYS), 02185050(GAMESYS,
# field map, a, &pos, d)); yield.
fn 021afe64 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02185050 force ret=u32 args=v,v,v,in:12,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v

# cmd 195: 02012374(02012104(GAMESYS), 021850A8(GAMESYS, field map)); yield.
fn 021afecc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 021850a8 force ret=u32 args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v

# cmd 196: a, b, c, d, e = u16 operands; 021A3074(02188CB4(field map)) (unused); pos = {b << 16, c << 16, d << 16};
# 02012374(02012104(GAMESYS), 021854B0(GAMESYS, field map, a, &pos, e)); yield.
fn 021afefc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 021854b0 force ret=u32 args=v,v,v,in:12,v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v

# cmd 192: a..e = u16 operands; 021BB3E8(02188C5C(field map), a, b, c, d, &loc, 0x15) (stack args d, &loc, 0x15);
# 02012374(02012104(GAMESYS), 02184D90(GAMESYS, field map, a, &loc, e, 0)); yield.
fn 021aff7c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 021bb3e8 force ret=none args=v,v,v,v,v,out:0xC,v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 02184d90 force ret=u32 args=v,v,v,in:0xC,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v
  stub 02188c5c force ret=u32 args=v

# cmd 602: as cmd 192 with 02184DE8 instead of 02184D90.
fn 021b0000 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012104 force ret=u32 args=v
  stub 021bb3e8 force ret=none args=v,v,v,v,v,out:0xC,v
  stub 02184de8 force ret=u32 args=v,v,v,in:0xC,v,v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v
  stub 02188c5c force ret=u32 args=v
