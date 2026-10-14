# fndiff spec: overlay-21 script commands without a __FILE__ anchor (F8-spec4): F1-SPLIT rows 21.91 (0x021CF7DC..0x021D0140,
# between enceff.c and fldeff_btrain.c) and 21.97 (0x021D1670..0x021D30B8, between field_saveanime.c and event_game_manual.c);
# neither passes a __FILE__ string to Heap_AllocDebug (they allocate nothing), so the file names are unknown (scrcmd_misc21).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_misc21 --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  233 021cfa88  309 021cfa58  347 021d1a04  348 021d1a54  349 021d1a74  350 021d1ab0  412 021d1b2c  413 021d1b8c
#  472 021cfea0  473 021cff28  474 021cff78  475 021cffdc  476 021d0084  479 021d00b8  481 021cfef0  482 021d0014
#  483 021d004c  521 021d1670  522 021d16d8  523 021d179c  524 021d213c  525 021d2468  551 021d2524  553 021d2710
#  569 021cf7dc  570 021d2adc  571 021d2b18  573 021d2b54  574 021d2edc  575 021d2f44  582 021d0114  597 021d2f6c
#  598 021d3018  599 021d3068  600 021d3090
# 81 functions specced (35 command ids).
#  * 21.91: 569 counts distinct other-trainer OT ids in party / boxes / daycare (GMEVENT 0x021CF845, 2 boxes per frame, at most
#    0x32); 472..483 / 476 / 479 / 582 work on the 20-record save block 02009470 (0200973C record, 02009634 fields; the block sits
#    just before wifilist.c in ARM9 and may be comm-fed: role UNVERIFIED, logic specced); 0x021CFAC0..0x021CFC4C keep a 20-slot
#    list of ROM positions 0x021DB834 ({s16 x, s16 y, s16 z, u16 zone}) for field code; 309 proc 0x39; 233 a save counter.
#  * 21.97: 521..523 the 5 records of 02012F8C; 347..350 an OBJ animation proc (0x021D182C create, 0x021D1894 start,
#    0x021D18F4 per-frame, 0x021D1988 position + direction from the anime rotation 0 / 90 / 180 / 270 degrees); 412 / 413 / the
#    init proc 0x021D1BD8 drive 021A0xxx with 02159144 / 02159168 flags 8; field events 0x021D2278 / 0x021D2358 / 0x021D29A8
#    (SE 0x63C, then a warp 02184FD4 / 02184F88); 524 flag pairs (020249B0); 525 lottery (0x021DD6D0); 551 shows a 3D model until
#    A / B (render parts left out); 0x021D2750 grotto / hidden spot check; 570 / 571 record counters; 573 a numbered reward (RTC,
#    0x021DD774 rows); 0x021D2D50 / 0x021D2D7C the camera shake event of cmd 328; 574 / 575 / 597..600 call the overlay at
#    0x021F3xxx (outside the field set).  Roles UNVERIFIED (named by address).
# Not specced (11, render / UI flow, D3):
#   021d1c8c GMEVENT of 0x021D1C24: UI flow (15-state message / 3D model / fade / SE sequence)
#   021d1f78 3D model + anime resource setup (render)
#   021d201c 3D model resource free (render)
#   021d2040 3D model wobble (render)
#   021d20bc BMP window + text setup (render)
#   021d2124 BMP window free (render)
#   021d25a4 BMP window setup of cmd 551 (render)
#   021d25e8 BMP window clear thunk (render)
#   021d25f4 3D model setup of cmd 551 (render)
#   021d26ac 3D model free of cmd 551 (render)
#   021d29f0 cmd 414: draws / clears a text window (render only)
# Fragments (not functions): 0x021D0000 (inside cmd 475), 0x021D2312 / 0x021D2B3C (mid-function `bl`s reached by data words).
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
area scrcmd_misc21
set field

# cmd 569: var; ev = 020122C0(GAMESYS, 0, 0x021CF845, 0xD8); W = 020124E8(ev) = {02008530(02012944(GD)) (own id), 0 (box), var,
# 0 (count), ids[0x32]}; child event; 1.
fn 021cf7dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02008530 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT of cmd 569 (event, seq, W): GD = 020120F4(020124E4(event)); seq 0: 021cf8e4(W[0], GD, &W[3]) (party), seq 1; seq 1:
# boxes W[1], W[1] + 1 via 021cf940 (W[1]++ each; W[1] >= 24 -> seq 2); seq 2: 021cf9ac(W[0], GD, 0, &W[3]) (daycare), done;
# count >= 0x32 -> done; done -> *var = count, 1; else 0.
fn 021cf844 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 4 u32 alt range 0 0x18 || range 0x16 0x18
  fix arg2 8 u32 addrof arg2 0xE0
  fix arg2 0xC u32 range 0 0x34
  fix arg2 0x10 u32 alt range 0 3 || any rep=0x32,4
  stub 021cf940 ret=none args=v,v,v,v
  stub 021cf8e4 ret=none args=v,v,v
  stub 020124e4 force ret=u32 args=v
  stub 021cf9ac ret=none args=v,v,v,v
  stub 020120f4 force ret=u32 args=v
  # closure: 021cf940 / 021cf8e4 / 021cf9ac
  stub 020079bc force ret=pick:0/1/2/3 args=v,v,v,v,v
  stub 0201293c force ret=u32 args=v
  stub 021cfa04 ret=pick:0/1 args=v,v,v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/3 args=v,v,v
  stub 0200bac8 force ret=u32 args=v
  stub 02017e88 force ret=pick:0/1/2/3 args=v,v,v
  stub 0200bbec force ret=u32 args=v,v,v
  stub 02012ebc force ret=u32 args=v
  stub 021cfa1c ret=u32 args=v,v

# count_party(id, GD, list): for each party mon not an egg: 021cfa04(id, get(mon, 7) (OT id), list) -> n++; n.
fn 021cf8e4 ret=u32 inputs=200
  nowork
  arg u32 range 0 3
  arg u32 any
  arg buf 0xD8
  fix arg2 0 u32 alt range 0 0x34 || near 0x32 1
  fix arg2 4 u32 alt range 0 3 || any rep=0x32,4
  stub 021cfa04 ret=pick:0/1 args=v,v,v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/3 args=v,v,v
  # closure: 021cfa04
  stub 021cfa1c ret=u32 args=v,v

