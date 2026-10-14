# fndiff spec: overlay-21 bsubway_scr.c (F8-spec5), Thumb code 0x021C9934..0x021CBFB0 (F1-SPLIT row 21.84; __FILE__ "bsubway_scr.c"
# 0x021DE58C passed by 0x021C9934 / 0x021CA4C0).  The Battle Subway script side: the work W (0x7EC bytes, magic 0x12345678; GAMEDATA slot
# 02012ED8 / 02012EE4), commands 343..346 and the helpers, GMEVENTs and wait callbacks of the big command 346.  The save side is
# bsubway_savedata.c in ARM9 (0x0200D85C..0x0200DFB4: play record P = save block 0x39, score record S = 0x3A, extra X = 0x3B; stubs).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_bsubway --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  343 021ca57c  344 021ca544  345 021ca598  346 021ca5c4
# 58 functions specced (4 command ids).
#  * 344 creates W (021C9934; resume from the play record or a new run of mode 0..8), 345 frees it (021C9B24), 343 calls 0x021C9924
#    (overlay 128, X1).  346 is one 0x1118-byte function (0x021CA5C4..0x021CB6DC) dispatching ~170 sub-commands (u16 operand; a binary
#    compare tree plus the jump tables 0x021CA6CC (0..0x71) and 0x021CAB40 (0x190..0x1A0)); MWCC reaches its far case bodies with `bl`
#    used as a long branch, so svflow's call check reports those targets (harness note: run `--strict-callees` with SVFLOW_ALLOW set to the
#    fragment list below; they are not callees).  Its block comment lists the stubbed callees of every sub-command.
#  * Wi-Fi / wireless multi (X1): comm_player.c (0x02167Bxx..0x02167Dxx), comm_entry_menu.c (0x021CD48C / 0x021CD5A4 / 0x021CD688), the
#    net layer (0x0203CDBC / 0x0203CEAC / 0x0203D1B4 / 0x0203D378 / 0x0203D7BC / 0x02039338) and 0x0200FB24 are force stubs; the GMEVENT
#    0x021CB8AC (wireless disconnect) and the waits 0x021CB6DC / 0x021CB700 / 0x021CB754 / 0x021CB7E8 / 0x021CB86C only poll them.
#  * Trainers: 021CBD90 picks a trainer id by level / round / mode from the ranges at 0x021DB3D0..0x021DB3F4 (RNG 021CBF98 = 020056EC(-1)
#    / 0xFFFF); 021CBED8 / 021CBF38 / 021CBD08 build them through fld_btl_inst_tool.c (ov10 0x021681C8 / 0x021684E8 / 0x02168878 /
#    0x021688BC, force stubs).  ROM tables: 0x021DACD0 (u8 level steps), 0x021DACDC (10-byte BP rows), 0x021DAD18 (BP by level), 0x021DAD24 /
#    0x021DAD36 (u16 by mode / by value), 0x021DAD48 / 0x021DAD5C (OBJ codes), 0x021DADD0 (5 x 8-byte NPC places), 0x021DADF8 (u32 levels),
#    0x021DAEBC (80 x 0x10-byte NPC records, 0xFFFF-terminated; +0xE nonzero only for records 25 / 47 / 51 / 55 / 79).
#  * Spec bounds: the work W is an `arg buf 0x1000` for the helpers and the ctx-arena zone 0x2000 (02012ED8 answers ptr:argC:0x2000) for
#    the commands (ctx `arg buf 0x4000`); W+8 (count) is 0..4 (the real party sizes), W+9 (mode) 0..9, W+0x7CC (battle setup) null or a
#    zone at ctx+0x3000.  The W-null early return of 346 (0x021CA668..0x021CA676: no W and a sub-command >= 0x12C) is not reached: a
#    null W with a smaller sub-command dereferences W, and the inputs cannot tie the two stubs (8 instructions; see the README).
# Not specced (0):
#   none
# Fragments (not functions; long-branch targets inside 346 and two other bodies): 0x021C9B7A, 0x021CAB72, 0x021CABC0, 0x021CAC20,
# 0x021CB01E..0x021CB6CA (55 case / tail labels), 0x021CBB6A; 0x021CB624 is a 346 case body.  Dead code: 26 `bl 0x021CB6CA` defaults of
# 346's compare tree (no sub-command value reaches them: every value 0..0x3FF was run), 0x021C9E36..0x021C9E38 (021C9DB0's jump-table
# entry for mode 4, which returns earlier).
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
area scrcmd_bsubway
set field

# bsub_create(GAMESYS, resume, mode): GD; party = 02012944(GD); save; W = Heap_AllocDebug(4, 0x7EC, 1, "bsubway_scr.c", 0x5F);
# W[4] = 4, W[0] = 0x12345678, +0x6C GD, +0xA = 02008550(party), +0x70 / 74 / 78 = save blocks 0x39 / 0x3A / 0x3B (020071CC);
# 0200D868(P, 0); 02012EE4(GD, W).  resume == 0: mode W+9, W+8 = 021CA270(mode), slots W+0x1E = 0xFF x4, ids W+0x32 = 0xFFFF x14,
# 0200D85C(P), 0200DA14(P), 0200DB44(S, mode) == 1 -> 021CA238(W, 0200DAC4(S, mode)); 0200D914(P, 0, &mode).  resume != 0: mode =
# 0200D884(P, 0), count, 0200D884(P, 0xA) != 0 -> 021CA1E4(W); P ids 5 / 8 into W; modes 2 / 7: k = 0200D884(P, 9) & 7 into W+0xC
# bits 5..7, 0200D884(P, 6, W+0x628+0x14k), 021CBF38(W, W+0x2C8+0x120k, 0x12F (or 0x12C when 02008550(020127A4(GD)+0x20)) + k,
# 0200D884(P, 7), W+0x628+0x14k, (u16)W[4]); S streak 0200DB44 == 1 -> 021CA258(W, 0200DB80(S, mode), 0200DA54(P)); W.  Arena: arg 3.
fn 021c9934 ret=u32 inputs=400
  nowork
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 alt range 0 9 || any
  arg buf 0x2000
  stub 0200db80 force ret=pick:0/1/2/3/0x14/0x31/0x32 args=v,v
  stub 0200d884 force ret=pick:0/1/2/7/8 args=v,v,ptr
  stub 0200da14 force ret=none args=v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 020127a4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg3:0x800:0x800 args=v,v,v,v,v
  stub 021cbf38 ret=none args=v,v,v,v,v,v
  stub 021ca238 ret=u32 args=v,v
  stub 02012ebc force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0200db44 force ret=pick:0/1 args=v,v
  stub 0200dac4 force ret=pick:0/1/2/0x270F/0x2710 args=v,v
  stub 020071cc force ret=u32 args=v,v
  stub 02008550 force ret=pick:0/1 args=v
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v
  stub 021ca258 ret=none args=v,v,v
  stub 0200d85c force ret=none args=v
  stub 02012ee4 force ret=none args=v,v
  stub 021ca1e4 ret=none args=v
  stub 0200d868 force ret=none args=v,v
  stub 021ca270 ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  # closure: 021cbf38 / 021ca238 / 021ca258 / 021ca1e4 / 021ca270
  stub 021684e8 force ret=none args=v,v,v,v,v,v,v,v
  stub 021688bc force ret=u32 args=v,v,v
  stub 021cbefc ret=u32 args=v
  stub 020307b0 force ret=none args=v
  stub 0200baf4 force ret=u32 args=v,v
  stub 0200bac8 force ret=u32 args=v

