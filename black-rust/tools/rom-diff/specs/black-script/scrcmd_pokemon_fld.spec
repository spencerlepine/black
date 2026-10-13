# fndiff spec: overlay-21 script command file scrcmd_pokemon_fld.c (F8-spec4), Thumb code 0x021B66B8..0x021B7400 (F1-SPLIT row
# 21.51).  __FILE__ "scrcmd_pokemon_fld.c" (0x021DE270) is passed by 0x021B6708.
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_pokemon_fld --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  308 021b69b0  459 021b6ae0  460 021b6afc  461 021b6b6c  470 021b6a78  471 021b69d8  494 021b6bdc  526 021b716c
#  527 021b70d8  564 021b7364  565 021b73b8  581 021b66b8  585 021b721c  586 021b72cc
# 48 functions specced (14 command ids).
#  * 581: species 0x283 (Reshiram) of the player's OT in the party / boxes: find (021b6934 / 021b6850), swap the party mon with
#    the box one (021b6708, keeps / moves mail, reverts form 0x1EC), release (021b6904).
#  * 494 (Dream World delivery, X2 / D9 out of scope, logic specced): the record from pdw_postman.c (021B7834 / 7864 / 785C)
#    dispatched through the ROM table 0x021D9970 (4 rows of 0x18 bytes: kind, check +4, give +8, kind +0xC, words +0x10 / +0x14;
#    row 1 Pokemon 0x021B6F00.., row 2 item 0x021B6FEC.., row 3 0x021B707C..); sel 5 / 6 look for OBJ code 0x46 / record 0x46.
#  * 459 / 460 / 461: script-relative jump (0215A6D0) and a 10-byte record table (0215A6C4) -> 0215A7F0 / warp 021854B0.
#  * 470 / 471 calendar names (0200A8xx, RTC 02087C88), 526 / 527 / 585 / 586 ARC 208 tables ({key, value}, index byte
#    0x021D99D0[i]), 564 table 0x021D99E8, 565 special days (0x021D99D4 pairs, else 0x021D99DE[d % 10]), 308 proc 0x37.
# Not specced (0):
#   none
# Fragments (not functions): 0x021B6BF8 (a decoded data word inside cmd 494).
#
# Read from the disassembly of the raw OVY_21.sbin (rizin `-a arm -b 16 -m 0x02187F00`; rizin on field.elf mis-disassembles
# mixed ARM / Thumb and Ghidra's field set drops the arguments of overlay calls).  Function list: command-table targets + init-
# proc table entries (0x021705F0) + `bl` targets from the whole field set + odd code pointers + labels, walked as Thumb from every
# entry (only 2-byte padding and literal pools stay uncovered).
#
# Spec conventions (as scrcmd_core.spec / scrcmd_work.spec / scrcmd_proc.spec): every block is `nowork`.  A command handler
# `fn(vm, ctx)` gets two arenas: arg0 = the VM (`arg buf 0x800`: regs at 0x40, script bytes at 0x500, PC +0x14, compare byte +0xE
# in 0..2), arg1 = the scrcmd ctx (`arg buf 0x800`: CTX at 0, SCRCMD WORK at 0x100, zone A 0x300 (Heap_AllocDebug blocks, SW
# records), zone B 0x400, 0x500 (records a stub hands out), vars 0x600..0x7FF where `02159ae8` points).  A helper that takes the
# ctx in r0 gets the ctx arena as arg0.  A GMEVENT function `(event, seq, work)` gets `arg u32, arg buf 0x10 (seq, bounded),
# arg buf 0x40/0x100 (work)`; an init proc `(task, task + 0xC)` gets `arg buf 0x100, arg buf 0x10`; a TCB callback `(tcb, work)`
# gets `arg u32, arg buf`.  Script-operand readers (vm.c 02011330 / 0201134c, script_sys 02159ae8 / 02159b10) are `seq` stubs.
# Closure stubs as in core4 (every function of the area treated as ported: no fault, stray write or unstubbed callee).
# **Every stub outside the area carries `force`** (overlay addresses collide with battle `PORTED` entries and the callees in
# script_sys / scrcmd_work are ported; the F8 areas call each other only through forced stubs, so each is verified on its own).
# Indirect calls (`blx rN` through a fn argument) go to the fake callback 0x02EE0011 (stub 02ee0010).  UI callees are `ScriptHost`
# requests in the port (crates/black-script/src/host.rs); results that steer a branch are `pick:` stubs.  Heap_AllocDebug
# (02030734) is a `seqptr` bump allocator into an arena (blocks of a block never overlap); its 4th argument is the __FILE__ string,
# which places the file boundaries (F8-spec4: 0x021DE210 "scrcmd_shop.c", 0x021DE220 "scrcmd_proc_fld.c", 0x021DE234
# "scrcmd_encount.c", 0x021DE248 "scrcmd_fld_battle.c", 0x021DE25C "scrcmd_c03center.c", 0x021DE270 "scrcmd_pokemon_fld.c").
# Callees of other overlay-21 files, overlays 10 / 11 / 20, ARM9 main and the overlays at 0x021F3xxx (outside the field set) are
# force stubs.  An unset callee-saved register or stack word that a path reads is 0 under the Z19 entry state; the block says so.
area scrcmd_pokemon_fld
set field

