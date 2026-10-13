# fndiff spec: overlay-21 message / yes-no / menu script commands (F8-spec2), Thumb code 0x021B0084..0x021B1668: the tail of
# F1-SPLIT row 21.42 (0x021B0084..0x021B0806), all of row 21.43 (0x021B0808..0x021B0D12: not event_trainer_eye.c but the
# balloon-window placement helpers 021b0808..021b0a40 of these commands) and the head of row 21.44 (0x021B0D14..0x021B1668).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_talk --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#   51 021b15c8   52 021b04d8   53 021b0530   54 021b0624   55 021b05ac   56 021b10d0   57 021b10e8   58 021b1100
#   59 021b11d8   60 021b0d14   61 021b0d74   62 021b0edc   63 021b14f4   64 021b063c   65 021b07e0   66 021b0730
#   67 021b12b0   68 021b13e0   69 021b143c   70 021b14dc   71 021b0130   72 021b0df0   73 021b0e74   74 021b10dc
#   75 021b15b8  135 021b0ef4  173 021b01dc  174 021b024c  175 021b0344  176 021b0394  177 021b03bc  178 021b02c8
#  579 021b1334
# 70 functions specced (33 command ids).  Roles (ScriptHost mapping in host.rs):
#  * 71 YES_NO: GMEVENT 021b0084: 8-frame delay (0215a684 / 690), menu = 02190BE4(msgbg, 0), poll 02190C78 (0 yes -> var 0,
#    2 open, other -> var 1), close 021909BC.
#  * 173 / 174 / 178 menu begin (0215A77C; 174 / 178 with the script's message data, 178 right-aligned), 175 add item (0215A7F0),
#    176 / 177 open + wait (GMEVENT 021b015c: 8-frame delay, 0215A864, poll 0215A954 / 0215A990).
#  * 52 message (talk window, streamed, waits for the end of printing: 02191084 + poll 021910C0), 53 message put at once (02191184,
#    no wait), 54 close the talk window, 55 window icon, 75 wait for A / B (020362DC & 3; SE 0x547), 63 close every window.
#  * 60 / 61 / 72 / 73 / 135 balloon messages over a field object (021b0bac; poll 02191388), 62 close the balloon.
#  * 56 / 74 system window (02191558 / 021916A8, poll 021916E0), 57 close; 58 / 59 sized info window; 64 / 65 / 66 info board
#    (message file 158 = 0x9E); 67 / 579 / 68 and 69 / 70 other window kinds; 51 a msgbg flag.
#  Talk window (021b03e4): init-proc bit 1, style 1 or 0x13 by kind (0 = by SCRCMD WORK +0x104, default 7), object SCRCMD WORK
#  +0xFC; text = 021b162c (02045CAC(msgdata, msg, STRBUF2) + 0201F250(WORDSET, STRBUF1, STRBUF2) -> STRBUF1).
#  Balloon work B = SCRCMD WORK +0xD4 (0x28 B, address from 0215A63C): +0 u16 object id, +2 u16 tail, +4..+0xC window position,
#  +0x10 object position, +0x1C offset (ROM table 0x021D939C, 2 x 4 records of 12 B).
# Not specced: 021b0386 (inside 021b0344), 021b0c26 (inside 021b0bac): stray `bl` patterns in data point at them.
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
area scrcmd_talk
set field

# GMEVENT fn of cmd 71 (yes / no; event, seq, work {ctx, result var ptr, menu}): seq 0: 0215A684(ctx, 8) (8-frame delay),
# seq 1; seq 1: once 0215A690(ctx) == 1: work +8 = 02190BE4(02188C54(field map), 0) (open the yes / no menu), seq 2;
# seq 2: r = 02190C78(menu): 0 -> *var = 0 (yes), 2 -> keep polling, other -> *var = 1 (no); then seq 3;
# seq 3: 021909BC(menu) (close), 1 (done).  Jump table on seq (0..3).
fn 021b0084 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 4 u32 addrof arg2 0x20
  stub 02190be4 force ret=u32 args=v,v
  stub 0215a684 force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a690 force ret=pick:0/1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188c54 force ret=u32 args=v
  stub 02190c78 force ret=pick:0/1/2/3 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 021909bc force ret=none args=v

# 021b010c(GAMESYS, ctx, var ptr): yes / no event = 020122C0(GAMESYS, 0, 0x021B0085, 0xC) with work {ctx, var ptr}; returns it.
fn 021b010c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v

# cmd 71 (YES_NO): result var = next operand (02159AE8); child event 021b010c(GAMESYS, ctx, var) (ScriptHost yes_no); yield.
fn 021b0130 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021b010c ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  # closure: 021b010c
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v

