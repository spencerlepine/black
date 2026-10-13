# fndiff spec: overlay-10 script commands of F1 rows 10.24 (calender.c), 10.32 (tail), 10.36, 10.37 (tail) and 10.40 (tail) (F8-spec5).
# calender.c (0x02164778..0x02164B6C): only the four command handlers and the GMEVENT of 468 (the rest of the file is F11's); cmd 505
# (0x0216A3F8) is the first function of the F8-spec3 unanchored file of row 10.33 (it allocates the fld_btl_inst_tool work of 506..520);
# row 10.36 (0x0216B86C..0x0216BF9C, no __FILE__ string) is a whole unanchored scrcmd file; cmd 572 + its helper close namein_setup.c
# (0x0216CC30 / 0x0216CC40) and cmds 590..592 + 0x0216F900 close trcard_sys.c.
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_misc10c --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  445 0216bf38  458 02164778  467 02164910  468 02164994  469 02164978  505 0216a3f8  534 0216b86c  544 0216bd54
#  545 0216bbd4  546 0216bc88  547 0216bce4  548 0216bd1c  552 0216bec0  572 0216cc30  590 0216f8b8  591 0216f8c8
#  592 0216f8dc
# 37 functions specced (17 command ids).
#  * calender.c 458 / 467 / 468 / 469: each loads an overlay (0x2F, 0x52) and calls into it (0x021F3800, 0x021F5260, 0x021F5550,
#    0x021F558C: outside the field set, force stubs); 467 / 468 set / clear the field-proc flag 7 (02159144 / 02159168).
#  * Row 10.36: a 47-entry ROM table of 7-byte challenge records at 0x021723BC {kind 0 / 1, n, ids[3], a, b} (ends at 0x02172508, where
#    the 8 u32 keys of 0x0216BF74 start); S = 0200C0E8(save), c = 0200C32C(S) the current challenge, T = 0200E394(save) the records;
#    534 = the challenge value left (kind 0: goal - min record sum, capped at 999999; kind 1: days left from the 64-bit start time
#    0200C37C and the clock 0203F524 / 020891E0); 0x0216B8B8 / 0x0216B91C (callers outside the table) the challenge state.  544 / 552
#    look for / update a party mon of species 0xF3..0xFB / 0x1EC / 0x287..0x289 with a form / move kind (0202F488 / 0202F630); 545..548
#    call the overlay at 0x021F39B8 / 0x021F3AA0 / 0x021F3B40 / 0x021F3B68 (camera / scroll events: fx32 positions x << 16 + 0x8000);
#    445 reads 0200D468(0200D418(save), i).
#  * 572 / 590: set the event-work flag words 0x34 / 0x36 (0200B3A4(02012EC4(GD), id)); 591 / 592: trainer-card overlay 0x021F385C /
#    0x021F38BC (an event).  Roles UNVERIFIED (named by address).
# Not specced (0):
#   none
# Fragments (not functions): 0x0216BE32 (a data word at 0x020A4808 decodes as `bl` into 0x0216BE20).  Dead code: 0x0216BDD2 (544's
# compare tree: no species is > 0x287, < 0x288 and != 0x288).
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
area scrcmd_misc10c
set field

# cmd 458: 02034AC4(0x2F) (overlay load); 021F3800(GAMESYS); 02034A5C(0x2F) (unload); 0.
fn 02164778 ret=u32 inputs=200
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

# cmd 467: a, b, c = u16 operands (a, b truncated to u8); 02159144(7) (field-proc flag); 02034AC4(0x52); ev = 021F5260(a, b, c,
# GAMESYS, ctx) (5th argument on the stack); ev -> child event; 1.
fn 02164910 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f5260 force ret=nullable:2:u32 args=v,v,v,v,v
  stub 02034ac4 force ret=none args=v
  stub 02159144 force ret=none args=v
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 469: 021F558C(0201210C(GAMESYS)); 0.
fn 02164978 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f558c force ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 468: ev = 020122C0(GAMESYS, 0, 0x021649E9, 0); ev -> child event; 1.
fn 02164994 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=nullable:2:u32 args=v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# GMEVENT of cmd 468 (event, seq, -): seq 0: 02012374(event, 021F5550()) (child), seq 1; seq 1: 02159168(7), 02034A5C(0x52), 1;
# else 0.
fn 021649e8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  fix arg1 0 u32 alt range 0 2 || any
  stub 020124e4 force ret=u32 args=v
  stub 02159168 force ret=none args=v
  stub 021f5550 force ret=u32 args=
  stub 02012374 force ret=none args=v,v
  stub 02034a5c force ret=none args=v