# count_box(id, GD, box, list): B = 0201293C(GD); for s < 30: 020079BC(B, box, s, 0xA9 (present) / 0x4C (egg) / 7 (OT id), 0)
# -> 021cfa04(id, OT, list) -> n++; n.
fn 021cf940 ret=u32 inputs=200
  nowork
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  arg buf 0xD8
  fix arg3 0 u32 alt range 0 0x34 || near 0x32 1
  fix arg3 4 u32 alt range 0 3 || any rep=0x32,4
  stub 020079bc force ret=pick:0/1/2/3 args=v,v,v,v,v
  stub 0201293c force ret=u32 args=v
  stub 021cfa04 ret=pick:0/1 args=v,v,v
  # closure: 021cfa04
  stub 021cfa1c ret=u32 args=v,v

# count_daycare(id, GD, k, list): S = 0200BAC8(SAVE); for i < 6: P = 0200BBEC(S, k, i); 02017E88(P, 0xA9) (present) ->
# 021cfa04(id, 02017E88(P, 7), list) -> n++; n.
fn 021cf9ac ret=u32 inputs=200
  nowork
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  arg buf 0xD8
  fix arg3 0 u32 alt range 0 0x34 || near 0x32 1
  fix arg3 4 u32 alt range 0 3 || any rep=0x32,4
  stub 021cfa04 ret=pick:0/1 args=v,v,v
  stub 0200bac8 force ret=u32 args=v
  stub 02017e88 force ret=pick:0/1/2/3 args=v,v,v
  stub 0200bbec force ret=u32 args=v,v,v
  stub 02012ebc force ret=u32 args=v
  # closure: 021cfa04
  stub 021cfa1c ret=u32 args=v,v

# add_other(id, ot, list): ot == id -> 0; else 021cfa1c(ot, list).
fn 021cfa04 ret=u32 inputs=200
  nowork
  arg u32 range 0 3
  arg u32 alt range 0 3 || any
  arg buf 0xD8
  fix arg2 0 u32 alt range 0 0x34 || near 0x32 1
  fix arg2 4 u32 alt range 0 3 || any rep=0x32,4
  stub 021cfa1c ret=pick:0/1 args=v,v

# add_unique(v, list = {count, ids[0x32]}): count >= 0x32 or v in ids -> 0; else ids[count++] = v, 1.
fn 021cfa1c ret=bool inputs=200
  nowork
  arg u32 alt range 0 3 || any
  arg buf 0xD8
  fix arg1 0 u32 alt range 0 0x34 || near 0x32 1
  fix arg1 4 u32 alt range 0 3 || any rep=0x32,4

# cmd 309: 021BB990(0x39, 02188D24(field map), 0x021F3900); 0.
fn 021cfa58 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188d24 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 021bb990 force ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 233: var; *var = 02008990(02008854(SAVE)) (u16); 0.
fn 021cfa88 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02008990 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02008854 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# find_slot(L, zone, x, y, z (stack)): for i < 20: e = L + 0x10 + 0x14 i; e+0x10 u16 == 0 -> 0xFF; R = 0x021DB834 + 8 e[0x12]
# ({s16 x, s16 y, s16 z, u16 zone}): R.zone == zone, R.x == x, R.z == z, |R.y - y| <= 1 -> i; none -> 0xFF.
# F8e-spec2: zone / x / z are weighted to records 0 (62, 417, 469, y 1) and 1 (96, 173, 412, y 0) so that all three match one
# record often; y covers both window edges of each (ry - 1 = 0 / -1, ry + 1 = 2 / 1) and a miss (3).
fn 021cfac0 ret=u32 inputs=600
  nowork
  arg buf 0x200
  arg u32 oneof 62 62 62 96 96 1
  arg u32 oneof 417 417 417 173 173 5
  arg u32 oneof 0 1 2 3 0xFFFFFFFF
  arg u32 oneof 469 469 469 412 412 9
  fix arg0 0x20 u16 alt const 0 || range 1 5 || range 1 5 || range 1 5 || range 1 5 || range 1 5 || range 1 5 || range 1 5 || range 1 5 || range 1 5 rep=20,0x14
  fix arg0 0x22 u8 range 0 1 rep=20,0x14

# take_slot(L, i, u16 out[10]): out = the 10 halfwords at L + 0x10 + 0x14 i; slots i + 1..19 move down one; L+0x19C = 0.
fn 021cfb1c ret=none inputs=200
  nowork
  arg buf 0x200
  arg u32 alt range 0 0x13 || const 0x14
  arg buf 0x20

# next_slot(L, u32 *i): from *i < 20 the first slot with e+0x10 u16 != 0 -> *i += 1 past it, 0x021DB834 + 8 e[0x12]; end -> *i =
# 20, 0.
fn 021cfb68 ret=u32 inputs=200
  nowork
  arg buf 0x200
  arg buf 0x10
  fix arg1 0 u32 alt range 0 0x13 || range 0x14 0x16
  fix arg0 0x20 u16 alt const 0 || any rep=20,0x14

# GMEVENT (event, seq, W = 10 halfwords): X = 021587B0(event, 0xAFC, 0, 0x15) (script); 02159058(X, W.u16[8], 0, 0, 0); L =
# 02158F74(X); B = 020457B0(8, 0x15); 02045994(B, &W (copy), 8); 0201EE0C(L, 3, B, 1, 1, 2); 02045808(B); 0.
fn 021cfba8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  stub 02045808 force ret=none args=v
  stub 0201ee0c force ret=none args=v,v,v,v,v,v
  stub 021587b0 force ret=u32 args=v,v,v,v
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 02045994 force ret=none args=v,in:16,v
  stub 02158f74 force ret=u32 args=v
  stub 02159058 force ret=none args=v,v,v,v,v

# take_slot_event(GAMESYS, -, i): ev = 020122C0(GAMESYS, 0, 0x021CFBA9, 0x14); 021cfb1c(0200BE48(SAVE), i, 020124E8(ev)); ev.
# Arena: arg 3 (never read).
fn 021cfc14 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 range 0 0x13
  arg buf 0x200
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 021cfb1c ret=none args=v,v,v
  stub 020124e8 force ret=ptr:arg3:0x1C0:1 args=v
  stub 0200be48 force ret=ptr:arg3:0:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# fill_matches(GAMESYS, a, b): R = 02012F38(GD); a < 0xE9 -> 021cfc80(02009470(SAVE), R, a, b); else 0200E878(R).  Arena: arg 3.