# bsub_free(GAMESYS, W): W != 0 -> W+0x7C8 != 0 -> Heap_Free, 0; W+0x7CC != 0 -> 020130B4 (battle setup free), 0; W cleared (0x7EC),
# Heap_Free(W); 02012EE4(GD, 0).
fn 021c9b24 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 alt const 0 || const 0x02E30000
  arg buf 0x1000
  fix arg2 0x7C8 u32 alt const 0 || any
  fix arg2 0x7CC u32 alt const 0 || any
  stub 02012ee4 force ret=none args=v,v
  stub 020307b0 force ret=none args=v
  stub 020130b4 force ret=none args=v
  stub 020120f4 force ret=u32 args=v

# bsub_mode_fix(W): mode 2 -> 3, 7 -> 8 (else the unset stack byte: 0 under Z19); W+9 = it; 0200D914(P, 0, &mode); 0200DB44(S, mode) ==
# 1 -> 021CA238(W, 0200DAC4(S, mode)); else 021CA250(W).
fn 021c9b78 ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  stub 0200dac4 force ret=pick:0/1/2/0x270F/0x2710 args=v,v
  stub 0200db44 force ret=pick:0/1 args=v,v
  stub 021ca250 ret=u32 args=v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 021ca238 ret=u32 args=v,v

# bsub_save(W): 0200D914(P, 0, &mode), (P, 5, W+0x1E), (P, 8, W+0x32); 0200D868(P, 1); modes 2 / 7: k = W+0xC bits 5..7: (P, 9, &k),
# (P, 6, W+0x628+0x14k), (P, 7, W+0x664+k).
fn 021c9bcc ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  stub 0200d868 force ret=none args=v,v
  stub 0200d914 force ret=none args=v,v,in:1

# bsub_load_party(W): W+8 = 021CA270(mode); 0200D884(P, 5, W+0x1E); party = 021CA210(W); for i < count: mon = 0201AB28(party,
# W+0x1E[i]); W+0x22+2i = species (02017E1C 5), W+0x2A+2i = item (6).
fn 021c9c54 ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 0201ab28 force ret=u32 args=v,v
  stub 021ca270 ret=pick:0/1/2/3/4 args=v
  stub 021ca210 ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  # closure: 021ca270 / 021ca210
  stub 02012934 force ret=u32 args=v

# bsub_save_round(W): 0200D914(P, 0, &mode); 0200DA14(P); (P, 5, W+0x1E); 0200D868(P, 1); modes 2 / 7: (P, 9, &k), (P, 6, ..), (P, 7, ..).
fn 021c9cac ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  stub 0200d868 force ret=none args=v,v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 0200da14 force ret=none args=v

# bsub_reset(GAMESYS): save; P, S; mode = 0200D884(P, 0) (u8); 0200DA14(P); 0200DB1C(S, mode); mode.
fn 021c9d30 ret=u8 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 0200db1c force ret=none args=v,v
  stub 020071cc force ret=u32 args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 0200da14 force ret=none args=v
  stub 02012ebc force ret=u32 args=v

# bsub_store_streak(-, W): n = W+0xE, mode, S: 0200DB08(S, mode, n), 0200DAEC(S, mode, n); 021CA2DC(W, save, 1, n); 0200DB1C(S, mode);
# 0200DA14(P).  Arena: arg 2.
fn 021c9d70 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x1000
  arg buf 0x400
  fix arg1 9 u8 alt range 0 9 || any
  stub 021ca2dc ret=none args=v,v
  stub 0200db1c force ret=none args=v,v
  stub 0200db08 force ret=none args=v,v,v
  stub 0200daec force ret=none args=v,v,v
  stub 0200da14 force ret=none args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 021ca2dc
  stub 021ca4c0 ret=u32 args=v,v,v
  stub 0200dc94 force ret=none args=v,v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 021ca210 ret=u32 args=v
  stub 0200dc50 force ret=none args=v,v,in:0xB4
  stub 020307b0 force ret=none args=v
  stub 021ca3b4 ret=none args=v,v
  stub 02030734 force ret=seqptr:arg2:0x100:0x100 args=v,v,v,v,v
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 02012934 force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# bsub_reward(W): mode 4: 0200D884(P, 0xB) == 1 -> r = 0200DC08(S) (s8, clamped 0..9), BP = table 0x021DAD18[r]; else 5; other modes:
# n = 0200DB80(S, mode) - 1 (s16, clamped 0..9); row by mode 0 / 1 / 2 / 3 / 5 / 6 / 7 / 8 = 0 / 2 / 4 / 4 / 1 / 3 / 5 / 5 (modes
# 5..8 set f; the jump-table entry for mode 4 is dead: mode 4 returned above); W+0xC bits 1..2 != 0 -> 0x1E (f) or 0xA; else table
# 0x021DACDC[10 * row + n]; 0 -> 1; 0200DA80(S, BP); BP.
fn 021c9db0 ret=u32 inputs=1000
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt oneof 4 8 || range 0 9 || any
  fix arg0 8 u8 range 0 4
  fix arg0 0xC u16 alt range 0 0x1F || any
  stub 0200dc08 force ret=pick:0/1/5/9/0xA/0xFF/0x80 args=v
  stub 0200db80 force ret=pick:0/1/2/9/10/11/0x8000 args=v,v
  stub 0200d884 force ret=pick:0/1/2 args=v,v,ptr
  stub 0200da80 force ret=none args=v,v

# bsub_pick_trainers(W): n = 021CA2CC(W+0xE) (streak / 7); modes 2 / 3 / 7 / 8: n = max(n, W+0x18); 14 ids W+0x32 from 021CBD90(W,
# n, i / 2, mode, i & 1) skipping repeats (021CA2A4); other modes: 7 ids from 021CBD90(W, n, i, mode, 0).  Spec: streak < 21 and
# W+0x18 < 3 (n < 3): 021CBD90 answers 0 for single modes with n >= 3, so the repeat loop would never end (closure run).
fn 021c9e8c ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 range 0 8
  fix arg0 8 u8 range 0 4
  fix arg0 0xE u16 range 0 20
  fix arg0 0x18 u16 range 0 2
  stub 021cbd90 ret=u16 args=v,v,v,v,v seq
  stub 021ca2a4 ret=pick:0/0/1 args=v,v,v
  stub 021ca2cc ret=pick:0/1/2/7 args=v
  stub 021ca234 ret=u16 args=v
  # closure: 021cbd90 / 021ca2a4 / 021ca2cc / 021ca234
  stub 021cbf98 ret=u16 args=
  stub 020056ec force ret=u32 args=v seq