# cmd 505: G = 02012F50(GAMEDATA) (record whose word 0 is the fld_btl_inst_tool work of cmds 506..520); *G == 0 -> *G =
# 021D096C(GAMESYS) (overlay-21 trial_house.c alloc); 0.
fn 0216a3f8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x4C0 u32 alt const 0 || any
  stub 02012f50 force ret=ptr:arg1:0x4C0:1 args=v
  stub 021d096c force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v

# cmd 534: save = 02012EBC(GAMEDATA); S; var; c; kind = 0216BB70(c): 0 -> *var = 0216BA88(save); 1 -> 0216BAE8(save); else
# *var = the caller's r5 (never set on that path: 0 under the Z19 entry state); 0.
fn 0216b86c ret=u32 inputs=200
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
  stub 0216bae8 ret=none args=v
  stub 0216ba88 ret=u32 args=v
  stub 0200c32c force ret=pick:0/1/2/11/12/13/23/24/34/35/46 args=v
  stub 0216bb70 ret=pick:0/1/2 args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 0216bae8 / 0216ba88 / 0216bb70
  stub 0216bbc4 ret=u32 args=v
  stub 0200e394 force ret=u32 args=v
  stub 0216bb18 ret=u16 args=v
  stub 0216bb80 ret=u32 args=v
  stub 0216bb90 ret=none args=v,outm:3:0xFF/0xFF/0xFF
  stub 0216b970 ret=u32 args=v,v
  stub 0216bbb4 ret=u32 args=v
  stub 020891e0 force ret=none args=outw:12:0/1/2,out:12,v,v
  stub 0203f524 force ret=u64 args=
  stub 0200c0f0 force ret=u32 args=v
  stub 0200c37c force ret=u64 args=v
  stub 0200e458 force ret=pick:0/1/2/5/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/2/999999 args=v,v

# challenge_state(save, u16 *out): T = 0200E394(save); c = 0200C32C(0200C0E8(save)); kind 0: *out = 0216B994(T, c) ? 1 : 2;
# kind 1: 0216B9D8(save) == 0 -> 3, else 0216BA08(save) ? 1 : 4; else nothing.
fn 0216b8b8 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  stub 0200c0e8 force ret=u32 args=v
  stub 0216ba08 ret=pick:0/1 args=v
  stub 0200c32c force ret=pick:0/1/2/11/12/13/23/24/34/35/46 args=v
  stub 0216bb70 ret=pick:0/1/2 args=v
  stub 0216b994 ret=pick:0/1 args=v,v
  stub 0200e394 force ret=u32 args=v
  stub 0216b9d8 ret=pick:0/1 args=v
  # closure: 0216ba08 / 0216bb70 / 0216b994 / 0216b9d8
  stub 0216bb80 ret=u32 args=v
  stub 0200c34c force ret=pick:0/1/1 args=v,v
  stub 0216bb90 ret=none args=v,outm:3:0xFF/0xFF/0xFF
  stub 0216b970 ret=u32 args=v,v
  stub 0216bbb4 ret=u32 args=v
  stub 0216bbc4 ret=u32 args=v
  stub 0216bb18 ret=u16 args=v
  stub 0200e458 force ret=pick:0/1/2/5/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/2/999999 args=v,v
  stub 020891e0 force ret=none args=outw:12:0/1/2,out:12,v,v
  stub 0203f524 force ret=u64 args=
  stub 0200c0f0 force ret=u32 args=v
  stub 0200c37c force ret=u64 args=v

# challenge_state_of(save, c, u16 *out): T; 0200C0E8(save); kind 0: *out = 0216B994(T, c) ? 1 : 2; kind 1: 0216BA40(T, c) ? 1 :
# 3; else nothing.
fn 0216b91c ret=none inputs=200
  nowork
  arg u32 any
  arg u32 oneof 0 1 2 11 12 13 23 24 34 35 46
  arg buf 0x10
  stub 0216ba40 ret=pick:0/1 args=v,v
  stub 0200c0e8 force ret=u32 args=v
  stub 0216bb70 ret=pick:0/1/2 args=v
  stub 0216b994 ret=pick:0/1 args=v,v
  stub 0200e394 force ret=u32 args=v
  # closure: 0216ba40 / 0216bb70 / 0216b994
  stub 0216bb90 ret=none args=v,outm:3:0xFF/0xFF/0xFF
  stub 0216b970 ret=u32 args=v,v
  stub 0216bbb4 ret=u32 args=v
  stub 0216bb80 ret=u32 args=v
  stub 0200e458 force ret=pick:0/1/2/5/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/2/999999 args=v,v

