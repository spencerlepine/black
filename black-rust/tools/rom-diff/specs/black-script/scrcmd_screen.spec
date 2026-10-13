# fndiff spec: overlay-21 screen-effect script commands (F8-spec2), Thumb code 0x021B2830..0x021B2FD8: the tail of F1-SPLIT row
# 21.44 (cmds 179 / 180 / 433 / 436) and row 21.45 scrcmd_screeneffects.c (0x021B28E0..0x021B2FAA) plus cmd 594 at 0x021B2FB0
# (between rows 21.45 and 21.46).  Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-script scrcmd_screen --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  179 021b2830  180 021b2a90  292 021b2e08  293 021b2ab0  294 021b2ce8  295 021b2d34  296 021b2d98  297 021b2dbc
#  298 021b2e60  299 021b2c08  300 021b2e3c  301 021b2ca0  302 021b2c5c  303 021b2adc  304 021b2b24  305 021b2b5c
#  306 021b2b94  429 021b2910  430 021b2940  431 021b2970  432 021b29a0  433 021b2aa0  434 021b28e0  436 021b2890
#  437 021b29d0  438 021b2a00  439 021b2a30  440 021b2a60  594 021b2fb0
# 36 functions specced (29 command ids).
#  * 179 = fade.c 0204E720(mode, start, end, wait) with four u16 operands; 180 waits until 0204E804() (fade running) != 1.
#    fade.c state (main RAM, *0x02146A7C): +0 mode, +4 start, +8 current, +0xC end, +0x10 / +0x14 wait counter, +0x18 step;
#    wait < 0: step |wait| + 1 per frame, else 1 every `wait` frames; brightness -16..16 (master brightness; UNVERIFIED sign).
#    mode bits: the field fade uses 3 (black, both screens) and 0xC (white, both screens).
#  * 429..432 / 434 / 437..440 field fades: task 021b2ebc / 021b2f18 (work 0x18 from heap 4: GAMESYS, field map, color, no_wait,
#    kind, seq): kind 0 / 2 fade in (16 -> 0), 1 / 3 fade out (0 -> 16), kinds 0 / 1 wait -1 (fast), 2 / 3 wait 4; color 0 black
#    (mode 3), 1 white (0xC); field map flag 02189A1C(fm, 1 / 0) while it runs; 433 waits until 02189A24(fm) != 1.
#  * 292..302 field 3D objects (0218BA98 by id, 0218C654 create at a grid position), 298 + its wait (u16 global 0x021EE720),
#    293 / 303..306 child events of overlay 21 (021C0xxx / 021C9xxx), 436 child event 021BE92C(GAMESYS, fm, 3, 0, 0, z, z),
#    594 021BC5A0.
# Not specced: 021b2c18 (inside 021b2c08).
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
area scrcmd_screen
set field

# cmd 179 (screen fade): mode, start, end, wait = u16 operands; 0204E720(mode, start, end, wait) (fade.c: brightness
# from start to end, step per `wait`; mode bits 1 / 2 / 4 / 8, see the header); 0.  ScriptHost::fade_start.
fn 021b2830 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=u16 args=v seq
  stub 0204e720 force ret=none args=v,v,v,v

# wait callback of cmd 180: done once 0204E804() != 1 (no fade running).  ScriptHost::fade_done.
fn 021b2860 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 0204e804 force ret=pick:0/1 args=

# wait callback of cmd 433: done once 02189A24(field map) != 1 (the field fade task finished).
fn 021b2874 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02189a24 force ret=pick:0/1/2 args=v