# bsub_party_from_list(W): W+0x84 != 0 or (W+0x82 - 7) & 0xFFFF <= 1 -> 0; party; for i < count: slot = W+0x7C[i] - 1 (>= 6 -> W+0x7C[i] =
# 1 first); W+0x1E[i] = slot; species / item of mon slot into W+0x22 / W+0x2A; 1.
fn 021c9f44 ret=bool inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  fix arg0 0x84 u16 alt const 0 || any
  fix arg0 0x82 u16 alt range 0 10 || any
  fix arg0 0x7C u8 alt range 0 8 || any rep=4,1
  stub 021ca210 ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  # closure: 021ca210
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 02012934 force ret=u32 args=v

# bsub_check_rounds(W): W+0xC bit 0 set -> 1; 0200DA54(P) >= 7 -> set bit 0, 1; else 0.
fn 021c9fd0 ret=bool inputs=200
  nowork
  arg buf 0x1000
  fix arg0 0xC u16 alt range 0 3 || any
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v

# bsub_store_result(W): mode, n = W+0xE; S: 0200DB08(S, mode, n), 0200DAEC(S, mode, 0200DAC4(S, mode)), 0200DB60(S, mode);
# 021CA2DC(W, save, 1, n); 0200DA14(P).  Arena: arg 1 (the function takes W only).
fn 021c9ffc ret=none inputs=200
  nowork
  arg buf 0x1000
  arg buf 0x400
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  stub 0200db60 force ret=none args=v,v
  stub 021ca2dc ret=none args=v,v
  stub 0200dac4 force ret=pick:0/1/2/0x270F/0x2710 args=v,v
  stub 0200db08 force ret=none args=v,v,v
  stub 0200daec force ret=none args=v,v,v
  stub 021ca234 ret=u32 args=v
  stub 0200da14 force ret=none args=v
  stub 02012ebc force ret=u32 args=v
  # closure: 021ca2dc / 021ca234
  stub 021ca4c0 ret=u32 args=v,v,v
  stub 0200dc94 force ret=none args=v,v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 021ca210 ret=u32 args=v
  stub 0200dc50 force ret=none args=v,v,in:0xB4
  stub 020307b0 force ret=none args=v
  stub 021ca3b4 ret=none args=v,v
  stub 02030734 force ret=seqptr:arg1:0x100:0x100 args=v,v,v,v,v
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 02012934 force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# bsub_clear_flags(W): W+0xC &= ~1, then &= ~6.
fn 021ca048 ret=none inputs=200
  nowork
  arg buf 0x1000

# bsub_setup_opponents(W): r = 0200DA54(P); modes 2 / 3 / 7 / 8 (multi): 021CBED8(W, W+0x88, W+0x32[2r], count, 0, 0, 0, (u16)W[4]) and
# 021CBED8(W, W+0x1A8, W+0x32[2r+1], count, a, b, 0, ..) with the species / items of the partner records W+0xB8 + 0x3C i (stack
# arrays a / b); mode 4: 0200DEA8(X, W+0x88, r, (u16)W[4]); modes 0 / 1 / 5 / 6 and > 8: 021CBED8(W, W+0x88, W+0x32[r], count, 0, 0,
# 0, (u16)W[4]).
fn 021ca05c ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  stub 021cbed8 ret=u32 args=v,v,v,v,p,p,v,v
  stub 0200da54 force ret=pick:0/1/6 args=v
  stub 0200dea8 force ret=u32 args=v,v,v
  # closure: 021cbed8
  stub 02168878 force ret=u32 args=v,v,v,v,v,v,v

# bsub_rec_word(W, i): tail call 02168AEC(u16 W+0x8C + 0x120 i) (thunk `ldr r3, =0x02168AED; bx r3`).
fn 021ca150 ret=u32 inputs=200
  nowork
  arg buf 0x1000
  arg u32 range 0 7
  stub 02168aec force ret=none args=v

# bsub_level_step(S, -, k): l = 0200DC08(S) (u8); k 0 -> l; 3 -> 0200DBA0(S, 0, 2), l == 10 -> 0, else 0200DBF0(S), 1; 4 -> c =
# 0200DC0C(S, 3), l == 1 -> 0; c >= table 0x021DACD0[l - 1] -> 0200DBFC(S), 0200DC0C(S, 2), 0200DBA0(S, 0, 2), 1; else 0; other k -> 0.
fn 021ca164 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt oneof 0 3 4 || any
  stub 0200dba0 force ret=none args=v,v,v
  stub 0200dc08 force ret=pick:1/2/3/5/9/0xA args=v
  stub 0200dc0c force ret=pick:0/1/2/3/5/0xFF args=v,v
  stub 0200dbf0 force ret=none args=v
  stub 0200dbfc force ret=none args=v

# bsub_alloc_extra(W): W+0x7C8 != 0 -> Heap_Free; W+0x7C8 = 0200BAF4(0200BAC8(save), 4).
fn 021ca1e4 ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 0x7C8 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v
  stub 0200baf4 force ret=u32 args=v,v
  stub 02012ebc force ret=u32 args=v
  stub 0200bac8 force ret=u32 args=v

# bsub_party(W): 0200D884(P, 0xA) == 0 -> 02012934(GD); else W+0x7C8.
fn 021ca210 ret=u32 inputs=200
  nowork
  arg buf 0x1000
  stub 0200d884 force ret=pick:0/1 args=v,v,ptr
  stub 02012934 force ret=u32 args=v

# bsub_streak(W): W+0xE.
fn 021ca234 ret=u16 inputs=200
  nowork
  arg buf 0x1000

# bsub_set_streak(W, n): W+0xE = n.
fn 021ca238 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u16 any

# bsub_inc_streak(W): W+0xE < 0xFFFF -> += 1.
fn 021ca23c ret=none inputs=200
  nowork
  arg buf 0x1000
  fix arg0 0xE u16 alt near 0xFFFF 1 || any

# bsub_zero_streak(W): W+0xE = 0.
fn 021ca250 ret=none inputs=200
  nowork
  arg buf 0x1000

# bsub_streak_from(W, rounds, n): W+0xE = min(7 * rounds + n, 0xFFFF).
fn 021ca258 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 alt range 0 0x2500 || any
  arg u32 alt range 0 10 || any

# bsub_party_size(mode): 0 / 4 / 5 -> 3; 1 / 6 -> 4; 2 / 3 / 7 / 8 -> 2 (jump table); else 0.
fn 021ca270 ret=u8 inputs=200
  nowork
  arg u32 alt range 0 9 || any

# u16_in(list, v, n): v among the first n u16s of list.
fn 021ca2a4 ret=bool inputs=200
  nowork
  arg buf 0x40
  arg u16 alt range 0 3 || any
  arg u16 range 0 14
  fix arg0 0 u16 range 0 3 rep=14,2

# div7(n): n / 7 (_s32_div_f).
fn 021ca2cc ret=u16 inputs=200
  nowork
  arg u32 alt range 0 100 || any