# record_sum(T, id): min(0200E458(T, id) + 0200E47C(T, id), 999999).
fn 0216b970 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 0200e458 force ret=pick:0/1/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/500000/999999 args=v,v

# challenge_met(T, c): ids = 0216BB90(c) (3 bytes), n = 0216BB80(c), goal = 0216BBB4(c); every ids[i] (i < n): record_sum(T,
# ids[i]) >= goal -> 1; else 0.
fn 0216b994 ret=bool inputs=200
  nowork
  arg u32 any
  arg u32 oneof 0 1 2 11 12 13 23 24 34 35 46
  stub 0216bb90 ret=none args=v,outm:3:0xFF/0xFF/0xFF
  stub 0216b970 ret=pick:0/1/2/999998/999999/0xF694E args=v,v
  stub 0216bbb4 ret=pick:0/1/2/999999 args=v
  stub 0216bb80 ret=pick:0/1/2/3 args=v
  # closure: 0216bb90 / 0216b970 / 0216bbb4 / 0216bb80
  stub 0200e458 force ret=pick:0/1/2/5/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/2/999999 args=v,v

# challenge_time_ok(save): T; c; d = 0216BB18(save) (elapsed days); 0216BBC4(c) (the record's limit) <= d -> 1 (signed); else 0.
fn 0216b9d8 ret=bool inputs=200
  nowork
  arg u32 any
  stub 0216bbc4 ret=pick:0/1/2/0x18 args=v
  stub 0200c0e8 force ret=u32 args=v
  stub 0200c32c force ret=pick:0/1/2/11/12/13/23/24/34/35/46 args=v
  stub 0200e394 force ret=u32 args=v
  stub 0216bb18 ret=pick:0/1/2/0x18 args=v
  # closure: 0216bbc4 / 0216bb18
  stub 020891e0 force ret=none args=outw:12:0/1/2,out:12,v,v
  stub 0203f524 force ret=u64 args=
  stub 0200c0f0 force ret=u32 args=v
  stub 0200c37c force ret=u64 args=v

# challenge_flags(S): n = 0216BB80(0200C32C(S)); every 0200C34C(S, i) (i < n) nonzero -> 1; else 0.
fn 0216ba08 ret=bool inputs=200
  nowork
  arg u32 any
  stub 0200c0e8 force ret=u32 args=v
  stub 0200c32c force ret=pick:0/1/2/11/12/13/23/24/34/35/46 args=v
  stub 0216bb80 ret=pick:0/1/2/3 args=v
  stub 0200c34c force ret=pick:0/1/1 args=v,v

# challenge_all_max(T, c): ids, n; every record_sum(T, ids[i]) >= 999999 -> 1; else 0.
fn 0216ba40 ret=bool inputs=200
  nowork
  arg u32 any
  arg u32 oneof 0 1 2 11 12 13 23 24 34 35 46
  stub 0216bb90 ret=none args=v,outm:3:0xFF/0xFF/0xFF
  stub 0216b970 ret=pick:0/1/2/999998/999999/0xF694E args=v,v
  stub 0216bbb4 ret=u32 args=v
  stub 0216bb80 ret=pick:0/1/2/3 args=v
  # closure: 0216bb90 / 0216b970 / 0216bbb4 / 0216bb80
  stub 0200e458 force ret=pick:0/1/2/5/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/2/999999 args=v,v