# cmd 436: child event 021BE92C(GAMESYS, field map, 3, 0, 0, z, z) with z = 02012984(GAMEDATA) & 0xFF (stack 0, z, z); 0.
fn 021b2890 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012984 force ret=u16 args=v
  stub 0201210c force ret=u32 args=v
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 434 (field fade in from black): 021b2ebc(GAMESYS, field map, kind 0, color 0, 0); 0.
fn 021b28e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 429 (field fade in from black): 021b2ebc(GAMESYS, field map, kind 0, color 0, 0); 0.
fn 021b2910 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 430 (field fade out from black): 021b2ebc(GAMESYS, field map, kind 1, color 0, 0); 0.
fn 021b2940 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 431 (field fade in from white): 021b2ebc(GAMESYS, field map, kind 0, color 1, 0); 0.
fn 021b2970 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 432 (field fade out from white): 021b2ebc(GAMESYS, field map, kind 1, color 1, 0); 0.
fn 021b29a0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 437 (field fade in from black): 021b2ebc(GAMESYS, field map, kind 2, color 0, 0); 0.
fn 021b29d0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 438 (field fade out from black): 021b2ebc(GAMESYS, field map, kind 3, color 0, 0); 0.
fn 021b2a00 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 439 (field fade in from white): 021b2ebc(GAMESYS, field map, kind 2, color 1, 0); 0.
fn 021b2a30 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 440 (field fade out from white): 021b2ebc(GAMESYS, field map, kind 3, color 1, 0); 0.
fn 021b2a60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b2ebc ret=none args=v,v,v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  # closure: 021b2ebc
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg1:0x300:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# cmd 180 (wait for the fade): wait on 0x021B2861; yield.
fn 021b2a90 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 433 (wait for the field fade): wait on 0x021B2875; yield.
fn 021b2aa0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 293: child event 021C0AA0(GAMESYS, GMEVENT (02158F50(SW))); yield.
fn 021b2ab0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c0aa0 force ret=u32 args=v,v
  stub 02158f50 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 303: v = value; 02188CB4(field map) (unused); child event 021C0820(GAMESYS, GMEVENT, v & 0xFF); yield.
fn 021b2adc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c0820 force ret=u32 args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f50 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 304: child event 021C978C(GMEVENT, GAMESYS, field map); yield.
fn 021b2b24 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c978c force ret=u32 args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02158f50 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 305: child event 021C9828(GMEVENT, GAMESYS, field map); yield.
fn 021b2b5c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c9828 force ret=u32 args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02158f50 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 306: child event 021C98F4(GMEVENT, GAMESYS, field map, value); yield.
fn 021b2b94 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02158f50 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021c98f4 force ret=u32 args=v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# 021b2bd8(ctx): 0218DC6C(02188C98(field map)) (the field's 3D-object system, UNVERIFIED).
fn 021b2bd8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# 021b2bec(sys, id, x, z): 0218C654(sys, id, &{x << 16, 0, z << 16}).
fn 021b2bec ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0218c654 force ret=u32 args=v,v,in:12