# cmd 581 (the Reshiram / species 0x283 party-box swap): v, var; v 0: *var = 021b6934(GAMEDATA) >= 0; v 1: 021b6904(ctx); v 2:
# 021b6708(ctx, *var), 021b6904(ctx); else nothing; 0.
fn 021b66b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x600 u16 range 0 5 rep=0x100,2
  stub 0215a5e4 force ret=u32 args=v
  stub 021b6904 ret=none args=v
  stub 021b6708 ret=none args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 021b6934 ret=pick:0/1/0xFFFFFFFF args=v
  # closure: 021b6904 / 021b6708 / 021b6934
  stub 0201ab8c force ret=u32 args=v,v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 02012934 force ret=u32 args=v
  stub 02008f08 force ret=u32 args=v
  stub 02017988 force ret=none args=v,v
  stub 02007a18 force ret=u32 args=v,v,v
  stub 02017e1c force ret=pick:0/1/0x283 args=v,v,v
  stub 02009020 force ret=pick:0/1/0xFFFFFFFF args=v,v
  stub 0201ab28 force ret=ptr:arg1:0x500:1 args=v,v
  stub 02008eac force ret=none args=v
  stub 020307b0 force ret=none args=v
  stub 020075b0 force ret=none args=v,v,v,v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0200903c force ret=none args=v,v,v,v
  stub 0201293c force ret=u32 args=v
  stub 02017e40 force ret=none args=v,v,v
  stub 02012944 force ret=u32 args=v
  stub 020185c4 force ret=none args=v
  stub 0201ab48 force ret=none args=v,v,v
  stub 021b6850 ret=u32 args=v,v,v,v
  stub 0202136c force ret=pick:0/1 args=v
  stub 02008fec force ret=u32 args=v
  stub 020185f0 force ret=u32 args=v
  stub 0201747c force ret=u32 args=v,v
  stub 02008530 force ret=pick:0/1 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02007698 force ret=pick:0/1/0x17 args=v
  stub 020079bc force ret=pick:0/1/0x283 args=v,v,v,v,v

# swap_with_box(ctx, slot): GD; party; 021b6850(GD, 0x283, &box, &pos) == 0 -> return; h = 0215A5CC(ctx); M = Heap_AllocDebug(h,
# 0xDC, 1, "scrcmd_pokemon_fld.c", 0x4C7); copy party[slot] (0201AB28) to M (0x02082D44, real); 0202136C(get(M, 6)) == 1
# (mail item) -> mail: X = 02008FEC(GD), Y = 02008F08(h), k = 02009020(X, 0), get(M, 0xA7, Y), k >= 0 -> 0200903C(X, 0, k, Y),
# 02008EAC(Y), put(M, 0xA7, Y), put(M, 6, 0), Heap_Free(Y); get(M, 0xAB) == 0x1EC -> 02017988(M, 0), 020185C4(M); S =
# 0201747C(02007A18(B, pos, box), h); 0201AB48(party, slot, S); Heap_Free(S); 020075B0(B, pos, box, 020185F0(M)); Heap_Free(M).
fn 021b6708 ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 range 0 5
  fix arg0 0x20 u32 addrof arg0 0x100
  stub 02008f08 force ret=u32 args=v
  stub 02017988 force ret=none args=v,v
  stub 02007a18 force ret=u32 args=v,v,v
  stub 02017e1c force ret=pick:0/6/0x1EC args=v,v,v
  stub 02009020 force ret=pick:0/1/0xFFFFFFFF args=v,v
  stub 0201ab28 force ret=ptr:arg0:0x500:1 args=v,v
  stub 02008eac force ret=none args=v
  stub 020307b0 force ret=none args=v
  stub 020075b0 force ret=none args=v,v,v,v
  stub 02012934 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg0:0x300:0x40 args=v,v,v,v,v
  stub 0200903c force ret=none args=v,v,v,v
  stub 0201293c force ret=u32 args=v
  stub 02017e40 force ret=none args=v,v,v
  stub 02012944 force ret=u32 args=v
  stub 020185c4 force ret=none args=v
  stub 0201ab48 force ret=none args=v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 021b6850 ret=pick:0/1 args=v,v,outm:2:0x1F/0,outm:2:0x1F/0
  stub 0215a5e4 force ret=u32 args=v
  stub 0202136c force ret=pick:0/1 args=v
  stub 02008fec force ret=u32 args=v
  stub 020185f0 force ret=u32 args=v
  stub 0201747c force ret=u32 args=v,v
  # closure: 021b6850
  stub 02008530 force ret=pick:0/1 args=v
  stub 02007698 force ret=pick:0/1/0x17 args=v
  stub 020079bc force ret=pick:0/0/0/0/0/1/0x283 args=v,v,v,v,v