# bsub_record(W, save, f, n): mode 0 -> 021CA4C0(W, save, 0); 4 -> 021CA4C0(W, save, 1), 0200D914(P, 0, &mode), 0200D914(P, 1,
# &(0200DA54(P) + 1)), 0200DC94(S, P); 1 / others -> nothing.  Arena: arg 4 (a stack word the function does not read).
fn 021ca2dc ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x400
  fix arg0 9 u8 alt oneof 0 1 4 || any
  stub 021ca4c0 ret=u32 args=v,v,v
  stub 0200dc94 force ret=none args=v,v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v
  # closure: 021ca4c0
  stub 0201ab28 force ret=u32 args=v,v
  stub 021ca210 ret=u32 args=v
  stub 0200dc50 force ret=none args=v,v,in:0xB4
  stub 020307b0 force ret=none args=v
  stub 021ca3b4 ret=none args=v,v
  stub 02030734 force ret=seqptr:arg4:0x100:0x100 args=v,v,v,v,v
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 02012934 force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# bsub_opp_records(W, f): base = 0x12C (+3 if f); for i < 3: W+0x664+i = 021CBED8(W, W+0x2C8 + 0x120 i, base + i, count, W+0x22,
# W+0x2A, W+0x628 + 0x14 i, (u16)W[4]).
fn 021ca338 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 alt const 0 || any
  stub 021cbed8 ret=u8 args=v,v,v,v,in:8,in:8,v,v
  # closure: 021cbed8
  stub 02168878 force ret=u32 args=v,v,v,v,v,v,v

# bsub_mon_record(rec, mon): rec+0 = species (11 bits) | form (0x6F) << 11; +2 item (6); moves 0x36.. into +4 (u16 x4), pp-ups 0x3E..
# into the bits of +0x1E; +0x1F friendship (0xC); +0xC id (7); +0x10 (0); +0x14 (0xAC); IVs / EVs 0xD.. into +0x18..0x1D; +0x20
# ability (0xA); +0x21 nature (9); 02017E1C(mon, 0x74, rec + 0x22) (nickname).
fn 021ca3b4 ret=none inputs=200
  nowork
  arg buf 0x40
  arg u32 any
  stub 02017e1c force ret=u32 args=v,v,v

# bsub_save_party(W, save, k): R = Heap_AllocDebug((u16)W[4] & 0x7FFF | 0x8000, 0xB4, 0, "bsubway_scr.c", 0x89A) cleared; party; for
# i < 3: 021CA3B4(R + 0x3C i, 0201AB28(party, W+0x1E[i])); 0200DC50(S, k, R); R cleared, Heap_Free.  Arena: arg 3.
fn 021ca4c0 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg buf 0x400
  fix arg0 0x1E u8 range 0 5 rep=4,1
  stub 0201ab28 force ret=u32 args=v,v
  stub 021ca210 ret=u32 args=v
  stub 0200dc50 force ret=none args=v,v,in:0xB4
  stub 020307b0 force ret=none args=v
  stub 021ca3b4 ret=none args=v,v
  stub 02030734 force ret=seqptr:arg3:0x100:0x100 args=v,v,v,v,v
  # closure: 021ca210 / 021ca3b4
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 02012934 force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 344: GAMESYS = 02158F54(SW); resume = u16 operand; mode = value; 021C9934(GAMESYS, resume, mode); 0.
fn 021ca544 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02011330 force ret=u16 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 021c9934 ret=none args=v,v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  # closure: 021c9934
  stub 0200db80 force ret=pick:0/1/2/3/0x14/0x31/0x32 args=v,v
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 0200da14 force ret=none args=v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 020127a4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x2800:0x800 args=v,v,v,v,v
  stub 021cbf38 ret=none args=v,v,v,v,v,v
  stub 021ca238 ret=u32 args=v,v
  stub 02012ebc force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 0200db44 force ret=pick:0/1 args=v,v
  stub 0200dac4 force ret=pick:0/1/2/0x270F/0x2710 args=v,v
  stub 020071cc force ret=u32 args=v,v
  stub 02008550 force ret=pick:0/1 args=v
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v
  stub 021ca258 ret=none args=v,v,v
  stub 0200d85c force ret=none args=v
  stub 02012ee4 force ret=none args=v,v
  stub 021ca1e4 ret=none args=v
  stub 0200d868 force ret=none args=v,v
  stub 021ca270 ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021684e8 force ret=none args=v,v,v,v,v,v,v,v
  stub 021688bc force ret=u32 args=v,v,v
  stub 021cbefc ret=u32 args=v
  stub 020307b0 force ret=none args=v
  stub 0200baf4 force ret=u32 args=v,v
  stub 0200bac8 force ret=u32 args=v

# cmd 343: 021C9924(02158F54(SW)) (overlay-128 X1 helper); 0.
fn 021ca57c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021c9924 force ret=none args=v
  stub 02158f54 force ret=u32 args=v

# cmd 345: 021C9B24(GAMESYS, 02012ED8(GD)) (bsub_free); 0.
fn 021ca598 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 021c9b24 ret=none args=v,v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  # closure: 021c9b24
  stub 02012ee4 force ret=none args=v,v
  stub 020307b0 force ret=none args=v
  stub 020130b4 force ret=none args=v

