# fndiff spec: overlay-21 field-object / camera script commands (F8-spec2), Thumb code 0x021B1668..0x021B2830 (F1-SPLIT row
# 21.44 from the movement commands on).  Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-script scrcmd_obj --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  100 021b1698  101 021b16f0  102 021b1700  103 021b176c  104 021b17b4  105 021b181c  106 021b19b0  107 021b18e4
#  108 021b18b8  109 021b1950  110 021b17f8  111 021b1cb0  112 021b1c2c  113 021b2070  114 021b20b0  115 021b1738
#  116 021b1bf4  117 021b1d20  118 021b1d64  119 021b1e80  120 021b1f60  121 021b19e0  122 021b2104  123 021b1fb8
#  124 021b2154  125 021b21ac  126 021b2218  384 021b2380  385 021b23b8  387 021b23f0  388 021b240c  389 021b2444
#  391 021b247c  392 021b24f4  393 021b24b8  394 021b268c  395 021b269c  396 021b252c  397 021b2568  398 021b25a4
#  400 021b25dc  401 021b2614  402 021b2650  403 021b26ac  404 021b26e4  405 021b271c  407 021b273c  408 021b277c
#  409 021b27f8  410 021b27b4  577 021b1a20  583 021b229c  588 021b2040  603 021b1dec  608 021b23e0
# 63 functions specced (55 command ids): 100 / 101 apply movement + wait (02197D3C movement lists into the SCRCMD WORK slots,
# 0215A6DC / 0215A6FC), 102..126 / 577 / 583 / 588 / 603 field objects (MMDL = overlay-10 fldmmdl: 0216DE94 by id, 0216DEE0 the
# player, 0216D83C / 86C grid x / z, 0216E084 position, 0216DAF8 / DB04 pause flags), the player (021A3xxx), camera / scroll
# tasks (021C6DE0 + 021C7xxx), the talk lock 021b1b08 / 021b1bb8 (flag byte global 0x021EE71C; 021b1bb8 is the target of the
# scrcmd_core thunk at 0x021598E4), 384..410 / 608 thin wrappers of an overlay at 0x021F3xxx (outside the field set; most return
# a child event that the command calls).
# Not specced: 021b1c04 (inside 021b1bf4; two main data words point at it), 021b2300 / 021b237e (inside 021b229c / its pool).
# Note: 021b1e80 (cmd 119) uses its caller's r7 when none of the 4 directions passes 021627C8: the stub always answers nonzero,
# so only direction 0 runs (71 / 89 instructions).
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
area scrcmd_obj
set field

# 021b1668(ctx, id): the object of an id: 0xF2 -> 0216DE54(MMDL system (0215A5EC), 0x30), else 0216DE94(MMDL system, id).
# 0xF1 calls 0215A5F4(ctx) but returns its caller's r6, never set here (cmd 100: the movement offset): not generated here.
fn 021b1668 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 alt const 0xF2 || range 0 0xF0 || range 0xF2 0xFFFF
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216de54 force ret=nullable:4:u32 args=v,v

# cmd 100 (apply movement): id = value, ofs = u32 operand (0201134C); obj = 021b1668(ctx, id); none -> 0; else
# 0215A6DC(ctx, 02197D3C(obj, PC + ofs)) (start the movement list at that script address; slot it for cmd 101); 0.
fn 021b1698 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a6dc force ret=none args=v,v
  stub 021b1668 ret=nullable:4:u32 args=v,v
  stub 0201134c force ret=u32 args=v seq
  stub 02159b10 force ret=pick:0/1/0xF1/0xF2/0xFF args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02197d3c force ret=u32 args=v,v
  # closure: 021b1668
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0216de54 force ret=nullable:4:u32 args=v,v

# wait callback of cmd 101: done once 0215A6FC(ctx) == 0 (no movement running).
fn 021b16dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a6fc force ret=pick:0/1 args=v

# cmd 101 (wait for movements): wait on 0x021B16DD; yield.
fn 021b16f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020113d0 force ret=none args=v,v

