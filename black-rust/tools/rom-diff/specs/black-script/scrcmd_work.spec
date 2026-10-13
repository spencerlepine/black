# fndiff spec: scrcmd_work.c (F8-spec1), overlay 10 Thumb code 0x0215A514..0x0215C63A (F1-SPLIT row 10.14; the debug string
# "scrcmd_work.c" is the file argument of the two allocations 0215a514 / 0215a55c).  Grammar: svflow_specs/README.md (generic
# mode).  Run: tools/check-fn.sh black-script scrcmd_work --per-function
# Read from the disassembly (rizin on the field-set ELF; Ghidra's field set drops the arguments of every overlay call).
#
# Command ids of this area (id  handler; table 0x0217062C, index = id; 140 / 378 share one body at two addresses):
#  140 0215b3ac  141 0215b3dc  149 0215b328  150 0215b350  151 0215b378  201 0215aa60  202 0215aa90  203 0215aabc
#  204 0215aadc  205 0215ab18  206 0215aaf4  207 0215ab3c  208 0215ab60  209 0215ab98  210 0215abd0  211 0215ad38
#  212 0215abec  213 0215ac3c  214 0215ac68  215 0215aca8  216 0215aeb8  217 0215ad50  218 0215ae34  219 0215ad7c
#  220 0215adcc  221 0215af98  222 0215aee8  223 0215af40  224 0215aa50  225 0215ac1c  226 0215b028  227 0215b068
#  228 0215b084  229 0215b0b4  230 0215b104  231 0215b138  232 0215b158  234 0215b190  249 0215c558  250 0215c584
#  251 0215c5b0  252 0215b704  253 0215b74c  254 0215b7e8  255 0215b824  257 0215b660  258 0215b6c0  259 0215b888
#  260 0215b5a8  264 0215bca0  266 0215bce8  267 0215bd2c  268 0215bb50  269 0215b848  270 0215bbd0  271 0215c2d0
#  272 0215c138  273 0215c0d0  274 0215b5bc  275 0215be24  276 0215b904  277 0215bab0  278 0215baf4  279 0215bf30
#  280 0215ba18  281 0215c01c  282 0215c064  283 0215b558  284 0215bf7c  289 0215b994  290 0215c180  291 0215c200
#  312 0215acc8  330 0215c5ec  331 0215c610  378 0215b408  379 0215b438  464 0215afe0  465 0215aff4  466 0215b014
#  488 0215bfd4  490 0215b214  491 0215b26c  492 0215b2c4  562 0215b464  563 0215b488
# 130 functions specced (86 command ids).  The rest of the file: SCRCMD WORK / CTX constructors and accessors, the movement-
# object slots, the choice-menu builder used by commands of other files (0215a77c / 7f0 / 864 / 954 / 990, called from overlay
# 21 0x021B0xxx), the party / PC-box helpers of the Pokemon commands (0215b4ac / 4dc / 500, 0215c40c / 4c8 / 510).
# Not specced: the 8 ctx getters 0x0215A5CC..0x0215A5F4 (script_sys.spec, ported in script_sys), 0215a554 (`ldr r3,=Heap_Free;
# bx r3`, the free veneer: callers stub 020307B0), 0215aec2 (a mid-function address of 0215aeb8 that only a stray `bl`
# pattern in main data at 0x020A482C points at; not a real entry).  The range has no other code (Thumb walk: only padding and
# literal pools uncovered).
#
# Objects:
#  SCRCMD CTX (0x24, 0215a55c): +0 u16 heap, +4..+0x13 init record {+4 u16 zone, +6 u16 script id, +8 mode, +0xC mode != 0,
#        +0x10 SW}, +0x14 message data (0215a760; freed by 0215a5a4 with 02045C04), +0x18 u16 message bank, +0x1A u8 the VM slot
#        command 28 waits for, +0x1C saved script vars (0215aa24 = script_sys save_vars 0215909C, 0215aa34 restores and frees
#        them), +0x20 -> SCRCMD WORK.
#  SCRCMD WORK (0x110, 0215a514, SW +0x148): +0 SW, +4 GAMESYS, +8 GAMEDATA, +0xC 02012964(GAMEDATA), +0x10 (0215a644 / 64c),
#        +0x14 8 movement-object slots (overlay 21 fldmmdl_acmd objects: 02197DA0 = finished?, 02197DAC = delete), +0x34 MENU
#        (0xA0 bytes, see 0215a77c), +0xD4 (0215a63c returns its address), +0xFC window / message object (0215a614 / 61c, used
#        by overlay 21 message commands), +0x100 (0215a624 / 630), +0x104 u8 (= 7; 0215a654 / 660), +0x105 u8 (0215a66c / 678),
#        +0x106 u16 frame countdown (command 3: 0215a684 / 690), +0x108 / +0x10C (0215a6ac..6d0).
#  MENU (work +0x34): +0 x, +2 y, +4 cursor (u16), +6 / +7 u8 (+7 == 1: right-aligned), +8 -> result var (u16: index,
#        0xFFFE cancel, 0xFFFD X), +0xC WORDSET, +0x10 own message data flag, +0x14 message data, +0x18 item list (02190A20),
#        +0x1C list-menu handle (overlay 21 02190988), +0x20 32 help-text STRBUFs.
#  Party helpers: 02012934(GAMEDATA) = party, 0201AA18 = count, 0201AA14 = capacity (compared signed), 0201AB28(party, i) =
#        mon, 02017E1C(mon, param, 0) / 02017E40(mon, param, v) = PokeParam get / set (black-pokemon poke_tool; param ids:
#        5 species, 7 OT id, 9 friendship, 0x0D..0x12 EVs, 0x36..0x39 moves, 0x3A.. PP-ups, 0x3E.. PP, 0x46..0x4B IVs, 0x4C
#        egg, 0x6F form, 0x73 nickname, 0x77 version, 0x8D OT name, 0x9A OT gender, 0xA0 / 0xA1 HP / max HP; names inferred).
#
# Spec conventions (as scrcmd_core.spec / script_sys.spec): every block is `nowork`.  A command's arguments are two arenas:
#   arg0 VM arena 0x800 (regs at 0x40, script bytes at 0x500, +0xE in 0..2), arg1 ctx arena 0x800 (CTX at 0, SCRCMD WORK at
#   0x100, zone A 0x300, zone B 0x400, vars 0x600..0x7FF: 02159ae8 answers `ptr:arg1:0x600:0x80`).
# A helper taking the ctx in r0 gets the ctx arena as arg0 (the MENU fields are then at 0x134..).  Functions that only
# allocate get a trailing arena as the `ptr:argK` target (an unread register).  ROM tables (0x021705DC conditions, 0x02171140
# menu header, 0x02171160 / 70 / 80 / 82 / 8E u16 / byte tables, 0x0209DF08 version byte) are read as ROM data: the port embeds
# them; operands that index them are bounded with `pick:` stubs.  The RAM word 0x020A7370 (a message-data pointer) is a
# `global`.  Closure stubs as in scrcmd_core (every function of the area treated as ported: no fault, stray write or
# unstubbed callee).  Pointer-valued callee results that the code dereferences: 02012AA0 / 020124E8 -> zone A, 02158F58 ->
# ctx zone A (helpers), 02020D0C -> the ctx arena.
area scrcmd_work
set field