# cmd 346 (vm, ctx): SW, GAMESYS = 02158F54(SW), GD, save, party = 0201296C(GD), W = 02012ED8(GD), P / S / X = save blocks
# 0x39 / 0x3A / 0x3B (020071CC), mode = 0200D884(P, 0), fm = 0201210C(GAMESYS); operands: sub = u16 (02011330), a, b = values
# (02159B10), var = 02158FE0(SW, GD, u16 operand); W == 0 and sub >= 0x12C -> 0.  ~170 sub-commands (a compare tree over sub;
# jump tables 0x021CA6CC for 0..0x71 and 0x021CAB40 for 0x190..0x1A0; MWCC long branches with `bl`): most store a result in
# *var (u16) and return 0; the event / wait ones (child events 0215897C, waits 020113D0 with the callbacks 0x021CB6DD /
# 0x021CB701 / 0x021CB755 / 0x021CB7E9 / 0x021CB86D / 0x021CBC15) return 1.  Sub-command (hex) -> the callees it reaches beyond
# the common getters above (plain run of the spec, first-call order; `-` = W fields / *var only):
# 0: 021CBCC4 | 1: 02087BCC | 2: 0200D85C | 3: 0200D87C | 4: 02188CB4 021A3050 021A3038 02188C9C 02014184 0201289C 02014314
# 5: 0201433C | 6: 0200DAC4 | 7: 021C9D30 | 8: 0200DA54 | B: 0200DA54 | C: 02188CB4 021A30B0 021A3190
# D: 02188C7C 0216DE94 0216DAC8 | E: 0200DBA0 | F: 0200DBA0 | 10: 0200DBA0 | 11: 0200DBA0 | 12: 021F37C0 | 13: 021F380C 021D024C
# 14: 021F380C 021D028C | 15: - | 16: - | 17: 02188C7C 0216DE94 0216D88C 0216D5F8 0216E014 0216DB4C
# 18: 02188C7C 0216DE94 0216DB4C | 19: 02188C7C 0216DE94 | 1A: 02188C7C 0216DE94 021CBBA0 | 1B: 021CA164 | 1C: 021CA164
# 1D: 021CA164 | 1E: 0200DB80 | 1F: 020127A4 02008550 021A31A4 | 20: 0200BAC8 0200BBA8 | 21: - | 22: 021F37C0
# 23: 02188C7C 0216DE94 0216D5E0 | 24: 02188C7C 0216DE94 0216DAF8 | 25: 02188C7C 0216DE94 0216D63C 0216CB48 0215897C
# 26: 0200DB44 | 27: 02188C7C 0216DE94 0216D63C 021CBBAC | 28: 02188C7C 0216DE94 0216D63C 021CBBD0
# 29: 02188C7C 0216DE94 0216D63C 021CBBD0 | 2A: 02188C7C 0216DE94 0216D63C 021CBBD0 | 2B: 0200DF58 0200DF94 020307B0
# 2C: 021CB9BC 0215897C | 2D: 021CBCC4 02014184 0201289C 02014314 | 64: 0200DBA0 | 65: 0200DBA0 | 67: 020124AC 0215897C
# 68: 020124AC 0215897C | 69: 020124AC 0215897C | 6A: 0200DE94 | 6B: 0200DE5C | 6C: 0200D914 | 6D: - | 6E: 0200DE64
# 6F: 0216C860 0215897C | 70: 0200DF40 | 71: 0200DF4C | C8: - | CB: - | 12C: 0216C538 0215897C | 12D: 021C9F44 | 12E: -
# 12F: 021C9FD0 | 130: 021C9D70 | 131: 021C9FFC | 132: 021C9BCC | 133: 021CA05C | 134: 021CA150 | 135: 0216C640 0215897C | 136: -
# 137: - | 138: - | 139: 021C9DB0 | 13A: - | 13B: - | 13C: 021C9E8C | 13D: - | 13F: - | 140: 0216C6E8 0215897C | 141: 021C9CAC
# 142: 02014184 0201289C 02014314 | 143: 021C9C54 | 144: 0200D914 | 145: 021C9B78 | 146: - | 149: 02188C7C 021CB9E4 | 14A: -
# 14B: - | 14C: - | 14D: 021CA048 | 14E: 0200DA54 02012134 0202B578 0202C0D0 | 14F: 020127A4 02008550 021CA338 | 150: - | 151: -
# 152: - | 153: 021CBC94 0201A354 0200BAC8 0200BAF4 02019ED4 020307B0 02012934 | 154: 021CA1E4 | 155: 021CBC94 | 156: 021CA270
# 157: 0200B5CC | 158: - | 159: 0200BA58 021EEE78 0200BA68 | 15A: 020113D0 | 15B: 0200B424 020130B4
# 15C: 021CA234 02014808 0215897C | 15D: 0200DA14 0200DB1C | 15E: 021F388C | 15F: 0200DA54 | 160: 0216C6E8 0215897C
# 161: 0201AA18 0200D9E8 0201AB28 02017E1C | 162: - | 163: - | 164: - | 165: 0200DA28 021CA23C 021CA234 | 166: 021CA234
# 167: 0200DB44 | 168: - | 169: - | 16A: - | 16B: - | 16C: - | 16D: - | 16E: - | 16F: - | 170: - | 171: - | 172: - | 173: -
# 174: - | 175: - | 176: - | 177: - | 178: - | 179: - | 17A: - | 17B: - | 17C: - | 17D: - | 17E: - | 17F: - | 180: - | 181: -
# 182: - | 183: - | 184: - | 185: - | 186: - | 187: - | 188: - | 189: - | 18A: - | 18B: - | 18C: - | 18D: - | 18E: - | 18F: -
# 190: 02167B00 02167AB0 | 191: 021CB914 0215897C | 192: 02167B24 020113D0 | 193: 02012944 021CD48C 020113D0
# 194: 02012944 021CD48C 020113D0 | 195: 02167BFC 020113D0 | 196: 02167CF4 020113D0 | 197: 02039338 | 198: 02014808 0215897C
# 199: - | 19A: 0200ED48 0200ED54 | 19B: 02039338 021A3190 02188CB4 021A30B0 | 19C: 02039338 02188CB4 021A30B0 021A3190
# 19D: 0200FB38 | 19E: 02012124 | 19F: 0200FB24 | 1A0: 0200BA38
fn 021ca5c4 ret=u32 inputs=4000
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  fix arg1 0x27CC u32 alt const 0 || addrof arg1 0x3000
  fix arg1 0x308C u32 alt const 1 || any
  fix arg1 0x27C0 u32 alt range 0xB 0xD || any
  stub 02014808 force ret=u32 args=v,v,v,v,v,v,v
  stub 021f380c force ret=u32 args=v
  stub 0200da14 force ret=none args=v
  stub 0216e014 force ret=none args=v,in:12,v
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  stub 0200b424 force ret=none args=
  stub 0200da28 force ret=u32 args=v
  stub 021ca234 ret=u32 args=v
  stub 0200ba38 force ret=u32 args=v
  stub 021a3038 force ret=none args=v,out:12
  stub 021ca23c ret=none args=v
  stub 0216d63c force ret=pick:0/1/2/25/47/51/55/79 args=v,v
  stub 0216c640 force ret=none args=v,v,v
  stub 021ca048 ret=none args=v
  stub 021d024c force ret=u32 args=v,v
  stub 021ca250 ret=u32 args=v
  stub 021a3050 force ret=pick:0/1/2/3/4 args=v
  stub 0200da54 force ret=pick:0/1/6/7/8/0x14 args=v
  stub 021c9c54 ret=none args=v
  stub 0200ba58 force ret=none args=
  stub 021ca05c ret=none args=v
  stub 0200de5c force ret=none args=v
  stub 0200d85c force ret=none args=v
  stub 0216c860 force ret=none args=v
  stub 0200de64 force ret=none args=v
  stub 0200ba68 force ret=none args=v
  stub 021ca270 ret=u32 args=v
  stub 021eee78 force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v
  stub 0200d87c force ret=none args=v
  stub 0200d884 force ret=pick:0/1/2/3/4/5/6/7/8/9 args=v,v,ptr
  stub 021f388c force ret=none args=v
  stub 021cd48c force ret=u32 args=v,v,v,v,v,v,v,v
  stub 021c9e8c ret=none args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 021d028c force ret=u32 args=v,v
  stub 021cbc94 ret=u32 args=v
  stub 0200de94 force ret=none args=v
  stub 0216de94 force ret=nullable:4:u32 args=v,v
  stub 0201289c force ret=none args=v,in:0x1C
  stub 02188c9c force ret=u32 args=v
  stub 021c9cac ret=none args=v
  stub 020124ac force ret=u32 args=v,v,v,in:16
  stub 021a30b0 force ret=u32 args=v
  stub 02167ab0 force ret=none args=v
  stub 020130b4 force ret=none args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  stub 021cbcc4 ret=u32 args=v
  stub 0200dac4 force ret=pick:0/1/2/0x270F/0x2710 args=v,v
  stub 0200bac8 force ret=u32 args=v
  stub 0216dac8 force ret=u32 args=v,v
  stub 0202c0d0 force ret=none args=v,v
  stub 02019ed4 force ret=pick:0/1/2 args=v,v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v
  stub 0216c6e8 force ret=none args=v,v,v,v,v
  stub 021c9ffc ret=none args=v
  stub 020120f4 force ret=u32 args=v
  stub 0200baf4 force ret=u32 args=v,v
  stub 02167cf4 force ret=none args=v,v
  stub 0216daf8 force ret=u32 args=v
  stub 02167b00 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4/5/6/7/8/9/10/25/47 args=v,v seq
  stub 021cb914 ret=none args=v,v
  stub 02014314 force ret=none args=v,v
  stub 0200d914 force ret=none args=v,v,in:1
  stub 0200db1c force ret=none args=v,v
  stub 0200fb24 force ret=u32 args=v
  stub 02012124 force ret=none args=v,v
  stub 02167b24 force ret=none args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02011330 force ret=nth:pick:0x0/0x1/0x2/0x3/0x4/0x5/0x6/0x7/0x8/0x9/0xB/0xC/0xD/0xE/0xF/0x10/0x11/0x12/0x13/0x14/0x15/0x16/0x17/0x18/0x19/0x1A/0x1B/0x1C/0x1D/0x1E/0x1F/0x20/0x21/0x22/0x23/0x24/0x25/0x26/0x27/0x28/0x29/0x2A/0x2B/0x2C/0x2D/0x2E/0x64/0x65/0x67/0x68/0x69/0x6A/0x6B/0x6C/0x6D/0x6E/0x6F/0x70/0x71/0x72/0xC8/0xC9/0xCB/0x12B/0x12C/0x12D/0x12E/0x12F/0x130/0x131/0x132/0x133/0x134/0x135/0x136/0x137/0x138/0x139/0x13A/0x13B/0x13C/0x13D/0x13E/0x13F/0x140/0x141/0x142/0x143/0x144/0x145/0x146/0x147/0x149/0x14A/0x14B/0x14C/0x14D/0x14E/0x14F/0x150/0x151/0x152/0x153/0x154/0x155/0x156/0x157/0x158/0x159/0x15A/0x15B/0x15C/0x15D/0x15E/0x15F/0x160/0x161/0x162/0x163/0x164/0x165/0x166/0x167/0x168/0x169/0x16A/0x16B/0x16C/0x16D/0x16E/0x16F/0x170/0x171/0x172/0x173/0x174/0x175/0x176/0x177/0x178/0x179/0x17A/0x17B/0x17C/0x17D/0x17E/0x17F/0x180/0x181/0x182/0x183/0x184/0x185/0x186/0x187/0x188/0x189/0x18A/0x18B/0x18C/0x18D/0x18E/0x18F/0x190/0x191/0x192/0x193/0x194/0x195/0x196/0x197/0x198/0x199/0x19A/0x19B/0x19C/0x19D/0x19E/0x19F/0x1A0/0x1A1/0x1F5/0xFFFF;u16 args=v seq
  stub 0200db30 force ret=none args=v,v
  stub 021c9d30 ret=none args=v
  stub 02012134 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 021ca338 ret=none args=v,v
  stub 0200fb38 force ret=none args=v
  stub 02039338 force ret=pick:0/1 args=
  stub 0216c538 force ret=none args=v,v,v
  stub 0201433c force ret=none args=v,v
  stub 0200df40 force ret=u32 args=v
  stub 0200db44 force ret=pick:0/1 args=v,v
  stub 02012944 force ret=u32 args=v
  stub 021c9f44 ret=none args=v
  stub 0200ed48 force ret=u32 args=v
  stub 0216cb48 force ret=none args=v,v,v
  stub 0200df4c force ret=u32 args=v
  stub 0216db4c force ret=u32 args=v,v
  stub 02008550 force ret=pick:0/1 args=v
  stub 021ca150 ret=none args=v,v
  stub 02158f54 force ret=u32 args=v
  stub 0201a354 force ret=u32 args=v,v
  stub 0200ed54 force ret=none args=v,v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 0200df58 force ret=ptr:arg1:0x3400:1 args=v,v
  stub 021ca164 ret=u32 args=v,v,v
  stub 0201296c force ret=u32 args=v
  stub 021c9d70 ret=none args=v,v
  stub 0202b578 force ret=u32 args=v
  stub 021c9b78 ret=none args=v
  stub 0215897c force ret=none args=v,v
  stub 0200db80 force ret=pick:0/1/2/3/0x14/0x31/0x32 args=v,v
  stub 02014184 force ret=none args=out:0x1C,v,v,v,v,v
  stub 021a3190 force ret=u32 args=v,v
  stub 0200df94 force ret=pick:0/1/2/5/6 args=v
  stub 0200dba0 force ret=none args=v,v,v
  stub 021cbba0 ret=none args=v
  stub 020127a4 force ret=u32 args=v
  stub 021a31a4 force ret=none args=v,v
  stub 0200bba8 force ret=pick:0/1 args=v
  stub 021cbbac ret=pick:0/1 args=v,v
  stub 020307b0 force ret=none args=v
  stub 021c9db0 ret=none args=v
  stub 021cb9bc ret=none args=v,v,v
  stub 021f37c0 force ret=u32 args=
  stub 020071cc force ret=u32 args=v,v
  stub 0200b5cc force ret=pick:0/1 args=v,v,v,v
  stub 021c9bcc ret=none args=v
  stub 020113d0 force ret=none args=v,v
  stub 021c9fd0 ret=pick:0/1 args=v
  stub 021cbbd0 ret=none args=v,v,v,v,v
  stub 02087bcc force ret=u32 args=v
  stub 02158fe0 force ret=ptr:arg1:0x600:0x80 args=v,v,v
  stub 0216d5e0 force ret=u32 args=v,v
  stub 021ca1e4 ret=none args=v
  stub 021cb9e4 ret=none args=v,v,v,v
  stub 0200d9e8 force ret=u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0216d5f8 force ret=u16 args=v
  stub 02167bfc force ret=none args=v,v,v
  # closure: 021ca234 / 021ca23c / 021ca048 / 021ca250 / 021c9c54 / 021ca05c / 021ca270 / 021c9e8c / 021cbc94 / 021c9cac / 021cbcc4 / 021c9ffc / 021cb914 / 021c9d30 / 021ca338 / 021c9f44 / 021ca150 / 021ca164 / 021c9d70 / 021c9b78 / 021cbba0 / 021cbbac / 021c9db0 / 021cb9bc / 021c9bcc / 021c9fd0 / 021cbbd0 / 021ca1e4 / 021cb9e4
  stub 021ca210 ret=u32 args=v
  stub 021cbed8 ret=u32 args=v,v,v,v,v,v,v,v
  stub 0200dea8 force ret=u32 args=v,v,v
  stub 021cbd90 ret=u32 args=v,v,v,v,v
  stub 021ca2a4 ret=u32 args=v,v,v
  stub 021ca2cc ret=u32 args=v
  stub 0200d868 force ret=none args=v,v
  stub 0200db60 force ret=none args=v,v
  stub 021ca2dc ret=none args=v,v
  stub 0200db08 force ret=none args=v,v,v
  stub 0200daec force ret=none args=v,v,v
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 02168aec force ret=none args=v
  stub 0200dc08 force ret=pick:0/1/2/3/5/9/0xA/0xB args=v
  stub 0200dc0c force ret=pick:0/1/2/3/5/0xFF args=v,v
  stub 0200dbf0 force ret=none args=v
  stub 0200dbfc force ret=none args=v
  stub 021ca238 ret=u32 args=v,v
  stub 0200da80 force ret=none args=v,v
  stub 0216d620 force ret=none args=v,v,v
  stub 0216ce6c force ret=u32 args=v,v,v,v,v,v,v,v
  stub 0200dfb4 force ret=u32 args=v
  stub 0216d5b4 force ret=none args=v,v
  stub 02168878 force ret=u32 args=v,v,v,v,v,v,v
  stub 021cbf98 ret=u16 args=
  stub 021ca4c0 ret=u32 args=v,v,v
  stub 0200dc94 force ret=none args=v,v
  stub 020056ec force ret=u32 args=v seq
  stub 0200dc50 force ret=none args=v,v,in:0xB4
  stub 021ca3b4 ret=none args=v,v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v