# find_in_box(GD, species, u16 *box, u16 *pos): B = 0201293C(GD), cur = 02007698(B), id = 02008530(02012944(GD)); for b < 24, s < 30:
# bx = (b + cur) % 24, 020079BC(B, bx, s, 0xA9 / 7 / 5 / 0x4C, 0) = present, OT id == id, species, not egg -> *box = bx, *pos =
# s, 1; else 0.
fn 021b6850 ret=bool inputs=100
  nowork
  arg u32 any
  arg u32 alt const 0x283 || const 5
  arg buf 0x10
  arg buf 0x10
  stub 02012944 force ret=u32 args=v
  stub 02008530 force ret=pick:0/1 args=v
  stub 02012934 force ret=u32 args=v
  stub 02007698 force ret=pick:0/1/0x17 args=v
  stub 020079bc force ret=pick:0/1/0x283 args=v,v,v,v,v
  stub 0201293c force ret=u32 args=v

# release_found(ctx): i = 021b6934(GAMEDATA); i >= 0 -> 0201AB8C(party, i, 0, 0215A5CC(ctx)) (r0 = that result, unused).
fn 021b6904 ret=none inputs=200
  nowork
  arg u32 any
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab8c force ret=u32 args=v,v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 021b6934 ret=pick:0/1/0xFFFFFFFF args=v
  stub 02012934 force ret=u32 args=v
  # closure: 021b6934
  stub 02012944 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02008530 force ret=pick:0/1 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/0x283 args=v,v,v

# find_in_party(GD): n = 0201AA18(party), id = 02008530(02012944(GD)); first i < n with get(mon, 7) == id, get(5) == 0x283,
# get(0x4C) == 0 -> i; else -1.
fn 021b6934 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 02012944 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02008530 force ret=pick:0/1 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/0x283 args=v,v,v

# cmd 308: R = 02158F58(SW); 021BB990(0x37, 02188D24(R+4), 0x021F45F0); 0.
fn 021b69b0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021bb990 force ret=u32 args=v,v,v
  stub 02188d24 force ret=u32 args=v

# cmd 471 (today's name): v; S = 0200A8A0(SAVE); X = 021690A4(GAMEDATA); 02087C88(&date) (RTC date); i = 0200A940(S, date[2],
# date[3], X); f = 0200A8F8(S, i) != 1; N = 0200A8C8(S, i); B = 020457B0(8, 0x8001) (strbuf); 02045924(B, N); 0201EE0C(L =
# 02158F74(SW), v, B, f, 1, 2); 02045808(B); 0.
fn 021b69d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200a8a0 force ret=u32 args=v
  stub 0200a940 force ret=u32 args=v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 021690a4 force ret=u32 args=v
  stub 02045924 force ret=none args=v,v
  stub 02087c88 force ret=none args=out:8
  stub 0200a8c8 force ret=u32 args=v,v
  stub 02045808 force ret=none args=v
  stub 0201ee0c force ret=none args=v,v,v,v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0200a8f8 force ret=pick:0/1 args=v,v
  stub 02012ebc force ret=u32 args=v

# cmd 470: var; *var = 0200A8AC(0200A8A0(SAVE)) (u16); 0.
fn 021b6a78 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200a8a0 force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200a8ac force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# entry(T, i): T + 10 * i.
fn 021b6aa4 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any

# (empty function; `bx lr`).
fn 021b6aac ret=none inputs=200
  nowork
  arg u32 any

# count(T): first i < 8 with T[10 i] u16 == 0xFFFF -> i; else 0.
fn 021b6ab0 ret=u32 inputs=200
  nowork
  arg buf 0x60
  fix arg0 0 u16 alt const 0xFFFF || any rep=8,10
  stub 021b6aac ret=none args=v
  stub 021b6aa4 ret=ptr:arg0:0:1 args=v,v