fn 021cfc4c ret=none inputs=200
  nowork
  arg u32 any
  arg u32 alt range 0 2 || range 0xE8 0xEA
  arg u32 any
  arg buf 0x800
  stub 021cfc80 ret=u32 args=v,v,v,v
  stub 0200e878 force ret=u32 args=v
  stub 02009470 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012f38 force ret=ptr:arg3:0x500:1 args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 021cfc80
  stub 0200856c force ret=pick:0/1/2 args=v
  stub 02008550 force ret=pick:0/1 args=v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 02008554 force ret=pick:0/5/0xF/0x10/0x20 args=v
  stub 02025c74 force ret=u32 args=v
  stub 0200973c force ret=nullable:3:u32 args=v,v

# fill_matches(S, R, a, b): R[0] = a, R[2] = b, R[1] = 0; for i < 20 while R[1] < 5: (u8)02009634(S, i, 6) and P =
# 0200973C(S, i) and 0200856C(P) == a -> t = 02008554(P), t > 0xF -> (02008550(P) ? 0xF : 0xB); R[8 + R[1]] = i,
# R[3 + R[1]] = 02025C74(t), R[1]++.
fn 021cfc80 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x20
  arg u32 range 0 2
  arg u32 any
  stub 0200856c force ret=pick:0/1/2 args=v
  stub 02008550 force ret=pick:0/1 args=v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 02008554 force ret=pick:0/5/0xF/0x10/0x20 args=v
  stub 02025c74 force ret=u32 args=v
  stub 0200973c force ret=nullable:3:u32 args=v,v

# match_index(R, i): i < 5 and i < R[1] -> R[8 + i]; else 0xFF.
fn 021cfcf4 ret=u8 inputs=200
  nowork
  arg buf 0x10
  arg u32 alt range 0 6 || any
  fix arg0 1 u8 range 0 6

# match_message(L, GAMESYS, R, i, x (stack)): S = 02009470(SAVE); i >= 5 -> 4; k = R[8 + i]; 021cfdbc(L, GD, k); P =
# 0200973C(S, k); return 021cfd88(02008550(P), 02008530(P), 02009634(S, k, 4), x, 02025B64(*02012F38(GD)) != 0).
fn 021cfd08 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 range 0 6
  arg u32 alt const 0 || any
  stub 02025b64 force ret=pick:0/1 args=v
  stub 0200973c force ret=nullable:3:u32 args=v,v
  stub 021cfd88 ret=u32 args=v,v,v,v,v
  stub 02009470 force ret=u32 args=v
  stub 02008550 force ret=pick:0/1 args=v
  stub 02008530 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 02012f38 force ret=ptr:arg2:0:1 args=v
  stub 021cfdbc ret=none args=v,v,v
  stub 02012ebc force ret=u32 args=v
  # closure: 021cfd88 / 021cfdbc
  stub 0201ee60 force ret=none args=v,v,v
  stub 02012944 force ret=u32 args=v
  stub 0201f198 force ret=u32 args=v,v,v
  stub 0201f048 force ret=none args=v,v,v
  stub 02009628 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 0201f02c force ret=none args=v,v,v
  stub 02008570 force ret=u32 args=v
  stub 0201f094 force ret=none args=v,v,v,v

# message_id(f, o, m, x, g (stack)): base = g ? 4 : 0x44; m < 5 -> base += 0xC m; f -> base += 6; x == 0 -> base += 1 + o % 5.
fn 021cfd88 ret=u32 inputs=200
  nowork
  arg u32 alt const 0 || any
  arg u32 any
  arg u32 range 0 6
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any

# match_words(L, GD, k): S = 02009470(SAVE); lang = *02012F38(GD); P = 0200973C(S, k); 0201F048(L, 0, lang); 02025B64(lang) ->
# 0201F094(L, 1, lang, 02008570(P)); 0201F02C(L, 3, 02012944(GD)); 0201EE60(L, 5, (u16)02009634(S, k, 0)); 0201F198(L, 6,
# 02009628(S)); 0201F02C(L, 7, P); 0201EFA4(L, 8, 02009634(S, k, 3) or 1, 3, 0, 1); 0201EE60(L, 9, (u16)02009634(S, k, 1));
# 0201F198(L, 0xA, (u8)02009634(S, k, 2)).  Arena: arg 3.
fn 021cfdbc ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 0201ee60 force ret=none args=v,v,v
  stub 02025b64 force ret=pick:0/1 args=v
  stub 02012944 force ret=u32 args=v
  stub 0200973c force ret=nullable:3:u32 args=v,v
  stub 0201f198 force ret=u32 args=v,v,v
  stub 0201f048 force ret=none args=v,v,v
  stub 02009628 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 0201f02c force ret=none args=v,v,v
  stub 02009470 force ret=u32 args=v
  stub 02008570 force ret=u32 args=v
  stub 0201f094 force ret=none args=v,v,v,v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 02012f38 force ret=ptr:arg3:0:1 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 472: v, var; *var = 02009590(02009470(SAVE), 02012944(GD), v); 0.
fn 021cfea0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012ebc force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02009470 force ret=u32 args=v
  stub 02009590 force ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 481: var; *var = 020095C8(02009470(SAVE)); 0.
fn 021cfef0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020095c8 force ret=u32 args=v
  stub 02009470 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 473: R = 02012F38(GD); k = 021cfcf4(R, (u8)value); k != 0xFF -> 02009698(02009470(SAVE), k, 5, 1); 0.
fn 021cff28 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x501 u8 range 0 6
  stub 02012ebc force ret=u32 args=v
  stub 02009698 force ret=none args=v,v,v,v
  stub 02159b10 force ret=pick:0/1/2/3/5 args=v,v seq
  stub 02009470 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021cfcf4 ret=pick:0/1/0xFF args=v,v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 474: R; i, var; *var = 0; k = 021cfcf4(R, i); k != 0xFF and (u8)02009634(S, k, 5) -> *var = 1; 0.