# 0215a514(SW, heap): SCRCMD WORK = Heap_AllocDebug(heap, 0x110, 1, "scrcmd_work.c", 0x78): +0 SW, +4 GAMESYS
# (02158F54(SW)), +8 GAMEDATA (020120F4), +0xC 02012964(GAMEDATA), +0x104 u8 = 7; returns it.  arg2 = allocation arena.
fn 0215a514 ret=ofs:arg2 inputs=200
  nowork
  arg u32 any
  arg u32 alt range 0 0x10 || any
  arg buf 0x200
  stub 02012964 ret=u32 args=v
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v
  stub 020120f4 ret=u32 args=v
  stub 02158f54 ret=u32 args=v

# 0215a55c(init, heap): SCRCMD CTX = Heap_AllocDebug((heap & 0x7FFF) | 0x8000, 0x24, 1, "scrcmd_work.c", 0x94): +0 u16
# heap, +4..+0x13 = the 16-byte init record, +0x20 = 02158F6C(init SW +0xC) (the scrcmd work); returns it.  arg2 = arena.
fn 0215a55c ret=ofs:arg2 inputs=200
  nowork
  arg buf 0x20
  arg u32 alt range 0 0x10 || any
  arg buf 0x80
  stub 02158f6c ret=u32 args=v
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v

# 0215a5a4(ctx): free the ctx: Heap_Free(+0x1C) if set, 02045C04(+0x14) (message data) if set, 0215a738 (movement
# objects), Heap_Free(ctx).
fn 0215a5a4 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  fix arg0 0x1C u32 alt const 0 || any
  fix arg0 0x14 u32 alt const 0 || any
  fix arg0 0x114 u32 alt const 0 || any || any rep=8,4
  stub 020307b0 ret=none args=v
  stub 0215a738 ret=none args=v
  stub 02045c04 ret=none args=v
  # closure: 0215a738
  stub 02197dac ret=none args=v

# 0215a5fc(ctx): *(02158F58(SW)) (the first word of the SW +0x18 field block = 02188C54(fieldmap)).
fn 0215a5fc ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02158f58 ret=ptr:arg0:0x300:0x10 args=v
  stub 0215a5f4 ret=u32 args=v

# 0215a60c(ctx): ctx +0x14 (the script's message data, 0215a760).
fn 0215a60c ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a610(ctx): u16 ctx +0x18 (the message bank id).
fn 0215a610 ret=u16 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a614(ctx, v): scrcmd work +0xFC = v (a window / message object, 21 callers in overlay 21).
fn 0215a614 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a61c(ctx): scrcmd work +0xFC.
fn 0215a61c ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a624(ctx, v): scrcmd work +0x100 = v.
fn 0215a624 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a630(ctx): scrcmd work +0x100.
fn 0215a630 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a63c(ctx): address of scrcmd work +0xD4.
fn 0215a63c ret=ofs:arg0 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a644(ctx, v): scrcmd work +0x10 = v.
fn 0215a644 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a64c(ctx): scrcmd work +0x10.
fn 0215a64c ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a654(ctx): u8 scrcmd work +0x104 (7 after 0215a514).
fn 0215a654 ret=u8 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a660(ctx, v): u8 scrcmd work +0x104 = v.
fn 0215a660 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a66c(ctx): u8 scrcmd work +0x105.
fn 0215a66c ret=u8 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a678(ctx, v): u8 scrcmd work +0x105 = v.
fn 0215a678 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a684(ctx, n): u16 scrcmd work +0x106 = n (frame countdown of command 3).
fn 0215a684 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a690(ctx): countdown: +0x106 == 0 -> 1 (done); else decrement, 0.
fn 0215a690 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  fix arg0 0x206 u16 alt range 0 2 || any