# GMEVENT fn of cmds 176 / 177 (choice menu; work {ctx, X allowed}): seq 0: 0215A684(ctx, 8), seq 1; seq 1: once
# 0215A690(ctx) == 1: 0215A864(ctx) (open the menu), seq 2; seq 2: 0215A990(ctx) (with X) or 0215A954(ctx) == 1 -> 1 (done).
fn 021b015c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 4 u32 alt const 0 || any
  stub 0215a684 force ret=none args=v,v
  stub 0215a864 force ret=none args=v
  stub 0215a990 force ret=pick:0/1 args=v
  stub 0215a690 force ret=pick:0/1 args=v
  stub 0215a954 force ret=pick:0/1 args=v

# 021b01b8(GAMESYS, ctx, x_allowed): menu event = 020122C0(GAMESYS, 0, 0x021B015D, 8) with work {ctx, x_allowed}.
fn 021b01b8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v

# cmd 173 (menu begin, no message data): x, y = script bytes, cursor = value, s0 = script byte, var = u16 operand;
# 0215A77C(ctx, x, y, cursor, s0, 0, var ptr (02158FE0(SW, GAMEDATA, var)), WORDSET, 0); yield.
fn 021b01dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158fe0 force ret=u32 args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a77c force ret=none args=v,v,v,v,v,v,v,v,v

# cmd 174: as cmd 173 with message data = the script's (0215A60C(ctx)): 0215A77C(ctx, x, y, cursor, s0, 0, var ptr,
# WORDSET, msgdata); yield.
fn 021b024c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158fe0 force ret=u32 args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0215a60c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a77c force ret=none args=v,v,v,v,v,v,v,v,v

# cmd 178: as cmd 174 with s1 = 1 (right-aligned menu): 0215A77C(ctx, x, y, cursor, s0, 1, var ptr, WORDSET, msgdata).
fn 021b02c8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158fe0 force ret=u32 args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0215a60c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a77c force ret=none args=v,v,v,v,v,v,v,v,v

# cmd 175 (menu item): msg, help = u16 operands, param = value; 0215A7F0(ctx, msg, help, param, STRBUF1, STRBUF2); 0.
fn 021b0344 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=u16 args=v seq
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a7f0 force ret=none args=v,v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 176 (menu open + wait): child event 021b01b8(GAMESYS, ctx, 0); yield.
fn 021b0394 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b01b8 ret=u32 args=v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  # closure: 021b01b8
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v

# cmd 177: same body as cmd 176 (x_allowed = 0 as well).
fn 021b03bc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b01b8 ret=u32 args=v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  # closure: 021b01b8
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v

# 021b03e4(ctx, kind): open the talk window (init-proc bit 1 = open): already open -> 02191164(window) (clear it) and
# return; else style = kind 1 -> 1, kind 2 -> 0x13, kind 0 -> by the u8 at scrcmd work +0x104 (1 / 3 / 5 -> style 1 kind 1,
# else style 0x13 kind 2), other kinds -> style 0x13 kind 2; window = 02190FA0(*field block (msgbg), msgdata, style);
# work +0x104 = kind (0215A660), work +0xFC = window (0215A614); bit 1 on.
fn 021b03e4 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 alt range 0 3 || any
  stub 02190fa0 force ret=u32 args=v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 02191164 force ret=none args=v
  stub 02159144 force ret=none args=v
  stub 0215a60c force ret=u32 args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg0:0x300:0x10 args=v
  stub 0215a61c force ret=u32 args=v

# 021b0480(ctx): close the talk window: bit 1 set -> 02191008(window) (delete); work +0x100 set -> 0202F038(it), +0x100 = 0;
# work +0xFC = 0; bit 1 off.
fn 021b0480 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0215a624 force ret=none args=v,v
  stub 02191008 force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a630 force ret=nullable:2:u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0202f038 force ret=none args=v
  stub 0215a61c force ret=u32 args=v

# wait callback of cmd 52: done once 021910C0(window) == 1 (printing finished).
fn 021b04c0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021910c0 force ret=pick:0/1/2 args=v
  stub 0215a61c force ret=u32 args=v

# cmd 52 (message, streamed): msg = value, kind = u16 operand & 0xFF; 021b03e4(ctx, kind); text = 021b162c(ctx, SW, msg);
# 02191084(window, 0, 0, text) (start printing); wait on 0x021B04C1; yield.
fn 021b04d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b03e4 ret=none args=v,v
  stub 02191084 force ret=none args=v,v,v,v
  stub 021b162c ret=u32 args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a61c force ret=u32 args=v
  # closure: 021b03e4 / 021b162c
  stub 02190fa0 force ret=u32 args=v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 02191164 force ret=none args=v
  stub 02159144 force ret=none args=v
  stub 0215a60c force ret=u32 args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 0201f250 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 53 (message, all at once): msg = value, kind; 021b03e4(ctx, kind); 02045CAC(msgdata, msg, STRBUF2);
