# fndiff spec: overlay-10 script commands of F1 rows 10.27 and 10.34 (F8-spec5).  Row 10.27 ("symbol_map.c", 0x021664CC..0x02166F9C):
# only the command handlers and their in-row helpers are here (the rest of symbol_map.c is F5b's: force stubs); row 10.34
# (0x0216AC40..0x0216B504, no __FILE__ string, between the F8-spec3 rows 10.33 / 10.35) is a whole unanchored scrcmd file.
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_misc10b --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  310 02166f1c  425 02166734  427 021667e0  477 02166980  478 021669c0  480 02166a00  493 02166d30  554 0216ae10
#  555 0216ad70  556 0216accc  557 0216ad1c  558 0216ac40  560 0216ac90
# 35 functions specced (13 command ids).
#  * Row 10.27: 425 / 427 query and wait on the pleasure-boat work (*02012ED0(GD), 0x1B8 bytes; its owner is overlay-21 pleasure_boat.c,
#    spec scrcmd_misc21b): time s16 W+4 (0x546 per unit), W[8], records W + 0x14 + 0x1C i; 427's event waits for the app 02012444(0xCB).
#    477 / 478 read a 20-record save-block record (02012F38(GD)); 480 + its event 0x02166908 read the save block 02009470 (the 20-record
#    block before wifilist.c in ARM9; F8-spec4 misc21 cmds 472..483 use it) into a 0x128-byte work and run the app 0xDA; 493 + its event
#    0x02166E50 run the app 0x6E with a 02024044 record (0x1C-byte work) and copy the result into three vars; 310 sets the field weather
#    (02188C6C / 021A1BDC / 021A1BE4; 0xFFFF = the zone's weather 0202C794) and 020129A0(GD, w).
#  * Row 10.34: the Entree Forest symbol encounters.  Y = 0200E0B0(save) (symbol save), the OBJ of an id via 0216AED0; 558 / 556 / 557 /
#    560 query / update the symbol of an OBJ (overlay-18 0217C274 / 0217C294 / 0217C2A0 / 0217C37C: intrude_field.c, X1 Entralink:
#    force stubs), 555 queries (021767A0 (ov11) state 4, 02166618 / 0216662C symbol_map flags, packed symbol word fields), 554 starts the
#    warp into / out of the forest (0216B3A8 -> 0216B340 GMEVENT 0x0216B3EC: OBJ pause, the comm wait 0216B24C / 0x0216B280 when the
#    intrude state is not 4, fade, map change 021854B0, fade in) or the warp 02184D38 with sub 1.  0216AF74 / 0x0216B004 are the symbol
#    battle event (callers outside the scrcmd table; overlays 0x62 / 0xE3 are loaded at 0x021B6100 / 0x021B95A0 around the app calls:
#    their entry points are force stubs).  Roles UNVERIFIED (named by address).
# Not specced (0):
#   none
# Fragments (not functions): 0x021666C8 (a data word at 0x0205CCE0 points into 0x021666B8), 0x0216B46C (a branch target inside
# 0x0216B3EC), 0x0216B488 (a data word at 0x020A3E6A decodes as `bl`).
#
# Read from the disassembly of the raw OVY_10 / OVY_20 / OVY_21.sbin (rizin `-a arm -b 16 -m <base>`: 0x02155160 / 0x02183B60 /
# 0x02187F00; Ghidra's field set drops the arguments of overlay calls).  Function list: command-table targets + `bl` targets from the whole
# field set + odd code pointers + labels, walked as Thumb from every entry; a `bl` into the function's own extent is a long branch, not a
# call (MWCC: bsubway cmd 346), and jump tables bounded by `subs; bpl` instead of `cmp #N` are sized by hand (0x021CAB40: 17,
# 0x0216BDAC: 9).
#
# Spec conventions (as the F8-spec1..4 areas): every block is `nowork`.  A command handler `fn(vm, ctx)` gets two arenas: arg0 = the VM
# (`arg buf 0x800`: regs at 0x40, script bytes at 0x500, PC +0x14, compare byte +0xE in 0..2), arg1 = the scrcmd ctx (`arg buf 0x800`:
# CTX at 0, SCRCMD WORK at 0x100, zone A 0x300 (Heap_AllocDebug blocks, SW records), zone B 0x400, 0x500 (records a stub hands out), vars
# 0x600..0x7FF where `02159ae8` points).  A helper that takes the ctx in r0 gets the ctx arena as arg0.  A GMEVENT function `(event,
# seq, work)` gets `arg u32, arg buf 0x10 (seq, bounded), arg buf (work)`.  Script-operand readers (vm.c 02011330 / 0201134c, script_sys
# 02159ae8 / 02159b10) are `seq` stubs; operands that pick a case or index a table are `pick:` / `nth:` sets.
# Closure stubs as in core4 (every function of the area treated as ported: no fault, stray write or unstubbed callee).  **Every stub
# outside the area carries `force`** (overlay addresses collide with battle `PORTED` entries; the F8 areas call each other only through
# forced stubs).  Heap_AllocDebug (02030734) is a `seqptr` bump allocator into an arena; GMEVENT create 020122C0 is `nullable` (null = no
# event).  Callees in overlays outside the field set (0x021F3xxx / 0x021F5xxx overlays loaded around the call, the 0x021B6100 / 0x021B95A0
# overlays of the symbol encounter) and the X1 networking callees (comm, beacon, Wi-Fi) are force stubs.  An unset callee-saved register
# or stack word that a path reads is 0 under the Z19 entry state; the block says so.
area scrcmd_misc10b
set field