# cmd 102: var = 0, then var = 0216D5A0(0216DE94(MMDL system, value)) (the result passed unchecked).
fn 021b1700 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216d5a0 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v

# cmd 115: id, v = values; 0216E0E8(0216DE94(MMDL system, id), v); 0.
fn 021b1738 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0216e0e8 force ret=none args=v,v
  stub 0215a5ec force ret=u32 args=v

# cmd 103: obj = 0216DE94(MMDL system, value); var1 = 0216D83C(obj), var2 = 0216D86C(obj) (grid x / z).
fn 021b176c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5ec force ret=u32 args=v
  stub 0216d86c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0216d83c force ret=u32 args=v

# cmd 104: player = 021A3074(02188CB4(field block +4)); var1 = 0216D83C(player), var2 = 0216D86C(player).
fn 021b17b4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216d86c force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0216d83c force ret=u32 args=v

# cmd 110: var = 0216D5F8(021b1e74(ctx)) (of the player MMDL).
fn 021b17f8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021b1e74 ret=nullable:4:u32 args=v
  stub 0216d5f8 force ret=u16 args=v
  # closure: 021b1e74
  stub 0216dee0 force ret=nullable:4:u32 args=v
  stub 0215a5ec force ret=u32 args=v

# cmd 105: a..f = values; obj = 0216CE6C(MMDL system, (s16)a, (s16)b, c, d, 0216E3D0(0201296C(GAMEDATA), e), f,
# 02188C9C(field block +4)) (stack d, .., ..); obj -> 0216DAF8(obj); 0.
fn 021b181c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0215a5ec force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 0216ce6c force ret=nullable:2:u32 args=v,v,v,v,v,v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216e3d0 force ret=u32 args=v,v
  stub 0216daf8 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02188c9c force ret=u32 args=v

# cmd 108: obj = 0216DE94(MMDL system, value); obj -> 0216CF2C(obj); 0.
fn 021b18b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0216cf2c force ret=none args=v

# cmd 107: data = 02162744(02012844(GAMEDATA)); data -> obj = 0216CEE4(MMDL system, 02162740(..), zone id
# (02012AC8(020127A4(GAMEDATA))), data, 0201296C(GAMEDATA), value) (stack ..); obj -> 0216DAF8(obj); 0.
fn 021b18e4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02162740 force ret=u32 args=v
  stub 020127a4 force ret=u32 args=v
  stub 02162744 force ret=nullable:2:u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02012844 force ret=u32 args=v
  stub 02012ac8 force ret=u16 args=v
  stub 0216cee4 force ret=nullable:2:u32 args=v,v,v,v,v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216daf8 force ret=none args=v

# cmd 109: id, a, b, c, d = values (a, b, c as s16); obj = 0216DE94(MMDL system, id); obj -> 0216E084(obj, a, b, c, d); 0.
fn 021b1950 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0216e084 force ret=none args=v,v,v,v,v

# cmd 106: v = value; obj = 0216DE94(MMDL system, v); obj -> var = 0216D5B0(obj); 0.
fn 021b19b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216d5b0 force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v

# cmd 121: id, n = values; obj = 0216DE94(MMDL system, id); n <= 2 and obj -> var = 0216D63C(obj, n); 0.
fn 021b19e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/0xFFFF args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0216d63c force ret=u32 args=v,v

# cmd 577: obj = 0216DE94(MMDL system, value); obj -> 0216DB80(obj, 1); 0.
fn 021b1a20 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 0216db80 force ret=none args=v,v