# 0215a6ac(ctx): scrcmd work +0x108.
fn 0215a6ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a6b8(ctx, v): scrcmd work +0x108 = v.
fn 0215a6b8 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a6c4(ctx): scrcmd work +0x10C.
fn 0215a6c4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215a6d0(ctx, v): scrcmd work +0x10C = v.
fn 0215a6d0 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215a6dc(ctx, obj): store obj in the first free one of the 8 slots at scrcmd work +0x14 (movement command objects of
# overlay 21 fldmmdl_acmd; none free -> dropped).
fn 0215a6dc ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  fix arg0 0x114 u32 alt const 0 || any || any rep=8,4

# 0215a6fc(ctx): for each slot object: finished (02197DA0 == 1) -> deleted (02197DAC) and the slot cleared; returns 1
# while any is still running (the movement wait).
fn 0215a6fc ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  fix arg0 0x114 u32 alt const 0 || any || any rep=8,4
  stub 02197da0 ret=pick:0/1 args=v
  stub 02197dac ret=none args=v

# 0215a738(ctx): delete (02197DAC) every slot object and clear the slots.
fn 0215a738 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  fix arg0 0x114 u32 alt const 0 || any || any rep=8,4
  stub 02197dac ret=none args=v

# 0215a760(ctx, type, msg): ctx +0x14 = 02045B38(0, type & 0xFFFF, msg & 0xFFFF, heap +0) (MsgData_Create), +0x18 = msg
# (ScriptHost::message_bank_open; called by script_sys vm_new for ctx mode 0).
fn 0215a760 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 any
  stub 02045b38 ret=u32 args=v,v,v,v

# 0215a77c(ctx, x, y, cursor, s0, s1, result var ptr, wordset, msgdata): begin a choice menu.  Menu work M = scrcmd work
# +0x34 (0xA0 bytes, zeroed): +0 x, +2 y, +4 cursor (u16), +6 s0, +7 s1 (u8), +8 result var pointer, +0xC WORDSET,
# +0x14 message data (0 -> own one, 02045B38(0, 3, 0x106, heap), +0x10 = 1), +0x18 = 02190A20(0x20, heap) (item list),
# +0x20.. 32 help-text STRBUF slots (zeroed).  Stack args s0..s4 = sp+0x20..0x30.
fn 0215a77c ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  stub 02045b38 ret=u32 args=v,v,v,v
  stub 02190a20 ret=u32 args=v,v

# 0215a7f0(ctx, msg, help, param, s0 strbuf, s1 strbuf): add a menu item: help != 0xFFFF and fewer than 0x20 items
# (0201F91C(list)) -> help text 02045CAC(msgdata, help, s1), expanded by 0201F250(wordset, s0, s1), copied into slot
# M+0x20+n*4 (02045874(s0, heap)); then the item text msg the same way and 02190A64(list, s0, param, heap).
fn 0215a7f0 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any
  arg u32 alt const 0xFFFF || any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02190a64 ret=none args=v,v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 0201f250 ret=none args=v,v,v
  stub 02045874 ret=u32 args=v,v
  stub 0201f91c ret=pick:0/1/5/0x1F/0x20/0x21 args=v

# 0215a864(ctx): open the menu: header = the 0x20-byte template at 0x02171140 (stack copy), width 02190ACC, item count
# 02190AC4 (min 6 rows), 02190AEC, layout 02190B08; M+0x1C = 02190988(0215a5fc(ctx), header, list, cursor callback
# 0x0215A9C1, ctx, 0, cursor, s0) (the overlay-21 list menu).  M+7 == 1: x -= width.
fn 0215a864 ret=none inputs=300
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02190ac4 ret=pick:0/1/3/6/7/0x20 args=v
  stub 02190b08 ret=none args=in:0x20,v,v,v,v,v
  stub 02190988 ret=u32 args=v,in:0x20,v,v,v,v,v,v
  stub 02190acc ret=u32 args=v,v,v,v
  stub 02190aec ret=u32 args=v,v,v,v
  stub 0215a5fc ret=u32 args=v
  # closure: 0215a5fc
  stub 02158f58 ret=ptr:arg0:0x300:0x10 args=v
  stub 0215a5f4 ret=u32 args=v

# 0215a920(M): close the menu: 021909BC(M+0x1C); own message data (+0x10 == 1) -> 02045C04(+0x14); free (02045808) and
# clear the 32 help-text slots.
fn 0215a920 ret=none inputs=200
  nowork
  arg buf 0x100
  fix arg0 0x10 u32 alt const 1 || any
  fix arg0 0x20 u32 alt const 0 || any rep=32,4
  stub 02045808 ret=none args=v
  stub 021909bc ret=none args=v
  stub 02045c04 ret=none args=v

# 0215a954(ctx): poll the menu: r = 021909E4(M+0x1C); -1 -> 0 (open); else close (0215a920) and *(u16 *)(M+8) =
# r == -2 (cancel) ? 0xFFFE : r; 1.
fn 0215a954 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  fix arg0 0x13C u32 addrof arg0 0x300
  fix arg0 0x144 u32 alt const 1 || any
  fix arg0 0x154 u32 alt const 0 || any rep=32,4
  stub 0215a920 ret=none args=v
  stub 021909e4 ret=pick:0xFFFFFFFF/0xFFFFFFFE/0/1/2/5 args=v
  # closure: 0215a920
  stub 02045808 ret=none args=v
  stub 021909bc ret=none args=v
  stub 02045c04 ret=none args=v

# 0215a990(ctx): menu wait with X: 0215a954; still open and 020362DC() & 0x400 -> close, result 0xFFFD, done.
fn 0215a990 ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  fix arg0 0x13C u32 addrof arg0 0x300
  fix arg0 0x144 u32 alt const 1 || any
  fix arg0 0x154 u32 alt const 0 || any rep=32,4
  stub 0215a920 ret=none args=v
  stub 020362dc ret=pick:0/0x400/0x401/1/2 args=
  stub 0215a954 ret=pick:0/1 args=v
  # closure: 0215a920 / 0215a954
  stub 02045808 ret=none args=v
  stub 021909bc ret=none args=v
  stub 02045c04 ret=none args=v
  stub 021909e4 ret=pick:0xFFFFFFFF/0xFFFFFFFE/0/1/2/5 args=v

# 0215a9c0(menu, ...): cursor-move callback of the list menu: ctx = 02020D0C(menu, 0); idx = 020202EC(menu, &idx);
# help text slot set -> 02159198(1) ? print it in window 0215a61c(ctx) (02191164, 02191184(w, 0, 0, text)) :
# 02159198(2) || 02159198(3).  arg1 = the ctx arena 02020D0C hands back (r1 is not read).
fn 0215a9c0 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x154 u32 alt const 0 || any rep=32,4
  stub 02191164 ret=none args=v
  stub 02191184 ret=none args=v,v,v,v
  stub 02020d0c ret=ptr:arg1:0:1 args=v,v
  stub 020202ec ret=none args=v,outm:2:0x1F/0
  stub 02159198 ret=pick:0/1 args=v
  stub 0215a61c ret=u32 args=v

# 0215aa24(ctx): ctx +0x1C = 0215909C(SW) (save the script vars into a heap block).
fn 0215aa24 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 0215909c ret=u32 args=v
  stub 0215a5f4 ret=u32 args=v

# 0215aa34(ctx): 021590D4(SW, ctx +0x1C) (restore and free them); +0x1C = 0.
fn 0215aa34 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 021590d4 ret=none args=v,v
  stub 0215a5f4 ret=u32 args=v

# 0215aa48(ctx, slot): u8 ctx +0x1A = slot (the called VM of command 28).
fn 0215aa48 ret=none inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 any

# 0215aa4c(ctx): u8 ctx +0x1A.
fn 0215aa4c ret=u8 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100

# 0215aa50: command 224: var = 0x15 (21; UNVERIFIED: a constant, e.g. the game version id).
fn 0215aa50 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq

# 0215aa60: command 201: x = 0200873C(02012EBC(GAMEDATA)); 020083EC(x, toggle(020083E4(x))) (0 <-> 1, other values
# kept; UNVERIFIED: flip a config option).
fn 0215aa60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020083e4 ret=pick:0/1/2 args=v
  stub 0200873c ret=u32 args=v
  stub 020083ec ret=none args=v,v
  stub 02012ebc ret=u32 args=v

# 0215aa90: command 202: var = 020083E4(0200873C(02012EBC(GAMEDATA))) (read that option).
fn 0215aa90 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020083e4 ret=pick:0/1/2 args=v
  stub 0200873c ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012ebc ret=u32 args=v

# 0215aabc: command 203: var = 020056EC(value).
fn 0215aabc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq
  stub 020056ec ret=u16 args=v

# 0215aadc: command 204: var = 0215a610(ctx) (the message bank id).
fn 0215aadc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a610 ret=u16 args=v

# 0215aaf4: command 206: var = 0216F830(GAMEDATA).
fn 0215aaf4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216f830 ret=u16 args=v
  stub 0215a5e4 ret=u32 args=v

# 0215ab18: command 205: var = 021690B4(GAMEDATA).
fn 0215ab18 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021690b4 ret=u16 args=v
  stub 0215a5e4 ret=u32 args=v

# 0215ab3c: command 207: var = 021690F0(GAMEDATA).
fn 0215ab3c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5e4 ret=u32 args=v
  stub 021690f0 ret=u16 args=v

# 0215ab60: command 208: var1 = 021690D0(GAMEDATA), var2 = 021690E0(GAMEDATA).
fn 0215ab60 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5e4 ret=u32 args=v
  stub 021690e0 ret=u16 args=v
  stub 021690d0 ret=u16 args=v

# 0215ab98: command 209: var1 = 02169100(GAMEDATA), var2 = 02169110(GAMEDATA).
fn 0215ab98 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02169100 ret=u16 args=v
  stub 02169110 ret=u16 args=v
  stub 0215a5e4 ret=u32 args=v

# 0215abd0: command 210: var = 02012984(GAMEDATA).
fn 0215abd0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012984 ret=u16 args=v
  stub 0215a5e4 ret=u32 args=v

# 0215abec: command 212: 02087D28(&t) (a 0x58-byte stack record; UNVERIFIED: RTC time); var1 = t[2], var2 = t[3].
fn 0215abec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02087d28 ret=none args=out:8

# 0215ac1c: command 225: var = 02008550(020127A4(GAMEDATA) + 0x20).
fn 0215ac1c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020127a4 ret=u32 args=v
  stub 02008550 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v

# 0215ac3c: command 213: var = 0200C234(02012F08(GAMEDATA), value).
fn 0215ac3c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f08 ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0200c234 ret=u16 args=v,v

# 0215ac68: command 214: v = value; x = 02012F08(GAMEDATA); 0200C248(x, v); 0203F4E4(&t) (12 bytes); 02008A7C(
# 02012F5C(GAMEDATA), v, t[0], t[1], t[2]) (UNVERIFIED: record v with a timestamp).
fn 0215ac68 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02008a7c ret=none args=v,v,v,v,v
  stub 0203f4e4 ret=none args=out:12
  stub 0215a5e4 ret=u32 args=v
  stub 02012f08 ret=u32 args=v
  stub 0200c248 ret=none args=v,v
  stub 02159b10 ret=u16 args=v,v seq
  stub 02012f5c ret=u32 args=v

# 0215aca8: command 215: var = 0200C258(02012F08(GAMEDATA)).
fn 0215aca8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f08 ret=u32 args=v
  stub 0200c258 ret=u16 args=v
  stub 0215a5e4 ret=u32 args=v

# 0215acc8: command 312: x = 02012EBC(GAMEDATA); var1 = 020071EC(x), var2 = 020071E4(x); 0200723C(x, &a, &b);
# var3 = (a * 3 >= b) (unsigned).
fn 0215acc8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020071e4 ret=u16 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020071ec ret=u16 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200723c ret=none args=v,out:4,out:4

# 0215ad38: command 211: var = zone id (0215a5d0(ctx)).
fn 0215ad38 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5d0 ret=u16 args=v

# 0215ad50: command 217: v = value; 02159F84(v) == 1 -> 02012900(GAMEDATA, v).
fn 0215ad50 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02159f84 ret=pick:0/1 args=v
  stub 02012900 ret=none args=v,v
  stub 0215a5e4 ret=u32 args=v

# 0215ad7c: command 219: l = 020127A4(GAMEDATA); p = 02012AA0(l) (3 words); 02014184(&rec, zone 02012AC8(l), (s16)
# 02012AE4(l), p[0], p[1], p[2]); 0201289C(GAMEDATA, &rec) (UNVERIFIED: save the current location as a warp point).
fn 0215ad7c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012aa0 ret=ptr:arg1:0x300:0x10 args=v
  stub 020127a4 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02012ae4 ret=u16 args=v
  stub 02014184 ret=none args=out:0x1C,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0201289c ret=none args=v,in:0x1C

# 0215adcc: command 220: 02014184(&rec, v0, (s16)v1, v2 << 16, v3 << 16, v4 << 16) from five value operands;
# 0201289C(GAMEDATA, &rec) (UNVERIFIED: set that warp point to a given location).
fn 0215adcc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02014184 ret=none args=out:0x1C,v,v,v,v,v
  stub 0201289c ret=none args=v,in:0x1C
  stub 0215a5e4 ret=u32 args=v

# 0215ae34: command 218: a, b, c = values; 02159F28(GAMEDATA, a, b); c != 0 -> x = 02012974(GAMEDATA), zone =
# 02012AC8(020127A4(GAMEDATA)), 02159B94(x, 02013B44(zone), zone, heap), 02159CC4(x, GAMESYS, heap) (map_matrix side, F2a).
fn 0215ae34 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020127a4 ret=u32 args=v
  stub 02013b44 ret=u32 args=v
  stub 02159cc4 ret=none args=v,v,v
  stub 02159f28 ret=none args=v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 02012974 ret=u32 args=v
  stub 02159b94 ret=none args=v,v,v,v
  stub 0215a5dc ret=u32 args=v

# 0215aeb8: command 216: var = 02159F54(GAMEDATA, value).
fn 0215aeb8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02159f54 ret=u16 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq

# 0215aee8: command 222: x = 0200CA1C(GAMEDATA); a, b = values; mon = 020173F0(a, 1, 0, -1, heap); b == 0 ->
# 0200CFA0(x, mon); Heap_Free(mon) (UNVERIFIED: register species a in the Pokedex).
fn 0215aee8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200cfa0 ret=none args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/0x1ED args=v,v seq
  stub 020173f0 ret=u32 args=v,v,v,v,v
  stub 020307b0 ret=none args=v
  stub 0200ca1c ret=u32 args=v

# 0215af40: command 223: x = 0200CA1C(GAMEDATA); mode, v = values; var = mode 0: 0200D064(x, v), 1: 0200CED4(x, v),
# else 0 (UNVERIFIED: Pokedex seen / caught check).
fn 0215af40 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 0200d064 ret=u16 args=v,v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/0x1ED args=v,v seq
  stub 0200ced4 ret=u16 args=v,v
  stub 0200ca1c ret=u32 args=v

# 0215af98: command 221: x = 0200CA1C(GAMEDATA); mode = value; var = mode 0: 0200D0CC(x), 1: 0200CF28(x), else 0.
fn 0215af98 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 0200cf28 ret=u16 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200d0cc ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/0x1ED args=v,v seq
  stub 0200ca1c ret=u32 args=v

# 0215afe0: command 464: 0200CA28(0200CA1C(GAMEDATA)).
fn 0215afe0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200ca28 ret=none args=v
  stub 0215a5e4 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v

# 0215aff4: command 465: var = 0200CA38(0200CA1C(GAMEDATA)).
fn 0215aff4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200ca38 ret=u16 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v

# 0215b014: command 466: 0200CA40(0200CA1C(GAMEDATA)).
fn 0215b014 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200ca40 ret=none args=v
  stub 0215a5e4 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v

# 0215b028: command 226: r = 02008428(0200873C(02012EBC(020120F4(GAMESYS)))); r 0 / 1 -> var = r (else unchanged).
fn 0215b028 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200873c ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02008428 ret=pick:0/1/2 args=v
  stub 020120f4 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  stub 02012ebc ret=u32 args=v

# 0215b068: command 227: set event flag 0x963 (02014314).
fn 0215b068 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201296c ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02014314 ret=none args=v,v

# 0215b084: command 228: v = value; v in 1..2 -> 02007A68(0201293C(GAMEDATA), v).
fn 0215b084 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=pick:0/1/2/3/0xFFFF args=v,v seq
  stub 02007a68 ret=none args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 0201293c ret=u32 args=v

# 0215b0b4: command 229: x = 020089C0(02012EBC(GAMEDATA)); var2 = 02025C88(02008554(020127A4(GAMEDATA) + 0x20));
# var1 = 020089F4(x); 02008A00(x).
fn 0215b0b4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020089c0 ret=u32 args=v
  stub 02008a00 ret=none args=v
  stub 020127a4 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02025c88 ret=u16 args=v
  stub 02008554 ret=u32 args=v
  stub 020089f4 ret=u16 args=v
  stub 02012ebc ret=u32 args=v

# 0215b104: command 230: var = 0200C308(02012F08(GAMEDATA), value) & 1.
fn 0215b104 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02012f08 ret=u32 args=v
  stub 0200c308 ret=pick:0/1/2/3 args=v,v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 0215b138: command 231: 0200C2F0(02012F08(GAMEDATA), value, 1).
fn 0215b138 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02012f08 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 0200c2f0 ret=none args=v,v,v

# 0215b158: command 232: var = (0200C308(02012F08(GAMEDATA), value) == 3).
fn 0215b158 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02012f08 ret=u32 args=v
  stub 0200c308 ret=pick:0/1/2/3 args=v,v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=u16 args=v,v seq

# 0215b190: command 234: flag 0x97B clear (020142E8) -> var = 0; else x = 02012EBC(GAMEDATA); r = 0200725C(x, 8, 4):
# 0 -> var 0, 1/2 -> var = (0200ECB4(020073D8(x, 8, 0)) != 0), other -> var 2; then 020072A4(x, 8).
fn 0215b190 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 020072a4 ret=none args=v,v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201296c ret=u32 args=v
  stub 0200ecb4 ret=pick:0/1 args=v
  stub 020073d8 ret=u32 args=v,v,v
  stub 02012ebc ret=u32 args=v
  stub 0200725c ret=pick:0/1/2/3 args=v,v,v

# 0215b214: command 490: 02162A74(02012844(GAMEDATA), v0, (u16)v1, (u16)v2, (u16)v3) (four value operands).
fn 0215b214 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02162a74 ret=none args=v,v,v,v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012844 ret=u32 args=v

# 0215b26c: command 491: 02162708(02012844(GAMEDATA), v0, (u16)v1, (u16)v2, (u16)v3).
fn 0215b26c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02162708 ret=none args=v,v,v,v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012844 ret=u32 args=v

# 0215b2c4: command 492: 02162748(02012844(GAMEDATA), v0, v1, (u16)v2, v3 << 16, (u16)v4) (the 5th word keeps v3 << 16).
fn 0215b2c4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=u16 args=v,v seq
  stub 02162748 ret=none args=v,v,v,v,v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012844 ret=u32 args=v

# 0215b328: command 149: 0215A4EC(event work, value).
fn 0215b328 ret=u32 inputs=200
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
  stub 0215a4ec ret=none args=v,v

# 0215b350: command 150: 0215A500(event work, value).
fn 0215b350 ret=u32 inputs=200
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
  stub 0215a500 ret=none args=v,v

# 0215b378: command 151: var = 0215A4D8(event work, value).
fn 0215b378 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201296c ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215a4d8 ret=u16 args=v,v

# 0215b3ac: command 140: 0215898C(SW, 0215F6E8(GAMESYS)) (the script event's end callback), stop the VM, yield.
# Command 378 (0215b408) is the same code.
fn 0215b3ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215f6e8 ret=u32 args=v
  stub 0215898c ret=none args=v,v
  stub 02011290 ret=none args=v
  stub 0215a5f4 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v

# 0215b3dc: command 141: var = (0216EFA8(02012A44(GAMEDATA), 1) == 1) ? 0 : 1.
fn 0215b3dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5e4 ret=u32 args=v
  stub 0216efa8 ret=pick:0/1/2 args=v,v
  stub 02012a44 ret=u32 args=v

# 0215b408: command 378: same as command 140.
fn 0215b408 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215f6e8 ret=u32 args=v
  stub 0215898c ret=none args=v,v
  stub 02011290 ret=none args=v
  stub 0215a5f4 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v

# 0215b438: command 379: var = (0216EFA8(02012A44(GAMEDATA), 0) == 1) ? 0 : 1.
fn 0215b438 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5e4 ret=u32 args=v
  stub 0216efa8 ret=pick:0/1/2 args=v,v
  stub 02012a44 ret=u32 args=v

# 0215b464: command 562: var = TABLE[value] (8 u16 at 0x02171160).
fn 0215b464 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=pick:0/1/2/3/4/5/6/7 args=v,v seq
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq

# 0215b488: command 563: var = TABLE[value] (8 u16 at 0x02171170).
fn 0215b488 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=pick:0/1/2/3/4/5/6/7 args=v,v seq
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq

# 0215b4ac(ctx, idx, &mon): party = 02012934(GAMEDATA); ok = idx < 0201AA18(party) (count; else idx = 0); *mon =
# 0201AB28(party, idx); returns ok.  arg2 = the 4-byte out word.
fn 0215b4ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 alt range 0 7 || any
  arg buf 0x10
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b4dc(ctx, idx, param): 0215b4ac(ctx, idx, &mon) == 1 ? 02017E1C(mon, param, 0) (PokeParam get) : 0.
fn 0215b4dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  fix arg0 0x20 u32 addrof arg0 0x100
  arg u32 alt range 0 7 || any
  arg u32 alt range 0 0xB0 || any
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=u32 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b500(mon, move): egg (02017E1C(mon, 0x4C)) -> 0; move in params 0x36..0x39 (the 4 moves) -> 1; else 0.
fn 0215b500 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 alt oneof 0 1 2 0x21 || any
  stub 02017e1c ret=pick:0/1/2/0x21 args=v,v,v

# 0215b558: command 283: party mon idx: var1 = param 0xAE, var2 = param 0xAF (both 0 without a mon).
fn 0215b558 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b5a8: command 260: 0201BA34(party) (UNVERIFIED: heal the party).
fn 0215b5a8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 0201ba34 ret=none args=v
  stub 02012934 ret=u32 args=v

# 0215b5bc: command 274: var = (u16) sum of params 0x0D..0x12 (EVs) of party mon idx (0 without one).
fn 0215b5bc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b660: command 257: var = 0; party mon idx: param 0xA0 == param 0xA1 (HP == max HP) or egg (0x4C == 1) -> 1.
fn 0215b660 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b6c0: command 258: var = party mon idx is an egg (param 0x4C != 0).
fn 0215b6c0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b704: command 252: var = 0; party mon idx, not an egg: var = param 9 (friendship).
fn 0215b704 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b74c: command 253: idx, v = values, op = u16 literal; party mon, not an egg: f = param 9 (u8); op 0: f = min(v,
# 255), 1: f = min(f + v, 255), 2: f = f >= v ? f - v : 0, else f; 02017E40(mon, 9, f) (set friendship).
fn 0215b74c ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02017e40 ret=none args=v,v,v
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02159b10 ret=pick:0/1/2/0x80/0xFF/0x100/0xFFFF args=v,v seq
  stub 02011330 ret=pick:0/1/2/3 args=v seq
  stub 02017e1c ret=pick:0/1/0x80/0xFE/0xFF args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b7e8: command 254: var = egg (0215b4dc(.., 0x4C)) ? 0x28A : species (0215b4dc(.., 5)).
fn 0215b7e8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4dc ret=pick:0/1/0x1ED args=v,v,v
  # closure: 0215b4dc
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b824: command 255: var = 0215b4dc(ctx, idx, 0x6F) (form).
fn 0215b824 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4dc ret=u32 args=v,v,v
  # closure: 0215b4dc
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215b848: command 269: v = value; var = v 1: 0201AB18(party), 2: (u16)0201AB08(party), else 0.
fn 0215b848 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab08 ret=u32 args=v
  stub 02159b10 ret=pick:0/1/2/3 args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0201ab18 ret=u16 args=v

# 0215b888: command 259: party counts: v 0: 0201AA18, 1: 0201AA1C, 2: 0201AA50, 3: 0201AA88, 4: 0201AAD4, 5: max(0,
# 0201AA14 - 0201AA18) (signed), else 0.  Jump table at 0x0215B8BA.
fn 0215b888 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201aa88 ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/3/4/5/6 args=v,v seq
  stub 0201aa50 ret=u16 args=v
  stub 0201aa14 ret=pick:0/1/3/6/0xFFFFFFFF args=v
  stub 02012934 ret=u32 args=v
  stub 0201aad4 ret=u16 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201aa1c ret=u16 args=v

# 0215b904: command 276: var = number of party mons (not eggs) of species v (0 < v <= 0x289).
fn 0215b904 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v

# 0215b994: command 289: box counts of 0201293C(GAMEDATA): v 0: 02007984, 1: 020079A0, 2: 02007984 - 020079A0, 3: 0x2D0 -
# 02007984 if <= 0x2D0 (signed) else 0, other: 0.  Jump table at 0x0215B9CE.
fn 0215b994 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020079a0 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02007984 ret=pick:0/1/0x2CF/0x2D0/0x2D1/0xFFFFFFFF args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/3/4/5/6 args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0201293c ret=u32 args=v

# 0215ba18: command 280: var1 = 1, var2 = index of the first party mon (not an egg) of species v; none -> var1 = 0.
fn 0215ba18 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v

# 0215bab0: command 277: var = party mon idx knows move v (0215b500).
fn 0215bab0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b500 ret=pick:0/1 args=v,v
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  # closure: 0215b500 / 0215b4ac
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215baf4: command 278: var = first party slot whose mon knows move v (0215b500), 6 if none.
fn 0215baf4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215b500 ret=pick:0/1 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  # closure: 0215b500
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v

# 0215bb50: command 268: give a Pokemon: rec = {u16 heap, a, b, c, 0, 2, 2, 2, 4} (0x24 bytes) from three values; var =
# 0215c510(GAMEDATA, &rec) (party add; 0 = party full).  02012934 / 02012944 results unused.
fn 0215bb50 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02012944 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215c510 ret=pick:0/1 args=v,in:0x24
  stub 02012934 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  # closure: 0215c510
  stub 0200cde0 ret=none args=v,v
  stub 0201a98c ret=none args=v,v
  stub 0215c40c ret=u32 args=v,in:0x24
  stub 020307b0 ret=none args=v
  stub 0201aa14 ret=pick:0/1/5/6/7 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02017e40 ret=none args=v,v,v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215bbd0: command 270: give a Pokemon with more fields: rec = {heap, v0, v1, v2, v7, min3(v3), min3(v4), min3(v5), v8}
# (min3(x) = x >= 3 ? 2 : x, signed) from nine values; var = 0215c510(GAMEDATA, &rec).
fn 0215bbd0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02012944 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/3/4/0x100/0xFFFF args=v,v seq
  stub 0215c510 ret=pick:0/1 args=v,in:0x24
  stub 02012934 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  # closure: 0215c510
  stub 0200cde0 ret=none args=v,v
  stub 0201a98c ret=none args=v,v
  stub 0215c40c ret=u32 args=v,in:0x24
  stub 020307b0 ret=none args=v
  stub 0201aa14 ret=pick:0/1/5/6/7 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02017e40 ret=none args=v,v,v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215bca0: command 264: var = number of nonzero moves (params 0x36..0x39) of party mon idx.
fn 0215bca0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215bce8: command 266: var = move in slot s (param 0x36 + s) of party mon idx, 0 without a mon.
fn 0215bce8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215bd2c: command 267: idx, slot, move = values; party mon: k = 02017D30(mon); slot > 4: 020180F4(mon, move) (learn);
# move == 0: delete move slot (shift moves / PP / PP-ups 0x37.. 0x3F.. 0x3B.. down, clear the last); else 020181F4(mon,
# move, slot); 02017D70(mon, k).
fn 0215bd2c ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02017e40 ret=none args=v,v,v
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02159b10 ret=pick:0/1/2/3/4/5/0x21 args=v,v seq
  stub 02017d30 ret=pick:0/1 args=v
  stub 02017d70 ret=none args=v,v
  stub 020180f4 ret=none args=v,v
  stub 020181f4 ret=none args=v,v,v
  stub 02017e1c ret=pick:0/1/0x21 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215be24: command 275: var = party mon idx is the player's own: OT id (param 7) == 02008530(player), OT name (0x8D)
# == 02008500(player) (02045 strbuf compare 0204589C), OT gender (0x9A) == 02008550(player), version (0x77) == byte
# 0x0209DF08 (0x15).  STRBUF allocation failure -> 0 (the other buffer is not freed).
fn 0215be24 ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02008500 ret=none args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012944 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02045808 ret=none args=v
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 02008530 ret=pick:0/0x15 args=v
  stub 020457b0 ret=nullable:4:u32 args=v,v
  stub 0204589c ret=pick:0/1 args=v,v
  stub 02008550 ret=pick:0/0x15 args=v
  stub 02017e1c ret=pick:0/0x15/0x15/1 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 02012934 ret=u32 args=v

# 0215bf30: command 279: mon = 0201AB28(party, idx); 02017988(mon, v); 0200CFA0(0200CA1C(GAMEDATA), mon) (UNVERIFIED:
# change something (form?) and re-register in the Pokedex).
fn 0215bf30 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200cfa0 ret=none args=v,v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 02017988 ret=none args=v,v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215a5f4 ret=u32 args=v
  stub 020120f4 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0200ca1c ret=u32 args=v

# 0215bf7c: command 284: mon = 0201AB28(party, v0); 02017A20(mon, v2, v1); 0200CFA0(dex, mon).
fn 0215bf7c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02017a20 ret=none args=v,v,v
  stub 0200cfa0 ret=none args=v,v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0215a5f4 ret=u32 args=v
  stub 020120f4 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0200ca1c ret=u32 args=v

# 0215bfd4: command 488: party mon idx: x = 0216F948(mon, heap); var = 0216FAA0(x); Heap_Free(x).
fn 0215bfd4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216faa0 ret=u16 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216f948 ret=u32 args=v,v
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 020307b0 ret=none args=v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215c01c: command 281: var = ((u16)0215b4dc(ctx, idx, 0x95) == TABLE[t]) (u16 table at 0x02171180, t = value).
fn 0215c01c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1 args=v,v seq
  stub 0215b4dc ret=pick:0/0x133/0x605 args=v,v,v
  # closure: 0215b4dc
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/0x133/0x605 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215c064: command 282: var1..3 = params 0x92 / 0x93 / 0x94 of party mon idx (0 without one).
fn 0215c064 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/2/5/6/0x1ED/0x289/0xFFFF args=v,v seq
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215c0d0: command 273: idx, p, v = values; p = TABLE[i].param (6 x {u16 0x46+i, u16 31} at 0x0217118E, the IVs) and
# v <= TABLE[i].max -> 02017E40(mon, p, v), 020185C4(mon) (recalculate stats).
fn 0215c0d0 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 ret=pick:0/1/0x1F/0x20/0x45/0x46/0x48/0x4B/0x4C args=v,v seq
  stub 020185c4 ret=none args=v
  stub 02017e40 ret=none args=v,v,v
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  # closure: 0215b4ac
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215c138: command 272: var = v in the 12-byte whitelist at 0x02171182 ? 0215b4dc(ctx, idx, v) : 0.
fn 0215c138 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 ret=pick:0/1/5/0xC/0x6F/0x9E/0x9F args=v,v seq
  stub 0215b4dc ret=u32 args=v,v,v
  # closure: 0215b4dc
  stub 0215b4ac ret=pick:0/1 args=v,v,out:4
  stub 02017e1c ret=pick:0/1/2/5/0x1ED/0xFF/0x100 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=nullable:5:u32 args=v,v
  stub 0215a5e4 ret=u32 args=v
  stub 02012934 ret=u32 args=v

# 0215c180: command 290: as command 268, but 0215c4c8 (send to a PC box; 0 = boxes full).
fn 0215c180 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02012944 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215c4c8 ret=pick:0/1 args=v,in:0x24
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  # closure: 0215c4c8
  stub 0200cde0 ret=none args=v,v
  stub 02007984 ret=pick:0/1/0x2CF/0x2D0/0x2D1/0xFFFFFFFF args=v
  stub 0215c40c ret=u32 args=v,in:0x24
  stub 020307b0 ret=none args=v
  stub 020185f4 ret=u32 args=v
  stub 020074d8 ret=none args=v,v
  stub 0201293c ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02017e40 ret=none args=v,v,v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215c200: command 291: as command 270, but 0215c4c8.
fn 0215c200 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 ret=u32 args=v
  stub 02012944 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215c4c8 ret=pick:0/1 args=v,in:0x24
  stub 0215a5cc ret=u16 args=v
  stub 02159b10 ret=pick:0/1/2/3/4/0x100/0xFFFF args=v,v seq
  stub 02012934 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  # closure: 0215c4c8
  stub 0200cde0 ret=none args=v,v
  stub 02007984 ret=pick:0/1/0x2CF/0x2D0/0x2D1/0xFFFFFFFF args=v
  stub 0215c40c ret=u32 args=v,in:0x24
  stub 020307b0 ret=none args=v
  stub 020185f4 ret=u32 args=v
  stub 020074d8 ret=none args=v,v
  stub 0201293c ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02017e40 ret=none args=v,v,v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215c2d0: command 271: give an egg: party full (0201AA14 <= 0201AA18, signed) -> var 0; else mon = 020173F0(species,
# 1, -1, -1, heap), form 0x6F, OT name 0x8D (02008510), friendship 9 = hatch cycles 0201AE1C(0201ADE4(species, form,
# heap), 0x15), egg flag 0x4C = 1, nickname 0x73 = 02045EC0(*0x020A7370, 0x28A), 020185C4, 0202F190(mon, 5, player, 0xEA63,
# heap), 0201A98C(party, mon), Heap_Free(mon); var 1.
fn 0215c2d0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  global 020a7370 4
  stub 02045808 ret=none args=v
  stub 0201a98c ret=none args=v,v
  stub 02159b10 ret=u16 args=v,v seq
  stub 02008510 ret=u32 args=v,v
  stub 0202f190 ret=none args=v,v,v,v,v
  stub 0201aa14 ret=pick:0/1/5/6/7 args=v
  stub 0201ae14 ret=none args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ae1c ret=u32 args=v,v
  stub 02017e1c ret=u32 args=v,v,v
  stub 020307b0 ret=none args=v
  stub 02012934 ret=u32 args=v
  stub 02017e40 ret=none args=v,v,v
  stub 02045ec0 ret=u32 args=v,v
  stub 02012944 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 0215a5cc ret=u16 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 0201ade4 ret=u32 args=v,v,v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020173f0 ret=u32 args=v,v,v,v,v

# 0215c40c(GAMEDATA, rec): build a Pokemon from rec (0x24 bytes {+0 u16 heap, +4 species, +8 level, +0xC, +0x10, +0x14,
# +0x18, +0x1C, +0x20 item?}): 02017430(species, level, OT id, 0, -1, 020186C8(..), 0, heap), params 0x6F / 6, ball
# 0x98 for 02020FE4(item, 0xF, heap) == 4, 0202F190(mon, 0, player, 02013E4C(zone), heap), 020185C4; returns the mon.
fn 0215c40c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  stub 02017e40 ret=none args=v,v,v
  stub 02012944 ret=u32 args=v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215c4c8(GAMEDATA, rec): box = 0201293C; 02007984(box) >= 0x2D0 (signed) -> 0; else mon = 0215c40c, 020074D8(box,
# 020185F4(mon)) (deposit), 0200CDE0(dex, mon), Heap_Free(mon), 1.
fn 0215c4c8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  stub 0200cde0 ret=none args=v,v
  stub 02007984 ret=pick:0/1/0x2CF/0x2D0/0x2D1/0xFFFFFFFF args=v
  stub 0215c40c ret=u32 args=v,in:0x24
  stub 020307b0 ret=none args=v
  stub 020185f4 ret=u32 args=v
  stub 020074d8 ret=none args=v,v
  stub 0201293c ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  # closure: 0215c40c
  stub 02017e40 ret=none args=v,v,v
  stub 02012944 ret=u32 args=v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215c510(GAMEDATA, rec): party = 02012934; 0201AA14 <= 0201AA18 (signed) -> 0; else mon = 0215c40c, 0201A98C(party,
# mon), 0200CDE0(dex, mon), Heap_Free(mon), 1.
fn 0215c510 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  stub 0200cde0 ret=none args=v,v
  stub 0201a98c ret=none args=v,v
  stub 0215c40c ret=u32 args=v,in:0x24
  stub 020307b0 ret=none args=v
  stub 02012934 ret=u32 args=v
  stub 0201aa14 ret=pick:0/1/5/6/7 args=v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0200ca1c ret=u32 args=v
  # closure: 0215c40c
  stub 02017e40 ret=none args=v,v,v
  stub 02012944 ret=u32 args=v
  stub 02020fe4 ret=pick:4/0/1 args=v,v,v
  stub 020127a4 ret=u32 args=v
  stub 020185c4 ret=none args=v
  stub 020186c8 ret=u32 args=v,v,v,v,v,v
  stub 02012ac8 ret=u16 args=v
  stub 0202142c ret=u32 args=v
  stub 02013e4c ret=u32 args=v
  stub 02008530 ret=u32 args=v
  stub 02017430 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0202f190 ret=none args=v,v,v,v,v

# 0215c558: command 249: 0200C278(02012F08(GAMEDATA), value) (020127A4 result unused).
fn 0215c558 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020127a4 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02012f08 ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0200c278 ret=none args=v,v

# 0215c584: command 250: 0200C29C(02012F08(GAMEDATA), value).
fn 0215c584 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020127a4 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02012f08 ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0200c29c ret=none args=v,v

# 0215c5b0: command 251: var = value <= 0200C274(02012F08(GAMEDATA)) (unsigned).
fn 0215c5b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020127a4 ret=u32 args=v
  stub 0215a5e4 ret=u32 args=v
  stub 02159ae8 ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f08 ret=u32 args=v
  stub 02159b10 ret=u16 args=v,v seq
  stub 0200c274 ret=pick:0/1/2/0xFFFF/0x10000 args=v

# 0215c5ec: command 330: call child event 020145CC(GAMESYS) (0215897c); yields.
fn 0215c5ec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215897c ret=none args=v,v
  stub 020145cc ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0215a5f4 ret=u32 args=v

# 0215c610: command 331: call child event 020144DC(GAMESYS, 0201210C(GAMESYS)); yields.
fn 0215c610 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c ret=u32 args=v
  stub 020144dc ret=u32 args=v,v
  stub 0215a5f4 ret=u32 args=v
  stub 0215a5dc ret=u32 args=v
  stub 0215897c ret=none args=v,v