# 0201F250(WORDSET, STRBUF1, STRBUF2); 02191164(window); 02191184(window, 0, 0, STRBUF1); 0.
fn 021b0530 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b03e4 ret=none args=v,v
  stub 02191164 force ret=none args=v
  stub 02191184 force ret=none args=v,v,v,v
  stub 0215a60c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  stub 0215a61c force ret=u32 args=v
  # closure: 021b03e4
  stub 02190fa0 force ret=u32 args=v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 55: v = value; window not open (bit 1 clear) -> 0; work +0x100 set -> 0202F038(it), +0x100 = 0; v == 0 ->
# work +0x100 = 0202EE30(020056BC(), 021911B4(window), 0xF, 0x10, heap (02188C84(field map))) (an icon on the window); 0.
fn 021b05ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020056bc force ret=u32 args=v
  stub 02188c84 force ret=u32 args=v
  stub 0215a624 force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2 args=v,v seq
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a630 force ret=nullable:2:u32 args=v
  stub 0202ee30 force ret=u32 args=v,v,v,v,v
  stub 021911b4 force ret=u32 args=v
  stub 0202f038 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a61c force ret=u32 args=v

# cmd 54 (close the talk window): 021b0480(ctx); 0.
fn 021b0624 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b0480 ret=none args=v
  # closure: 021b0480
  stub 0215a624 force ret=none args=v,v
  stub 02191008 force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a630 force ret=nullable:2:u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0202f038 force ret=none args=v
  stub 0215a61c force ret=u32 args=v

# init proc (slot 1; task): 021b0480(task ctx); 1.
fn 021b0630 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 021b0480 ret=none args=v
  # closure: 021b0480
  stub 0215a624 force ret=none args=v,v
  stub 02191008 force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a630 force ret=nullable:2:u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0202f038 force ret=none args=v
  stub 0215a61c force ret=u32 args=v

# cmd 64 (info board): a, b = values; msgdata = 02045B38(0, 3, 0x9E, heap) (message file 158); s = 020457B0(0x80, heap);
# old board (02188D48(field map)) -> 02190728(it); board = 02190708(msgbg, msgdata, a - 9, b, 9, 2); 02188D4C(field map,
# board); 0201EFA4(WORDSET, 2, 0200C274(02012F08(GAMEDATA)), 7, 1, 1); 02045CAC(msgdata, 1, STRBUF2); 0201F250(WORDSET,
# s, STRBUF2); 02190758(board, 0, 0, s); free s and msgdata; 0.
fn 021b063c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c84 force ret=u32 args=v
  stub 02045c04 force ret=none args=v
  stub 02012f08 force ret=u32 args=v
  stub 02190708 force ret=u32 args=v,v,v,v,v,v
  stub 02045808 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020127a4 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 02190728 force ret=none args=v
  stub 02045cac force ret=none args=v,v,v
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 02188d48 force ret=nullable:2:u32 args=v
  stub 02188d4c force ret=none args=v,v
  stub 0201f250 force ret=none args=v,v,v
  stub 02188c54 force ret=u32 args=v
  stub 02190758 force ret=none args=v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0200c274 force ret=pick:0/1/2/0xFFFF/0x10000 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 66: refresh the info board text: as cmd 64 without creating it: 02190788(board), 02190758(board, 0, 0, s); 0.
fn 021b0730 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c84 force ret=u32 args=v
  stub 02045c04 force ret=none args=v
  stub 02012f08 force ret=u32 args=v
  stub 02190788 force ret=none args=v
  stub 02045808 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 020127a4 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 02045cac force ret=none args=v,v,v
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 02188d48 force ret=nullable:2:u32 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 02190758 force ret=none args=v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0200c274 force ret=pick:0/1/2/0xFFFF/0x10000 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 65: board = 02188D48(field map); board -> 02190728(board), 02188D4C(field map, 0); 0.
fn 021b07e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188d48 force ret=nullable:2:u32 args=v
  stub 02190728 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02188d4c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# 021b0808(pos, out, camera): balloon placement: p = *pos (12 B); camera vectors 02048AD0 / 02048B30 / 02048AE8, VEC math
# 0207CA00 / 0207CCA4 / 0207CAE8 (constant 0x021D9390) move p; 0206DF3C(&p, &y, &x) projects it to the screen; row = y >> 3
# (< 10: 0, 10..15: 1, 16..21: 2, else 3), col = (x >> 3) / 12; col < 2 -> *out = the 12-byte record of the ROM table
# 0x021D939C [col * 4 + row], else {0, 0, 0}.
fn 021b0808 ret=none inputs=200
  nowork
  arg buf 0x10
  arg buf 0x10
  arg u32 any
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0

# 021b090c(v, camera): camera matrices 02048AE8 / 02048B0C / 02048B30 -> 0207BCA0 (look-at), 02082B60 (copy), 0207B0A4
# (invert), 0207B51C(v, m, v) (transform v in place, 12 B).
fn 021b090c ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 any
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048ae8 force ret=none args=v,out:12
  stub 02048b0c force ret=none args=v,out:12
  stub 02048b30 force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# 021b0960(pos, out, camera, a3 (unused), u8 kind (stack)): kinds 0..2 -> 021b0808(pos, out, camera) (projected placement);