fn 021cff78 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x501 u8 range 0 6
  stub 02012ebc force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=pick:0/1/2/3/5 args=v,v seq
  stub 02009470 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021cfcf4 ret=pick:0/1/0xFF args=v,v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 475: 020095FC(02009470(SAVE), (u8)value); 0.
fn 021cffdc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02009470 force ret=u32 args=v
  stub 020095fc force ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 482: 02009618(02009470(SAVE), (u8)value); 0.
fn 021d0014 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02009470 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02009618 force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 483: var; *var = 02009628(02009470(SAVE)); 0.
fn 021d004c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02009628 force ret=u32 args=v
  stub 02009470 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 476: a, b; 021cfc4c(GAMESYS, a, b); 0.
fn 021d0084 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/2/0xE8/0xE9 args=v,v seq
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021cfc4c ret=none args=v,v,v
  # closure: 021cfc4c
  stub 021cfc80 ret=u32 args=v,v,v,v
  stub 0200e878 force ret=u32 args=v
  stub 02009470 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 02012ebc force ret=u32 args=v
  stub 0200856c force ret=pick:0/1/2 args=v
  stub 02008550 force ret=pick:0/1 args=v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 02008554 force ret=pick:0/5/0xF/0x10/0x20 args=v
  stub 02025c74 force ret=u32 args=v
  stub 0200973c force ret=nullable:3:u32 args=v,v

# cmd 479: L = 02158F74(SW); R = 02012F38(GD); i = value; x = u16 operand (02011330); var; *var = 021cfd08(L, GAMESYS, R, i, x); 0.
fn 021d00b8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x501 u8 range 0 6
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021cfd08 ret=u16 args=v,v,v,v,v
  stub 02159b10 force ret=pick:0/1/2/4/5 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012f38 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5dc force ret=u32 args=v
  # closure: 021cfd08
  stub 02025b64 force ret=pick:0/1 args=v
  stub 0200973c force ret=nullable:3:u32 args=v,v
  stub 021cfd88 ret=u32 args=v,v,v,v,v
  stub 02009470 force ret=u32 args=v
  stub 02008550 force ret=pick:0/1 args=v
  stub 02008530 force ret=u32 args=v
  stub 02009634 force ret=pick:0/1/2/0x100/0x10002 args=v,v,v
  stub 021cfdbc ret=none args=v,v,v
  stub 02012ebc force ret=u32 args=v
  stub 0201ee60 force ret=none args=v,v,v
  stub 02012944 force ret=u32 args=v
  stub 0201f198 force ret=u32 args=v,v,v
  stub 0201f048 force ret=none args=v,v,v
  stub 02009628 force ret=u32 args=v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 0201f02c force ret=none args=v,v,v
  stub 02008570 force ret=u32 args=v
  stub 0201f094 force ret=none args=v,v,v,v

# cmd 582: var; *var = 0200856C(02012944(GD)) != 0; 0.
fn 021d0114 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012944 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200856c force ret=pick:0/1/2 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 521: k, _, _, var; Z = 02012F8C(GD); k == 0 -> *var = 020092EC(Z) == 1 && 020092D8(Z) == 1; else nothing; 0.
fn 021d1670 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f8c force ret=u32 args=v
  stub 020092ec force ret=pick:0/1/2 args=v
  stub 02159b10 force ret=pick:0/1/2 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020092d8 force ret=pick:0/1/2 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 522: k, a, _, var; Z = 02012F8C(GD); k 0: *var = 1, then 0 if any of the 5 records 02009390(Z, i) is null or tagged 0x7E /
# 0; k 1: *var = 020093D8(Z) != 0x7F; k 2: *var = 02009418(Z) == 1; k 3: 02009428(Z, 0); k 4: 020093F0(Z, a); 0.
fn 021d16d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u16 alt const 0x7E || const 0 || any
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02009428 force ret=none args=v,v
  stub 02012f8c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4/5 args=v,v seq
  stub 020093f0 force ret=none args=v,v
  stub 02009390 force ret=nullable:3:ptr:arg1:0x500:1 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020093d8 force ret=pick:0/0x7F args=v
  stub 02009418 force ret=pick:0/1/2 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 523: L = 02158F74(SW); h = 0215A5CC(ctx); Z; k, i; p = 02009390(Z, i); p and *p not 0x7E / 0 -> B = 020457B0(0xE, h);
# 02045994(B, p + 2, 0xD); 0201EE0C(L, k, B, 0, 0, 2); 02045808(B); 0.
fn 021d179c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u16 alt const 0x7E || const 0 || any
  stub 0215a5e4 force ret=u32 args=v
  stub 02045808 force ret=none args=v
  stub 0215a5cc force ret=u16 args=v
  stub 02012f8c force ret=u32 args=v
  stub 0201ee0c force ret=none args=v,v,v,v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02009390 force ret=nullable:3:ptr:arg1:0x500:1 args=v,v
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02045994 force ret=none args=v,v,v
  stub 0215a5dc force ret=u32 args=v

# obj_anim_create(fm, id, vec, h): S = 02188C7C(fm); X = 021BB990(-1, 02188D24(fm), 0x021DD5F0) (field proc); W = 021BBA6C(X);
# W+0x188 = X, W+0x19C = h (u16), W+0x18C..0x198 = *vec (12 bytes), W[0] = 0216DE94(S, id) (the OBJ); W.  Arena: arg 4.
fn 021d182c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u16 any
  arg buf 0x200
  stub 02188d24 force ret=u32 args=v
  stub 021bba6c force ret=ptr:arg4:0:1 args=v
  stub 021bb990 force ret=u32 args=v,v,v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 02188c7c force ret=u32 args=v

# obj_anim_delete(W): tail call 021BBA1C(W+0x188) (thunk `ldr r3, =0x021BBA1D; bx r3`).
fn 021d1884 ret=none inputs=200
  nowork
  arg buf 0x200
  stub 021bba1c force ret=none args=v