# wait of cmd 346 (vm, ctx): 02167CB4(W) == 1 (comm_player.c, X1).
fn 021cb6dc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 02167cb4 force ret=pick:0/1/2 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v

# wait of cmd 346: var = 02158FE0(SW, GD, W+0x72A); 0200FB24() != 0 -> *var = 0, 1; 02167B34(W+0x727) == 1 -> *var = 1, 1; else 0.
fn 021cb700 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 02158fe0 force ret=ptr:arg1:0x600:0x80 args=v,v,v
  stub 0200fb24 force ret=pick:0/1 args=
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02167b34 force ret=pick:0/1 args=v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v

# wait of cmd 346 (entry menu): ...; M = W+0x71C; r = 021CD688(M) (comm_entry_menu.c, X1); r != 0 -> 021CD5A4(M), W+0x71C = 0; r 1 ->
# *W+0x720 = 0; 2 -> 1; else (4 / 3) -> 2; 1; r == 0 -> 0.
fn 021cb754 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  fix arg1 0x2720 u32 addrof arg1 0x780
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 021cd5a4 force ret=none args=v
  stub 021cd688 force ret=pick:0/1/2/3/4 args=v
  stub 0201296c force ret=u32 args=v
  stub 020071cc force ret=u32 args=v,v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02012ebc force ret=u32 args=v