# boat_slot_word(W, i): W == 0 -> 10; else u32 W + 0x14 + 0x1C * i.
fn 02166658 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || const 0x02E30000
  arg u32 range 0 15
  arg buf 0x200

# boat_wait_event(GAMESYS, W, u16 *var): ev = 020122C0(GAMESYS, 0, 0x021666B9, 0x20); E = 020124E8(ev); E+0x18 = W, E+0x1C = var;
# 02161CDC(E, GAMESYS, 1, 0); E+8 = 0216670C(W) (s16 time); ev.  Arena: arg 3.
fn 0216666c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  arg u32 any
  arg buf 0x100
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v
  stub 02161cdc force ret=none args=v,v,v,v
  stub 0216670c ret=u32 args=v

# GMEVENT of 0216666C (event, seq, -): E = 020124E8(event); seq 0: 02012444(event, 0xCB, 0x021BFB3C) (child app proc), seq 1;
# seq 1: *E[0x1C] = (E[0x14] == 2) (u16); 02166714(E[0x18], E[0x10]); 1; else 0.  Arena: arg 2 (E; W at +0x40, var at +0x80).
fn 021666b8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 0x18 u32 addrof arg2 0x40
  fix arg2 0x1C u32 addrof arg2 0x80
  fix arg2 0x14 u32 alt range 1 3 || any
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 02166714 ret=none args=v,v
  stub 02012444 force ret=none args=v,v,v

# boat_time(W): s16 W+4 (sign-extended).
fn 0216670c ret=u32 inputs=200
  nowork
  arg buf 0x10

# boat_time_set(W, t): W+4 = t (u16); d = (s16 W+4) / 0x546 (_s32_div_f); W[8] <= d (signed) -> W[8] = d.
fn 02166714 ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 alt range 0 0x4000 || any
  fix arg0 8 u32 alt range 0 12 || any

# cmd 425: B = 02012ED0(GD) (boat-work slot); i = value; k = u16 operand; v = value; var; k 0..3 -> *var = 0; 4: 02166658(*B, i);
# 5: 021C87A0(*B, i); 6: 021C87AC(*B, i); 7: 021C87B8(*B, i, v); 8: 021C87C8(*B, i) != 0; k > 8 -> 0 (pleasure_boat.c queries; u16).
fn 02166734 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 addrof arg1 0x200
  stub 021c87a0 force ret=none args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021c87c8 force ret=pick:0/1/2 args=v,v
  stub 02166658 ret=none args=v,v
  stub 021c87ac force ret=none args=v,v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 02159b10 force ret=pick:0/1/14 args=v,v seq
  stub 02011330 force ret=pick:0/1/2/3/4/5/6/7/8/9 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021c87b8 force ret=none args=v,v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 427: B; var; ev = 0216666C(GAMESYS, *B, var); ev -> child event; 1.