# obj_anim_start(W, k): A = 0201147C(W+0x19C u16, 0xCB, k, 0xA, W + 4, 0x180) (anime); W+0x184 = A, W+0x19A = 1, W+0x199 = k;
# 021d1988(A, W[0], W + 0x18C); 0216D568(W[0], 0x8000).
fn 021d1894 ret=none inputs=200
  nowork
  arg buf 0x200
  arg u32 range 0 7
  stub 021d1988 ret=none args=v,v,v
  stub 0216d568 force ret=none args=v,v
  stub 0201147c force ret=u32 args=v,v,v,v,v,v
  # closure: 021d1988
  stub 0216d60c force ret=u32 args=v,v
  stub 02011610 force ret=none args=v,out:12
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 02011638 force ret=none args=v,outw:12:0/0x5A000/0xB4000/0x10E000/0x2D000
  stub 0216d89c force ret=none args=v,in:12

# obj_anim_done(W): W+0x19A == 0.
fn 021d18e0 ret=bool inputs=200
  nowork
  arg buf 0x200
  fix arg0 0x19A u8 alt const 0 || any

# obj_anim_proc(-, -, W): W+0x19A == 1: A = W+0x184; 02011590(A, 0x1000) == 0 -> 021d1988(A, W[0], W + 0x18C); else
# 02011558(A), W+0x184 = 0, W+0x19A = 0, 0216D88C(W[0], &p), 0216E014(W[0], &p, 0216D5F8(W[0])), table 0x021DD5E0[W+0x199] !=
# 0 -> 0216D570(W[0], 0x8000); 0216D758(W[0]).
fn 021d18f4 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x200
  fix arg2 0x19A u8 alt range 0 1 || any
  fix arg2 0x199 u8 range 0 7
  stub 02011558 force ret=none args=v
  stub 021d1988 ret=none args=v,v,v
  stub 0216d5f8 force ret=u16 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02011590 force ret=pick:0/1 args=v,v
  stub 0216d570 force ret=none args=v,v
  stub 0216e014 force ret=none args=v,in:12,v
  stub 0216d758 force ret=u32 args=v
  # closure: 021d1988
  stub 0216d60c force ret=u32 args=v,v
  stub 02011610 force ret=none args=v,out:12
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 02011638 force ret=none args=v,outw:12:0/0x5A000/0xB4000/0x10E000/0x2D000
  stub 0216d89c force ret=none args=v,in:12

# (empty proc callback; `bx lr`).
fn 021d1984 ret=none inputs=200
  nowork

# obj_anim_apply(A, M, vec): 02011610(A, &t); 02011638(A, &r); VEC_Add(t, vec, t) (0207C9D0); 0216D89C(M, &t); r.y rotation 0 /
# 0x5A000 / 0xB4000 / 0x10E000 (fx32 degrees) -> 0216D60C(M, 1 / 3 / 0 / 2); other angles: no direction change.
fn 021d1988 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 0216d60c force ret=u32 args=v,v
  stub 02011610 force ret=none args=v,out:12
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 02011638 force ret=none args=v,outw:12:0/0x5A000/0xB4000/0x10E000/0x2D000
  stub 0216d89c force ret=none args=v,in:12

# cmd 347: R = 02158F58(SW); h = 02188C84(R+4); k; 021A3038(02188CB4(R+4), &v) (player position); W = 021d182c(R+4, k, &v, h);
# 02159104(SW, W); 0.
fn 021d1a04 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c84 force ret=u32 args=v
  stub 02159104 force ret=none args=v,v
  stub 021d182c ret=u32 args=v,v,in:12,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 021a3038 force ret=none args=v,out:12
  # closure: 021d182c
  stub 02188d24 force ret=u32 args=v
  stub 021bba6c force ret=ptr:arg1:0x500:1 args=v
  stub 021bb990 force ret=u32 args=v,v,v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 02188c7c force ret=u32 args=v

# cmd 348: W = 0215910C(SW); 021d1884(W); 02159104(SW, 0); 0.
fn 021d1a54 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021d1884 ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215910c force ret=ptr:arg1:0x500:1 args=v
  stub 02159104 force ret=none args=v,v
  stub 021bba1c force ret=none args=v

# cmd 349: k; 021d1894(0215910C(SW), k); 0.
fn 021d1a74 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x699 u8 range 0 7
  stub 02159b10 force ret=pick:0/1/2/3/4/5/6/7 args=v,v seq
  stub 021d1894 ret=none args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215910c force ret=ptr:arg1:0x500:1 args=v
  # closure: 021d1894
  stub 021d1988 ret=none args=v,v,v
  stub 0216d568 force ret=none args=v,v
  stub 0201147c force ret=u32 args=v,v,v,v,v,v
  stub 0216d60c force ret=u32 args=v,v
  stub 02011610 force ret=none args=v,out:12
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12
  stub 02011638 force ret=none args=v,outw:12:0/0x5A000/0xB4000/0x10E000/0x2D000
  stub 0216d89c force ret=none args=v,in:12

# GMEVENT of cmd 350 (event, seq, W = {obj anim W}): 021d18e0(W[0]) == 1.
fn 021d1a9c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x200
  fix arg2 0 u32 addrof arg2 0x40
  fix arg2 0x1DA u8 alt const 0 || any
  stub 021d18e0 ret=pick:0/1 args=v

# cmd 350: ev = 020122C0(GAMESYS, 0, 0x021D1A9D, 4); W = 020124E8(ev) = {0215910C(SW)}; child event; 1.
fn 021d1ab0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 0215910c force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT of cmds 412 / 413 (event, seq, W = {X, mode}): seq 0: 021A0ED4(X) -> 021A0F30(X, mode), seq 1; seq 1: 021A0ED4(X) ->
# 1; else 0.
fn 021d1aec ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021a0f30 force ret=none args=v,v
  stub 021a0ed4 force ret=pick:0/1 args=v

# cmd 412: R; k; X = 02188CCC(R+4); ev = 020122C0(GAMESYS, 0, 0x021D1AED, 8); W = {X, u32 table 0x021DD608[k]}; child event;
# 02159144(8); 1.
fn 021d1b2c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 02159144 force ret=none args=v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02188ccc force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 413: X; W = {X, 0}; child event (0x021D1AED); 02159168(8); 1.
fn 021d1b8c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02159168 force ret=none args=v
  stub 02188ccc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# init proc (table 0x021705F0; task, w): X = 02188CCC(02158F58(task[8])+4); w[0] 0: 021A0ED4(X) -> 021A0F30(X, 0),