# cmd 459: off = u32 operand (0201134C); 0215A6D0(ctx, vm+0x14 (PC) + off); 0.
fn 021b6ae0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a6d0 force ret=none args=v,v
  stub 0201134c force ret=u32 args=v seq

# cmd 460: T = 0215A6C4(ctx); A = 02158F78(SW), B = 02158F7C(SW); x = 0215A5D0(ctx); for i < 021b6ab0(T): e = T + 10 i; x !=
# e[1] u16 -> 0215A7F0(ctx, e[0] u16, 0xFFFF, i, A, B); 0.
fn 021b6afc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u16 alt const 0xFFFF || any rep=8,10
  fix arg1 0x502 u16 alt range 0 2 || any rep=8,10
  stub 0215a6c4 force ret=ptr:arg1:0x500:1 args=v
  stub 021b6aa4 ret=ptr:arg1:0x500:8 args=v,v
  stub 0215a5d0 force ret=pick:0/1/2 args=v
  stub 021b6ab0 ret=pick:0/1/2/8 args=v
  stub 0215a7f0 force ret=none args=v,v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f78 force ret=u32 args=v
  stub 02158f7c force ret=u32 args=v
  # closure: 021b6aa4 / 021b6ab0
  stub 021b6aac ret=none args=v

# cmd 461: i; T = 0215A6C4(ctx); 0215A5D0(ctx); i < 021b6ab0(T) -> e = T + 10 i, Y = 02158F54(SW), ev = 021854B0(Y,
# 0201210C(Y), e[1] u16, &{e[2] << 16, e[3] << 16, e[4] << 16}, 1) (warp), child event; 1.
fn 021b6b6c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u16 alt const 0xFFFF || any rep=8,10
  stub 021b6aa4 ret=ptr:arg1:0x500:8 args=v,v
  stub 0215a6c4 force ret=ptr:arg1:0x500:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/7/8 args=v,v seq
  stub 021b6ab0 ret=pick:0/1/2/8 args=v
  stub 0215a5d0 force ret=u16 args=v
  stub 021854b0 force ret=u32 args=v,v,v,in:12,v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  # closure: 021b6aa4 / 021b6ab0
  stub 021b6aac ret=none args=v

# cmd 494 (Dream World delivery, X2): sel, var; h = 0215A5CC(ctx); S = 0200A300(SAVE, 0, h); R = 021B7834(S, &a, &buf)
# (pdw_postman.c); k = 021B7864(S); table 0x021D9970 row k (0x18 bytes: kind, 5 handlers) kind != k -> 0200A338(S), 0; sel 0:
# *var = k != 0; 1: 021b6d0c(k, ctx, GD, R); 2: 021b6d30(k, R, ctx); 3: 021b6d64(k, R, ctx); 4: 021b6d98(k, ctx, GD, R) == 1 ->
# 021B785C(S, a), 021b6eb0(R, k) == 1 -> 0216BF74(02012F08(GD), 1, 02012944(GD)); *var = 0; 5: 021b6de4(fm, GD); 6:
# 021b6e0c(fm); 7: 021b6dc0(k, ctx, GD, R); 0200A338(S); 0.
fn 021b6bdc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x5B0 u16 alt const 0x7FE || any
  fix arg1 0x500 u32 alt const 0x23E || range 0 0x280 || any
  fix arg1 0x5B3 u8 alt const 2 || any
  stub 0200a300 force ret=u32 args=v,v,v
  stub 02012f08 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 021b6e0c ret=none args=v
  stub 021b6d0c ret=none args=v,v,v,v
  stub 02159b10 force ret=pick:0/1/2/3/4/5/6/7/8 args=v,v seq
  stub 021b6d98 ret=pick:0/1 args=v,v,v,v
  stub 021b6eb0 ret=pick:0/1 args=v,v
  stub 021b6d30 ret=none args=v,v,v
  stub 021b7834 force ret=nullable:4:ptr:arg1:0x500:1 args=v,outm:4:0xFF/0xFF/0xFF/0xFF,p
  stub 0200a338 force ret=none args=v,v,v
  stub 02012ebc force ret=u32 args=v
  stub 021b6dc0 ret=u32 args=v,v,v,v
  stub 02012944 force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 021b785c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 021b7864 force ret=pick:0/1/2/3/4 args=v
  stub 021b6de4 ret=none args=v,v
  stub 021b6d64 ret=none args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216bf74 force ret=none args=v,v,v
  # closure: 021b6e0c / 021b6d0c / 021b6d98 / 021b6eb0 / 021b6d30 / 021b6dc0 / 021b6de4 / 021b6d64
  stub 021b6e24 ret=nullable:2:u32 args=v
  stub 021b6f00 ret=none args=v
  stub 021b6fec ret=none args=v
  stub 021b707c ret=none args=v
  stub 021b6f18 ret=none args=v
  stub 021b6ff0 ret=none args=v
  stub 021b7080 ret=none args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021b6f88 ret=none args=v
  stub 021b7038 ret=none args=v
  stub 021b709c ret=none args=v
  stub 021b6f5c ret=none args=v
  stub 021b7020 ret=none args=v
  stub 021b7098 ret=none args=v
  stub 0216d588 force ret=u16 args=v
  stub 021b6e68 ret=pick:0/5/0xFFFFFFFF args=v
  stub 021b6fe8 ret=none args=v
  stub 021b7068 ret=none args=v
  stub 021b70d4 ret=none args=v
  stub 0216d598 force ret=pick:0x46/1/2 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0216dd1c force ret=pick:1/1/1/0 args=v,outm:4:0xFF/0xFF/0xFF/0xFF,p seq
  stub 0201aa18 force ret=pick:4/5/6/7 args=v
  stub 02012934 force ret=u32 args=v
  stub 021b6ee0 ret=u32 args=v,v
  stub 0200cde0 force ret=none args=v,v
  stub 0201a98c force ret=none args=v,v
  stub 020307b0 force ret=none args=v
  stub 0200ca1c force ret=u32 args=v
  stub 02007e50 force ret=u32 args=v,v,v,v
  stub 0201292c force ret=u32 args=v
  stub 0200be48 force ret=none args=v
  stub 0200bf18 force ret=u32 args=v,v
  stub 0201eec4 force ret=none args=v,v,v
  stub 0201f02c force ret=none args=v,v,v
  stub 0201eed4 force ret=none args=v,v,v
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 0201ef40 force ret=none args=v,v,v
  stub 0201f184 force ret=none args=v,v,v
  stub 02162740 force ret=nullable:3:ptr:arg1:0x400:1 args=v
  stub 02012844 force ret=u32 args=v
  stub 02162744 force ret=pick:0/1/2/3/4 args=v
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v