fn 021667e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 addrof arg1 0x200
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216666c ret=nullable:2:u32 args=v,v,v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 0216666c
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02161cdc force ret=none args=v,v,v,v
  stub 0216670c ret=u32 args=v

# rec20_byte(R, i): i < R[1] (u8) -> R[3 + i]; else 0xB.
fn 02166824 ret=u8 inputs=200
  nowork
  arg buf 0x40
  arg u32 range 0 0x20
  fix arg0 1 u8 alt range 0 0x20 || any

# rec20_field(R, k): k 0 -> R[1]; 1 -> R[0]; 2 -> R[2]; else 0.
fn 02166834 ret=u8 inputs=200
  nowork
  arg buf 0x10
  arg u32 alt range 0 3 || any

# rec20_event(GAMESYS, a, A, B, C): ev = 020122C0(GAMESYS, 0, 0x02166909, 0x128); E = 020124E8(ev); E[0] = a (u8), E+0x11C = A,
# E+0x120 = B, E+0x124 = C (5th argument); X = 02009470(02012EBC(GAMEDATA)) (the 20-record save block); E[0x30 + i - 1] =
# 02009794(X, i) != 0 for i = 1..0xE8; n = min(020095C8(X), 0x14); E+8 + 2i = 0200856C(0200973C(X, i)) (u16) for i < n, 0 for
# n <= i < 0x14; ev.  Arena: arg 5.
fn 02166854 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x200
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 0200973c force ret=u32 args=v,v
  stub 020124e8 force ret=ptr:arg5:0:1 args=v
  stub 020095c8 force ret=pick:0/1/2/0x13/0x14/0x15/0xFFFFFFFF args=v
  stub 0200856c force ret=u16 args=v
  stub 02009470 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02009794 force ret=pick:0/1 args=v,v
  stub 02012ebc force ret=u32 args=v

# GMEVENT of 02166854 (event, seq, -): E; seq 0: 02012444(event, 0xDA, 0x021BAB5C, E) (child app), seq 1; seq 1: E[4] == 1 ->
# *A = 1, *B = E[1] (u8), *C = (u16)E[2]; else *A = *B = *C = 0; 1; else 0.  Arena: arg 2 (E; A / B / C at +0x40 / +0x80 / +0xC0).
fn 02166908 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x200
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 0x11C u32 addrof arg2 0x40
  fix arg2 0x120 u32 addrof arg2 0x80
  fix arg2 0x124 u32 addrof arg2 0xC0
  fix arg2 4 u32 alt range 0 2 || any
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 02012444 force ret=none args=v,v,v,v

# cmd 477: R = 02012F38(GD); k = u16 operand (u8); var; *var = 02166824(R, k); 0.
fn 02166980 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x501 u8 alt range 0 0x14 || any
  stub 02166824 ret=u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02011330 force ret=pick:0/1/2/3/0x13/0x14/0xFF args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 478: R; k = value; var; *var = 02166834(R, k); 0.
fn 021669c0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02166834 ret=u32 args=v,v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 480: a = value; A, B, C = vars; ev = 02166854(GAMESYS, a, A, B, C); ev -> child event; 1.
fn 02166a00 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02166854 ret=nullable:2:u32 args=v,v,v,v,v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 02166854
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 0200973c force ret=u32 args=v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 020095c8 force ret=u32 args=v
  stub 0200856c force ret=u32 args=v
  stub 02009470 force ret=u32 args=v
  stub 02009794 force ret=u32 args=v,v
  stub 02012ebc force ret=u32 args=v

# cmd 493: k = value; A, B, C = vars; ev = 020122C0(GAMESYS, 0, 0x02166E51, 0x1C); E = 020124E8(ev) cleared (0x1C bytes); E[0] =
# k, E[4] = A, E[8] = B, E[0xC] = C, *C = 0; k 0..5: (E[0x14], E[0x18], s) = (1, 1, 2), (2, 1, 2), (3, 1, 2), (4, 1, 2), (-, 0, 0),
# (-, 0, 1); E[0x10] = 02024044(s, 0, 0, 0, save, 1); E[0x18] != 0 -> 0200E6F0(save) + 0200E6FC(R, E[0x14], &rec) (0xC-byte
# record), 02024108(E[0x10], &rec); k > 5: E[0x10] = 0; 0215897C(SW, ev); 1.
fn 02166d30 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 02012ebc force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02024044 force ret=u32 args=v,v,v,v,v,v
  stub 0200e6fc force ret=none args=v,v,out:12
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02024108 force ret=none args=v,in:12
  stub 02159b10 force ret=pick:0/1/2/3/4/5/6 args=v,v seq
  stub 0200e6f0 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT of cmd 493 (event, seq, -): E; X = E[0x10]; seq 0: X == 0 -> 1 (end); else 02012444(event, 0x6E, 0x021C39B8, X), seq 1;