# 02159168(8), w[0] 1; w[0] 1: 021A0ED4(X) -> 1; else 0.
fn 021d1bd8 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  stub 02159168 force ret=none args=v
  stub 02188ccc force ret=u32 args=v
  stub 021a0f30 force ret=none args=v,v
  stub 021a0ed4 force ret=pick:0/1 args=v
  stub 02158f58 force ret=ptr:arg0:0x80:1 args=v

# show_mon_event(GAMESYS, fm, slot): ev = 020122C0(GAMESYS, 0, 0x021D1C8D, 0x58); W = 020124E8(ev) cleared (0x58 bytes) =
# {02007B44(SAVE), GAMESYS, GAMEDATA, fm, 02188CB4(fm), party mon slot, -, 02188C54(fm), .., +0x4E slot (u8), +0x54 0}; ev.
# (Its GMEVENT 0x021D1C8D is UI flow: left out.)  Arena: arg 3.
fn 021d1c24 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 range 0 5
  arg buf 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 02007b44 force ret=u32 args=v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 02188c54 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# callback: *p = 1; 1.
fn 021d1f48 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10

# pause_objs(W): 0216D9A0(02188C7C(0201210C(W[1]))).
fn 021d1f50 ret=none inputs=200
  nowork
  arg buf 0x60
  stub 02188c7c force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 0216d9a0 force ret=u32 args=v

# resume_objs(W): 0216D9E4(02188C7C(0201210C(W[1]))).
fn 021d1f64 ret=none inputs=200
  nowork
  arg buf 0x60
  stub 02188c7c force ret=u32 args=v
  stub 0216d9e4 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# cmd 524: k, a, b, var; k 1 -> (f, g) = (0x5C0, 0x67B), k 2 -> (0x664, 0x6B7), else *var = 0, 0; *var = 020249B0(f, a & 0x7FF)
# && 020249B0(g, b & 0x7FF); 0.
fn 021d213c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 020249b0 force ret=pick:0/1 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# in_table_a(v): v is a key of the 21 {u32 key, u32 flag} records at 0x021DD614.
fn 021d21c0 ret=bool inputs=200
  nowork
  arg u32 oneof 0x68a 0x690 0x692 0x7ef 0x77f 5 0

# in_table_b(v): v is a key of 0x021DD614 whose flag is 1.
fn 021d21e0 ret=bool inputs=200
  nowork
  arg u32 oneof 0x68a 0x690 0x692 0x7ef 0x77f 5 0

# count_script(GAMESYS, fm): n = 02008990(02008854(SAVE)); n == key of the 5 {u16 n, u16 script} at 0x021DD6BC -> 02158768(GAMESYS,
# script, 0, 02188C84(fm)); n > 500 -> 02158768(GAMESYS, 0x28E1, 0, heap); else 0.
fn 021d220c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 02188c84 force ret=u32 args=v
  stub 02158768 force ret=u32 args=v,v,v,v
  stub 02008990 force ret=pick:0/7/100/300/450/500/501 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02008854 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# GMEVENT (event, seq, W = {GAMESYS?, fm, -, n}): 0: 021C6DE0(02188D38(fm), 021C7858(fm, 0x28, &{0, -0x40000, 0}), 0), n = 8;
# 1: --n <= 0 -> 020061E4(0x63C) (SE); 2: 02006224(02006148(0x63C)) == 0 -> M = 021A3074(P), 021A30A4(P, 3), 021CD440(fm, &z),
# 02012374(event, 02184FD4(W[0], z)); 3: 0216D410(M, 8); 4: 0216D454(M) -> 1; each step seq++; 0.
fn 021d2278 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0xC u32 alt range 0 2 || any
  stub 021cd440 force ret=none args=v,outm:2:0xFF/0xFF
  stub 021c6de0 force ret=none args=v,v,v
  stub 02188d38 force ret=u32 args=v
  stub 02006224 force ret=pick:0/1 args=v
  stub 021a30a4 force ret=none args=v,v
  stub 020061e4 force ret=none args=v
  stub 02006148 force ret=pick:0/1/2/3/4 args=v
  stub 0216d410 force ret=none args=v,v
  stub 021a3074 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0216d454 force ret=pick:0/1 args=v
  stub 02184fd4 force ret=u32 args=v,v
  stub 02012374 force ret=none args=v,v
  stub 021c7858 force ret=u32 args=v,v,in:12

# GMEVENT (event, seq, W = {GAMESYS?, fm, flag, n}): 0: 020061E4(0x63C), n = 0x18, flag -> S = 02188C64(fm), a = 021C7260(fm,
# 0x14, 3), b = 021C71D4(fm, 0xC, 3), 021C6DE0(T, a, 0), 021C6DE0(T, b, a), 021A01FC(S, 0x7F0F, 021A01D4(S), 0x1A); 1:
# 021A30A4(02188CB4(fm), 2); 2: --n <= 0 -> 02012374(event, 02184F88(W[0])); 3: 1; each step seq++; 0.
fn 021d2358 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 8 u32 alt const 0 || any
  fix arg2 0xC u32 alt range 0 2 || any
  stub 021c7260 force ret=u32 args=v,v,v
  stub 021c6de0 force ret=none args=v,v,v
  stub 021a30a4 force ret=none args=v,v
  stub 020061e4 force ret=none args=v
  stub 02188c64 force ret=u32 args=v
  stub 02184f88 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02012374 force ret=none args=v,v
  stub 021c71d4 force ret=u32 args=v,v,v
  stub 021a01d4 force ret=u32 args=v
  stub 02188d38 force ret=u32 args=v
  stub 021a01fc force ret=none args=v,v,v,v

# event_2278(GAMESYS, fm): ev = 020122C0(GAMESYS, 0, 0x021D2279, 0x10); W = {GAMESYS, fm, 0}; ev.  Arena: arg 2.
fn 021d2418 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v

# event_2358(GAMESYS, fm, flag): W = {GAMESYS, fm, flag}; ev (0x021D2359).  Arena: arg 3.
fn 021d2440 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg3:0:1 args=v