# challenge_left(save): T; S; c; goal = 0216BBB4(c); n; ids; m = min over ids[i] of record_sum(T, ids[i]) (start 0xF694E);
# (u16)(goal - m).
fn 0216ba88 ret=u16 inputs=200
  nowork
  arg u32 any
  stub 0216bb80 ret=pick:0/1/2/3 args=v
  stub 0200c0e8 force ret=u32 args=v
  stub 0200c32c force ret=pick:0/1/2/11/12/13/23/24/34/35/46 args=v
  stub 0216bb90 ret=none args=v,outm:3:0xFF/0xFF/0xFF
  stub 0216b970 ret=pick:0/1/2/999998/999999/0xF694E args=v,v
  stub 0216bbb4 ret=pick:0/1/2/999999 args=v
  stub 0200e394 force ret=u32 args=v
  # closure: 0216bb80 / 0216bb90 / 0216b970 / 0216bbb4
  stub 0200e458 force ret=pick:0/1/2/5/500000/999999/0xFFFFFFFF args=v,v
  stub 0200e47c force ret=pick:0/1/2/999999 args=v,v

# challenge_days_left(save): T; c; d = 0216BB18(save); max(0216BBC4(c) - d, 0) (u16).
fn 0216bae8 ret=u16 inputs=200
  nowork
  arg u32 any
  stub 0216bbc4 ret=pick:0/1/2/0x18 args=v
  stub 0200c0e8 force ret=u32 args=v
  stub 0200c32c force ret=pick:0/1/2/11/12/13/23/24/34/35/46 args=v
  stub 0200e394 force ret=u32 args=v
  stub 0216bb18 ret=pick:0/1/2/0x18 args=v
  # closure: 0216bbc4 / 0216bb18
  stub 020891e0 force ret=none args=outw:12:0/1/2,out:12,v,v
  stub 0203f524 force ret=u64 args=
  stub 0200c0f0 force ret=u32 args=v
  stub 0200c37c force ret=u64 args=v

# challenge_days(save): S = 0200C0F0(save); 0200C32C(S) == 0 -> 0; else t0 = 0200C37C(S) (64-bit start), now = 0203F524() (64-bit
# clock); 020891E0(&date, &time, now - t0) (seconds -> date + time); date.y != 0 or date.m > 1 or date.d > 1 -> 0x18; else
# (u16) time word 0 (hours).
fn 0216bb18 ret=u16 inputs=400
  nowork
  arg u32 any
  stub 020891e0 force ret=none args=outw:12:0/1/2,out:12,v,v
  stub 0203f524 force ret=u64 args=
  stub 0200c32c force ret=pick:0/1/2 args=v
  stub 0200c0f0 force ret=u32 args=v
  stub 0200c37c force ret=u64 args=v

# chal_kind(c): byte 0 of record c (table 0x021723BC, 7-byte records).
fn 0216bb70 ret=u8 inputs=200
  nowork
  arg u32 range 0 46

# chal_count(c): byte 1 of record c.
fn 0216bb80 ret=u8 inputs=200
  nowork
  arg u32 range 0 46

# chal_ids(c, u8 out[3]): bytes 2..4 of record c.
fn 0216bb90 ret=none inputs=200
  nowork
  arg u32 range 0 46
  arg buf 0x10

# chal_a(c): byte 5 of record c.
fn 0216bbb4 ret=u8 inputs=200
  nowork
  arg u32 range 0 46

# chal_b(c): byte 6 of record c.
fn 0216bbc4 ret=u8 inputs=200
  nowork
  arg u32 range 0 46

# cmd 545: fm = 0201210C(GAMESYS); k, x, y, z, w, f = values; P = 02188CB4(fm); 021A3038(P, &pos) (player position, 12 bytes);
# t = {x << 16 + 0x8000, y << 15, z << 16 + 0x8000}; k == 0 -> ev = 021F39B8(GAMESYS, 0, &pos, &t, w << 15, f); else
# ev = 021F39B8(GAMESYS, k, &t, &pos, w << 15, f) (from / to swapped); ev -> child event; 1.
fn 0216bbd4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f39b8 force ret=nullable:2:u32 args=v,v,in:12,in:12,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/0x10/0xFFFF args=v,v seq
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021a3038 force ret=none args=v,out:12
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 546: k, x, y, z = values; t = {x << 16 + 0x8000, y << 15, z << 16 + 0x8000}; 021F3AA0(GAMESYS, k, &t); 0.
fn 0216bc88 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5dc force ret=u32 args=v
  stub 021f3aa0 force ret=none args=v,v,in:12

# cmd 547: k; ev = 021F3B68(GAMESYS, k); ev -> child event; 1.
fn 0216bce4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3b68 force ret=nullable:2:u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 548: k; ev = 021F3B40(GAMESYS, k); ev -> child event; 1.
fn 0216bd1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3b40 force ret=nullable:2:u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v