# pdw_check(k, a, b, c): f = row k +4; c && f -> f(a, b, c); else 0 (rows 1..3: 021b6f00 / 021b6fec / 021b707c).
fn 021b6d0c ret=u32 inputs=200
  nowork
  region rec 0x100
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || addrof region:rec
  fix region:rec 0xB0 u16 alt const 0x7FE || any
  fix region:rec 0 u32 alt const 0x23E || range 0 0x280 || any
  fix region:rec 0xB3 u8 alt const 2 || any
  stub 021b6f00 ret=none args=v
  stub 021b6fec ret=none args=v
  stub 021b707c ret=none args=v
  # closure: 021b6f00 / 021b6fec / 021b707c
  stub 0201aa18 force ret=pick:4/5/6/7 args=v
  stub 02012934 force ret=u32 args=v

# pdw_name_a(k, R, ctx): L = 02158F74(0215A5F4(ctx)); f = row k +0x10; R && f -> f(L, R, ctx); else 7 (rows: 021b6f88 /
# 021b7038 / 021b709c).
fn 021b6d30 ret=u32 inputs=200
  nowork
  region rec 0x100
  arg u32 range 0 3
  arg u32 alt const 0 || addrof region:rec
  arg u32 any
  fix region:rec 0xB0 u16 alt const 0x7FE || any
  fix region:rec 0 u32 alt const 0x23E || range 0 0x280 || any
  fix region:rec 0xB3 u8 alt const 2 || any
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021b6f88 ret=none args=v
  stub 021b7038 ret=none args=v
  stub 021b709c ret=none args=v
  # closure: 021b6f88 / 021b7038 / 021b709c
  stub 021b6ee0 ret=u32 args=v,v
  stub 0201eec4 force ret=none args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0201f02c force ret=none args=v,v,v
  stub 020307b0 force ret=none args=v
  stub 0201eed4 force ret=none args=v,v,v
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 0201ef40 force ret=none args=v,v,v
  stub 0201f184 force ret=none args=v,v,v
  stub 0200be48 force ret=none args=v
  stub 02012ebc force ret=u32 args=v
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v
  stub 0215a5cc force ret=u16 args=v