# cmd 525 (lottery): v, A, B, C; T = the 31 u16 of 0x021DD6D0; S = 02008B08(SAVE); m = 021690D0(GD), d = 021690E0(GD); d ==
# 02008B98(S) and m == 02008B94(S) -> *A = 0x34, *B = 0x53, *C = T[30]; else r = 020056EC(v ? 0x1E : 0x16), *A = r + 0x16, *B
# = r + 0x35, *C = T[r]; 0.
fn 021d2468 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021690e0 force ret=pick:1/2 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 02008b08 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020056ec force ret=lt:0 args=v
  stub 02159b10 force ret=pick:0/1 args=v,v seq
  stub 021690d0 force ret=pick:1/2 args=v
  stub 02008b94 force ret=pick:1/2 args=v
  stub 02008b98 force ret=pick:1/2 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 551: a, b, c, d; ev = 020122C0(GAMESYS, 0, 0x021D26D1, 0x18); W = 020124E8(ev); 021d25a4(W) (window, render);
# 021d25f4(W, a, b, c, d, 0) (3D model, render); child event; 1.
fn 021d2524 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 021d25a4 ret=none args=v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021d25f4 ret=none args=v,v,v,v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# GMEVENT of cmd 551 (event, seq, W): seq 0: 020362DC() & 3 (A / B held) -> 021d26ac(W), 021d25e8(W) (render frees), seq 1;
# seq 1: 020452E8(W[0]) (window delete), 1; else 0.
fn 021d26d0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 020452e8 force ret=u32 args=v
  stub 021d26ac ret=none args=v
  stub 020362dc force ret=pick:0/1/2/3/4 args=
  stub 021d25e8 ret=none args=v

# cmd 553: v, var; *var = 0216BF9C(02012F08(GD), v, 02012944(GD)) (u16); 0.
fn 021d2710 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012944 force ret=u32 args=v
  stub 02012f08 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0216bf9c force ret=u32 args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# grotto_check(GAMESYS) (0 = blocked): R = 02012844(GD), A = 02162740(R), n = 02162744(R); fm; P = 02188CB4(fm); z = 02188C9C(fm);
# 02013C60(z) == 0 -> 0; 021A3098(P) in 2..3 -> 0; (!A or !n) and z not 0 / 0x1A8 -> 1; per record e = A + 0x24 i: e[0x18] 0 ->
# the player (021A35FC) inside e's rect (x e+0x1C .. + o[0xB], z e+0x1E - o[0xC] ..; o = 0216E2E8 header) -> 0; e[0x18] 1 and
# 02188CFC(fm) == 1 -> 021A36BC position == (e+0x1C, e+0x1E, e+0x20) -> 0; then 02188CFC(fm) == 0 and 02162890(R,
# 0201296C(GD), pos of 021A3074(P)) -> 0; z 0 / 0x1A8: the (x, z) byte pairs of 021867C4 / 02186830 tables (0218684 4 / C,
# 10 each) == player -> 0; 1.  Arena: arg 1.  02162740 never answers null here: with n > 0 and A == 0 the loop reads
# address 0 (the original does not test A there), so the `!A` arm is not covered.
fn 021d2750 ret=bool inputs=200
  nowork
  arg u32 any
  arg buf 0x800
  fix arg1 0x118 u32 alt range 0 1 || any rep=4,0x24
  fix arg1 0x102 u16 range 0 7 rep=4,0x24
  fix arg1 0x11C u16 range 0 1 rep=4,0x24
  fix arg1 0x11E u16 range 0 1 rep=4,0x24
  fix arg1 0x120 u16 range 0 1 rep=4,0x24
  fix arg1 0x200 u8 range 0 7 rep=0x20,1
  stub 02186804 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02162890 force ret=pick:0/1 args=v,v,in:12
  stub 021a3098 force ret=pick:0/1/2/3/4 args=v
  stub 02188c9c force ret=pick:0/0x1A8/5 args=v
  stub 021a35fc force ret=none args=v,outm:2:7/0,outm:2:7/0,outm:2:7/0
  stub 02186830 force ret=u32 args=v,v,v
  stub 02188cb4 force ret=u32 args=v
  stub 0218683c force ret=none args=v
  stub 021a36bc force ret=none args=v,outm:8:1/0
  stub 02162740 force ret=ptr:arg1:0x100:1 args=v
  stub 02162744 force ret=pick:0/1/2/3 args=v
  stub 02012844 force ret=u32 args=v
  stub 021867c4 force ret=u32 args=v,v
  stub 02186844 force ret=ptr:arg1:0x200:4 args=v,v
  stub 0218684c force ret=ptr:arg1:0x210:4 args=v,v
  stub 02188c7c force ret=u32 args=v
  stub 02013c60 force ret=pick:0/1/1 args=v
  stub 0216e2e8 force ret=none args=v,v,outm:0x10:0/0/0/0/0/0/0/0/0/0/0/7/7/0/0/0
  stub 0201296c force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 02188cfc force ret=pick:0/1 args=v

# event_29a8(GAMESYS): ev = 020122C0(GAMESYS, 0, 0x021D29A9, 8); W = {fm, 02188CCC(fm)}; ev.  Arena: arg 1.
fn 021d297c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0:1 args=v
  stub 02188ccc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v

# GMEVENT (event, seq, W = {fm, X}): 0: 021A0F30(X, 9), seq 1; 1: 021A0F94(X) == 0xE -> 021A0FA8(X), seq 2; 2: 021A0F30(X, 0), 1;
# else 0.
fn 021d29a8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  stub 021a0f30 force ret=none args=v,v
  stub 021a0f94 force ret=pick:0/0xE/1 args=v
  stub 021a0fa8 force ret=none args=v

# cmd 570: a, b; 02008DAC(02008C58(SAVE), a, b); 0.
fn 021d2adc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02008dac force ret=none args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020120f4 force ret=u32 args=v
  stub 02008c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 571: a, var; *var = 02008DFC(02008C58(SAVE), a) (u16); 0.