# cmd 544: party; species, f = values; A, B = vars; (k, m) = 0xF3..0xF5 (2, 0), 0xFB (0, 0), 0x1EC (7, 0), 0x287 (4, 0x224), 0x288
# (5, 0x223), 0x289 (6, 0); f == 0 -> m = 0; *B = 0216BE20(party, species, m, k, A); other species (0xF6..0xFA too): *B = 0; 0.
fn 0216bd54 ret=u32 inputs=600
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216be20 ret=u16 args=v,v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=nth:pick:0xF2/0xF3/0xF4/0xF5/0xF6/0xF8/0xFA/0xFB/0xFC/0x1EC/0x286/0x287/0x288/0x289/0x28A/0;pick:0/1 args=v,v seq
  stub 02012934 force ret=u32 args=v
  # closure: 0216be20
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  stub 0202f488 force ret=u32 args=v,v

# find_mon(party, species, move, k, u16 *slot): *slot = 0; for i < 0201AA18(party): mon i passes 0202F488(mon, k) and its species
# (5) == species and (move == 0 or none of moves 0x36..0x39 == move) -> *slot = i, 1; none -> 0.
fn 0216be20 ret=u32 inputs=600
  nowork
  arg u32 any
  arg u32 oneof 0 1 0x287
  arg u32 oneof 0 1 0x224
  arg u32 range 0 7
  arg buf 0x10
  stub 0201aa18 force ret=pick:0/1/2/6 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02017e1c force ret=pick:0/1/0x287/0x224 args=v,v,v
  stub 0202f488 force ret=pick:0/1/1 args=v,v

# cmd 552: party; species, slot = values; species 0xF3..0xF5 -> k = 2, 0xFB -> k = 0, others (0xF6..0xFA too) -> nothing; mon =
# 0201AB28(party, slot); 0202F488(mon, k) != 0 -> 0202F630(mon, k); 0.
fn 0216bec0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 0202f488 force ret=pick:0/1 args=v,v
  stub 02159b10 force ret=nth:pick:0xF2/0xF3/0xF5/0xF6/0xF9/0xFB/0xFC/0;u16 args=v,v seq
  stub 0202f630 force ret=none args=v,v
  stub 02012934 force ret=u32 args=v

# cmd 445: D = 0200D418(save); i = value (u8); var; *var = 0200D468(D, i); 0.
fn 0216bf38 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200d468 force ret=u32 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200d418 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# set_key(S, i, mystatus): i < 8 -> 0200C38C(S, i, 02008530(mystatus) ^ table 0x02172508[i]) (8 u32 keys); else nothing.
fn 0216bf74 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 alt range 0 8 || range 9 0x7FFFFFFF
  arg u32 any
  stub 02008530 force ret=u32 args=v
  stub 0200c38c force ret=u32 args=v,v,v

# cmd 572: 0216CC40(GAMESYS); 0.
fn 0216cc30 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216cc40 ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  # closure: 0216cc40
  stub 0200b3a4 force ret=ptr:arg1:0x500:1 args=v,v
  stub 02012ec4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v

# namein_flag(GAMESYS): *0200B3A4(02012EC4(GAMEDATA), 0x34) = 1 (u32).  Arena: arg 1.
fn 0216cc40 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  stub 0200b3a4 force ret=ptr:arg1:0:1 args=v,v
  stub 02012ec4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v

# cmd 590: 0216F900(GAMESYS); 0.
fn 0216f8b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216f900 ret=none args=v
  stub 0215a5dc force ret=u32 args=v
  # closure: 0216f900
  stub 0200b3a4 force ret=ptr:arg1:0x500:1 args=v,v
  stub 02012ec4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v

# cmd 591: 021F385C(0201210C(GAMESYS)) (trcard overlay); 0.
fn 0216f8c8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5dc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 021f385c force ret=none args=v

# cmd 592: ev = 021F38BC(GAMESYS) (trcard overlay); 0215897C(SW, ev) (child event); 1.
fn 0216f8dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215897c force ret=none args=v,v
  stub 021f38bc force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# trcard_flag(GAMESYS): *0200B3A4(02012EC4(GAMEDATA), 0x36) = 1 (u32).  Arena: arg 1.
fn 0216f900 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  stub 0200b3a4 force ret=ptr:arg1:0:1 args=v,v
  stub 02012ec4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