# pdw_name_b(k, R, ctx): L; f = row k +0x14; R && f -> f(L, R, ctx); else 8 (rows: 021b6fe8 / 021b7068 / 021b70d4).
fn 021b6d64 ret=u32 inputs=200
  nowork
  region rec 0x100
  arg u32 range 0 3
  arg u32 alt const 0 || addrof region:rec
  arg u32 any
  fix region:rec 0xB0 u16 alt const 0x7FE || any
  fix region:rec 0 u32 alt const 0x23E || range 0 0x280 || any
  fix region:rec 0xB3 u8 alt const 2 || any
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021b6fe8 ret=none args=v
  stub 021b7068 ret=none args=v
  stub 021b70d4 ret=none args=v
  # closure: 021b6fe8 / 021b7068 / 021b70d4
  stub 0201ef40 force ret=none args=v,v,v

# pdw_give(k, a, b, c): f = row k +8; c && f -> f(a, b, c), 1; else 0 (rows: 021b6f18 / 021b6ff0 / 021b7080).
fn 021b6d98 ret=u32 inputs=200
  nowork
  region rec 0x100
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || addrof region:rec
  fix region:rec 0xB0 u16 alt const 0x7FE || any
  fix region:rec 0 u32 alt const 0x23E || range 0 0x280 || any
  fix region:rec 0xB3 u8 alt const 2 || any
  stub 021b6f18 ret=none args=v
  stub 021b6ff0 ret=none args=v
  stub 021b7080 ret=none args=v
  # closure: 021b6f18 / 021b6ff0 / 021b7080
  stub 021b6ee0 ret=u32 args=v,v
  stub 0200cde0 force ret=none args=v,v
  stub 0201a98c force ret=none args=v,v
  stub 020307b0 force ret=none args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:4/5/6/7 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 02007e50 force ret=u32 args=v,v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201292c force ret=u32 args=v
  stub 0200be48 force ret=none args=v
  stub 0200bf18 force ret=u32 args=v,v
  stub 02012ebc force ret=u32 args=v
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v
  stub 0215a5e4 force ret=u32 args=v

# pdw_kind(k, a, b, c): f = row k +0xC; c && f -> f(a, b, c); else 0 (rows: 021b6f5c / 021b7020 / 021b7098).
fn 021b6dc0 ret=u32 inputs=200
  nowork
  region rec 0x100
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || addrof region:rec
  fix region:rec 0xB0 u16 alt const 0x7FE || any
  fix region:rec 0 u32 alt const 0x23E || range 0 0x280 || any
  fix region:rec 0xB3 u8 alt const 2 || any
  stub 021b6f5c ret=none args=v
  stub 021b7020 ret=none args=v
  stub 021b7098 ret=none args=v
  # closure: 021b6f5c / 021b7020 / 021b7098
  stub 021b6ee0 ret=u32 args=v,v
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 020307b0 force ret=none args=v
  stub 021b6eb0 ret=u32 args=v,v
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0215a5e4 force ret=u32 args=v

# pdw_obj_or_slot(fm, GD): o = 021b6e24(fm) -> 0216D588(o); else r = 021b6e68(GD), r >= 0 ? (u16)r : 0.
fn 021b6de4 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x402 u16 alt const 0x46 || any rep=4,0x24
  stub 0216d588 force ret=u16 args=v
  stub 021b6e24 ret=nullable:2:u32 args=v
  stub 021b6e68 ret=pick:0/5/0xFFFFFFFF args=v
  # closure: 021b6e24 / 021b6e68
  stub 0216d598 force ret=pick:0x46/1/2 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0216dd1c force ret=pick:1/1/1/0 args=v,outm:4:0xFF/0xFF/0xFF/0xFF,p seq
  stub 02162740 force ret=nullable:3:ptr:arg1:0x400:1 args=v
  stub 02012844 force ret=u32 args=v
  stub 02162744 force ret=pick:0/1/2/3/4 args=v

# pdw_has_obj(fm): 021b6e24(fm) != 0.
fn 021b6e0c ret=u16 inputs=200
  nowork
  arg u32 any
  stub 021b6e24 ret=nullable:2:u32 args=v
  # closure: 021b6e24
  stub 0216d598 force ret=pick:0x46/1/2 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0216dd1c force ret=pick:1/1/1/0 args=v,outm:4:0xFF/0xFF/0xFF/0xFF,p seq

# find_obj_0x46(fm): S = 02188C7C(fm); iterate 0216DD1C(S, &o, &i) == 1: 0216D598(o) == 0x46 -> o; end -> 0.
fn 021b6e24 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 0216d598 force ret=pick:0x46/1/2 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0216dd1c force ret=pick:1/1/1/0 args=v,outm:4:0xFF/0xFF/0xFF/0xFF,p seq