# wait callback of the object lock (cmd ..., 021b1b08): flag byte 0x021EE71C: bit 0 and 0216D454(player) == 1 ->
# 0216DAF8(player), clear; bit 2 and 0216DA5C(talk target) == 0 -> 0216DAF8(target), clear; bit 1: obj = 0216DE54(MMDL
# system, 0x30), 0216DA5C(obj) == 0 -> 0216DAF8(obj), clear; bit 3: pair = 0219BBA4(target): none -> clear, else
# 0216DA5C(pair) == 0 -> 0216DAF8(pair), clear; done (1) once the byte is 0.
fn 021b1a44 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  global 021ee71c 4
  fix global0 0 u8 alt range 0 15 || any
  stub 0219bba4 force ret=nullable:2:u32 args=v
  stub 0215a5ec force ret=u32 args=v
  stub 02158f90 force ret=nullable:4:u32 args=v
  stub 0216d454 force ret=pick:0/1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021b1e74 ret=nullable:4:u32 args=v
  stub 0216de54 force ret=nullable:4:u32 args=v,v
  stub 0216daf8 force ret=none args=v
  stub 0216da5c force ret=pick:0/1 args=v
  # closure: 021b1e74
  stub 0216dee0 force ret=nullable:4:u32 args=v

# 021b1b08(vm, ctx): lock the field objects for a talk: player = 021b1e74, target = 02158F90(SW), 0216DE54(MMDL system,
# 0x30), pair = 0219BBA4(target); flags = 0; 0216D9A0(MMDL system) (pause all); 0216D454(player) == 0 -> flags |= 1,
# 0216DB04(player); target and 0216DA5C(target) == 1 -> |= 4, 0216DB04(target); pair and 0216DA5C(pair) == 1 -> |= 8,
# 0216DB04(pair); wait on 0x021B1A45; 1.
fn 021b1b08 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  global 021ee71c 4
  stub 0216d9a0 force ret=none args=v
  stub 0219bba4 force ret=nullable:2:u32 args=v
  stub 0216db04 force ret=none args=v
  stub 0215a5ec force ret=u32 args=v
  stub 02158f90 force ret=nullable:4:u32 args=v
  stub 020113d0 force ret=none args=v,v
  stub 0216de54 force ret=nullable:4:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 021b1e74 ret=nullable:4:u32 args=v
  stub 0216d454 force ret=pick:0/1 args=v
  stub 0216da5c force ret=pick:0/1 args=v
  # closure: 021b1e74
  stub 0216dee0 force ret=nullable:4:u32 args=v

# 021b1bb8(vm, ctx) (the scrcmd_core command at 0x021598E4 jumps here): no talk target -> 0216D9A0(MMDL system) (pause
# all), 0; else 021b1b08(vm, ctx).
fn 021b1bb8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  global 021ee71c 4
  stub 0216d9a0 force ret=none args=v
  stub 021b1b08 ret=u32 args=v,v
  stub 0215a5ec force ret=u32 args=v
  stub 02158f90 force ret=nullable:4:u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  # closure: 021b1b08
  stub 0219bba4 force ret=nullable:2:u32 args=v
  stub 0216db04 force ret=none args=v
  stub 020113d0 force ret=none args=v,v
  stub 0216de54 force ret=nullable:4:u32 args=v,v
  stub 021b1e74 ret=nullable:4:u32 args=v
  stub 0216d454 force ret=pick:0/1 args=v
  stub 0216da5c force ret=pick:0/1 args=v
  stub 0216dee0 force ret=nullable:4:u32 args=v

# 021b1be8(ctx): 0216D9E4(MMDL system) (release the pause; returns its r0).
fn 021b1be8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0216d9e4 force ret=u32 args=v
  stub 0215a5ec force ret=u32 args=v

# cmd 116: d = 021642A4(0216D5F8(player)); target = 02158F90(SW); target -> 0216D5E0(target, d), 0216D674(target, 0); 0.
fn 021b1bf4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216d5e0 force ret=none args=v,v
  stub 021642a4 force ret=u32 args=v
  stub 02158f90 force ret=nullable:4:u32 args=v
  stub 021b1e74 ret=nullable:4:u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216d674 force ret=none args=v,v
  stub 0216d5f8 force ret=u16 args=v
  # closure: 021b1e74
  stub 0216dee0 force ret=nullable:4:u32 args=v
  stub 0215a5ec force ret=u32 args=v