# seq 1: 0202411C(X) != 0 -> *C = 0; else *C = 1 and: k 5: 02024128(X, &w) -> *A = w.lo, *B = w.hi (u16s); k 4: *A =
# 02024124(X), *B = 0; other k: 02024134(X, &r) (0xC bytes), E[0x18] -> 0200E714(0200E6F0(save), E[0x14], &r); *A = r+4, *B =
# r+6 (u16s); 020240FC(X) (free); 1; else 0.  Arena: arg 2 (E; A / B / C at +0x40 / +0x80 / +0xC0).
fn 02166e50 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x200
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 4 u32 addrof arg2 0x40
  fix arg2 8 u32 addrof arg2 0x80
  fix arg2 0xC u32 addrof arg2 0xC0
  fix arg2 0 u32 alt range 3 6 || any
  fix arg2 0x10 u32 alt const 0 || any
  fix arg2 0x18 u32 alt const 0 || any
  stub 02012ebc force ret=u32 args=v
  stub 02024124 force ret=u32 args=v
  stub 020124e4 force ret=u32 args=v
  stub 02012444 force ret=none args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 02024128 force ret=none args=v,out:4
  stub 0200e6f0 force ret=u32 args=v
  stub 020240fc force ret=none args=v
  stub 02024134 force ret=none args=v,out:12
  stub 020120f4 force ret=u32 args=v
  stub 0200e714 force ret=none args=v,v,in:12
  stub 0202411c force ret=pick:0/1 args=v