# find_record_0x46(GD): S = 02012844(GD); A = 02162740(S), n = 02162744(S); !A or !n -> -1; first i < n with A[0x24 i + 2] u16 ==
# 0x46 -> A[0x24 i] u16; else -1.  Arena: arg 1 (never read).
fn 021b6e68 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x402 u16 alt const 0x46 || any rep=4,0x24
  stub 02162740 force ret=nullable:3:ptr:arg1:0x400:1 args=v
  stub 02012844 force ret=u32 args=v
  stub 02162744 force ret=pick:0/1/2/3/4 args=v

# pdw_is_special(R, k): k == 2 and R+0xB0 u16 == 0x7FE and R[0] == 0x23E.
fn 021b6eb0 ret=bool inputs=200
  nowork
  arg buf 0x100
  arg u32 alt const 2 || range 0 3
  fix arg0 0xB0 u16 alt const 0x7FE || any
  fix arg0 0 u32 alt const 0x23E || any

# pdw_make_mon(ctx, R): 02157E9C(R, 0215A5CC(ctx), 0215A5E4(ctx)) (mon from the record).
fn 021b6ee0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 02157e9c force ret=u32 args=v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0215a5e4 force ret=u32 args=v

# row 1 +4 (ctx, GD): party count < 6.
fn 021b6f00 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 0201aa18 force ret=pick:4/5/6/7 args=v
  stub 02012934 force ret=u32 args=v

# row 1 +8 (ctx, GD, R): party; mon = 021b6ee0(ctx, R); count < 6 and mon -> 0201A98C(party, mon), 0200CDE0(0200CA1C(GD), mon),
# Heap_Free(mon).
fn 021b6f18 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021b6ee0 ret=nullable:2:u32 args=v,v
  stub 0200cde0 force ret=none args=v,v
  stub 0201a98c force ret=none args=v,v
  stub 020307b0 force ret=none args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:4/5/6/7 args=v
  stub 0200ca1c force ret=u32 args=v
  # closure: 021b6ee0
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0215a5e4 force ret=u32 args=v

# row 1 +0xC (ctx, GD, R): mon = 021b6ee0(ctx, R); !mon -> 1; e = get(mon, 0x4C); Heap_Free(mon); e == 1 ? 2 : 1.
fn 021b6f5c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021b6ee0 ret=nullable:2:u32 args=v,v
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  stub 020307b0 force ret=none args=v
  # closure: 021b6ee0
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0215a5e4 force ret=u32 args=v

# row 1 +0x10 (L, R, ctx): X = 02012944(GD); mon = 021b6ee0(ctx, R); 0201F02C(L, 0, X); !mon -> 0201EEC4(L, 1, 0), 5; egg -> 0xB;
# else 0201EED4(L, 1, mon), 5; Heap_Free(mon).
fn 021b6f88 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021b6ee0 ret=nullable:2:u32 args=v,v
  stub 0201eec4 force ret=none args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0201f02c force ret=none args=v,v,v
  stub 020307b0 force ret=none args=v
  stub 0201eed4 force ret=none args=v,v,v
  stub 02017e1c force ret=pick:0/1 args=v,v,v
  # closure: 021b6ee0
  stub 02157e9c force ret=nullable:2:u32 args=v,v,v
  stub 0215a5cc force ret=u16 args=v

# row 1 +0x14: 6.
fn 021b6fe8 ret=u32 inputs=200
  nowork

# row 2 +4: 1.
fn 021b6fec ret=u32 inputs=200
  nowork

# row 2 +8 (ctx, GD, R): h = 0215A5CC(ctx); S = 0201292C(GD); it = (u16)R[0]; 1 <= it <= 0x272 -> 02007E50(S, it, 1, h).
fn 021b6ff0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x100
  fix arg2 0 u32 alt const 0x23E || range 0 0x280 || any
  fix arg2 0xB0 u16 alt const 0x7FE || any
  fix arg2 0xB3 u8 alt const 2 || any
  stub 02007e50 force ret=u32 args=v,v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201292c force ret=u32 args=v

# row 2 +0xC (ctx, GD, R): 021b6eb0(R, R[0xB3]) == 1 ? 4 : 3.
fn 021b7020 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x100
  fix arg2 0 u32 alt const 0x23E || range 0 0x280 || any
  fix arg2 0xB0 u16 alt const 0x7FE || any
  fix arg2 0xB3 u8 alt const 2 || any
  stub 021b6eb0 ret=pick:0/1 args=v,v

# row 2 +0x10 (L, R, ctx): 0201F02C(L, 0, 02012944(GD)); 0201EF40(L, 1, (u16)R[0]); 7.
fn 021b7038 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x100
  arg u32 any
  stub 0201ef40 force ret=none args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0201f02c force ret=none args=v,v,v
  stub 02012944 force ret=u32 args=v