# cmd 299: id, x, z = values; o = 021b2bec(021b2bd8(ctx), id, x, z); o -> 0218C69C(sys, o, 0, 1), 0218C69C(sys, o, 0, 3); 0.
fn 021b2c08 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c69c force ret=none args=v,v,v,v
  stub 021b2bec ret=nullable:3:u32 args=v,v,v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bec / 021b2bd8
  stub 0218c654 force ret=nullable:3:u32 args=v,v,in:12
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 302: id, x, z = values; o = 021b2bec(..); o -> 0218C69C(sys, o, 0, 2); 0.
fn 021b2c5c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c69c force ret=none args=v,v,v,v
  stub 021b2bec ret=nullable:3:u32 args=v,v,v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bec / 021b2bd8
  stub 0218c654 force ret=nullable:3:u32 args=v,v,in:12
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 301: id, x, z, v = values; o = 021b2bec(..); o -> 0218C54C(o, v); 0.
fn 021b2ca0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 021b2bec ret=nullable:3:u32 args=v,v,v,v
  stub 0218c54c force ret=none args=v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bec / 021b2bd8
  stub 0218c654 force ret=nullable:3:u32 args=v,v,in:12
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 294: id, x, z, v = values; o = 021b2bec(..); o -> 0218C574(sys, o, v); 0.
fn 021b2ce8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c574 force ret=none args=v,v,v
  stub 021b2bec ret=nullable:3:u32 args=v,v,v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bec / 021b2bd8
  stub 0218c654 force ret=nullable:3:u32 args=v,v,in:12
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 295: var; id, x, z = values; sys = 0218DC6C(02188C98(field map)); var = 0218BACC(sys, 0218C7D8(sys, id, &{x << 16,
# 0, z << 16})).
fn 021b2d34 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0218bacc force ret=u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c7d8 force ret=u32 args=v,v,in:12
  stub 02188c98 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 296: o = 0218BA98(021b2bd8(ctx), value); o -> 0218C860(o); 0.
fn 021b2d98 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c860 force ret=none args=v
  stub 0218ba98 force ret=nullable:3:u32 args=v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bd8
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 297: id, a = values; o = 0218BA98(sys, id); o -> 0218C88C(o, a, 0), 0218C990(o, &se) != 0 -> 020061E4(u16 se); 0.
fn 021b2dbc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020061e4 force ret=none args=v
  stub 0218c88c force ret=none args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c990 force ret=pick:0/1 args=v,out:4
  stub 021b2bd8 ret=u32 args=v
  stub 0218ba98 force ret=nullable:3:u32 args=v,v
  # closure: 021b2bd8
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 292: id, a = values; o = 0218BA98(sys, id); o -> 0218C88C(o, a, 2); 0.
fn 021b2e08 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c88c force ret=none args=v,v,v
  stub 0218ba98 force ret=nullable:3:u32 args=v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bd8
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 300: o = 0218BA98(sys, value); o -> 0218C89C(o, 3); 0.
fn 021b2e3c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218c89c force ret=none args=v,v
  stub 0218ba98 force ret=nullable:3:u32 args=v,v
  stub 021b2bd8 ret=u32 args=v
  # closure: 021b2bd8
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 298: u16 global 0x021EE720 = value; wait on 0x021B2E81; yield.
fn 021b2e60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  global 021ee720 4
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020113d0 force ret=none args=v,v

# wait callback of cmd 298: o = 0218BA98(021b2bd8(ctx), u16 0x021EE720); none -> done; 0218C8D8(o) == 0 -> 0; else
# done once 0218C9A4(o) == 0.
fn 021b2e80 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  global 021ee720 4
  stub 021b2bd8 ret=u32 args=v
  stub 0218c9a4 force ret=pick:0/1 args=v
  stub 0218ba98 force ret=nullable:3:u32 args=v,v
  stub 0218c8d8 force ret=pick:0/1 args=v
  # closure: 021b2bd8
  stub 02188c98 force ret=u32 args=v
  stub 0218dc6c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# 021b2ebc(GAMESYS, field map, kind, color, no_wait (stack)): the field fade task: work W = Heap_AllocDebug(4, 0x18, 1,
# 0x021DE14C, 0x369); 02030DA8(02188D30(field map), 0x021B2F19, W, 0) (add a task); 02189A1C(field map, 1) (fading);
# W = {GAMESYS, field map, color, no_wait, kind, seq 0}.
fn 021b2ebc ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt range 0 3 || any
  arg u32 alt range 0 1 || any
  arg u32 alt range 0 1 || any
  arg buf 0x40
  stub 02188d30 force ret=u32 args=v
  stub 02030734 force ret=ptr:arg5:0:1 args=v,v,v,v,v
  stub 02189a1c force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v

# task fn of the field fade (task, W): seq 0: wait = kind <= 1 ? -1 : 4; kind 0 / 2 (fade in): from 16 to 0 after
# 02188DA0(02043658(0x10?)) / 02043534 / 02188DB0(field map) (UNVERIFIED: re-enable the field display); kind 1 / 3 (out):
# from 0 to 16; 0204E720(color ? 0xC : 3, from, to, wait); seq 1.  seq 1: no_wait or 0204E804() == 0 -> 02189A1C(field
# map, 0), 02030EAC(task) (remove), Heap_Free(W).
fn 021b2f18 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x20
  fix arg1 0x14 u32 alt range 0 2 || any
  fix arg1 0x10 u32 alt range 0 3 || any
  fix arg1 8 u32 alt range 0 1 || any
  fix arg1 0xC u32 alt range 0 1 || any
  stub 02188da0 force ret=none args=v
  stub 0204e720 force ret=none args=v,v,v,v
  stub 0204e804 force ret=pick:0/1 args=
  stub 02030eac force ret=none args=v
  stub 020307b0 force ret=none args=v
  stub 02188db0 force ret=none args=v
  stub 02043534 force ret=none args=v
  stub 02043658 force ret=u32 args=v
  stub 02189a1c force ret=none args=v,v

# cmd 594: 021BC5A0(02188D20(field map), 02188C9C(field map)); 0.
fn 021b2fb0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188d20 force ret=u32 args=v
  stub 021bc5a0 force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02188c9c force ret=u32 args=v