# kind 3 -> *out = ROM table 0x021D939C record (row 1, col 0), 4 -> (1, 1), 5 -> (2, 0), 6 -> (2, 1), others (0, 0)
# (record = col * 0x30 + row * 0xC); then 021b090c(out, camera) (jump table on kind).
fn 021b0960 ret=none inputs=200
  nowork
  arg buf 0x10
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 alt range 0 7 || any
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  # closure: 021b0808 / 021b090c
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# 021b09d8(pos, out, camera, a3, u8 kind (stack)): 021b0960 with the same arguments (a veneer).
fn 021b09d8 ret=none inputs=200
  nowork
  arg buf 0x10
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 alt range 0 7 || any
  stub 021b0960 ret=none args=v,v,v,v,v
  # closure: 021b0960
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# 021b09ec(kind, &a, &b): balloon tail of a kind: a = b = 0, then kind 0: a = 1, b = 2; 1: (none); 2: a = 1; 3: b = 7;
# 4: a = 1, b = 5; 5: b = 8; 6: a = 1, b = 6 (jump table; other kinds: 0, 0).
fn 021b09ec ret=none inputs=200
  nowork
  arg u32 alt range 0 7 || any
  arg buf 0x10
  arg buf 0x10

# 021b0a40(mmdl, pos): 0216D88C(mmdl, &p) (its position); pos +8 < p +8 (signed) -> 1, else 2 (balloon above / below).
fn 021b0a40 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  stub 0216d88c force ret=none args=v,out:12

# wait callback of cmd 62: 021912F8(window) == 1 -> work +0xFC = 0, init-proc bit 2 off, done.
fn 021b0a60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021912f8 force ret=pick:0/1/2 args=v
  stub 02159168 force ret=none args=v
  stub 0215a61c force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v

# 021b0a88(vm, ctx): balloon open (bit 2) -> 021912D8(window) (start closing) and wait on 0x021B0A61.
fn 021b0a88 ret=none inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159190 force ret=pick:0/1 args=v
  stub 020113d0 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 021912d8 force ret=none args=v

# 021b0ab0(ctx): balloon open -> 0219131C(window) (delete at once), work +0xFC = 0, bit 2 off.
fn 021b0ab0 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0219131c force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# 021b0adc(ctx, msgbg, pos, tail, text, a5, a6) (stack args text, a5, a6): window = 0219129C(msgbg, pos, tail, text, a5, a6)
# (a balloon window); work +0xFC = window; init-proc bit 2 on.
fn 021b0adc ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v

# 021b0b08(ctx, force): B = the balloon work (scrcmd work +0xD4, 0215A63C); mmdl = 0216DE94(MMDL system (0215A5EC), u16 B +0)
# none -> return; 0216D5F8(mmdl); 0216D88C(mmdl, &p); force == 1 or p != B +0x10..+0x18 -> B +0x10 = p and
# 021b0960(B +0x10, B +0x1C, 0218EC2C(cam = 02188C58(field map)), cam, u8 work +0x105) (re-place); always B +4..+0xC =
# B +0x10 + B +0x1C (3 words).
fn 021b0b08 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 alt const 1 || any
  fix arg0 0x1E4 u32 alt const 0 || any rep=3,4
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a5ec force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a63c force ret=ptr:arg0:0x1D4:1 args=v
  # closure: 021b0960
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# 021b0bac(ctx, obj, bank, msg, u16 kind, a5) (stack args kind, a5): print a message in a balloon over object obj:
# player = 0216DE94(mmdls, 0xFF), target = 0216DE94(mmdls, obj); no target -> 0.  bank == 0x400 -> 02045CAC(script
# msgdata, msg, STRBUF2), else a temporary 02045B38(0, 3, bank, heap); 0201F250(WORDSET, STRBUF1, STRBUF2).  Balloon open
# (bit 2) -> 02191348(window, STRBUF1) (next page); else B (work +0xD4) = 0, B +0 = obj, kind 0 -> kind =
# 021b0a40(player, target's position) (0215A654 is read but unused); 021b09ec(kind, &tail, &b); B +2 = tail; work +0x104
# = kind, +0x105 = the stack kind; 021b0b08(ctx, 1); 021b0adc(ctx, msgbg, B +4, tail, STRBUF1, a5, b).  1.
fn 021b0bac ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 alt const 0x400 || any
  arg u32 any
  arg u32 alt const 0 || range 1 7 || any
  arg u32 any
  stub 02045c04 force ret=none args=v
  stub 021b0b08 ret=none args=v,v
  stub 0215a60c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a678 force ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0215a63c force ret=ptr:arg0:0x1D4:1 args=v
  stub 021b0a40 ret=pick:1/2 args=v,v
  stub 02191348 force ret=none args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg0:0x300:0x10 args=v
  stub 021b0adc ret=none args=v,v,v,v,v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 021b09ec ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  # closure: 021b0b08 / 021b0a40 / 021b0adc / 021b09ec
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# wait callback of the balloon commands: 021b0b08(ctx, 0) (follow the object); done once 02191388(window) == 1.
fn 021b0cec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b0b08 ret=none args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 02191388 force ret=pick:0/1/2 args=v
  # closure: 021b0b08
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a5ec force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a63c force ret=ptr:arg1:0x1D4:1 args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# cmd 60 (balloon message): bank, msg, obj, kind, a5 = values; 021b0bac(ctx, obj, bank, msg, kind, a5); 1 -> wait on 0x021B0CED, yield; else 0.
fn 021b0d14 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/3/6/0x400/0xFF/0x1234 args=v,v seq
  stub 021b0bac ret=pick:0/1 args=v,v,v,v,v,v
  stub 020113d0 force ret=none args=v,v
  # closure: 021b0bac
  stub 02045c04 force ret=none args=v
  stub 021b0b08 ret=none args=v,v
  stub 0215a60c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a678 force ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0215a63c force ret=ptr:arg1:0x1D4:1 args=v
  stub 021b0a40 ret=pick:1/2 args=v,v
  stub 02191348 force ret=none args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021b0adc ret=none args=v,v,v,v,v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 021b09ec ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# cmd 61 (balloon message): bank, msg, kind, a5 = values; talk target = 02158F90(SW) (none -> yield 1);