# row 2 +0x14 (L, R): 0201EF40(L, 0, (u16)R[0]); 8.
fn 021b7068 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x100
  stub 0201ef40 force ret=none args=v,v,v

# row 3 +4: 1.
fn 021b707c ret=u32 inputs=200
  nowork

# row 3 +8 (ctx, GD, R): 0200BF18(0200BE48(SAVE), R[0]) (r0 = its result).
fn 021b7080 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x100
  fix arg2 0 u32 alt const 0x23E || range 0 0x280 || any
  fix arg2 0xB0 u16 alt const 0x7FE || any
  fix arg2 0xB3 u8 alt const 2 || any
  stub 0200be48 force ret=none args=v
  stub 0200bf18 force ret=u32 args=v,v
  stub 02012ebc force ret=u32 args=v

# row 3 +0xC: 5.
fn 021b7098 ret=u32 inputs=200
  nowork

# row 3 +0x10 (L, R, ctx): 0200BE48(SAVE); 0201F02C(L, 0, 02012944(GD)); 0201F184(L, 1, R[0]); 9.
fn 021b709c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x100
  arg u32 any
  stub 0215a5e4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0201f184 force ret=none args=v,v,v
  stub 0200be48 force ret=none args=v
  stub 0201f02c force ret=none args=v,v,v
  stub 02012ebc force ret=u32 args=v

# row 3 +0x14: 0xA.
fn 021b70d4 ret=u32 inputs=200
  nowork

# cmd 527: h = 02188C84(R+4) (R = 02158F58(SW)); key, i, var; T = 0204A298(0xD0, byte 0x021D99D0[i], 0, (h & 0x7FFF) | 0x8000,
# &size) (ARC 208 table of {key, value}); *var = 0, the value of the first entry with T.key == key; Heap_Free(T); 0.
fn 021b70d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 alt range 0 3 || any rep=31,8
  stub 02188c84 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0204a298 force ret=ptr:arg1:0x500:1 args=v,v,v,v,outm:4:0xF8/0/0/0
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 526: i, var; bag = 0201292C(GD); T = ARC 208 table (byte 0x021D99D0[i]); *var = 0; first entry whose item (u16) has
# 02007F68(bag, item, 1, h) -> *var = 1; Heap_Free(T); 0.
fn 021b716c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 alt range 0 3 || any rep=31,8
  stub 02188c84 force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02007f68 force ret=pick:0/0/1 args=v,v,v,v
  stub 0201292c force ret=u32 args=v
  stub 0204a298 force ret=ptr:arg1:0x500:1 args=v,v,v,v,outm:4:0xF8/0/0/0
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 585: key, i, a, b; L = 02158F74(SW); T = ARC 208 table (byte 0x021D99D0[i]); entry with T.key == key -> 0201EFA4(L, a,
# value, b, 0, 1) (word-set number); Heap_Free(T); 0.
fn 021b721c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 alt range 0 3 || any rep=31,8
  stub 02188c84 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v
  stub 0204a298 force ret=ptr:arg1:0x500:1 args=v,v,v,v,outm:4:0xF8/0/0/0
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 586: key, i; 020127A4(GD); T = ARC 208 table; entry with T.key == key -> 0200C278(02012F08(GD), value); Heap_Free(T); 0.
fn 021b72cc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 alt range 0 3 || any rep=31,8
  stub 02188c84 force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 020127a4 force ret=u32 args=v
  stub 02012f08 force ret=u32 args=v
  stub 0200c278 force ret=none args=v,v
  stub 0204a298 force ret=ptr:arg1:0x500:1 args=v,v,v,v,outm:4:0xF8/0/0/0
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 020307b0 force ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v

# cmd 564: v, var; *var = u16 table 0x021D99E8[v < 0xB ? v : 0]; 0.
fn 021b7364 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/5/0xA/0xB/0xC args=v,v seq
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# is_special_day(m, d): (m << 8 | d) (u16) in the 5 u16 of 0x021D99D4.
fn 021b7390 ret=bool inputs=200
  nowork
  arg u32 oneof 1 2 6 0xA 0xC 3
  arg u32 oneof 1 0xC 0xE 0x1E 5

# cmd 565: var; m = 021690D0(GD), d = 021690E0(GD); *var = 021b7390(m, d) ? 0xA : byte 0x021D99DE[d % 10]; 0.
fn 021b73b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021690e0 force ret=pick:1/5/0xC/0xE/0x1E/0x1F args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021b7390 ret=pick:0/1 args=v,v
  stub 021690d0 force ret=pick:1/2/6/0xA/0xC args=v