# cmd 310: fm; W = 02188C6C(fm) (weather); w, f = values; w == 0xFFFF -> w = 0202C794(GAMESYS, 02188C9C(fm)) (zone weather);
# f != 0 -> 021A1BE4(W, w) else 021A1BDC(W, w); 020129A0(GAMEDATA, w & 0xFF); 0.
fn 02166f1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020129a0 force ret=none args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 021a1be4 force ret=none args=v,v
  stub 021a1bdc force ret=none args=v,v
  stub 02188c6c force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/0xFFFF args=v,v seq
  stub 02188c9c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0202c794 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 558: fm; I = 02012138(GAMESYS) (intrude / comm system); R = 021664CC(h, GAMESYS, &k) (symbol_map.c: the symbol record,
# heap block or null); R -> 0217C37C(fm, k, R, R[0x50] & 0x3F, (R[0x50] >> 18) & 0x3F) (ov18), Heap_Free(R); 0.
fn 0216ac40 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0217c37c force ret=none args=v,v,v,v,v
  stub 021664cc force ret=nullable:3:ptr:arg1:0x300:1 args=v,v,outm:4:0xFF/0xFF/0/0
  stub 0215a5cc force ret=u16 args=v
  stub 0201210c force ret=u32 args=v
  stub 020307b0 force ret=none args=v
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 560: id = value; Y; m = 0216AED0(ctx, id); m -> 0200E18C(Y, 0217C294(m)) (symbol id of the OBJ), 0216AF3C(ctx); 0.
fn 0216ac90 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0216af3c ret=none args=v
  stub 0200e18c force ret=none args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200e0b0 force ret=u32 args=v
  stub 0216aed0 ret=nullable:2:u32 args=v,v
  stub 0217c294 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 0216af3c / 0216aed0
  stub 02158344 force ret=nullable:2:ptr:arg2:0:1 args=v
  stub 0217688c force ret=u32 args=v,v
  stub 02012fb0 force ret=u32 args=v
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0201210c force ret=u32 args=v

# cmd 556: id; var; Y; m = 0216AED0(ctx, id); m -> *var = 0216628C(Y, 0217C294(m)), 0216AF3C(ctx); else *var = 0; 0.
fn 0216accc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0216af3c ret=none args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216628c force ret=u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200e0b0 force ret=u32 args=v
  stub 0216aed0 ret=nullable:2:u32 args=v,v
  stub 0217c294 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 0216af3c / 0216aed0
  stub 02158344 force ret=nullable:2:ptr:arg2:0:1 args=v
  stub 0217688c force ret=u32 args=v,v
  stub 02012fb0 force ret=u32 args=v
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0201210c force ret=u32 args=v

# cmd 557: id, k = values; m = 0216AED0(ctx, id); m -> B = 0216AF1C(ctx, m, h, &tmp); B -> 0201EE84(02158F74(SW), k, B) (word set
# from a strbuf), Heap_Free(B); 0.
fn 0216ad1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201ee84 force ret=none args=v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0216aed0 ret=nullable:2:u32 args=v,v
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0216af1c ret=nullable:2:u32 args=v,v,v,v
  # closure: 0216aed0 / 0216af1c
  stub 02188c7c force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0217c274 force ret=u32 args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0217c2a0 force ret=u32 args=v,v,v

# cmd 555: k = value; var; save; Y; I = 02012138(GAMESYS); k 0: *var = 021767A0(I, GAMEDATA) == 4; 1: 02166618(02012FB0(GD));
# 2: 0216662C(02012FB0(GD)); 3: 0216AEE8(ctx, &w), *var = w & 0x7FF; 4: 0216AEE8(ctx, &w), *var = (w >> 23) & 0x3F; else
# nothing; 0.
fn 0216ad70 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021767a0 force ret=pick:0/3/4 args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216aee8 ret=none args=v,outm:4:0xFF/0xFF/0xFF/0xFF
  stub 02166618 force ret=none args=v
  stub 0216662c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4/5 args=v,v seq
  stub 0200e0b0 force ret=u32 args=v
  stub 02012fb0 force ret=u32 args=v
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 0216aee8
  stub 0216aed0 ret=nullable:3:u32 args=v,v
  stub 0217c274 force ret=u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215901c force ret=u32 args=v,v,v
  stub 02188c7c force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0201210c force ret=u32 args=v

# cmd 554: SW; k = value; D = 02012FB0(GD); k == 1 and 0216662C() != 0 -> P = 02188CB4(fm), I, 021A3038(P, &pos), 021767F0(I, GD,
# &dst) (12 bytes), dst.x += pos.x - 0xF8000, ev = 02184D38(GAMESYS, fm, 0x117, &dst, 1, 0) (warp); else k != 9 -> D = 021665F0(
# GAMESYS, 02012FB0(GD), k); ev = 0216B3A8(GAMESYS, k, D); child event; 1.
fn 0216ae10 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02184d38 force ret=u32 args=v,v,v,in:12,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0216b3a8 ret=u32 args=v,v,v
  stub 0216662c force ret=pick:0/1 args=
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/8/9/10 args=v,v seq
  stub 02012fb0 force ret=u32 args=v
  stub 021767f0 force ret=none args=v,v,out:12
  stub 021665f0 force ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02012138 force ret=u32 args=v
  stub 021a3038 force ret=none args=v,out:12
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 0216b3a8
  stub 0216b340 ret=u32 args=v,v,v,v,v
  stub 0216b2e4 ret=none args=v,v,v
  stub 0216d888 force ret=ptr:arg1:0x5C0:1 args=v
  stub 021a3074 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 021767a0 force ret=u32 args=v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v

# symbol_mmdl(ctx, id): 0216DE94(02188C7C(0201210C(GAMESYS)), id).
fn 0216aed0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  stub 02188c7c force ret=u32 args=v
  stub 0216de94 force ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# symbol_word(ctx, out): v = 0215901C(SW, GAMEDATA, 0x8011) (script var); m = 0216AED0(ctx, v); 0217C274(out, m) (packed symbol
# word of the OBJ).
fn 0216aee8 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg buf 0x10
  stub 0215a5e4 force ret=u32 args=v
  stub 0216aed0 ret=nullable:3:u32 args=v,v
  stub 0217c274 force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215901c force ret=u16 args=v,v,v
  # closure: 0216aed0
  stub 02188c7c force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# symbol_name(ctx, m, h, w): GD; 0217C274(w, m); 0217C2A0(4, GD, w) (the name strbuf; returned).
fn 0216af1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 0217c274 force ret=none args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0217c2a0 force ret=u32 args=v,v,v

# symbol_sync(ctx): I = 02012138(GAMESYS); Y; X = 02158344(I) (X1 delivery object); X -> 0217688C(X, 02012FB0(GD)).
fn 0216af3c ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02158344 force ret=nullable:2:u32 args=v
  stub 0217688c force ret=none args=v,v
  stub 0200e0b0 force ret=u32 args=v
  stub 02012fb0 force ret=u32 args=v
  stub 02012138 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# symbol_battle_event(GAMESYS, a, mon, w, p5, u16 p6) (push {r0-r3}: 6 argument words): ev = 020122C0(GAMESYS, 0, 0x0216B005,
# 0x3C); E cleared; E[0] = a, E+0x28 = p5, E+0x24 = p6, E+0xC = mon, E+0x20 = w; E+0x2C = 0203F1FC(mon, 020173EC()) (u16);
# 0200E2BC(save, &w, p6, 0) == 0 -> E+0x20 = (E+0x20 & ~0x7FF) | 0x28B; 021AA304(E + 0x10, GAMEDATA, a); ev.  Arena: arg 6.
fn 0216af74 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 02012ebc force ret=u32 args=v
  stub 0200e2bc force ret=pick:0/1 args=v,in:4,v,v
  stub 021aa304 force ret=none args=v,v,v
  stub 020124e8 force ret=ptr:arg6:0:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0203f1fc force ret=u16 args=v,v

# GMEVENT of 0216AF74 (event, seq, E): the symbol encounter: 9 states (fade 02029BCC / 02029C00, the app 02012444 calls with the
# overlays 0x62 / 0xE3 loaded at 0x021B6100 / 0x021B95A0 (their entry points are force stubs), the encounter record update
# 0200E2BC, 0x021CF498 effect, 020144DC / 020145CC / 02029CC0); ends freeing E+0xC / E+4.  Arena: arg 2.
fn 0216b004 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  fix arg1 0 u32 alt range 0 9 || any
  fix arg2 0x28 u32 addrof arg2 0xC0
  stub 02029c00 force ret=none args=v
  stub 021b6100 force ret=u32 args=v,v,v,v,v,v,v,v
  stub 02189a08 force ret=u32 args=v
  stub 0200ca10 force ret=u32 args=v
  stub 021cf498 force ret=none args=v,v,v
  stub 02007f1c force ret=none args=v,v,v,v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  stub 021b95a0 force ret=u32 args=v,v,v,v
  stub 0201292c force ret=u32 args=v
  stub 020307b0 force ret=none args=v
  stub 02012ebc force ret=u32 args=v
  stub 0200e2bc force ret=pick:0/1 args=v,in:4,v,v
  stub 0200873c force ret=u32 args=v
  stub 02029cc0 force ret=none args=v
  stub 02034ac4 force ret=none args=v
  stub 02012444 force ret=none args=v,v,v,v
  stub 02012944 force ret=u32 args=v
  stub 021b95c8 force ret=none args=v
  stub 020145cc force ret=u32 args=v
  stub 02029bcc force ret=u32 args=v,v
  stub 02007e50 force ret=none args=v,v,v,v
  stub 021b6158 force ret=pick:0/1/2 args=v
  stub 02034a5c force ret=none args=v
  stub 020144dc force ret=u32 args=v,v
  stub 020124e4 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v
  stub 020120f4 force ret=u32 args=v

# symbol_wait_event(GAMESYS, rec, k): ev = 020122C0(GAMESYS, 0, 0x0216B281, 8); E cleared = {rec, u8 k}; ev.  Arena: arg 3.
fn 0216b24c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v
  stub 020120f4 force ret=u32 args=v

# GMEVENT of 0216B24C (event, seq, E): X = 02158344(02012138(GAMESYS)); X null -> *E[0] = 0 (if E[0]), 1; seq 0: 02176850(X, E[4])
# == 1 -> seq 1; seq 1: 02176870(X) == 1 -> seq 2; seq 2: *E[0] = 1 (if E[0]), 1; else 0.
fn 0216b280 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 0 u32 alt const 0 || addrof arg2 0x20
  stub 020124e4 force ret=u32 args=v
  stub 02158344 force ret=nullable:3:u32 args=v
  stub 02176870 force ret=pick:0/1 args=v
  stub 02176850 force ret=pick:0/1 args=v,v
  stub 02012138 force ret=u32 args=v

# symbol_warp_pos(src, k, dst): dst = *src (12 bytes); k 0: dst.z = 0x188000; 1: dst.z = 0x98000; 2: dst.x = 0x1A8000; 3: dst.x =
# 0x58000; 4..9: nothing.
fn 0216b2e4 ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 alt range 0 10 || any
  arg buf 0x10

# symbol_warp_event(GAMESYS, f, &pos, zone, k): ev = 020122C0(GAMESYS, 0, 0x0216B3ED, 0x24); E cleared; E+8 = pos (12 bytes),
# E+0x14 = zone (u16), E[0] = 0, E+0x16 = k (u8, 5th argument), E+0x1C = f; 021767A0(02012138(GAMESYS), GAMEDATA) == 4 ->
# E+0x20 = 1; ev.  Arena: arg 5.
fn 0216b340 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 021767a0 force ret=pick:0/4 args=v,v
  stub 020124e8 force ret=ptr:arg5:0:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012138 force ret=u32 args=v

# symbol_warp(GAMESYS, k, zone): fm; P; 0216D888(021A3074(P)) (OBJ position, 12 bytes); 0216B2E4(pos, k, &dst); ev =
# 0216B340(GAMESYS, k != 9, &dst, k, zone); ev.
fn 0216b3a8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 alt range 0 10 || any
  arg u32 any
  arg buf 0x400
  stub 0216b340 ret=u32 args=v,v,in:12,v,v
  stub 0216b2e4 ret=none args=in:12,v,out:12
  stub 0216d888 force ret=ptr:arg3:0:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  # closure: 0216b340 / 0216b2e4
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 021767a0 force ret=u32 args=v,v
  stub 020124e8 force ret=ptr:arg3:0x300:1 args=v
  stub 02012138 force ret=u32 args=v

# GMEVENT of 0216B340 (event, seq, E): seq 0: 0216D9A0 (OBJ pause); E+0x20 == 1 -> seq 2, else child 0216B24C(GAMESYS, E + 0x18,
# E+0x16), seq 1; seq 1: E+0x18 == 0 -> *E[0] = 0, seq 6; else seq 2; seq 2: E+0x1C -> SE 0x559 (020061E4); child 021BE8D4(
# GAMESYS, fm, 9, 0) (fade out), seq 3; seq 3: E+4 = 021665D0(GAMESYS, E+0x16), 02012FA4(GD, E+0x16), child 021854B0(GAMESYS, fm,
# E+4, E + 8, E+0x14) (map change), seq 4; seq 4: 0219C630(MMDLs) != 1 -> child 021BE92C(GAMESYS, fm, 9, 0, 0, 0, 0) (fade in),
# seq 5; seq 5: *E[0] = 1, seq 6; seq 6: 0216D9E4 (resume), 1; else 0.
fn 0216b3ec ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x400
  fix arg1 0 u32 alt range 0 7 || any
  fix arg2 0 u32 alt const 0 || addrof arg2 0x40
  fix arg2 0x20 u32 alt range 0 1 || any
  fix arg2 0x18 u16 alt const 0 || any
  stub 0216d9a0 force ret=none args=v
  stub 020124e4 force ret=u32 args=v
  stub 0216d9e4 force ret=none args=v
  stub 02012fa4 force ret=none args=v,v
  stub 020061e4 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v
  stub 0216b24c ret=u32 args=v,v,v
  stub 0219c630 force ret=pick:0/1 args=v
  stub 021665d0 force ret=u16 args=v,v
  stub 021854b0 force ret=u32 args=v,v,v,in:12,v
  stub 020120f4 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v
  stub 021be8d4 force ret=u32 args=v,v,v,v
  stub 02012138 force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v
  # closure: 0216b24c
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0x300:1 args=v