# 021b0bac(ctx, 0216D588(target) & 0xFF, bank, msg, kind, a5); 1 -> wait on 0x021B0CED, yield; else 0.
fn 021b0d74 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216d588 force ret=u16 args=v
  stub 021b0bac ret=pick:0/1 args=v,v,v,v,v,v
  stub 02158f90 force ret=nullable:4:u32 args=v
  stub 02159b10 force ret=pick:0/1/3/6/0x400/0xFF/0x1234 args=v,v seq
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  # closure: 021b0bac
  stub 02045c04 force ret=none args=v
  stub 021b0b08 ret=none args=v,v
  stub 0215a60c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a678 force ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0215a63c force ret=ptr:arg1:0x1D4:1 args=v
  stub 021b0a40 ret=pick:1/2 args=v,v
  stub 02191348 force ret=none args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021b0adc ret=none args=v,v,v,v,v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 021b09ec ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# cmd 72 (balloon message): bank, msg1, msg2, obj, kind, a5 = values; msg = 02008550(020127A4(GAMEDATA) + 0x20) ? msg2 :
# msg1 (a player-gender / version pick, UNVERIFIED); 021b0bac(ctx, obj, bank, msg, kind, a5); 1 -> wait on 0x021B0CED, yield; else 0.
fn 021b0df0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020127a4 force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 021b0bac ret=pick:0/1 args=v,v,v,v,v,v
  stub 02159b10 force ret=pick:0/1/3/6/0x400/0xFF/0x1234 args=v,v seq
  stub 02008550 force ret=pick:0/1 args=v
  stub 020113d0 force ret=none args=v,v
  # closure: 021b0bac
  stub 02045c04 force ret=none args=v
  stub 021b0b08 ret=none args=v,v
  stub 0215a60c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a678 force ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0215a63c force ret=ptr:arg1:0x1D4:1 args=v
  stub 021b0a40 ret=pick:1/2 args=v,v
  stub 02191348 force ret=none args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021b0adc ret=none args=v,v,v,v,v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 021b09ec ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# cmd 73 (balloon message): bank, (skipped), msg, obj, kind, a5 = values; 021b0bac(ctx, obj, bank, msg, kind, a5); 1 -> wait on 0x021B0CED, yield; else 0.
fn 021b0e74 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/3/6/0x400/0xFF/0x1234 args=v,v seq
  stub 021b0bac ret=pick:0/1 args=v,v,v,v,v,v
  stub 020113d0 force ret=none args=v,v
  # closure: 021b0bac
  stub 02045c04 force ret=none args=v
  stub 021b0b08 ret=none args=v,v
  stub 0215a60c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 0215a678 force ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0215a63c force ret=ptr:arg1:0x1D4:1 args=v
  stub 021b0a40 ret=pick:1/2 args=v,v
  stub 02191348 force ret=none args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a654 force ret=pick:0/1/2/3/4/5/6/7/8 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021b0adc ret=none args=v,v,v,v,v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 021b09ec ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# cmd 62 (close the balloon): 021b0a88(vm, ctx); yield.
fn 021b0edc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b0a88 ret=none args=v,v
  # closure: 021b0a88
  stub 02159190 force ret=pick:0/1 args=v
  stub 020113d0 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v
  stub 021912d8 force ret=none args=v