# wait of cmd 346 (entry menu, two results): r = 021CD688(M); r != 0 -> free, r 1 -> *W+0x720 = 0, else 1; 1; r == 0 -> 0.
fn 021cb7e8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  fix arg1 0x2720 u32 addrof arg1 0x780
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 021cd5a4 force ret=none args=v
  stub 021cd688 force ret=pick:0/1/2/3 args=v
  stub 0201296c force ret=u32 args=v
  stub 020071cc force ret=u32 args=v,v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:0x10 args=v
  stub 02012ebc force ret=u32 args=v

# wait of cmd 346: var = 02158FE0(SW, GD, W+0x72A); 02167D00(W, var) == 1.
fn 021cb86c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 02158fe0 force ret=ptr:arg1:0x600:0x80 args=v,v,v
  stub 02167d00 force ret=pick:0/1 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v

# GMEVENT of 021CB914 (event, seq, E = {W*}): seq 0: 0203D378() == 1 -> 0203D7BC(0); W+0xC bit 9 -> 02167B18(W); seq 1; seq 1:
# 0203D378() == 1 -> 0203D1B4() <= 1 -> 0203CEAC(0), seq 2; else (not connected) the same; seq 2: 0203CDBC() == 1 -> 1; else 0 (wireless
# disconnect, X1).
fn 021cb8ac ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 0 u32 addrof arg2 0x20
  stub 0203ceac force ret=none args=v
  stub 02167b18 force ret=none args=v,v
  stub 0203d1b4 force ret=pick:0/1/2 args=
  stub 0203d378 force ret=pick:0/1 args=
  stub 0203cdbc force ret=pick:0/1 args=
  stub 0203d7bc force ret=none args=v

# bsub_disconnect_event(GAMESYS, W): ev = 020122C0(GAMESYS, 0, 0x021CB8AD, 4); *020124E8(ev) = W; ev.  Arena: arg 2.
fn 021cb914 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v

# GMEVENT of 021CB9BC (event, seq, E = {ctx, var, X, k}): 0215A5F4(ctx); seq 0: 0215A684(ctx, 8), seq 1; seq 1: 0215A690(ctx) == 1 ->
# X = 02190BE4(02188C54(fm), k), seq 2; seq 2: r = 02190C78(X): 0 -> *var = 0, seq 3; 2 -> wait; else *var = 1, seq 3; seq 3:
# 021909BC(X), 1; else 0 (a yes / no style window, UI).
fn 021cb934 ret=u32 inputs=200
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
  stub 02190c78 force ret=pick:0/1/2 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 021909bc force ret=none args=v

# bsub_window_event(GAMESYS, ctx, var, k): ev = 020122C0(GAMESYS, 0, 0x021CB935, 0x10); E = {ctx, var, -, k}; ev.  Arena: arg 4.
fn 021cb9bc ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg4:0:1 args=v

# bsub_place_npcs(W, GD, S, fm): z = 02188C9C(fm); mode 4: records 0x021DADD0 ({s16 x, -, s16 z, ..} x 5) for the up-to-5 entries of
# 0200DF94(L = 0200DF58(save block 0x3B, 4)) from the last: g = 0200DFB4(e) % 10, OBJ code table 0x021DAD48 / 0x021DAD5C [g] by
# e+0x20 bit 1; M = 0216CE6C(S, x, z, dir(i), id 0x80 + i, code, 2, z) (MMDL add), 0216D620(M, n - 1, 0), 0216D5B4(M, 5); Heap_Free(L).
# Other modes: r = 0200DB80(S, mode) (+3 for modes 5..8); every record of 0x021DAEBC (0x10 bytes, ends at 0xFFFF) whose u32
# 0x021DADF8[rec+2 - 1] == r adds an OBJ the same way (code rec[0], 0216D620 twice, ids from 0x80).
fn 021cb9e4 ret=none inputs=400
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 9 u8 alt oneof 4 0 5 8 || any
  stub 0216d620 force ret=none args=v,v,v
  stub 0200db80 force ret=pick:0/1/2/3/4/5/6/7/8/9/10 args=v,v
  stub 020071cc force ret=u32 args=v,v
  stub 0216ce6c force ret=u32 args=v,v,v,v,v,v,v,v
  stub 020307b0 force ret=none args=v
  stub 02188c9c force ret=u32 args=v
  stub 0200df94 force ret=pick:0/1/2/5/6 args=v
  stub 0200dfb4 force ret=u32 args=v
  stub 0216d5b4 force ret=none args=v,v
  stub 0200df58 force ret=ptr:arg0:0x800:1 args=v,v
  stub 02012ebc force ret=u32 args=v