# cmd 112: found, id = two vars; a, b, c = values; obj = 0216DDDC(02188C7C(field block +4), (s16)a, (s16)c, b << 16, 0x8000,
# 0); obj -> found = 1, id = 0216D588(obj); else found = 0.
fn 021b1c2c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216d588 force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0216dddc force ret=nullable:2:u32 args=v,v,v,v,v,v
  stub 02188c7c force ret=u32 args=v

# cmd 111: id, found = two vars; obj = 021A32A8(02188CB4(field map), 0x8000) (the object in front of the player);
# obj -> found = 1, id = 0216D588(obj); else found = 0.
fn 021b1cb0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021a32a8 force ret=nullable:2:u32 args=v,v
  stub 0216d588 force ret=u16 args=v
  stub 0201210c force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# wait callback of cmd 117: done once 021A2F54(02188CB4(field block +4)) == 1.
fn 021b1cfc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021a2f54 force ret=pick:0/1/2 args=v

# cmd 117: v = u16 operand; p = 02188CB4(field block +4); 021A2F48(p, v); 021A2F54(p) == 1 -> 0; else wait on 0x021B1CFD,
# yield.
fn 021b1d20 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021a2f48 force ret=none args=v,v
  stub 02011330 force ret=u16 args=v seq
  stub 020113d0 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021a2f54 force ret=pick:0/1/2 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 118: s, a, b, c = values; s != 0 -> a = -a; v = {0, c == 0 ? -b << 12 : b << 12, 0}; 021C6DE0(02188D38(field
# block +4), 021C78DC(field block +4, a, &v, 021A3074(02188CB4(..))), 0) (a camera / scroll task); 0.
fn 021b1d64 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c6de0 force ret=none args=v,v,v
  stub 02188d38 force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/0x10/0xFFFF args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021c78dc force ret=u32 args=v,v,in:12,v

# cmd 603: s, a, b, c = values; s != 0 -> a = -a; v = {0, c == 0 ? -b << 12 : b << 12, 0}; 021C6DE0(02188D38(field
# block +4), 021C78DC(field block +4, a, &v, 021A3074(02188CB4(..))), 0) (a camera / scroll task); 0.
fn 021b1dec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c6de0 force ret=none args=v,v,v
  stub 02188d38 force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/0x10/0xFFFF args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021c78dc force ret=u32 args=v,v,in:12,v

# 021b1e74(ctx): 0216DEE0(MMDL system) (the player's MMDL).
fn 021b1e74 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0216dee0 force ret=nullable:4:u32 args=v
  stub 0215a5ec force ret=u32 args=v

# cmd 119: dirs = the u16 table 0x021D93FC {0, 1, 2, 3} (stack copy); 021A3038(player = 02188CB4(field map), &pos); the
# first d of dirs with 021627C8(02012844(GAMEDATA), 0201296C(GAMEDATA), &pos, d) != 0 (none: d is the caller's r7, never
# set: the stub always answers nonzero, so only d = 0 is exercised); d == 021A3050(player) -> 0; d 0..3 -> child event
# 021618A0(GAMESYS, field map, 0xFF, the 8-byte ROM record 0x021D941C / 9414 / 940C / 9404 of d); yield.
fn 021b1e80 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021618a0 force ret=u32 args=v,v,v,in:8
  stub 0215a5e4 force ret=u32 args=v
  stub 02012844 force ret=u32 args=v
  stub 021627c8 force ret=pick:1/2/3 args=v,v,in:12,v
  stub 0201296c force ret=u32 args=v
  stub 021a3050 force ret=pick:0/1/2/3 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021a3038 force ret=none args=v,out:12
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 120: var = r = 021A3098(02188CB4(field block +4)) for r 0..3, else 0 (jump table).
fn 021b1f60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021a3098 force ret=pick:0/1/2/3/4/0xFFFFFFFF args=v

# cmd 123: id, x, y, z = values; obj = 0216DE94(MMDL system, id); 0216D88C(obj, &p); v = {x << 16 + 0x8000, y << 16,
# z << 16 + 0x8000}; child event 02161AC0(GAMESYS, obj, &p, &v); yield.
fn 021b1fb8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02161ac0 force ret=u32 args=v,v,in:12,in:12
  stub 0215a5ec force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 588: var = 021A3238(02188CB4(field block +4)).
fn 021b2040 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021a3238 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 113: v1, v2, v3 = var addresses; 021B83FC(0219E348(player MMDL), v1, v2, v3); 0.
fn 021b2070 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0219e348 force ret=u32 args=v
  stub 021b1e74 ret=nullable:4:u32 args=v
  stub 021b83fc force ret=none args=v,v,v,v
  # closure: 021b1e74
  stub 0216dee0 force ret=nullable:4:u32 args=v
  stub 0215a5ec force ret=u32 args=v

# cmd 114: obj = 0216DE94(MMDL system, value); v1, v2, v3 = var addresses; 021B83FC(0219E348(obj), v1, v2, v3); 0.
fn 021b20b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0219e348 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 021b83fc force ret=none args=v,v,v,v

# cmd 122: obj = 0216DE94(MMDL system, value); child event 021C474C(GAMESYS, field block +4, obj); yield.
fn 021b2104 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5ec force ret=u32 args=v
  stub 021c474c force ret=u32 args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 124: a, b, c = values; x = 0219E348(player MMDL); 021B8484(x, a, b, c); 021B7DC0(x, &t); 0219E2C0(player, &t); 0.
fn 021b2154 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b7dc0 force ret=none args=v,out:8
  stub 0219e2c0 force ret=none args=v,in:8
  stub 021b8484 force ret=none args=v,v,v,v
  stub 0219e348 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 021b1e74 ret=nullable:4:u32 args=v
  # closure: 021b1e74
  stub 0216dee0 force ret=nullable:4:u32 args=v
  stub 0215a5ec force ret=u32 args=v

# cmd 125: as cmd 124 for obj = 0216DE94(MMDL system, value).
fn 021b21ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b7dc0 force ret=none args=v,out:8
  stub 0219e2c0 force ret=none args=v,in:8
  stub 021b8484 force ret=none args=v,v,v,v
  stub 0219e348 force ret=u32 args=v
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v

# cmd 126: obj = 0216DE94(MMDL system, value); cam = 02188D38(fm = field block +4); 020061E4(0x676) (SE); v = {0, 0x96000,
# 0}; t1 = 021C7878(fm, 0x18, &v, obj); t2 = 021C7220(fm, 0x18, 3, obj); 021C6DE0(cam, t2, 0); 021C6DE0(cam, t1, 0); 0.
fn 021b2218 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c7220 force ret=u32 args=v,v,v,v
  stub 021c6de0 force ret=none args=v,v,v
  stub 02188d38 force ret=u32 args=v
  stub 020061e4 force ret=none args=v
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021c7878 force ret=u32 args=v,v,in:12,v

# cmd 583: id, x, z, d = values; obj = 0216DE94(MMDL system, id); none -> 0; p = {x << 16 + 0x8000, 0xFA000, z << 16 +
# 0x8000}; h = 0; 02163E58(obj, &p, &h) (ground height); p +4 = h; 0216E084(obj, (s16)x, (s16)(h / 16 rounded to 0 * 16
# >> 16), (s16)z, d); 0216D89C(obj, &p); 0216D8C0(obj, &ROM vector 0x021D9424 {0, 0xFA000, 0}); 021C6DE0(02188D38(fm),
# 021C7684(fm, obj, 0x28, 0xFA), 0); 0.
fn 021b229c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216d8c0 force ret=none args=v,in:12
  stub 021c6de0 force ret=none args=v,v,v
  stub 02188d38 force ret=u32 args=v
  stub 0216e084 force ret=none args=v,v,v,v,v
  stub 021c7684 force ret=u32 args=v,v,v,v
  stub 02163e58 force ret=none args=v,in:12,out:4
  stub 0215a5ec force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0216d89c force ret=none args=v,in:12

# cmd 384 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3E24(GAMESYS, u16 operand & 0xFF); e -> child event e; yield
# (1 either way).
fn 021b2380 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3e24 force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 385: 021F4134(GAMESYS, u16 operand & 0xFF); 0.
fn 021b23b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021f4134 force ret=none args=v,v

# cmd 608: 021F3D74(GAMESYS); 0.
fn 021b23e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3d74 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 387: 021F37A4(field map); 0.
fn 021b23f0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f37a4 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 388: var = 021F37C8(field map) != 0.
fn 021b240c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021f37c8 force ret=pick:0/1/2 args=v
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 389 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F37EC(GAMESYS, u16 operand & 0xFF); e -> child event e; yield
# (1 either way).
fn 021b2444 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f37ec force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 391 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3834(GAMESYS, u16 operand & 0xFF, 1); e -> child event e; yield
# (1 either way).
fn 021b247c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021f3834 force ret=nullable:3:u32 args=v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 393 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3834(GAMESYS, u16 operand & 0xFF, 0); e -> child event e; yield
# (1 either way).
fn 021b24b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021f3834 force ret=nullable:3:u32 args=v,v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 392 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3888(GAMESYS, u16 operand & 0xFF); e -> child event e; yield
# (1 either way).
fn 021b24f4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3888 force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 396 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F4184(GAMESYS, *var (u16)); e -> child event e; yield
# (1 either way).
fn 021b252c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f4184 force ret=nullable:3:u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 397 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F41B0(GAMESYS, *var (u16)); e -> child event e; yield
# (1 either way).
fn 021b2568 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021f41b0 force ret=nullable:3:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 398 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F41DC(GAMESYS, u16 operand); e -> child event e; yield
# (1 either way).
fn 021b25a4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021f41dc force ret=nullable:3:u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 400 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F398C(GAMESYS, u16 operand); e -> child event e; yield
# (1 either way).
fn 021b25dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f398c force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 401 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3A28(GAMESYS, u16 operand != 0); e -> child event e; yield
# (1 either way).
fn 021b2614 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3a28 force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 402 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3770(GAMESYS, u16 operand != 0); e -> child event e; yield
# (1 either way).
fn 021b2650 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 021f3770 force ret=nullable:3:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 394: 021F42A8(GAMESYS); 0.
fn 021b268c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f42a8 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 395: 021F3970(GAMESYS); 0.
fn 021b269c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3970 force ret=none args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 403 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F384C(GAMESYS, u16 operand); e -> child event e; yield
# (1 either way).
fn 021b26ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f384c force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 404 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3888(GAMESYS, u16 operand); e -> child event e; yield
# (1 either way).
fn 021b26e4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3888 force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 405: 021F38C4(GAMESYS, u16 operand); 0.
fn 021b271c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02011330 force ret=u16 args=v seq
  stub 021f38c4 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 407 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3A88(GAMESYS, u16 a, u16 b); e -> child event e; yield
# (1 either way).
fn 021b273c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3a88 force ret=nullable:3:u32 args=v,v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 408 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3B5C(GAMESYS, u16 operand); e -> child event e; yield
# (1 either way).
fn 021b277c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3b5c force ret=nullable:3:u32 args=v,v
  stub 02011330 force ret=pick:0/1/2/0xFF/0x1FF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 410 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3AD0(GAMESYS, a, b) (values); e -> child event e; yield
# (1 either way).
fn 021b27b4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 021f3ad0 force ret=nullable:3:u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 409 (calls into the overlay at 0x021F3xxx, outside the field set): e = 021F3B18(GAMESYS, value); e -> child event e; yield
# (1 either way).
fn 021b27f8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021f3b18 force ret=nullable:3:u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