# init proc (slot 2; task): 021b0ab0(task ctx); 1.
fn 021b0ee8 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 021b0ab0 ret=none args=v
  # closure: 021b0ab0
  stub 0219131c force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# cmd 135 (balloon with a trainer text): a, b, obj = values; balloon open (bit 2) -> yield 1; else 0202A310(a, b, STRBUF1,
# heap) (the text); 0216DE94(mmdls, obj) none -> obj = 0xFF; B = 0, B +0 = obj; kind = 021b0a40(021A3074(player
# 02188CB4(field map)), target position); tail = kind != 1; B +2 = tail; 021b0b08(ctx, 1); 021b0adc(ctx, msgbg, B +4,
# tail, STRBUF1, 0, 0); work +0x104 = kind; wait on 0x021B0CED; yield.
fn 021b0ef4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b0b08 ret=none args=v,v
  stub 0216d88c force ret=none args=v,outm:12:3/0/0/0
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02159190 force ret=pick:0/1 args=v
  stub 0202a310 force ret=none args=v,v,v,v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a63c force ret=ptr:arg1:0x1D4:1 args=v
  stub 021b0a40 ret=pick:1/2 args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 020113d0 force ret=none args=v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021b0adc ret=none args=v,v,v,v,v,v,v
  stub 0215a660 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  # closure: 021b0b08 / 021b0a40 / 021b0adc
  stub 021b0960 ret=none args=v,v,v,p,v
  stub 02188c58 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec2c force ret=u32 args=v
  stub 0215a66c force ret=pick:0/1/3/6/7 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0219129c force ret=u32 args=v,v,v,v,v,v
  stub 0215a614 force ret=none args=v,v
  stub 02159144 force ret=none args=v
  stub 021b0808 ret=none args=v,v,v
  stub 021b090c ret=none args=v,v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 0207cca4 force ret=none args=in:12,out:12
  stub 02048ae8 force ret=none args=v,out:12
  stub 0207cae8 force ret=none args=in:12,in:12,out:12
  stub 02048ad0 force ret=none args=v
  stub 02048b30 force ret=none args=v,out:12
  stub 0206df3c force ret=none args=in:12,outm:4:0xFF/0/0/0,outm:4:0xFF/1/0/0
  stub 0207bca0 force ret=none args=in:12,in:12,in:12,out:0x30
  stub 02082b60 force ret=none args=in:0x30,out:0x30
  stub 0207b0a4 force ret=none args=in:0x30,out:0x30
  stub 02048b0c force ret=none args=v,out:12
  stub 0207b51c force ret=none args=in:12,in:0x30,out:12

# wait callback of cmds 56 / 74: done once 021916E0(window) == 1.
fn 021b0ff0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021916e0 force ret=pick:0/1/2 args=v
  stub 0215a61c force ret=u32 args=v

# 021b1008(vm, ctx, kind) (body of cmds 56 / 74): msg = value, style = script byte == 1 ? 1 : 0x13; text = 021b162c(ctx,
# SW, msg); system window open (bit 3) -> 021915EC(window) (clear) else window = 02191558(msgbg, 0, kind, 1, style, 0x1E, 4),
# work +0xFC = window; 021916A8(window, 8, 0, text) (print); bit 3 on; wait on 0x021B0FF1; yield.
fn 021b1008 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  arg u32 alt range 0 1 || any
  stub 02191558 force ret=u32 args=v,v,v,v,v,v,v
  stub 02159144 force ret=none args=v
  stub 021916a8 force ret=none args=v,v,v,v
  stub 021b162c ret=u32 args=v,v,v
  stub 021915ec force ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02159190 force ret=pick:0/1 args=v
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215a61c force ret=u32 args=v
  # closure: 021b162c
  stub 02045cac force ret=none args=v,v,v
  stub 0215a60c force ret=u32 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v