fn 021d2b18 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02008dfc force ret=u32 args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 02008c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# cmd 573: L = 02158F74(SW); X = 02012944(GD); v, var; 0201F02C(L, 0, X); first party mon not an egg -> 0201EE60(L, 3, species);
# 0203F4CC(&t) (RTC); t[1] < 10: r = 020056EC(0x1D), row = 0x021DD774 + 16 r = {a, b, na, nb}: a != -1 -> s1 = 02008DFC(Z =
# 02012F2C(GD), a), 0201EFA4(L, 1, s1, na, 0, 1); b != -1 -> s2, 0201EFA4(L, 2, s2, nb, 0, 1); r < 11 and the record not
# complete (a / b present with s 0) -> r = 0; r >= 11 or 02008550(X) == 1 -> r += 11; base 0; t[1] < 0x23 -> base 0x28, r =
# 020056EC(0x32); else base 0x5A, r = 020056EC(0x50); *var = 021d2d24(GD, 021d2ce8(GD, base + r), v); 0.
fn 021d2b54 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0201aa18 force ret=pick:0/1/2/3 args=v
  stub 02017e1c force ret=pick:0/1/2 args=v,v,v
  stub 0201efa4 force ret=none args=v,v,v,v,v,v
  stub 021d2d24 ret=u16 args=v,v,v
  stub 0201ab28 force ret=u32 args=v,v
  stub 0201f02c force ret=none args=v,v,v
  stub 02012f2c force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0203f4cc force ret=none args=outm:16:0xFF/0xFF/0xFF/0xFF/0x3F/0/0/0
  stub 02008550 force ret=pick:0/1/2 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0201ee60 force ret=none args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021d2ce8 ret=pick:0/0x87/0x66/5 args=v,v
  stub 020056ec force ret=lt:0 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02008dfc force ret=pick:0/1/5 args=v,v
  # closure: 021d2d24 / 021d2ce8
  stub 0200c0f0 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  stub 0200c234 force ret=pick:0/1 args=v

# adjust_a(GD, x): x in the 8 u32 of 0x021DD718 -> 0200C234(0200C0F0(SAVE)) ? x : 0x5A + 020056EC(0xC); else x.
fn 021d2ce8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 oneof 0x66 0x6d 5 0x70
  stub 0200c0f0 force ret=u32 args=v
  stub 020056ec force ret=lt:0 args=v
  stub 02012ebc force ret=u32 args=v
  stub 0200c234 force ret=pick:0/1 args=v

# adjust_b(GD, x, v): v == 0 and x in the 15 u32 of 0x021DD738 -> 0x6E + 020056EC(0x14); else x.
fn 021d2d24 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 oneof 0x87 0x95 3 0x96
  arg u32 alt const 0 || any
  stub 020056ec force ret=lt:0 args=v

# shake_event(GAMESYS, rec): ev = 020122C0(GAMESYS, 0, 0x021D2D7D, 0x1C); W = 020124E8(ev) = the 0x18 bytes of rec; ev.
# Arena: arg 2.
fn 021d2d50 ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x20
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v

# GMEVENT of cmd 328 (event, seq; W = 020124E8(event) = {s16 ax, s16 ay, s16 dx, s16 dy, u16 every, u16 k, u16 period, u8 n0,
# u8 n, frames, t, pos *}): E = 02188C58(fm); 0: 0218F0A8(E); 1: 0218F0E8(E) == 0 -> W.pos = 0218EC5C(E), 0218EC54(E); 2: t++,
# phase = (u16)((t << 16) / frames); every && n0 - n >= every -> ++k >= period -> k = 0, a -= d (min 1); t >= frames -> n--,
# t = 0; n == 0 -> 0218ED04(E, &0), 0218EC30(E, W.pos), 0218F05C(E), next; else s = sin table 0x020A1B38[phase >> 4],
# 0218EC90(E, &(*W.pos + (ax s, ay s, 0))), 0218ED04(E, &(ax s, ay s, 0)); 3: 1; steps 0 / 1 / 2-done: seq++; 0.
# V-0027: frames 0 is drawn too (_u32_div_f returns the dividend for a zero divisor).
fn 021d2d7c ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 0x10 u32 alt const 0 || range 1 8 || range 9 0xFFFF
  fix arg2 0x14 u32 alt range 0 8 || any
  fix arg2 0x18 u32 addrof arg2 0x80
  fix arg2 8 u16 alt const 0 || range 1 4
  fix arg2 0xA u16 range 0 3
  fix arg2 0xC u16 range 0 3
  fix arg2 0xE u8 range 0 8
  fix arg2 0xF u8 range 0 3
  fix arg2 0 u16 alt range 0 4 || any
  fix arg2 2 u16 alt range 0 4 || any
  fix arg2 4 u16 alt range 0 4 || any
  fix arg2 6 u16 alt range 0 4 || any
  stub 020124e4 force ret=u32 args=v
  stub 0218ed04 force ret=none args=v,in:12
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 0218f0e8 force ret=pick:0/1 args=v
  stub 0218f0a8 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 0218ec5c force ret=ptr:arg2:0x80:1 args=v
  stub 0218ec30 force ret=none args=v,v
  stub 0218ec90 force ret=none args=v,in:12
  stub 0218ec54 force ret=none args=v
  stub 02188c58 force ret=u32 args=v
  stub 0218f05c force ret=none args=v

# cmd 574: a, b; o = 0216DE94(0215A5EC(ctx), a); 0216D88C(o, &pos); ev = 021F3CFC(GAMESYS, &pos, a, b); ev -> child event; 1.
fn 021d2edc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 0215a5ec force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 021f3cfc force ret=nullable:2:u32 args=v,in:12,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 575: ev = 021F37B8(GAMESYS); ev -> child event; 1.
fn 021d2f44 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f37b8 force ret=nullable:2:u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 597: id, a, b, c, d, e; o = 0216DE94(02188C7C(fm), id); o -> 0216D88C(o, &pos) (else pos stays the unset stack, 0 under
# Z19); ev = 021F3970(GAMESYS, &pos, &{a << 16 + 0x8000, b << 15, c << 16 + 0x8000}, d << 15, e); ev -> child event; 1.
fn 021d2f6c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02188c7c force ret=u32 args=v
  stub 021f3970 force ret=nullable:2:u32 args=v,in:12,in:12,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216de94 force ret=nullable:2:u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 598: a, b, c; 021F3A44(GAMESYS, &{a << 16 + 0x8000, b << 15, c << 16 + 0x8000}); 0.
fn 021d3018 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5dc force ret=u32 args=v
  stub 021f3a44 force ret=none args=v,in:12

# cmd 599: ev = 021F3ADC(GAMESYS); ev -> child event; 1.
fn 021d3068 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021f3adc force ret=nullable:2:u32 args=v

# cmd 600: ev = 021F3AC0(GAMESYS); ev -> child event; 1.
fn 021d3090 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021f3ac0 force ret=nullable:2:u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