# bsub_obj_code(M, v): tail call 0216D63C(M, 1, v) (`ldr r3, =0x0216D63D; movs r1, #1; bx r3`).
fn 021cbba0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  stub 0216d63c force ret=pick:0/1/2/25/47/51/55/79 args=v,v

# bsub_npc_has_battle(W, i): mode 4 -> 0; else record 0x021DAEBC[i] +0xE != 0.
fn 021cbbac ret=bool inputs=200
  nowork
  arg buf 0x1000
  arg u32 alt oneof 25 47 51 55 79 || range 0 79
  fix arg0 9 u8 alt oneof 4 0 || any

# bsub_npc_info(W, i, u16 *a, u16 *b, u16 *c) (5th argument on the stack): mode 4 or record +0xE == 0 -> *a = *b = 0, *c = 4; else
# *a = rec+0xA, *b = rec+0xC, *c = rec+0xE.
fn 021cbbd0 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 alt oneof 25 47 51 55 79 || range 0 79
  arg buf 0x10
  arg buf 0x10
  arg buf 0x10
  fix arg0 9 u8 alt oneof 4 0 || any

# wait callback of cmd 346 (vm, ctx): W; v = min(W+0x10, 9999); kind = 0x14 / 0x15 / 0x16 by mode (as 021CBC94); 0200B6E0(GD, 4,
# kind, v, W+0x7E8, W+0x7EA) - 2 <= 1 (unsigned) -> 1; else 0.
fn 021cbc14 ret=bool inputs=200
  nowork
  arg buf 0x800
  arg buf 0x4000
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x2009 u8 range 0 8
  fix arg1 0x2008 u8 range 0 4
  fix arg1 0x200E u16 range 0 20
  fix arg1 0x2018 u16 range 0 2
  stub 0200b6e0 force ret=pick:0/1/2/3/4 args=v,v,v,v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f54 force ret=u32 args=v
  stub 02012ed8 force ret=ptr:arg1:0x2000:1 args=v

# bsub_kind(mode): modes 1 / 6 -> 0x15, 2 / 3 / 7 / 8 -> 0x16, 0 / 4 / 5 and > 8 -> 0x14.
fn 021cbc94 ret=u32 inputs=200
  nowork
  arg u32 alt range 0 9 || any

# bsub_line(fm): z = 02188C9C(fm) - 0x43: 0 -> 0, 1 -> 5, 2 -> 1, 3 -> 6, 4 -> 2, 5 -> 7, 6 -> 4; else 0 (train line of the zone).
fn 021cbcc4 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 02188c9c force ret=pick:0x42/0x43/0x44/0x45/0x46/0x47/0x48/0x49/0x4A args=v

# bsub_battle_setup(W, h): P = 0201A920(0x8004) (a temp party); party; 0201A954(P, count); 0201A98C(P, mon) for the count slots;
# B = 021681C8(h, P, mode, W+0x88, W+0x2C8 + 0x120 k, count) (fld_btl_inst_tool.c); Heap_Free(P); B.
fn 021cbd08 ret=u32 inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 9 u8 alt range 0 9 || any
  fix arg0 8 u8 range 0 4
  fix arg0 0x1E u8 range 0 5 rep=4,1
  stub 0201a920 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 021681c8 force ret=u32 args=v,v,v,v,v,v
  stub 0201a98c force ret=none args=v,v
  stub 021ca210 ret=u32 args=v
  stub 020307b0 force ret=none args=v
  stub 0201a954 force ret=none args=v,v
  # closure: 021ca210
  stub 0200d884 force ret=u32 args=v,v,ptr
  stub 02012934 force ret=u32 args=v

# bsub_rec_ptr(W): W + 0x74C.
fn 021cbd84 ret=u32 inputs=200
  nowork
  arg buf 0x1000

# bsub_trainer_id(W, n, round, mode, multi) (5th argument on the stack): d = mode >= 5; round 6 is the boss round when n == 2 (d 0)
# or n == 6 (d 1): b = 0 / 1; boss: modes 0 / 5 -> 0x132 + b, 1 / 6 -> 0x134 + b, 2 / 3 / 7 / 8 -> 0x137 / 0x136 (b 0, multi 0 / 1)
# or 0x139 / 0x138 (b 1), 4 -> 0.  Otherwise: d 0: n < 3 -> table 0x021DB3D8[n] (round < 6) or 0x021DB3D0[n] (round 6, n < 2)
# else 0; d 1: n < 4 -> 0x021DB3E4[n] / 0x021DB3F4[n] (round < 6 / 6); n >= 4 -> 0xC8 + rand % 100; a table entry {lo, hi} gives
# lo + rand % (hi - lo + 1) (rand = 021CBF98, _s32_div_f remainder).
fn 021cbd90 ret=u16 inputs=1500
  nowork
  arg buf 0x1000
  arg u32 alt oneof 2 6 || range 0 8 || any
  arg u32 alt oneof 6 || range 0 7
  arg u32 alt range 0 8 || any
  arg u8 range 0 1
  stub 021cbf98 ret=u16 args=
  # closure: 021cbf98
  stub 020056ec force ret=u32 args=v seq

# bsub_make_trainer(W, rec, id, count, a, b, c, h) (4 stack words): 02168878(rec, id, count, a, b, c, h).
fn 021cbed8 ret=u32 inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02168878 force ret=u32 args=v,v,v,v,v,v,v

# bsub_iv(id): id < 100 -> 3, < 120 -> 6, < 140 -> 9, < 160 -> 12, < 180 -> 15, < 200 -> 18, < 220 -> 21, else 31.
fn 021cbefc ret=u32 inputs=200
  nowork
  arg u32 alt oneof 99 100 119 120 139 140 159 160 179 180 199 200 219 220 || range 0 0xFF || any

# bsub_build_opponents(W, rec, id, n, p5, p6) (2 stack words): T = 021688BC(rec, id, p6) (trainer data); iv = 021CBEFC(id); for i < 2:
# 021684E8(W + 0x30 + 0x3C i, p5[2 + i] (u16 at +4), p5[0], p5 word 8 + 4i, iv, i, n, p6); Heap_Free(T).
fn 021cbf38 ret=none inputs=200
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  arg u32 any
  stub 021684e8 force ret=none args=v,v,v,v,v,v,v,v
  stub 021688bc force ret=u32 args=v,v,v
  stub 021cbefc ret=u32 args=v
  stub 020307b0 force ret=none args=v

# bsub_rand(): (u16)(020056EC(-1) / 0xFFFF) (_u32_div_f).
fn 021cbf98 ret=u16 inputs=200
  nowork
  stub 020056ec force ret=u32 args=v