# 021b10a4(ctx): system window open (bit 3) -> 021915B4(window) (delete), work +0xFC = 0, bit 3 off.
fn 021b10a4 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 021915b4 force ret=none args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# cmd 56 (system message window): `ldr r3, =0x021B1009; movs r2, 0; bx r3` = 021b1008(vm, ctx, 0).
fn 021b10d0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b1008 ret=none args=v
  # closure: 021b1008
  stub 02191558 force ret=u32 args=v,v,v,v,v,v,v
  stub 02159144 force ret=none args=v
  stub 021916a8 force ret=none args=v,v,v,v
  stub 021b162c ret=u32 args=v,v,v
  stub 021915ec force ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02159190 force ret=pick:0/1 args=v
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 0215a60c force ret=u32 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 74: 021b1008(vm, ctx, 1) (the same thunk shape).
fn 021b10dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b1008 ret=none args=v
  # closure: 021b1008
  stub 02191558 force ret=u32 args=v,v,v,v,v,v,v
  stub 02159144 force ret=none args=v
  stub 021916a8 force ret=none args=v,v,v,v
  stub 021b162c ret=u32 args=v,v,v
  stub 021915ec force ret=none args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02159190 force ret=pick:0/1 args=v
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215a61c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 0215a60c force ret=u32 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 57 (close the system window): 021b10a4(ctx); 0.
fn 021b10e8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b10a4 ret=none args=v
  # closure: 021b10a4
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 021915b4 force ret=none args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# init proc (slot 3; task): 021b10a4(task ctx); 1.
fn 021b10f4 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 021b10a4 ret=none args=v
  # closure: 021b10a4
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 021915b4 force ret=none args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# cmd 58 (sized info window): msg = u16 operand, x, y = values, a = u16 operand; s = 020457B0(0x40, heap); 02045CAC(msgdata,
# msg, STRBUF2); 0201F250(WORDSET, s, STRBUF2); 02191830(msgbg, s, &h, &w) (text size, w preset 2); x + h > 0x1F -> x =
# 0x1F - h; y + w > 0x18 -> y = 4, w = 4; 021918DC(msgbg, s, a, x, y, h, w) (stack y, h, w); free s; init-proc bit 5 on; 0.
fn 021b1100 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159144 force ret=none args=v
  stub 02045808 force ret=none args=v
  stub 0215a5cc force ret=u16 args=v
  stub 0215a60c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 0201f250 force ret=none args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02191830 force ret=none args=v,v,outm:1:0x1F,outm:1:0x1F
  stub 021918dc force ret=none args=v,v,v,v,v,v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 59: 0219196C(msgbg, u16 operand); 021919AC(msgbg) == 0 -> init-proc bit 5 off; 0.
fn 021b11d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159168 force ret=none args=v
  stub 021919ac force ret=pick:0/1 args=v
  stub 0219196c force ret=none args=v,v
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# 021b120c(ctx): bit 5 set -> 02191978(msgbg), bit 5 off.
fn 021b120c ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 02158f58 force ret=ptr:arg0:0x300:0x10 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02191978 force ret=none args=v

# init proc (slot 5; task): 021b120c(task ctx); 1.
fn 021b1234 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 021b120c ret=none args=v
  # closure: 021b120c
  stub 02159168 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 02158f58 force ret=ptr:arg0:0x40:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02191978 force ret=none args=v

# wait callback of cmds 67 / 579: done once 02191BE8(window, STRBUF1) == 1.
fn 021b1240 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f78 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a61c force ret=u32 args=v
  stub 02191be8 force ret=pick:0/1/2 args=v,v

# 021b1268(ctx): bit 4 set -> 02191B14(window); x = SW +0x38 (02158F84), SW +0x38 = 0; x -> 0201D83C(x); work +0xFC = 0;
# bit 4 off.
fn 021b1268 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02158f80 force ret=none args=v,v
  stub 02158f84 force ret=nullable:2:u32 args=v
  stub 02159168 force ret=none args=v
  stub 0201d83c force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 02191b14 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# cmd 67 (another window kind): msg, a = u16 operands; work +0xFC = 02191A04(msgbg, a); 02045CAC(msgdata, msg, STRBUF2);
# 0201F250(WORDSET, STRBUF1, STRBUF2); wait on 0x021B1241; init-proc bit 4 on; yield.
fn 021b12b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02191a04 force ret=u32 args=v,v
  stub 02159144 force ret=none args=v
  stub 0215a60c force ret=u32 args=v
  stub 02045cac force ret=none args=v,v,v
  stub 02011330 force ret=u16 args=v seq
  stub 0201f250 force ret=none args=v,v,v
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 02158f78 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 579: msg, a = u16 operands; x = 0201D7EC(0x17, 4, 0, 0, heap (02188C84(field map))); SW +0x38 = x; work +0xFC =
# 02191A10(msgbg, a, x); text as cmd 67; wait on 0x021B1241; bit 4 on; yield.
fn 021b1334 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f80 force ret=none args=v,v
  stub 02188c84 force ret=u32 args=v
  stub 02159144 force ret=none args=v
  stub 0215a60c force ret=u32 args=v
  stub 0201d7ec force ret=u32 args=v,v,v,v,v
  stub 02045cac force ret=none args=v,v,v
  stub 02011330 force ret=u16 args=v seq
  stub 02191a10 force ret=u32 args=v,v,v
  stub 0201f250 force ret=none args=v,v,v
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 02158f78 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 68: 021b1268(ctx); 0.
fn 021b13e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b1268 ret=none args=v
  # closure: 021b1268
  stub 02158f80 force ret=none args=v,v
  stub 02158f84 force ret=nullable:2:u32 args=v
  stub 02159168 force ret=none args=v
  stub 0201d83c force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 02191b14 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# init proc (slot 4; task): 021b1268(task ctx); 1.
fn 021b13ec ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 021b1268 ret=none args=v
  # closure: 021b1268
  stub 02158f80 force ret=none args=v,v
  stub 02158f84 force ret=nullable:2:u32 args=v
  stub 02159168 force ret=none args=v
  stub 0201d83c force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 02191b14 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# 021b13f8(ctx): bit 6 set -> 02192628(window), work +0xFC = 0, bit 6 off.
fn 021b13f8 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02159168 force ret=none args=v
  stub 02192628 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# wait callback of cmd 69: done once 021926D8(window) == 1.
fn 021b1424 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021926d8 force ret=pick:0/1/2 args=v
  stub 0215a61c force ret=u32 args=v

# cmd 69: msg = u16, x, y = script bytes, a = u16; text = 021b162c(ctx, SW, msg); w = u16 0219270C(msgbg, text, 4),
# h = u16 0219272C(msgbg, text, 4); window = 02192570(msgbg, a, x, y, w, h); work +0xFC = window; 0219269C(window, 4, 4,
# text); wait on 0x021B1425; bit 6 on; yield.
fn 021b143c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159144 force ret=none args=v
  stub 0219270c force ret=u32 args=v,v,v
  stub 021b162c ret=u32 args=v,v,v
  stub 0219272c force ret=u32 args=v,v,v
  stub 02011330 force ret=u16 args=v seq
  stub 02192570 force ret=u32 args=v,v,v,v,v,v
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0219269c force ret=none args=v,v,v,v
  # closure: 021b162c
  stub 02045cac force ret=none args=v,v,v
  stub 0215a60c force ret=u32 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v

# cmd 70: 021b13f8(ctx); 0.
fn 021b14dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b13f8 ret=none args=v
  # closure: 021b13f8
  stub 02159168 force ret=none args=v
  stub 02192628 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# init proc (slot 6; task): 021b13f8(task ctx); 1.
fn 021b14e8 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 021b13f8 ret=none args=v
  # closure: 021b13f8
  stub 02159168 force ret=none args=v
  stub 02192628 force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0215a61c force ret=u32 args=v

# cmd 63 (close every message window): bit 1 -> 021b0480, bit 3 -> 021b10a4, bit 4 -> 021b1268, bit 6 -> 021b13f8 (ctx),
# bit 2 -> 021b0a88(vm, ctx) (balloon close + wait); yield.
fn 021b14f4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b0480 ret=none args=v
  stub 021b10a4 ret=none args=v
  stub 021b1268 ret=none args=v
  stub 021b0a88 ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v
  stub 021b13f8 ret=none args=v
  # closure: 021b0480 / 021b10a4 / 021b1268 / 021b0a88 / 021b13f8
  stub 0215a624 force ret=none args=v,v
  stub 02191008 force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 0215a630 force ret=nullable:2:u32 args=v
  stub 0215a614 force ret=none args=v,v
  stub 0202f038 force ret=none args=v
  stub 0215a61c force ret=u32 args=v
  stub 021915b4 force ret=none args=v
  stub 02158f80 force ret=none args=v,v
  stub 02158f84 force ret=nullable:2:u32 args=v
  stub 0201d83c force ret=none args=v
  stub 02191b14 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020113d0 force ret=none args=v,v
  stub 021912d8 force ret=none args=v
  stub 02192628 force ret=none args=v

# wait callback of cmd 75 (wait for A / B): pad = 020362DC(); pad & 3 -> 020061E4(0x547) (SE), done; else talk window
# (bit 1) -> 02191134(window), system window (bit 3) -> 02191794, balloon (bit 2) -> 021914CC (the 'next' cursor), 0; no
# window -> done.
fn 021b1550 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020061e4 force ret=none args=v
  stub 021914cc force ret=none args=v
  stub 02159190 force ret=pick:0/1 args=v
  stub 02191134 force ret=none args=v
  stub 02191794 force ret=none args=v
  stub 020362dc force ret=pick:0/1/2/3/0x400 args=
  stub 0215a61c force ret=u32 args=v

# cmd 75 (wait button): wait on 0x021B1551; yield.
fn 021b15b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 51: u16 operand != 0 -> 021904DC(msgbg, 1), init-proc bit 9 on; else 021904DC(msgbg, 0), bit 9 off; 0.
fn 021b15c8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159144 force ret=none args=v
  stub 02159168 force ret=none args=v
  stub 02011330 force ret=pick:0/1/2 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021904dc force ret=none args=v,v

# init proc (slot 9; task): bit 9 set -> 021904DC(*field block of the task's SW, 0), bit 9 off; 1.
fn 021b1604 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  stub 02158f58 force ret=ptr:arg0:0x40:1 args=v
  stub 02159168 force ret=none args=v
  stub 021904dc force ret=none args=v,v
  stub 02159190 force ret=pick:0/1 args=v

# 021b162c(ctx, SW, msg): 02045CAC(msgdata (0215A60C), msg, STRBUF2); 0201F250(WORDSET, STRBUF1, STRBUF2); returns STRBUF1.
fn 021b162c ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 any
  stub 02045cac force ret=none args=v,v,v
  stub 0215a60c force ret=u32 args=v
  stub 0201f250 force ret=none args=v,v,v
  stub 02158f74 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
